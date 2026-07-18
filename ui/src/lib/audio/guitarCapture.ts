/**
 * Guitar Audio Capture — JS audio capture + Rust WASM DSP
 *
 * Audio path:
 *   JS: getUserMedia → AudioWorkletNode → Float32Array (via MessagePort)
 *   WASM: WasmGuitarInput.process_block(samples) → JSON events
 *   JS: parse events → NoteOn/Off/PitchBend callbacks
 *
 * Falls back to ScriptProcessorNode on browsers without AudioWorklet.
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

function bufferSizeForLatency(latencyMs: number, sampleRate: number): number {
	const samples = Math.max(1, latencyMs) * sampleRate / 1000;
	return Math.min(4096, Math.max(256, 2 ** Math.ceil(Math.log2(samples))));
}

export function serializeGuitarCaptureOperation(
	after: Promise<void>,
	operation: () => Promise<void>
): { result: Promise<void>; tail: Promise<void> } {
	const result = after.then(operation);
	return { result, tail: result.catch(() => {}) };
}

/**
 * Process a buffer of samples through the WASM DSP and fire callbacks.
 * Shared by both AudioWorklet and ScriptProcessor paths.
 */
function processAudioSamples(
	samples: Float32Array,
	// eslint-disable-next-line @typescript-eslint/no-explicit-any
	dsp: any,
	callbacks: GuitarCaptureCallbacks,
	frameCounter: { count: number },
	channelUsed: number,
): void {
	// Compute RMS for UI signal graph
	let rmsSum = 0;
	for (let i = 0; i < samples.length; i++) rmsSum += samples[i] * samples[i];
	const rms = Math.sqrt(rmsSum / samples.length);

	// No JS-side noise gate — the Rust pipeline handles gating
	// internally (matching the guitar_input_demo which sends ALL
	// audio to process_block without pre-filtering)

	frameCounter.count++;

	// Send raw audio directly to WASM — the Rust ring buffer handles windowing.
	const eventsJson = dsp.process_block(samples);
	// eslint-disable-next-line @typescript-eslint/no-explicit-any
	let allEvents: any[];
	try {
		allEvents = JSON.parse(eventsJson);
	} catch (err) {
		console.error('[guitar] Failed to parse WASM events:', err);
		return;
	}

	// Fire detection callback for UI
	if (callbacks.onDetection) {
		// eslint-disable-next-line @typescript-eslint/no-explicit-any
		const lastNoteOn = allEvents.findLast?.((e: any) => e.type === 'note_on');
		if (lastNoteOn) {
			callbacks.onDetection({
				frequency: null,
				clarity: 1.0,
				noteName: midiToNoteName(lastNoteOn.note),
				midi: lastNoteOn.note,
				cents: 0,
				rms
			});
		} else {
			callbacks.onDetection({
				frequency: null, clarity: 0, noteName: '-', midi: 0, cents: 0, rms
			});
		}
	}

	// Dispatch MIDI events to callbacks
	for (const e of allEvents) {
		switch (e.type) {
			case 'note_on':
				callbacks.onNoteOn(e.note, e.velocity);
				break;
			case 'note_off':
				callbacks.onNoteOff(e.note);
				break;
			case 'pitch_bend':
				callbacks.onPitchBend?.(e.channel, e.cents);
				break;
			case 'midi_pitch_bend':
				callbacks.onMidiPitchBend?.(e.channel, e.value);
				break;
			case 'cc':
				callbacks.onCC?.(e.channel, e.controller, e.value);
				break;
			case 'channel_pressure':
				callbacks.onChannelPressure?.(e.channel, e.pressure);
				break;
			case 'vibrato':
				callbacks.onVibratoStatus?.(e.active, e.rate_hz, e.depth_cents);
				break;
		}
	}
}

type GuitarDspConfig = {
	bends?: boolean;
	legato?: boolean;
	slides?: boolean;
	vibrato?: boolean;
	gain?: number;
	onsetThreshold?: number;
	stringConfidence?: number;
};

