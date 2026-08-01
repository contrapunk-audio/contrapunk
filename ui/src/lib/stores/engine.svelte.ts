/**
 * Engine Store — Reactive Harmony Engine State (Svelte 5 Runes)
 *
 * Tracks the harmony engine configuration and real-time note state.
 * All mutations call the platform adapter and update local state
 * optimistically (set locally, then call backend; revert on error).
 */

import { adapter } from '$lib/adapter';
import type { HarmonicLimit, NoteState, TuningStyle } from '$lib/adapter';
import { phrase } from '$lib/stores/phrase.svelte';

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
	| 'BachChorale'
	| 'ExplicitIntervals';

export interface ExplicitIntervalMapConfig {
	degreeOffsets: number[][];
	fallbackOffsets: number[];
}

export const DEFAULT_EXPLICIT_INTERVAL_MAP: ExplicitIntervalMapConfig = {
	degreeOffsets: Array.from({ length: 7 }, () => [7]),
	fallbackOffsets: [7]
};

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

export type ImitativeFormName = 'strict_canon' | 'free_imitation';

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

/** Display labels for keys — uses Unicode music-notation glyphs (♯, ♭)
 *  and flat spellings where musicians expect them. The dict keys remain
 *  the ASCII wire-format identifiers ('C#', 'D#') because those have to
 *  match the Rust engine's KeyName enum on the wire; only the values
 *  change for display. */
export const KEY_DISPLAY: Record<KeyName, string> = {
	'C': 'C',
	'C#': 'C♯/D♭',
	'D': 'D',
	'D#': 'E♭',
	'E': 'E',
	'F': 'F',
	'F#': 'F♯/G♭',
	'G': 'G',
	'G#': 'A♭',
	'A': 'A',
	'A#': 'B♭',
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
		label: 'Counterpoint',
		shortLabel: 'Cpt',
		tooltip:
			'Note-against-note voice leading with scoring. No parallel 5ths/octaves, prefers contrary/stepwise motion.'
	},
	{
		name: 'ExplicitIntervals',
		label: 'Explicit Interval Map',
		shortLabel: 'Map',
		tooltip: 'Direct anchor-relative semitone stacks selected by the played note’s scale degree.'
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
// Version 8: defaults switched to Palestrina voice-leading +
// Octave Spread to match the Renaissance counterpoint style of
// the StrictCounterpoint mode + Pure Counterpoint default form.
const SETTINGS_VERSION = 10;

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
	tuningStyle: TuningStyle;
	tuningDepth: number;
	harmonicLimit: HarmonicLimit;
	counterpointSpecies: CounterpointSpeciesName;
	counterpointStrictness: CounterpointStrictnessName;
	explicitIntervalMap: ExplicitIntervalMapConfig;
	// Companion + Canon (#3) — persisted in version 3+.
	companionEnabled: boolean;
	canonEnabled: boolean;
	imitativeForm: ImitativeFormName;
	canonVoices: Array<{
		delay_beats: number;
		transpose_degrees: number;
		time_ratio: number;
		harmony_mode?: string | null | undefined;
		reference_voice?: number | null | undefined;
		voice_count?: number | null | undefined;
		voice_position?: number | null | undefined;
		voice_leading_enabled?: boolean | null | undefined;
		voice_leading_style?: string | null | undefined;
		octave_mode?: string | null | undefined;
		counterpoint_species?: string | null | undefined;
		counterpoint_strictness?: string | null | undefined;
		hold_mode?:
			| null
			| { kind: 'cancel' }
			| { kind: 'near_future'; tail_beats: number }
			| { kind: 'phrase_end' }
			| { kind: 'forever' };
		preset_id?: string | null | undefined;
	}>;
	// HoldMode (#11) — persisted from v10. Global default for the
	// Companion; lane-level overrides for canon + counterpoint.
	// Per-voice overrides ride along inside canonVoices.
	companionHoldMode:
		| { kind: 'cancel' }
		| { kind: 'near_future'; tail_beats: number }
		| { kind: 'phrase_end' }
		| { kind: 'forever' };
	canonLaneHoldMode:
		| null
		| { kind: 'cancel' }
		| { kind: 'near_future'; tail_beats: number }
		| { kind: 'phrase_end' }
		| { kind: 'forever' };
	counterpointLaneHoldMode:
		| null
		| { kind: 'cancel' }
		| { kind: 'near_future'; tail_beats: number }
		| { kind: 'phrase_end' }
		| { kind: 'forever' };
}

