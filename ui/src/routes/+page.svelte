<script lang="ts">
	import StatusBar from '$lib/components/StatusBar.svelte';
	import ControlPanel from '$lib/components/ControlPanel.svelte';
	import Piano from '$lib/components/Piano.svelte';
	import MidiDevices from '$lib/components/MidiDevices.svelte';
	import PresetManager from '$lib/components/PresetManager.svelte';
	import ActiveNotes from '$lib/components/ActiveNotes.svelte';
	import HumanizePanel from '$lib/components/HumanizePanel.svelte';
	import GeneratorPanel from '$lib/components/GeneratorPanel.svelte';
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
		<!-- Left column: MIDI devices + Presets -->
		<div class="column column-left">
			<MidiDevices />
			<PresetManager />
		</div>

		<!-- Center column: Harmony controls -->
		<div class="column column-center">
			<ControlPanel />
		</div>

		<!-- Right column: Active Notes / Humanize / Generator -->
		<div class="column column-right">
			<ActiveNotes />
			<HumanizePanel />
			<GeneratorPanel />
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

</style>
