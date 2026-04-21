/**
 * Synth Store — Reactive built-in synth parameters (Svelte 5 runes).
 *
 * Mirrors the Rust `SynthParams` atomics. Each setter calls through to
 * the adapter which invokes the backend command; local state is
 * updated optimistically so UI widgets feel responsive.
 */

import { adapter } from '$lib/adapter';

export const WAVEFORMS = [
	{ value: 0, label: 'Sine' },
	{ value: 1, label: 'Saw' },
	{ value: 2, label: 'Square' },
	{ value: 3, label: 'Triangle' }
];

class SynthStore {
	enabled = $state(true);
	waveform = $state(0);
	attackMs = $state(5);
	decayMs = $state(120);
	sustain = $state(0.7);
	releaseMs = $state(250);
	cutoffHz = $state(6000);
	resonance = $state(0.2);
	masterGain = $state(0.25);

	async syncFromBackend() {
		try {
			const s = await adapter.getSynthState();
			this.enabled = s.enabled;
			this.waveform = s.waveform;
			this.attackMs = s.attackMs;
			this.decayMs = s.decayMs;
			this.sustain = s.sustain;
			this.releaseMs = s.releaseMs;
			this.cutoffHz = s.cutoffHz;
			this.resonance = s.resonance;
			this.masterGain = s.masterGain;
		} catch {
			// Backend not ready yet.
		}
	}

	async setEnabled(v: boolean) {
		this.enabled = v;
		await adapter.setSynthEnabled(v);
	}
	async setWaveform(v: number) {
		this.waveform = v;
		await adapter.setSynthWaveform(v);
	}
	async setAttackMs(v: number) {
		this.attackMs = v;
		await adapter.setSynthAttackMs(v);
	}
	async setDecayMs(v: number) {
		this.decayMs = v;
		await adapter.setSynthDecayMs(v);
	}
	async setSustain(v: number) {
		this.sustain = v;
		await adapter.setSynthSustain(v);
	}
	async setReleaseMs(v: number) {
		this.releaseMs = v;
		await adapter.setSynthReleaseMs(v);
	}
	async setCutoffHz(v: number) {
		this.cutoffHz = v;
		await adapter.setSynthCutoffHz(v);
	}
	async setResonance(v: number) {
		this.resonance = v;
		await adapter.setSynthResonance(v);
	}
	async setMasterGain(v: number) {
		this.masterGain = v;
		await adapter.setSynthMasterGain(v);
	}
}

export const synth = new SynthStore();
