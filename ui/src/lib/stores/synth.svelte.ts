import { adapter } from '$lib/adapter';

class SynthStore {
	enabled = $state(true);
	masterGain = $state(0.25);
	mixGains = $state([1, 1, 1, 1]);

	async syncFromBackend() {
		try {
			const state = await adapter.getSynthState();
			this.enabled = state.enabled;
			this.masterGain = state.masterGain;
			this.mixGains = state.mixGains?.slice(0, 4) ?? [1, 1, 1, 1];
		} catch {
			// Backend not ready yet.
		}
	}

	async setEnabled(enabled: boolean) {
		this.enabled = enabled;
		await adapter.setSynthEnabled(enabled);
	}

	async setMasterGain(gain: number) {
		this.masterGain = gain;
		await adapter.setSynthMasterGain(gain);
	}

	async setMixGain(role: number, gain: number) {
		if (role < 0 || role >= this.mixGains.length) return;
		this.mixGains = this.mixGains.map((current, index) => (index === role ? gain : current));
		await adapter.setSynthMixGain(role, gain);
	}
}

export const synth = new SynthStore();
