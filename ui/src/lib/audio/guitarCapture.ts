/**
 * Guitar Audio Capture — Web Audio API pitch-to-MIDI bridge
 *
 * Opens a microphone/audio input via getUserMedia, runs per-block
 * autocorrelation pitch detection, and emits NoteOn/NoteOff callbacks
 * compatible with the Contrapunk adapter interface.
 */

import { detectPitch, frequencyToMidi, midiToNoteName } from './pitchDetector';

/** Callbacks for guitar capture events. */
export interface GuitarCaptureCallbacks {
	/** Fired when a new note is detected. */
	onNoteOn(note: number, velocity: number): void;
	/** Fired when the current note ends (silence or new note). */
	onNoteOff(note: number): void;
	/** Fired every detection frame with debug info. */
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

/** Number of consecutive silent frames before sending NoteOff. */
const SILENCE_FRAMES_THRESHOLD = 15;

/** ScriptProcessor buffer size — 2048 samples at 44.1kHz ~ 46ms latency. */
const BUFFER_SIZE = 2048;

/**
 * Captures audio from a browser audio input device,
 * runs pitch detection per block, and generates MIDI-like events.
 */
export class GuitarAudioCapture {
	private audioContext: AudioContext | null = null;
	private mediaStream: MediaStream | null = null;
	private sourceNode: MediaStreamAudioSourceNode | null = null;
	private processorNode: ScriptProcessorNode | null = null;
	private callbacks: GuitarCaptureCallbacks | null = null;

	// Note tracking state
	private currentNote: number | null = null;
	private silenceFrameCount = 0;
	private _isRunning = false;

	get isRunning(): boolean {
		return this._isRunning;
	}

	/**
	 * Start capturing audio and detecting pitch.
	 *
	 * @param deviceId     - MediaDevices deviceId for the audio input
	 * @param channelIndex - 0-based channel index to analyze (for multi-channel interfaces)
	 * @param callbacks    - NoteOn/NoteOff/Detection callbacks
	 */
	async start(
		deviceId: string,
		channelIndex: number,
		callbacks: GuitarCaptureCallbacks
	): Promise<void> {
		if (this._isRunning) {
			await this.stop();
		}

		this.callbacks = callbacks;
		this.currentNote = null;
		this.silenceFrameCount = 0;

		// Open audio stream
		const constraints: MediaStreamConstraints = {
			audio: {
				deviceId: deviceId ? { exact: deviceId } : undefined,
				echoCancellation: false,
				noiseSuppression: false,
				autoGainControl: false,
				channelCount: { ideal: channelIndex + 1 }
			}
		};

		this.mediaStream = await navigator.mediaDevices.getUserMedia(constraints);
		this.audioContext = new AudioContext();
		this.sourceNode = this.audioContext.createMediaStreamSource(this.mediaStream);

		// Request the full channel count from the source so we can pick the
		// correct channel. Using channelCountMode='explicit' + 'discrete'
		// prevents the browser from downmixing to mono.
		const inputChannels = Math.max(channelIndex + 1, this.sourceNode.channelCount);
		this.processorNode = this.audioContext.createScriptProcessor(BUFFER_SIZE, inputChannels, 1);
		this.processorNode.channelCountMode = 'explicit';
		this.processorNode.channelInterpretation = 'discrete';

		const sampleRate = this.audioContext.sampleRate;
		const self = this;

		this.processorNode.onaudioprocess = (event: AudioProcessingEvent) => {
			if (!self._isRunning || !self.callbacks) return;

			// Get the mono channel data
			const inputBuffer = event.inputBuffer;
			const channelData =
				channelIndex < inputBuffer.numberOfChannels
					? inputBuffer.getChannelData(channelIndex)
					: inputBuffer.getChannelData(0);

			// Compute RMS for velocity estimation
			let rmsSum = 0;
			for (let i = 0; i < channelData.length; i++) {
				rmsSum += channelData[i] * channelData[i];
			}
			const rms = Math.sqrt(rmsSum / channelData.length);

			// Run pitch detection
			const result = detectPitch(channelData, sampleRate);

			if (result) {
				const midiInfo = frequencyToMidi(result.frequency);
				const midi = midiInfo.note;

				// Fire detection callback
				if (self.callbacks.onDetection) {
					self.callbacks.onDetection({
						frequency: result.frequency,
						clarity: result.clarity,
						noteName: midiToNoteName(midi),
						midi,
						cents: midiInfo.cents,
						rms
					});
				}

				// Valid guitar range check
				if (midi >= GUITAR_MIDI_MIN && midi <= GUITAR_MIDI_MAX) {
					self.silenceFrameCount = 0;

					if (self.currentNote !== midi) {
						// New note detected — send NoteOff for old, NoteOn for new
						if (self.currentNote !== null) {
							self.callbacks.onNoteOff(self.currentNote);
						}

						// Velocity from RMS (scale to MIDI 1-127)
						const velocity = Math.max(1, Math.min(127, Math.round(rms * 800)));
						self.currentNote = midi;
						self.callbacks.onNoteOn(midi, velocity);
					}
				}
			} else {
				// No pitch detected — count silence frames
				self.silenceFrameCount++;

				if (self.callbacks.onDetection) {
					self.callbacks.onDetection({
						frequency: null,
						clarity: 0,
						noteName: '-',
						midi: 0,
						cents: 0,
						rms
					});
				}

				if (
					self.currentNote !== null &&
					self.silenceFrameCount >= SILENCE_FRAMES_THRESHOLD
				) {
					self.callbacks.onNoteOff(self.currentNote);
					self.currentNote = null;
				}
			}
		};

		// Connect the graph: source -> processor -> destination (required for ScriptProcessor)
		this.sourceNode.connect(this.processorNode);
		this.processorNode.connect(this.audioContext.destination);

		this._isRunning = true;
	}

