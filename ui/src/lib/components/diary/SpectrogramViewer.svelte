<script lang="ts">
	import { buildColorLUT } from '$lib/components/diary/colorScale';
	import { audioStore } from '$lib/components/diary/audioContext.svelte';

	let {
		data,
		height = 200,
		label = '',
		audioUrl = '',
	}: {
		data: number[][];
		height?: number;
		label?: string;
		audioUrl?: string;
	} = $props();

	let canvasEl: HTMLCanvasElement | undefined = $state();
	let containerEl: HTMLDivElement | undefined = $state();

	// Tooltip state
	let tooltipVisible = $state(false);
	let tooltipX = $state(0);
	let tooltipY = $state(0);
	let tooltipFreqBin = $state(0);
	let tooltipTimeFrame = $state(0);
	let tooltipDb = $state(0);

	// Compute dB range from data
	let minDb = $derived.by(() => {
		let min = 0;
		for (const row of data) {
			for (const val of row) {
				if (val < min) min = val;
			}
		}
		return min;
	});

	let maxDb = $derived.by(() => {
		let max = -Infinity;
		for (const row of data) {
			for (const val of row) {
				if (val > max) max = val;
			}
		}
		return max;
	});

	// Render spectrogram to canvas
	$effect(() => {
		if (!canvasEl || !data || data.length === 0) return;

		const nMels = data.length;       // 64
		const nFrames = data[0].length;  // 94

		const dpr = typeof window !== 'undefined' ? window.devicePixelRatio || 1 : 1;
		const w = canvasEl.clientWidth;
		const h = canvasEl.clientHeight;
		canvasEl.width = w * dpr;
		canvasEl.height = h * dpr;

		const ctx = canvasEl.getContext('2d');
		if (!ctx) return;
		ctx.scale(dpr, dpr);

		// Build lookup table
		const lut = buildColorLUT(minDb, maxDb);
		const range = maxDb - minDb;

		// Use ImageData for fast pixel-level rendering
		const imgData = ctx.createImageData(nFrames, nMels);
		const pixels = imgData.data;

		for (let mel = 0; mel < nMels; mel++) {
			// Flip vertically: low frequencies at bottom
			const rowIdx = nMels - 1 - mel;
			const row = data[rowIdx];
			for (let frame = 0; frame < nFrames; frame++) {
				const db = row[frame];
				const t = range === 0 ? 0 : Math.max(0, Math.min(1, (db - minDb) / range));
				const lutIdx = Math.round(t * 255);

				const pixelIdx = (mel * nFrames + frame) * 4;
				pixels[pixelIdx] = lut[lutIdx * 3];
				pixels[pixelIdx + 1] = lut[lutIdx * 3 + 1];
				pixels[pixelIdx + 2] = lut[lutIdx * 3 + 2];
				pixels[pixelIdx + 3] = 255;
			}
		}

		// Draw the small ImageData scaled up to fill the canvas
		const offscreen = new OffscreenCanvas(nFrames, nMels);
		const offCtx = offscreen.getContext('2d');
		if (!offCtx) return;
		offCtx.putImageData(imgData, 0, 0);

		// Disable smoothing for crisp pixel look
		ctx.imageSmoothingEnabled = false;
		ctx.drawImage(offscreen, 0, 0, w, h);

		// Draw playhead if audio is playing
		if (audioUrl) {
			const progress = audioStore.progress(audioUrl);
			if (progress > 0 && progress < 1) {
				const px = progress * w;
				ctx.strokeStyle = '#ff3388';
				ctx.lineWidth = 1.5;
				ctx.beginPath();
				ctx.moveTo(px, 0);
				ctx.lineTo(px, h);
				ctx.stroke();
			}
		}
	});

	function handleMouseMove(e: MouseEvent) {
		if (!canvasEl || !data || data.length === 0) return;

		const rect = canvasEl.getBoundingClientRect();
		const x = e.clientX - rect.left;
		const y = e.clientY - rect.top;
		const w = rect.width;
		const h = rect.height;

		const nMels = data.length;
		const nFrames = data[0].length;

		const frame = Math.floor((x / w) * nFrames);
		const mel = Math.floor((y / h) * nMels);

		// Flipped: top of canvas = high frequency = last row of data
		const dataRow = nMels - 1 - mel;

		if (frame >= 0 && frame < nFrames && dataRow >= 0 && dataRow < nMels) {
			tooltipFreqBin = dataRow;
			tooltipTimeFrame = frame;
			tooltipDb = data[dataRow][frame];
			tooltipX = e.clientX;
			tooltipY = e.clientY;
			tooltipVisible = true;
		} else {
			tooltipVisible = false;
		}
	}

	function handleMouseLeave() {
		tooltipVisible = false;
	}
</script>

<div class="spectrogram-viewer" bind:this={containerEl}>
	{#if label}
		<div class="spectrogram-label">{label}</div>
	{/if}
	<div class="spectrogram-canvas-wrap" style="height: {height}px">
		<canvas
			bind:this={canvasEl}
			class="spectrogram-canvas"
			style="height: {height}px"
			onmousemove={handleMouseMove}
			onmouseleave={handleMouseLeave}
			aria-label={label || 'Mel-spectrogram visualization'}
		></canvas>
		<div class="axis-label axis-y">Freq (mel)</div>
		<div class="axis-label axis-x">Time</div>
	</div>

	{#if tooltipVisible}
		<div
			class="spectrogram-tooltip"
			style="left: {tooltipX}px; top: {tooltipY}px"
		>
			<span>bin {tooltipFreqBin}</span>
			<span>frame {tooltipTimeFrame}</span>
			<span>{tooltipDb.toFixed(1)} dB</span>
		</div>
	{/if}
</div>

<style>
	.spectrogram-viewer {
		position: relative;
		width: 100%;
	}

	.spectrogram-label {
		font-family: var(--font-ui);
		font-size: var(--font-size-xs);
		color: var(--color-text-dim);
		letter-spacing: 1px;
		margin-bottom: 6px;
	}

	.spectrogram-canvas-wrap {
		position: relative;
		width: 100%;
		background: var(--color-bg-deep);
		border: 1px solid var(--color-border);
		overflow: hidden;
	}

	.spectrogram-canvas {
		display: block;
		width: 100%;
		cursor: crosshair;
	}

	.axis-label {
		position: absolute;
		font-size: 9px;
		color: var(--color-text-dim);
		pointer-events: none;
		font-family: var(--font-code);
		letter-spacing: 1px;
	}

	.axis-y {
		left: 4px;
		top: 50%;
		transform: translateY(-50%) rotate(-90deg);
		transform-origin: center center;
	}

	.axis-x {
		bottom: 2px;
		right: 6px;
	}

	.spectrogram-tooltip {
		position: fixed;
		transform: translate(12px, -100%);
		background: var(--color-bg-panel);
		border: 1px solid var(--color-border);
		padding: 4px 8px;
		font-size: 11px;
		color: var(--color-text-secondary);
		display: flex;
		gap: 10px;
		pointer-events: none;
		z-index: 100;
		white-space: nowrap;
	}
</style>
