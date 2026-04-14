/**
 * Engine Store — Reactive Harmony Engine State (Svelte 5 Runes)
 *
 * Tracks the harmony engine configuration and real-time note state.
 * All mutations call the platform adapter and update local state
 * optimistically (set locally, then call backend; revert on error).
 */

import { adapter } from '$lib/adapter';
import type { NoteState } from '$lib/adapter';

/** Compare two note arrays (order-independent) to avoid unnecessary Svelte re-renders.
 *  HashSet iteration order is non-deterministic, so we must sort before comparing. */
function sameNotes(a: number[], b: number[]): boolean {
	if (a.length !== b.length) return false;
	if (a.length === 0) return true;
	// Sort copies to compare regardless of HashSet ordering
	const sa = [...a].sort();
	const sb = [...b].sort();
	for (let i = 0; i < sa.length; i++) {
		if (sa[i] !== sb[i]) return false;
	}
	return true;
}

// === Type definitions ===

export type KeyName =
	| 'C'
	| 'C#'
	| 'D'
	| 'D#'
	| 'E'
	| 'F'
	| 'F#'
	| 'G'
	| 'G#'
	| 'A'
	| 'A#'
	| 'B';

export type HarmonyModeName =
	| 'PassThrough'
	| 'DiatonicThirds'
	| 'DiatonicFourths'
	| 'RandomBelow'
	| 'RandomBelowNoSeconds'
	| 'ContraryMotion'
	| 'StrictCounterpoint'
	| 'BarryHarris'
	| 'FunctionalHarmony'
	| 'BachChorale';

export type ScaleFamilyName =
	| 'Diatonic'
	| 'HarmonicMinor'
	| 'MelodicMinor'
	| 'HarmonicMajor'
	| 'DoubleHarmonic'
	| 'Pentatonic'
	| 'Blues'
	| 'Symmetric'
	| 'World'
	| 'BarryHarris';

export type ScaleModeName =
	// Diatonic (7)
	| 'Ionian'
	| 'Dorian'
	| 'Phrygian'
	| 'Lydian'
	| 'Mixolydian'
	| 'Aeolian'
	| 'Locrian'
	// Harmonic Minor (7)
	| 'HarmonicMinor'
	| 'LocrianNat6'
	| 'IonianAug'
	| 'DorianSharp4'
	| 'PhrygianDominant'
	| 'LydianSharp2'
	| 'SuperLocrianDim'
	// Melodic Minor (7)
	| 'MelodicMinor'
	| 'DorianFlat2'
	| 'LydianAug'
	| 'LydianDominant'
	| 'MixolydianFlat6'
	| 'LocrianNat2'
	| 'SuperLocrian'
	// Harmonic Major (7)
	| 'HarmonicMajor'
	| 'DorianFlat5'
	| 'PhrygianFlat4'
	| 'LydianFlat3'
	| 'MixolydianFlat2'
	| 'LydianAugSharp2'
	| 'LocrianDoubleFlat7'
	// Double Harmonic (7)
	| 'DoubleHarmonic'
	| 'LydianSharp2Sharp6'
	| 'Ultraphrygian'
	| 'HungarianMinor'
	| 'Oriental'
	| 'IonianSharp2Sharp5'
	| 'LocrianDoubleFlat3DoubleFlat7'
	// Pentatonic (8)
	| 'MajorPentatonic'
	| 'MinorPentatonic'
	| 'Hirajoshi'
	| 'InSen'
	| 'Iwato'
	| 'Yo'
	| 'Kumoi'
	| 'Pelog'
	// Blues & Bebop (3)
	| 'MinorBlues'
	| 'MajorBlues'
	| 'BebopDominant'
	// Symmetric (4)
	| 'WholeTone'
	| 'DiminishedWholeHalf'
	| 'DiminishedHalfWhole'
	| 'AugmentedHex'
	// World (5)
	| 'Enigmatic'
	| 'NeapolitanMinor'
	| 'NeapolitanMajor'
	| 'Persian'
	| 'HungarianMajor'
	// Barry Harris (2)
	| 'BHMajor6thDim'
	| 'BHMinor6thDim';

export type OctaveModeName = 'None' | 'Spread' | 'BassTrebleSplit' | 'Mirror';

export type VoiceLeadingStyleName = 'Free' | 'Palestrina' | 'BachChorale' | 'Jazz';

export type CounterpointSpeciesName =
	| 'Species1'
	| 'Species2'
	| 'Species3'
	| 'Species4';

export type CounterpointStrictnessName = 'Relaxed' | 'Strict';