	/**
	 * Stop capturing and release all resources.
	 */
	async stop(): Promise<void> {
		this._isRunning = false;

		// Send final NoteOff if a note is still active
		if (this.currentNote !== null && this.callbacks) {
			this.callbacks.onNoteOff(this.currentNote);
			this.currentNote = null;
		}

		// Disconnect audio graph
		if (this.processorNode) {
			this.processorNode.onaudioprocess = null;
			this.processorNode.disconnect();
			this.processorNode = null;
		}

		if (this.sourceNode) {
			this.sourceNode.disconnect();
			this.sourceNode = null;
		}

		// Close audio context
		if (this.audioContext) {
			try {
				await this.audioContext.close();
			} catch {
				// Already closed
			}
			this.audioContext = null;
		}

		// Stop media stream tracks
		if (this.mediaStream) {
			this.mediaStream.getTracks().forEach((t) => t.stop());
			this.mediaStream = null;
		}

		this.callbacks = null;
		this.silenceFrameCount = 0;
	}

	/**
	 * Measure noise floor RMS over a specified duration.
	 * Used for simple browser-side calibration.
	 *
	 * @param deviceId  - Audio input device ID
	 * @param durationMs - Duration to measure in milliseconds (default 3000)
	 * @returns Average RMS over the measurement period
	 */
	async measureNoiseFloor(deviceId: string, durationMs = 3000, channelIndex = 0): Promise<number> {
		const stream = await navigator.mediaDevices.getUserMedia({
			audio: {
				deviceId: deviceId ? { exact: deviceId } : undefined,
				echoCancellation: false,
				noiseSuppression: false,
				autoGainControl: false
			}
		});

		const ctx = new AudioContext();
		const source = ctx.createMediaStreamSource(stream);
		const inputChannels = Math.max(channelIndex + 1, source.channelCount);
		const processor = ctx.createScriptProcessor(BUFFER_SIZE, inputChannels, 1);
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
