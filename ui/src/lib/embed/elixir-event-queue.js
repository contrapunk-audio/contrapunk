// Fixed-capacity, allocation-free storage used by the AudioWorklet callback.
/** @typedef {{ kind: number, atFrame: number, seq: number, voiceId?: number, role?: number, anchor?: number, frequency?: number, velocity?: number, value?: number, slideVoice?: number, travelKind?: number, travelValue?: number, trigger?: number, curve?: number }} ElixirEvent */
export class ElixirEventQueue {
	constructor(capacity = 1024) {
		this.capacity = capacity;
		this.length = 0;
		this.kind = new Uint8Array(capacity);
		this.atFrame = new Float64Array(capacity);
		this.voiceId = new Uint32Array(capacity);
		this.role = new Uint8Array(capacity);
		this.anchor = new Uint8Array(capacity);
		this.frequency = new Float32Array(capacity);
		this.velocity = new Uint8Array(capacity);
		this.value = new Float32Array(capacity);
		this.slideVoice = new Uint8Array(capacity);
		this.travelKind = new Uint8Array(capacity);
		this.travelValue = new Float32Array(capacity);
		this.trigger = new Uint8Array(capacity);
		this.curve = new Uint8Array(capacity);
		this.seq = new Uint32Array(capacity);
	}

	/** @param {ElixirEvent} event */
	insert(event) {
		if (this.length === this.capacity) return false;
		let index = this.length;
		while (index > 0 && this.atFrame[index - 1] > event.atFrame) {
			this.copy(index - 1, index);
			index--;
		}
		this.kind[index] = event.kind;
		this.atFrame[index] = event.atFrame;
		this.voiceId[index] = event.voiceId ?? 0;
		this.role[index] = event.role ?? 0;
		this.anchor[index] = event.anchor ?? 0;
		this.frequency[index] = event.frequency ?? 0;
		this.velocity[index] = event.velocity ?? 0;
		this.value[index] = event.value ?? 0;
		this.slideVoice[index] = event.slideVoice ?? 0;
		this.travelKind[index] = event.travelKind ?? 0;
		this.travelValue[index] = event.travelValue ?? 0;
		this.trigger[index] = event.trigger ?? 0;
		this.curve[index] = event.curve ?? 0;
		this.seq[index] = event.seq;
		this.length++;
		return true;
	}

	removeFirst() {
		// ponytail: O(n) shift; use an indexed heap only if event rates become audio-rate.
		for (let i = 1; i < this.length; i++) this.copy(i, i - 1);
		this.length--;
	}

	clear() {
		this.length = 0;
	}

	/** @param {number} from @param {number} to */
	copy(from, to) {
		this.kind[to] = this.kind[from];
		this.atFrame[to] = this.atFrame[from];
		this.voiceId[to] = this.voiceId[from];
		this.role[to] = this.role[from];
		this.anchor[to] = this.anchor[from];
		this.frequency[to] = this.frequency[from];
		this.velocity[to] = this.velocity[from];
		this.value[to] = this.value[from];
		this.slideVoice[to] = this.slideVoice[from];
		this.travelKind[to] = this.travelKind[from];
		this.travelValue[to] = this.travelValue[from];
		this.trigger[to] = this.trigger[from];
		this.curve[to] = this.curve[from];
		this.seq[to] = this.seq[from];
	}
}