export const COUNTERPOINT_SPECIES: {
	name: CounterpointSpeciesName;
	label: string;
	shortLabel: string;
	tooltip: string;
}[] = [
	{
		name: 'Species1',
		label: 'Species 1 (1:1)',
		shortLabel: 'I',
		tooltip: 'Note-against-note. One harmony note per melody note.'
	},
	{
		name: 'Species2',
		label: 'Species 2 (2:1)',
		shortLabel: 'II',
		tooltip: 'Two harmony notes per melody note; passing tones on weak beats.'
	},
	{
		name: 'Species3',
		label: 'Species 3 (4:1)',
		shortLabel: 'III',
		tooltip: 'Four harmony notes per melody note; figuration on weak beats.'
	},
	{
		name: 'Species4',
		label: 'Species 4 (Susp.)',
		shortLabel: 'IV',
		tooltip: 'Syncopated harmony: prepare / suspend / resolve.'
	}
];

export const COUNTERPOINT_STRICTNESS: {
	name: CounterpointStrictnessName;
	label: string;
	tooltip: string;
}[] = [
	{
		name: 'Relaxed',
		label: 'Relaxed',
		tooltip: 'Lighter penalties — more permissive harmonic choices.'
	},
	{
		name: 'Strict',
		label: 'Strict',
		tooltip: 'Fux-aligned scoring — enforces species rules strictly.'
	}
];

export interface ScaleFamilyGroup {
	family: ScaleFamilyName;
	label: string;
	modes: { name: ScaleModeName; label: string }[];
}

// === Constants ===

export const ALL_KEYS: KeyName[] = [
	'C',
	'C#',
	'D',
	'D#',
	'E',
	'F',
	'F#',
	'G',
	'G#',
	'A',
	'A#',
	'B'
];

/** Display labels for keys — uses flats where musicians expect them. */
export const KEY_DISPLAY: Record<KeyName, string> = {
	'C': 'C',
	'C#': 'C#/Db',
	'D': 'D',
	'D#': 'Eb',
	'E': 'E',
	'F': 'F',
	'F#': 'F#/Gb',
	'G': 'G',
	'G#': 'Ab',
	'A': 'A',
	'A#': 'Bb',
	'B': 'B',
};

export const ALL_MODES: {
	name: HarmonyModeName;
	label: string;
	shortLabel: string;
	tooltip: string;
}[] = [
	{
		name: 'PassThrough',
		label: 'Pass Through',
		shortLabel: 'Pass',
		tooltip: 'Notes pass unchanged to output'
	},
	{
		name: 'DiatonicThirds',
		label: 'Parallel Thirds',
		shortLabel: '3rds',
		tooltip: 'Adds +2 scale degrees per voice. Multiple voices stack into 7th chords.'
	},
	{
		name: 'DiatonicFourths',
		label: 'Parallel Fourths',
		shortLabel: '4ths',
		tooltip: 'Adds +3 scale degrees per voice. Multiple voices stack into extended chords.'
	},
	{
		name: 'RandomBelow',
		label: 'Random Below',
		shortLabel: 'Rand',
		tooltip: 'Random diatonic interval (2nd-7th) below the melody'
	},
	{
		name: 'RandomBelowNoSeconds',
		label: 'Random Below (consonant)',
		shortLabel: 'Rand-',
		tooltip: 'Random diatonic interval below, excluding dissonant 2nds'
	},
	{
		name: 'ContraryMotion',
		label: 'Contrary Motion',
		shortLabel: 'Contra',
		tooltip: 'Harmony moves opposite to melody direction. Tracks previous notes.'
	},
	{
		name: 'StrictCounterpoint',
		label: 'Counterpoint (Species 1)',
		shortLabel: 'Cpt',
		tooltip:
			'Note-against-note voice leading with scoring. No parallel 5ths/octaves, prefers contrary/stepwise motion.'
	}
];

