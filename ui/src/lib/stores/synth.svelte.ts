import { adapter, platformName, type SynthRolePatch } from '$lib/adapter';
import { cloneRolePatch, defaultRolePatch } from '$lib/elixir/patch';
import { appliedMixGain as computeAppliedMixGain } from './synth-mix.mjs';

class SynthStore {
	enabled = $state(true);
	masterGain = $state(0.25);
	mixGains = $state([1, 1, 1, 1]);
	rolePatches = $state(Array.from({ length: 4 }, defaultRolePatch));
	muted = $state([false, false, false, false]);
	solo = $state<number | null>(null);
	mixError = $state<string | null>(null);
	patchError = $state<string | null>(null);
	private patchQueues: Promise<void>[] = Array.from({ length: 4 }, () => Promise.resolve());
	private restoredLocalPatches = false;

	async syncFromBackend() {
		try {
			const state = await adapter.getSynthState();
			this.enabled = state.enabled;
			this.masterGain = state.masterGain;
			if (state.mixGains?.length === 4 && this.solo === null && !this.muted.some(Boolean)) {
				this.mixGains = state.mixGains.map(clamp01);
			}
			if (state.rolePatches?.length === 4) {
				this.rolePatches = state.rolePatches.map(cloneRolePatch);
			}
			if (!this.restoredLocalPatches && platformName !== 'plugin' && typeof localStorage !== 'undefined') {
				this.restoredLocalPatches = true;
				try {
					const saved = JSON.parse(localStorage.getItem('contrapunk.elixir.rolePatches.v1') ?? 'null');
					if (Array.isArray(saved) && saved.length === 4 && saved.every(isRolePatch)) {
						this.rolePatches = saved.map(cloneRolePatch);
						for (let role = 0; role < 4; role++) {
							await adapter.setSynthRolePatch(role, this.rolePatches[role]);
						}
					}
				} catch {
					localStorage.removeItem('contrapunk.elixir.rolePatches.v1');
				}
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

	async setRolePatch(role: number, patch: SynthRolePatch, persist = true): Promise<boolean> {
		if (role < 0 || role >= this.rolePatches.length) return false;
		const next = cloneRolePatch(patch);
		this.rolePatches = this.rolePatches.map((current, index) =>
			index === role ? next : current
		);
		if (persist && platformName !== 'plugin' && typeof localStorage !== 'undefined') {
			localStorage.setItem('contrapunk.elixir.rolePatches.v1', JSON.stringify(this.rolePatches));
		}
		const request = this.patchQueues[role]
			.then(() => adapter.setSynthRolePatch(role, next))
			.then(() => { this.patchError = null; })
			.catch(async (error) => {
				this.patchError = `Could not change sound: ${errorMessage(error)}`;
				await this.syncFromBackend();
				throw error;
			});
		this.patchQueues[role] = request.catch(() => {});
		try {
			await request;
			return true;
		} catch {
			return false;
		}
	}

	async updateRolePatch(role: number, update: (patch: SynthRolePatch) => void) {
		const patch = cloneRolePatch(this.rolePatches[role] ?? defaultRolePatch());
		update(patch);
		return this.setRolePatch(role, patch);
	}

	async setAllRolePatches(patches: SynthRolePatch[], persist = true) {
		for (let role = 0; role < Math.min(4, patches.length); role++) {
			await this.setRolePatch(role, patches[role], persist);
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

function isRolePatch(value: unknown): value is SynthRolePatch {
	if (!value || typeof value !== 'object') return false;
	const patch = value as Partial<SynthRolePatch>;
	const amplitudes = patch.harmonics?.amplitudes;
	const phases = patch.harmonics?.phases;
	if (!Array.isArray(amplitudes) || amplitudes.length !== 6 || !amplitudes.every(Number.isFinite)) return false;
	if (!Array.isArray(phases) || phases.length !== 6 || !phases.every(Number.isFinite)) return false;
	if (!patch.secondary || !['primary_only', 'add', 'ring'].includes(patch.secondary.mode)) return false;
	if (!patch.envelope || !patch.vibrato) return false;
	return [
		patch.secondary.semitones,
		patch.secondary.fineCents,
		patch.secondary.phase,
		patch.secondary.level,
		patch.envelope.attackSecs,
		patch.envelope.decaySecs,
		patch.envelope.sustainLevel,
		patch.envelope.releaseSecs,
		patch.envelope.velocitySensitivity,
		patch.envelope.expressionSensitivity,
		patch.vibrato.rateHz,
		patch.vibrato.depthCents,
		patch.vibrato.modWheelDepthCents
	].every(Number.isFinite);
}

function errorMessage(error: unknown): string {
	return error instanceof Error ? error.message : String(error);
}

function clamp01(value: number): number {
	return Math.max(0, Math.min(1, Number.isFinite(value) ? value : 1));
}

export const synth = new SynthStore();
