/**
 * Voice Library — named harmony-voice presets.
 *
 * Each VoicePreset bundles the harmony-engine fields that define a voice's
 * character (mode, scale, voice-leading, species, etc.) under a name. The
 * library serves two consumers today:
 *
 * - The Voices tab — CRUD over user-defined presets.
 * - The Companion tab — each canon voice picks a preset whose fields apply
 *   to that voice's mini-engine.
 *
 * Built-in presets are baked into code and never persisted. User presets
 * live in localStorage under VOICE_LIBRARY_KEY. Built-ins always appear
 * first in `engine.allPresets`.
 */
import type {
	CounterpointSpeciesName,
	CounterpointStrictnessName,
	HarmonyModeName,
	OctaveModeName,
	ScaleModeName,
	VoiceLeadingStyleName
} from './engine.svelte';

const VOICE_LIBRARY_KEY = 'contrapunk-voice-library';
const VOICE_LIBRARY_VERSION = 1;

export interface VoicePreset {
	id: string;
	name: string;
	builtIn: boolean;
	harmony_mode: HarmonyModeName;
	scale_mode: ScaleModeName;
	voice_leading_enabled: boolean;
	voice_leading_style: VoiceLeadingStyleName;
	octave_mode: OctaveModeName;
	counterpoint_species: CounterpointSpeciesName;
	counterpoint_strictness: CounterpointStrictnessName;
	voice_count: number;
	voice_position: number;
}

/** Built-in SATB chorale roles + a few solo voices. */
const BUILTIN_PRESETS: VoicePreset[] = [
	{
		id: 'builtin-echo',
		name: 'Pass-Through Echo',
		builtIn: true,
		harmony_mode: 'PassThrough',
		scale_mode: 'Ionian',
		voice_leading_enabled: false,
		voice_leading_style: 'Free',
		octave_mode: 'None',
		counterpoint_species: 'Species1',
		counterpoint_strictness: 'Strict',
		voice_count: 1,
		voice_position: 0
	},
	{
		id: 'builtin-soprano',
		name: 'Soprano',
		builtIn: true,
		harmony_mode: 'BachChorale',
		scale_mode: 'Ionian',
		voice_leading_enabled: true,
		voice_leading_style: 'BachChorale',
		octave_mode: 'Spread',
		counterpoint_species: 'Species1',
		counterpoint_strictness: 'Strict',
		voice_count: 4,
		voice_position: 0
	},
	{
		id: 'builtin-alto',
		name: 'Alto',
		builtIn: true,
		harmony_mode: 'StrictCounterpoint',
		scale_mode: 'Ionian',
		voice_leading_enabled: true,
		voice_leading_style: 'Palestrina',
		octave_mode: 'Spread',
		counterpoint_species: 'Species1',
		counterpoint_strictness: 'Strict',
		voice_count: 4,
		voice_position: 1
	},
	{
		id: 'builtin-tenor',
		name: 'Tenor',
		builtIn: true,
		harmony_mode: 'StrictCounterpoint',
		scale_mode: 'Ionian',
		voice_leading_enabled: true,
		voice_leading_style: 'Palestrina',
		octave_mode: 'Spread',
		counterpoint_species: 'Species1',
		counterpoint_strictness: 'Strict',
		voice_count: 4,
		voice_position: 2
	},
	{
		id: 'builtin-bass',
		name: 'Bass',
		builtIn: true,
		harmony_mode: 'BachChorale',
		scale_mode: 'Ionian',
		voice_leading_enabled: true,
		voice_leading_style: 'BachChorale',
		octave_mode: 'Spread',
		counterpoint_species: 'Species1',
		counterpoint_strictness: 'Strict',
		voice_count: 4,
		voice_position: 3
	},
	{
		id: 'builtin-cp-solo',
		name: 'Counterpoint Solo',
		builtIn: true,
		harmony_mode: 'StrictCounterpoint',
		scale_mode: 'Ionian',
		voice_leading_enabled: false,
		voice_leading_style: 'Free',
		octave_mode: 'None',
		counterpoint_species: 'Species1',
		counterpoint_strictness: 'Strict',
		voice_count: 2,
		voice_position: 0
	},
	{
		id: 'builtin-thirds',
		name: 'Diatonic Thirds',
		builtIn: true,
		harmony_mode: 'DiatonicThirds',
		scale_mode: 'Ionian',
		voice_leading_enabled: false,
		voice_leading_style: 'Free',
		octave_mode: 'None',
		counterpoint_species: 'Species1',
		counterpoint_strictness: 'Strict',
		voice_count: 2,
		voice_position: 0
	},
	{
		id: 'builtin-chorale-4',
		name: 'Bach 4-Voice Chorale',
		builtIn: true,
		harmony_mode: 'BachChorale',
		scale_mode: 'Ionian',
		voice_leading_enabled: true,
		voice_leading_style: 'BachChorale',
		octave_mode: 'Spread',
		counterpoint_species: 'Species1',
		counterpoint_strictness: 'Strict',
		voice_count: 4,
		voice_position: 3
	}
];

function loadUserPresets(): VoicePreset[] {
	try {
		const raw = localStorage.getItem(VOICE_LIBRARY_KEY);
		if (!raw) return [];
		const parsed = JSON.parse(raw);
		if (!parsed || parsed.version !== VOICE_LIBRARY_VERSION) return [];
		if (!Array.isArray(parsed.presets)) return [];
		return parsed.presets.filter(
			(p: unknown): p is VoicePreset =>
				typeof p === 'object' &&
				p !== null &&
				typeof (p as Record<string, unknown>).id === 'string' &&
				typeof (p as Record<string, unknown>).name === 'string'
		);
	} catch {
		return [];
	}
}

function saveUserPresets(presets: VoicePreset[]) {
	try {
		localStorage.setItem(
			VOICE_LIBRARY_KEY,
			JSON.stringify({ version: VOICE_LIBRARY_VERSION, presets })
		);
	} catch {
		/* localStorage full or unavailable */
	}
}

class VoiceLibraryStore {
	private _userPresets = $state<VoicePreset[]>(loadUserPresets());

	get builtIn(): VoicePreset[] {
		return BUILTIN_PRESETS;
	}

	get userPresets(): VoicePreset[] {
		return this._userPresets;
	}

	/** Built-ins + user presets, in display order. */
	get all(): VoicePreset[] {
		return [...BUILTIN_PRESETS, ...this._userPresets];
	}

	byId(id: string): VoicePreset | undefined {
		return this.all.find((p) => p.id === id);
	}

	/** Save a new user preset. Returns the assigned id. */
	create(preset: Omit<VoicePreset, 'id' | 'builtIn'>): string {
		const id = `user-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`;
		const next: VoicePreset = { ...preset, id, builtIn: false };
		this._userPresets = [...this._userPresets, next];
		saveUserPresets(this._userPresets);
		return id;
	}

	/** Update an existing user preset. Built-ins are immutable. */
	update(id: string, patch: Partial<Omit<VoicePreset, 'id' | 'builtIn'>>) {
		const idx = this._userPresets.findIndex((p) => p.id === id);
		if (idx < 0) return;
		this._userPresets = this._userPresets.map((p, i) =>
			i === idx ? { ...p, ...patch } : p
		);
		saveUserPresets(this._userPresets);
	}

	/** Delete a user preset. Built-ins cannot be deleted. */
	delete(id: string) {
		this._userPresets = this._userPresets.filter((p) => p.id !== id);
		saveUserPresets(this._userPresets);
	}
}

export const voiceLibrary = new VoiceLibraryStore();
