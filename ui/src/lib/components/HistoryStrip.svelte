<script lang="ts">
	/**
	 * HistoryStrip — rolling window of recent input + harmony notes
	 * rendered as sheet-music-style canvas. Two staves: top = melody in
	 * (teal), bottom = harmony out (magenta). Notes connect with a line so
	 * you can see the interval motion over time. Playhead at the right edge.
	 *
	 * The engine exposes "currently sounding" note arrays (snapshots).
	 * We diff those snapshots on every change and push each new arrival
	 * into a FIFO buffer keyed to the arrival timestamp, then draw.
	 */
	import { onMount, onDestroy } from 'svelte';
	import { engine } from '$lib/stores/engine.svelte';
	import { ui } from '$lib/stores/ui.svelte';

	const WINDOW = 16;
	const MIN_MIDI = 40; // E2
	const MAX_MIDI = 88; // E6

	// Canvas pixel dimensions. Height mirrors the Fretboard (150) so the
	// two instruments visually stack as equals. Width is fluid via CSS.
	const H = 150;

	type Kind = 'input' | 'harmony';
	interface Entry { kind: Kind; midi: number; ts: number; }

	let history = $state<Entry[]>([]);
	let prevInput: Set<number> = new Set();
	let prevHarmony: Set<number> = new Set();

	function push(kind: Kind, midi: number) {
		history = [...history.slice(-(WINDOW - 1)), { kind, midi, ts: Date.now() }];
	}

	$effect(() => {
		const curr = new Set(engine.inputNotes);
		for (const m of curr) if (!prevInput.has(m)) push('input', m);
		prevInput = curr;
	});

	$effect(() => {
		const curr = new Set(engine.harmonyNotes);
		for (const m of curr) if (!prevHarmony.has(m)) push('harmony', m);
		prevHarmony = curr;
	});

	// ===== Canvas draw =====

	let canvasRef: HTMLCanvasElement | null = null;
	let rafId: number | null = null;

	function draw() {
		const c = canvasRef;
		if (!c) return;
		const ctxOrNull = c.getContext('2d');
		if (!ctxOrNull) return;
		// Narrow once so nested closures (drawVoice) see the non-null type.
		const ctx: CanvasRenderingContext2D = ctxOrNull;

		const dpr = window.devicePixelRatio || 1;
		const rect = c.getBoundingClientRect();
		const W = Math.max(rect.width, 400);
		if (c.width !== W * dpr || c.height !== H * dpr) {
			c.width = W * dpr;
			c.height = H * dpr;
			ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
		}

		ctx.clearRect(0, 0, W, H);

		// Background — match the fretboard wood-on-void treatment
		ctx.fillStyle = '#0f0e1a';
		ctx.fillRect(0, 0, W, H);

		// Two stave bands (top = melody, bottom = harmony). 5 lines each.
		const bandPad = 18;
		const topY = bandPad;
		const bandH = (H - bandPad * 2) / 2;
		const bottomY = bandPad + bandH + 6;
		const lineSpacing = (bandH - 20) / 4; // 5 lines → 4 spaces
		const staffPad = 56; // room on the left for the CLEF labels

		ctx.strokeStyle = 'rgba(245,233,201,0.16)';
		ctx.lineWidth = 1;
		for (let band = 0; band < 2; band++) {
			const y0 = band === 0 ? topY : bottomY;
			for (let i = 0; i < 5; i++) {
				const y = y0 + 10 + i * lineSpacing;
				ctx.beginPath();
				ctx.moveTo(staffPad, y);
				ctx.lineTo(W - 10, y);
				ctx.stroke();
			}
		}

		// Clef-style voice labels
		ctx.font = '9px "JetBrains Mono", ui-monospace, monospace';
		ctx.fillStyle = '#4fe8c3';
		ctx.fillText('MELODY · IN',  10, topY + 10 + lineSpacing * 1 + 3);
		ctx.fillStyle = '#ff2e88';
		ctx.fillText('HARMONY · OUT', 10, bottomY + 10 + lineSpacing * 1 + 3);

		// Fade overlay left edge so older notes wash out
		const fadeW = 80;
		const grad = ctx.createLinearGradient(staffPad, 0, staffPad + fadeW, 0);
		grad.addColorStop(0, '#0f0e1a');
		grad.addColorStop(1, 'rgba(15,14,26,0)');

		if (history.length === 0) {
			// Playhead only — staves wait for input
			ctx.strokeStyle = 'rgba(245,233,201,0.35)';
			ctx.lineWidth = 1;
			ctx.beginPath();
			ctx.moveTo(W - 12, topY);
			ctx.lineTo(W - 12, bottomY + bandH);
			ctx.stroke();
			return;
		}

		// Place each entry along the time axis. Index (older → newer)
		// maps linearly to x from staffPad → W - 14.
		const usableW = W - 14 - staffPad;
		const xFor = (i: number, total: number) =>
			staffPad + (total <= 1 ? usableW : (i / (total - 1)) * usableW);

		/** y for a midi within one of the bands. Each band spans MIN_MIDI..MAX_MIDI. */
		function yFor(midi: number, band: 0 | 1): number {
			const y0 = band === 0 ? topY : bottomY;
			const clamped = Math.max(MIN_MIDI, Math.min(MAX_MIDI, midi));
			const norm = 1 - (clamped - MIN_MIDI) / (MAX_MIDI - MIN_MIDI);
			return y0 + 8 + norm * (bandH - 16);
		}

		// Split entries by voice keeping their global ordering (needed for
		// connecting lines — consecutive notes of the same voice link up).
		const inputEntries:  { i: number; e: Entry }[] = [];
		const harmEntries:   { i: number; e: Entry }[] = [];
		history.forEach((e, i) => {
			if (e.kind === 'input')   inputEntries.push({ i, e });
			if (e.kind === 'harmony') harmEntries.push({ i, e });
		});

		function drawVoice(list: { i: number; e: Entry }[], band: 0 | 1, color: string) {
			if (list.length === 0) return;

			// Connecting line through note heads (shows interval motion)
			ctx.strokeStyle = color;
			ctx.globalAlpha = 0.7;
			ctx.lineWidth = 1.5;
			ctx.shadowColor = color;
			ctx.shadowBlur = 6;
			ctx.beginPath();
			list.forEach((p, j) => {
				const x = xFor(p.i, history.length);
				const y = yFor(p.e.midi, band);
				if (j === 0) ctx.moveTo(x, y);
				else ctx.lineTo(x, y);
			});
			ctx.stroke();
			ctx.shadowBlur = 0;

			// Note heads — brighter for recent, fading toward the left
			list.forEach((p) => {
				const x = xFor(p.i, history.length);
				const y = yFor(p.e.midi, band);
				const freshness = p.i / Math.max(history.length - 1, 1);
				ctx.globalAlpha = 0.4 + freshness * 0.6;
				ctx.fillStyle = color;
				ctx.shadowColor = color;
				ctx.shadowBlur = 6 + freshness * 10;
				ctx.fillRect(x - 3, y - 3, 6, 6);
				ctx.shadowBlur = 0;
			});
			ctx.globalAlpha = 1;

			// Labels, gated on toggle
			if (ui.showNoteLabels) {
				ctx.fillStyle = color;
				ctx.font = '8px "JetBrains Mono", ui-monospace, monospace';
				list.forEach((p) => {
					const x = xFor(p.i, history.length);
					const y = yFor(p.e.midi, band);
					ctx.fillText(midiName(p.e.midi), x + 5, y - 5);
				});
			}
		}

		drawVoice(inputEntries, 0, '#4fe8c3');
		drawVoice(harmEntries,  1, '#ff2e88');

		// Left-edge fade
		ctx.fillStyle = grad;
		ctx.fillRect(staffPad, 0, fadeW, H);

		// Playhead
		ctx.strokeStyle = 'rgba(245,233,201,0.45)';
		ctx.lineWidth = 1;
		ctx.beginPath();
		ctx.moveTo(W - 12, topY);
		ctx.lineTo(W - 12, bottomY + bandH);
		ctx.stroke();
	}

	function midiName(m: number): string {
		const names = ['C', 'C#', 'D', 'D#', 'E', 'F', 'F#', 'G', 'G#', 'A', 'A#', 'B'];
		return names[((m % 12) + 12) % 12] + (Math.floor(m / 12) - 1);
	}

	function tick() {
		draw();
		rafId = requestAnimationFrame(tick);
	}

	onMount(() => {
		tick();
		window.addEventListener('resize', draw);
	});
	onDestroy(() => {
		if (rafId !== null) cancelAnimationFrame(rafId);
		window.removeEventListener('resize', draw);
	});
</script>

<div class="history-strip" aria-label="Recent melody + harmony history">
	<canvas bind:this={canvasRef} class="history-canvas" style:height="{H}px"></canvas>
</div>

<style>
	.history-strip {
		width: 100%;
		background: var(--color-bg-deep);
		border-bottom: 1px solid var(--color-border);
	}

	.history-canvas {
		width: 100%;
		display: block;
		touch-action: none;
	}
</style>
