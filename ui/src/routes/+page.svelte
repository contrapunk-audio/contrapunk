<script lang="ts">
	import StatusBar from '$lib/components/StatusBar.svelte';
	import ControlPanel from '$lib/components/ControlPanel.svelte';
	import Piano from '$lib/components/Piano.svelte';
	import MidiDevices from '$lib/components/MidiDevices.svelte';
	import GuitarInputPanel from '$lib/components/GuitarInputPanel.svelte';
	import PerformanceView from '$lib/components/PerformanceView.svelte';
	// PresetManager temporarily unmounted (backend + component file
	// kept). Tracked in contrapunk#65 — will return with a redesigned UX.
	import ActiveNotes from '$lib/components/ActiveNotes.svelte';
	import Fretboard from '$lib/components/Fretboard.svelte';
	import HistoryStrip from '$lib/components/HistoryStrip.svelte';
	import SettingsModal from '$lib/components/SettingsModal.svelte';
	import CompanionPanel from '$lib/components/CompanionPanel.svelte';
	import VoicesPanel from '$lib/components/VoicesPanel.svelte';
	import InputPanel from '$lib/components/InputPanel.svelte';
	import OutputPanel from '$lib/components/OutputPanel.svelte';
	import { adapter } from '$lib/adapter';
	import { engine } from '$lib/stores/engine.svelte';
	import { midi } from '$lib/stores/midi.svelte';
	import { ui } from '$lib/stores/ui.svelte';
	import { synth } from '$lib/stores/synth.svelte';
	import { transport } from '$lib/stores/transport.svelte';
	import { attachKeyboardInput } from '$lib/keyboard-input';

	// Virtual input sentinels (must match MidiDevices.svelte and engine.rs)
	const VIRTUAL_COMPUTER_KEYBOARD = 999_998;
	const VIRTUAL_GUITAR_AUDIO = 999_997;

	// Derived: is Guitar Audio selected as input?
	let isGuitarAudioSelected = $derived(midi.selectedInput === VIRTUAL_GUITAR_AUDIO);

	// QWERTY keyboard input — see $lib/keyboard-input for the keymap.

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
				ui.restorePanels();
				ui.restoreViewMode();
				ui.restoreTabs();
				await adapter.init();
				await engine.syncFromBackend();
				await engine.restoreSettings();
				// Browser path: only refreshes devices if the user previously
				// granted Web MIDI access. First-time visitors see the
				// "Enable MIDI" button in MidiDevices.svelte instead.
				// Tauri / Plugin: resolves immediately and refreshes.
				await midi.hydratePermission();
				await synth.syncFromBackend();
				// Force the metronome OFF on every fresh launch. Backend
				// AppState already defaults `metronome_enabled` to false,
				// but explicitly resetting here defends against any
				// startup race condition (audio thread firing a beat
				// crossing before the UI's syncFromBackend completes) and
				// guarantees the user never gets a click on first boot.
				try {
					await transport.setMetronomeEnabled(false);
				} catch {
					/* no transport adapter (browser w/o backend) */
				}

				// FTUX: if the user has no saved MIDI input and didn't
				// pick one from the device list, default to the
				// Computer-Keyboard virtual input and auto-start routing
				// so the first note they play (typed) immediately
				// produces audible companion voices. Skip auto-start if
				// the user already has a real input selected (saved
				// from a prior session — they may want to wire up before
				// routing fires).
				try {
					if (midi.selectedInput === null) {
						midi.selectVirtualInput(VIRTUAL_COMPUTER_KEYBOARD);
					}
					if (!engine.isRunning && midi.selectedInput !== null) {
						await engine.start(midi.selectedInput, midi.selectedOutputs);
					}
				} catch (e) {
					console.warn('[contrapunk] FTUX auto-routing skipped:', e);
				}

				initDone = true;
			} catch (e) {
				initError = `Init failed: ${e}`;
				console.error('[contrapunk] Initialization error:', e);
			}
		})();
	});

	// Computer-Keyboard virtual input — wired via the shared keymap
	// in $lib/keyboard-input. Gated on Computer-Keyboard being the
	// selected input AND the engine running, so other input modes
	// (real MIDI, Guitar Audio) keep ignoring keystrokes.
	$effect(() =>
		attachKeyboardInput({
			onNoteOn: (midi) => {
				adapter.injectNoteOn(midi, 100);
			},
			onNoteOff: (midi) => {
				adapter.injectNoteOff(midi);
			},
			enabled: () =>
				midi.selectedInput === VIRTUAL_COMPUTER_KEYBOARD && engine.isRunning
		})
	);

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
		<!-- Tab switcher (orthogonal to panel visibility — tabs decide
		     content type, panels decide which Play surfaces render). -->
		<div class="tab-strip">
			<button
				class="tab-btn font-ui"
				class:active={ui.activeTab === 'play'}
				onclick={() => ui.setActiveTab('play')}
			>
				Harmony
			</button>
			<button
				class="tab-btn font-ui"
				class:active={ui.activeTab === 'io'}
				onclick={() => ui.setActiveTab('io')}
			>
				I/O
			</button>
			<button
				class="tab-btn font-ui"
				class:active={ui.activeTab === 'companion'}
				onclick={() => ui.setActiveTab('companion')}
			>
				Companion
			</button>
			<button
				class="tab-btn font-ui"
				class:active={ui.activeTab === 'voices'}
				onclick={() => ui.setActiveTab('voices')}
			>
				Voices
			</button>
		</div>

		{#if ui.activeTab === 'io'}
			<!-- I/O subtab strip — only rendered when I/O tab is active.
			     Input subtab = MIDI/Guitar/Voice source pickers; Output
			     subtab = Voice Generation Chain + synth/FX. -->
			<div class="subtab-strip">
				<button
					class="subtab-btn font-ui"
					class:active={ui.ioSubtab === 'input'}
					onclick={() => ui.setIoSubtab('input')}
				>
					Input
				</button>
				<button
					class="subtab-btn font-ui"
					class:active={ui.ioSubtab === 'output'}
					onclick={() => ui.setIoSubtab('output')}
				>
					Output
				</button>
			</div>
		{/if}

		{#if ui.activeTab === 'play'}
			{#if ui.panels.midi || ui.panels.controls}
				<!-- Setup row: MIDI devices + Harmony controls. Each
				     column is a togglable panel; the row collapses to
				     a single column when only one is visible, and
				     disappears entirely when both are off. The right
				     column swaps between PerformanceView (8 knobs) and
				     ControlPanel (today's full surface) based on
				     `ui.viewMode`. -->
				<div
					class="content-area"
					class:two-col={ui.panels.midi &&
						ui.panels.controls &&
						adapter.capabilities.midiDevicePicker}
					class:one-col={((ui.panels.midi && adapter.capabilities.midiDevicePicker
						? 1
						: 0) + (ui.panels.controls ? 1 : 0)) ===
						1}
				>
					{#if ui.panels.midi && adapter.capabilities.midiDevicePicker}
						<div class="column column-left">
							<MidiDevices>
								{#if isGuitarAudioSelected}
									<GuitarInputPanel />
								{/if}
							</MidiDevices>
						</div>
					{/if}
					{#if ui.panels.controls}
						<div class="column column-center">
							{#if ui.viewMode === 'performance'}
								<PerformanceView />
							{:else}
								<ControlPanel />
							{/if}
						</div>
					{/if}
				</div>
			{/if}

			{#if ui.panels.activeNotes}
				<div class="active-notes-strip">
					<ActiveNotes />
				</div>
			{/if}

			{#if ui.panels.history || ui.panels.fretboard || ui.panels.piano}
				<div class="piano-area">
					{#if ui.panels.history}<HistoryStrip />{/if}
					{#if ui.panels.fretboard}<Fretboard />{/if}
					{#if ui.panels.piano}<Piano />{/if}
				</div>
			{/if}
		{:else if ui.activeTab === 'io'}
			<!-- I/O tab — subtabs hold Input (sources + calibration) and
			     Output (Voice Generation Chain + synth/FX). The Output
			     subtab fills in across steps 3-4 of the restructure. -->
			<div class="io-area">
				{#if ui.ioSubtab === 'input'}
					{#if adapter.capabilities.inputSourcePicker}
						<InputPanel />
					{:else}
						<div class="surface-unavailable font-ui">
							Input source is owned by the DAW in plugin mode.
							Route MIDI / audio to the plugin from your host.
						</div>
					{/if}
				{:else}
					<OutputPanel />
				{/if}
			</div>
		{:else if ui.activeTab === 'companion'}
			<!-- Companion tab: delayed-voice configuration (canon lanes
			     + per-voice settings). The voice management UX lives
			     here so the Play tab stays focused on real-time harmony.
			     Hidden in plugin mode where the plugin core doesn't wire
			     Companion lanes yet — deferred to v1.4. -->
			<div class="companion-area">
				{#if adapter.capabilities.companionLanes}
					<CompanionPanel />
				{:else}
					<div class="surface-unavailable font-ui">
						Companion lanes (canon + counterpoint) are not yet
						wired in plugin mode. Use the desktop app for now —
						this will land in v1.4.
					</div>
				{/if}
			</div>
		{:else if ui.activeTab === 'voices'}
			<!-- Voices tab: named voice presets (built-in SATB chorale
			     roles + user-defined). Canon voice cards pick from this
			     library to apply a full harmony config to each voice. -->
			<div class="voices-area">
				<VoicesPanel />
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
		/* Lock the grid to a single viewport-width column. Without this,
		   the implicit column defaults to `auto` (= max-content), and any
		   child whose intrinsic width exceeds 100vw (e.g. ActiveNotes'
		   nowrap note-list spans when many notes are pressed) silently
		   widens the column. Every sibling — including .piano-area, which
		   contains the fretboard — then inherits that wider column,
		   making the fretboard grow on note count. minmax(0, 1fr) tells
		   the grid: "ignore children's min-content; cap at 1fr of the
		   container width." Architectural lock against any conditional
		   content pushing the layout. */
		grid-template-columns: minmax(0, 1fr);
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

	.subtab-strip {
		display: flex;
		background: rgba(10, 9, 18, 0.92);
		border-bottom: 1px solid var(--color-border);
		padding: 0 16px;
	}

	.subtab-btn {
		background: transparent;
		border: none;
		border-bottom: 2px solid transparent;
		color: var(--color-text-dim);
		font-size: var(--font-size-xs);
		padding: 4px 12px;
		cursor: pointer;
	}

	.subtab-btn:hover {
		color: var(--color-accent-cyan);
	}

	.subtab-btn.active {
		color: var(--color-accent-cyan);
		border-bottom-color: var(--color-accent-cyan);
	}

	.io-area {
		background: rgba(15, 14, 26, 0.88);
		overflow-y: auto;
		height: 100%;
	}

	.chain-area {
		background: rgba(15, 14, 26, 0.88);
		overflow-y: auto;
	}

	.companion-area,
	.voices-area {
		background: rgba(15, 14, 26, 0.88);
		overflow-y: auto;
		height: 100%;
	}

	.surface-unavailable {
		padding: 32px;
		color: var(--color-text-secondary);
		max-width: 500px;
		font-size: var(--font-size-sm);
		line-height: 1.5;
	}

	.content-area {
		display: grid;
		gap: 1px;
		overflow: hidden;
		background: var(--color-border);
	}
	.content-area.two-col {
		grid-template-columns: 1fr 1.6fr;
	}
	.content-area.one-col {
		grid-template-columns: 1fr;
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
