<script lang="ts">
	import { audioStore } from '$lib/components/diary/audioContext.svelte';

	let { url, height = 64 }: { url: string; height?: number } = $props();

	let canvasEl: HTMLCanvasElement | undefined = $state();
	let peaks: number[] = $state([]);
	let loaded = $state(false);

	const POINTS = 500;

	// Decode audio and downsample to peaks
	$effect(() => {
		loaded = false;
		audioStore.decode(url).then((buffer) => {
			const raw = buffer.getChannelData(0);
			const blockSize = Math.floor(raw.length / POINTS);
			const result: number[] = [];
			for (let i = 0; i < POINTS; i++) {
				let max = 0;
				const start = i * blockSize;
				for (let j = 0; j < blockSize; j++) {
					const abs = Math.abs(raw[start + j]);
					if (abs > max) max = abs;
				}
				result.push(max);
			}
			peaks = result;
			loaded = true;
		}).catch(() => {
			loaded = false;
		});
	});

	// Draw waveform and playhead
	$effect(() => {
		if (!canvasEl || !loaded || peaks.length === 0) return;

		const ctx = canvasEl.getContext('2d');
		if (!ctx) return;

		const dpr = typeof window !== 'undefined' ? window.devicePixelRatio || 1 : 1;
		const w = canvasEl.clientWidth;
		const h = canvasEl.clientHeight;
		canvasEl.width = w * dpr;
		canvasEl.height = h * dpr;
		ctx.scale(dpr, dpr);

		// Background
		ctx.fillStyle = getComputedStyle(canvasEl).getPropertyValue('--color-bg-deep').trim() || '#0a0a1a';
		ctx.fillRect(0, 0, w, h);

		// Waveform bars
		const barWidth = w / peaks.length;
		const midY = h / 2;

		const progress = audioStore.progress(url);

		for (let i = 0; i < peaks.length; i++) {
			const x = i * barWidth;
			const barH = peaks[i] * midY * 0.9;
			const fraction = i / peaks.length;

			if (fraction <= progress && progress > 0) {
				// Played portion — brighter cyan
				ctx.fillStyle = '#33ddff';
			} else {
				// Unplayed portion — dim cyan
				ctx.fillStyle = '#2299cc66';
			}

			ctx.fillRect(x, midY - barH, Math.max(barWidth - 0.5, 1), barH * 2);
		}

		// Playhead line
		if (progress > 0 && progress < 1) {
			const px = progress * w;
			ctx.strokeStyle = '#ff3388';
			ctx.lineWidth = 1.5;
			ctx.beginPath();
			ctx.moveTo(px, 0);
			ctx.lineTo(px, h);
			ctx.stroke();
		}
	});
</script>

<div class="waveform-container" style="height: {height}px">
	{#if !loaded}
		<div class="waveform-loading">Loading waveform...</div>
	{/if}
	<canvas
		bind:this={canvasEl}
		class="waveform-canvas"
		style="height: {height}px"
	></canvas>
</div>

<style>
	.waveform-container {
		position: relative;
		width: 100%;
		background: var(--color-bg-deep);
		border: 1px solid var(--color-border);
		overflow: hidden;
	}

	.waveform-canvas {
		display: block;
		width: 100%;
	}

	.waveform-loading {
		position: absolute;
		inset: 0;
		display: flex;
		align-items: center;
		justify-content: center;
		font-size: 10px;
		color: var(--color-text-dim);
		font-family: var(--font-ui);
	}
</style>
