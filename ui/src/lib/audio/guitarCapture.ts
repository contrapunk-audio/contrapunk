/**
 * Guitar Audio Capture — JS audio capture + Rust WASM DSP
 *
 * Audio path:
 *   JS: getUserMedia → ScriptProcessor → Float32Array
 *   WASM: WasmGuitarInput.process_block(samples) → JSON events
 *   JS: parse events → NoteOn/Off/PitchBend callbacks
 *
 * All onset detection, pitch detection, note state machine, and
 * expression output runs in the Rust GuitarInput pipeline compiled
 * to WASM — identical to the guitar_input_demo.
 */

// Dynamic import to avoid loading uninitialized WASM module
// eslint-disable-next-line @typescript-eslint/no-explicit-any
let WasmGuitarInputClass: any = null;
// eslint-disable-next-line @typescript-eslint/no-explicit-any
async function getWasmGuitarInput(): Promise<any> {
	if (!WasmGuitarInputClass) {
		const mod = await import('$lib/wasm-pkg');
		WasmGuitarInputClass = mod.WasmGuitarInput;
	}
	return WasmGuitarInputClass;
}

export interface GuitarCaptureCallbacks {
	onNoteOn(note: number, velocity: number): void;
	onNoteOff(note: number): void;
	onPitchBend?(channel: number, cents: number): void;
	onMidiPitchBend?(channel: number, value: number): void;
	onCC?(channel: number, controller: number, value: number): void;
	onChannelPressure?(channel: number, pressure: number): void;
	onVibratoStatus?(active: boolean, rateHz: number, depthCents: number): void;
	/** Fired every frame with signal info for UI graphs. */
	onDetection?(info: {
		frequency: number | null;
		clarity: number;
		noteName: string;
		midi: number;
		cents: number;
		rms: number;
	}): void;
}

const DEFAULT_BUFFER_SIZE = 1024; // Match demo's default

export class GuitarAudioCapture {
	private audioContext: AudioContext | null = null;
	private mediaStream: MediaStream | null = null;
	private sourceNode: MediaStreamAudioSourceNode | null = null;
	private processorNode: ScriptProcessorNode | null = null;
	private callbacks: GuitarCaptureCallbacks | null = null;
	// eslint-disable-next-line @typescript-eslint/no-explicit-any
	private dsp: any = null; // WasmGuitarInput instance

	private _isRunning = false;
	private _actualChannel = 0;

	get isRunning(): boolean { return this._isRunning; }
	get actualChannel(): number { return this._actualChannel; }

	/** Live noise gate threshold — updated from UI. */
	noiseGateThreshold = 0.01;
	noiseGateEnabled = true;
	// Unused but kept for interface compat
	clarityGateEnabled = false;
	clarityThreshold = 0.7;

