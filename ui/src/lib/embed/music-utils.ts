/**
 * Shared music utilities for the embed components.
 *
 * Used by the canonical embed components in this directory AND
 * mirror-copied to the website repo (same pattern as the WASM bundle
 * and `keyboard-input.ts`). Keep this file dependency-free so it can
 * be `cp`'d across without pulling Svelte / Tauri / nanostores into
 * the host repo.
 *
 * Per contrapunk-audio/contrapunk#66.
 */

/** Display-spelling note names with the Unicode sharp glyph (♯, U+266F)
 *  instead of ASCII '#'. All UI surfaces consume this so notes look
 *  like proper music notation rather than programmer-style identifiers. */
export const NOTE_NAMES = [
	'C',
	'C♯',
	'D',
	'D♯',
	'E',
	'F',
	'F♯',
	'G',
	'G♯',
	'A',
	'A♯',
	'B'
] as const;

/** ASCII spelling of the same names. Kept for any code path that needs
 *  to talk to the Rust engine wire format (which uses ASCII 'C#'). */
export const NOTE_NAMES_ASCII = [
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
] as const;

/** MIDI note number → "C4", "E♭5", "F♯3" form. Octave per the standard
 *  Yamaha convention where MIDI 60 = C4. */
export function midiToName(midi: number): string {
	const octave = Math.floor(midi / 12) - 1;
	const idx = ((midi % 12) + 12) % 12;
	return NOTE_NAMES[idx] + octave;
}

/**
 * Replace ASCII accidentals with proper Unicode music-notation glyphs.
 * Use anywhere a string from the engine, settings, or any other source
 * may contain '#' or 'b' that the user should see as ♯ or ♭.
 *
 * Patterns covered:
 *   - Note + accidental:        Bb       → B♭     · C#      → C♯
 *   - Note + double accidental: Cbb      → C♭♭    · F##     → F♯♯
 *   - Scale degree:             " b6"    → " ♭6"  · " #4"   → " ♯4"
 *   - Compound chord ext:       "maj7b9" → "maj7♭9" · "7#11" → "7♯11"
 *
 * Word-internal 'b' is intentionally left alone so tokens like "Bebop"
 * or "blues" don't get mangled. The replacements only fire after a note
 * letter [A-G], a digit, or a whitespace boundary.
 */
export function formatMusicalString(s: string): string {
	if (!s) return s;
	return s
		.replace(/([A-G])bb/g, '$1♭♭')
		.replace(/([A-G])##/g, '$1♯♯')
		.replace(/([A-G])#/g, '$1♯')
		.replace(/([A-G])b(?![A-Za-z])/g, '$1♭')
		.replace(/(\s|\d)bb(\d)/g, '$1♭♭$2')
		.replace(/(\s|\d)b(\d)/g, '$1♭$2')
		.replace(/(\s|\d)#(\d)/g, '$1♯$2');
}

/** True if the MIDI note is a black key on a piano (C#, D#, F#, G#, A#). */
export function isBlackKey(midi: number): boolean {
	return [1, 3, 6, 8, 10].includes(midi % 12);
}

// === Voice color palette ===
//
// Three colors map onto the three "voice roles" the engine exposes.
// Used by both Piano and Fretboard so a key lit by the same role
// looks the same across surfaces.

/** Input voices — what the user played. Cyan/teal. */
export const COLOR_INPUT = '#4fe8c3';
/** Harmony voices — what the engine generated. Magenta. */
export const COLOR_HARMONY = '#ff2e88';
/** Borrowed voices — modal interchange picks. Violet. */
export const COLOR_BORROWED = '#8a5cff';

/** Open-string MIDI values for standard guitar tuning, low to high. */
export const STANDARD_TUNING = [40, 45, 50, 55, 59, 64] as const;
export const STANDARD_TUNING_LABELS = ['E2', 'A2', 'D3', 'G3', 'B3', 'E4'] as const;

/** MIDI range for a standard 88-key piano: A0 through C8. */
export const PIANO_START = 21;
export const PIANO_END = 108;
