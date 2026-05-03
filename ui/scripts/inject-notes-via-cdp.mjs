/**
 * One-shot helper: connects to the headed Chromium opened by
 * live-size-monitor.mjs (via the CDP endpoint Playwright exposes
 * implicitly) and injects a note sequence into engine.inputNotes.
 *
 * Use this WHILE live-size-monitor.mjs is running to trigger note
 * visuals so the size monitor catches any resulting mutation.
 *
 * Usage: node ui/scripts/inject-notes-via-cdp.mjs
 *
 * Limitation: this opens its own browser context and navigates to
 * localhost:5173. So it doesn't share state with the existing
 * Chromium — instead it serves as a SEPARATE controlled browser
 * to capture the mutation. Open this Chromium beside the existing
 * one to see what happens visually.
 */

import { chromium } from 'playwright';

const URL = 'http://localhost:5173/';

async function main() {
	const browser = await chromium.launch({ headless: false, devtools: true });
	const context = await browser.newContext({ viewport: { width: 1400, height: 900 } });
	const page = await context.newPage();

	// Same panel config as live-size-monitor (default = all panels visible).
	// If you want only the fretboard, uncomment the next block:
	// await page.addInitScript(() => {
	// 	localStorage.setItem(
	// 		'contrapunk-panels',
	// 		JSON.stringify({ midi: false, controls: false, activeNotes: false, history: false, fretboard: true, piano: false, pattern: false })
	// 	);
	// });

	page.on('pageerror', (err) => console.error('[PAGE ERROR]', err.message));

	await page.goto(URL, { waitUntil: 'domcontentloaded' });
	await page.waitForSelector('.fretboard-wrapper', { timeout: 10_000 });

	// Inject a sequence of note events via direct engine state mutation.
	// Each step is a different note configuration; we wait between steps
	// so the user (and any size monitor running in parallel) can react.
	const result = await page.evaluate(async () => {
		const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
		const mod = await import('/src/lib/stores/engine.svelte.ts');
		const engine = mod.engine;
		const log = [];

		// Step 1: open low E (40) — open string, lowest fret
		log.push('1: open low-E [40]');
		engine.inputNotes = [40];
		await sleep(800);

		// Step 2: + open A (45)
		log.push('2: low-E + open-A [40, 45]');
		engine.inputNotes = [40, 45];
		await sleep(800);

		// Step 3: a chord that would trigger chord detection (C major: 60, 64, 67)
		log.push('3: C major chord [60, 64, 67]');
		engine.inputNotes = [60, 64, 67];
		// Also fake the chordName since auto-detection only runs in routing mode
		engine.chordName = 'C';
		await sleep(1500);

		// Step 4: change chord (G major: 67, 71, 74)
		log.push('4: G major chord [67, 71, 74]');
		engine.inputNotes = [67, 71, 74];
		engine.chordName = 'G';
		await sleep(1500);

		// Step 5: very long chord name to test text-width effects
		log.push('5: long chord name "Cmaj7add9/E"');
		engine.chordName = 'Cmaj7add9/E';
		await sleep(1500);

		// Step 6: borrowed note label (modal interchange)
		log.push('6: lastBorrowedFrom = "Phrygian Dominant"');
		engine.lastBorrowedFrom = 'Phrygian Dominant';
		await sleep(1500);

		// Step 7: clear borrowed
		log.push('7: clear lastBorrowedFrom');
		engine.lastBorrowedFrom = '';
		await sleep(1000);

		// Step 8: release everything
		log.push('8: release all');
		engine.inputNotes = [];
		engine.harmonyNotes = [];
		engine.chordName = '';
		await sleep(1000);

		return log;
	});

	console.log('\nSequence completed:');
	for (const l of result) console.log('  ' + l);

	console.log('\nBrowser left open. Close manually when done. Press Ctrl+C in this terminal.');
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
