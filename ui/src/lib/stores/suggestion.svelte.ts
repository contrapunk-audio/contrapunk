/**
 * Suggestion Store — Reactive Next-Note Suggestion State (Svelte 5 Runes)
 *
 * Manages the visual suggestion overlay state. Suggestions are computed
 * by the Rust scorer via WASM, polled at frame rate, and exposed as
 * reactive state for Piano and Fretboard components.
 *
 * This is a VISUAL OVERLAY only — suggestions are never played audibly.
 */

import { adapter } from '$lib/adapter';

// === Types ===

export interface SuggestionScore {
	note: number;
	score: number;
}

export type SuggestionWeightTerm =
	| 'chord_tone'
	| 'scale_tone'
	| 'dissonance'
	| 'proximity'
	| 'contour'
	| 'leap_recovery'
	| 'repetition'
	| 'next_chord_prep'
	| 'leading_tone'
	| 'narmour'
	| 'tessitura';

export const WEIGHT_TERMS: {
	term: SuggestionWeightTerm;
	label: string;
	group: 'harmonic' | 'melodic' | 'contextual';
}[] = [
	// Harmonic
	{ term: 'chord_tone', label: 'Chord Tone', group: 'harmonic' },
	{ term: 'scale_tone', label: 'Scale Tone', group: 'harmonic' },
	{ term: 'dissonance', label: 'Dissonance', group: 'harmonic' },
	// Melodic
	{ term: 'proximity', label: 'Proximity', group: 'melodic' },
	{ term: 'contour', label: 'Contour', group: 'melodic' },
	{ term: 'leap_recovery', label: 'Leap Recovery', group: 'melodic' },
	{ term: 'repetition', label: 'Repetition', group: 'melodic' },
	// Contextual
	{ term: 'next_chord_prep', label: 'Next Chord Prep', group: 'contextual' },
	{ term: 'leading_tone', label: 'Leading Tone', group: 'contextual' },
	{ term: 'narmour', label: 'Narmour', group: 'contextual' },
	{ term: 'tessitura', label: 'Tessitura', group: 'contextual' }
];

// === Presets ===

export interface SuggestionPreset {
	name: string;
	weights: Record<SuggestionWeightTerm, number>;
}

// Default preset weights must match Rust SuggestionConfig::default()
// in src/harmony/suggestion.rs. Keep both in sync.
export const SUGGESTION_PRESETS: SuggestionPreset[] = [
	{
		name: 'Default',
		weights: {
			chord_tone: 10,
			scale_tone: 4,
			dissonance: 6,
			proximity: 8,
			contour: 3,
			leap_recovery: 6,
			repetition: 5,
			next_chord_prep: 4,
			leading_tone: 7,
			narmour: 3,
			tessitura: 2
		}
	},
	{
		name: 'Chord-focused',
		weights: {
			chord_tone: 10,
			scale_tone: 8,
			dissonance: 8,
			proximity: 3,
			contour: 2,
			leap_recovery: 2,
			repetition: 3,
			next_chord_prep: 6,
			leading_tone: 4,
			narmour: 1,
			tessitura: 1
		}
	},
	{
		name: 'Jazz',
		weights: {
			chord_tone: 6,
			scale_tone: 3,
			dissonance: 3,
			proximity: 5,
			contour: 4,
			leap_recovery: 4,
			repetition: 6,
			next_chord_prep: 5,
			leading_tone: 3,
			narmour: 8,
			tessitura: 2
		}
	},
	{
		name: 'Stepwise',
		weights: {
			chord_tone: 4,
			scale_tone: 6,
			dissonance: 4,
			proximity: 10,
			contour: 8,
			leap_recovery: 8,
			repetition: 5,
			next_chord_prep: 2,
			leading_tone: 5,
			narmour: 6,
			tessitura: 3
		}
	}
];

// === Persistence ===

const SUGGESTION_SETTINGS_KEY = 'contrapunk-suggestion-settings';

interface PersistedSuggestionSettings {
	enabled: boolean;
	weights: Record<string, number>;
}

function loadSuggestionSettings(): PersistedSuggestionSettings | null {
	try {
		const raw = localStorage.getItem(SUGGESTION_SETTINGS_KEY);
		if (!raw) return null;
		return JSON.parse(raw);
	} catch {
		return null;
	}
}

function saveSuggestionSettings(s: PersistedSuggestionSettings) {
	try {
		localStorage.setItem(SUGGESTION_SETTINGS_KEY, JSON.stringify(s));
	} catch {
		// silently ignore
	}
}

// === Store ===