export const SCALE_FAMILIES: ScaleFamilyGroup[] = [
	{
		family: 'Diatonic',
		label: 'Diatonic Modes',
		modes: [
			{ name: 'Ionian', label: 'Ionian (Major)' },
			{ name: 'Dorian', label: 'Dorian' },
			{ name: 'Phrygian', label: 'Phrygian' },
			{ name: 'Lydian', label: 'Lydian' },
			{ name: 'Mixolydian', label: 'Mixolydian' },
			{ name: 'Aeolian', label: 'Aeolian (Minor)' },
			{ name: 'Locrian', label: 'Locrian' }
		]
	},
	{
		family: 'HarmonicMinor',
		label: 'Harmonic Minor',
		modes: [
			{ name: 'HarmonicMinor', label: 'Harmonic Minor' },
			{ name: 'LocrianNat6', label: 'Locrian Nat 6' },
			{ name: 'IonianAug', label: 'Ionian Aug' },
			{ name: 'DorianSharp4', label: 'Dorian #4' },
			{ name: 'PhrygianDominant', label: 'Phrygian Dom' },
			{ name: 'LydianSharp2', label: 'Lydian #2' },
			{ name: 'SuperLocrianDim', label: 'Super Locrian Dim' }
		]
	},
	{
		family: 'MelodicMinor',
		label: 'Melodic Minor',
		modes: [
			{ name: 'MelodicMinor', label: 'Melodic Minor' },
			{ name: 'DorianFlat2', label: 'Dorian b2' },
			{ name: 'LydianAug', label: 'Lydian Aug' },
			{ name: 'LydianDominant', label: 'Lydian Dom' },
			{ name: 'MixolydianFlat6', label: 'Mixolydian b6' },
			{ name: 'LocrianNat2', label: 'Locrian Nat 2' },
			{ name: 'SuperLocrian', label: 'Super Locrian' }
		]
	},
	{
		family: 'HarmonicMajor',
		label: 'Harmonic Major',
		modes: [
			{ name: 'HarmonicMajor', label: 'Harmonic Major' },
			{ name: 'DorianFlat5', label: 'Dorian b5' },
			{ name: 'PhrygianFlat4', label: 'Phrygian b4' },
			{ name: 'LydianFlat3', label: 'Lydian b3' },
			{ name: 'MixolydianFlat2', label: 'Mixolydian b2' },
			{ name: 'LydianAugSharp2', label: 'Lydian Aug #2' },
			{ name: 'LocrianDoubleFlat7', label: 'Locrian bb7' }
		]
	},
	{
		family: 'DoubleHarmonic',
		label: 'Double Harmonic',
		modes: [
			{ name: 'DoubleHarmonic', label: 'Double Harmonic' },
			{ name: 'LydianSharp2Sharp6', label: 'Lydian #2 #6' },
			{ name: 'Ultraphrygian', label: 'Ultraphrygian' },
			{ name: 'HungarianMinor', label: 'Hungarian Minor' },
			{ name: 'Oriental', label: 'Oriental' },
			{ name: 'IonianSharp2Sharp5', label: 'Ionian #2 #5' },
			{ name: 'LocrianDoubleFlat3DoubleFlat7', label: 'Locrian bb3 bb7' }
		]
	},
	{
		family: 'Pentatonic',
		label: 'Pentatonic',
		modes: [
			{ name: 'MajorPentatonic', label: 'Major Pentatonic' },
			{ name: 'MinorPentatonic', label: 'Minor Pentatonic' },
			{ name: 'Hirajoshi', label: 'Hirajoshi' },
			{ name: 'InSen', label: 'In Sen' },
			{ name: 'Iwato', label: 'Iwato' },
			{ name: 'Yo', label: 'Yo' },
			{ name: 'Kumoi', label: 'Kumoi' },
			{ name: 'Pelog', label: 'Pelog' }
		]
	},
	{
		family: 'Blues',
		label: 'Blues & Bebop',
		modes: [
			{ name: 'MinorBlues', label: 'Minor Blues' },
			{ name: 'MajorBlues', label: 'Major Blues' },
			{ name: 'BebopDominant', label: 'Bebop Dominant' }
		]
	},
	{
		family: 'Symmetric',
		label: 'Symmetric',
		modes: [
			{ name: 'WholeTone', label: 'Whole Tone' },
			{ name: 'DiminishedWholeHalf', label: 'Diminished (WH)' },
			{ name: 'DiminishedHalfWhole', label: 'Diminished (HW)' },
			{ name: 'AugmentedHex', label: 'Augmented' }
		]
	},
	{
		family: 'World',
		label: 'World Scales',
		modes: [
			{ name: 'Enigmatic', label: 'Enigmatic' },
			{ name: 'NeapolitanMinor', label: 'Neapolitan Minor' },
			{ name: 'NeapolitanMajor', label: 'Neapolitan Major' },
			{ name: 'Persian', label: 'Persian' },
			{ name: 'HungarianMajor', label: 'Hungarian Major' }
		]
	},
	{
		family: 'BarryHarris',
		label: 'Barry Harris',
		modes: [
			{ name: 'BHMajor6thDim', label: 'Major 6th Dim' },
			{ name: 'BHMinor6thDim', label: 'Minor 6th Dim' }
		]
	}
];

export const OCTAVE_MODES: { name: OctaveModeName; label: string }[] = [
	{ name: 'None', label: 'None' },
	{ name: 'Spread', label: 'Spread' },
	{ name: 'BassTrebleSplit', label: 'Split' },
	{ name: 'Mirror', label: 'Mirror' }
];

