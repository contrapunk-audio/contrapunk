/**
 * Live size monitor + auto note-injection sequence.
 *
 * Opens a headed WebKit (Playwright's bundled WebKit, the same engine
 * family Tauri's WKWebView uses on macOS) pointed at the running Vite
 * dev server, attaches ResizeObservers to every layout-relevant
 * element, then runs a scripted note sequence so we can capture
 * exactly which elements resize when note names appear.
 *
 * Why WebKit: Chromium's layout engine handles aspect-ratio + contain
 * differently than WebKit. The user reports flicker in Tauri (WebKit),
 * not Chromium, so we test in the matching engine.
 *
 * Run: node ui/scripts/live-size-monitor.mjs
 *      Press Ctrl+C to stop. Browser closes cleanly.
 */

// WebKit (matches Tauri's WKWebView engine family). Use chromium
// only when explicitly A/B-ing against historical chromium baselines.
import { webkit as engine } from 'playwright';

const URL = 'http://localhost:5173/';

const SELECTORS = [
	'html',
	'body',
	'.app-layout',
	'.tab-strip',
	'.content-area',
	'.column-left',
	'.column-center',
	'.active-notes-strip',
	'.pattern-strip',
	'.piano-area',
	'.fretboard-wrapper',
	'.fretboard-svg',
	'.history-strip',
	'.staff-host',
	'.beat-header',
	'.piano-wrapper',
	'.piano-container',
	'.chord-display',
	'.status-bar',
	'.vignette-overlay',
	'.middle',         // ActiveNotes middle column (chord name + borrowed-from)
	'.active-notes-strip',
	'.borrowed-from'   // mounts conditionally inside ActiveNotes .middle
];

