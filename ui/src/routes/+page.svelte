<script lang="ts">
	import StatusBar from '$lib/components/StatusBar.svelte';
	import ControlPanel from '$lib/components/ControlPanel.svelte';
	import Piano from '$lib/components/Piano.svelte';
	import MidiDevices from '$lib/components/MidiDevices.svelte';
	import GuitarInputPanel from '$lib/components/GuitarInputPanel.svelte';
	import PresetManager from '$lib/components/PresetManager.svelte';
	import ActiveNotes from '$lib/components/ActiveNotes.svelte';
	import HumanizePanel from '$lib/components/HumanizePanel.svelte';
	import GeneratorPanel from '$lib/components/GeneratorPanel.svelte';
	import { adapter } from '$lib/adapter';
	import { engine } from '$lib/stores/engine.svelte';
	import { midi } from '$lib/stores/midi.svelte';
	import { ui } from '$lib/stores/ui.svelte';

	// Virtual input sentinels (must match MidiDevices.svelte and engine.rs)
	const VIRTUAL_COMPUTER_KEYBOARD = 999_998;
	const VIRTUAL_GUITAR_AUDIO = 999_997;

	// Derived: is Guitar Audio selected as input?
	let isGuitarAudioSelected = $derived(midi.selectedInput === VIRTUAL_GUITAR_AUDIO);

	// Piano-style QWERTY mapping:
	// Lower octave — Z row = white keys, S/D/G/H/J = black keys
	// Upper octave — Q row = white keys, 2/3/5/6/7 = black keys
	// Offsets are semitones from C of that octave
	const LOWER_KEYS: Record<string, number> = {
		'z': 0, 's': 1, 'x': 2, 'd': 3, 'c': 4,       // C C# D D# E
		'v': 5, 'g': 6, 'b': 7, 'h': 8, 'n': 9,        // F F# G G# A
		'j': 10, 'm': 11                                  // A# B
	};
	const UPPER_KEYS: Record<string, number> = {
		'q': 0, '2': 1, 'w': 2, '3': 3, 'e': 4,         // C C# D D# E
		'r': 5, '5': 6, 't': 7, '6': 8, 'y': 9,         // F F# G G# A
		'7': 10, 'u': 11, 'i': 12                         // A# B C(+1)
	};

	// Octave state: lower row starts at this MIDI octave, upper row = +1
	let baseOctave = $state(3); // C3 = MIDI 48
	const MIN_OCTAVE = 1;
	const MAX_OCTAVE = 7;

	function keyToMidi(key: string): number | null {
		let midi: number;
		if (key in LOWER_KEYS) midi = (baseOctave + 1) * 12 + LOWER_KEYS[key];
		else if (key in UPPER_KEYS) midi = (baseOctave + 2) * 12 + UPPER_KEYS[key];
		else return null;
		// Clamp to valid MIDI range
		if (midi < 0 || midi > 127) return null;
		return midi;
	}

	// Track held keys → MIDI note they triggered (for correct Note-Off after octave change)
	const heldKeys = new Map<string, number>();

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
				await engine.restoreSettings();
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

			// Octave shift with +/- (= key is + on US keyboards)
			if (key === '=' || key === '+') {
				if (baseOctave < MAX_OCTAVE) baseOctave++;
				e.preventDefault();
				return;
			}
			if (key === '-') {
				if (baseOctave > MIN_OCTAVE) baseOctave--;
				e.preventDefault();
				return;
			}

			const midiNote = keyToMidi(key);
			if (midiNote === null) return;
			if (heldKeys.has(key)) return; // Ignore key repeat

			heldKeys.set(key, midiNote);
			adapter.injectNoteOn(midiNote, 100);
			e.preventDefault();
		}

		function handleKeyUp(e: KeyboardEvent) {
			const key = e.key.toLowerCase();
			const midiNote = heldKeys.get(key);
			if (midiNote === undefined) return;

			heldKeys.delete(key);
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
			<!-- Left column: MIDI devices + Guitar Input + Presets -->
			<div class="column column-left">
				<MidiDevices>
					{#if isGuitarAudioSelected}
						<GuitarInputPanel />
					{/if}
				</MidiDevices>
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
