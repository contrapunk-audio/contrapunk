/**
 * Guitar Audio Capture — Web Audio API pitch-to-MIDI bridge
 *
 * Opens a microphone/audio input via getUserMedia, runs per-block
 * pitch detection with a 4-state note machine matching the Rust
 * GuitarInput pipeline's noise rejection techniques:
 *
 * 1. Noise gate (RMS threshold)
 * 2. Onset detection (amplitude slope — RMS must jump 2× previous frame)
 * 3. Note state machine (Idle→Attack→Sustain→Decay) with 3-frame pitch confirmation
 * 4. Frequency suppression (±1 semitone of previous note for 100ms after onset)
 * 5. Adaptive threshold (20-frame running mean for self-calibrating noise floor)
 * 6. Cooldown (100ms lockout after each NoteOn)
 */

import { detectPitch, frequencyToMidi, midiToNoteName } from './pitchDetector';

/** Callbacks for guitar capture events. */
export interface GuitarCaptureCallbacks {
	onNoteOn(note: number, velocity: number): void;
	onNoteOff(note: number): void;
	onDetection?(info: {
		frequency: number | null;
		clarity: number;
		noteName: string;
		midi: number;
		cents: number;
		rms: number;
	}): void;
}

/** Valid guitar MIDI range: E2 (28) through C7 (96). */
const GUITAR_MIDI_MIN = 28;
const GUITAR_MIDI_MAX = 96;

const DEFAULT_BUFFER_SIZE = 2048;

// ── Note State Machine ────────────────────────────────────────────

const enum NoteState {
	Idle,    // Waiting for onset
	Attack,  // Onset detected, confirming pitch (3 frames)
	Sustain, // Note confirmed, tracking
	Decay,   // Signal dropping, waiting for silence or re-attack
}

/** Number of consecutive frames with same pitch to confirm a note. */
const PITCH_CONFIRM_FRAMES = 3;
/** Attack timeout in frames. Must be > PITCH_CONFIRM_FRAMES to allow confirmation.
 *  5 frames ≈ 215ms at 2048/48kHz — gives time for attack transient to stabilize. */
const ATTACK_TIMEOUT_FRAMES = 5;
/** Decay timeout in frames (~500ms). */
const DECAY_TIMEOUT_FRAMES = 12;
/** Cooldown after NoteOn in frames (~100ms). */
const COOLDOWN_FRAMES = 3;
/** Frequency suppression duration in frames (~100ms). */
const SUPPRESSION_FRAMES = 3;
/** Running mean window for adaptive threshold. */
const ADAPTIVE_WINDOW = 20;

export class GuitarAudioCapture {
	private audioContext: AudioContext | null = null;
	private mediaStream: MediaStream | null = null;
	private sourceNode: MediaStreamAudioSourceNode | null = null;
	private processorNode: ScriptProcessorNode | null = null;
	private callbacks: GuitarCaptureCallbacks | null = null;

	// Note state machine
	private state: NoteState = NoteState.Idle;
	private currentNote: number | null = null;
	private stateFrameCount = 0;
	private pitchVoteBuffer: number[] = [];
	private cooldownRemaining = 0;
	private prevRms = 0;

	// Frequency suppression
	private suppressedNote: number | null = null;
	private suppressionRemaining = 0;

	// Adaptive threshold
	private rmsHistory: number[] = [];

	private _isRunning = false;
	private _actualChannel = 0;

	get isRunning(): boolean { return this._isRunning; }
	get actualChannel(): number { return this._actualChannel; }

	/** Live gate thresholds — updated from the UI while capture is running. */
	noiseGateThreshold = 0.01;
	noiseGateEnabled = true;
	clarityGateEnabled = false;
	clarityThreshold = 0.7;

