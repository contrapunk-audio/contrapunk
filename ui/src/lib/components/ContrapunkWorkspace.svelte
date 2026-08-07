<script lang="ts">
	import { onMount, tick } from 'svelte';
	import { adapter } from '$lib/adapter';
	import {
		ALL_KEYS,
		ALL_MODES,
		KEY_DISPLAY,
		SCALE_FAMILIES,
		engine,
		type HarmonyModeName,
		type KeyName,
		type ScaleModeName
	} from '$lib/stores/engine.svelte';
	import { midi } from '$lib/stores/midi.svelte';
	import { phrase } from '$lib/stores/phrase.svelte';
	import { synth } from '$lib/stores/synth.svelte';
	import { slide } from '$lib/stores/slide.svelte';
	import { tone } from '$lib/stores/tone.svelte';
	import { transport } from '$lib/stores/transport.svelte';
	import { PIANO_KEY_COUNTS, ui } from '$lib/stores/ui.svelte';
	import { attachKeyboardInput } from '$lib/keyboard-input';
	import CompanionPanel from '$lib/components/CompanionPanel.svelte';
	import ControlPanel from '$lib/components/ControlPanel.svelte';
	import EnsemblePresetBar from '$lib/components/EnsemblePresetBar.svelte';
	import InputPanel from '$lib/components/InputPanel.svelte';
	import OutputPanel from '$lib/components/OutputPanel.svelte';
	import PerformanceView from '$lib/components/PerformanceView.svelte';
	import PluginRoutingPanel from '$lib/components/PluginRoutingPanel.svelte';
	import PresetManager from '$lib/components/PresetManager.svelte';
	import VoicesPanel from '$lib/components/VoicesPanel.svelte';
	import ExplicitIntervalMapPanel from '$lib/components/ExplicitIntervalMapPanel.svelte';
	import ArrangementMixer from '$lib/prototype/ArrangementMixer.svelte';
	import ExpressionRoll from '$lib/prototype/ExpressionRoll.svelte';
	import PatternLanePanel from '$lib/prototype/PatternLanePanel.svelte';
	import PhraseControl from '$lib/components/PhraseControl.svelte';
	import ElixirWorkspace from '$lib/components/elixir/ElixirWorkspace.svelte';

	type SetupSection = 'input' | 'harmony' | 'canon' | 'counterpoint' | 'output' | 'presets' | 'advanced';

	const VIRTUAL_COMPUTER_KEYBOARD = 999_998;
	const VIRTUAL_GUITAR_AUDIO = 999_997;
	const VIRTUAL_TONE_SOURCE = 999_996;
	const scaleOptions = SCALE_FAMILIES.flatMap((family) => family.modes);

	let initialized = $state(false);
	let initError = $state<string | null>(null);
	let panicSent = $state(false);
	let setupDialog = $state<HTMLDialogElement>();
	let setupOpen = $state(false);
	let setupSection = $state<SetupSection>('input');
	let companionFocus = $state<'imitative' | 'species'>('imitative');
	let companionFocusVersion = $state(0);
	let activeView = $state<'harmony' | 'synth'>('harmony');

	let inputLive = $derived(engine.inputNotes.length > 0);
	let ensembleLive = $derived(engine.harmonyNotes.length > 0 || engine.canonNotes.length > 0 || engine.counterpointNotes.length > 0);
	let outputLive = $derived(inputLive || ensembleLive);
	let sourceName = $derived.by(() => {
		if (adapter.capabilities.pluginMidiOutputMode) return 'Host MIDI';
		if (midi.selectedInput === VIRTUAL_GUITAR_AUDIO) return 'Guitar Audio';
		if (midi.selectedInput === VIRTUAL_COMPUTER_KEYBOARD) return 'Computer Keys';
		if (midi.selectedInput === VIRTUAL_TONE_SOURCE) return 'Tone';
		if (midi.selectedInput === null) return 'No input';
		return midi.inputs.find((device) => device.index === midi.selectedInput)?.name ?? 'MIDI Controller';
	});
	let spread = $derived(engine.octaveMode === 'None' ? 0 : engine.octaveIntensity);
	let maxVoiceCount = $derived(adapter.capabilities.pluginMidiOutputMode ? 4 : 8);
	let registerOptions = $derived(
		Array.from({ length: engine.voiceCount }, (_, index) => ({
			value: index,
			label: engine.voiceCount <= 4 ? (['Soprano', 'Alto', 'Tenor', 'Bass'][index] ?? `Voice ${index + 1}`) : `Voice ${index + 1}`
		}))
	);
	let quickModes = $derived.by(() => {
		const supported = ALL_MODES.filter(
			(mode) => adapter.capabilities.intervalMaps || mode.name !== 'ExplicitIntervals'
		);
		if (supported.some((mode) => mode.name === engine.mode)) return supported;
		return [{ name: engine.mode, label: engine.mode, shortLabel: engine.mode, tooltip: '' }, ...supported];
	});

	onMount(() => {
		let cancelled = false;
		const unsubscribeSynth = adapter.capabilities.pluginMidiOutputMode
			? adapter.onPluginParamsUpdate(() => void synth.syncFromBackend())
			: () => {};
		void (async () => {
			try {
				ui.restoreAppearance();
				await adapter.init();
				await engine.syncFromBackend();
				await slide.init();
				if (adapter.capabilities.pluginMidiOutputMode) await engine.restoreCompanionSettings();
				else await engine.restoreSettings();
				await midi.hydratePermission();
				await midi.hydrateVoiceOutputs(engine.voicePosition);
				await Promise.all([synth.syncFromBackend(), transport.syncFromBackend(), phrase.init()]);
				try {
					await transport.setMetronomeEnabled(false);
				} catch {
					/* Surface has no transport. */
				}
				if (!adapter.capabilities.pluginMidiOutputMode) {
					if (midi.selectedInput === null) {
						midi.selectVirtualInput(VIRTUAL_COMPUTER_KEYBOARD);
					}
					if (!engine.isRunning && midi.selectedInput !== null) {
						await engine.start(midi.selectedInput, midi.selectedOutputs);
					}
				}
				if (!cancelled) initialized = true;
			} catch (error) {
				if (!cancelled) initError = `Could not initialize Contrapunk: ${error}`;
			}
		})();
		return () => {
			cancelled = true;
			unsubscribeSynth();
			void tone.stop();
		};
	});

	$effect(() =>
		attachKeyboardInput({
			onNoteOn: (note) => void adapter.injectNoteOn(note, 100),
			onNoteOff: (note) => void adapter.injectNoteOff(note),
			enabled: () => midi.selectedInput === VIRTUAL_COMPUTER_KEYBOARD && engine.isRunning
		})
	);

	async function toggleRouting() {
		if (engine.isRunning) await engine.stop();
		else if (midi.selectedInput !== null) await engine.start(midi.selectedInput, midi.selectedOutputs);
	}

	async function panic() {
		await tone.panic();
		panicSent = true;
		window.setTimeout(() => (panicSent = false), 700);
	}

	async function setSpread(value: number) {
		if (value <= 0.01) {
			await engine.setOctaveIntensity(0);
			await engine.setOctaveMode('None');
			return;
		}
		if (engine.octaveMode === 'None') await engine.setOctaveMode('Spread');
		await engine.setOctaveIntensity(value);
	}

	async function openSetup(section: string = 'input', focus?: string) {
		setupOpen = true;
		setupSection = section as SetupSection;
		if (section === 'canon' || section === 'counterpoint') {
			companionFocus = section === 'canon' ? 'imitative' : 'species';
			companionFocusVersion += 1;
		}
		if (!setupDialog?.open) setupDialog?.showModal();
		await tick();
		const targetSection = section === 'counterpoint' ? 'canon' : section;
		const target = (focus ? document.getElementById(focus) : null) ?? document.getElementById(`setup-${targetSection}`);
		target?.scrollIntoView({ block: 'start', behavior: 'smooth' });
		target?.focus({ preventScroll: true });
	}
