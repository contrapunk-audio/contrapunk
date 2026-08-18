import './elixir-worklet-polyfills.js';
import { ElixirAudio, initSync } from '../wasm-pkg/contrapunk_wasm.js';
import { ElixirEventQueue } from './elixir-event-queue.js';

declare const sampleRate: number;
declare const currentFrame: number;
declare abstract class AudioWorkletProcessor {
	readonly port: MessagePort;
	constructor();
	abstract process(inputs: Float32Array[][], outputs: Float32Array[][]): boolean;
}
declare function registerProcessor(
	name: string,
	processor: new () => AudioWorkletProcessor
): void;

type ElixirEngine = InstanceType<typeof ElixirAudio>;
const MAX_FRAMES = 128;
const EVENT_CAPACITY = 1024;

class ElixirAudioProcessor extends AudioWorkletProcessor {
	private engine: ElixirEngine | null = null;
	private readonly events = new ElixirEventQueue(EVENT_CAPACITY);
	private memoryBuffer: ArrayBuffer | null = null;
	private rendered: Float32Array | null = null;
	private slideVoiceIds: Uint32Array | null = null;
	private slideSlots: Uint8Array | null = null;
	private slideFrequencies: Float32Array | null = null;
	private slideTargets: Float32Array | null = null;
	private slideProgresses: Float32Array | null = null;
	private slideDurations: Float32Array | null = null;
	private slideCurves: Uint8Array | null = null;
	private readonly scratch = new ArrayBuffer(4);
	private readonly scratchFloat = new Float32Array(this.scratch);
	private readonly scratchInt = new Int32Array(this.scratch);
	private telemetry: Int32Array | null = null;

	constructor() {
		super();
		this.port.onmessage = ({ data }: MessageEvent) => {
			if (data && 'wasmBytes' in data) {
				this.initialize(data.wasmBytes);
				return;
			}
			if (
				typeof SharedArrayBuffer !== 'undefined' &&
				data?.telemetryBuffer instanceof SharedArrayBuffer
			) {
				this.telemetry = new Int32Array(data.telemetryBuffer);
				this.port.postMessage({ telemetry: true, ack: 0 });
				return;
			}
			if (!this.events.insert(data)) {
				this.engine?.panic();
				this.events.clear();
				this.port.postMessage({ ack: data.seq, panic: true });
				return;
			}
			this.port.postMessage({ ack: data.seq });
		};
	}

	private initialize(module: ArrayBuffer) {
		try {
			const wasm = initSync({ module });
			const engine = new ElixirAudio(sampleRate, MAX_FRAMES);
			this.engine = engine;
			this.memoryBuffer = wasm.memory.buffer;
			this.rendered = new Float32Array(
				this.memoryBuffer,
				engine.output_ptr(),
				engine.output_capacity()
			);
			this.slideVoiceIds = new Uint32Array(
				this.memoryBuffer,
				engine.slide_voice_ids_ptr(),
				32
			);
			this.slideSlots = new Uint8Array(
				this.memoryBuffer,
				engine.slide_slots_ptr(),
				32
			);
			this.slideFrequencies = new Float32Array(
				this.memoryBuffer,
				engine.slide_frequencies_ptr(),
				32
			);
			this.slideTargets = new Float32Array(
				this.memoryBuffer,
				engine.slide_targets_ptr(),
				32
			);
			this.slideProgresses = new Float32Array(
				this.memoryBuffer,
				engine.slide_progresses_ptr(),
				32
			);
			this.slideDurations = new Float32Array(
				this.memoryBuffer,
				engine.slide_durations_ptr(),
				32
			);
			this.slideCurves = new Uint8Array(
				this.memoryBuffer,
				engine.slide_curves_ptr(),
				32
			);
			this.port.postMessage({ ready: true, ack: 0 });
		} catch (error) {
			this.port.postMessage({ initError: String(error), ack: 0 });
		}
	}