	async start(
		deviceId: string,
		channelIndex: number,
		callbacks: GuitarCaptureCallbacks
	): Promise<void> {
		if (this._isRunning) await this.stop();

		this.callbacks = callbacks;
		this.resetState();

		const constraints: MediaStreamConstraints = {
			audio: {
				deviceId: deviceId ? { exact: deviceId } : undefined,
				echoCancellation: false,
				noiseSuppression: false,
				autoGainControl: false,
				channelCount: { ideal: 32 }
			}
		};

		this.mediaStream = await navigator.mediaDevices.getUserMedia(constraints);
		this.audioContext = new AudioContext();
		this.sourceNode = this.audioContext.createMediaStreamSource(this.mediaStream);

		const actualChannels = this.sourceNode.channelCount;
		console.log(`[guitar] Device gave ${actualChannels} channels, want index ${channelIndex}`);

		const inputChannels = Math.max(channelIndex + 1, actualChannels);
		this.processorNode = this.audioContext.createScriptProcessor(DEFAULT_BUFFER_SIZE, inputChannels, 1);
		this.processorNode.channelCountMode = 'explicit';
		this.processorNode.channelInterpretation = 'discrete';

		const sampleRate = this.audioContext.sampleRate;
		const self = this;
		let actualChannelLogged = false;
		this._actualChannel = channelIndex;

		this.processorNode.onaudioprocess = (event: AudioProcessingEvent) => {
			if (!self._isRunning || !self.callbacks) return;

			const inputBuffer = event.inputBuffer;
			const availableChannels = inputBuffer.numberOfChannels;
			const useChannel = channelIndex < availableChannels ? channelIndex : 0;
			if (!actualChannelLogged) {
				console.log(`[guitar] Processing: requested ch=${channelIndex}, available=${availableChannels}, using ch=${useChannel}`);
				self._actualChannel = useChannel;
				actualChannelLogged = true;
			}
			const channelData = inputBuffer.getChannelData(useChannel);

			// ── Compute RMS ──────────────────────────────────────
			let rmsSum = 0;
			for (let i = 0; i < channelData.length; i++) {
				rmsSum += channelData[i] * channelData[i];
			}
			const rms = Math.sqrt(rmsSum / channelData.length);

			// ── [5] Adaptive threshold — update running mean ─────
			self.rmsHistory.push(rms);
			if (self.rmsHistory.length > ADAPTIVE_WINDOW) self.rmsHistory.shift();
			const adaptiveFloor = self.rmsHistory.reduce((a, b) => a + b, 0) / self.rmsHistory.length;

			// ── [1] Noise gate ───────────────────────────────────
			const effectiveThreshold = self.noiseGateEnabled
				? Math.max(self.noiseGateThreshold, adaptiveFloor * 1.5)
				: 0;

			if (rms < effectiveThreshold) {
				self.handleSilence(rms);
				self.prevRms = rms;
				return;
			}

			// ── [2] Onset detection (amplitude slope) ────────────
			// Primary: amplitude jump (pluck attack)
			// Secondary: signal above threshold with valid pitch (for sustained notes)
			const isOnset = (rms > self.prevRms * 1.5 && rms > effectiveThreshold)
				|| (rms > effectiveThreshold * 1.5);

			// ── Pitch detection ──────────────────────────────────
			const result = detectPitch(channelData, sampleRate);
			let midi = -1;
			let noteName = '-';
			let clarity = 0;

			if (result) {
				const midiInfo = frequencyToMidi(result.frequency);
				midi = midiInfo.note;
				noteName = midiToNoteName(midi);
				clarity = result.clarity;

				if (self.callbacks.onDetection) {
					self.callbacks.onDetection({
						frequency: result.frequency, clarity, noteName,
						midi, cents: midiInfo.cents, rms
					});
				}
			} else {
				if (self.callbacks.onDetection) {
					self.callbacks.onDetection({
						frequency: null, clarity: 0, noteName: '-', midi: 0, cents: 0, rms
					});
				}
			}

			// ── [6] Cooldown ─────────────────────────────────────
			if (self.cooldownRemaining > 0) {
				self.cooldownRemaining--;
				self.prevRms = rms;
				return; // onDetection already fired above
			}

			// ── [4] Frequency suppression ────────────────────────
			if (self.suppressionRemaining > 0) {
				self.suppressionRemaining--;
				if (self.suppressedNote !== null && midi >= 0) {
					const dist = Math.abs(midi - self.suppressedNote);
					if (dist <= 1) {
						self.prevRms = rms;
						return; // onDetection already fired above
					}
				}
			}

			// ── [3] Note state machine ───────────────────────────
			const validMidi = midi >= GUITAR_MIDI_MIN && midi <= GUITAR_MIDI_MAX;

			switch (self.state) {
				case NoteState.Idle:
					// Enter Attack if onset detected OR signal is clearly above noise with valid pitch
					if (validMidi && (isOnset || rms > effectiveThreshold)) {
						self.state = NoteState.Attack;
						self.stateFrameCount = 0;
						self.pitchVoteBuffer = [midi];
					}
					break;

				case NoteState.Attack:
					self.stateFrameCount++;
					if (validMidi) {
						self.pitchVoteBuffer.push(midi);
					}

					// Check if pitch is stable (3 consecutive same-pitch frames)
					if (self.pitchVoteBuffer.length >= PITCH_CONFIRM_FRAMES) {
						const last3 = self.pitchVoteBuffer.slice(-PITCH_CONFIRM_FRAMES);
						const allSame = last3.every(n => n === last3[0]);

						if (allSame) {
							// Pitch confirmed — emit NoteOn
							const confirmedNote = last3[0];
							const previousNote = self.currentNote; // capture BEFORE overwrite
							if (previousNote !== null && previousNote !== confirmedNote) {
								self.callbacks.onNoteOff(previousNote);
							}
							self.currentNote = confirmedNote;
							const velocity = Math.max(1, Math.min(127, Math.round(rms * 800)));
							self.callbacks.onNoteOn(confirmedNote, velocity);
							self.state = NoteState.Sustain;
							self.stateFrameCount = 0;
							self.cooldownRemaining = COOLDOWN_FRAMES;

							// Suppress the OLD note's frequency (±1 semitone)
							// to prevent sympathetic ringing from retriggering
							if (previousNote !== null && previousNote !== confirmedNote) {
								self.suppressedNote = previousNote;
								self.suppressionRemaining = SUPPRESSION_FRAMES;
							}
							break;
						}
					}

					// Attack timeout — false trigger, go back to Idle
					if (self.stateFrameCount >= ATTACK_TIMEOUT_FRAMES) {
						self.state = NoteState.Idle;
						self.pitchVoteBuffer = [];
					}
					break;

				case NoteState.Sustain:
					self.stateFrameCount++;

					// Check for re-attack (new pluck on a different note)
					if (isOnset && validMidi && midi !== self.currentNote) {
						self.state = NoteState.Attack;
						self.stateFrameCount = 0;
						self.pitchVoteBuffer = [midi];
						break;
					}

					// Transition to Decay when signal drops
					const sustainThreshold = effectiveThreshold * 0.3;
					if (rms < sustainThreshold) {
						self.state = NoteState.Decay;
						self.stateFrameCount = 0;
					}
					break;

				case NoteState.Decay:
					self.stateFrameCount++;

					// Recovery — signal came back
					if (rms > effectiveThreshold && validMidi && midi === self.currentNote) {
						self.state = NoteState.Sustain;
						self.stateFrameCount = 0;
						break;
					}

					// New onset — new note
					if (isOnset && validMidi) {
						self.state = NoteState.Attack;
						self.stateFrameCount = 0;
						self.pitchVoteBuffer = [midi];
						break;
					}

					// Decay timeout — emit NoteOff
					if (self.stateFrameCount >= DECAY_TIMEOUT_FRAMES || rms < adaptiveFloor * 2.0) {
						if (self.currentNote !== null) {
							self.callbacks.onNoteOff(self.currentNote);
							self.currentNote = null;
						}
						self.state = NoteState.Idle;
					}
					break;
			}

			self.prevRms = rms;
		};

		this.sourceNode.connect(this.processorNode);
		this.processorNode.connect(this.audioContext.destination);
		this._isRunning = true;
	}

