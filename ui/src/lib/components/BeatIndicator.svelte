<script lang="ts">
	/**
	 * BeatIndicator — Visual beat position indicator synced to BPM
	 *
	 * A small pulsing bar near the status bar that flashes on each beat.
	 * Downbeats flash magenta, other beats flash cyan. When the WASM
	 * adapter is driving its RAF tick loop it feeds the `beat` store
	 * directly from the Rust BeatClock, so the indicator stays in sync
	 * with the actual metronome / humanize swing timing rather than a
	 * separate JS interval timer that drifts.
	 *
	 * Falls back gracefully: when `beat.running` is false (no adapter
	 * tick loop active) we show a static indicator instead of guessing.
	 */

	import { ui } from '$lib/stores/ui.svelte';
	import { beat } from '$lib/stores/beat.svelte';

	let bpm = $derived(beat.bpm || 120);
	let beatDuration = $derived(60 / bpm);
	let beatCount = $derived(beat.beatNumber % 4);
</script>

<div class="beat-indicator-container">
	<div class="beat-bar-row">
		{#each [0, 1, 2, 3] as b}
			<div
				class="beat-pip"
				class:active={beat.running && beatCount === b}
				class:downbeat={b === 0 && beat.running && beatCount === 0}
				class:animate={ui.animationsEnabled && beat.running}
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