class SuggestionStore {
	/** Current ranked suggestions (top 12). */
	suggestions = $state<SuggestionScore[]>([]);

	/** Whether the suggestion overlay is enabled. */
	enabled = $state(false);

	/**
	 * Current weight values.
	 *
	 * These defaults are duplicated from the Rust SuggestionConfig
	 * (src/harmony/suggestion.rs) for offline/initial rendering.
	 * Keep in sync, or call get_suggestion_weights() from WASM at init.
	 */
	weights = $state<Record<SuggestionWeightTerm, number>>({
		chord_tone: 10,
		scale_tone: 4,
		dissonance: 6,
		proximity: 8,
		contour: 3,
		leap_recovery: 6,
		repetition: 5,
		next_chord_prep: 4,
		leading_tone: 7,
		narmour: 3,
		tessitura: 2
	});

	/** RAF handle for polling loop. */
	private rafHandle: number | null = null;

	/** Debounce timer for weight changes. */
	private weightDebounce: ReturnType<typeof setTimeout> | null = null;

	/** Start the suggestion polling loop. */
	start() {
		if (this.rafHandle !== null) return;
		this.enabled = true;
		this.persist();

		const tick = () => {
			if (!this.enabled) return;
			this.refresh();
			this.rafHandle = requestAnimationFrame(tick);
		};
		this.rafHandle = requestAnimationFrame(tick);
	}

	/** Stop the suggestion polling loop. */
	stop() {
		this.enabled = false;
		if (this.rafHandle !== null) {
			cancelAnimationFrame(this.rafHandle);
			this.rafHandle = null;
		}
		this.suggestions = [];
		this.persist();
	}

	/** Toggle enabled state. */
	toggle() {
		if (this.enabled) {
			this.stop();
		} else {
			this.start();
		}
	}

	/** Refresh suggestions from the backend. */
	private refresh() {
		try {
			const raw = adapter.getSuggestions?.();
			if (raw && Array.isArray(raw) && raw.length > 0) {
				this.suggestions = raw;
			}
		} catch {
			// Silently ignore errors during polling
		}
	}

	/** Set a weight value (debounced backend update). */
	setWeight(term: SuggestionWeightTerm, value: number) {
		const clamped = Math.max(0, Math.min(10, Math.round(value * 2) / 2));
		this.weights[term] = clamped;

		// Debounce backend update
		if (this.weightDebounce) clearTimeout(this.weightDebounce);
		this.weightDebounce = setTimeout(() => {
			try {
				adapter.setSuggestionWeight?.(term, clamped);
			} catch {
				// silently ignore
			}
			this.persist();
		}, 100);
	}

	/** Apply a preset. */
	applyPreset(preset: SuggestionPreset) {
		for (const [term, value] of Object.entries(preset.weights)) {
			this.weights[term as SuggestionWeightTerm] = value;
			try {
				adapter.setSuggestionWeight?.(term, value);
			} catch {
				// silently ignore
			}
		}
		this.persist();
	}

	/** Reset to default weights. */
	resetToDefaults() {
		this.applyPreset(SUGGESTION_PRESETS[0]);
		try {
			adapter.resetSuggestionWeights?.();
		} catch {
			// silently ignore
		}
	}

	/** Check if a MIDI note is in the top-3 suggestions. */
	isTop3(midi: number): boolean {
		return this.suggestions.slice(0, 3).some((s) => s.note === midi);
	}

	/** Check if a MIDI note is in the next-5 (positions 3-7) suggestions. */
	isNext5(midi: number): boolean {
		return this.suggestions.slice(3, 8).some((s) => s.note === midi);
	}

	/** Get the suggestion score for a MIDI note, or 0 if not in suggestions. */
	scoreFor(midi: number): number {
		return this.suggestions.find((s) => s.note === midi)?.score ?? 0;
	}

	/** Restore saved settings from localStorage. */
	restoreSettings() {
		const saved = loadSuggestionSettings();
		if (!saved) return;

		if (saved.weights) {
			for (const [term, value] of Object.entries(saved.weights)) {
				if (term in this.weights) {
					this.weights[term as SuggestionWeightTerm] = value;
					try {
						adapter.setSuggestionWeight?.(term, value);
					} catch {
						// silently ignore
					}
				}
			}
		}

		if (saved.enabled) {
			this.start();
		}
	}

	/** Persist current settings. */
	private persist() {
		saveSuggestionSettings({
			enabled: this.enabled,
			weights: { ...this.weights }
		});
	}
}

export const suggestions = new SuggestionStore();
