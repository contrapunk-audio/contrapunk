<script lang="ts">
	import { ui } from '$lib/stores/ui.svelte';
	import { onDestroy } from 'svelte';

	/**
	 * HLD Atmospheric Particle System
	 *
	 * Renders 30 small ambient dots drifting slowly in magenta/cyan/teal
	 * behind all UI content. Uses a canvas element for efficient rendering
	 * with minimal CPU cost. Respects ui.animationsEnabled.
	 */

	const PARTICLE_COUNT = 30;
	const COLORS = [
		'rgba(255, 51, 136, 0.2)',   // magenta
		'rgba(51, 221, 255, 0.2)',   // cyan
		'rgba(0, 204, 170, 0.2)',    // teal
		'rgba(255, 119, 170, 0.15)', // soft pink
		'rgba(51, 221, 255, 0.12)',  // faint cyan
	];

	interface Particle {
		x: number;
		y: number;
		vx: number;
		vy: number;
		size: number;
		color: string;
		opacity: number;
		opacitySpeed: number;
		opacityDir: number;
	}

	let canvas: HTMLCanvasElement | undefined = $state();
	let animId: number | null = null;
	let particles: Particle[] = [];

	function initParticles(w: number, h: number) {
		particles = [];
		for (let i = 0; i < PARTICLE_COUNT; i++) {
			particles.push({
				x: Math.random() * w,
				y: Math.random() * h,
				vx: (Math.random() - 0.5) * 0.5,
				vy: (Math.random() - 0.5) * 0.4,
				size: 1 + Math.random() * 2,
				color: COLORS[Math.floor(Math.random() * COLORS.length)],
				opacity: 0.3 + Math.random() * 0.5,
				opacitySpeed: 0.003 + Math.random() * 0.005,
				opacityDir: Math.random() > 0.5 ? 1 : -1,
			});
		}
	}

	function animate() {
		if (!canvas) return;
		const ctx = canvas.getContext('2d');
		if (!ctx) return;

		const w = canvas.width;
		const h = canvas.height;

		ctx.clearRect(0, 0, w, h);

		for (const p of particles) {
			// Move
			p.x += p.vx;
			p.y += p.vy;

			// Wrap around edges
			if (p.x < -5) p.x = w + 5;
			if (p.x > w + 5) p.x = -5;
			if (p.y < -5) p.y = h + 5;
			if (p.y > h + 5) p.y = -5;

			// Fade in/out cycle
			p.opacity += p.opacitySpeed * p.opacityDir;
			if (p.opacity >= 0.8) { p.opacityDir = -1; }
			if (p.opacity <= 0.15) { p.opacityDir = 1; }

			// Draw
			ctx.globalAlpha = p.opacity;
			ctx.fillStyle = p.color;
			ctx.fillRect(
				Math.round(p.x),
				Math.round(p.y),
				Math.round(p.size),
				Math.round(p.size)
			);
		}

		ctx.globalAlpha = 1;
		animId = requestAnimationFrame(animate);
	}

	function startAnimation() {
		if (!canvas) return;
		canvas.width = window.innerWidth;
		canvas.height = window.innerHeight;
		initParticles(canvas.width, canvas.height);
		animId = requestAnimationFrame(animate);
	}

	function stopAnimation() {
		if (animId !== null) {
			cancelAnimationFrame(animId);
			animId = null;
		}
	}

	function handleResize() {
		if (!canvas || !ui.animationsEnabled) return;
		canvas.width = window.innerWidth;
		canvas.height = window.innerHeight;
		initParticles(canvas.width, canvas.height);
	}

	// Start/stop based on animationsEnabled
	$effect(() => {
		if (ui.animationsEnabled && canvas) {
			startAnimation();
			window.addEventListener('resize', handleResize);
		} else {
			stopAnimation();
		}

		return () => {
			stopAnimation();
			window.removeEventListener('resize', handleResize);
		};
	});

	onDestroy(() => {
		stopAnimation();
	});
</script>

{#if ui.animationsEnabled}
	<canvas
		bind:this={canvas}
		class="particles-canvas"
		aria-hidden="true"
	></canvas>
{/if}

<style>
	.particles-canvas {
		position: fixed;
		top: 0;
		left: 0;
		width: 100vw;
		height: 100vh;
		z-index: 0;
		pointer-events: none;
		image-rendering: pixelated;
	}
</style>
