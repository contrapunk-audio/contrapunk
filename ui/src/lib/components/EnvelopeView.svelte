<script lang="ts">
	/**
	 * ADSR envelope curve preview. Durations are in ms, sustain is 0..1.
	 * Layout: horizontal axis = time, vertical = level. Segments scaled
	 * so the whole envelope fits; total visual width is proportional to
	 * sum of stages (with the sustain shown as a flat segment of fixed
	 * length).
	 */

	let {
		attackMs,
		decayMs,
		sustain,
		releaseMs,
		width = 360,
		height = 96,
		color = 'var(--color-accent-magenta)'
	}: {
		attackMs: number;
		decayMs: number;
		sustain: number;
		releaseMs: number;
		width?: number;
		height?: number;
		color?: string;
	} = $props();

	let canvas: HTMLCanvasElement | undefined = $state();

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

		// Grid midline
		ctx.strokeStyle = '#2a2848';
		ctx.lineWidth = 1;
		ctx.beginPath();
		ctx.moveTo(0, h / 2);
		ctx.lineTo(w, h / 2);
		ctx.stroke();

		// Allocate widths proportionally. Sustain gets a fixed 25% of
		// the canvas so you always see a flat bit regardless of A/D/R.
		const pad = 6;
		const usable = w - pad * 2;
		const sustainWidth = usable * 0.25;
		const timedTotal = Math.max(1, attackMs + decayMs + releaseMs);
		const timedWidth = usable - sustainWidth;
		const aW = (attackMs / timedTotal) * timedWidth;
		const dW = (decayMs / timedTotal) * timedWidth;
		const rW = (releaseMs / timedTotal) * timedWidth;

		const baselineY = h - pad;
		const peakY = pad;
		const sustainY = pad + (h - 2 * pad) * (1 - sustain);

		let x = pad;
		ctx.strokeStyle = getComputedStyle(canvas).color || '#ff3388';
		ctx.lineWidth = 2;
		ctx.lineCap = 'round';
		ctx.lineJoin = 'round';
		ctx.shadowColor = ctx.strokeStyle as string;
		ctx.shadowBlur = 6;

		ctx.beginPath();
		ctx.moveTo(x, baselineY);
		// Attack
		x += aW;
		ctx.lineTo(x, peakY);
		// Decay
		x += dW;
		ctx.lineTo(x, sustainY);
		// Sustain hold
		x += sustainWidth;
		ctx.lineTo(x, sustainY);
		// Release
		x += rW;
		ctx.lineTo(x, baselineY);
		ctx.stroke();
		ctx.shadowBlur = 0;

		// Fill under the curve
		ctx.fillStyle = ctx.strokeStyle as string;
		ctx.globalAlpha = 0.12;
		ctx.lineTo(x, baselineY);
		ctx.lineTo(pad, baselineY);
		ctx.closePath();
		ctx.fill();
		ctx.globalAlpha = 1;

		// Dotted note-off marker where Release begins (end of sustain)
		const releaseStartX = pad + aW + dW + sustainWidth;
		ctx.strokeStyle = '#555577';
		ctx.lineWidth = 1;
		ctx.setLineDash([2, 3]);
		ctx.beginPath();
		ctx.moveTo(releaseStartX, pad);
		ctx.lineTo(releaseStartX, h - pad);
		ctx.stroke();
		ctx.setLineDash([]);
	}

	$effect(() => {
		void attackMs;
		void decayMs;
		void sustain;
		void releaseMs;
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