	async start(
		deviceId: string,
		channelIndex: number,
		callbacks: GuitarCaptureCallbacks,
		bufferSize: number = DEFAULT_BUFFER_SIZE
	): Promise<void> {
		if (this._isRunning) await this.stop();

		this.callbacks = callbacks;

		// Initialize WASM DSP pipeline
		const WasmGuitarInput = await getWasmGuitarInput();
		// Will be created after we know the sample rate

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
		const sampleRate = this.audioContext.sampleRate;
		console.log(`[guitar] Device: ${actualChannels}ch @ ${sampleRate}Hz, want ch${channelIndex}, buffer=${bufferSize}`);

		// Create WASM DSP with actual sample rate
		this.dsp = new WasmGuitarInput(sampleRate, bufferSize);
		this.dsp.set_onset_threshold(0.015);
		this.dsp.set_string_confidence(0.4);

		const inputChannels = Math.max(channelIndex + 1, actualChannels);
		this.processorNode = this.audioContext.createScriptProcessor(bufferSize, inputChannels, 1);
		this.processorNode.channelCountMode = 'explicit';
		this.processorNode.channelInterpretation = 'discrete';

		const self = this;
		let actualChannelLogged = false;
		this._actualChannel = channelIndex;

		this.processorNode.onaudioprocess = (event: AudioProcessingEvent) => {
			if (!self._isRunning || !self.callbacks || !self.dsp) return;

			const inputBuffer = event.inputBuffer;
			const availableChannels = inputBuffer.numberOfChannels;
			const useChannel = channelIndex < availableChannels ? channelIndex : 0;
			if (!actualChannelLogged) {
				console.log(`[guitar] Processing: requested ch=${channelIndex}, available=${availableChannels}, using ch=${useChannel}`);
				self._actualChannel = useChannel;
				actualChannelLogged = true;
			}

			const samples = inputBuffer.getChannelData(useChannel);

			// Compute RMS for UI signal graph
			let rmsSum = 0;
			for (let i = 0; i < samples.length; i++) rmsSum += samples[i] * samples[i];
			const rms = Math.sqrt(rmsSum / samples.length);

			// Noise gate (JS-side, before sending to WASM for efficiency)
			if (self.noiseGateEnabled && rms < self.noiseGateThreshold) {
				if (self.callbacks.onDetection) {
					self.callbacks.onDetection({
						frequency: null, clarity: 0, noteName: '-', midi: 0, cents: 0, rms
					});
				}
				return;
			}

			// Send audio to Rust WASM DSP pipeline
			const eventsJson = self.dsp.process_block(samples);
			let events: any[];
			try {
				events = JSON.parse(eventsJson);
			} catch {
				return;
			}

			// Fire detection callback for UI (rms every frame)
			if (self.callbacks.onDetection) {
				// Find the most recent NoteOn for display
				const lastNoteOn = events.findLast?.((e: any) => e.type === 'note_on');
				if (lastNoteOn) {
					self.callbacks.onDetection({
						frequency: null, // WASM doesn't expose raw freq
						clarity: 1.0, // Rust pipeline confirmed it
						noteName: midiToNoteName(lastNoteOn.note),
						midi: lastNoteOn.note,
						cents: 0,
						rms
					});
				} else {
					self.callbacks.onDetection({
						frequency: null, clarity: 0, noteName: '-', midi: 0, cents: 0, rms
					});
				}
			}

			// Dispatch MIDI events to callbacks
			for (const e of events) {
				switch (e.type) {
					case 'note_on':
						self.callbacks.onNoteOn(e.note, e.velocity);
						break;
					case 'note_off':
						self.callbacks.onNoteOff(e.note);
						break;
					case 'pitch_bend':
						self.callbacks.onPitchBend?.(e.channel, e.cents);
						break;
					case 'midi_pitch_bend':
						self.callbacks.onMidiPitchBend?.(e.channel, e.value);
						break;
					case 'cc':
						self.callbacks.onCC?.(e.channel, e.controller, e.value);
						break;
					case 'channel_pressure':
						self.callbacks.onChannelPressure?.(e.channel, e.pressure);
						break;
					case 'vibrato':
						self.callbacks.onVibratoStatus?.(e.active, e.rate_hz, e.depth_cents);
						break;
				}
			}
		};

		this.sourceNode.connect(this.processorNode);
		this.processorNode.connect(this.audioContext.destination);
		this._isRunning = true;
	}

	async stop(): Promise<void> {
		this._isRunning = false;

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
		if (this.dsp) {
			try { this.dsp.free(); } catch {}
			this.dsp = null;
		}

		this.callbacks = null;
	}

	setConfig(opts: {
		bends?: boolean;
		legato?: boolean;
		slides?: boolean;
		vibrato?: boolean;
		gain?: number;
		onsetThreshold?: number;
		stringConfidence?: number;
	}) {
		if (!this.dsp) return;
		if (opts.bends !== undefined) this.dsp.set_bends_enabled(opts.bends);
		if (opts.legato !== undefined) this.dsp.set_legato_enabled(opts.legato);
		if (opts.slides !== undefined) this.dsp.set_slides_enabled(opts.slides);
		if (opts.vibrato !== undefined) this.dsp.set_vibrato_enabled(opts.vibrato);
		if (opts.gain !== undefined) this.dsp.set_input_gain(opts.gain);
		if (opts.onsetThreshold !== undefined) this.dsp.set_onset_threshold(opts.onsetThreshold);
		if (opts.stringConfidence !== undefined) this.dsp.set_string_confidence(opts.stringConfidence);
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
				for (let i = 0; i < data.length; i++) sum += data[i] * data[i];
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

// Minimal helper — just for the onDetection display
function midiToNoteName(midi: number): string {
	const names = ['C', 'C#', 'D', 'Eb', 'E', 'F', 'F#', 'G', 'Ab', 'A', 'Bb', 'B'];
	const noteIndex = ((midi % 12) + 12) % 12;
	const octave = Math.floor(midi / 12) - 1;
	return `${names[noteIndex]}${octave}`;
}