export const VOICE_LEADING_STYLES: { name: VoiceLeadingStyleName; label: string }[] = [
	{ name: 'Free', label: 'Free' },
	{ name: 'Palestrina', label: 'Palestrina' },
	{ name: 'BachChorale', label: 'Bach' },
	{ name: 'Jazz', label: 'Jazz' }
];

// === Scale Intervals (mirrors Rust ScaleMode::intervals) ===

const SCALE_INTERVALS: Record<ScaleModeName, number[]> = {
	// Diatonic modes (7 notes)
	Ionian: [0, 2, 4, 5, 7, 9, 11],
	Dorian: [0, 2, 3, 5, 7, 9, 10],
	Phrygian: [0, 1, 3, 5, 7, 8, 10],
	Lydian: [0, 2, 4, 6, 7, 9, 11],
	Mixolydian: [0, 2, 4, 5, 7, 9, 10],
	Aeolian: [0, 2, 3, 5, 7, 8, 10],
	Locrian: [0, 1, 3, 5, 6, 8, 10],
	// Harmonic minor modes (7 notes)
	HarmonicMinor: [0, 2, 3, 5, 7, 8, 11],
	LocrianNat6: [0, 1, 3, 5, 6, 9, 10],
	IonianAug: [0, 2, 4, 5, 8, 9, 11],
	DorianSharp4: [0, 2, 3, 6, 7, 9, 10],
	PhrygianDominant: [0, 1, 4, 5, 7, 8, 10],
	LydianSharp2: [0, 3, 4, 6, 7, 9, 11],
	SuperLocrianDim: [0, 1, 3, 4, 6, 8, 9],
	// Melodic minor modes (7 notes)
	MelodicMinor: [0, 2, 3, 5, 7, 9, 11],
	DorianFlat2: [0, 1, 3, 5, 7, 9, 10],
	LydianAug: [0, 2, 4, 6, 8, 9, 11],
	LydianDominant: [0, 2, 4, 6, 7, 9, 10],
	MixolydianFlat6: [0, 2, 4, 5, 7, 8, 10],
	LocrianNat2: [0, 2, 3, 5, 6, 8, 10],
	SuperLocrian: [0, 1, 3, 4, 6, 8, 10],
	// Harmonic major modes (7 notes)
	HarmonicMajor: [0, 2, 4, 5, 7, 8, 11],
	DorianFlat5: [0, 2, 3, 5, 6, 9, 10],
	PhrygianFlat4: [0, 1, 3, 4, 7, 9, 10],
	LydianFlat3: [0, 2, 3, 6, 7, 9, 11],
	MixolydianFlat2: [0, 1, 4, 5, 7, 9, 10],
	LydianAugSharp2: [0, 3, 4, 6, 8, 9, 11],
	LocrianDoubleFlat7: [0, 1, 3, 5, 6, 8, 9],
	// Double harmonic modes (7 notes)
	DoubleHarmonic: [0, 1, 4, 5, 7, 8, 11],
	LydianSharp2Sharp6: [0, 3, 4, 6, 7, 10, 11],
	Ultraphrygian: [0, 1, 3, 4, 7, 8, 9],
	HungarianMinor: [0, 2, 3, 6, 7, 8, 11],
	Oriental: [0, 1, 4, 5, 6, 9, 10],
	IonianSharp2Sharp5: [0, 3, 4, 5, 8, 9, 11],
	LocrianDoubleFlat3DoubleFlat7: [0, 1, 2, 5, 6, 8, 9],
	// Pentatonic (5 notes)
	MajorPentatonic: [0, 2, 4, 7, 9],
	MinorPentatonic: [0, 3, 5, 7, 10],
	Hirajoshi: [0, 2, 3, 7, 8],
	InSen: [0, 1, 5, 7, 8],
	Iwato: [0, 1, 5, 6, 10],
	Yo: [0, 2, 5, 7, 9],
	Kumoi: [0, 2, 3, 7, 9],
	Pelog: [0, 1, 3, 7, 8],
	// Blues & Bebop
	MinorBlues: [0, 3, 5, 6, 7, 10],
	MajorBlues: [0, 2, 3, 4, 7, 9],
	BebopDominant: [0, 2, 4, 5, 7, 9, 10, 11],
	// Symmetric
	WholeTone: [0, 2, 4, 6, 8, 10],
	DiminishedWholeHalf: [0, 2, 3, 5, 6, 8, 9, 11],
	DiminishedHalfWhole: [0, 1, 3, 4, 6, 7, 9, 10],
	AugmentedHex: [0, 3, 4, 7, 8, 11],
	// World scales (7 notes)
	Enigmatic: [0, 1, 4, 6, 8, 10, 11],
	NeapolitanMinor: [0, 1, 3, 5, 7, 8, 11],
	NeapolitanMajor: [0, 1, 3, 5, 7, 9, 11],
	Persian: [0, 1, 4, 5, 6, 8, 11],
	HungarianMajor: [0, 3, 4, 6, 7, 9, 10],
	// Barry Harris 6th Diminished (8 notes)
	BHMajor6thDim: [0, 2, 4, 5, 7, 8, 9, 11],
	BHMinor6thDim: [0, 2, 3, 5, 7, 8, 9, 11]
};

