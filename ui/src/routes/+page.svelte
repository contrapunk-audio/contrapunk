<script lang="ts">
	import StatusBar from '$lib/components/StatusBar.svelte';
	import ControlPanel from '$lib/components/ControlPanel.svelte';
	import Piano from '$lib/components/Piano.svelte';
	import MidiDevices from '$lib/components/MidiDevices.svelte';
	import GuitarInputPanel from '$lib/components/GuitarInputPanel.svelte';
	// PresetManager temporarily unmounted (backend + component file
	// kept). Tracked in contrapunk#65 — will return with a redesigned UX.
	import ActiveNotes from '$lib/components/ActiveNotes.svelte';
	import Fretboard from '$lib/components/Fretboard.svelte';
	import HistoryStrip from '$lib/components/HistoryStrip.svelte';
	import SettingsModal from '$lib/components/SettingsModal.svelte';
	import ChainPanel from '$lib/components/ChainPanel.svelte';
	import { adapter } from '$lib/adapter';
	import { engine } from '$lib/stores/engine.svelte';
	import { midi } from '$lib/stores/midi.svelte';
	import { ui } from '$lib/stores/ui.svelte';
	import { synth } from '$lib/stores/synth.svelte';

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
				ui.restoreAppearance();
				ui.restoreViewMode();
				await adapter.init();
				await engine.syncFromBackend();
				await engine.restoreSettings();
				await midi.refresh();
				await synth.syncFromBackend();
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
  |  (MIDI/     |  (Key/Mode/    |  (Active Notes)     |
  |   Presets)   |   Scale/VL)    |                     |
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
		<div class="init-error font-ui">{initError}</div>
	{/if}

	<!-- Top: Status bar -->
	<StatusBar />

	<!-- Settings modal (overlays the whole app when open) -->
	<SettingsModal />

	{#if initDone}
		{#if ui.viewMode === 'full'}
			<!-- Tab switcher -->
			<div class="tab-strip">
				<button
					class="tab-btn font-ui"
					class:active={ui.activeTab === 'play'}
					onclick={() => (ui.activeTab = 'play')}
				>
					Play
				</button>
				<button
					class="tab-btn font-ui"
					class:active={ui.activeTab === 'chain'}
					onclick={() => (ui.activeTab = 'chain')}
				>
					Chain
				</button>
			</div>

			{#if ui.activeTab === 'play'}
				<!-- Middle: 2-column content area (Active Notes moved
				     out to a strip directly above the piano so it sits
				     visually next to the keyboards it describes). -->
				<div class="content-area two-col">
					<!-- Left column: MIDI devices + Guitar Input
					     (PresetManager temporarily unmounted — see contrapunk#65) -->
					<div class="column column-left">
						<MidiDevices>
							{#if isGuitarAudioSelected}
								<GuitarInputPanel />
							{/if}
						</MidiDevices>
					</div>

					<!-- Right column: Harmony controls (now spans the
					     vacated right slot for more breathing room). -->
					<div class="column column-center">
						<ControlPanel />
					</div>
				</div>

				<!-- Active-notes strip — sits between the controls and
				     the visual instruments so users can read the active
				     pitches inline with the keys they light up below. -->
				<div class="active-notes-strip">
					<ActiveNotes />
				</div>

				<!-- Bottom: History strip + Fretboard + Sacred piano keyboard -->
				<div class="piano-area">
					<HistoryStrip />
					<Fretboard />
					<Piano />
				</div>
			{:else}
				<!-- Chain tab: audio signal flow + synth params -->
				<div class="chain-area">
					<ChainPanel />
				</div>
			{/if}
		{:else if ui.viewMode === 'performance'}
			<!-- Performance: history + both instruments, no setup chrome -->
			<div class="solo-area piano-area">
				<HistoryStrip />
				<Fretboard />
				<Piano />
			</div>
		{:else if ui.viewMode === 'fretboard'}
			<!-- Fretboard-only: centered, fills available space -->
			<div class="solo-area solo-fretboard">
				<Fretboard />
			</div>
		{:else if ui.viewMode === 'piano'}
			<!-- Piano-only: centered, fills available space -->
			<div class="solo-area solo-piano">
				<Piano />
			</div>
		{/if}
	{:else if !initError}
		<div class="init-loading font-ui">Initializing engine...</div>
	{/if}
</div>

<style>
	.app-layout {
		display: grid;
		grid-template-rows: auto auto 1fr auto;
		height: 100vh;
		width: 100vw;
		overflow: hidden;
		/* Transparent so particles show through wherever panels don't
		   paint. Panels below set their own semi-transparent backgrounds. */
		background: transparent;
	}

	.tab-strip {
		display: flex;
		background: rgba(15, 14, 26, 0.88);
		border-bottom: 1px solid var(--color-border);
		padding: 0 8px;
	}

	.tab-btn {
		background: transparent;
		border: none;
		border-bottom: 2px solid transparent;
		color: var(--color-text-secondary);
		font-size: var(--font-size-xs);
		padding: 6px 14px;
		cursor: pointer;
	}

	.tab-btn:hover {
		color: var(--color-accent-cyan);
	}

	.tab-btn.active {
		color: var(--color-accent-magenta);
		border-bottom-color: var(--color-accent-magenta);
		box-shadow: 0 2px 0 var(--color-accent-magenta) inset;
	}

	.chain-area {
		background: rgba(15, 14, 26, 0.88);
		overflow-y: auto;
	}

	.content-area {
		display: grid;
		grid-template-columns: 1fr 1.4fr 1fr;
		gap: 1px;
		overflow: hidden;
		background: var(--color-border);
	}
	/* When the right column is unused (Active Notes moved to a strip
	   above the piano), give the controls extra width instead of
	   leaving dead space. */
	.content-area.two-col {
		grid-template-columns: 1fr 1.6fr;
	}

	/* Active-notes strip — full-width thin row directly above the
	   piano area. Keeps the chord readout + INPUT/HARMONY note lists
	   close to the visual keys they correspond to. */
	.active-notes-strip {
		border-top: 1px solid var(--color-border);
		background: rgba(15, 14, 26, 0.88);
		padding: 4px 8px;
	}

	.column {
		background: rgba(15, 14, 26, 0.88);
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
		background: rgba(10, 10, 26, 0.88);
	}

	/* === Solo view modes (issue #44) ===
	   Each fills the remaining vertical space below the StatusBar so
	   single-instrument layouts feel deliberate, not cramped. */
	.solo-area {
		display: flex;
		flex-direction: column;
		justify-content: center;
		align-items: stretch;
		padding: 24px clamp(16px, 4vw, 64px);
		background: rgba(10, 10, 26, 0.88);
		border-top: 1px solid var(--color-border);
		overflow: hidden;
	}
	.solo-fretboard,
	.solo-piano {
		gap: 16px;
	}
	/* Performance mode keeps the same look as the bottom strip in full
	   mode but takes the whole content area. */
	.solo-area.piano-area {
		padding: 0;
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
