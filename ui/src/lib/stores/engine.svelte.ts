/**
 * Engine Store — Svelte 5 rune-based reactive state for the harmony engine.
 *
 * Stub implementation for UI development. Will be wired to the platform
 * adapter (Tauri IPC or WASM direct calls) in Plan 03.
 */

// === Type definitions ===

export type KeyName = 'C' | 'C#' | 'D' | 'D#' | 'E' | 'F' | 'F#' | 'G' | 'G#' | 'A' | 'A#' | 'B';

export type HarmonyModeName =
	| 'PassThrough'
	| 'DiatonicThirds'
	| 'DiatonicFourths'
	| 'RandomBelow'
	| 'RandomBelowNoSeconds'
	| 'ContraryMotion'
	| 'StrictCounterpoint'
	| 'BarryHarris';

export type ScaleFamilyName = 'Church' | 'HarmonicMinor' | 'MelodicMinor' | 'Exotic' | 'BarryHarris';

export type ScaleModeName =
	// Church (7)
	| 'Ionian' | 'Dorian' | 'Phrygian' | 'Lydian' | 'Mixolydian' | 'Aeolian' | 'Locrian'
	// Harmonic Minor (7)
	| 'HarmonicMinor' | 'LocrianNat6' | 'IonianAug' | 'DorianSharp4'
	| 'PhrygianDominant' | 'LydianSharp2' | 'SuperLocrianDim'
	// Melodic Minor (7)
	| 'MelodicMinor' | 'DorianFlat2' | 'LydianAug' | 'LydianDominant'
	| 'MixolydianFlat6' | 'LocrianNat2' | 'SuperLocrian'
	// Exotic (5)
	| 'DoubleHarmonic' | 'HungarianMinor' | 'Enigmatic' | 'NeapolitanMinor' | 'NeapolitanMajor'
	// Barry Harris (2)
	| 'BHMajor6thDim' | 'BHMinor6thDim';

export type OctaveModeName = 'None' | 'Spread' | 'BassTrebleSplit' | 'Mirror';

export type VoiceLeadingStyleName = 'Free' | 'Palestrina' | 'BachChorale' | 'Jazz';

export interface ScaleFamilyGroup {
	family: ScaleFamilyName;
	label: string;
	modes: { name: ScaleModeName; label: string }[];
}

// === Constants ===

export const ALL_KEYS: KeyName[] = ['C', 'C#', 'D', 'D#', 'E', 'F', 'F#', 'G', 'G#', 'A', 'A#', 'B'];

export const ALL_MODES: { name: HarmonyModeName; label: string; shortLabel: string }[] = [
	{ name: 'PassThrough', label: 'Pass Through', shortLabel: 'Pass' },
	{ name: 'DiatonicThirds', label: 'Diatonic Thirds', shortLabel: '3rds' },
	{ name: 'DiatonicFourths', label: 'Diatonic Fourths', shortLabel: '4ths' },
	{ name: 'RandomBelow', label: 'Random Below', shortLabel: 'Rand' },
	{ name: 'RandomBelowNoSeconds', label: 'Random No 2nds', shortLabel: 'Rand-' },
	{ name: 'ContraryMotion', label: 'Contrary Motion', shortLabel: 'Contra' },
	{ name: 'StrictCounterpoint', label: 'Strict Counterpoint', shortLabel: 'Strict' },
	{ name: 'BarryHarris', label: 'Barry Harris', shortLabel: 'Barry' },
];

