import {
	ARRANGEMENT_PRESET_SCHEMA_VERSION,
	type ArrangementPresetV2
} from '$lib/arrangement/presets';
import { BUILT_IN_ARRANGEMENT_PRESETS } from '$lib/arrangement/catalog';
import {
	loadUserArrangementPresets,
	saveUserArrangementPresets
} from '$lib/arrangement/persistence';

class ArrangementPresetStore {
	userPresets = $state<ArrangementPresetV2[]>(loadUserArrangementPresets());
	selectedId = $state('02-modal-linework');
	appliedId = $state<string | null>(null);

	get builtInPresets(): readonly ArrangementPresetV2[] {
		return BUILT_IN_ARRANGEMENT_PRESETS;
	}

	get allPresets(): readonly ArrangementPresetV2[] {
		return [...BUILT_IN_ARRANGEMENT_PRESETS, ...this.userPresets];
	}

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
