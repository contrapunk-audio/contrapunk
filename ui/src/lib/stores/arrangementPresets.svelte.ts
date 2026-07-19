import {
	ARRANGEMENT_PRESET_SCHEMA_VERSION,
	type ArrangementPresetV2
} from '$lib/arrangement/presets';
import {
	loadUserArrangementPresets,
	saveUserArrangementPresets
} from '$lib/arrangement/persistence';

class ArrangementPresetStore {
	userPresets = $state<ArrangementPresetV2[]>(loadUserArrangementPresets());

	create(preset: Omit<ArrangementPresetV2, 'schemaVersion' | 'id' | 'builtIn'>): string {
		const id = `user-${globalThis.crypto?.randomUUID?.() ?? Date.now()}`;
		this.userPresets = [
			...this.userPresets,
			{
				...preset,
				schemaVersion: ARRANGEMENT_PRESET_SCHEMA_VERSION,
				id,
				builtIn: false
			}
		];
		this.persist();
		return id;
	}

	update(id: string, patch: Partial<Omit<ArrangementPresetV2, 'schemaVersion' | 'id' | 'builtIn'>>) {
		this.userPresets = this.userPresets.map((preset) =>
			preset.id === id ? { ...preset, ...patch } : preset
		);
		this.persist();
	}

	delete(id: string) {
		this.userPresets = this.userPresets.filter((preset) => preset.id !== id);
		this.persist();
	}

	private persist() {
		saveUserArrangementPresets(this.userPresets);
	}
}

export const arrangementPresets = new ArrangementPresetStore();
