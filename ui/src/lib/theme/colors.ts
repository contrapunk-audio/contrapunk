/**
 * HLD Design System — TypeScript Color Constants
 *
 * Mirrors the CSS custom properties in tokens.css for use in
 * Svelte components, canvas rendering, and dynamic styling.
 */

// === Background Layers ===
export const BG_DEEP = '#0a0a1a';
export const BG_BASE = '#0f0e1a';
export const BG_PANEL = '#151428';
export const WIDGET_BG = '#1a1833';
export const WIDGET_INACTIVE = '#25223d';

// === HLD Accent Colors ===
export const ACCENT_MAGENTA = '#ff3388';
export const ACCENT_MAGENTA_DIM = '#cc2266';
export const ACCENT_CYAN = '#33ddff';
export const ACCENT_CYAN_DIM = '#2299cc';
export const ACCENT_TEAL = '#00ccaa';
export const ACCENT_PINK = '#ff77aa';
export const ACCENT_AMBER = '#ffaa33';
export const ACCENT_GOLD = '#ffdd44';

// === Sacred Piano Key Colors ===
// Refreshed to match the redesigned contrapunk.com palette
// (teal input, magenta harmony, violet borrowed). See issue #37.
export const PIANO_INPUT = '#4fe8c3';     // teal — melody in
export const PIANO_HARMONY = '#ff2e88';   // magenta — harmony out
export const PIANO_BORROWED = '#8a5cff';  // violet — borrowed / modal interchange
export const PIANO_IN_SCALE = 'rgba(255, 253, 180, 0.35)';

// === Text ===
export const TEXT_PRIMARY = '#e8e0f0';
export const TEXT_SECONDARY = '#8888aa';
export const TEXT_DIM = '#555577';

// === Borders ===
export const BORDER = '#2a2848';
export const BORDER_ACTIVE = '#ff3388';

// === Status Colors ===
export const STATUS_RUNNING = '#00e436';
export const STATUS_STOPPED = '#555577';
export const STATUS_ERROR = '#ff3344';

/**
 * Get the display color for a piano key based on its state.
 *
 * Priority: input (green) > harmony (orange) > borrowed (amber) > default
 *
 * @param midi - MIDI note number (21-108 for 88-key piano)
 * @param inputNotes - Currently sounding input/melody notes
 * @param harmonyNotes - Currently sounding harmony notes
 * @param borrowedNotes - Notes borrowed via modal interchange
 * @param inScaleNotes - Notes that belong to the current scale
 * @returns CSS color string or empty string for default key color
 */
export function getPianoKeyColor(
	midi: number,
	inputNotes: number[] = [],
	harmonyNotes: number[] = [],
	borrowedNotes: number[] = [],
	inScaleNotes: number[] = []
): string {
	if (inputNotes.includes(midi)) return PIANO_INPUT;
	if (harmonyNotes.includes(midi)) return PIANO_HARMONY;
	if (borrowedNotes.includes(midi)) return PIANO_BORROWED;
	return '';
}

/**
 * Full HLD color palette as a single object for easy iteration.
 */
export const HLD_PALETTE = {
	bgDeep: BG_DEEP,
	bgBase: BG_BASE,
	bgPanel: BG_PANEL,
	widgetBg: WIDGET_BG,
	widgetInactive: WIDGET_INACTIVE,
	accentMagenta: ACCENT_MAGENTA,
	accentMagentaDim: ACCENT_MAGENTA_DIM,
	accentCyan: ACCENT_CYAN,
	accentCyanDim: ACCENT_CYAN_DIM,
	accentTeal: ACCENT_TEAL,
	accentPink: ACCENT_PINK,
	accentAmber: ACCENT_AMBER,
	accentGold: ACCENT_GOLD,
	pianoInput: PIANO_INPUT,
	pianoHarmony: PIANO_HARMONY,
	pianoBorrowed: PIANO_BORROWED,
	pianoInScale: PIANO_IN_SCALE,
	textPrimary: TEXT_PRIMARY,
	textSecondary: TEXT_SECONDARY,
	textDim: TEXT_DIM,
	border: BORDER,
	borderActive: BORDER_ACTIVE
} as const;
