<script lang="ts">
	import StatusBar from '$lib/components/StatusBar.svelte';
	import ControlPanel from '$lib/components/ControlPanel.svelte';
	import Piano from '$lib/components/Piano.svelte';
</script>

<!--
  Ableton Session View-inspired layout:
  +--------------------------------------------------+
  | StatusBar (Start/Stop, chord, status)            |
  +--------------------------------------------------+
  |             |                |                     |
  |  Left Col   |  Center Col    |  Right Col          |
  |  (MIDI/     |  (Key/Mode/    |  (Humanize/         |
  |   Presets)   |   Scale/VL)    |   Generator)         |
  |             |                |                     |
  +--------------------------------------------------+
  | Piano Keyboard (always visible, full width)      |
  +--------------------------------------------------+
-->

<div class="app-layout">
	<!-- Top: Status bar -->
	<StatusBar />

	<!-- Middle: 3-column content area -->
	<div class="content-area">
		<!-- Left column: MIDI / Presets (placeholder) -->
		<div class="column column-left">
			<div class="placeholder-card">
				<div class="placeholder-header font-pixel">MIDI I/O</div>
				<p class="placeholder-text font-pixel">Input/Output selection</p>
				<p class="placeholder-hint font-pixel">Plan 05</p>
			</div>
			<div class="placeholder-card">
				<div class="placeholder-header font-pixel">Presets</div>
				<p class="placeholder-text font-pixel">Save/Load presets</p>
				<p class="placeholder-hint font-pixel">Plan 06</p>
			</div>
		</div>

		<!-- Center column: Harmony controls -->
		<div class="column column-center">
			<ControlPanel />
		</div>

		<!-- Right column: Humanize / Generator (placeholder) -->
		<div class="column column-right">
			<div class="placeholder-card">
				<div class="placeholder-header font-pixel">Humanize</div>
				<p class="placeholder-text font-pixel">Jitter, Velocity, Swing</p>
				<p class="placeholder-hint font-pixel">Plan 05</p>
			</div>
			<div class="placeholder-card">
				<div class="placeholder-header font-pixel">Generator</div>
				<p class="placeholder-text font-pixel">Note patterns</p>
				<p class="placeholder-hint font-pixel">Plan 06</p>
			</div>
		</div>
	</div>

	<!-- Bottom: Sacred piano keyboard -->
	<div class="piano-area">
		<Piano />
	</div>
</div>

<style>
	.app-layout {
		display: grid;
		grid-template-rows: auto 1fr auto;
		height: 100vh;
		width: 100vw;
		overflow: hidden;
		background: var(--color-bg-deep);
	}

	.content-area {
		display: grid;
		grid-template-columns: 1fr 1.4fr 1fr;
		gap: 1px;
		overflow: hidden;
		background: var(--color-border);
	}

	.column {
		background: var(--color-bg-base);
		overflow-y: auto;
		overflow-x: hidden;
		padding: 4px;
	}

	/* Hide scrollbar but keep scrollable */
	.column::-webkit-scrollbar {
		width: 4px;
	}

	.column::-webkit-scrollbar-track {
		background: var(--color-bg-deep);
	}

	.column::-webkit-scrollbar-thumb {
		background: var(--color-widget-inactive);
		border-radius: 0;
	}

	.piano-area {
		border-top: 1px solid var(--color-border);
		background: var(--color-bg-deep);
	}

	/* Placeholder cards for future panels */
	.placeholder-card {
		background: var(--color-widget-bg);
		border: 1px solid var(--color-border);
		padding: 8px;
		margin-bottom: 4px;
		border-radius: 0;
	}

	.placeholder-header {
		color: var(--color-accent-gold);
		font-size: var(--font-size-xs);
		margin-bottom: 6px;
		-webkit-font-smoothing: none;
		text-rendering: optimizeSpeed;
	}

	.placeholder-text {
		color: var(--color-text-dim);
		font-size: 7px;
		margin: 0 0 4px 0;
		-webkit-font-smoothing: none;
		text-rendering: optimizeSpeed;
	}

	.placeholder-hint {
		color: var(--color-text-dim);
		font-size: 6px;
		margin: 0;
		opacity: 0.5;
		-webkit-font-smoothing: none;
		text-rendering: optimizeSpeed;
	}
</style>