const SETTINGS_DEFAULTS: PersistedSettings = {
	version: SETTINGS_VERSION,
	key: 'C',
	// Counterpoint mode for the global engine — every note the
	// player or canon voices route through gets harmonized with
	// proper species-counterpoint rules (no parallel 5ths/8ves,
	// stepwise motion preferred).
	mode: 'StrictCounterpoint',
	scaleMode: 'Ionian',
	// Octave Spread — distributes the 4 generated voices across
	// octaves above the bass so the chorale spans a real range
	// rather than clustering tight.
	octaveMode: 'Spread',
	// Palestrina voice-leading — Renaissance species rules:
	// stepwise lines, prepared dissonances, contrary motion
	// preferred over parallel. The classical fit alongside
	// StrictCounterpoint mode.
	voiceLeadingEnabled: true,
	voiceLeadingStyle: 'Palestrina',
	// Modal interchange ON so out-of-scale input borrows from
	// parallel modes instead of dropping back to bare unison.
	interchangeEnabled: true,
	interchangeRange: 3,
	// 4-voice SATB chorale with the user playing the Bass line.
	// voicePosition is the index of the user's voice in the stack:
	// 0 = Soprano, 1 = Alto, 2 = Tenor, 3 = Bass — so the player is
	// at the bottom and the engine generates SAT above.
	voiceCount: 4,
	voicePosition: 3,
	detuneCents: 0,
	tuningStyle: 'standard',
	tuningDepth: 0.6,
	harmonicLimit: 'five',
	counterpointSpecies: 'Species1',
	counterpointStrictness: 'Strict',
	explicitIntervalMap: {
		degreeOffsets: Array.from({ length: 7 }, () => [7]),
		fallbackOffsets: [7]
	},
	// Fresh-install defaults: Companion + Canon ON with the Pure
	// Counterpoint form — 4 voices, every voice in StrictCounterpoint
	// mode, cascading in 3rds above the player's bass line. The user
	// at beat 0 + 4 SAT voices through species-counterpoint rules =
	// a 4-part chorale texture from the first note. (Other harmony
	// modes are one click away in the FORMS rail.)
	companionEnabled: true,
	canonEnabled: true,
	// The existing cascaded/stateful configuration is broader than a
	// strict canon, so describe it honestly until the user opts in.
	imitativeForm: 'free_imitation',
	// Stateful cascade: V1 references the player, V2 references V1,
	// V3 references V2. Each mini-engine inherits the prior voice's
	// CounterpointState (interval history + harmony pitch buffer +
	// melodic contour) before generating, so V2's rule scoring is
	// aware of V1's emitted pitch and avoids parallels with it.
	canonVoices: [
		{
			delay_beats: 1,
			transpose_degrees: 2,
			time_ratio: 1.0,
			harmony_mode: 'StrictCounterpoint',
			reference_voice: null
		},
		{
			delay_beats: 2,
			transpose_degrees: 2,
			time_ratio: 1.0,
			harmony_mode: 'StrictCounterpoint',
			reference_voice: 0
		},
		{
			delay_beats: 3,
			transpose_degrees: 2,
			time_ratio: 1.0,
			harmony_mode: 'StrictCounterpoint',
			reference_voice: 1
		}
	],
	// HoldMode (#11) defaults — global is the canon-preserving
	// NearFuture(1.0) value; lane-level overrides start null so they
	// inherit the global. Per-voice overrides ride along in canonVoices.
	companionHoldMode: { kind: 'near_future', tail_beats: 1.0 },
	canonLaneHoldMode: null,
	counterpointLaneHoldMode: null
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

function cloneExplicitIntervalMap(config: ExplicitIntervalMapConfig): ExplicitIntervalMapConfig {
	return {
		degreeOffsets: config.degreeOffsets.map((offsets) => [...offsets]),
		fallbackOffsets: [...config.fallbackOffsets]
	};
}

function parseExplicitIntervalMap(value: unknown): ExplicitIntervalMapConfig {
	if (typeof value !== 'object' || value === null) {
		return cloneExplicitIntervalMap(DEFAULT_EXPLICIT_INTERVAL_MAP);
	}
	const candidate = value as { degreeOffsets?: unknown; fallbackOffsets?: unknown };
	const parseOffsets = (offsets: unknown): number[] | null => {
		if (!Array.isArray(offsets) || offsets.length > 7) return null;
		const parsed = offsets.map(Number);
		if (
			parsed.some(
				(offset, index) =>
					!Number.isInteger(offset) ||
					offset === 0 ||
					offset < -48 ||
					offset > 48 ||
					parsed.indexOf(offset) !== index
			)
		) return null;
		return parsed;
	};
	if (!Array.isArray(candidate.degreeOffsets) || candidate.degreeOffsets.length !== 7) {
		return cloneExplicitIntervalMap(DEFAULT_EXPLICIT_INTERVAL_MAP);
	}
	const degreeOffsets = candidate.degreeOffsets.map(parseOffsets);
	const fallbackOffsets = parseOffsets(candidate.fallbackOffsets);
	if (degreeOffsets.some((offsets) => offsets === null) || fallbackOffsets === null) {
		return cloneExplicitIntervalMap(DEFAULT_EXPLICIT_INTERVAL_MAP);
	}
	return {
		degreeOffsets: degreeOffsets as number[][],
		fallbackOffsets
	};
}

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
			tuningStyle: parsed.tuningStyle === 'pure' ? 'pure' : 'standard',
			tuningDepth:
				typeof parsed.tuningDepth === 'number' &&
				parsed.tuningDepth >= 0 &&
				parsed.tuningDepth <= 1
					? parsed.tuningDepth
					: SETTINGS_DEFAULTS.tuningDepth,
			harmonicLimit: parsed.harmonicLimit === 'seven' ? 'seven' : 'five',
			counterpointSpecies: VALID_CP_SPECIES.has(parsed.counterpointSpecies)
				? parsed.counterpointSpecies
				: SETTINGS_DEFAULTS.counterpointSpecies,
			counterpointStrictness: VALID_CP_STRICTNESS.has(parsed.counterpointStrictness)
				? parsed.counterpointStrictness
				: SETTINGS_DEFAULTS.counterpointStrictness,
			explicitIntervalMap: parseExplicitIntervalMap(parsed.explicitIntervalMap),
			companionEnabled:
				typeof parsed.companionEnabled === 'boolean'
					? parsed.companionEnabled
					: SETTINGS_DEFAULTS.companionEnabled,
			canonEnabled:
				typeof parsed.canonEnabled === 'boolean'
					? parsed.canonEnabled
					: SETTINGS_DEFAULTS.canonEnabled,
			imitativeForm:
				parsed.imitativeForm === 'strict_canon' || parsed.imitativeForm === 'free_imitation'
					? parsed.imitativeForm
					: SETTINGS_DEFAULTS.imitativeForm,
			canonVoices: Array.isArray(parsed.canonVoices)
				? parsed.canonVoices
						.slice(0, 8)
						.filter(
							(v: unknown): v is { delay_beats: number; transpose_degrees: number; time_ratio: number } =>
								typeof v === 'object' &&
								v !== null &&
								typeof (v as Record<string, unknown>).delay_beats === 'number' &&
								typeof (v as Record<string, unknown>).transpose_degrees === 'number'
						)
						.map(
							(v: {
								delay_beats: number;
								transpose_degrees: number;
								time_ratio?: number;
								harmony_mode?: string | null;
								reference_voice?: number | null;
								voice_count?: number | null;
								voice_position?: number | null;
								voice_leading_enabled?: boolean | null;
								voice_leading_style?: string | null;
								octave_mode?: string | null;
								counterpoint_species?: string | null;
								counterpoint_strictness?: string | null;
								hold_mode?: unknown;
								preset_id?: string | null;
							}) => ({
								delay_beats: Math.max(0, Math.min(16, v.delay_beats)),
								transpose_degrees: Math.max(-7, Math.min(7, Math.round(v.transpose_degrees))),
								time_ratio:
									typeof v.time_ratio === 'number'
										? Math.max(0.125, Math.min(8, v.time_ratio))
										: 1.0,
								harmony_mode: typeof v.harmony_mode === 'string' ? v.harmony_mode : null,
								reference_voice:
									typeof v.reference_voice === 'number' ? v.reference_voice : null,
								voice_count:
									typeof v.voice_count === 'number' ? Math.max(1, Math.min(4, Math.round(v.voice_count))) : null,
								voice_position:
									typeof v.voice_position === 'number' ? Math.max(0, Math.min(3, Math.round(v.voice_position))) : null,
								voice_leading_enabled:
									typeof v.voice_leading_enabled === 'boolean' ? v.voice_leading_enabled : null,
								voice_leading_style:
									typeof v.voice_leading_style === 'string' ? v.voice_leading_style : null,
								octave_mode: typeof v.octave_mode === 'string' ? v.octave_mode : null,
								counterpoint_species:
									typeof v.counterpoint_species === 'string' ? v.counterpoint_species : null,
								counterpoint_strictness:
									typeof v.counterpoint_strictness === 'string' ? v.counterpoint_strictness : null,
								hold_mode: parseLaneHoldMode(v.hold_mode),
								preset_id: typeof v.preset_id === 'string' ? v.preset_id : null
							})
						)
				: SETTINGS_DEFAULTS.canonVoices,
			companionHoldMode:
				parseHoldMode(parsed.companionHoldMode) ?? SETTINGS_DEFAULTS.companionHoldMode,
			canonLaneHoldMode: parseLaneHoldMode(parsed.canonLaneHoldMode),
			counterpointLaneHoldMode: parseLaneHoldMode(parsed.counterpointLaneHoldMode)
		};
	} catch {
		localStorage.removeItem(SETTINGS_KEY);
		return null;
	}
}

