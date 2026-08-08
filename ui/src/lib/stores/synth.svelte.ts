import { adapter } from '$lib/adapter';
import { appliedMixGain as computeAppliedMixGain } from './synth-mix.mjs';

class SynthStore {
	enabled = $state(true);
	masterGain = $state(0.25);
	mixGains = $state([1, 1, 1, 1]);
	muted = $state([false, false, false, false]);
	solo = $state<number | null>(null);
	mixError = $state<string | null>(null);

	async syncFromBackend() {
		try {
			const state = await adapter.getSynthState();
			this.enabled = state.enabled;
			this.masterGain = state.masterGain;
			if (state.mixGains?.length === 4 && this.solo === null && !this.muted.some(Boolean)) {
				this.mixGains = state.mixGains.map(clamp01);
			}
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

	appliedMixGain(role: number) {
		return computeAppliedMixGain(this.mixGains, this.muted, this.solo, role);
	}

	async setMixGain(role: number, gain: number): Promise<boolean> {
		if (role < 0 || role >= this.mixGains.length) return false;
		const previous = this.mixGains[role] ?? 1;
		this.mixGains = this.mixGains.map((current, index) =>
			index === role ? clamp01(gain) : current
		);
		if (!adapter.capabilities.roleMix) return true;
		try {
			await this.pushMixGain(role);
			this.mixError = null;
			return true;
		} catch (error) {
			this.mixGains = this.mixGains.map((current, index) =>
				index === role ? previous : current
			);
			try { await this.pushMixGain(role); } catch { /* Preserve the original error. */ }
			this.mixError = `Could not change mix level: ${errorMessage(error)}`;
			return false;
		}
	}

	async toggleMute(role: number) {
		if (!adapter.capabilities.roleMix || role < 0 || role >= this.mixGains.length) return;
		const previous = this.muted;
		this.muted = this.muted.map((muted, index) => index === role ? !muted : muted);
		try {
			await this.pushMixGain(role);
			this.mixError = null;
		} catch (error) {
			this.muted = previous;
			try { await this.pushMixGain(role); } catch { /* Preserve the original error. */ }
			this.mixError = `Could not change mute: ${errorMessage(error)}`;
		}
	}

	async toggleSolo(role: number) {
		if (!adapter.capabilities.roleMix || role < 0 || role >= this.mixGains.length) return;
		const previous = this.solo;
		this.solo = this.solo === role ? null : role;
		try {
			await this.pushAllMixGains();
			this.mixError = null;
		} catch (error) {
			this.solo = previous;
			try { await this.pushAllMixGains(); } catch { /* Preserve the original error. */ }
			this.mixError = `Could not change solo: ${errorMessage(error)}`;
		}
	}

	private async pushMixGain(role: number) {
		await adapter.setSynthMixGain(role, clamp01(this.appliedMixGain(role)));
	}

	private async pushAllMixGains() {
		for (let role = 0; role < this.mixGains.length; role++) await this.pushMixGain(role);
	}
}

function errorMessage(error: unknown): string {
	return error instanceof Error ? error.message : String(error);
}

function clamp01(value: number): number {
	return Math.max(0, Math.min(1, Number.isFinite(value) ? value : 1));
}

export const synth = new SynthStore();
