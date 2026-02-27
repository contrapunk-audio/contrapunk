<script lang="ts">
	import StatusBar from '$lib/components/StatusBar.svelte';
	import ControlPanel from '$lib/components/ControlPanel.svelte';
	import Piano from '$lib/components/Piano.svelte';
	import MidiDevices from '$lib/components/MidiDevices.svelte';
	import PresetManager from '$lib/components/PresetManager.svelte';
	import ActiveNotes from '$lib/components/ActiveNotes.svelte';
	import HumanizePanel from '$lib/components/HumanizePanel.svelte';
	import GeneratorPanel from '$lib/components/GeneratorPanel.svelte';
	import { adapter } from '$lib/adapter';
	import { engine } from '$lib/stores/engine.svelte';
	import { midi } from '$lib/stores/midi.svelte';
	import { ui } from '$lib/stores/ui.svelte';

	// Virtual input sentinel (must match MidiDevices.svelte)
	const VIRTUAL_COMPUTER_KEYBOARD = Number.MAX_SAFE_INTEGER - 1;

	// QWERTY → MIDI note mapping (standard DAW layout)
	// Z-M: C3-B3 (MIDI 48-59), Q-U: C4-B4 (MIDI 60-71)
	const KEY_TO_MIDI: Record<string, number> = {
		'z': 48, 'x': 49, 'c': 50, 'v': 51, 'b': 52, 'n': 53, 'm': 54,
		',': 55, '.': 56, '/': 57,
		'a': 55, 's': 56, 'd': 57, 'f': 58, 'g': 59, 'h': 60, 'j': 61,
		'k': 62, 'l': 63, ';': 64,
		'q': 60, 'w': 61, 'e': 62, 'r': 63, 't': 64, 'y': 65, 'u': 66,
		'i': 67, 'o': 68, 'p': 69,
	};

	// Track held keys to prevent repeat events
	const heldKeys = new Set<string>();

	// Initialize adapter, sync engine state, and enumerate MIDI devices on mount
	let initError = $state<string | null>(null);
	let initDone = $state(false);
	let initStarted = false;

	$effect(() => {
		if (initStarted) return;
		initStarted = true;

		(async () => {
			try {
				await adapter.init();
				await engine.syncFromBackend();
				await midi.refresh();
				initDone = true;
			} catch (e) {
				initError = `Init failed: ${e}`;
				console.error('[contrapunk] Initialization error:', e);
			}
		})();
	});

	// Computer keyboard input handler
	$effect(() => {
		if (typeof window === 'undefined') return;

		function handleKeyDown(e: KeyboardEvent) {
			// Skip if typing in an input field
			if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) return;
			// Skip if not in Computer Keyboard mode or not running
			if (midi.selectedInput !== VIRTUAL_COMPUTER_KEYBOARD) return;
			if (!engine.isRunning) return;

			const key = e.key.toLowerCase();
			if (!(key in KEY_TO_MIDI)) return;
			if (heldKeys.has(key)) return; // Ignore key repeat

			heldKeys.add(key);
			const midiNote = KEY_TO_MIDI[key];
			adapter.injectNoteOn(midiNote, 100);
			e.preventDefault();
		}

		function handleKeyUp(e: KeyboardEvent) {
			const key = e.key.toLowerCase();
			if (!(key in KEY_TO_MIDI)) return;
			if (!heldKeys.has(key)) return;

			heldKeys.delete(key);
			const midiNote = KEY_TO_MIDI[key];
			adapter.injectNoteOff(midiNote);
			e.preventDefault();
		}

		window.addEventListener('keydown', handleKeyDown);
		window.addEventListener('keyup', handleKeyUp);

		return () => {
			window.removeEventListener('keydown', handleKeyDown);
			window.removeEventListener('keyup', handleKeyUp);
			heldKeys.clear();
		};
	});

	// Music-reactive: detect when notes are actively sounding
	let hasActiveNotes = $derived(
		engine.inputNotes.length > 0 || engine.harmonyNotes.length > 0
	);
	let manyNotesActive = $derived(
		engine.inputNotes.length + engine.harmonyNotes.length >= 4
	);
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
	<!-- Music-reactive vignette overlay (CSS-only, no JS cost) -->
	{#if ui.animationsEnabled && hasActiveNotes}
		<div
			class="vignette-overlay"
			class:intense={manyNotesActive}
			aria-hidden="true"
		></div>
	{/if}

	{#if initError}
		<div class="init-error font-pixel">{initError}</div>
	{/if}

	<!-- Top: Status bar -->
	<StatusBar />

	{#if initDone}
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
	{:else if !initError}
		<div class="init-loading font-pixel">Initializing engine...</div>
	{/if}
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

	/* === Music-reactive vignette === */
	.vignette-overlay {
		position: fixed;
		inset: 0;
		pointer-events: none;
		z-index: 100;
		background: radial-gradient(
			ellipse at center,
			transparent 50%,
			rgba(255, 51, 136, 0.04) 80%,
			rgba(255, 51, 136, 0.08) 100%
		);
		animation: vignette-pulse 2s ease-in-out infinite;
	}

	.vignette-overlay.intense {
		background: radial-gradient(
			ellipse at center,
			transparent 40%,
			rgba(255, 51, 136, 0.06) 70%,
			rgba(51, 221, 255, 0.06) 85%,
			rgba(255, 51, 136, 0.1) 100%
		);
	}

	@keyframes vignette-pulse {
		0%, 100% { opacity: 0.5; }
		50% { opacity: 1; }
	}

	.init-error {
		background: #cc0044;
		color: #ffffff;
		font-size: 7px;
		padding: 4px 8px;
		text-align: center;
	}

	.init-loading {
		color: var(--color-text-dim);
		font-size: 8px;
		text-align: center;
		padding: 40px 8px;
		-webkit-font-smoothing: none;
	}
</style>
