<script lang="ts">
	import { transport } from '$lib/stores/transport.svelte';

	let bpmInput = $state(120);

	// Sync local BPM input with store changes (from Ableton Link sync
	// later, or tap tempo, or initial load).
	$effect(() => {
		bpmInput = Math.round(transport.bpm);
	});

	async function togglePlay() {
		if (transport.running) {
			await transport.stop();
		} else {
			await transport.play();
		}
	}

	async function commitBpm() {
		await transport.setBpm(bpmInput);
	}

	function onBpmKey(e: KeyboardEvent) {
		if (e.key === 'Enter') {
			(e.target as HTMLInputElement).blur();
		}
	}

	/** Pips 0..beatsPerBar-1, highlighted when === beatInBar. */
	let pips = $derived(Array.from({ length: transport.beatsPerBar }, (_, i) => i));
</script>

<div class="transport-bar">
	<button
		class="play-btn font-pixel"
		class:running={transport.running}
		onclick={togglePlay}
		title={transport.running ? 'Stop transport' : 'Start transport'}
	>
		{transport.running ? '■' : '▶'}
	</button>

	<label class="bpm-field">
		<span class="bpm-label font-pixel">BPM</span>
		<input
			class="bpm-input font-pixel"
			type="number"
			min="40"
			max="240"
			step="1"
			bind:value={bpmInput}
			onchange={commitBpm}
			onkeydown={onBpmKey}
		/>
	</label>

	<div class="beat-pips">
		{#each pips as i}
			<span
				class="pip"
				class:active={transport.running && transport.beatInBar === i}
				class:downbeat={i === 0}
			></span>
		{/each}
	</div>
</div>

<style>
	.transport-bar {
		display: flex;
		align-items: center;
		gap: 8px;
	}

	.play-btn {
		font-size: var(--font-size-xs);
		padding: 2px 8px;
		min-width: 28px;
		border: 1px solid var(--color-border);
		background: var(--color-widget-bg);
		color: var(--color-text-primary);
		cursor: pointer;
	}

	.play-btn.running {
		background: var(--color-accent-magenta);
		border-color: var(--color-accent-magenta);
		color: #ffffff;
		box-shadow: 0 0 6px var(--color-accent-magenta);
	}

	.play-btn:hover:not(.running) {
		border-color: var(--color-accent-cyan);
		color: var(--color-accent-cyan);
	}

	.bpm-field {
		display: flex;
		align-items: center;
		gap: 4px;
	}

	.bpm-label {
		font-size: 6px;
		color: var(--color-text-dim);
	}

	.bpm-input {
		width: 42px;
		padding: 2px 4px;
		border: 1px solid var(--color-border);
		background: var(--color-widget-bg);
		color: var(--color-accent-cyan);
		font-size: var(--font-size-xs);
		text-align: right;
		-moz-appearance: textfield;
	}
	.bpm-input::-webkit-outer-spin-button,
	.bpm-input::-webkit-inner-spin-button {
		-webkit-appearance: none;
		margin: 0;
	}
	.bpm-input:focus {
		outline: none;
		border-color: var(--color-accent-cyan);
	}

	.beat-pips {
		display: flex;
		gap: 3px;
	}

	.pip {
		width: 6px;
		height: 6px;
		background: var(--color-widget-inactive);
		border-radius: 1px;
		transition: background 60ms ease-out, box-shadow 60ms ease-out;
	}

	.pip.downbeat {
		background: var(--color-border);
	}

	.pip.active {
		background: var(--color-accent-cyan);
		box-shadow: 0 0 4px var(--color-accent-cyan);
	}

	.pip.active.downbeat {
		background: var(--color-accent-magenta);
		box-shadow: 0 0 6px var(--color-accent-magenta);
	}
</style>