</script>

<div class="prototype-shell">
	<header class="app-header">
		<div class="identity">
			<img src="/logo.svg" alt="Contrapunk" />
			{#if adapter.capabilities.audioFx}
				<nav class="view-tabs" aria-label="Main view">
					<button class:active={activeView === 'harmony'} aria-pressed={activeView === 'harmony'} onclick={() => (activeView = 'harmony')}>Perform</button>
					<span aria-hidden="true">|</span>
					<button class:active={activeView === 'synth'} aria-pressed={activeView === 'synth'} onclick={() => (activeView = 'synth')}>Synth</button>
				</nav>
			{/if}
		</div>
		<div class="signal-path" aria-label="Current signal path">
			<span class:live={inputLive}><i></i>{sourceName}</span>
			<b aria-hidden="true">→</b>
			<span class:live={ensembleLive}><i></i>Ensemble</span>
			<b aria-hidden="true">→</b>
			<span class:live={outputLive}><i></i>Output</span>
		</div>
		<div class="header-actions">
			{#if adapter.capabilities.transportControl}
				<button class="transport-button" aria-label={transport.running ? 'Stop transport' : 'Play transport'} onclick={() => transport.running ? transport.stop() : transport.play()}>{transport.running ? '■' : '▶'}</button>
				<label class="tempo"><span>BPM</span><input type="number" min="20" max="400" value={transport.bpm} onchange={(event) => transport.setBpm(Number(event.currentTarget.value))} /></label>
			{/if}
			{#if !adapter.capabilities.pluginMidiOutputMode}<button class:running={engine.isRunning} disabled={!engine.isRunning && midi.selectedInput === null} onclick={toggleRouting}>{engine.isRunning ? 'Routing on' : 'Start routing'}</button>{:else}<span class="host-owned">DAW HOST</span>{/if}
			<button class:confirmed={panicSent} onclick={panic}>{panicSent ? 'Cleared' : 'Panic'}</button>
			<button class="setup-button" onclick={() => openSetup('input')}>Setup</button>
		</div>
	</header>

	{#if activeView === 'synth' && adapter.capabilities.audioFx}
		<div class="synth-body"><ElixirWorkspace embedded /></div>
	{:else}
	<main class="performance-body">
		{#if initError}
			<div class="notice error" role="alert">{initError}</div>
		{:else if !initialized}
			<div class="notice">Initializing the real Contrapunk engine…</div>
		{:else}
			<section class="quick-controls" aria-label="Performance controls">
				<label><span>KEY</span><select value={engine.key} onchange={(event) => engine.setKey(event.currentTarget.value as KeyName)}>{#each ALL_KEYS as key}<option value={key}>{KEY_DISPLAY[key]}</option>{/each}</select></label>
				<label><span>SCALE</span><select value={engine.scaleMode} onchange={(event) => engine.setScaleMode(event.currentTarget.value as ScaleModeName)}>{#each scaleOptions as option}<option value={option.name}>{option.label}</option>{/each}</select></label>
				<label><span>HARMONY</span><select value={engine.mode} onchange={(event) => engine.setMode(event.currentTarget.value as HarmonyModeName)}>{#each quickModes as mode}<option value={mode.name}>{mode.label}</option>{/each}</select></label>
				<label><span>VOICES</span><select value={engine.voiceCount} onchange={(event) => engine.setVoiceCount(Number(event.currentTarget.value))}>{#each Array.from({ length: maxVoiceCount }, (_, index) => index + 1) as count}<option value={count}>{count}</option>{/each}</select></label>
				<label><span>YOUR REGISTER</span><select value={engine.voicePosition} onchange={(event) => engine.setVoicePosition(Number(event.currentTarget.value))}>{#each registerOptions as option}<option value={option.value}>{option.label}</option>{/each}</select></label>
				<label class="spread"><span>SPREAD <output>{Math.round(spread * 100)}</output></span><input type="range" min="0" max="1" step="0.01" value={spread} oninput={(event) => setSpread(Number(event.currentTarget.value))} /></label>
			</section>

			<div class="live-grid">
				<ExpressionRoll />
			</div>
			<ArrangementMixer {openSetup} openSynth={() => (activeView = 'synth')} />
		{/if}
	</main>
	{/if}

	<dialog class="setup-dialog" bind:this={setupDialog} aria-labelledby="setup-title" onclose={() => (setupOpen = false)}>
		<div class="dialog-frame">
			<header class="dialog-header">
				<div><p>CONTRAPUNK</p><h1 id="setup-title">Setup</h1></div>
				<button aria-label="Close setup" onclick={() => setupDialog?.close()}>Close</button>
			</header>
			<div class="dialog-body">
				<nav aria-label="Setup sections">
					<button class:active={setupSection === 'input'} onclick={() => openSetup('input')}>01 Input</button>
					<button class:active={setupSection === 'harmony'} onclick={() => openSetup('harmony')}>02 Harmony</button>
					<button class:active={setupSection === 'canon' || setupSection === 'counterpoint'} onclick={() => openSetup('canon')}>03 Canon + Counterpoint</button>
					<button class:active={setupSection === 'output'} onclick={() => openSetup('output')}>04 Output + Sound</button>
					<button class:active={setupSection === 'presets'} onclick={() => openSetup('presets')}>05 Presets + Voices</button>
					<button class:active={setupSection === 'advanced'} onclick={() => openSetup('advanced')}>06 Advanced</button>
				</nav>
				<div class="setup-scroll">
					<section id="setup-input" tabindex="-1"><div class="section-heading"><span>01</span><h2>Input</h2></div>{#if adapter.capabilities.inputSourcePicker}<InputPanel />{:else}<div class="notice">The host owns input selection.</div>{/if}</section>
					<section id="setup-harmony" tabindex="-1"><div class="section-heading"><span>02</span><h2>Harmony</h2></div><div id="harmony-controls">{#if setupOpen}<PerformanceView midiLearnEnabled={setupSection === 'harmony' && midi.selectedInput !== VIRTUAL_GUITAR_AUDIO} />{/if}</div>{#if adapter.capabilities.intervalMaps && engine.mode === 'ExplicitIntervals'}<ExplicitIntervalMapPanel />{/if}</section>
					<section id="setup-canon" tabindex="-1"><div class="section-heading"><span>03</span><h2>Canon + Counterpoint</h2></div>{#if adapter.capabilities.phraseContext}<PhraseControl />{/if}{#if adapter.capabilities.companionLanes}<div id="canon-controls"><CompanionPanel focusGroup={companionFocus} focusVersion={companionFocusVersion} /></div>{#if adapter.capabilities.patternLanes}<PatternLanePanel />{/if}{:else}<div class="notice">Companion lanes are unavailable on this surface.</div>{/if}</section>
					<section id="setup-output" tabindex="-1"><div class="section-heading"><span>04</span><h2>Output + Sound</h2></div>{#if adapter.capabilities.pluginMidiOutputMode}<PluginRoutingPanel />{/if}<div id="output-routing"><OutputPanel /></div></section>
					<section id="setup-presets" tabindex="-1"><div class="section-heading"><span>05</span><h2>Presets + Voices</h2></div><EnsemblePresetBar /><div class="preset-grid"><PresetManager /><VoicesPanel /></div></section>
					<section id="setup-advanced" tabindex="-1">
						<div class="section-heading"><span>06</span><h2>Advanced</h2></div>
						<div class="interface-settings" aria-label="Interface preferences">
							<label><span>Interface scale</span><input type="range" min="0.75" max="2" step="0.05" value={ui.uiScale} oninput={(event) => ui.setUiScale(Number(event.currentTarget.value))} /><output>{Math.round(ui.uiScale * 100)}%</output></label>
							<label><span>Text scale</span><input type="range" min="0.75" max="1.5" step="0.05" value={ui.fontScale} oninput={(event) => ui.setFontScale(Number(event.currentTarget.value))} /><output>{Math.round(ui.fontScale * 100)}%</output></label>
							<label><span>MIDI keyboard size</span><select value={ui.pianoKeyCount} onchange={(event) => ui.setPianoKeyCount(Number(event.currentTarget.value))}>{#each PIANO_KEY_COUNTS as count}<option value={count}>{count} keys</option>{/each}</select><output>{ui.pianoKeyCount}</output></label>
							<label class="check"><input type="checkbox" checked={ui.showNoteLabels} onchange={(event) => ui.setShowNoteLabels(event.currentTarget.checked)} /><span>Show note labels</span></label>
							<label class="check"><input type="checkbox" checked={ui.noteLingering} onchange={(event) => ui.setNoteLingering(event.currentTarget.checked)} /><span>Released-note trail</span></label>
							<label class="check"><input type="checkbox" checked={ui.animationsEnabled} onchange={() => ui.toggleAnimations()} /><span>Interface motion</span></label>
						</div>
						<ControlPanel />
					</section>
				</div>
			</div>
		</div>
	</dialog>
</div>

<style>
	:global(html), :global(body) { background: #050505 !important; }
	.prototype-shell {
		--proto-bg: #050505;
		--proto-panel: #0a0a0a;
		--proto-surface: #0e0e0e;
		--proto-hover: #171717;
		--proto-line: #292929;
		--proto-line-strong: #4a4a4a;
		--proto-text: #f2f2f2;
		--proto-muted: #a2a2a2;
		--proto-dim: #666;
		--color-bg-deep: #050505;
		--color-bg-base: #090909;
		--color-bg-panel: #0d0d0d;
		--color-widget-bg: #111;
		--color-widget-inactive: #181818;
		--color-accent-magenta: #fff;
		--color-accent-magenta-dim: #aaa;
		--color-accent-cyan: #fff;
		--color-accent-cyan-dim: #aaa;
		--color-accent-teal: #ddd;
		--color-accent-pink: #ddd;
		--color-accent-amber: #bbb;
		--color-accent-gold: #fff;
		--color-piano-input: #4fe8c3;
		--color-piano-harmony: #ff2e88;
		--color-piano-borrowed: #8a5cff;
		--color-piano-in-scale: #222;
		--color-text-primary: #f2f2f2;
		--color-text-secondary: #aaa;
		--color-text-dim: #666;
		--color-border: #292929;
		--color-border-active: #fff;
		height: 100vh;
		overflow: hidden;
		background: var(--proto-bg);
		color: var(--proto-text);
		font-family: var(--font-grotesk);
	}
	.app-header { position: relative; z-index: 20; display: grid; height: 52px; grid-template-columns: minmax(150px, .8fr) minmax(240px, 1fr) auto; align-items: center; gap: 18px; padding: 0 14px; border-bottom: 1px solid var(--proto-line-strong); background: rgba(5, 5, 5, .96); }
	.identity { display: flex; align-items: center; gap: 12px; }
	.identity img { width: 28px; height: 34px; object-fit: contain; }
	.view-tabs { display: flex; align-items: center; gap: 7px; color: var(--proto-line-strong); }
	.view-tabs button { padding: 4px 0; border: 0; background: transparent; color: var(--proto-dim); font: 650 11px var(--font-grotesk); }
	.view-tabs button:hover, .view-tabs button.active { color: var(--proto-text); }
	.view-tabs button.active { box-shadow: 0 1px var(--proto-text); }
	.signal-path { display: flex; align-items: center; justify-content: center; gap: 9px; color: var(--proto-dim); font-size: 10px; }
	.signal-path span { display: inline-flex; align-items: center; gap: 5px; max-width: 130px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
	.signal-path i { width: 5px; height: 5px; border: 1px solid currentColor; border-radius: 50%; }
	.signal-path span.live { color: var(--proto-text); }
	.signal-path span.live i { background: currentColor; }
	.signal-path b { color: var(--proto-line-strong); font-family: var(--font-code); }
	.header-actions { display: flex; align-items: center; justify-content: flex-end; gap: 6px; }
	.header-actions button, .dialog-header button { min-height: 30px; padding: 0 10px; border: 1px solid var(--proto-line-strong); background: transparent; color: var(--proto-text); font: 650 10px var(--font-grotesk); }
	.header-actions button:hover, .dialog-header button:hover { border-color: var(--proto-text); background: var(--proto-hover); }
	.header-actions button.running, .header-actions button.confirmed, .header-actions .setup-button { background: var(--proto-text); color: var(--proto-bg); }
	.header-actions button:disabled { opacity: .35; }
	.host-owned { padding: 7px 9px; border: 1px solid var(--proto-line-strong); color: var(--proto-muted); font: 700 9px var(--font-code); letter-spacing: .1em; }
	.transport-button { width: 31px; padding: 0 !important; font-family: var(--font-code) !important; }
	.tempo { display: flex; height: 30px; align-items: center; gap: 5px; padding: 0 7px; border: 1px solid var(--proto-line); color: var(--proto-muted); font: 8px var(--font-code); }
	.tempo input { width: 39px; border: 0; background: transparent; color: var(--proto-text); font: 10px var(--font-code); }
	.synth-body { height: calc(100vh - 52px); overflow: hidden; }
	.synth-body > :global(*) { height: 100%; }
	.performance-body { box-sizing: border-box; display: grid; width: min(1480px, calc(100% - 20px)); height: calc(100vh - 52px); grid-template-rows: auto minmax(0, 1fr) 146px; margin: 0 auto; gap: 8px; overflow: hidden; padding: 8px 0; }
	.quick-controls { display: grid; grid-template-columns: .7fr 1.2fr 1.3fr .55fr .9fr 1fr; border: 1px solid var(--proto-line); background: var(--proto-panel); }
	.quick-controls label { display: grid; min-width: 0; gap: 3px; padding: 6px 10px; border-right: 1px solid var(--proto-line); }
	.quick-controls label:last-child { border-right: 0; }
	.quick-controls span { color: var(--proto-muted); font: 700 8px var(--font-code); letter-spacing: .1em; }
	.quick-controls select { width: 100%; min-width: 0; border: 0; background: transparent; color: var(--proto-text); font: 600 11px var(--font-grotesk); }
	.quick-controls .spread span { display: flex; justify-content: space-between; }
	.quick-controls input[type='range'] { width: 100%; accent-color: var(--proto-text); }
	.live-grid { min-height: 0; }
	.live-grid > :global(*) { height: 100%; }
	.notice { padding: 18px; border: 1px solid var(--proto-line); background: var(--proto-panel); color: var(--proto-muted); font-size: 12px; }
	.notice.error { border-color: #777; color: #fff; }
	.setup-dialog { width: calc(100vw - 16px); max-width: none; height: calc(100vh - 16px); max-height: none; padding: 0; border: 1px solid #777; background: var(--proto-bg); color: var(--proto-text); font-family: var(--font-grotesk); filter: grayscale(1); }
	.setup-dialog::backdrop { background: rgba(0, 0, 0, .82); backdrop-filter: blur(5px); }
	.dialog-frame { display: grid; height: 100%; grid-template-rows: 62px 1fr; }
	.dialog-header { display: flex; align-items: center; justify-content: space-between; padding: 0 16px; border-bottom: 1px solid var(--proto-line-strong); }
	.dialog-header p { margin: 0 0 2px; color: var(--proto-muted); font: 700 8px var(--font-code); letter-spacing: .16em; }
	.dialog-header h1 { margin: 0; font-size: 21px; font-weight: 600; }
	.dialog-body { display: grid; min-height: 0; grid-template-columns: 160px 1fr; }
	.dialog-body > nav { display: flex; flex-direction: column; padding: 10px; border-right: 1px solid var(--proto-line); background: #080808; }
	.dialog-body > nav button { min-height: 42px; padding: 0 9px; border: 0; border-bottom: 1px solid var(--proto-line); background: transparent; color: var(--proto-muted); font: 600 10px var(--font-grotesk); text-align: left; }
	.dialog-body > nav button:hover, .dialog-body > nav button.active { background: var(--proto-hover); color: var(--proto-text); }
	.dialog-body > nav button.active { box-shadow: inset 2px 0 var(--proto-text); }
	.setup-scroll { min-width: 0; overflow-y: auto; scroll-behavior: smooth; }
	.setup-scroll > section { min-height: 300px; padding: 28px; border-bottom: 1px solid var(--proto-line-strong); scroll-margin-top: 0; outline: none; }
	.setup-scroll > section:focus .section-heading { box-shadow: inset 2px 0 var(--proto-text); }
	.section-heading { display: flex; align-items: flex-start; gap: 12px; margin: -4px 0 24px; padding: 4px 0 14px; border-bottom: 1px solid var(--proto-line); }
	.section-heading > span { color: var(--proto-dim); font: 11px var(--font-code); }
	.section-heading h2 { margin: 0; font-size: 18px; font-weight: 600; }
	.preset-grid { display: grid; grid-template-columns: minmax(260px, .7fr) minmax(420px, 1.3fr); gap: 12px; margin-top: 12px; }
	.interface-settings { display: grid; grid-template-columns: repeat(2, minmax(180px, 1fr)); gap: 8px; margin-bottom: 16px; padding: 12px; border: 1px solid var(--proto-line); }
	.interface-settings label { display: grid; grid-template-columns: 110px 1fr 42px; align-items: center; gap: 8px; color: var(--proto-muted); font-size: 10px; }
	.interface-settings input[type='range'] { width: 100%; accent-color: var(--proto-text); }
	.interface-settings select { width: 100%; border: 1px solid var(--proto-line); background: var(--proto-surface); color: var(--proto-text); font: 10px var(--font-code); }
	.interface-settings output { color: var(--proto-text); font: 9px var(--font-code); text-align: right; }
	.interface-settings .check { display: flex; min-height: 28px; align-items: center; }
	.interface-settings input[type='checkbox'] { accent-color: var(--proto-text); }
	.setup-dialog :global(.companion-root > .header) { align-items: flex-start; flex-direction: column; gap: 12px; padding: 12px 14px; }
	.setup-dialog :global(.companion-root > .header .header-controls) { width: 100%; flex-wrap: wrap; justify-content: flex-start; gap: 10px 14px; }
	.setup-dialog :global(.companion-root .header-left .subtitle),
	.setup-dialog :global(.companion-root .lane-subtitle),
	.setup-dialog :global(.companion-root .footer-hint) { display: none; }
	.setup-dialog :global(.companion-root .body-stack) { gap: 12px; padding: 12px; }
	.setup-dialog :global(.companion-root .lane-header) { grid-template-columns: 1fr; gap: 10px; padding: 10px 12px; }
	.setup-dialog :global(.companion-root .lane-actions) { justify-self: start; flex-wrap: wrap; gap: 8px; }
	@media (max-width: 920px) {
		.app-header { grid-template-columns: auto 1fr; }
		.signal-path { display: none; }
		.quick-controls { grid-template-columns: repeat(3, 1fr); }
		.quick-controls label:nth-child(3) { border-right: 0; }
		.quick-controls label:nth-child(-n+3) { border-bottom: 1px solid var(--proto-line); }
		.dialog-body { grid-template-columns: 140px 1fr; }
		.preset-grid { grid-template-columns: 1fr; }
	}
	@media (max-width: 820px) {
		.performance-body { grid-template-rows: auto minmax(0, 1fr) 230px; }
	}
	@media (max-width: 680px) {
		.app-header { position: static; grid-template-columns: 1fr; padding: 8px; }
		.header-actions { justify-content: flex-start; flex-wrap: wrap; }
		.quick-controls { grid-template-columns: repeat(2, 1fr); }
		.quick-controls label { border-bottom: 1px solid var(--proto-line); }
		.dialog-body { grid-template-columns: 1fr; }
		.dialog-body > nav { display: grid; grid-template-columns: repeat(3, 1fr); border-right: 0; border-bottom: 1px solid var(--proto-line); }
	}
</style>