	process(_inputs: Float32Array[][], outputs: Float32Array[][]): boolean {
		const output = outputs[0];
		if (!output?.length || !this.engine || !this.memoryBuffer) return true;
		if (this.memoryBuffer.byteLength === 0) {
			this.engine.panic();
			return false;
		}

		const frames = output[0].length;
		const blockEnd = currentFrame + frames;
		let offset = 0;
		while (this.events.length > 0 && this.events.atFrame[0] < blockEnd) {
			const eventOffset = Math.max(0, Math.floor(this.events.atFrame[0] - currentFrame));
			this.render(output, offset, eventOffset - offset);
			offset = eventOffset;
			this.applyFirstEvent();
			this.events.removeFirst();
		}
		this.render(output, offset, frames - offset);
		this.publishTelemetry();
		return true;
	}

	private render(output: Float32Array[], offset: number, frames: number) {
		if (frames <= 0 || !this.engine || !this.rendered) return;
		const channels = Math.min(output.length, 2);
		this.engine.process(frames, channels);
		for (let frame = 0; frame < frames; frame++) {
			for (let channel = 0; channel < output.length; channel++) {
				output[channel][offset + frame] =
					channel < channels ? this.rendered[frame * channels + channel] : 0;
			}
		}
	}

	private publishTelemetry() {
		if (
			!this.telemetry ||
			!this.engine ||
			!this.slideVoiceIds ||
			!this.slideSlots ||
			!this.slideFrequencies ||
			!this.slideTargets ||
			!this.slideProgresses ||
			!this.slideDurations ||
			!this.slideCurves
		) return;
		Atomics.add(this.telemetry, 0, 1);
		const count = Math.min(32, this.engine.slide_snapshot_count());
		for (let index = 0; index < count; index++) {
			const base = 2 + index * 7;
			Atomics.store(this.telemetry, base, this.slideVoiceIds[index]);
			Atomics.store(this.telemetry, base + 1, this.slideSlots[index]);
			this.scratchFloat[0] = this.slideFrequencies[index];
			Atomics.store(this.telemetry, base + 2, this.scratchInt[0]);
			this.scratchFloat[0] = this.slideTargets[index];
			Atomics.store(this.telemetry, base + 3, this.scratchInt[0]);
			this.scratchFloat[0] = this.slideProgresses[index];
			Atomics.store(this.telemetry, base + 4, this.scratchInt[0]);
			this.scratchFloat[0] = this.slideDurations[index];
			Atomics.store(this.telemetry, base + 5, this.scratchInt[0]);
			Atomics.store(this.telemetry, base + 6, this.slideCurves[index]);
		}
		Atomics.store(this.telemetry, 1, count);
		Atomics.add(this.telemetry, 0, 1);
	}

	private applyFirstEvent() {
		if (!this.engine) return;
		switch (this.events.kind[0]) {
			case 0:
				this.engine.note_on_slide(
					this.events.voiceId[0],
					this.events.role[0],
					this.events.anchor[0],
					this.events.frequency[0],
					this.events.velocity[0],
					this.events.slideVoice[0],
					this.events.travelKind[0],
					this.events.travelValue[0],
					this.events.trigger[0],
					this.events.curve[0]
				);
				break;
			case 1:
				this.engine.note_off(this.events.voiceId[0]);
				break;
			case 2:
				this.engine.set_sustain(this.events.value[0] !== 0);
				break;
			case 3:
				this.engine.panic();
				break;
			case 4:
				this.engine.set_master_gain(this.events.value[0]);
				break;
			case 5:
				this.engine.set_role_gain(this.events.role[0], this.events.value[0]);
				break;
			case 6:
				this.engine.retune(this.events.voiceId[0], this.events.frequency[0]);
				break;
			case 7:
				this.engine.set_role_parameter(
					this.events.role[0],
					this.events.anchor[0],
					this.events.value[0]
				);
				break;
			case 8:
				this.engine.set_pitch_bend_cents(this.events.value[0]);
				break;
			case 9:
				this.engine.set_expression(this.events.value[0]);
				break;
			case 10:
				this.engine.set_mod_wheel(this.events.value[0]);
				break;
		}
	}
}

registerProcessor('elixir-audio', ElixirAudioProcessor);
