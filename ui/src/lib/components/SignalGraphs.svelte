<script lang="ts">
	import { guitar } from '$lib/stores/guitar.svelte';

	let ampCanvas: HTMLCanvasElement;
	let animFrame: number | null = null;

	const WIDTH = 240;
	const HEIGHT = 48;

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
			const noiseThreshold = guitar.noiseGateEnabled
				? guitar.noiseGateThreshold * 5 // scale to match signalLevel (rms * 5)
				: null;
			drawGraph(ampCanvas, guitar.amplitudeHistory, noiseThreshold, '#00e5cc', '#ffaa00');
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
		<span class="graph-label font-pixel">AMP</span>
		<canvas bind:this={ampCanvas} width={WIDTH} height={HEIGHT} class="signal-canvas"></canvas>
		<div class="gate-controls">
			<button
				class="gate-btn font-pixel"
				class:gate-on={guitar.noiseGateEnabled}
				onclick={() => { guitar.noiseGateEnabled = !guitar.noiseGateEnabled; }}
				title="Noise Gate"
			>NG</button>
			<input
				type="range"
				class="gate-slider"
				min="0.001"
				max="0.1"
				step="0.001"
				value={guitar.noiseGateThreshold}
				oninput={(e) => { guitar.noiseGateThreshold = parseFloat((e.target as HTMLInputElement).value); }}
				title={`Threshold: ${(guitar.noiseGateThreshold * 1000).toFixed(0)} mRMS`}
			/>
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

	.gate-controls {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 2px;
		width: 32px;
	}

	.gate-btn {
		font-size: var(--font-size-xs);
		padding: 2px 4px;
		background: var(--color-widget-inactive);
		border: 1px solid var(--color-border);
		color: var(--color-text-dim);
		cursor: pointer;
	}

	.gate-btn.gate-on {
		background: var(--color-accent-teal);
		color: #fff;
		border-color: var(--color-accent-cyan-dim);
	}

	.gate-slider {
		width: 28px;
		height: 8px;
		-webkit-appearance: none;
		appearance: none;
		background: var(--color-bg-panel);
		border: 1px solid var(--color-border);
		outline: none;
		cursor: pointer;
	}

	.gate-slider::-webkit-slider-thumb {
		-webkit-appearance: none;
		width: 6px;
		height: 10px;
		background: var(--color-accent-amber);
		border: none;
		cursor: pointer;
	}
</style>
