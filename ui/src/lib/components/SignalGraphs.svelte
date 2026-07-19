<script lang="ts">
	import { guitar } from '$lib/stores/guitar.svelte';

	let ampCanvas: HTMLCanvasElement;
	let animFrame: number | null = null;

	const WIDTH = 240;
	const HEIGHT = 48;
	// Shipping onset threshold (0.02 RMS), scaled like signalLevel (RMS × 5).
	const TRIGGER_LINE = 0.1;

	function drawGraph(
		canvas: HTMLCanvasElement,
		data: number[],
		threshold: number | null,
		color: string,
		thresholdColor: string
	) {
		const ctx = canvas.getContext('2d');
		if (!ctx) return;

		ctx.clearRect(0, 0, WIDTH, HEIGHT);

		// Draw filled signal area
		if (data.length > 1) {
			ctx.beginPath();
			ctx.moveTo(0, HEIGHT);
			for (let i = 0; i < data.length; i++) {
				const x = (i / (data.length - 1)) * WIDTH;
				const y = HEIGHT - data[i] * HEIGHT;
				ctx.lineTo(x, y);
			}
			ctx.lineTo(WIDTH, HEIGHT);
			ctx.closePath();
			ctx.fillStyle = color + '40'; // semi-transparent fill
			ctx.fill();

			// Draw signal line on top
			ctx.beginPath();
			for (let i = 0; i < data.length; i++) {
				const x = (i / (data.length - 1)) * WIDTH;
				const y = HEIGHT - data[i] * HEIGHT;
				if (i === 0) ctx.moveTo(x, y);
				else ctx.lineTo(x, y);
			}
			ctx.strokeStyle = color;
			ctx.lineWidth = 1.5;
			ctx.stroke();
		}

		// Draw threshold line
		if (threshold !== null && threshold > 0) {
			const ty = HEIGHT - threshold * HEIGHT;
			ctx.beginPath();
			ctx.moveTo(0, ty);
			ctx.lineTo(WIDTH, ty);
			ctx.strokeStyle = thresholdColor;
			ctx.lineWidth = 1;
			ctx.setLineDash([4, 3]);
			ctx.stroke();
			ctx.setLineDash([]);
		}
	}

	function render() {
		if (ampCanvas) {
			drawGraph(ampCanvas, guitar.amplitudeHistory, TRIGGER_LINE, '#00e5cc', '#ffaa00');
		}
		animFrame = requestAnimationFrame(render);
	}

	$effect(() => {
		if (guitar.detecting && ampCanvas) {
			if (!animFrame) render();
		}
		return () => {
			if (animFrame) {
				cancelAnimationFrame(animFrame);
				animFrame = null;
			}
		};
	});
</script>

<div class="graphs-container">
	<div class="graph-row">
		<span class="graph-label font-ui">AMP</span>
		<canvas bind:this={ampCanvas} width={WIDTH} height={HEIGHT} class="signal-canvas"></canvas>
		<div class="trigger-legend font-ui" title="Fixed Phase 10.1 onset threshold: 20 mRMS">
			<span class="trigger-swatch"></span>
			TRIGGER
		</div>
	</div>
</div>

<style>
	.graphs-container {
		display: flex;
		flex-direction: column;
		gap: 3px;
		margin: 4px 0;
	}

	.graph-row {
		display: flex;
		align-items: center;
		gap: 4px;
	}

	.graph-label {
		font-size: var(--font-size-xs);
		color: var(--color-text-dim);
		width: 18px;
		text-align: right;
	}

	.signal-canvas {
		flex: 1;
		height: 48px;
		background: var(--color-bg-dark, #0a0a0a);
		border: 1px solid var(--color-border);
		image-rendering: pixelated;
	}

	.trigger-legend {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: 3px;
		width: 42px;
		font-size: 9px;
		color: var(--color-text-secondary);
	}

	.trigger-swatch {
		width: 22px;
		border-top: 1px dashed var(--color-accent-amber);
	}
</style>
