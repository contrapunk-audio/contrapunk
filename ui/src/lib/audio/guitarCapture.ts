/**
 * Guitar Audio Capture — Web Audio API → WASM DSP pipeline
 *
 * Captures audio via getUserMedia, feeds Float32Array blocks to the
 * WASM-compiled GuitarInput (full Rust DSP pipeline), and emits MIDI events.
 */

// Dynamic import to avoid loading uninitialized WASM module
let WasmGuitarInputClass: any = null;
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
	onCC?(channel: number, controller: number, value: number): void;
	onChannelPressure?(channel: number, pressure: number): void;
	onVibratoStatus?(active: boolean, rateHz: number, depthCents: number): void;
	onDetection?(info: { noteName: string; midi: number; rms: number }): void;
}

const BUFFER_SIZE = 1024;

export class GuitarAudioCapture {
	private context: AudioContext | null = null;
	private stream: MediaStream | null = null;
	private processor: ScriptProcessorNode | null = null;
	private dsp: WasmGuitarInput | null = null;

	async start(
		deviceId: string | null,
		channelIndex: number,
		callbacks: GuitarCaptureCallbacks
	) {
		const constraints: MediaStreamConstraints = {
			audio: {
				...(deviceId ? { deviceId: { exact: deviceId } } : {}),
				echoCancellation: false,
				noiseSuppression: false,
				autoGainControl: false,
			},
		};

		this.stream = await navigator.mediaDevices.getUserMedia(constraints);
		this.context = new AudioContext({ sampleRate: 48000 });
		const source = this.context.createMediaStreamSource(this.stream);
		const numChannels = source.channelCount;

		const WasmGuitarInput = await getWasmGuitarInput();
		this.dsp = new WasmGuitarInput(this.context.sampleRate, BUFFER_SIZE);
		// Match guitar_input_demo defaults
		this.dsp.set_onset_threshold(0.015);
		this.dsp.set_string_confidence(0.4);

		this.processor = this.context.createScriptProcessor(
			BUFFER_SIZE,
			numChannels,
			1
		);

		let frameCount = 0;
		this.processor.onaudioprocess = (event) => {
			const input = event.inputBuffer;
			const ch = Math.min(channelIndex, input.numberOfChannels - 1);
			const samples = input.getChannelData(ch);

			// Debug: log RMS every 50 frames (~1s) to verify audio is flowing
			frameCount++;
			if (frameCount % 50 === 0) {
				let sum = 0;
				for (let i = 0; i < samples.length; i++) sum += samples[i] * samples[i];
				const rms = Math.sqrt(sum / samples.length);
				console.log(`[guitar] frame=${frameCount} rms=${rms.toFixed(4)} ch=${ch}/${input.numberOfChannels}`);
			}

			const eventsJson = this.dsp!.process_block(samples);
			let events: any[];
			try {
				events = JSON.parse(eventsJson);
			} catch {
				return;
			}

			for (const e of events) {
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
		};

		source.connect(this.processor);
		this.processor.connect(this.context.destination);
	}

	stop() {
		this.processor?.disconnect();
		this.stream?.getTracks().forEach((t) => t.stop());
		this.context?.close();
		this.dsp?.free();
		this.processor = null;
		this.stream = null;
		this.context = null;
		this.dsp = null;
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

	async measureNoiseFloor(deviceId: string | null, durationMs = 3000): Promise<number> {
		const constraints: MediaStreamConstraints = {
			audio: {
				...(deviceId ? { deviceId: { exact: deviceId } } : {}),
				echoCancellation: false,
				noiseSuppression: false,
				autoGainControl: false,
			},
		};
		const stream = await navigator.mediaDevices.getUserMedia(constraints);
		const ctx = new AudioContext({ sampleRate: 48000 });
		const src = ctx.createMediaStreamSource(stream);
		const proc = ctx.createScriptProcessor(2048, 1, 1);
		const rmsValues: number[] = [];

		return new Promise((resolve) => {
			proc.onaudioprocess = (e) => {
				const buf = e.inputBuffer.getChannelData(0);
				let sum = 0;
				for (let i = 0; i < buf.length; i++) sum += buf[i] * buf[i];
				rmsValues.push(Math.sqrt(sum / buf.length));
			};
			src.connect(proc);
			proc.connect(ctx.destination);

			setTimeout(() => {
				proc.disconnect();
				stream.getTracks().forEach((t) => t.stop());
				ctx.close();
				const avg = rmsValues.reduce((a, b) => a + b, 0) / Math.max(rmsValues.length, 1);
				resolve(avg);
			}, durationMs);
		});
	}
}
