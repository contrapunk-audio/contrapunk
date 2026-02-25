/**
 * Engine Store — Reactive Harmony Engine State (Svelte 5 Runes)
 *
 * Tracks the harmony engine configuration and real-time note state.
 * All mutations call the platform adapter and update local state
 * optimistically (set locally, then call backend; revert on error).
 */

import { adapter } from '$lib/adapter';
import type { NoteState } from '$lib/adapter';

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
	| 'BarryHarris';

export type ScaleFamilyName =
	| 'Church'
	| 'HarmonicMinor'
	| 'MelodicMinor'
	| 'Exotic'
	| 'BarryHarris';

export type ScaleModeName =
	// Church (7)
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
	// Exotic (5)
	| 'DoubleHarmonic'
	| 'HungarianMinor'
	| 'Enigmatic'
	| 'NeapolitanMinor'
	| 'NeapolitanMajor'
	// Barry Harris (2)
	| 'BHMajor6thDim'
	| 'BHMinor6thDim';

export type OctaveModeName = 'None' | 'Spread' | 'BassTrebleSplit' | 'Mirror';

export type VoiceLeadingStyleName = 'Free' | 'Palestrina' | 'BachChorale' | 'Jazz';

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

export const ALL_MODES: { name: HarmonyModeName; label: string; shortLabel: string }[] = [
	{ name: 'PassThrough', label: 'Pass Through', shortLabel: 'Pass' },
	{ name: 'DiatonicThirds', label: 'Diatonic Thirds', shortLabel: '3rds' },
	{ name: 'DiatonicFourths', label: 'Diatonic Fourths', shortLabel: '4ths' },
	{ name: 'RandomBelow', label: 'Random Below', shortLabel: 'Rand' },
	{ name: 'RandomBelowNoSeconds', label: 'Random No 2nds', shortLabel: 'Rand-' },
	{ name: 'ContraryMotion', label: 'Contrary Motion', shortLabel: 'Contra' },
	{ name: 'StrictCounterpoint', label: 'Strict Counterpoint', shortLabel: 'Strict' },
	{ name: 'BarryHarris', label: 'Barry Harris', shortLabel: 'Barry' }
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
		family: 'Exotic',
		label: 'Exotic',
		modes: [
			{ name: 'DoubleHarmonic', label: 'Double Harmonic' },
			{ name: 'HungarianMinor', label: 'Hungarian Minor' },
			{ name: 'Enigmatic', label: 'Enigmatic' },
			{ name: 'NeapolitanMinor', label: 'Neapolitan Minor' },
			{ name: 'NeapolitanMajor', label: 'Neapolitan Major' }
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

	// -- Transport --
	isRunning = $state(false);

	// -- Real-time note state (updated by adapter events) --
	inputNotes = $state<number[]>([]);
	harmonyNotes = $state<number[]>([]);
	borrowedNotes = $state<number[]>([]);
	generatorNotes = $state<number[]>([]);
	inScaleNotes = $state<number[]>([]);

	// -- Display --
	chordName = $state('');
	lastBorrowedFrom = $state('');

	// -- Internal --
	private unsubNotes: (() => void) | null = null;

	// === Adapter-wired actions (optimistic update with rollback) ===

	async setKey(newKey: KeyName) {
		const prev = this.key;
		this.key = newKey;
		try {
			await adapter.setKey(newKey);
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
		} catch (e) {
			this.voicePosition = prev;
			throw e;
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
		this.isRunning = state.isRunning;
	}

	/**
	 * Subscribe to real-time note update events from the adapter.
	 */
	startNoteUpdates() {
		if (this.unsubNotes) return;

		this.unsubNotes = adapter.onNoteUpdate((state: NoteState) => {
			this.inputNotes = state.inputNotes;
			this.harmonyNotes = state.harmonyNotes;
			this.borrowedNotes = state.borrowedNotes;
			this.chordName = state.chordName;
			this.lastBorrowedFrom = state.lastBorrowedFrom;
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
