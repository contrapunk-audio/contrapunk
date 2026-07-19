import type { HoldMode } from '$lib/adapter';
import type {
	CounterpointSpeciesName,
	CounterpointStrictnessName,
	HarmonyModeName,
	ImitativeFormName,
	OctaveModeName,
	ScaleModeName,
	VoiceLeadingStyleName
} from '$lib/stores/engine.svelte';

export const ARRANGEMENT_PRESET_SCHEMA_VERSION = 2 as const;

export const ARRANGEMENT_CAPABILITIES = [
	'harmony',
	'voice_leading',
	'strict_canon',
	'free_imitation',
	'species_counterpoint',
	'role_mix',
	'interval_stacks',
	'axis_inversion',
	'tintinnabuli',
	'jazz_extensions',
	'bounded_clusters',
	'spectral_voicing',
	'phrase_capture',
	'phrase_reverse',
	'motif_transform',
	'motif_memory',
	'pattern_lane',
	'odd_meter',
	'polymeter',
	'phase',
	'stable_lane_groups',
	'harmonic_timeline',
	'adaptive_scenes',
	'independent_tonal_centers',
	'microtiming',
	'per_voice_detune',
	'probability_density'
] as const;

export type ArrangementCapability = (typeof ARRANGEMENT_CAPABILITIES)[number];
export type ArrangementFamily = 'classical' | 'jazz' | 'game' | 'custom';
export type PresetResearchStatus =
	| 'pending'
	| 'researched'
	| 'approved'
	| 'blocked'
	| 'not_required';

export interface PresetReference {
	name: string;
	context: string;
}

export interface PresetPlayGuide {
	prompt: string;
	input: 'single_notes' | 'motif' | 'chords';
	articulation: string;
	density: string;
	space: string;
	tempo?: string;
	transportRequired: boolean;
}

export interface ArrangementHarmonyConfig {
	scaleMode: ScaleModeName;
	mode: HarmonyModeName;
	voiceCount: number;
	voicePosition: number;
	voiceLeadingEnabled: boolean;
	voiceLeadingStyle: VoiceLeadingStyleName;
	octaveMode: OctaveModeName;
	octaveIntensity: number;
	interchangeEnabled: boolean;
	interchangeRange: number;
	counterpointSpecies: CounterpointSpeciesName;
	counterpointStrictness: CounterpointStrictnessName;
}

export interface ArrangementCanonVoiceConfig {
	delayBeats: number;
	transposeDegrees: number;
	timeRatio: number;
	harmonyMode: HarmonyModeName | null;
	referenceVoice: number | null;
	voiceCount: number | null;
	voicePosition: number | null;
	voiceLeadingEnabled: boolean | null;
	voiceLeadingStyle: VoiceLeadingStyleName | null;
	octaveMode: OctaveModeName | null;
	counterpointSpecies: CounterpointSpeciesName | null;
	counterpointStrictness: CounterpointStrictnessName | null;
	holdMode: HoldMode | null;
}

export interface ArrangementCanonConfig {
	enabled: boolean;
	form: ImitativeFormName;
	holdMode: HoldMode | null;
	voices: ArrangementCanonVoiceConfig[];
}

export interface ArrangementCounterpointConfig {
	enabled: boolean;
	species: CounterpointSpeciesName;
	transposeDegrees: number;
	preferAbove: boolean;
	holdMode: HoldMode | null;
}

export interface ArrangementConfig {
	harmony: ArrangementHarmonyConfig;
	companion: {
		enabled: boolean;
		globalHoldMode: HoldMode;
		canon: ArrangementCanonConfig;
		counterpoint: ArrangementCounterpointConfig;
	};
	mix: {
		input: number;
		harmony: number;
		canon: number;
		counterpoint: number;
	};
}

/**
 * Musical arrangement only. The absence of tonic, tempo, devices,
 * routing, sound, master level, mute/solo, plugins, and transport state
 * is intentional: applying a preset must preserve the performance
 * environment.
 */
export interface ArrangementPresetV2 {
	schemaVersion: typeof ARRANGEMENT_PRESET_SCHEMA_VERSION;
	id: string;
	name: string;
	family: ArrangementFamily;
	tags: string[];
	builtIn: boolean;
	result: string;
	approximation?: string;
	play: PresetPlayGuide;
	references: PresetReference[];
	researchStatus: PresetResearchStatus;
	requirements: ArrangementCapability[];
	suggestedSoundPresetId?: string;
	config: ArrangementConfig;
}

export function missingArrangementCapabilities(
	preset: Pick<ArrangementPresetV2, 'requirements'>,
	available: ReadonlySet<ArrangementCapability>
): ArrangementCapability[] {
	return preset.requirements.filter((capability) => !available.has(capability));
}

export function arrangementConfigCapabilities(config: ArrangementConfig): ArrangementCapability[] {
	const capabilities = new Set<ArrangementCapability>(['harmony']);
	if (config.harmony.voiceLeadingEnabled) capabilities.add('voice_leading');
	if (config.companion.enabled && config.companion.canon.enabled) {
		capabilities.add(config.companion.canon.form);
	}
	if (config.companion.enabled && config.companion.counterpoint.enabled) {
		capabilities.add('species_counterpoint');
	}
	if (Object.values(config.mix).some((value) => value !== 1)) capabilities.add('role_mix');
	return [...capabilities];
}

export function validateArrangementConfig(config: ArrangementConfig): string[] {
	const errors: string[] = [];
	if (config.harmony.voiceCount < 1 || config.harmony.voiceCount > 8) {
		errors.push('voiceCount must be between 1 and 8');
	}
	if (
		config.harmony.voicePosition < 0 ||
		config.harmony.voicePosition >= config.harmony.voiceCount
	) {
		errors.push('voicePosition must address an active voice');
	}
	if (config.companion.canon.voices.length > 8) {
		errors.push('Canon supports at most 8 voices');
	}
	for (const [role, value] of Object.entries(config.mix)) {
		if (!Number.isFinite(value) || value < 0 || value > 1) {
			errors.push(`${role} mix must be between 0 and 1`);
		}
	}
	return errors;
}