async function main() {
	const browser = await engine.launch({ headless: false, devtools: false });
	const context = await browser.newContext({ viewport: { width: 1400, height: 900 } });
	const page = await context.newPage();

	page.on('console', (msg) => {
		const text = msg.text();
		if (
			text.startsWith('[size]') ||
			text.startsWith('[mount]') ||
			text.startsWith('[unmount]') ||
			text.startsWith('[seq]')
		) {
			console.log(text);
		}
	});
	page.on('pageerror', (err) => console.error('[PAGE ERROR]', err.message));

	await page.addInitScript({
		content: `
			(() => {
				const SELECTORS = ${JSON.stringify(SELECTORS)};
				const t0 = performance.now();
				const ts = () => ((performance.now() - t0) / 1000).toFixed(3);
				const prevSize = new WeakMap();
				const observed = new WeakSet();

				function sizeLog(selector, el) {
					const r = el.getBoundingClientRect();
					const prev = prevSize.get(el);
					const ww = r.width.toFixed(2);
					const hh = r.height.toFixed(2);
					const xx = r.x.toFixed(2);
					const yy = r.y.toFixed(2);
					if (prev) {
						const dw = (r.width - prev.w).toFixed(2);
						const dh = (r.height - prev.h).toFixed(2);
						const dx = (r.x - prev.x).toFixed(2);
						const dy = (r.y - prev.y).toFixed(2);
						console.log('[size] +' + ts() + 's  ' + selector +
							'  w=' + ww + ' h=' + hh + ' x=' + xx + ' y=' + yy +
							'  Δw=' + dw + ' Δh=' + dh + ' Δx=' + dx + ' Δy=' + dy);
					} else {
						console.log('[size] +' + ts() + 's  ' + selector +
							'  w=' + ww + ' h=' + hh + ' x=' + xx + ' y=' + yy + '  (initial)');
					}
					prevSize.set(el, { w: r.width, h: r.height, x: r.x, y: r.y });
				}

				function boot() {
					const ro = new ResizeObserver((entries) => {
						for (const entry of entries) {
							const sel = entry.target.dataset.monitorSelector || '?';
							sizeLog(sel, entry.target);
						}
					});

					function attach() {
						for (const sel of SELECTORS) {
							const els = document.querySelectorAll(sel);
							for (const el of els) {
								if (observed.has(el)) continue;
								observed.add(el);
								el.dataset.monitorSelector = sel;
								ro.observe(el);
								sizeLog(sel, el);
								console.log('[mount] +' + ts() + 's  ' + sel);
							}
						}
					}

					const mo = new MutationObserver((mutations) => {
						for (const m of mutations) {
							for (const node of m.removedNodes) {
								if (node instanceof Element) {
									const sel = node.dataset && node.dataset.monitorSelector;
									if (sel) console.log('[unmount] +' + ts() + 's  ' + sel);
								}
							}
						}
						attach();
					});
					mo.observe(document.body, { childList: true, subtree: true });
					attach();
					console.log('[size] +' + ts() + 's  monitor armed; selectors=' + SELECTORS.length);
				}

				if (document.readyState === 'loading') {
					document.addEventListener('DOMContentLoaded', boot, { once: true });
				} else {
					boot();
				}
			})();
		`
	});

	await page.goto(URL, { waitUntil: 'domcontentloaded' });
	await page.waitForSelector('.fretboard-wrapper', { timeout: 10_000 });

	console.log('\n========================================================');
	console.log('Browser open. Auto-injecting note sequence in 3 seconds');
	console.log('so the size monitor catches what mutates as note names');
	console.log('appear. Watch the Chromium window and the log below.');
	console.log('========================================================\n');

	await page.waitForTimeout(3000);

	// Auto-injection sequence — separate from user-driven interaction.
	// Each step pauses long enough that any animation (220ms fret-glitch-in)
	// has settled before the next mutation.
	await page.evaluate(async () => {
		const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
		const mod = await import('/src/lib/stores/engine.svelte.ts');
		const engine = mod.engine;

		const step = (label, fn) => {
			console.log('[seq] ▶ ' + label);
			fn();
		};

		await sleep(500);

		step('1: open low-E only [40] — open-string cell-label appears', () => {
			engine.inputNotes = [40];
		});
		await sleep(1500);

		step('2: + open-A [40, 45] — second open-string cell-label', () => {
			engine.inputNotes = [40, 45];
		});
		await sleep(1500);

		step('3: C major chord [60, 64, 67] + chordName="C"', () => {
			engine.inputNotes = [60, 64, 67];
			engine.chordName = 'C';
		});
		await sleep(1500);

		step('4: chord text widens → "Cmaj7" (text-width effect)', () => {
			engine.chordName = 'Cmaj7';
		});
		await sleep(1500);

		step('5: chord text widens → "Cmaj7add9/E" (longest)', () => {
			engine.chordName = 'Cmaj7add9/E';
		});
		await sleep(1500);

		step('6: lastBorrowedFrom = "Phrygian Dominant" (mounts borrowed-from line)', () => {
			engine.lastBorrowedFrom = 'Phrygian Dominant';
		});
		await sleep(1500);

		step('7: clear lastBorrowedFrom (unmounts the line)', () => {
			engine.lastBorrowedFrom = '';
		});
		await sleep(1500);

		step('8a: many notes flooding fretboard — 12 input + 12 harmony cells', () => {
			// Cover most of the fretboard: low E open through high E 12th fret
			engine.inputNotes = [40, 45, 50, 55, 59, 64, 52, 57, 62, 67, 71, 76];
			engine.harmonyNotes = [42, 47, 52, 57, 61, 66, 54, 59, 64, 69, 73, 78];
		});
		await sleep(2000);

		step('8b: borrowed-notes flood', () => {
			engine.borrowedNotes = [44, 49, 54, 56, 61];
		});
		await sleep(2000);

		step('8c: clear flood', () => {
			engine.inputNotes = [];
			engine.harmonyNotes = [];
			engine.borrowedNotes = [];
		});
		await sleep(1500);

		step('8: G chord [67, 71, 74] + chordName="G" (chord change while sounding)', () => {
			engine.inputNotes = [67, 71, 74];
			engine.chordName = 'G';
		});
		await sleep(1500);

		step('9: release everything', () => {
			engine.inputNotes = [];
			engine.harmonyNotes = [];
			engine.chordName = '';
		});
		await sleep(1000);

		console.log('[seq] ✓ sequence complete');
	});

	console.log('\n========================================================');
	console.log('Sequence complete. Browser stays open for manual driving.');
	console.log('Press Ctrl+C in this terminal to close.');
	console.log('========================================================\n');

	await new Promise((resolve) => {
		process.on('SIGINT', resolve);
		process.on('SIGTERM', resolve);
	});

	await browser.close();
}

main().catch((err) => {
	console.error('FATAL:', err);
	process.exit(1);
});
