/**
 * Empirical probe: does the fretboard wrapper mutate horizontally
 * when an open-string note is shown?
 *
 * Boots a Chromium pointed at the running Vite dev server, configures
 * only the fretboard panel, then injects a note and samples
 * .fretboard-wrapper bounds across ~60 frames. Reports any width or
 * x mutation ≥ 0.5px.
 *
 * Run: node ui/scripts/probe-fretboard-mutation.mjs
 */

import { chromium } from 'playwright';

const URL = 'http://localhost:5173/';
const SAMPLE_FRAMES = 60;
const NOTE_MIDI_LOW_E = 40; // open low-E on guitar
const NOTE_MIDI_OPEN_A = 45; // open A
const NOTE_MIDI_C4 = 60; // mid-piano

async function main() {
  const browser = await chromium.launch({ headless: true });
  const context = await browser.newContext({ viewport: { width: 1400, height: 900 } });
  const page = await context.newPage();

  page.on('pageerror', (err) => console.error('[PAGE ERROR]', err.message));
  page.on('console', (msg) => {
    if (msg.type() === 'error') console.error('[CONSOLE ERROR]', msg.text());
  });

  // Force only-fretboard panel state BEFORE the app reads localStorage.
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
        pattern: false,
      })
    );
    // Disable particles to keep canvas resize handler quiet during probe.
    localStorage.setItem('contrapunk-fx', 'off');
  });

  await page.goto(URL, { waitUntil: 'domcontentloaded' });

  // Wait for the fretboard to render.
  await page.waitForSelector('.fretboard-wrapper', { timeout: 10000 });

  // Sample bounds before any note plays — establishes the baseline.
  const baseline = await page.evaluate(() => {
    const w = document.querySelector('.fretboard-wrapper');
    const s = document.querySelector('.fretboard-svg');
    const wb = w.getBoundingClientRect();
    const sb = s.getBoundingClientRect();
    return {
      wrapper: { x: wb.x, y: wb.y, w: wb.width, h: wb.height },
      svg: { x: sb.x, y: sb.y, w: sb.width, h: sb.height },
      vw: window.innerWidth,
      vh: window.innerHeight,
    };
  });

  console.log('--- Baseline (no note) ---');
  console.log(JSON.stringify(baseline, null, 2));

  // Now inject an open-string E2 note via the engine store directly.
  // Mimics what adapter.injectNoteOn does, but bypasses the routing
  // start requirement so the visual cell mounts purely from a state
  // mutation of engine.inputNotes.
  //
  // We mutate engine.inputNotes directly because the adapter requires
  // the routing engine to be started, which needs MIDI device selection.
  // For pure visual-side measurement, mutating the store is sufficient
  // — Fretboard's render reads engine.inputNotes via $derived/$state.
  const samples = await page.evaluate(
    async ({ frames, openLowE, openA, midC }) => {
      const wrapper = document.querySelector('.fretboard-wrapper');
      const svg = document.querySelector('.fretboard-svg');
      const measure = () => {
        const wb = wrapper.getBoundingClientRect();
        const sb = svg.getBoundingClientRect();
        return {
          t: performance.now(),
          wx: wb.x, wy: wb.y, ww: wb.width, wh: wb.height,
          sx: sb.x, sy: sb.y, sw: sb.width, sh: sb.height,
          // Also check if a cell-fill at fret 0 is in the DOM.
          openCellsCount: document.querySelectorAll('.cell-fill').length,
        };
      };

      const samples = [];

      // Reach into the Vite dev server's module graph to grab the engine
      // store directly. Mutating inputNotes is the cleanest way to
      // simulate "note arrived" without spinning up the audio routing.
      let engine;
      try {
        const mod = await import('/src/lib/stores/engine.svelte.ts');
        engine = mod.engine;
      } catch (e) {
        return { error: 'failed to import engine store: ' + e.message, samples };
      }
      if (!engine) return { error: 'engine store import returned undefined', samples };

      // Sample baseline (no notes)
      samples.push({ tag: 'pre', ...measure() });

      // Open low-E (MIDI 40) is the lowest open string on standard tuning
      // and the one that paints in the cell area between the string label
      // and the nut — the case the user flagged.
      engine.inputNotes = [openLowE];

      // Sample for N frames while the cell is mounted (animation runs ~220ms)
      for (let i = 0; i < frames; i++) {
        await new Promise((r) => requestAnimationFrame(r));
        samples.push({ tag: `frame-${i}`, ...measure() });
      }

      // Add open A (45) — second cell mounts in the same column
      engine.inputNotes = [openLowE, openA];
      for (let i = 0; i < 20; i++) {
        await new Promise((r) => requestAnimationFrame(r));
        samples.push({ tag: `dual-${i}`, ...measure() });
      }

      // Release everything
      engine.inputNotes = [];

      // A few more frames after release
      for (let i = 0; i < 15; i++) {
        await new Promise((r) => requestAnimationFrame(r));
        samples.push({ tag: `release-${i}`, ...measure() });
      }

      return { samples };
    },
    { frames: SAMPLE_FRAMES, openLowE: NOTE_MIDI_LOW_E, openA: NOTE_MIDI_OPEN_A, midC: NOTE_MIDI_C4 }
  );

  if (samples.error) {
    console.error('PROBE FAILED:', samples.error);
    if (samples.samples) console.log(JSON.stringify(samples.samples.slice(0, 3), null, 2));
    await browser.close();
    process.exitCode = 2;
    return;
  }

  // Analyze: find the max delta vs baseline.
  const ww0 = samples.samples[0].ww;
  const wh0 = samples.samples[0].wh;
  const wx0 = samples.samples[0].wx;
  const sw0 = samples.samples[0].sw;
  const sh0 = samples.samples[0].sh;
  const sx0 = samples.samples[0].sx;

  let maxWrapperWDelta = 0;
  let maxWrapperHDelta = 0;
  let maxWrapperXDelta = 0;
  let maxSvgWDelta = 0;
  let maxSvgHDelta = 0;
  let maxSvgXDelta = 0;
  let maxCells = 0;

  for (const s of samples.samples) {
    maxWrapperWDelta = Math.max(maxWrapperWDelta, Math.abs(s.ww - ww0));
    maxWrapperHDelta = Math.max(maxWrapperHDelta, Math.abs(s.wh - wh0));
    maxWrapperXDelta = Math.max(maxWrapperXDelta, Math.abs(s.wx - wx0));
    maxSvgWDelta = Math.max(maxSvgWDelta, Math.abs(s.sw - sw0));
    maxSvgHDelta = Math.max(maxSvgHDelta, Math.abs(s.sh - sh0));
    maxSvgXDelta = Math.max(maxSvgXDelta, Math.abs(s.sx - sx0));
    maxCells = Math.max(maxCells, s.openCellsCount);
  }

  console.log('\n--- Mutation report (sampled across ~70 frames; press → release low-E open string) ---');
  console.log(`baseline wrapper: w=${ww0.toFixed(2)} h=${wh0.toFixed(2)} x=${wx0.toFixed(2)}`);
  console.log(`baseline SVG:     w=${sw0.toFixed(2)} h=${sh0.toFixed(2)} x=${sx0.toFixed(2)}`);
  console.log(`max wrapper Δw: ${maxWrapperWDelta.toFixed(3)}px`);
  console.log(`max wrapper Δh: ${maxWrapperHDelta.toFixed(3)}px`);
  console.log(`max wrapper Δx: ${maxWrapperXDelta.toFixed(3)}px`);
  console.log(`max SVG     Δw: ${maxSvgWDelta.toFixed(3)}px`);
  console.log(`max SVG     Δh: ${maxSvgHDelta.toFixed(3)}px`);
  console.log(`max SVG     Δx: ${maxSvgXDelta.toFixed(3)}px`);
  console.log(`peak cell-fill count: ${maxCells}`);

  // Pretty-print frames where ANY dimension drifted ≥ 0.25px.
  const drifted = samples.samples.filter(
    (s) =>
      Math.abs(s.ww - ww0) >= 0.25 ||
      Math.abs(s.wh - wh0) >= 0.25 ||
      Math.abs(s.wx - wx0) >= 0.25 ||
      Math.abs(s.sw - sw0) >= 0.25 ||
      Math.abs(s.sh - sh0) >= 0.25 ||
      Math.abs(s.sx - sx0) >= 0.25
  );
  console.log(`\nFrames with ≥0.25px drift in any dimension: ${drifted.length} / ${samples.samples.length}`);
  if (drifted.length > 0) {
    console.log('First 5 drifted frames:');
    drifted.slice(0, 5).forEach((s) => {
      console.log(
        `  ${s.tag}: ww=${s.ww.toFixed(3)} wh=${s.wh.toFixed(3)} wx=${s.wx.toFixed(3)} sw=${s.sw.toFixed(3)} sh=${s.sh.toFixed(3)} sx=${s.sx.toFixed(3)}`
      );
    });
  }

  await browser.close();
}

main().catch((err) => {
  console.error('FATAL:', err);
  process.exit(1);
});
