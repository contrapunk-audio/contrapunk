/**
 * Transport Store — Reactive clock / tempo state (Svelte 5 runes).
 *
 * Mirrors the backend's sample-accurate transport. Updated by:
 *  - `syncFromBackend()` after init / setter calls
 *  - `beat-update` Tauri events emitted by the audio clock on every
 *    beat crossing
 *
 * This store is purely reactive state — it never drives the clock
 * itself. Audio-thread timing lives in Rust (see `src/transport/`).
 */

import { adapter } from '$lib/adapter';

class TransportStore {
	/** Whether the clock is currently advancing. */
	running = $state(false);

	/** Current tempo in BPM. */
	bpm = $state(120);

	/** Time signature numerator. */
	beatsPerBar = $state(4);

	/** Time signature denominator. */
	beatUnit = $state(4);

	/** Integer beat within the current bar, 0-based. */
	beatInBar = $state(0);

	/** Monotonic total beat count since the transport last reset. */
	totalBeat = $state(0);

	/** Monotonic bar count since the transport last reset. */
	bar = $state(0);

	/** Sample rate reported by the audio device. */
	sampleRate = $state(48_000);

	/** Incremented on every beat-update event so components can $effect
	 *  on `pulse` to flash without manual timers. */
	pulse = $state(0);

	/** performance.now() timestamp of the last beat crossing. */
	lastCrossedAt = $state(0);

	/** Pull a fresh snapshot from the backend. */
	async syncFromBackend() {
		try {
			const s = await adapter.getTransportState();
			this.running = s.running;
			this.bpm = s.bpm;
			this.beatsPerBar = s.beatsPerBar;
			this.beatUnit = s.beatUnit;
			this.sampleRate = s.sampleRate;
			this.bar = s.bar;
		} catch {
			// Backend not ready; try again after init completes.
		}
	}

	async play() {
		await adapter.transportPlay();
		this.running = true;
	}

	async stop() {
		await adapter.transportStop();
		this.running = false;
	}

	async reset() {
		await adapter.transportReset();
		this.totalBeat = 0;
		this.bar = 0;
		this.beatInBar = 0;
	}

	async setBpm(bpm: number) {
		const clamped = Math.max(20, Math.min(400, bpm));
		this.bpm = clamped;
		await adapter.setBpm(clamped);
	}

	async setTimeSignature(beatsPerBar: number, beatUnit: number) {
		this.beatsPerBar = beatsPerBar;
		this.beatUnit = beatUnit;
		await adapter.setTimeSignature(beatsPerBar, beatUnit);
	}

	/** Called by the Tauri adapter when a `beat-update` event fires. */
	applyBeatUpdate(update: {
		totalBeat: number;
		beatInBar: number;
		bar: number;
		bpm: number;
		running: boolean;
	}) {
		this.totalBeat = update.totalBeat;
		this.beatInBar = update.beatInBar;
		this.bar = update.bar;
		this.bpm = update.bpm;
		this.running = update.running;
		this.pulse = this.pulse + 1;
		this.lastCrossedAt = performance.now();
	}
}

export const transport = new TransportStore();
