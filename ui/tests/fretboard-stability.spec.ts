/**
 * Regression test for fretboard length-wise (horizontal) and height-wise
 * mutation when an open-string note is shown.
 *
 * Symptom this guards against:
 *   When a note in the open-string register (fret 0, e.g. low E = MIDI 40)
 *   is sounded, the cell-fill <rect> + cell-label <text> mount inside the
 *   SVG between the string label (x=6) and the nut (x=42). Without an
 *   explicit aspect-ratio + containment lock on the wrapper, the SVG's
 *   intrinsic-sizing recalc could ripple outward, resizing the wrapper
 *   and visibly perturbing the fretboard.
 *
 * Fix lives in ui/src/lib/embed/Fretboard.svelte:228-246
 *   .fretboard-wrapper { aspect-ratio: 1040/150; contain: layout paint; }
 *
 * The test directly mutates engine.inputNotes via Vite's module graph
 * to simulate "note arrived" without spinning up the audio routing
 * (which would need real MIDI device selection). Then it samples
 * .fretboard-wrapper and .fretboard-svg bounding rects across many
 * frames during the cell mount → mount-of-second → unmount lifecycle
 * and asserts zero subpixel drift.
 */
import { test, expect } from '@playwright/test';

const NOTE_LOW_E_OPEN = 40; // MIDI for low-E open string
const NOTE_OPEN_A = 45; // MIDI for A string open
const TOLERANCE_PX = 0.25; // sub-half-pixel guard

test.describe('Fretboard layout stability', () => {
	test.beforeEach(async ({ page }) => {
		// Force only-fretboard panel state BEFORE the app reads localStorage.
		// Mirrors the user-reported configuration: only fretboard is active.
		await page.addInitScript(() => {
			localStorage.setItem(
				'contrapunk-panels',
				JSON.stringify({
					midi: false,
					controls: false,
					activeNotes: false,
					history: false,
					fretboard: true,
					piano: false,
					pattern: false
				})
			);
			// Disable particles so its resize handler doesn't cloud the picture.
			localStorage.setItem('contrapunk-fx', 'off');
		});
	});

	test('open-string note does not mutate fretboard wrapper or SVG dimensions', async ({
		page
	}) => {
		await page.goto('/');
		await page.waitForSelector('.fretboard-wrapper', { timeout: 10_000 });

		const result = await page.evaluate(
			async ({ openLowE, openA }) => {
				const wrapper = document.querySelector('.fretboard-wrapper') as HTMLElement;
				const svg = document.querySelector('.fretboard-svg') as SVGSVGElement;
				if (!wrapper || !svg) return { error: 'fretboard not in DOM' };

				const measure = () => {
					const wb = wrapper.getBoundingClientRect();
					const sb = svg.getBoundingClientRect();
					return {
						wx: wb.x,
						wy: wb.y,
						ww: wb.width,
						wh: wb.height,
						sx: sb.x,
						sy: sb.y,
						sw: sb.width,
						sh: sb.height,
						cellCount: document.querySelectorAll('.cell-fill').length
					};
				};

				// Reach into the dev server's module graph to grab the engine
				// store. Direct state mutation is the cleanest way to simulate
				// "note arrived" without engine.start() + MIDI device routing.
				type EngineModule = { engine: { inputNotes: number[] } };
				let engine: EngineModule['engine'];
				try {
					const mod = (await import('/src/lib/stores/engine.svelte.ts')) as EngineModule;
					engine = mod.engine;
				} catch (e) {
					return { error: 'engine import failed: ' + (e as Error).message };
				}

				const samples: Array<{ tag: string } & ReturnType<typeof measure>> = [];
				samples.push({ tag: 'pre', ...measure() });

				// Mount one open-string cell (low E)
				engine.inputNotes = [openLowE];
				for (let i = 0; i < 60; i++) {
					await new Promise((r) => requestAnimationFrame(r));
					samples.push({ tag: `single-${i}`, ...measure() });
				}

				// Mount a second open-string cell (open A)
				engine.inputNotes = [openLowE, openA];
				for (let i = 0; i < 30; i++) {
					await new Promise((r) => requestAnimationFrame(r));
					samples.push({ tag: `dual-${i}`, ...measure() });
				}

				// Release everything
				engine.inputNotes = [];
				for (let i = 0; i < 20; i++) {
					await new Promise((r) => requestAnimationFrame(r));
					samples.push({ tag: `release-${i}`, ...measure() });
				}

				return { samples };
			},
			{ openLowE: NOTE_LOW_E_OPEN, openA: NOTE_OPEN_A }
		);

		if ('error' in result && result.error) throw new Error(result.error);
		const samples = result.samples!;

		// Sanity: cells did mount at some point during the test.
		const peakCellCount = Math.max(...samples.map((s) => s.cellCount));
		expect(peakCellCount).toBeGreaterThanOrEqual(2);

		const baseline = samples[0];
		const drift = samples.map((s) => ({
			tag: s.tag,
			dWw: Math.abs(s.ww - baseline.ww),
			dWh: Math.abs(s.wh - baseline.wh),
			dWx: Math.abs(s.wx - baseline.wx),
			dSw: Math.abs(s.sw - baseline.sw),
			dSh: Math.abs(s.sh - baseline.sh),
			dSx: Math.abs(s.sx - baseline.sx)
		}));

		const maxDrift = drift.reduce(
			(acc, d) => ({
				ww: Math.max(acc.ww, d.dWw),
				wh: Math.max(acc.wh, d.dWh),
				wx: Math.max(acc.wx, d.dWx),
				sw: Math.max(acc.sw, d.dSw),
				sh: Math.max(acc.sh, d.dSh),
				sx: Math.max(acc.sx, d.dSx)
			}),
			{ ww: 0, wh: 0, wx: 0, sw: 0, sh: 0, sx: 0 }
		);

		// Surface the maximums in test output for easy diagnosis.
		console.log(
			`max drift: wrapper(w=${maxDrift.ww.toFixed(3)}, h=${maxDrift.wh.toFixed(3)}, x=${maxDrift.wx.toFixed(3)})`
		);
		console.log(
			`           SVG    (w=${maxDrift.sw.toFixed(3)}, h=${maxDrift.sh.toFixed(3)}, x=${maxDrift.sx.toFixed(3)}) over ${samples.length} samples`
		);

		expect(maxDrift.ww, 'wrapper width must not mutate when open-string note shows').toBeLessThan(
			TOLERANCE_PX
		);
		expect(maxDrift.wh, 'wrapper height must not mutate when open-string note shows').toBeLessThan(
			TOLERANCE_PX
		);
		expect(maxDrift.wx, 'wrapper x position must not shift').toBeLessThan(TOLERANCE_PX);
		expect(maxDrift.sw, 'SVG width must not mutate').toBeLessThan(TOLERANCE_PX);
		expect(maxDrift.sh, 'SVG height must not mutate').toBeLessThan(TOLERANCE_PX);
		expect(maxDrift.sx, 'SVG x position must not shift').toBeLessThan(TOLERANCE_PX);
	});
});
