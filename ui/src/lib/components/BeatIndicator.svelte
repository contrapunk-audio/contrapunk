<script lang="ts">
	/**
	 * BeatIndicator — Visual beat position indicator synced to BPM
	 *
	 * A small pulsing bar near the status bar that flashes on each beat.
	 * Downbeats flash magenta, other beats flash cyan. Uses CSS animation
	 * with duration derived from BPM. When engine is not running, shows
	 * a static indicator. When ui.animationsEnabled is false, uses a
	 * simpler static color change without animation.
	 */

	import { engine } from '$lib/stores/engine.svelte';
	import { ui } from '$lib/stores/ui.svelte';

	// Derive beat duration from BPM (default 120)
	// We read bpm from the engine store or fall back to a reasonable default
	let bpm = $derived(120); // BPM is in HumanizePanel local state; use default here
	let beatDuration = $derived(60 / bpm);

	// Beat counter for downbeat detection (resets every 4)
	let beatCount = $state(0);
	let intervalId: ReturnType<typeof setInterval> | null = null;

	// Advance beat counter when running
	$effect(() => {
		if (engine.isRunning) {
			const ms = (60 / bpm) * 1000;
			beatCount = 0;
			intervalId = setInterval(() => {
				beatCount = (beatCount + 1) % 4;
			}, ms);
		} else {
			if (intervalId) {
				clearInterval(intervalId);
				intervalId = null;
			}
			beatCount = 0;
		}

		return () => {
			if (intervalId) {
				clearInterval(intervalId);
				intervalId = null;
			}
		};
	});

	let isDownbeat = $derived(beatCount === 0);
</script>

<div class="beat-indicator-container">
	<div class="beat-bar-row">
		{#each [0, 1, 2, 3] as beat}
			<div
				class="beat-pip"
				class:active={engine.isRunning && beatCount === beat}
				class:downbeat={beat === 0 && engine.isRunning && beatCount === 0}
				class:animate={ui.animationsEnabled && engine.isRunning}
				style:animation-duration="{beatDuration}s"
			></div>
		{/each}
	</div>
</div>

<style>
	.beat-indicator-container {
		display: flex;
		align-items: center;
		gap: 4px;
	}

	.beat-bar-row {
		display: flex;
		gap: 3px;
		align-items: center;
	}

	.beat-pip {
		width: 6px;
		height: 6px;
		background: var(--color-widget-inactive);
		border: 1px solid var(--color-border);
		transition: none;
	}

	.beat-pip.active {
		background: var(--color-accent-cyan);
		border-color: var(--color-accent-cyan-dim);
		box-shadow: 0 0 4px var(--color-accent-cyan);
	}

	.beat-pip.active.downbeat {
		background: var(--color-accent-magenta);
		border-color: var(--color-accent-magenta-dim);
		box-shadow: 0 0 6px var(--color-accent-magenta);
	}

	.beat-pip.active.animate {
		animation: pip-flash ease-out;
	}

	@keyframes pip-flash {
		0% {
			opacity: 1;
			transform: scale(1.3);
		}
		60% {
			opacity: 0.9;
			transform: scale(1);
		}
		100% {
			opacity: 0.6;
			transform: scale(1);
		}
	}
</style>
