import type {
	ArrangementCapability,
	ArrangementPresetV2,
	ArrangementCanonVoiceConfig
} from './presets';

export const ARRANGEMENT_PRESET_STORAGE_KEY = 'contrapunk-arrangement-presets-v2';
const LEGACY_STORAGE_KEY = 'contrapunk-ensemble-presets-v1';

export function loadUserArrangementPresets(): ArrangementPresetV2[] {
	if (typeof localStorage === 'undefined') return [];
	try {
		const saved = JSON.parse(localStorage.getItem(ARRANGEMENT_PRESET_STORAGE_KEY) ?? 'null');
		if (saved?.version === 2 && Array.isArray(saved.presets)) {
			return saved.presets.filter(isUserArrangementPreset);
		}

		const legacy = JSON.parse(localStorage.getItem(LEGACY_STORAGE_KEY) ?? '[]');
		if (!Array.isArray(legacy)) return [];
		const migrated = legacy.map(migrateLegacyPreset).filter((preset) => preset !== null);
		if (migrated.length) saveUserArrangementPresets(migrated);
		return migrated;
	} catch {
		return [];
	}
}

export function saveUserArrangementPresets(presets: ArrangementPresetV2[]) {
	if (typeof localStorage === 'undefined') return;
	localStorage.setItem(
		ARRANGEMENT_PRESET_STORAGE_KEY,
		JSON.stringify({ version: 2, presets: presets.filter((preset) => !preset.builtIn) })
	);
}

export function migrateLegacyPreset(value: unknown): ArrangementPresetV2 | null {
	if (!isRecord(value) || value.builtIn !== false) return null;
	if (typeof value.id !== 'string' || typeof value.name !== 'string') return null;
	if (
		typeof value.scaleMode !== 'string' ||
		typeof value.mode !== 'string' ||
		typeof value.voiceCount !== 'number' ||
		typeof value.voicePosition !== 'number' ||
		typeof value.voiceLeadingEnabled !== 'boolean' ||
		typeof value.voiceLeadingStyle !== 'string' ||
		typeof value.octaveMode !== 'string' ||
		typeof value.companionEnabled !== 'boolean' ||
		typeof value.canonEnabled !== 'boolean' ||
		(value.imitativeForm !== 'strict_canon' && value.imitativeForm !== 'free_imitation') ||
		!Array.isArray(value.canonVoices)
	) {
		return null;
	}

	const counterpoint = isRecord(value.counterpoint) ? value.counterpoint : null;
	const counterpointEnabled = counterpoint?.enabled === true;
	const requirements = new Set<ArrangementCapability>(['harmony']);
	if (value.voiceLeadingEnabled) requirements.add('voice_leading');
	if (value.canonEnabled) requirements.add(value.imitativeForm);
	if (counterpointEnabled) requirements.add('species_counterpoint');

	return {
		schemaVersion: 2,
		id: value.id,
		name: value.name,
		family: 'custom',
		tags: ['migrated'],
		builtIn: false,
		result: 'Migrated custom arrangement.',
		play: {
			prompt: 'Play naturally; edit this preset to add performance guidance.',
			input: 'single_notes',
			articulation: 'Any',
			density: 'Any',
			space: 'As needed',
			transportRequired: value.canonEnabled
		},
		references: [],
		researchStatus: 'not_required',
		requirements: [...requirements],
		config: {
			harmony: {
				scaleMode: value.scaleMode as ArrangementPresetV2['config']['harmony']['scaleMode'],
				mode: value.mode as ArrangementPresetV2['config']['harmony']['mode'],
				voiceCount: value.voiceCount,
				voicePosition: value.voicePosition,
				voiceLeadingEnabled: value.voiceLeadingEnabled,
				voiceLeadingStyle: value.voiceLeadingStyle as ArrangementPresetV2['config']['harmony']['voiceLeadingStyle'],
				octaveMode: value.octaveMode as ArrangementPresetV2['config']['harmony']['octaveMode'],
				octaveIntensity: 1,
				interchangeEnabled: false,
				interchangeRange: 3,
				counterpointSpecies: (typeof value.harmonySpecies === 'string' ? value.harmonySpecies : 'Species1') as ArrangementPresetV2['config']['harmony']['counterpointSpecies'],
				counterpointStrictness: 'Strict'
			},
			companion: {
				enabled: value.companionEnabled,
				globalHoldMode: { kind: 'near_future', tail_beats: 1 },
				canon: {
					enabled: value.canonEnabled,
					form: value.imitativeForm,
					holdMode: null,
					voices: value.canonVoices.map(migrateLegacyCanonVoice).filter((voice) => voice !== null)
				},
				counterpoint: {
					enabled: counterpointEnabled,
					species: (typeof counterpoint?.species === 'string' ? counterpoint.species : 'Species1') as ArrangementPresetV2['config']['companion']['counterpoint']['species'],
					transposeDegrees: typeof counterpoint?.transpose_degrees === 'number' ? counterpoint.transpose_degrees : 2,
					preferAbove: counterpoint?.prefer_above !== false,
					holdMode: null
				}
			},
			mix: { input: 1, harmony: 1, canon: 1, counterpoint: 1 }
		}
	};
}

function migrateLegacyCanonVoice(value: unknown): ArrangementCanonVoiceConfig | null {
	if (!isRecord(value)) return null;
	if (typeof value.delay_beats !== 'number' || typeof value.transpose_degrees !== 'number') {
		return null;
	}
	return {
		delayBeats: value.delay_beats,
		transposeDegrees: value.transpose_degrees,
		timeRatio: typeof value.time_ratio === 'number' ? value.time_ratio : 1,
		harmonyMode: typeof value.harmony_mode === 'string' ? value.harmony_mode as ArrangementCanonVoiceConfig['harmonyMode'] : null,
		referenceVoice: typeof value.reference_voice === 'number' ? value.reference_voice : null,
		voiceCount: typeof value.voice_count === 'number' ? value.voice_count : null,
		voicePosition: typeof value.voice_position === 'number' ? value.voice_position : null,
		voiceLeadingEnabled: typeof value.voice_leading_enabled === 'boolean' ? value.voice_leading_enabled : null,
		voiceLeadingStyle: typeof value.voice_leading_style === 'string' ? value.voice_leading_style as ArrangementCanonVoiceConfig['voiceLeadingStyle'] : null,
		octaveMode: typeof value.octave_mode === 'string' ? value.octave_mode as ArrangementCanonVoiceConfig['octaveMode'] : null,
		counterpointSpecies: typeof value.counterpoint_species === 'string' ? value.counterpoint_species as ArrangementCanonVoiceConfig['counterpointSpecies'] : null,
		counterpointStrictness: typeof value.counterpoint_strictness === 'string' ? value.counterpoint_strictness as ArrangementCanonVoiceConfig['counterpointStrictness'] : null,
		holdMode: isRecord(value.hold_mode) ? value.hold_mode as ArrangementCanonVoiceConfig['holdMode'] : null
	};
}

function isUserArrangementPreset(value: unknown): value is ArrangementPresetV2 {
	return isRecord(value)
		&& value.schemaVersion === 2
		&& value.builtIn === false
		&& typeof value.id === 'string'
		&& typeof value.name === 'string'
		&& isRecord(value.config);
}

function isRecord(value: unknown): value is Record<string, unknown> {
	return typeof value === 'object' && value !== null;
}