const KEY_TO_PITCH_CLASS: Record<KeyName, number> = {
	C: 0, 'C#': 1, D: 2, 'D#': 3, E: 4, F: 5,
	'F#': 6, G: 7, 'G#': 8, A: 9, 'A#': 10, B: 11
};

/** Compute all MIDI note numbers (0-127) that belong to the given key + scale. */
function computeScaleNotes(key: KeyName, scaleMode: ScaleModeName): number[] {
	const tonic = KEY_TO_PITCH_CLASS[key];
	const intervals = SCALE_INTERVALS[scaleMode];
	const pitchClasses = new Set(intervals.map((i) => (tonic + i) % 12));
	const notes: number[] = [];
	for (let midi = 0; midi <= 127; midi++) {
		if (pitchClasses.has(midi % 12)) notes.push(midi);
	}
	return notes;
}

// === Settings Persistence ===

const SETTINGS_KEY = 'contrapunk-settings';
// Version bumped from 1 → 2 when counterpointSpecies / counterpointStrictness
// fields were added. Older payloads fall back to defaults on migration.
const SETTINGS_VERSION = 2;

interface PersistedSettings {
	version: number;
	key: KeyName;
	mode: HarmonyModeName;
	scaleMode: ScaleModeName;
	octaveMode: OctaveModeName;
	voiceLeadingEnabled: boolean;
	voiceLeadingStyle: VoiceLeadingStyleName;
	interchangeEnabled: boolean;
	interchangeRange: number;
	voicePosition: number;
	voiceCount: number;
	detuneCents: number;
	counterpointSpecies: CounterpointSpeciesName;
	counterpointStrictness: CounterpointStrictnessName;
}

const SETTINGS_DEFAULTS: PersistedSettings = {
	version: SETTINGS_VERSION,
	key: 'C',
	mode: 'PassThrough',
	scaleMode: 'Ionian',
	octaveMode: 'None',
	voiceLeadingEnabled: false,
	voiceLeadingStyle: 'Free',
	interchangeEnabled: false,
	interchangeRange: 3,
	voicePosition: 0,
	voiceCount: 2,
	detuneCents: 0,
	counterpointSpecies: 'Species1',
	counterpointStrictness: 'Strict'
};

// Enum validation sets
const VALID_KEYS = new Set(ALL_KEYS);
const VALID_MODES = new Set(ALL_MODES.map((m) => m.name));
const VALID_SCALE_MODES = new Set(SCALE_FAMILIES.flatMap((f) => f.modes.map((m) => m.name)));
const VALID_OCTAVE_MODES = new Set(OCTAVE_MODES.map((m) => m.name));
const VALID_VL_STYLES = new Set(VOICE_LEADING_STYLES.map((s) => s.name));
const VALID_CP_SPECIES = new Set<CounterpointSpeciesName>(
	COUNTERPOINT_SPECIES.map((s) => s.name)
);
const VALID_CP_STRICTNESS = new Set<CounterpointStrictnessName>(
	COUNTERPOINT_STRICTNESS.map((s) => s.name)
);

