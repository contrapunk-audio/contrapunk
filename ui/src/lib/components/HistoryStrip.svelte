<script lang="ts">
	/**
	 * HistoryStrip — rolling window of recent notes as a single-staff
	 * sheet-music strip. Every note plots at its MIDI pitch on one shared
	 * staff; color encodes source (teal = what the user played, magenta =
	 * engine-generated harmony, violet = borrowed / modal interchange).
	 *
	 * Any voice can be input — if the user plays the alto line, harmony
	 * fills in soprano/tenor/bass. Treating them as separate staves was
	 * wrong; they share the same pitch axis.
	 */
	import { onMount, onDestroy } from 'svelte';
	import { engine } from '$lib/stores/engine.svelte';
	import { ui } from '$lib/stores/ui.svelte';

	const WINDOW = 16;
	const MIN_MIDI = 40; // E2
	const MAX_MIDI = 88; // E6
	const H = 150;

	type Kind = 'input' | 'harmony' | 'borrowed';
	interface Entry { kind: Kind; midi: number; ts: number; }

	let history = $state<Entry[]>([]);
	let prevInput: Set<number> = new Set();
	let prevHarmony: Set<number> = new Set();
	let prevBorrowed: Set<number> = new Set();

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
	$effect(() => {
		const curr = new Set(engine.borrowedNotes);
		for (const m of curr) if (!prevBorrowed.has(m)) push('borrowed', m);
		prevBorrowed = curr;
	});

	const COLORS: Record<Kind, string> = {
		input: '#4fe8c3',
		harmony: '#ff2e88',
		borrowed: '#8a5cff',
	};

	let canvasRef: HTMLCanvasElement | null = null;
	let rafId: number | null = null;

	function midiName(m: number): string {
		const names = ['C', 'C#', 'D', 'D#', 'E', 'F', 'F#', 'G', 'G#', 'A', 'A#', 'B'];
		return names[((m % 12) + 12) % 12] + (Math.floor(m / 12) - 1);
	}

	function draw() {
		const c = canvasRef;
		if (!c) return;
		const ctxOrNull = c.getContext('2d');
		if (!ctxOrNull) return;
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
		ctx.fillStyle = '#0f0e1a';
		ctx.fillRect(0, 0, W, H);

		// Single staff: 5 lines, centered vertically
		const staffPad = 56;
		const topY = 24;
		const staffH = H - topY - 24;
		const lineSpacing = staffH / 6;

		ctx.strokeStyle = 'rgba(245,233,201,0.16)';
		ctx.lineWidth = 1;
		for (let i = 0; i < 5; i++) {
			const y = topY + lineSpacing * (i + 1);
			ctx.beginPath();
			ctx.moveTo(staffPad, y);
			ctx.lineTo(W - 10, y);
			ctx.stroke();
		}

		// Legend (cream clef label + dot colors)
		ctx.font = '9px "JetBrains Mono", ui-monospace, monospace';
		ctx.fillStyle = '#f5e9c9';
		ctx.fillText('MEMORY', 10, topY + lineSpacing * 2);
		ctx.fillStyle = '#6a5b86';
		ctx.fillText('pitch → time', 10, topY + lineSpacing * 3);

		// Playhead bar on the right edge — always drawn
		ctx.strokeStyle = 'rgba(245,233,201,0.45)';
		ctx.lineWidth = 1;
		ctx.beginPath();
		ctx.moveTo(W - 12, topY);
		ctx.lineTo(W - 12, H - 12);
		ctx.stroke();

		if (history.length === 0) return;

		const usableW = W - 14 - staffPad;
		const xFor = (i: number, total: number) =>
			staffPad + (total <= 1 ? usableW : (i / (total - 1)) * usableW);

		function yFor(midi: number): number {
			const clamped = Math.max(MIN_MIDI, Math.min(MAX_MIDI, midi));
			const norm = 1 - (clamped - MIN_MIDI) / (MAX_MIDI - MIN_MIDI);
			return topY + 6 + norm * (staffH - 12);
		}

		// Draw per-source connecting lines so you can see interval motion
		// within each voice. Separate pass per kind to avoid cross-color lines.
		(['input', 'harmony', 'borrowed'] as Kind[]).forEach((kind) => {
			const entries = history
				.map((e, i) => ({ i, e }))
				.filter((p) => p.e.kind === kind);
			if (entries.length < 2) return;
			const color = COLORS[kind];
			ctx.strokeStyle = color;
			ctx.globalAlpha = 0.55;
			ctx.lineWidth = 1.5;
			ctx.shadowColor = color;
			ctx.shadowBlur = 4;
			ctx.beginPath();
			entries.forEach((p, j) => {
				const x = xFor(p.i, history.length);
				const y = yFor(p.e.midi);
				if (j === 0) ctx.moveTo(x, y);
				else ctx.lineTo(x, y);
			});
			ctx.stroke();
			ctx.shadowBlur = 0;
			ctx.globalAlpha = 1;
		});

		// Note heads with freshness-based alpha + glow
		history.forEach((e, i) => {
			const color = COLORS[e.kind];
			const x = xFor(i, history.length);
			const y = yFor(e.midi);
			const freshness = i / Math.max(history.length - 1, 1);
			ctx.globalAlpha = 0.4 + freshness * 0.6;
			ctx.fillStyle = color;
			ctx.shadowColor = color;
			ctx.shadowBlur = 4 + freshness * 10;
			ctx.fillRect(x - 3.5, y - 3.5, 7, 7);
			ctx.shadowBlur = 0;
		});
		ctx.globalAlpha = 1;

		if (ui.showNoteLabels) {
			ctx.font = '8px "JetBrains Mono", ui-monospace, monospace';
			history.forEach((e, i) => {
				ctx.fillStyle = COLORS[e.kind];
				const x = xFor(i, history.length);
				const y = yFor(e.midi);
				ctx.fillText(midiName(e.midi), x + 5, y - 5);
			});
		}

		// Soft fade on the left edge so oldest notes wash out
		const grad = ctx.createLinearGradient(staffPad, 0, staffPad + 80, 0);
		grad.addColorStop(0, '#0f0e1a');
		grad.addColorStop(1, 'rgba(15,14,26,0)');
		ctx.fillStyle = grad;
		ctx.fillRect(staffPad, 0, 80, H);
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

<div class="history-strip" aria-label="Recent notes — shared-staff memory">
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