	/** Handle silence (below noise gate). */
	private handleSilence(rms: number) {
		if (this.callbacks?.onDetection) {
			this.callbacks.onDetection({
				frequency: null, clarity: 0, noteName: '-', midi: 0, cents: 0, rms
			});
		}

		if (this.state === NoteState.Sustain || this.state === NoteState.Decay) {
			if (this.state === NoteState.Sustain) {
				// Reset frame count when first entering Decay from Sustain
				this.stateFrameCount = 0;
			}
			this.state = NoteState.Decay;
			this.stateFrameCount++;
			if (this.stateFrameCount >= DECAY_TIMEOUT_FRAMES) {
				if (this.currentNote !== null && this.callbacks) {
					this.callbacks.onNoteOff(this.currentNote);
					this.currentNote = null;
				}
				this.state = NoteState.Idle;
			}
		} else if (this.state === NoteState.Attack) {
			// Attack interrupted by silence — false trigger
			this.state = NoteState.Idle;
			this.pitchVoteBuffer = [];
		}
	}

	private resetState() {
		this.state = NoteState.Idle;
		this.currentNote = null;
		this.stateFrameCount = 0;
		this.pitchVoteBuffer = [];
		this.cooldownRemaining = 0;
		this.suppressedNote = null;
		this.suppressionRemaining = 0;
		this.prevRms = 0;
		this.rmsHistory = [];
	}