/** Parse a global HoldMode from persisted JSON. Returns null on
 *  malformed shape; caller falls back to default. */
function parseHoldMode(v: unknown):
	| { kind: 'cancel' }
	| { kind: 'near_future'; tail_beats: number }
	| { kind: 'phrase_end' }
	| { kind: 'forever' }
	| null {
	if (typeof v !== 'object' || v === null) return null;
	const obj = v as { kind?: unknown; tail_beats?: unknown };
	if (obj.kind === 'cancel') return { kind: 'cancel' };
	if (obj.kind === 'phrase_end') return { kind: 'phrase_end' };
	if (obj.kind === 'forever') return { kind: 'forever' };
	if (obj.kind === 'near_future') {
		const tb = typeof obj.tail_beats === 'number' ? obj.tail_beats : 1.0;
		return { kind: 'near_future', tail_beats: Math.max(0, Math.min(32, tb)) };
	}
	return null;
}

/** Lane override variant. null means "inherit Companion global." */
function parseLaneHoldMode(v: unknown):
	| null
	| { kind: 'cancel' }
	| { kind: 'near_future'; tail_beats: number }
	| { kind: 'phrase_end' }
	| { kind: 'forever' } {
	if (v === null || v === undefined) return null;
	return parseHoldMode(v);
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
	octaveIntensity = $state(1.0);

	// -- Voice leading --
	voiceLeadingEnabled = $state(false);
	voiceLeadingStyle = $state<VoiceLeadingStyleName>('Free');

	// -- Modal interchange --
	interchangeEnabled = $state(false);
	interchangeRange = $state(3);

	// -- Voice position --
	voicePosition = $state(0);
	voiceCount = $state(2);

	// -- Native exact-frequency tuning --
	tuningStyle = $state<TuningStyle>('standard');
	tuningDepth = $state(0.6);
	harmonicLimit = $state<HarmonicLimit>('five');

	// -- Auto-key detection --
	autoKey = $state(false);

	// -- Counterpoint species / strictness (active when mode === 'StrictCounterpoint') --
	counterpointSpecies = $state<CounterpointSpeciesName>('Species1');
	counterpointStrictness = $state<CounterpointStrictnessName>('Strict');
	explicitIntervalMap = $state<ExplicitIntervalMapConfig>(
		cloneExplicitIntervalMap(DEFAULT_EXPLICIT_INTERVAL_MAP)
	);

	// -- Detune --
	detuneCents = $state(0);

	// -- Companion / Canon (#3, #91) --
	companionEnabled = $state(false);
	/** Companion's global HoldMode — what lanes do with pending
	 *  emissions when the player releases the seeding NoteOn (#11).
	 *  Default `near_future` with `tail_beats: 1.0` mirrors the Rust
	 *  side default. Lanes / voices can still override per-slot
	 *  (Rust-plumbed but UI for overrides is deferred). */
	companionHoldMode = $state<
		| { kind: 'cancel' }
		| { kind: 'near_future'; tail_beats: number }
		| { kind: 'phrase_end' }
		| { kind: 'forever' }
	>({ kind: 'near_future', tail_beats: 1.0 });
	canonEnabled = $state(false);
	imitativeForm = $state<ImitativeFormName>('free_imitation');
	canonDelayBeats = $state(1.0);
	canonTransposeDegrees = $state(0);
	/** Canon lane HoldMode override (#11). `null` = inherit Companion
	 *  global. Otherwise this lane's own HoldMode takes precedence
	 *  over global at NoteOff resolution time. */
	canonLaneHoldMode = $state<
		| null
		| { kind: 'cancel' }
		| { kind: 'near_future'; tail_beats: number }
		| { kind: 'phrase_end' }
		| { kind: 'forever' }
	>(null);
	/** Counterpoint lane HoldMode override (#11). Same semantics as
	 *  canonLaneHoldMode. */
	counterpointLaneHoldMode = $state<
		| null
		| { kind: 'cancel' }
		| { kind: 'near_future'; tail_beats: number }
		| { kind: 'phrase_end' }
		| { kind: 'forever' }
	>(null);
	/** Multi-voice canon (#3). Each entry independently delays,
	 *  transposes, time-scales (augmentation/diminution), and can
	 *  override the engine's harmony mode for its own stack. Mirrors
	 *  the engine-side `voices` field. `harmony_mode = null` means
	 *  "inherit global mode". */
	canonVoices = $state<
		Array<{
			delay_beats: number;
			transpose_degrees: number;
			time_ratio: number;
			harmony_mode?: string | null;
			reference_voice?: number | null;
			voice_count?: number | null;
			voice_position?: number | null;
			voice_leading_enabled?: boolean | null;
			voice_leading_style?: string | null;
			octave_mode?: string | null;
			counterpoint_species?: string | null;
			counterpoint_strictness?: string | null;
			/** Per-voice HoldMode override (#11). Wire-format object;
			 *  `null`/undefined = inherit from lane / global. Persisted
			 *  via `canonSetVoices` JSON pass-through. */
			hold_mode?:
				| null
				| { kind: 'cancel' }
				| { kind: 'near_future'; tail_beats: number }
				| { kind: 'phrase_end' }
				| { kind: 'forever' };
			/** Voice Library binding. UI-only; the backend never sees
			 *  this. When set, the CompanionPanel's reactivity effect
			 *  re-applies the named preset's fields whenever the preset
			 *  changes in the Voices tab. */
			preset_id?: string | null;
		}>
	>([
		{
			delay_beats: 1.0,
			transpose_degrees: 0,
			time_ratio: 1.0,
			harmony_mode: null,
			reference_voice: null
		}
	]);

	// -- Transport --
	isRunning = $state(false);

	// -- Real-time note state (updated by adapter events) --
	inputNotes = $state<number[]>([]);
	harmonyNotes = $state<number[]>([]);
	borrowedNotes = $state<number[]>([]);
	/** Companion-emitted notes attributed by lane. All surfaces keep
	 *  these separate from generic harmony so visualizations can render
	 *  Canon and Counterpoint with distinct colors. */
	canonNotes = $state<number[]>([]);
	counterpointNotes = $state<number[]>([]);
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
			tuningStyle: this.tuningStyle,
			tuningDepth: this.tuningDepth,
			harmonicLimit: this.harmonicLimit,
			counterpointSpecies: this.counterpointSpecies,
			counterpointStrictness: this.counterpointStrictness,
			explicitIntervalMap: cloneExplicitIntervalMap(this.explicitIntervalMap),
			companionEnabled: this.companionEnabled,
			canonEnabled: this.canonEnabled,
			imitativeForm: this.imitativeForm,
			canonVoices: this.canonVoices.map((v) => ({
				delay_beats: v.delay_beats,
				transpose_degrees: v.transpose_degrees,
				time_ratio: v.time_ratio,
				harmony_mode: v.harmony_mode ?? null,
				reference_voice: v.reference_voice ?? null,
				voice_count: v.voice_count ?? null,
				voice_position: v.voice_position ?? null,
				voice_leading_enabled: v.voice_leading_enabled ?? null,
				voice_leading_style: v.voice_leading_style ?? null,
				octave_mode: v.octave_mode ?? null,
				counterpoint_species: v.counterpoint_species ?? null,
				counterpoint_strictness: v.counterpoint_strictness ?? null,
				hold_mode: v.hold_mode ?? null,
				preset_id: v.preset_id ?? null
			})),
			companionHoldMode: this.companionHoldMode,
			canonLaneHoldMode: this.canonLaneHoldMode,
			counterpointLaneHoldMode: this.counterpointLaneHoldMode
		});
	}

	/** Restore only Companion state on host-owned plugin surfaces.
	 * Opening a plugin editor must never overwrite DAW/session parameters. */
	async restoreCompanionSettings() {
		const saved = loadSettings() ?? SETTINGS_DEFAULTS;
		const ops: Array<() => Promise<void>> = [
			() => adapter.canonSetVoices(saved.canonVoices),
			() => adapter.canonSetEnabled(saved.canonEnabled),
			() => adapter.companionSetEnabled(saved.companionEnabled),
			() => adapter.companionSetGlobalHoldMode(saved.companionHoldMode),
			() => adapter.canonConfigure({ hold_mode: saved.canonLaneHoldMode }),
			() => adapter.counterpointConfigure({ hold_mode: saved.counterpointLaneHoldMode })
		];
		for (const op of ops) {
			try {
				await op();
			} catch (e) {
				console.warn('[contrapunk] Failed to restore plugin ensemble state:', e);
			}
		}
		this.companionEnabled = saved.companionEnabled;
		this.canonEnabled = saved.canonEnabled;
		this.imitativeForm = saved.imitativeForm;
		this.canonVoices = saved.canonVoices;
		this.canonDelayBeats = saved.canonVoices[0]?.delay_beats ?? 1;
		this.canonTransposeDegrees = saved.canonVoices[0]?.transpose_degrees ?? 0;
		this.companionHoldMode = saved.companionHoldMode;
		this.canonLaneHoldMode = saved.canonLaneHoldMode;
		this.counterpointLaneHoldMode = saved.counterpointLaneHoldMode;
	}

	/**
	 * Restore saved settings from localStorage and apply to backend.
	 * Call after adapter.init() and syncFromBackend().
	 *
	 * When no payload is saved (fresh install or schema-version bump),
	 * SETTINGS_DEFAULTS is applied instead so the backend + reactive
	 * store match what we declared as the out-of-box configuration.
	 */
	async restoreSettings() {
		const saved = loadSettings() ?? SETTINGS_DEFAULTS;

		// Apply each setting independently so one failure doesn't block the rest
		const ops: [string, () => Promise<void>][] = [
			['key', () => adapter.setKey(saved.key)],
			[
				'explicitIntervalMap',
				() => adapter.capabilities.intervalMaps
					? adapter.setExplicitIntervalMap(
						saved.explicitIntervalMap.degreeOffsets,
						saved.explicitIntervalMap.fallbackOffsets
					)
					: Promise.resolve()
			],
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
			// Order matters: voiceCount must be set before voicePosition
			// because the backend clamps position to count-1. Sending
			// voicePosition=3 while the Rust-side default count=2 still
			// applies would clamp position to 1 (Alto), leaving the UI
			// store showing Bass while the engine runs as Alto.
			['voiceCount', () => adapter.setVoiceCount(saved.voiceCount)],
			['voicePosition', () => adapter.setVoicePosition(saved.voicePosition)],
			['detune', () => { adapter.setDetune(saved.detuneCents); return Promise.resolve(); }],
			[
				'tuningStyle',
				() => adapter.capabilities.nativeTuning
					? adapter.setTuningStyle(saved.tuningStyle)
					: Promise.resolve()
			],
			[
				'tuningDepth',
				() => adapter.capabilities.nativeTuning
					? adapter.setTuningDepth(saved.tuningDepth)
					: Promise.resolve()
			],
			[
				'harmonicLimit',
				() => adapter.capabilities.nativeTuning
					? adapter.setHarmonicLimit(saved.harmonicLimit)
					: Promise.resolve()
			],
			[
				'counterpointSpecies',
				() => adapter.setCounterpointSpecies(saved.counterpointSpecies)
			],
			[
				'counterpointStrictness',
				() => adapter.setCounterpointStrictness(saved.counterpointStrictness)
			],
			// Companion + Canon state (#3): voices first so the backend
			// has the right voice list before we flip canon enabled.
			['canonVoices', () => adapter.canonSetVoices(saved.canonVoices)],
			['canonEnabled', () => adapter.canonSetEnabled(saved.canonEnabled)],
			['companionEnabled', () => adapter.companionSetEnabled(saved.companionEnabled)],
			// HoldMode (#11) — restore global first, then per-lane
			// overrides. Order matters because the lane override on
			// the Rust side is the wrong field write order — but with
			// the engine-mirror pattern both writes are independent
			// and idempotent, so the order is forward-compatible.
			[
				'companionHoldMode',
				() => adapter.companionSetGlobalHoldMode(saved.companionHoldMode)
			],
			[
				'canonLaneHoldMode',
				() => adapter.canonConfigure({ hold_mode: saved.canonLaneHoldMode })
			],
			[
				'counterpointLaneHoldMode',
				() => adapter.counterpointConfigure({ hold_mode: saved.counterpointLaneHoldMode })
			]
		];

		for (const [name, op] of ops) {
			try {
				await op();
			} catch (e) {
				console.warn(`[contrapunk] Failed to restore ${name}:`, e);
			}
		}

		// Mirror the full restored state into the reactive store so the
		// UI renders the persisted config immediately, without waiting
		// for the syncFromBackend round-trip below. Without this, fresh
		// installs (no localStorage payload) would still display the
		// store's $state initializer values (voiceLeading off, mode
		// PassThrough, etc.) even though SETTINGS_DEFAULTS specifies
		// otherwise and the backend has been updated.
		this.key = saved.key;
		this.mode = saved.mode;
		this.scaleMode = saved.scaleMode;
		this.octaveMode = saved.octaveMode;
		this.voiceLeadingEnabled = saved.voiceLeadingEnabled;
		this.voiceLeadingStyle = saved.voiceLeadingStyle;
		this.interchangeEnabled = saved.interchangeEnabled;
		this.interchangeRange = saved.interchangeRange;
		this.voicePosition = saved.voicePosition;
		this.voiceCount = saved.voiceCount;
		this.detuneCents = saved.detuneCents;
		this.tuningStyle = saved.tuningStyle;
		this.tuningDepth = saved.tuningDepth;
		this.harmonicLimit = saved.harmonicLimit;
		this.counterpointSpecies = saved.counterpointSpecies;
		this.counterpointStrictness = saved.counterpointStrictness;
		this.explicitIntervalMap = cloneExplicitIntervalMap(saved.explicitIntervalMap);
		this.companionEnabled = saved.companionEnabled;
		this.canonEnabled = saved.canonEnabled;
		this.imitativeForm = saved.imitativeForm;
		this.canonVoices = saved.canonVoices;

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

	async setExplicitIntervalMap(config: ExplicitIntervalMapConfig) {
		const next = parseExplicitIntervalMap(config);
		const previous = this.explicitIntervalMap;
		this.explicitIntervalMap = cloneExplicitIntervalMap(next);
		try {
			await adapter.setExplicitIntervalMap(next.degreeOffsets, next.fallbackOffsets);
			this.persist();
		} catch (error) {
			this.explicitIntervalMap = previous;
			throw error;
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

	/** Continuous octave-spread intensity coefficient. Range [0,1].
	 *  Used by the Performance view's Spread knob — knob value passes
	 *  through directly. Combined with setOctaveMode, the engine
	 *  produces a smooth audible morph as the knob sweeps. */
	async setOctaveIntensity(amount: number) {
		const clamped = Math.max(0, Math.min(1, amount));
		this.octaveIntensity = clamped;
		try {
			await adapter.setOctaveIntensity(clamped);
		} catch (e) {
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
			// Engine clamps voice_position to count-1 on shrink (e.g. 4→2 with
			// position=Bass=3 ends up at position 1). Mirror that locally so
			// the "You play" dropdown still matches a valid option.
			if (this.voicePosition >= count) {
				this.voicePosition = Math.max(0, count - 1);
			}
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

	async setTuningStyle(style: TuningStyle) {
		const previous = this.tuningStyle;
		this.tuningStyle = style;
		try {
			await adapter.setTuningStyle(style);
			this.persist();
		} catch (error) {
			this.tuningStyle = previous;
			throw error;
		}
	}

	async setTuningDepth(depth: number) {
		const previous = this.tuningDepth;
		this.tuningDepth = Math.max(0, Math.min(1, depth));
		try {
			await adapter.setTuningDepth(this.tuningDepth);
			this.persist();
		} catch (error) {
			this.tuningDepth = previous;
			throw error;
		}
	}

	async setHarmonicLimit(limit: HarmonicLimit) {
		const previous = this.harmonicLimit;
		this.harmonicLimit = limit;
		try {
			await adapter.setHarmonicLimit(limit);
			this.persist();
		} catch (error) {
			this.harmonicLimit = previous;
			throw error;
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

	async setCompanionEnabled(enabled: boolean) {
		const prev = this.companionEnabled;
		this.companionEnabled = enabled;
		try {
			await adapter.companionSetEnabled(enabled);
			this.persist();
		} catch (e) {
			this.companionEnabled = prev;
			throw e;
		}
	}

	/** Set the Companion's global HoldMode (#11). Pushes through to
	 *  the active adapter (Tauri / WASM / plugin); Rust side resolves
	 *  voice > lane > global on each NoteOff. */
	async setCompanionHoldMode(
		mode:
			| { kind: 'cancel' }
			| { kind: 'near_future'; tail_beats: number }
			| { kind: 'phrase_end' }
			| { kind: 'forever' }
	) {
		const prev = this.companionHoldMode;
		this.companionHoldMode = mode;
		try {
			await adapter.companionSetGlobalHoldMode(mode);
			this.persist();
		} catch (e) {
			this.companionHoldMode = prev;
			throw e;
		}
	}

	/** Canon lane override (#11). Pass `null` to clear and inherit
	 *  the Companion global. Pushes through `canonSetVoices` (the
	 *  configure_canon JSON path includes the lane-level hold_mode). */
	async setCanonLaneHoldMode(
		mode:
			| null
			| { kind: 'cancel' }
			| { kind: 'near_future'; tail_beats: number }
			| { kind: 'phrase_end' }
			| { kind: 'forever' }
	) {
		const prev = this.canonLaneHoldMode;
		this.canonLaneHoldMode = mode;
		try {
			// configure_canon already routes through the lane's
			// deserialize_state which handles top-level `hold_mode`.
			// Sending null clears the override.
			await adapter.canonConfigure({ hold_mode: mode });
			this.persist();
		} catch (e) {
			this.canonLaneHoldMode = prev;
			throw e;
		}
	}

	/** Counterpoint lane override (#11). Same semantics as canon
	 *  lane override. */
	async setCounterpointLaneHoldMode(
		mode:
			| null
			| { kind: 'cancel' }
			| { kind: 'near_future'; tail_beats: number }
			| { kind: 'phrase_end' }
			| { kind: 'forever' }
	) {
		const prev = this.counterpointLaneHoldMode;
		this.counterpointLaneHoldMode = mode;
		try {
			await adapter.counterpointConfigure({ hold_mode: mode });
			this.persist();
		} catch (e) {
			this.counterpointLaneHoldMode = prev;
			throw e;
		}
	}

	async setCanonEnabled(enabled: boolean) {
		const prev = this.canonEnabled;
		this.canonEnabled = enabled;
		try {
			await adapter.canonSetEnabled(enabled);
			this.persist();
		} catch (e) {
			this.canonEnabled = prev;
			throw e;
		}
	}

	async setImitativeForm(form: ImitativeFormName) {
		if (form === this.imitativeForm) return;
		const previousForm = this.imitativeForm;
		const previousVoices = this.canonVoices.map((voice) => ({ ...voice }));
		const previousLaneHold = this.canonLaneHoldMode;
		this.imitativeForm = form;
		try {
			if (form === 'strict_canon') {
				const followers = (this.canonVoices.length ? this.canonVoices : [{ delay_beats: 1, transpose_degrees: 0 }]).map(
					(voice) => ({
						...voice,
						time_ratio: 1,
						harmony_mode: 'PassThrough',
						reference_voice: null,
						voice_count: 1,
						voice_position: 0,
						voice_leading_enabled: false,
						octave_mode: 'None',
						// Strict Canon owns Hold at group level. A per-voice
						// Forever override would silently beat the visible
						// group Cancel/Near/Phrase selection.
						hold_mode: null,
						preset_id: null
					})
				);
				if (!this.companionEnabled) await this.setCompanionEnabled(true);
				await this.setCanonVoices(followers);
				await this.setCanonLaneHoldMode({ kind: 'forever' });
			} else {
				// Clear legacy Strict Canon voice overrides and return
				// Free Imitation to the global Hold default. Per-voice and
				// group choices remain editable after the transition.
				await this.setCanonVoices(
					this.canonVoices.map((voice) => ({ ...voice, hold_mode: null }))
				);
				await this.setCanonLaneHoldMode(null);
			}
			this.persist();
		} catch (error) {
			this.imitativeForm = previousForm;
			this.canonVoices = previousVoices;
			this.canonLaneHoldMode = previousLaneHold;
			throw error;
		}
	}

	async setCanonDelay(beats: number) {
		const prev = this.canonDelayBeats;
		const clamped = Math.max(0, Math.min(8, beats));
		this.canonDelayBeats = clamped;
		try {
			await adapter.canonSetDelay(clamped);
		} catch (e) {
			this.canonDelayBeats = prev;
			throw e;
		}
	}

	async setCanonTranspose(degrees: number) {
		const prev = this.canonTransposeDegrees;
		const clamped = Math.max(-7, Math.min(7, Math.round(degrees)));
		this.canonTransposeDegrees = clamped;
		try {
			await adapter.canonSetTranspose(clamped);
		} catch (e) {
			this.canonTransposeDegrees = prev;
			throw e;
		}
	}

	/** Replace the multi-voice canon configuration. (#3) */
	async setCanonVoices(
		voices: Array<{
			delay_beats: number;
			transpose_degrees: number;
			time_ratio?: number;
			harmony_mode?: string | null;
			reference_voice?: number | null;
			voice_count?: number | null;
			voice_position?: number | null;
			voice_leading_enabled?: boolean | null;
			voice_leading_style?: string | null;
			octave_mode?: string | null;
			counterpoint_species?: string | null;
			counterpoint_strictness?: string | null;
			hold_mode?:
				| null
				| { kind: 'cancel' }
				| { kind: 'near_future'; tail_beats: number }
				| { kind: 'phrase_end' }
				| { kind: 'forever' };
			preset_id?: string | null;
		}>
	) {
		const prev = this.canonVoices;
		const clamped = voices.slice(0, 8).map((v, idx) => {
			// reference_voice must be < this voice's own index to avoid
			// circular cascades. Out-of-range or self-referential values
			// fall back to Player.
			let ref: number | null = null;
			if (typeof v.reference_voice === 'number' && v.reference_voice >= 0 && v.reference_voice < idx) {
				ref = v.reference_voice;
			}
			const vc =
				typeof v.voice_count === 'number'
					? Math.max(1, Math.min(4, Math.round(v.voice_count)))
					: null;
			const vp =
				typeof v.voice_position === 'number'
					? Math.max(0, Math.min(3, Math.round(v.voice_position)))
					: null;
			return {
				delay_beats: Math.max(0, Math.min(16, v.delay_beats)),
				transpose_degrees: Math.max(-7, Math.min(7, Math.round(v.transpose_degrees))),
				time_ratio: Math.max(0.125, Math.min(8, v.time_ratio ?? 1.0)),
				harmony_mode: v.harmony_mode ?? null,
				reference_voice: ref,
				voice_count: vc,
				voice_position: vp,
				voice_leading_enabled:
					typeof v.voice_leading_enabled === 'boolean' ? v.voice_leading_enabled : null,
				voice_leading_style: v.voice_leading_style ?? null,
				octave_mode: v.octave_mode ?? null,
				counterpoint_species: v.counterpoint_species ?? null,
				counterpoint_strictness: v.counterpoint_strictness ?? null,
				hold_mode: v.hold_mode ?? null,
				preset_id: v.preset_id ?? null
			};
		});
		this.canonVoices = clamped;
		try {
			await adapter.canonSetVoices(clamped);
			this.persist();
		} catch (e) {
			this.canonVoices = prev;
			throw e;
		}
	}

	/** Add a new voice with sensible defaults. UI convenience. */
	async addCanonVoice() {
		const next = [
			...this.canonVoices,
			{
				delay_beats: 1.0,
				transpose_degrees: 0,
				time_ratio: 1.0,
				harmony_mode: null,
				reference_voice: null
			}
		];
		await this.setCanonVoices(next);
	}

	/** Remove voice at the given index. Must keep at least one
	 *  voice or the canon goes silent (still valid but the UI
	 *  surfaces a "voices: 0" state). */
	async removeCanonVoice(idx: number) {
		const next = this.canonVoices.filter((_, i) => i !== idx);
		await this.setCanonVoices(next);
	}

	/** Update a single voice in place. */
	async updateCanonVoice(
		idx: number,
		patch: Partial<{
			delay_beats: number;
			transpose_degrees: number;
			time_ratio: number;
			harmony_mode: string | null;
			reference_voice: number | null;
			voice_count: number | null;
			voice_position: number | null;
			voice_leading_enabled: boolean | null;
			voice_leading_style: string | null;
			octave_mode: string | null;
			counterpoint_species: string | null;
			counterpoint_strictness: string | null;
			hold_mode:
				| null
				| { kind: 'cancel' }
				| { kind: 'near_future'; tail_beats: number }
				| { kind: 'phrase_end' }
				| { kind: 'forever' };
			preset_id: string | null;
		}>
	) {
		const next = this.canonVoices.map((v, i) => (i === idx ? { ...v, ...patch } : v));
		await this.setCanonVoices(next);
	}

	/** Pull canon state from the backend so the UI reflects the truth
	 *  after restart / external config change. */
	async syncCanonFromBackend() {
		try {
			this.companionEnabled = await adapter.companionIsEnabled();
			const s = await adapter.canonState();
			if (s) {
				this.canonEnabled = s.enabled;
				this.canonDelayBeats = s.delay_beats;
				this.canonTransposeDegrees = s.transpose_degrees;
				if (Array.isArray(s.voices) && s.voices.length > 0) {
					// Preserve UI-only preset_id by index — the backend
					// doesn't send it back. Falling out of sync would
					// silently unbind voices from their preset.
					const existingByIdx = this.canonVoices;
					this.canonVoices = s.voices.map((v, i) => {
						const rec = v as Record<string, unknown>;
						return {
							delay_beats: v.delay_beats,
							transpose_degrees: v.transpose_degrees,
							time_ratio:
								typeof rec.time_ratio === 'number' ? (rec.time_ratio as number) : 1.0,
							harmony_mode:
								typeof rec.harmony_mode === 'string' ? (rec.harmony_mode as string) : null,
							reference_voice:
								typeof rec.reference_voice === 'number'
									? (rec.reference_voice as number)
									: null,
							preset_id: existingByIdx[i]?.preset_id ?? null
						};
					});
				}
			}
		} catch {
			// Adapter may not implement canon — leave defaults in place.
		}
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
		this.canonNotes = [];
		this.counterpointNotes = [];
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
		if (typeof state.octaveIntensity === 'number') {
			this.octaveIntensity = Math.max(0, Math.min(1, state.octaveIntensity));
		}
		this.voiceLeadingEnabled = state.voiceLeadingEnabled;
		this.voiceLeadingStyle = state.voiceLeadingStyle as VoiceLeadingStyleName;
		this.interchangeEnabled = state.interchangeEnabled;
		this.interchangeRange = state.interchangeRange;
		this.voicePosition = state.voicePosition;
		this.voiceCount = state.voiceCount;
		this.autoKey = state.autoKey;
		this.tuningStyle = state.tuningStyle;
		this.tuningDepth = Math.max(0, Math.min(1, state.tuningDepth));
		this.harmonicLimit = state.harmonicLimit;
		this.isRunning = state.isRunning;
		if (this.isRunning && adapter.capabilities.noteUpdates) {
			this.startNoteUpdates();
		}
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
		if (state.explicitIntervalMap) {
			this.explicitIntervalMap = parseExplicitIntervalMap(state.explicitIntervalMap);
		}
	}

	/**
	 * Subscribe to real-time note update events from the adapter.
	 * Only assigns when values actually change to avoid Svelte re-renders.
	 */
	startNoteUpdates() {
		if (this.unsubNotes) return;

		this.unsubNotes = adapter.onNoteUpdate((state: NoteState) => {
			if (state.phrase) phrase.applySnapshot(state.phrase);
			if (!sameNotes(this.inputNotes, state.inputNotes))
				this.inputNotes = state.inputNotes;
			if (!sameNotes(this.harmonyNotes, state.harmonyNotes))
				this.harmonyNotes = state.harmonyNotes;
			if (!sameNotes(this.borrowedNotes, state.borrowedNotes))
				this.borrowedNotes = state.borrowedNotes;
			// Per-lane attribution (#11 follow-up). Optional fields —
			// fall back to empty arrays so Tauri builds (which don't
			// emit per-lane today) don't disturb existing UI behavior.
			const nextCanon = state.canonNotes ?? [];
			const nextCp = state.counterpointNotes ?? [];
			if (!sameNotes(this.canonNotes, nextCanon)) this.canonNotes = nextCanon;
			if (!sameNotes(this.counterpointNotes, nextCp))
				this.counterpointNotes = nextCp;
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
