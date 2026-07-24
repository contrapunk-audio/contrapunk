import { adapter } from '$lib/adapter';

class SynthStore {
	enabled = $state(true);
	masterGain = $state(0.25);

	async syncFromBackend() {
		try {
			const state = await adapter.getSynthState();
			this.enabled = state.enabled;
			this.masterGain = state.masterGain;
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
}

export const synth = new SynthStore();