	async stop(): Promise<void> {
		this._isRunning = false;

		if (this.currentNote !== null && this.callbacks) {
			this.callbacks.onNoteOff(this.currentNote);
			this.currentNote = null;
		}

		if (this.processorNode) {
			this.processorNode.onaudioprocess = null;
			this.processorNode.disconnect();
			this.processorNode = null;
		}
		if (this.sourceNode) {
			this.sourceNode.disconnect();
			this.sourceNode = null;
		}
		if (this.audioContext) {
			try { await this.audioContext.close(); } catch {}
			this.audioContext = null;
		}
		if (this.mediaStream) {
			this.mediaStream.getTracks().forEach((t) => t.stop());
			this.mediaStream = null;
		}

		this.callbacks = null;
		this.resetState();
	}

	async measureNoiseFloor(deviceId: string, durationMs = 3000, channelIndex = 0): Promise<number> {
		const stream = await navigator.mediaDevices.getUserMedia({
			audio: {
				deviceId: deviceId ? { exact: deviceId } : undefined,
				echoCancellation: false,
				noiseSuppression: false,
				autoGainControl: false,
				channelCount: { ideal: 32 }
			}
		});

		const ctx = new AudioContext();
		const source = ctx.createMediaStreamSource(stream);
		const inputChannels = Math.max(channelIndex + 1, source.channelCount);
		const processor = ctx.createScriptProcessor(DEFAULT_BUFFER_SIZE, inputChannels, 1);
		processor.channelCountMode = 'explicit';
		processor.channelInterpretation = 'discrete';

		let totalRms = 0;
		let frameCount = 0;

		return new Promise<number>((resolve) => {
			processor.onaudioprocess = (event: AudioProcessingEvent) => {
				const ch = Math.min(channelIndex, event.inputBuffer.numberOfChannels - 1);
				const data = event.inputBuffer.getChannelData(ch);
				let sum = 0;
				for (let i = 0; i < data.length; i++) {
					sum += data[i] * data[i];
				}
				totalRms += Math.sqrt(sum / data.length);
				frameCount++;
			};

			source.connect(processor);
			processor.connect(ctx.destination);

			setTimeout(() => {
				processor.onaudioprocess = null;
				processor.disconnect();
				source.disconnect();
				ctx.close().catch(() => {});
				stream.getTracks().forEach((t) => t.stop());
				resolve(frameCount > 0 ? totalRms / frameCount : 0);
			}, durationMs);
		});
	}
}