export const SCALE_FAMILIES: ScaleFamilyGroup[] = [
	{
		family: 'Church',
		label: 'Church Modes',
		modes: [
			{ name: 'Ionian', label: 'Ionian (Major)' },
			{ name: 'Dorian', label: 'Dorian' },
			{ name: 'Phrygian', label: 'Phrygian' },
			{ name: 'Lydian', label: 'Lydian' },
			{ name: 'Mixolydian', label: 'Mixolydian' },
			{ name: 'Aeolian', label: 'Aeolian (Minor)' },
			{ name: 'Locrian', label: 'Locrian' },
		],
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
			{ name: 'SuperLocrianDim', label: 'Super Locrian Dim' },
		],
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
			{ name: 'SuperLocrian', label: 'Super Locrian' },
		],
	},
	{
		family: 'Exotic',
		label: 'Exotic',
		modes: [
			{ name: 'DoubleHarmonic', label: 'Double Harmonic' },
			{ name: 'HungarianMinor', label: 'Hungarian Minor' },
			{ name: 'Enigmatic', label: 'Enigmatic' },
			{ name: 'NeapolitanMinor', label: 'Neapolitan Minor' },
			{ name: 'NeapolitanMajor', label: 'Neapolitan Major' },
		],
	},
	{
		family: 'BarryHarris',
		label: 'Barry Harris',
		modes: [
			{ name: 'BHMajor6thDim', label: 'Major 6th Dim' },
			{ name: 'BHMinor6thDim', label: 'Minor 6th Dim' },
		],
	},
];

export const OCTAVE_MODES: { name: OctaveModeName; label: string }[] = [
	{ name: 'None', label: 'None' },
	{ name: 'Spread', label: 'Spread' },
	{ name: 'BassTrebleSplit', label: 'Split' },
	{ name: 'Mirror', label: 'Mirror' },
];

export const VOICE_LEADING_STYLES: { name: VoiceLeadingStyleName; label: string }[] = [
	{ name: 'Free', label: 'Free' },
	{ name: 'Palestrina', label: 'Palestrina' },
	{ name: 'BachChorale', label: 'Bach' },
	{ name: 'Jazz', label: 'Jazz' },
];

// === Engine Store (Svelte 5 runes) ===

class EngineStore {
	// Harmony configuration
	key = $state<KeyName>('C');
	mode = $state<HarmonyModeName>('PassThrough');
	scaleMode = $state<ScaleModeName>('Ionian');
	octaveMode = $state<OctaveModeName>('None');

	// Voice leading
	voiceLeadingEnabled = $state(false);
	voiceLeadingStyle = $state<VoiceLeadingStyleName>('Free');

	// Modal interchange
	interchangeEnabled = $state(false);
	borrowingRange = $state(1);

	// Transport
	isRunning = $state(false);

	// Real-time note state (updated by adapter)
	inputNotes = $state<number[]>([]);
	harmonyNotes = $state<number[]>([]);
	borrowedNotes = $state<number[]>([]);
	generatorNotes = $state<number[]>([]);
	inScaleNotes = $state<number[]>([]);

	// Display
	chordName = $state('');
	borrowedFrom = $state('');

	// Voice position
	voicePosition = $state(0);
	voiceCount = $state(1);

	// === Actions (stubs — will be wired to adapter in Plan 03) ===

	setKey(key: KeyName) {
		this.key = key;
	}

	setMode(mode: HarmonyModeName) {
		this.mode = mode;
	}

	setScaleMode(mode: ScaleModeName) {
		this.scaleMode = mode;
	}

	setOctaveMode(mode: OctaveModeName) {
		this.octaveMode = mode;
	}

	setVoiceLeading(enabled: boolean, style?: VoiceLeadingStyleName) {
		this.voiceLeadingEnabled = enabled;
		if (style) this.voiceLeadingStyle = style;
	}

	setInterchange(enabled: boolean, range?: number) {
		this.interchangeEnabled = enabled;
		if (range !== undefined) this.borrowingRange = range;
	}

	setVoicePosition(position: number) {
		this.voicePosition = position;
	}

	start() {
		this.isRunning = true;
	}

	stop() {
		this.isRunning = false;
	}

	toggle() {
		if (this.isRunning) {
			this.stop();
		} else {
			this.start();
		}
	}
}

export const engine = new EngineStore();