function loadSettings(): PersistedSettings | null {
	try {
		const raw = localStorage.getItem(SETTINGS_KEY);
		if (!raw) return null;
		const parsed = JSON.parse(raw);

		// Schema migration: discard if version mismatch or missing
		if (!parsed || parsed.version !== SETTINGS_VERSION) {
			localStorage.removeItem(SETTINGS_KEY);
			return null;
		}

		// Validate each field, falling back to defaults for invalid values
		return {
			version: SETTINGS_VERSION,
			key: VALID_KEYS.has(parsed.key) ? parsed.key : SETTINGS_DEFAULTS.key,
			mode: VALID_MODES.has(parsed.mode) ? parsed.mode : SETTINGS_DEFAULTS.mode,
			scaleMode: VALID_SCALE_MODES.has(parsed.scaleMode)
				? parsed.scaleMode
				: SETTINGS_DEFAULTS.scaleMode,
			octaveMode: VALID_OCTAVE_MODES.has(parsed.octaveMode)
				? parsed.octaveMode
				: SETTINGS_DEFAULTS.octaveMode,
			voiceLeadingEnabled:
				typeof parsed.voiceLeadingEnabled === 'boolean'
					? parsed.voiceLeadingEnabled
					: SETTINGS_DEFAULTS.voiceLeadingEnabled,
			voiceLeadingStyle: VALID_VL_STYLES.has(parsed.voiceLeadingStyle)
				? parsed.voiceLeadingStyle
				: SETTINGS_DEFAULTS.voiceLeadingStyle,
			interchangeEnabled:
				typeof parsed.interchangeEnabled === 'boolean'
					? parsed.interchangeEnabled
					: SETTINGS_DEFAULTS.interchangeEnabled,
			interchangeRange:
				typeof parsed.interchangeRange === 'number' &&
				parsed.interchangeRange >= 1 &&
				parsed.interchangeRange <= 5
					? parsed.interchangeRange
					: SETTINGS_DEFAULTS.interchangeRange,
			voicePosition:
				typeof parsed.voicePosition === 'number' && parsed.voicePosition >= 0
					? parsed.voicePosition
					: SETTINGS_DEFAULTS.voicePosition,
			voiceCount:
				typeof parsed.voiceCount === 'number' &&
				parsed.voiceCount >= 1 &&
				parsed.voiceCount <= 8
					? parsed.voiceCount
					: SETTINGS_DEFAULTS.voiceCount,
			detuneCents:
				typeof parsed.detuneCents === 'number' &&
				parsed.detuneCents >= -100 &&
				parsed.detuneCents <= 100
					? parsed.detuneCents
					: SETTINGS_DEFAULTS.detuneCents,
			counterpointSpecies: VALID_CP_SPECIES.has(parsed.counterpointSpecies)
				? parsed.counterpointSpecies
				: SETTINGS_DEFAULTS.counterpointSpecies,
			counterpointStrictness: VALID_CP_STRICTNESS.has(parsed.counterpointStrictness)
				? parsed.counterpointStrictness
				: SETTINGS_DEFAULTS.counterpointStrictness
		};
	} catch {
		localStorage.removeItem(SETTINGS_KEY);
		return null;
	}
}

function saveSettings(s: Omit<PersistedSettings, 'version'>) {
	try {
		localStorage.setItem(SETTINGS_KEY, JSON.stringify({ ...s, version: SETTINGS_VERSION }));
	} catch {
		// localStorage full or unavailable — silently ignore
	}
}

// === Engine Store (Svelte 5 runes) ===

class EngineStore {
	// -- Harmony configuration --
	key = $state<KeyName>('C');
	mode = $state<HarmonyModeName>('PassThrough');
	modeNumber = $state(1);
	scaleMode = $state<ScaleModeName>('Ionian');
	octaveMode = $state<OctaveModeName>('None');

	// -- Voice leading --
	voiceLeadingEnabled = $state(false);
	voiceLeadingStyle = $state<VoiceLeadingStyleName>('Free');

	// -- Modal interchange --
	interchangeEnabled = $state(false);
	interchangeRange = $state(3);

	// -- Voice position --
	voicePosition = $state(0);
	voiceCount = $state(2);

	// -- Auto-key detection --
	autoKey = $state(false);

	// -- Counterpoint species / strictness (active when mode === 'StrictCounterpoint') --
	counterpointSpecies = $state<CounterpointSpeciesName>('Species1');
	counterpointStrictness = $state<CounterpointStrictnessName>('Strict');

	// -- Detune --
	detuneCents = $state(0);

	// -- Transport --
	isRunning = $state(false);

	// -- Real-time note state (updated by adapter events) --
	inputNotes = $state<number[]>([]);
	harmonyNotes = $state<number[]>([]);
	borrowedNotes = $state<number[]>([]);
	generatorNotes = $state<number[]>([]);
	inScaleNotes = $derived(computeScaleNotes(this.key, this.scaleMode));

	// -- Display --
	chordName = $state('');
	lastBorrowedFrom = $state('');

	// -- Internal --
	private unsubNotes: (() => void) | null = null;

	/** Persist current config settings to localStorage. */
	private persist() {
		saveSettings({
			key: this.key,
			mode: this.mode,
			scaleMode: this.scaleMode,
			octaveMode: this.octaveMode,
			voiceLeadingEnabled: this.voiceLeadingEnabled,
			voiceLeadingStyle: this.voiceLeadingStyle,
			interchangeEnabled: this.interchangeEnabled,
			interchangeRange: this.interchangeRange,
			voicePosition: this.voicePosition,
			voiceCount: this.voiceCount,
			detuneCents: this.detuneCents,
			counterpointSpecies: this.counterpointSpecies,
			counterpointStrictness: this.counterpointStrictness
		});
	}

