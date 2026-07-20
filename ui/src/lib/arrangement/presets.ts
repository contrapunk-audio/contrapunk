import type { HoldMode } from '$lib/adapter';
import type {
	CounterpointSpeciesName,
	CounterpointStrictnessName,
	ExplicitIntervalMapConfig,
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
	/** Required when mode is ExplicitIntervals; optional for schema-v2 compatibility. */
	explicitIntervalMap?: ExplicitIntervalMapConfig;
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

export type ArrangementPatternLaneId = 'pattern_low' | 'pattern_counter';

export interface ArrangementPatternEventConfig {
	beat: number;
	degree: number;
	octave: number;
	durationBeats: number;
	velocity: number;
}

export interface ArrangementPatternLaneConfig {
	enabled: boolean;
	cycleBeats: number;
	tailBeats: number;
	events: ArrangementPatternEventConfig[];
}

export interface ArrangementPatternConfig {
	lowSupport: ArrangementPatternLaneConfig;
	counterline: ArrangementPatternLaneConfig;
}

export interface ArrangementConfig {
	harmony: ArrangementHarmonyConfig;
	companion: {
		enabled: boolean;
		globalHoldMode: HoldMode;
		canon: ArrangementCanonConfig;
		counterpoint: ArrangementCounterpointConfig;
		/** Optional for backward-compatible schema-v2 user presets. */
		patterns?: ArrangementPatternConfig;
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
	if (config.harmony.mode === 'ExplicitIntervals') capabilities.add('interval_stacks');
	if (config.companion.enabled && config.companion.canon.enabled) {
		capabilities.add(config.companion.canon.form);
	}
	if (config.companion.enabled && config.companion.counterpoint.enabled) {
		capabilities.add('species_counterpoint');
	}
	const patterns = config.companion.patterns;
	if (config.companion.enabled && patterns && (patterns.lowSupport.enabled || patterns.counterline.enabled)) {
		capabilities.add('pattern_lane');
		if (patterns.lowSupport.enabled && patterns.counterline.enabled) {
			capabilities.add('stable_lane_groups');
		}
	}
	if (Object.values(config.mix).some((value) => value !== 1)) capabilities.add('role_mix');
	return [...capabilities];
}

export function validateArrangementPreset(preset: ArrangementPresetV2): string[] {
	const errors = validateArrangementConfig(preset.config);
	if (
		preset.researchStatus === 'approved' &&
		preset.config.companion.canon.enabled &&
		!preset.requirements.includes('phrase_capture') &&
		preset.config.companion.canon.voices.some((voice) => voice.timeRatio < 1)
	) {
		errors.push('Live proportional ratios below 1 require phrase capture');
	}
	if (
		preset.researchStatus === 'approved' &&
		preset.config.companion.canon.form === 'strict_canon' &&
		preset.config.companion.canon.voices.some(
			(voice) =>
				voice.timeRatio !== 1 || voice.harmonyMode !== 'PassThrough' || voice.voiceCount !== 1
		)
	) {
		errors.push('Strict Canon followers must be single PassThrough lines at timeRatio 1');
	}
	return errors;
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
	if (config.harmony.mode === 'ExplicitIntervals') {
		const map = config.harmony.explicitIntervalMap;
		if (!map) {
			errors.push('ExplicitIntervals mode requires explicitIntervalMap');
		} else {
			if (map.degreeOffsets.length !== 7) {
				errors.push('explicitIntervalMap requires exactly seven degree entries');
			}
			for (const [label, offsets] of [
				...map.degreeOffsets.map((offsets, index) => [`degree ${index + 1}`, offsets] as const),
				['fallback', map.fallbackOffsets] as const
			]) {
				if (offsets.length > 7) errors.push(`${label} supports at most seven interval offsets`);
				if (offsets.some((offset, index) => !Number.isInteger(offset) || offset === 0 || offset < -48 || offset > 48 || offsets.indexOf(offset) !== index)) {
					errors.push(`${label} interval offsets must be unique nonzero integers between -48 and 48`);
				}
			}
		}
	}
	if (config.companion.canon.voices.length > 8) {
		errors.push('Canon supports at most 8 voices');
	}
	for (const [index, voice] of config.companion.canon.voices.entries()) {
		if (!Number.isFinite(voice.timeRatio) || voice.timeRatio < 0.125 || voice.timeRatio > 8) {
			errors.push(`Canon voice ${index + 1} timeRatio must be between 0.125 and 8`);
		}
	}
	if (config.companion.patterns) {
		for (const [role, pattern] of Object.entries(config.companion.patterns)) {
			if (!Number.isFinite(pattern.cycleBeats) || pattern.cycleBeats < 0.25 || pattern.cycleBeats > 32) {
				errors.push(`${role} cycleBeats must be between 0.25 and 32`);
			}
			if (!Number.isFinite(pattern.tailBeats) || pattern.tailBeats < 0.25 || pattern.tailBeats > 32) {
				errors.push(`${role} tailBeats must be between 0.25 and 32`);
			}
			if (pattern.events.length > 16) errors.push(`${role} supports at most 16 events`);
			for (const event of pattern.events) {
				if (!Number.isFinite(event.beat) || event.beat < 0 || event.beat >= pattern.cycleBeats) {
					errors.push(`${role} event beat must fall inside the cycle`);
				}
				if (!Number.isInteger(event.degree) || event.degree < 0 || event.degree > 6) {
					errors.push(`${role} event degree must be between 0 and 6`);
				}
				if (!Number.isInteger(event.octave) || event.octave < -4 || event.octave > 4) {
					errors.push(`${role} event octave must be between -4 and 4`);
				}
				if (!Number.isFinite(event.durationBeats) || event.durationBeats < 0.03125 || event.durationBeats > 32) {
					errors.push(`${role} event durationBeats must be between 0.03125 and 32`);
				}
				if (!Number.isInteger(event.velocity) || event.velocity < 1 || event.velocity > 127) {
					errors.push(`${role} event velocity must be between 1 and 127`);
				}
			}
		}
	}
	for (const [role, value] of Object.entries(config.mix)) {
		if (!Number.isFinite(value) || value < 0 || value > 1) {
			errors.push(`${role} mix must be between 0 and 1`);
		}
	}
	return errors;
}
