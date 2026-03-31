/**
 * Maps a dB value to an HLD-palette color for spectrogram heatmaps.
 *
 * Gradient (low energy to high energy):
 *   silence  -> dark purples -> cyans -> magentas -> gold
 *   #0a0a1a  -> #151428/#2a2848 -> #2299cc/#33ddff -> #ff3388/#ff77aa -> #ffdd44
 */

type ColorStop = { t: number; r: number; g: number; b: number };

const STOPS: ColorStop[] = [
	{ t: 0.0, r: 10, g: 10, b: 26 },    // #0a0a1a — silence / bg-deep
	{ t: 0.15, r: 21, g: 20, b: 40 },    // #151428 — dark purple
	{ t: 0.3, r: 42, g: 40, b: 72 },     // #2a2848 — medium purple
	{ t: 0.5, r: 34, g: 153, b: 204 },   // #2299cc — dim cyan
	{ t: 0.65, r: 51, g: 221, b: 255 },  // #33ddff — bright cyan
	{ t: 0.8, r: 255, g: 51, b: 136 },   // #ff3388 — magenta
	{ t: 0.9, r: 255, g: 119, b: 170 },  // #ff77aa — light magenta
	{ t: 1.0, r: 255, g: 221, b: 68 },   // #ffdd44 — gold (peak)
];

function lerp(a: number, b: number, t: number): number {
	return a + (b - a) * t;
}

/**
 * Convert a dB value to an HLD-palette hex color.
 *
 * @param db    The decibel value for this cell
 * @param minDb The minimum dB in the dataset (most negative, e.g. -80)
 * @param maxDb The maximum dB in the dataset (least negative, e.g. 0)
 * @returns     A CSS hex color string, e.g. "#2299cc"
 */
export function dbToColor(db: number, minDb: number, maxDb: number): string {
	const range = maxDb - minDb;
	// Normalize to 0..1 (0 = silence, 1 = peak)
	const t = range === 0 ? 0 : Math.max(0, Math.min(1, (db - minDb) / range));

	// Find the two stops we sit between
	let lo = STOPS[0];
	let hi = STOPS[STOPS.length - 1];
	for (let i = 0; i < STOPS.length - 1; i++) {
		if (t >= STOPS[i].t && t <= STOPS[i + 1].t) {
			lo = STOPS[i];
			hi = STOPS[i + 1];
			break;
		}
	}

	const segT = hi.t === lo.t ? 0 : (t - lo.t) / (hi.t - lo.t);
	const r = Math.round(lerp(lo.r, hi.r, segT));
	const g = Math.round(lerp(lo.g, hi.g, segT));
	const b = Math.round(lerp(lo.b, hi.b, segT));

	return `rgb(${r},${g},${b})`;
}

/**
 * Build a pre-computed lookup table for fast canvas rendering.
 * Returns an array of 256 [r, g, b] values mapping quantized dB levels.
 */
export function buildColorLUT(minDb: number, maxDb: number): Uint8Array {
	const lut = new Uint8Array(256 * 3);
	const range = maxDb - minDb;

	for (let i = 0; i < 256; i++) {
		const t = range === 0 ? 0 : i / 255;

		// Find the two stops we sit between
		let lo = STOPS[0];
		let hi = STOPS[STOPS.length - 1];
		for (let s = 0; s < STOPS.length - 1; s++) {
			if (t >= STOPS[s].t && t <= STOPS[s + 1].t) {
				lo = STOPS[s];
				hi = STOPS[s + 1];
				break;
			}
		}

		const segT = hi.t === lo.t ? 0 : (t - lo.t) / (hi.t - lo.t);
		lut[i * 3] = Math.round(lerp(lo.r, hi.r, segT));
		lut[i * 3 + 1] = Math.round(lerp(lo.g, hi.g, segT));
		lut[i * 3 + 2] = Math.round(lerp(lo.b, hi.b, segT));
	}

	return lut;
}
