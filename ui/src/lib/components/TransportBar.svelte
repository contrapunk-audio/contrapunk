<script lang="ts">
	import { transport } from '$lib/stores/transport.svelte';

	let bpmInput = $state(120);
	// True while the user is mid-edit (input focused). The transport
	// store ticks `bpm` every router cycle in Tauri; without this flag
	// the `$effect` below kept overwriting whatever the user typed
	// with the polled backend value, making it impossible to reduce
	// BPM below the current backend reading. Tracking focus lets us
	// stop the sync while the user is editing — `commitBpm` flushes
	// their value to the backend on blur/Enter.
	let bpmFocused = $state(false);

	// Sync local BPM input with store changes (from Ableton Link sync
	// later, tap tempo, or initial load) — but only when the user
	// isn't actively typing into it.
	$effect(() => {
		if (!bpmFocused) {
			bpmInput = Math.round(transport.bpm);
		}
	});

	async function togglePlay() {
		if (transport.running) {
			await transport.stop();
		} else {
			await transport.play();
		}
	}

	async function commitBpm() {
		bpmFocused = false;
		await transport.setBpm(bpmInput);
	}

	function onBpmKey(e: KeyboardEvent) {
		if (e.key === 'Enter') {
			(e.target as HTMLInputElement).blur();
		}
	}

	async function toggleClick() {
		await transport.toggleMetronome();
	}

	/** Pips 0..beatsPerBar-1, highlighted when === beatInBar. */
	let pips = $derived(Array.from({ length: transport.beatsPerBar }, (_, i) => i));
</script>

<div class="transport-bar">
	<button
		class="play-btn font-ui"
		class:running={transport.running}
		onclick={togglePlay}
		title={transport.running ? 'Stop transport' : 'Start transport'}
	>
		{transport.running ? '■' : '▶'}
	</button>

	<label class="bpm-field">
		<span class="bpm-label font-ui">BPM</span>
		<input
			class="bpm-input font-code"
			type="number"
			min="40"
			max="240"
			step="1"
			bind:value={bpmInput}
			onfocus={() => (bpmFocused = true)}
			onblur={commitBpm}
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

	<button
		class="click-btn font-ui"
		class:on={transport.metronomeEnabled}
		onclick={toggleClick}
		title={transport.metronomeEnabled ? 'Mute metronome click' : 'Enable metronome click'}
	>
		Click
	</button>
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
		font-size: var(--font-size-xs);
		color: var(--color-text-dim);
	}

	.bpm-input {
		min-width: 4ch;
		width: auto;
		padding: 2px 6px;
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

	.click-btn {
		font-size: var(--font-size-xs);
		padding: 4px 10px;
		border: 1px solid var(--color-border);
		background: var(--color-widget-bg);
		color: var(--color-text-dim);
		cursor: pointer;
	}

	.click-btn.on {
		color: var(--color-accent-gold);
		border-color: var(--color-accent-gold);
		box-shadow: 0 0 4px var(--color-accent-gold);
	}

	.click-btn:hover {
		border-color: var(--color-accent-cyan);
	}
</style>
