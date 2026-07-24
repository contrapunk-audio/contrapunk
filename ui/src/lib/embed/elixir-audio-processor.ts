import init, { ElixirAudio } from '../wasm-pkg/contrapunk_wasm.js';
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

const wasm = await init();
const MAX_FRAMES = 128;
const EVENT_CAPACITY = 1024;

class ElixirAudioProcessor extends AudioWorkletProcessor {
	private readonly engine = new ElixirAudio(sampleRate, MAX_FRAMES);
	private readonly events = new ElixirEventQueue(EVENT_CAPACITY);
	private readonly memoryBuffer = wasm.memory.buffer;
	private readonly rendered = new Float32Array(
		this.memoryBuffer,
		this.engine.output_ptr(),
		this.engine.output_capacity()
	);

	constructor() {
		super();
		this.port.onmessage = ({ data }: MessageEvent) => {
			if (!this.events.insert(data)) {
				this.engine.panic();
				this.events.clear();
				this.port.postMessage({ ack: data.seq, panic: true });
				return;
			}
			this.port.postMessage({ ack: data.seq });
		};
	}

	process(_inputs: Float32Array[][], outputs: Float32Array[][]): boolean {
		const output = outputs[0];
		if (!output?.length) return true;
		if (wasm.memory.buffer !== this.memoryBuffer) {
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
		return true;
	}

	private render(output: Float32Array[], offset: number, frames: number) {
		if (frames <= 0) return;
		const channels = Math.min(output.length, 2);
		this.engine.process(frames, channels);
		for (let frame = 0; frame < frames; frame++) {
			for (let channel = 0; channel < output.length; channel++) {
				output[channel][offset + frame] =
					channel < channels ? this.rendered[frame * channels + channel] : 0;
			}
		}
	}

	private applyFirstEvent() {
		switch (this.events.kind[0]) {
			case 0:
				this.engine.note_on(
					this.events.voiceId[0],
					this.events.role[0],
					this.events.anchor[0],
					this.events.frequency[0],
					this.events.velocity[0]
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
		}
	}
}

registerProcessor('elixir-audio', ElixirAudioProcessor);