export class GuitarAudioCapture {
	private audioContext: AudioContext | null = null;
	private mediaStream: MediaStream | null = null;
	private sourceNode: MediaStreamAudioSourceNode | null = null;
	private workletNode: AudioWorkletNode | null = null;
	private processorNode: ScriptProcessorNode | null = null; // fallback
	private callbacks: GuitarCaptureCallbacks | null = null;
	// eslint-disable-next-line @typescript-eslint/no-explicit-any
	private dsp: any = null; // WasmGuitarInput instance

	private _isRunning = false;
	private _actualChannel = 0;
	private _frameCount = 0;

	// Overlap buffer — 75% overlap means each hop (window/4) triggers analysis
	private overlapBuffer: Float32Array | null = null;
	private overlapWritePos = 0;
	private hopSize = 0;
	private windowSize = 0;

	// Accumulation buffer for AudioWorklet (128-sample blocks → bufferSize blocks)
	private accumBuffer: Float32Array | null = null;
	private accumWritePos = 0;
	private config: GuitarDspConfig = {};

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
		latencyMs = 21
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
		const bufferSize = bufferSizeForLatency(latencyMs, sampleRate);
		console.log(`[guitar] Device: ${actualChannels}ch @ ${sampleRate}Hz, want ch${channelIndex}, buffer=${bufferSize}`);

		// Size the analysis window only after the browser's real sample rate
		// is known; 44.1 kHz devices must not inherit a 48 kHz conversion.
		this.windowSize = bufferSize;
		this.hopSize = Math.floor(bufferSize / 4);
		this.overlapBuffer = new Float32Array(bufferSize);
		this.overlapWritePos = 0;

		this.dsp = new WasmGuitarInput(sampleRate, bufferSize);
		this.dsp.set_onset_threshold(0.015);
		this.dsp.set_string_confidence(0.4);
		this.setConfig(this.config);
		console.log(`[guitar] Overlap: window=${this.windowSize} hop=${this.hopSize} (75% overlap)`);

		const inputChannels = Math.max(channelIndex + 1, actualChannels);

		// Prefer AudioWorkletNode; fall back to ScriptProcessorNode
		if (this.audioContext.audioWorklet) {
			await this._startWithWorklet(channelIndex, inputChannels, bufferSize);
		} else {
			console.warn('[guitar] AudioWorklet not supported — falling back to ScriptProcessorNode');
			this._startWithScriptProcessor(channelIndex, inputChannels, bufferSize);
		}

