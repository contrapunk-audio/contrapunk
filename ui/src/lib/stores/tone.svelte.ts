import { adapter } from '$lib/adapter';

class ToneStore {
	readonly noteNames = ['C', 'C♯', 'D', 'E♭', 'E', 'F', 'F♯', 'G', 'A♭', 'A', 'B♭', 'B'];
	pitchClass = $state(0);
	octave = $state(4);
	velocity = $state(100);
	sounding = $state<number | null>(null);
	desired = $state<number | null>(null);
	error = $state<string | null>(null);
	private operation: Promise<void> = Promise.resolve();

	get midiNote() {
		return Math.max(0, Math.min(127, (this.octave + 1) * 12 + this.pitchClass));
	}

	get frequency() {
		return 440 * 2 ** ((this.midiNote - 69) / 12);
	}

	setPitchClass(value: number) {
		this.pitchClass = Math.max(0, Math.min(11, Math.trunc(value)));
		this.retarget();
	}

	setOctave(value: number) {
		this.octave = Math.max(0, Math.min(7, Math.trunc(value)));
		this.retarget();
	}

	setVelocity(value: number) {
		this.velocity = Math.max(1, Math.min(127, Math.trunc(value)));
	}

	toggle() {
		this.desired = this.desired === null ? this.midiNote : null;
		return this.updateGate();
	}

	stop() {
		this.desired = null;
		return this.updateGate();
	}

	panic() {
		this.desired = null;
		this.operation = this.operation
			.then(() => adapter.panicAllNotesOff())
			.then(() => {
				this.sounding = null;
				this.error = null;
			})
			.catch((error) => {
				this.sounding = null;
				this.error = error instanceof Error ? error.message : String(error);
			});
		return this.operation;
	}

	private retarget() {
		if (this.desired === null) return;
		this.desired = this.midiNote;
		void this.updateGate();
	}

	private updateGate() {
		this.operation = this.operation
			.then(async () => {
				const next = this.desired;
				const previous = this.sounding;
				if (next !== null && next !== previous) {
					await adapter.injectNoteOn(next, this.velocity);
				}
				if (previous !== null && previous !== next) {
					await adapter.injectNoteOff(previous);
				}
				this.sounding = next;
				this.error = null;
			})
			.catch(async (error) => {
				this.error = error instanceof Error ? error.message : String(error);
				try {
					await adapter.panicAllNotesOff();
				} catch {
					/* The main Panic control remains available. */
				}
				this.desired = null;
				this.sounding = null;
			});
		return this.operation;
	}
}

export const tone = new ToneStore();
