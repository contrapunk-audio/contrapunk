<script lang="ts">
	/**
	 * Waveform preview. Renders two cycles of the selected oscillator
	 * shape to a canvas. Updates reactively when waveform changes.
	 */
	import { onMount } from 'svelte';

	let {
		waveform,
		width = 360,
		height = 96,
		color = 'var(--color-accent-cyan)'
	}: {
		waveform: number;
		width?: number;
		height?: number;
		color?: string;
	} = $props();

	let canvas: HTMLCanvasElement | undefined = $state();
	let resolvedColor = $state('#33ddff');
	let resolvedSecondary = $state('#2a2848');

	onMount(() => {
		const cs = getComputedStyle(canvas!);
		// Resolve color once. CSS var may be dynamic but the canvas text
		// update reads the computed values when draw runs.
		resolvedColor = cs.color || '#33ddff';
	});

	function waveValue(wf: number, phase: number): number {
		// phase in [0, 2π]
		const TAU = Math.PI * 2;
		switch (wf) {
			case 1:
				return phase / Math.PI - 1;
			case 2:
				return phase < Math.PI ? 1 : -1;
			case 3: {
				const p = phase / Math.PI;
				return p < 1 ? 2 * p - 1 : 3 - 2 * p;
			}
			default:
				return Math.sin(phase);
		}
	}

	function draw() {
		if (!canvas) return;
		const ctx = canvas.getContext('2d');
		if (!ctx) return;
		const dpr = window.devicePixelRatio || 1;
		const w = canvas.clientWidth;
		const h = canvas.clientHeight;
		canvas.width = Math.floor(w * dpr);
		canvas.height = Math.floor(h * dpr);
		ctx.scale(dpr, dpr);

		// Background
		ctx.fillStyle = '#0f0e1a';
		ctx.fillRect(0, 0, w, h);

		// Grid center line
		ctx.strokeStyle = resolvedSecondary;
		ctx.lineWidth = 1;
		ctx.beginPath();
		ctx.moveTo(0, h / 2);
		ctx.lineTo(w, h / 2);
		ctx.stroke();

		// Waveform (two cycles)
		ctx.strokeStyle = resolvedColor;
		ctx.lineWidth = 2;
		ctx.lineCap = 'round';
		ctx.lineJoin = 'round';
		ctx.shadowColor = resolvedColor;
		ctx.shadowBlur = 6;
		ctx.beginPath();
		const cycles = 2;
		for (let x = 0; x <= w; x++) {
			const t = (x / w) * cycles * Math.PI * 2;
			const v = waveValue(waveform, t % (Math.PI * 2));
			const y = h / 2 - v * (h / 2 - 6);
			if (x === 0) ctx.moveTo(x, y);
			else ctx.lineTo(x, y);
		}
		ctx.stroke();
		ctx.shadowBlur = 0;

		// Subtle fill below
		ctx.fillStyle = `${resolvedColor}`.replace(')', ', 0.08)').replace('rgb', 'rgba');
		ctx.globalAlpha = 0.15;
		ctx.lineTo(w, h);
		ctx.lineTo(0, h);
		ctx.closePath();
		ctx.fill();
		ctx.globalAlpha = 1;
	}

	// Redraw when waveform prop changes. Also on mount.
	$effect(() => {
		// Re-resolve color after initial render (CSS vars resolve in browser)
		if (canvas) {
			const cs = getComputedStyle(canvas);
			resolvedColor = cs.color;
		}
		// Explicit dep on waveform to retrigger
		void waveform;
		draw();
	});
</script>

<canvas
	bind:this={canvas}
	style:width="{width}px"
	style:height="{height}px"
	style:color={color}
></canvas>

<style>
	canvas {
		display: block;
		width: 100%;
		background: #0f0e1a;
		border: 1px solid var(--color-border);
	}
</style>