		this._isRunning = true;
	}

	/**
	 * AudioWorkletNode path — preferred.
	 * The worklet posts 128-sample chunks via MessagePort; we accumulate
	 * them into bufferSize blocks before feeding WASM.
	 */
	private async _startWithWorklet(
		channelIndex: number,
		inputChannels: number,
		bufferSize: number,
	): Promise<void> {
		const ctx = this.audioContext!;
		await ctx.audioWorklet.addModule('/audio-capture-processor.js');

		this.workletNode = new AudioWorkletNode(ctx, 'audio-capture-processor', {
			numberOfInputs: 1,
			numberOfOutputs: 0, // we don't need audio output from the worklet
			channelCount: inputChannels,
			channelCountMode: 'explicit',
			channelInterpretation: 'discrete',
			processorOptions: { channelIndex },
		});

		// Accumulation buffer: worklet fires every 128 samples; WASM expects bufferSize
		this.accumBuffer = new Float32Array(bufferSize);
		this.accumWritePos = 0;

		const self = this;
		let actualChannelLogged = false;
		this._actualChannel = channelIndex;
		const frameCounter = { count: 0 };

		this.workletNode.port.onmessage = (event: MessageEvent) => {
			if (!self._isRunning || !self.callbacks || !self.dsp || !self.accumBuffer) return;

			const chunk: Float32Array = event.data.samples;
			const usedChannel: number = event.data.channel;

			if (!actualChannelLogged) {
				console.log(`[guitar] Worklet processing: requested ch=${channelIndex}, using ch=${usedChannel}`);
				self._actualChannel = usedChannel;
				actualChannelLogged = true;
			}

			// Accumulate 128-sample worklet chunks into bufferSize blocks
			let srcOffset = 0;
			while (srcOffset < chunk.length) {
				const remaining = bufferSize - self.accumWritePos;
				const toCopy = Math.min(remaining, chunk.length - srcOffset);
				self.accumBuffer!.set(chunk.subarray(srcOffset, srcOffset + toCopy), self.accumWritePos);
				self.accumWritePos += toCopy;
				srcOffset += toCopy;

				if (self.accumWritePos >= bufferSize) {
					// Full buffer ready — send to WASM DSP
					processAudioSamples(
						self.accumBuffer!,
						self.dsp,
						self.callbacks!,
						frameCounter,
						usedChannel,
					);
					self._frameCount = frameCounter.count;
					self.accumWritePos = 0;
				}
			}
		};

		this.sourceNode!.connect(this.workletNode);
	}

	/**
	 * ScriptProcessorNode path — fallback for browsers without AudioWorklet.
	 */
	private _startWithScriptProcessor(
		channelIndex: number,
		inputChannels: number,
		bufferSize: number,
	): void {
		const ctx = this.audioContext!;
		this.processorNode = ctx.createScriptProcessor(bufferSize, inputChannels, 1);
		this.processorNode.channelCountMode = 'explicit';
		this.processorNode.channelInterpretation = 'discrete';

		const self = this;
		let actualChannelLogged = false;
		this._actualChannel = channelIndex;
		const frameCounter = { count: 0 };

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
			processAudioSamples(samples, self.dsp, self.callbacks!, frameCounter, useChannel);
			self._frameCount = frameCounter.count;
		};

		this.sourceNode!.connect(this.processorNode);
		this.processorNode.connect(ctx.destination);
	}

	async stop(): Promise<void> {
		this._isRunning = false;

		if (this.workletNode) {
			this.workletNode.port.onmessage = null;
			this.workletNode.disconnect();
			this.workletNode = null;
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
		if (this.dsp) {
			try { this.dsp.free(); } catch {}
			this.dsp = null;
		}

		this.callbacks = null;
		this.accumBuffer = null;
		this.accumWritePos = 0;
	}

	setConfig(opts: GuitarDspConfig) {
		this.config = { ...this.config, ...opts };
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

		let totalRms = 0;
		let frameCount = 0;

		const computeRms = (data: Float32Array) => {
			let sum = 0;
			for (let i = 0; i < data.length; i++) sum += data[i] * data[i];
			totalRms += Math.sqrt(sum / data.length);
			frameCount++;
		};

		if (ctx.audioWorklet) {
			// AudioWorklet path
			await ctx.audioWorklet.addModule('/audio-capture-processor.js');
			const worklet = new AudioWorkletNode(ctx, 'audio-capture-processor', {
				numberOfInputs: 1,
				numberOfOutputs: 0,
				channelCount: inputChannels,
				channelCountMode: 'explicit',
				channelInterpretation: 'discrete',
				processorOptions: { channelIndex },
			});
			worklet.port.onmessage = (event: MessageEvent) => {
				computeRms(event.data.samples);
			};
			source.connect(worklet);

			return new Promise<number>((resolve) => {
				setTimeout(() => {
					worklet.port.onmessage = null;
					worklet.disconnect();
					source.disconnect();
					ctx.close().catch(() => {});
					stream.getTracks().forEach((t) => t.stop());
					resolve(frameCount > 0 ? totalRms / frameCount : 0);
				}, durationMs);
			});
		} else {
			// ScriptProcessorNode fallback
			const processor = ctx.createScriptProcessor(DEFAULT_BUFFER_SIZE, inputChannels, 1);
			processor.channelCountMode = 'explicit';
			processor.channelInterpretation = 'discrete';

			return new Promise<number>((resolve) => {
				processor.onaudioprocess = (event: AudioProcessingEvent) => {
					const ch = Math.min(channelIndex, event.inputBuffer.numberOfChannels - 1);
					computeRms(event.inputBuffer.getChannelData(ch));
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
}

// Minimal helper — just for the onDetection display
function midiToNoteName(midi: number): string {
	const names = ['C', 'C#', 'D', 'Eb', 'E', 'F', 'F#', 'G', 'Ab', 'A', 'Bb', 'B'];
	const noteIndex = ((midi % 12) + 12) % 12;
	const octave = Math.floor(midi / 12) - 1;
	return `${names[noteIndex]}${octave}`;
}
