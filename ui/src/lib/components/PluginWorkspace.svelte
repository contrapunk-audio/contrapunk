<script lang="ts">
	import { onMount } from 'svelte';
	import {
		engine,
		ALL_KEYS,
		KEY_DISPLAY,
		OCTAVE_MODES,
		type HarmonyModeName,
		type KeyName,
		type OctaveModeName,
		type CounterpointSpeciesName
	} from '$lib/stores/engine.svelte';
	import { adapter } from '$lib/adapter';
	import type { PluginInputMode, PluginMidiOutputMode } from '$lib/adapter/types';
	import { ui } from '$lib/stores/ui.svelte';
	import { midi } from '$lib/stores/midi.svelte';
	import { compositeModeOptions, decodeMode, encodeMode } from '$lib/harmony/modeComposite';
	import PixelSelect from './PixelSelect.svelte';
	import Knob from './Knob.svelte';
	import ActiveNotes from './ActiveNotes.svelte';
	import Piano from './Piano.svelte';
	import Fretboard from './Fretboard.svelte';
	import HistoryStrip from './HistoryStrip.svelte';
	import LiveLines from './LiveLines.svelte';
	import VoiceGenerationChain from './VoiceGenerationChain.svelte';
	import CompanionPanel from './CompanionPanel.svelte';
	import VoicesPanel from './VoicesPanel.svelte';
	import InputPanel from './InputPanel.svelte';
	import OutputPanel from './OutputPanel.svelte';
	import EnsemblePresetBar from './EnsemblePresetBar.svelte';

	type Visual = 'lines' | 'piano' | 'fretboard' | 'history';
	type LineFilter = 'all' | 'player' | 'harmony' | 'canon' | 'counterpoint';

	const pluginSurface = adapter.capabilities.pluginMidiOutputMode;
	const keyOptions = ALL_KEYS.map((key) => ({ value: key, label: KEY_DISPLAY[key] }));
	// Plugin host currently exposes species 1 only. Do not offer species
	// controls that the audio processor cannot apply or automate yet.
	const modeOptions = compositeModeOptions().filter(
		(option) => !option.value.startsWith('StrictCounterpoint:') || option.value.endsWith('Species1')
	);
	const voiceOptions = [1, 2, 3, 4].map((count) => ({
		value: String(count),
		label: count === 1 ? '1 voice' : `${count} voices`
	}));
	const inputOptions = [
		{ value: 'midi', label: 'MIDI' },
		{ value: 'audio', label: 'Guitar audio' }
	];
	const outputOptions = [
		{ value: 'full', label: 'Input + Contrapunk' },
		{ value: 'pass_through', label: 'Input only' }
	];
	const octaveOptions = OCTAVE_MODES.map((mode) => ({ value: mode.name, label: mode.label }));

	let inputMode = $state<PluginInputMode>('midi');
	let outputMode = $state<PluginMidiOutputMode>('full');
	let synthEnabled = $state(true);
	let visual = $state<Visual>('lines');
	let lineFilter = $state<LineFilter>('all');
	let panicSent = $state(false);
	let counterpointDetails: HTMLDetailsElement;
	let counterpointFocus = $state<'imitative' | 'species'>('imitative');
	let counterpointFocusVersion = $state(0);

	let positionOptions = $derived(
		Array.from({ length: engine.voiceCount }, (_, index) => ({
			value: String(index),
			label: index === engine.voicePosition ? `Voice ${index + 1} · you` : `Voice ${index + 1}`
		}))
	);
	let inputActive = $derived(engine.inputNotes.length > 0);
	let generatedActive = $derived(
		engine.harmonyNotes.length > 0 ||
		engine.canonNotes.length > 0 ||
		engine.counterpointNotes.length > 0
	);
	let outputActive = $derived(
		outputMode === 'pass_through' ? inputActive : inputActive || generatedActive
	);
	let harmonyLabel = $derived(
		modeOptions.find((option) => option.value === encodeMode(engine.mode, engine.counterpointSpecies))?.label ?? engine.mode
	);
	let ensembleParts = $derived(
		1 +
		Math.max(0, engine.voiceCount - 1) +
		(engine.companionEnabled && engine.canonEnabled ? Math.max(1, engine.canonVoices.length) : 0) +
		(engine.counterpointNotes.length > 0 ? 1 : 0)
	);
	let timingLabel = $derived(pluginSurface ? 'HOST' : engine.isRunning ? 'LIVE' : 'READY');
	let groupFilter = $derived<LineFilter>(engine.canonEnabled ? 'canon' : 'counterpoint');
	let companionSummary = $derived(
		engine.companionEnabled
			? `${engine.canonEnabled ? `Canon ${engine.canonVoices.length || 1}v` : 'Canon off'} · ${engine.counterpointSpecies}`
			: 'Off'
	);

	async function refreshHostState() {
		await engine.syncFromBackend();
		[inputMode, outputMode, synthEnabled] = await Promise.all([
			adapter.getPluginInputMode(),
			adapter.getPluginMidiOutputMode(),
			adapter.getPluginSynthEnabled()
		]);
	}

	onMount(() => {
		try {
			const saved = localStorage.getItem('contrapunk-plugin-visual');
			if (saved === 'lines' || saved === 'piano' || saved === 'fretboard' || saved === 'history') visual = saved;
			if (saved === 'ensemble') visual = 'lines';
		} catch {
			/* localStorage unavailable */
		}
		void refreshHostState();
		return adapter.onPluginParamsUpdate(() => void refreshHostState());
	});

	function setVisual(next: Visual) {
		visual = next;
		try {
			localStorage.setItem('contrapunk-plugin-visual', next);
		} catch {
			/* localStorage unavailable */
		}
	}

	function onKeyChange(value: string) {
		void engine.setAutoKey(false);
		void engine.setKey(value as KeyName);
	}

	function onModeChange(value: string) {
		const decoded = decodeMode(value);
		if (decoded.mode) void engine.setMode(decoded.mode as HarmonyModeName);
		if (decoded.species) {
			void engine.setCounterpointSpecies(decoded.species as CounterpointSpeciesName);
		}
	}

	function onVoiceCountChange(value: string) {
		void engine.setVoiceCount(Number(value));
	}

	function onPositionChange(value: string) {
		void engine.setVoicePosition(Number(value));
	}

	function onSpread(value: number) {
		const amount = Math.max(0, Math.min(1, value));
		void engine.setOctaveMode(amount <= 0.01 ? 'None' : 'Spread');
		void engine.setOctaveIntensity(amount);
	}

	async function onInputMode(value: string) {
		inputMode = value === 'audio' ? 'audio' : 'midi';
		await adapter.setPluginInputMode(inputMode);
	}

	async function onOutputMode(value: string) {
		outputMode = value === 'pass_through' ? 'pass_through' : 'full';
		await adapter.setPluginMidiOutputMode(outputMode);
	}

	async function toggleSynth() {
		synthEnabled = !synthEnabled;
		await adapter.setPluginSynthEnabled(synthEnabled);
	}

	async function toggleRouting() {
		if (engine.isRunning) {
			await engine.stop();
		} else if (midi.selectedInput !== null) {
			await engine.start(midi.selectedInput, midi.selectedOutputs);
		}
	}

	async function panic() {
		await adapter.panicAllNotesOff();
		panicSent = true;
		window.setTimeout(() => (panicSent = false), 800);
	}

	function revealCounterpoint(group: 'imitative' | 'species') {
		counterpointFocus = group;
		counterpointFocusVersion += 1;
		counterpointDetails.open = true;
		requestAnimationFrame(() => counterpointDetails.scrollIntoView({ behavior: 'smooth', block: 'start' }));
	}

	async function addCanonVoice() {
		if (!engine.companionEnabled) await engine.setCompanionEnabled(true);
		if (!engine.canonEnabled) await engine.setCanonEnabled(true);
		if (engine.canonVoices.length < 8) await engine.addCanonVoice();
		revealCounterpoint('imitative');
	}

	async function addSpeciesLine() {
		if (!engine.companionEnabled) await engine.setCompanionEnabled(true);
		await adapter.counterpointSetConfig({ enabled: true });
		revealCounterpoint('species');
	}