	/**
	 * Restore saved settings from localStorage and apply to backend.
	 * Call after adapter.init() and syncFromBackend().
	 */
	async restoreSettings() {
		const saved = loadSettings();
		if (!saved) return;

		// Apply each setting independently so one failure doesn't block the rest
		const ops: [string, () => Promise<void>][] = [
			['key', () => adapter.setKey(saved.key)],
			['mode', () => adapter.setMode(saved.mode)],
			['scaleMode', () => adapter.setScaleMode(saved.scaleMode)],
			['octaveMode', () => adapter.setOctaveMode(saved.octaveMode)],
			[
				'voiceLeading',
				() => adapter.setVoiceLeading(saved.voiceLeadingEnabled, saved.voiceLeadingStyle)
			],
			[
				'interchange',
				() => adapter.setInterchange(saved.interchangeEnabled, saved.interchangeRange)
			],
			['voicePosition', () => adapter.setVoicePosition(saved.voicePosition)],
			['voiceCount', () => adapter.setVoiceCount(saved.voiceCount)],
			['detune', () => { adapter.setDetune(saved.detuneCents); return Promise.resolve(); }],
			[
				'counterpointSpecies',
				() => adapter.setCounterpointSpecies(saved.counterpointSpecies)
			],
			[
				'counterpointStrictness',
				() => adapter.setCounterpointStrictness(saved.counterpointStrictness)
			]
		];

		for (const [name, op] of ops) {
			try {
				await op();
			} catch (e) {
				console.warn(`[contrapunk] Failed to restore ${name}:`, e);
			}
		}

		// Sync back to pick up any clamped/validated values from the backend
		try {
			await this.syncFromBackend();
		} catch (e) {
			console.warn('[contrapunk] Failed to sync after restore:', e);
		}
	}

	// === Adapter-wired actions (optimistic update with rollback) ===

	async setKey(newKey: KeyName) {
		const prev = this.key;
		this.key = newKey;
		try {
			await adapter.setKey(newKey);
			this.persist();
		} catch (e) {
			this.key = prev;
			throw e;
		}
	}

	async setMode(newMode: HarmonyModeName) {
		const prev = this.mode;
		this.mode = newMode;
		try {
			await adapter.setMode(newMode);
			this.persist();
		} catch (e) {
			this.mode = prev;
			throw e;
		}
	}

	async setScaleMode(newMode: ScaleModeName) {
		const prev = this.scaleMode;
		this.scaleMode = newMode;
		try {
			await adapter.setScaleMode(newMode);
			this.persist();
		} catch (e) {
			this.scaleMode = prev;
			throw e;
		}
	}

	async setOctaveMode(newMode: OctaveModeName) {
		const prev = this.octaveMode;
		this.octaveMode = newMode;
		try {
			await adapter.setOctaveMode(newMode);
			this.persist();
		} catch (e) {
			this.octaveMode = prev;
			throw e;
		}
	}

	async setVoiceLeading(enabled: boolean, style?: VoiceLeadingStyleName) {
		const prevEnabled = this.voiceLeadingEnabled;
		const prevStyle = this.voiceLeadingStyle;
		this.voiceLeadingEnabled = enabled;
		if (style) this.voiceLeadingStyle = style;
		try {
			await adapter.setVoiceLeading(enabled, style ?? this.voiceLeadingStyle);
			this.persist();
		} catch (e) {
			this.voiceLeadingEnabled = prevEnabled;
			this.voiceLeadingStyle = prevStyle;
			throw e;
		}
	}

	async setInterchange(enabled: boolean, range?: number) {
		const prevEnabled = this.interchangeEnabled;
		const prevRange = this.interchangeRange;
		this.interchangeEnabled = enabled;
		if (range !== undefined) this.interchangeRange = range;
		try {
			await adapter.setInterchange(enabled, range ?? this.interchangeRange);
			this.persist();
		} catch (e) {
			this.interchangeEnabled = prevEnabled;
			this.interchangeRange = prevRange;
			throw e;
		}
	}

	async setVoicePosition(position: number) {
		const prev = this.voicePosition;
		this.voicePosition = position;
		try {
			await adapter.setVoicePosition(position);
			this.persist();
		} catch (e) {
			this.voicePosition = prev;
			throw e;
		}
	}

