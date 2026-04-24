<script lang="ts">
	import { engine } from '$lib/stores/engine.svelte';
	import { midi } from '$lib/stores/midi.svelte';
	import { ui } from '$lib/stores/ui.svelte';
	import TransportBar from './TransportBar.svelte';

	/**
	 * Toggle MIDI routing on/off.
	 * Start requires an input device and at least one output from the midi store.
	 */
	async function toggleTransport() {
		if (engine.isRunning) {
			await engine.stop();
		} else if (midi.selectedInput !== null) {
			await engine.start(midi.selectedInput, midi.selectedOutputs);
		}
	}

	function openSettings() {
		ui.openSettings();
	}

	// Restore FX preference from localStorage on mount
	$effect(() => {
		try {
			const saved = localStorage.getItem('contrapunk-fx');
			if (saved === 'off') {
				ui.animationsEnabled = false;
				ui.reducedMotion = true;
				ui.applyMotionPreference();
			}
		} catch {
			// localStorage unavailable
		}
	});
</script>

<div class="status-bar">
	<!-- Transport: Start/Stop button -->
	<button
		class="transport-btn font-ui"
		class:running={engine.isRunning}
		disabled={!engine.isRunning && midi.selectedInput === null}
		onclick={toggleTransport}
		title={!engine.isRunning && midi.selectedInput === null ? 'Select an input device first' : ''}
	>
		{engine.isRunning ? 'Stop' : 'Start'}
	</button>

	<!-- Status indicator -->
	<span
		class="status-indicator font-ui"
		class:active={engine.isRunning}
	>
		{engine.isRunning ? 'ACTIVE' : 'STOPPED'}
	</span>

	<!-- Transport: play/stop + BPM + beat pips -->
	<TransportBar />

	<!-- Chord display -->
	<div class="chord-info">
		{#if engine.chordName}
			<span class="chord-name font-code">{engine.chordName}</span>
		{:else}
			<span class="chord-name font-code dim">---</span>
		{/if}
		{#if engine.interchangeEnabled && engine.lastBorrowedFrom}
			<span class="borrowed-label font-code">from {engine.lastBorrowedFrom}</span>
		{/if}
	</div>

	<!-- Spacer -->
	<div class="spacer"></div>

	<!-- Settings -->
	<button
		class="settings-btn pixel-btn font-ui"
		onclick={openSettings}
		title="Settings"
		aria-label="Open settings"
	>
		Settings
	</button>

	<!-- Brand -->
	<img src="/logo.svg" alt="Contrapunk" class="brand-logo" />
	<span class="brand font-ui">Contrapunk</span>
</div>

<style>
	.status-bar {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 4px 8px;
		/* Semi-transparent so particles drift through behind the top bar. */
		background: rgba(21, 20, 40, 0.88);
		border-bottom: 1px solid var(--color-border);
		height: 32px;
		min-height: 32px;
	}

	.transport-btn {
		font-size: var(--font-size-xs);
		padding: 4px 12px;
		border: 1px solid var(--color-border);
		border-radius: 0;
		cursor: pointer;
		color: #ffffff;
		background: var(--color-accent-teal);
		transition: none;
		-webkit-font-smoothing: none;
		text-rendering: optimizeSpeed;
	}

	.transport-btn.running {
		background: #cc0044;
		border-color: #ff3366;
		box-shadow: 0 0 6px #ff336666;
	}

	.transport-btn:hover {
		border-color: var(--color-accent-cyan);
	}

	.status-indicator {
		font-size: var(--font-size-xs);
		color: var(--color-text-dim);
		-webkit-font-smoothing: none;
		text-rendering: optimizeSpeed;
	}

	.status-indicator.active {
		color: var(--color-piano-input);
		text-shadow: 0 0 6px var(--color-piano-input);
	}

	.chord-info {
		display: flex;
		align-items: center;
		gap: 6px;
		flex: 1;
		min-width: 0;
	}

	.chord-name {
		font-size: var(--font-size-xs);
		color: var(--color-accent-cyan);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
		-webkit-font-smoothing: none;
		text-rendering: optimizeSpeed;
	}

	.chord-name.dim {
		color: var(--color-text-dim);
	}

	.borrowed-label {
		font-size: var(--font-size-xs);
		color: var(--color-accent-amber);
		white-space: nowrap;
		-webkit-font-smoothing: none;
		text-rendering: optimizeSpeed;
	}

	.spacer {
		flex: 1;
	}

	.settings-btn {
		padding: 4px 10px;
		font-size: var(--font-size-xs);
		background: var(--color-widget-inactive);
		border: 1px solid var(--color-border);
		color: var(--color-text-secondary);
	}

	.settings-btn:hover {
		border-color: var(--color-accent-cyan);
		color: var(--color-accent-cyan);
	}

	.brand-logo {
		height: 20px;
		width: auto;
		opacity: 0.8;
	}

	.brand {
		font-size: var(--font-size-xs);
		color: var(--color-accent-magenta);
		white-space: nowrap;
		opacity: 0.6;
		-webkit-font-smoothing: none;
		text-rendering: optimizeSpeed;
	}
</style>