</script>

<div class="workspace">
	<header class="signal-bar">
		<div class="brand-block">
			<img src="/logo.svg" alt="" class="brand-logo" />
			<span class="brand font-ui">CONTRAPUNK</span>
		</div>

		<div class="signal-path" aria-label="Ensemble signal path">
			<div class="signal-stage" class:active={inputActive}>
				<span class="signal-dot input-dot"></span>
				<span class="font-ui">PLAYER</span>
			</div>
			<span class="signal-arrow" aria-hidden="true">›</span>
			<div class="signal-stage" class:active={generatedActive}>
				<span class="signal-dot generated-dot"></span>
				<span class="font-ui">ENSEMBLE</span>
			</div>
			<span class="signal-arrow" aria-hidden="true">›</span>
			<div class="signal-stage" class:active={outputActive}>
				<span class="signal-dot output-dot"></span>
				<span class="font-ui">OUTPUT</span>
			</div>
		</div>

		<div class="host-controls">
			{#if pluginSurface}
				<div class="mini-select">
					<span class="mini-label font-ui">SOURCE</span>
					<PixelSelect options={inputOptions} value={inputMode} label="Source" help="Chooses whether the plugin listens to MIDI notes or detected guitar pitches." small={true} onchange={onInputMode} />
				</div>
				<div class="mini-select output-select">
					<span class="mini-label font-ui">OUTPUT</span>
					<PixelSelect options={outputOptions} value={outputMode} label="Output" help="Chooses whether output contains the original notes plus Contrapunk, or only the original input." small={true} onchange={onOutputMode} />
				</div>
			{:else}
				<button
					class="surface-state font-ui"
					class:active={engine.isRunning}
					type="button"
					disabled={!engine.isRunning && midi.selectedInput === null}
					title="Starts or stops listening to the selected standalone input source."
					onclick={toggleRouting}
				>
					{engine.isRunning ? '■ STOP' : '▶ START'}
				</button>
			{/if}
			<button class="panic-btn font-ui" class:sent={panicSent} type="button" title="Immediately releases every sounding and scheduled note if anything becomes stuck." onclick={panic}>
				{panicSent ? 'CLEARED' : 'PANIC'}
			</button>
			<button class="settings-btn font-ui" type="button" title="Open application settings." onclick={() => ui.openSettings()} aria-label="Open settings">⚙</button>
		</div>
	</header>

	<main class="workspace-scroll">
		<EnsemblePresetBar />

		<section class="context-strip" aria-label="Ensemble context">
			<div><span>KEY</span><strong>{KEY_DISPLAY[engine.key]}</strong></div>
			<div><span>SCALE</span><strong>{engine.scaleMode}</strong></div>
			<div><span>TIMING</span><strong>{timingLabel}</strong></div>
			<div class:live={generatedActive}><span>ENSEMBLE</span><strong>● {ensembleParts} PARTS ACTIVE</strong></div>
		</section>

		<section class="core-panel" aria-label="Essential harmony controls">
			<div class="core-heading">
				<strong class="font-ui">SELECTED · HARMONIC SUPPORT</strong>
				<span class="font-code">{harmonyLabel} · {engine.voiceCount} total voices</span>
			</div>
			<div class="control-cell">
				<div class="control-label"><span>KEY</span>{#if pluginSurface}<span class="host-badge">HOST</span>{/if}</div>
				<PixelSelect options={keyOptions} value={engine.key} label="Key" help="Sets the tonal center used to choose in-key harmony and counterpoint notes." small={true} onchange={onKeyChange} />
			</div>
			<div class="control-cell mode-cell">
				<div class="control-label"><span>HARMONY</span>{#if pluginSurface}<span class="host-badge">HOST</span>{/if}</div>
				<PixelSelect
					options={modeOptions}
					value={encodeMode(engine.mode, engine.counterpointSpecies)}
					label="Harmony mode"
					help="Chooses the musical rule Contrapunk uses to generate harmonic support from the notes you play."
					small={true}
					onchange={onModeChange}
				/>
			</div>
			<div class="control-cell">
				<div class="control-label"><span>VOICES</span>{#if pluginSurface}<span class="host-badge">HOST</span>{/if}</div>
				<PixelSelect options={voiceOptions} value={String(engine.voiceCount)} label="Voices" help="Sets the total harmonic texture size, including the part you play." small={true} onchange={onVoiceCountChange} />
			</div>
			<div class="control-cell">
				<div class="control-label"><span>YOU PLAY</span>{#if pluginSurface}<span class="host-badge">HOST</span>{/if}</div>
				<PixelSelect options={positionOptions} value={String(engine.voicePosition)} label="You play" help="Places your performed note within the texture: upper voice, inner voice, or bass." small={true} onchange={onPositionChange} />
			</div>
			<div class="control-cell spread-cell">
				<Knob
					value={engine.octaveIntensity}
					min={0}
					max={1}
					step={0.01}
					defaultValue={0}
					label="Spread"
					help="Widens generated harmony across registers. At 0% voices stay close; at 100% each successive harmony voice can move another octave upward."
					size={42}
					format={(value) => `${Math.round(value * 100)}%`}
					onchange={onSpread}
				/>
			</div>
			<div class="control-cell toggle-cell">
				<div class="control-label"><span>VOICE LEADING</span>{#if pluginSurface}<span class="host-badge">HOST</span>{/if}</div>
				<button
					class="toggle-btn font-ui"
					class:on={engine.voiceLeadingEnabled}
					type="button"
					aria-label="Voice leading"
					aria-pressed={engine.voiceLeadingEnabled}
					title="Smooths generated parts by preferring nearby notes and reducing awkward jumps between chords."
					onclick={() => engine.setVoiceLeading(!engine.voiceLeadingEnabled)}
				>
					{engine.voiceLeadingEnabled ? 'ON' : 'OFF'}
				</button>
			</div>
		</section>

		<section class="performance-panel">
			<ActiveNotes />
			<div class="visual-toolbar">
				<div class="visual-tabs" role="group" aria-label="Visualization">
					<button class:active={visual === 'lines'} aria-pressed={visual === 'lines'} type="button" title="Shows recent player, harmony, Canon, and Counterpoint notes as moving relationships." onclick={() => setVisual('lines')}>LIVE LINES</button>
					<button class:active={visual === 'piano'} aria-pressed={visual === 'piano'} type="button" title="Shows currently sounding notes on a playable piano keyboard." onclick={() => setVisual('piano')}>PIANO</button>
					<button class:active={visual === 'fretboard'} aria-pressed={visual === 'fretboard'} type="button" title="Shows currently sounding notes and scale positions on a guitar fretboard." onclick={() => setVisual('fretboard')}>FRETBOARD</button>
					<button class:active={visual === 'history'} aria-pressed={visual === 'history'} type="button" title="Shows a compact chronological history of performed and generated notes." onclick={() => setVisual('history')}>HISTORY</button>
				</div>
				{#if visual !== 'lines'}
					<div class="legend font-ui" aria-label="Note color legend">
						<span><i class="legend-input"></i>Input</span>
						<span><i class="legend-harmony"></i>Harmony</span>
						{#if visual === 'piano'}
							<span><i class="legend-canon"></i>Canon</span>
							<span><i class="legend-counterpoint"></i>Counterpoint</span>
						{/if}
						<span><i class="legend-borrowed"></i>Borrowed</span>
					</div>
				{/if}
			</div>
			<div class="visual-stage" class:live-lines-stage={visual === 'lines'}>
				{#if visual === 'lines'}
					<LiveLines bind:filter={lineFilter} />
				{:else if visual === 'piano'}
					<Piano />
				{:else if visual === 'fretboard'}
					<Fretboard />
				{:else}
					<HistoryStrip />
				{/if}
			</div>
		</section>

		<section class="relationship-cards" aria-label="How the ensemble responds">
			<button class="relationship player" class:active={lineFilter === 'player'} type="button" aria-pressed={lineFilter === 'player'} title="Filters Live Lines to only the melody you perform." onclick={() => (lineFilter = 'player')}>
				<div><strong>YOU</strong><span>{inputActive ? '● PLAYING' : 'SUBJECT'}</span></div>
				<p>Teal is the live melody you perform. Every generated part listens to it.</p>
			</button>
			<button class="relationship harmony" class:active={lineFilter === 'harmony'} type="button" aria-pressed={lineFilter === 'harmony'} title="Filters Live Lines to the chordal voices generated alongside your melody." onclick={() => (lineFilter = 'harmony')}>
				<div><strong>HARMONIC SUPPORT</strong><span>{Math.max(0, engine.voiceCount - 1)} VOICES</span></div>
				<p>Magenta voices arrive with you, creating chordal support rather than a separate melody.</p>
			</button>
			<button class="relationship counterpoint-group" class:active={lineFilter === groupFilter} type="button" aria-pressed={lineFilter === groupFilter} title="Filters Live Lines to the active imitative or species Counterpoint group." onclick={() => (lineFilter = groupFilter)}>
				<div><strong>COUNTERPOINT GROUP</strong><span>{engine.canonEnabled ? (engine.imitativeForm === 'strict_canon' ? 'STRICT CANON' : 'FREE IMITATION') : engine.counterpointSpecies}</span></div>
				<p>Gold imitation follows your subject; lime can move as an independent species line.</p>
			</button>
		</section>
		<div class="relationship-explainer font-ui">
			<strong>LIVE ENSEMBLE:</strong> compare color, timing, and note shape to see which parts support, imitate, or move independently.
		</div>

		<section class="progressive-sections" id="ensemble-groups">
			<details class="section-card harmonic-section">
				<summary>
					<span class="summary-title">HARMONIC SUPPORT</span>
					<span class="summary-state">{harmonyLabel} · {Math.max(0, engine.voiceCount - 1)} supporting voices</span>
				</summary>
				<div class="section-body harmony-details">
					<div class="detail-control">
						<span class="detail-label">Octave mode</span>
						<PixelSelect options={octaveOptions} value={engine.octaveMode} label="Octave mode" help="Chooses how generated harmony is distributed across registers: unchanged, spread, bass/treble split, or mirrored." small={true} onchange={(value) => engine.setOctaveMode(value as OctaveModeName)} />
					</div>
					<div class="detail-control">
						<span class="detail-label">Key detection</span>
						<button class="toggle-btn font-ui" class:on={engine.autoKey} type="button" aria-label="Automatic key detection" aria-pressed={engine.autoKey} title="When enabled, Contrapunk estimates the key from recent notes. Manual mode uses the selected Key control." onclick={() => engine.setAutoKey(!engine.autoKey)}>
							{engine.autoKey ? 'AUTO' : 'MANUAL'}
						</button>
					</div>
					<p class="section-copy">
						{pluginSurface
							? 'Controls marked HOST are stable DAW parameters for automation and controller assignment.'
							: 'These controls shape the chordal support that moves with your live melody.'}
					</p>
				</div>
			</details>

			<details class="section-card arrange-section" open>
				<summary>
					<span class="summary-title">ARRANGE YOUR ENSEMBLE</span>
					<span class="summary-state">{outputMode === 'full' ? 'Input + Contrapunk' : 'Input only'} · Synth {synthEnabled ? 'on' : 'off'}</span>
				</summary>
				<div class="section-body arrange-grid">
					<div class="arrange-main">
						<VoiceGenerationChain />
						<div class="arrange-actions">
							<button class="add-part canon-add font-ui" type="button" title="Adds another delayed imitative voice and opens its group editor." disabled={engine.canonVoices.length >= 8} onclick={addCanonVoice}>+ ADD CANON VOICE</button>
							<button class="add-part species-add font-ui" type="button" title="Enables the independent species Counterpoint line and opens its group editor." onclick={addSpeciesLine}>+ ADD SPECIES LINE</button>
						</div>
						<div class="channel-map font-code">
							<span>CH1 Melody</span><span>CH2–5 Harmony</span><span>CH6 Canon</span><span>CH7 Counterpoint</span>
						</div>
					</div>
					<aside class="arrange-side">
						{#if pluginSurface}
							<div class="setting-row">
								<span>Built-in synth</span>
								<button class="toggle-btn font-ui" class:on={synthEnabled} type="button" aria-label="Built-in synth" aria-pressed={synthEnabled} title="Turns Contrapunk's built-in instrument sound on or off. MIDI generation continues either way." onclick={toggleSynth}>{synthEnabled ? 'ON' : 'OFF'}</button>
							</div>
						{:else}
							<p class="section-copy">Choose the built-in synth or external MIDI per voice below.</p>
						{/if}
						<p class="section-copy">Routing state stays here; host setup recipes live in the documentation.</p>
					</aside>
				</div>
				{#if adapter.capabilities.inputSourcePicker}
					<div class="io-progressive">
						<InputPanel />
						<OutputPanel />
					</div>
				{/if}
			</details>

			<details class="section-card counterpoint-section" bind:this={counterpointDetails}>
				<summary>
					<span class="summary-title">COUNTERPOINT GROUPS</span>
					<span class="summary-state">Imitative · Species · {companionSummary}</span>
				</summary>
				<div class="section-body component-body"><CompanionPanel focusGroup={counterpointFocus} focusVersion={counterpointFocusVersion} /></div>
			</details>

			<details class="section-card advanced-section">
				<summary>
					<span class="summary-title">VOICE LIBRARY & ADVANCED</span>
					<span class="summary-state">SATB · immutable built-ins · user voices</span>
				</summary>
				<div class="section-body component-body"><VoicesPanel /></div>
			</details>
		</section>
	</main>
</div>

<style>
	.workspace {
		height: 100vh;
		width: 100vw;
		display: flex;
		flex-direction: column;
		background:
			linear-gradient(rgba(10, 9, 20, 0.94), rgba(10, 9, 20, 0.98)),
			repeating-linear-gradient(0deg, transparent 0 3px, rgba(79, 232, 195, 0.025) 3px 4px);
		color: var(--color-text-primary);
		overflow: hidden;
	}

	.signal-bar {
		position: relative;
		z-index: 10;
		display: grid;
		grid-template-columns: auto minmax(260px, 1fr) auto;
		align-items: center;
		gap: 14px;
		padding: 7px 9px;
		background: rgba(21, 20, 40, 0.98);
		border-bottom: 1px solid var(--color-border-active);
		box-shadow: 0 3px 0 rgba(255, 46, 136, 0.08);
	}

	.brand-block,
	.signal-path,
	.host-controls,
	.signal-stage,
	.legend,
	.legend span,
	.control-label,
	.setting-row {
		display: flex;
		align-items: center;
	}

	.brand-block { gap: 6px; }
	.brand-logo { width: 18px; height: 18px; image-rendering: pixelated; }
	.brand { color: var(--color-accent-magenta); font-size: 10px; letter-spacing: 1.4px; }

	.signal-path {
		justify-content: center;
		gap: 8px;
		min-width: 0;
	}
	.signal-stage { gap: 5px; color: var(--color-text-dim); font-size: 9px; letter-spacing: 0.8px; }
	.signal-stage.active { color: var(--color-text-primary); }
	.signal-dot { width: 7px; height: 7px; border: 1px solid #38344f; background: #171525; }
	.signal-stage.active .input-dot { background: var(--color-piano-input); box-shadow: 0 0 7px var(--color-piano-input); }
	.signal-stage.active .generated-dot { background: var(--color-piano-harmony); box-shadow: 0 0 7px var(--color-piano-harmony); }
	.signal-stage.active .output-dot { background: var(--color-accent-cyan); box-shadow: 0 0 7px var(--color-accent-cyan); }
	.signal-arrow { color: var(--color-text-dim); }

	.host-controls { gap: 7px; }
	.surface-state { align-self: center; padding: 5px 8px; border: 1px solid var(--color-border); background: var(--color-widget-bg); color: var(--color-text-dim); font-size: 8px; white-space: nowrap; cursor: pointer; }
	.surface-state.active { border-color: var(--color-piano-input); color: var(--color-piano-input); }
	.surface-state:disabled { opacity: 0.45; cursor: not-allowed; }
	.mini-select { display: flex; flex-direction: column; gap: 1px; min-width: 88px; }
	.output-select { min-width: 138px; }
	.mini-label { color: var(--color-text-dim); font-size: 8px; letter-spacing: 1px; }
	.panic-btn,
	.settings-btn,
	.toggle-btn,
	.visual-tabs button {
		border: 1px solid var(--color-border);
		border-radius: 0;
		background: var(--color-widget-bg);
		color: var(--color-text-secondary);
		cursor: pointer;
	}
	.panic-btn { align-self: flex-end; padding: 5px 9px; border-color: #ff335f; color: #fff; background: #59162b; font-size: 9px; }
	.panic-btn:hover, .panic-btn.sent { background: #ff335f; box-shadow: 0 0 9px rgba(255, 51, 95, 0.65); }
	.settings-btn { align-self: flex-end; width: 25px; height: 25px; font-size: 12px; }

	.workspace-scroll {
		flex: 1;
		min-height: 0;
		overflow-y: auto;
		padding: 8px;
		display: flex;
		flex-direction: column;
	}
	.context-strip { order: 2; display: grid; grid-template-columns: repeat(4, minmax(0, 1fr)); gap: 5px; margin-top: 5px; }
	.context-strip > div { min-width: 0; display: flex; align-items: center; gap: 7px; min-height: 31px; padding: 0 8px; border: 1px solid var(--color-border); background: rgba(18, 16, 32, 0.94); }
	.context-strip span { color: var(--color-text-secondary); font: 8px/1 var(--font-ui); }
	.context-strip strong { margin-left: auto; color: var(--color-text-primary); font: 9px/1 var(--font-code); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
	.context-strip .live strong { color: var(--color-piano-input); }
	.core-panel { order: 6; margin-top: 7px;
		display: grid;
		grid-template-columns: 0.8fr 1.35fr 0.8fr 0.9fr 1.2fr 0.9fr;
		gap: 5px;
		padding: 6px;
		background: rgba(26, 24, 51, 0.92);
		border: 1px solid var(--color-border);
		box-shadow: inset 3px 0 0 var(--color-accent-magenta);
	}
	.core-heading { grid-column: 1 / -1; display: flex; align-items: center; justify-content: space-between; gap: 8px; padding-bottom: 5px; border-bottom: 1px solid var(--color-border); }
	.core-heading strong { color: var(--color-accent-gold); font-size: 9px; letter-spacing: 0.9px; }
	.core-heading span { color: var(--color-text-secondary); font-size: 8px; }
	.control-cell { min-width: 0; display: flex; flex-direction: column; gap: 4px; }
	.control-label { justify-content: space-between; gap: 4px; color: var(--color-accent-gold); font: 9px/1 var(--font-ui); letter-spacing: 0.8px; }
	.host-badge { color: var(--color-accent-cyan); font-size: 7px; border: 1px solid rgba(51, 221, 255, 0.4); padding: 1px 2px; }
	.spread-cell { align-items: center; justify-content: center; }
	.spread-cell :global(.knob-tooltip) { z-index: 300; }
	.toggle-btn { min-height: 24px; font-size: 9px; padding: 3px 8px; }
	.toggle-btn.on { color: #07130f; background: var(--color-piano-input); border-color: var(--color-accent-cyan); box-shadow: 0 0 6px rgba(79, 232, 195, 0.45); }

	.performance-panel { order: 3; margin-top: 7px; border: 1px solid var(--color-border); background: rgba(15, 14, 26, 0.9); }
	.performance-panel :global(.strip) { padding: 5px 8px; border-bottom: 1px solid var(--color-border); }
	.visual-toolbar { display: flex; justify-content: space-between; align-items: center; gap: 8px; padding: 4px 6px; background: rgba(26, 24, 51, 0.8); }
	.visual-tabs { display: flex; gap: 2px; }
	.visual-tabs button { padding: 3px 8px; font: 8px/1 var(--font-ui); letter-spacing: 0.6px; }
	.visual-tabs button.active { color: #080612; background: var(--color-accent-cyan); border-color: var(--color-accent-cyan); }
	.legend { gap: 9px; flex-wrap: wrap; justify-content: flex-end; color: var(--color-text-secondary); font-size: 8px; }
	.legend span { gap: 3px; }
	.legend i { width: 7px; height: 7px; display: inline-block; }
	.legend-input { background: var(--color-piano-input); }
	.legend-harmony { background: var(--color-piano-harmony); }
	.legend-canon { background: #ffdd44; }
	.legend-counterpoint { background: #a3e635; }
	.legend-borrowed { background: var(--color-piano-borrowed); }
	.visual-stage { min-height: 112px; max-height: 220px; overflow: auto; }
	.visual-stage.live-lines-stage { max-height: none; overflow: hidden; }
	.visual-stage :global(.piano-wrapper), .visual-stage :global(.fretboard-wrapper) { margin: 0; }

	.relationship-cards { order: 4; display: grid; grid-template-columns: 0.8fr 1.1fr 1.1fr; gap: 5px; margin-top: 5px; }
	.relationship { min-width: 0; min-height: 58px; padding: 7px 8px; border: 1px solid var(--color-border); border-radius: 0; background: rgba(25, 22, 45, 0.96); text-align: left; cursor: pointer; }
	.relationship > div { display: flex; align-items: center; gap: 6px; }
	.relationship strong { color: var(--color-text-primary); font: 9px/1 var(--font-ui); }
	.relationship span { margin-left: auto; color: var(--color-text-dim); font: 7px/1 var(--font-ui); white-space: nowrap; }
	.relationship p { margin: 5px 0 0; color: var(--color-text-secondary); font: 8px/1.35 var(--font-ui); }
	.relationship.player { border-left: 4px solid var(--color-piano-input); }
	.relationship.harmony { border-left: 4px solid var(--color-piano-harmony); }
	.relationship.counterpoint-group { border-left: 4px solid #ffdd44; box-shadow: inset -3px 0 0 rgba(163, 230, 53, 0.55); }
	.relationship:hover { background: rgba(31, 28, 52, 0.98); }
	.relationship.active { border-color: var(--color-accent-cyan); background: rgba(23, 39, 52, 0.96); }
	.relationship.active span { color: var(--color-piano-input); }
	.relationship-explainer { order: 5; margin-top: 5px; min-height: 34px; padding: 7px 9px; border: 1px solid rgba(51, 221, 255, 0.34); background: rgba(10, 24, 31, 0.94); color: var(--color-text-secondary); font-size: 8px; line-height: 1.45; }
	.relationship-explainer strong { color: var(--color-accent-cyan); }

	.progressive-sections { order: 7; display: grid; gap: 4px; margin-top: 7px; }
	.arrange-section { order: 1; }
	.harmonic-section { order: 2; }
	.counterpoint-section { order: 3; }
	.advanced-section { order: 4; }
	.section-card { border: 1px solid var(--color-border); background: rgba(21, 20, 40, 0.92); }
	.section-card[open] { border-color: rgba(51, 221, 255, 0.45); }
	.section-card summary { list-style: none; display: flex; align-items: center; justify-content: space-between; gap: 8px; cursor: pointer; padding: 7px 9px; user-select: none; }
	.section-card summary::-webkit-details-marker { display: none; }
	.section-card summary::before { content: '▸'; color: var(--color-accent-cyan); margin-right: 5px; }
	.section-card[open] summary::before { content: '▾'; }
	.summary-title { margin-right: auto; color: var(--color-accent-gold); font: 10px/1 var(--font-ui); letter-spacing: 1px; }
	.summary-state { color: var(--color-text-secondary); font: 9px/1 var(--font-code); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
	.section-body { border-top: 1px solid var(--color-border); padding: 8px; }
	.harmony-details { display: grid; grid-template-columns: 1fr 1fr 2fr; gap: 8px; align-items: end; }
	.detail-control { display: flex; flex-direction: column; gap: 4px; }
	.detail-label { color: var(--color-text-secondary); font: 9px/1 var(--font-ui); }
	.section-copy { margin: 0; color: var(--color-text-secondary); font: 10px/1.45 var(--font-ui); }
	.arrange-grid { display: grid; grid-template-columns: minmax(0, 1.7fr) minmax(190px, 0.8fr); gap: 10px; }
	.arrange-actions { display: flex; gap: 5px; margin-top: 6px; }
	.add-part {
		min-height: 28px;
		padding: 5px 9px;
		border: 1px solid var(--color-border);
		border-radius: 0;
		background: var(--color-widget-bg);
		font-size: 8px;
		letter-spacing: 0.7px;
		cursor: pointer;
	}
	.add-part:disabled { opacity: 0.4; cursor: not-allowed; }
	.canon-add { border-color: var(--color-accent-gold); color: var(--color-accent-gold); }
	.species-add { border-color: #a3e635; color: #a3e635; }
	.add-part:hover:not(:disabled) { background: rgba(255, 255, 255, 0.07); }
	.channel-map { display: flex; flex-wrap: wrap; gap: 5px 10px; margin-top: 6px; color: var(--color-text-secondary); font-size: 9px; }
	.arrange-side { display: flex; flex-direction: column; gap: 8px; border-left: 1px solid var(--color-border); padding-left: 9px; }
	.setting-row { justify-content: space-between; color: var(--color-text-primary); font: 10px/1 var(--font-ui); }
	.io-progressive { display: grid; grid-template-columns: minmax(0, 1fr) minmax(0, 1fr); gap: 8px; margin-top: 8px; padding-top: 8px; border-top: 1px solid var(--color-border); }
	.component-body { padding: 0; max-height: 520px; overflow: auto; }

	@media (max-width: 760px) {
		.signal-bar { grid-template-columns: 1fr auto; }
		.signal-path { grid-column: 1 / -1; grid-row: 2; }
		.context-strip { grid-template-columns: repeat(2, minmax(0, 1fr)); }
		.core-panel { grid-template-columns: repeat(3, 1fr); }
		.relationship-cards { grid-template-columns: 1fr; }
		.arrange-grid, .io-progressive { grid-template-columns: 1fr; }
		.arrange-side { border-left: 0; border-top: 1px solid var(--color-border); padding: 8px 0 0; }
		.legend { display: none; }
	}
</style>
