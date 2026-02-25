<script lang="ts">
	import { engine } from '$lib/stores/engine.svelte';
	import { ui } from '$lib/stores/ui.svelte';
	import BeatIndicator from '$lib/components/BeatIndicator.svelte';

	/**
	 * Toggle animations on/off. Persists the preference in localStorage.
	 */
	function toggleFx() {
		ui.toggleAnimations();
		try {
			localStorage.setItem('contrapunk-fx', ui.animationsEnabled ? 'on' : 'off');
		} catch {
			// localStorage unavailable (SSR or privacy mode)
		}
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
		class="transport-btn font-pixel"
		class:running={engine.isRunning}
		onclick={() => engine.toggle()}
	>
		{engine.isRunning ? 'Stop' : 'Start'}
	</button>

	<!-- Status indicator -->
	<span
		class="status-indicator font-pixel"
		class:active={engine.isRunning}
	>
		{engine.isRunning ? 'ACTIVE' : 'STOPPED'}
	</span>

	<!-- Beat indicator (near status) -->
	<BeatIndicator />

	<!-- Chord display -->
	<div class="chord-info">
		{#if engine.chordName}
			<span class="chord-name font-pixel">{engine.chordName}</span>
		{:else}
			<span class="chord-name font-pixel dim">---</span>
		{/if}
		{#if engine.interchangeEnabled && engine.lastBorrowedFrom}
			<span class="borrowed-label font-pixel">from {engine.lastBorrowedFrom}</span>
		{/if}
	</div>

	<!-- Spacer -->
	<div class="spacer"></div>

	<!-- FX toggle -->
	<button
		class="fx-btn pixel-btn font-pixel"
		class:fx-off={!ui.animationsEnabled}
		onclick={toggleFx}
		title={ui.animationsEnabled ? 'Disable visual effects' : 'Enable visual effects'}
	>
		FX
	</button>

	<!-- Brand -->
	<span class="brand font-pixel">Contrapunk</span>
</div>

<style>
	.status-bar {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 4px 8px;
		background: var(--color-bg-panel);
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
		font-size: 7px;
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
		font-size: 6px;
		color: var(--color-accent-amber);
		white-space: nowrap;
		-webkit-font-smoothing: none;
		text-rendering: optimizeSpeed;
	}

	.spacer {
		flex: 1;
	}

	/* FX Toggle Button */
	.fx-btn {
		padding: 2px 6px;
		font-size: 7px;
		min-width: 24px;
		background: var(--color-accent-teal);
		border: 1px solid var(--color-accent-cyan-dim);
		color: #ffffff;
		box-shadow: var(--glow-teal);
	}

	.fx-btn:hover {
		border-color: var(--color-accent-cyan);
	}

	.fx-btn.fx-off {
		background: var(--color-widget-inactive);
		border-color: var(--color-border);
		color: var(--color-text-dim);
		box-shadow: none;
	}

	.brand {
		font-size: 7px;
		color: var(--color-accent-magenta);
		white-space: nowrap;
		opacity: 0.6;
		-webkit-font-smoothing: none;
		text-rendering: optimizeSpeed;
	}
</style>