	async setVoiceCount(count: number) {
		const prev = this.voiceCount;
		this.voiceCount = count;
		try {
			await adapter.setVoiceCount(count);
			this.persist();
		} catch (e) {
			this.voiceCount = prev;
			throw e;
		}
	}

	async setAutoKey(enabled: boolean) {
		this.autoKey = enabled;
		try {
			await adapter.setAutoKey(enabled);
		} catch (e) {
			this.autoKey = !enabled;
			throw e;
		}
	}

	async setCounterpointSpecies(species: CounterpointSpeciesName) {
		const prev = this.counterpointSpecies;
		this.counterpointSpecies = species;
		try {
			await adapter.setCounterpointSpecies(species);
			this.persist();
		} catch (e) {
			this.counterpointSpecies = prev;
			throw e;
		}
	}

	async setCounterpointStrictness(strictness: CounterpointStrictnessName) {
		const prev = this.counterpointStrictness;
		this.counterpointStrictness = strictness;
		try {
			await adapter.setCounterpointStrictness(strictness);
			this.persist();
		} catch (e) {
			this.counterpointStrictness = prev;
			throw e;
		}
	}

	setDetune(cents: number) {
		this.detuneCents = cents;
		adapter.setDetune(cents);
		this.persist();
	}

	/**
	 * Start MIDI routing from the given input to the given outputs.
	 */
	async start(inputIdx: number, outputIndices: number[]) {
		await adapter.startRouting(inputIdx, outputIndices);
		this.isRunning = true;
		this.startNoteUpdates();
	}

	/**
	 * Stop MIDI routing and clear note state.
	 */
	async stop() {
		await adapter.stopRouting();
		this.isRunning = false;
		this.stopNoteUpdates();
		this.inputNotes = [];
		this.harmonyNotes = [];
		this.borrowedNotes = [];
		this.chordName = '';
		this.lastBorrowedFrom = '';
	}

	/** Toggle start/stop (requires MIDI store state for device indices). */
	toggle() {
		if (this.isRunning) {
			this.stop();
		}
		// Start requires device indices; callers should use start() directly
	}

	/**
	 * Pull full engine state from the backend and update local reactive state.
	 * Call after init or preset load.
	 */
	async syncFromBackend() {
		const state = await adapter.getEngineState();
		this.key = state.key as KeyName;
		this.mode = state.mode as HarmonyModeName;
		this.modeNumber = state.modeNumber;
		this.scaleMode = state.scaleMode as ScaleModeName;
		this.octaveMode = state.octaveMode as OctaveModeName;
		this.voiceLeadingEnabled = state.voiceLeadingEnabled;
		this.voiceLeadingStyle = state.voiceLeadingStyle as VoiceLeadingStyleName;
		this.interchangeEnabled = state.interchangeEnabled;
		this.interchangeRange = state.interchangeRange;
		this.voicePosition = state.voicePosition;
		this.voiceCount = state.voiceCount;
		this.autoKey = state.autoKey;
		this.isRunning = state.isRunning;
		if (VALID_CP_SPECIES.has(state.counterpointSpecies as CounterpointSpeciesName)) {
			this.counterpointSpecies = state.counterpointSpecies as CounterpointSpeciesName;
		}
		if (
			VALID_CP_STRICTNESS.has(
				state.counterpointStrictness as CounterpointStrictnessName
			)
		) {
			this.counterpointStrictness =
				state.counterpointStrictness as CounterpointStrictnessName;
		}
	}

	/**
	 * Subscribe to real-time note update events from the adapter.
	 * Only assigns when values actually change to avoid Svelte re-renders.
	 */
	startNoteUpdates() {
		if (this.unsubNotes) return;

		this.unsubNotes = adapter.onNoteUpdate((state: NoteState) => {
			if (!sameNotes(this.inputNotes, state.inputNotes))
				this.inputNotes = state.inputNotes;
			if (!sameNotes(this.harmonyNotes, state.harmonyNotes))
				this.harmonyNotes = state.harmonyNotes;
			if (!sameNotes(this.borrowedNotes, state.borrowedNotes))
				this.borrowedNotes = state.borrowedNotes;
			if (this.chordName !== state.chordName)
				this.chordName = state.chordName;
			if (this.lastBorrowedFrom !== state.lastBorrowedFrom)
				this.lastBorrowedFrom = state.lastBorrowedFrom;
			// Update key display when auto-key detection changes the key
			if (this.autoKey && state.currentKey && this.key !== state.currentKey)
				this.key = state.currentKey as KeyName;
		});
	}

	/**
	 * Unsubscribe from note update events.
	 */
	stopNoteUpdates() {
		this.unsubNotes?.();
		this.unsubNotes = null;
	}
}

export const engine = new EngineStore();
