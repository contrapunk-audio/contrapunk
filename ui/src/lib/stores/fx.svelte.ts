/**
 * FX Store — Reactive state for built-in FX blocks (Svelte 5 runes).
 *
 * Mirrors the Rust FX params atomics. Each setter calls through to
 * the adapter which invokes the backend command; local state is
 * updated optimistically so UI widgets feel responsive.
 */

import { adapter } from '$lib/adapter';
import type { DelaySubdivision } from '$lib/adapter/types';

class ReverbStore {
	enabled = $state(false);
	mix = $state(0.3);
	roomSize = $state(0.7);
	damping = $state(0.5);

	async syncFromBackend() {
		try {
			const s = await adapter.getReverbState();
			this.enabled = s.enabled;
			this.mix = s.mix;
			this.roomSize = s.roomSize;
			this.damping = s.damping;
		} catch {
			// Backend not ready yet.
		}
	}

	async setEnabled(v: boolean) {
		this.enabled = v;
		await adapter.setReverbEnabled(v);
	}
	async setMix(v: number) {
		this.mix = v;
		await adapter.setReverbMix(v);
	}
	async setRoomSize(v: number) {
		this.roomSize = v;
		await adapter.setReverbRoomSize(v);
	}
	async setDamping(v: number) {
		this.damping = v;
		await adapter.setReverbDamping(v);
	}
}

export const reverb = new ReverbStore();

class DelayStore {
	enabled = $state(false);
	mix = $state(0.3);
	timeMs = $state(375);
	feedback = $state(0.35);
	syncEnabled = $state(false);
	subdivision = $state<DelaySubdivision>('1/8d');

	async syncFromBackend() {
		try {
			const s = await adapter.getDelayState();
			this.enabled = s.enabled;
			this.mix = s.mix;
			this.timeMs = s.timeMs;
			this.feedback = s.feedback;
			this.syncEnabled = s.syncEnabled;
			this.subdivision = s.subdivision;
		} catch {
			// Backend not ready yet.
		}
	}

	async setEnabled(v: boolean) {
		this.enabled = v;
		await adapter.setDelayEnabled(v);
	}
	async setMix(v: number) {
		this.mix = v;
		await adapter.setDelayMix(v);
	}
	async setTimeMs(v: number) {
		this.timeMs = v;
		await adapter.setDelayTimeMs(v);
	}
	async setFeedback(v: number) {
		this.feedback = v;
		await adapter.setDelayFeedback(v);
	}
	async setSyncEnabled(v: boolean) {
		this.syncEnabled = v;
		await adapter.setDelaySyncEnabled(v);
	}
	async setSubdivision(v: DelaySubdivision) {
		this.subdivision = v;
		await adapter.setDelaySubdivision(v);
	}
}

export const delay = new DelayStore();
