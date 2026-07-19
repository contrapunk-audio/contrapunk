<script lang="ts">
	import { onMount } from 'svelte';
	import { adapter } from '$lib/adapter';
	import { engine, KEY_DISPLAY } from '$lib/stores/engine.svelte';
	import { midi } from '$lib/stores/midi.svelte';
	import { synth } from '$lib/stores/synth.svelte';
	import { transport } from '$lib/stores/transport.svelte';
	import { ui } from '$lib/stores/ui.svelte';
	import { attachKeyboardInput } from '$lib/keyboard-input';
	import ActiveNotes from '$lib/components/ActiveNotes.svelte';
	import CompanionPanel from '$lib/components/CompanionPanel.svelte';
	import ControlPanel from '$lib/components/ControlPanel.svelte';
	import EnsemblePresetBar from '$lib/components/EnsemblePresetBar.svelte';
	import Fretboard from '$lib/components/Fretboard.svelte';
	import HistoryStrip from '$lib/components/HistoryStrip.svelte';
	import InputPanel from '$lib/components/InputPanel.svelte';
	import OutputPanel from '$lib/components/OutputPanel.svelte';
	import PerformanceView from '$lib/components/PerformanceView.svelte';
	import Piano from '$lib/components/Piano.svelte';
	import PresetManager from '$lib/components/PresetManager.svelte';
	import SettingsModal from '$lib/components/SettingsModal.svelte';
	import TransportBar from '$lib/components/TransportBar.svelte';
	import VoiceGenerationChain from '$lib/components/VoiceGenerationChain.svelte';
	import VoicesPanel from '$lib/components/VoicesPanel.svelte';
	import ExpressionRoll from './ExpressionRoll.svelte';

	type Tab = 'perform' | 'harmony' | 'ensemble' | 'io';
	type PerformanceAnchor = 'piano' | 'fretboard' | 'history';
	type HarmonyDepth = 'simple' | 'advanced';
	type EnsembleView = 'arrangement' | 'groups' | 'voices';

	const TABS: Array<{ id: Tab; label: string; description: string }> = [
		{ id: 'perform', label: 'Perform', description: 'Live sound and conversion' },
		{ id: 'harmony', label: 'Harmony', description: 'Rules and musical context' },
		{ id: 'ensemble', label: 'Ensemble', description: 'Canon, counterpoint and voices' },
		{ id: 'io', label: 'I/O', description: 'Sources, routing and sound' }
	];
	const VIRTUAL_COMPUTER_KEYBOARD = 999_998;
	const VIRTUAL_GUITAR_AUDIO = 999_997;

	let activeTab = $state<Tab>('perform');
	let performanceAnchor = $state<PerformanceAnchor>('piano');
	let harmonyDepth = $state<HarmonyDepth>('simple');
	let ensembleView = $state<EnsembleView>('arrangement');
	let initialized = $state(false);
	let initError = $state<string | null>(null);
	let panicSent = $state(false);

	let inputLive = $derived(engine.inputNotes.length > 0);
	let ensembleLive = $derived(
		engine.harmonyNotes.length > 0 || engine.canonNotes.length > 0 || engine.counterpointNotes.length > 0
	);
	let outputLive = $derived(inputLive || ensembleLive);
	let sourceName = $derived.by(() => {
		if (midi.selectedInput === VIRTUAL_GUITAR_AUDIO) return 'Guitar';
		if (midi.selectedInput === VIRTUAL_COMPUTER_KEYBOARD) return 'Computer Keys';
		if (midi.selectedInput === null) return 'No input';
		return midi.inputs.find((device) => device.index === midi.selectedInput)?.name ?? 'MIDI Controller';
	});

	onMount(() => {
		let cancelled = false;
		void (async () => {
			try {
				ui.restoreAppearance();
				await adapter.init();
				await engine.syncFromBackend();
				if (adapter.capabilities.pluginMidiOutputMode) await engine.restoreCompanionSettings();
				else await engine.restoreSettings();
				await midi.hydratePermission();
				await Promise.all([synth.syncFromBackend(), transport.syncFromBackend()]);
				try {
					await transport.setMetronomeEnabled(false);
				} catch {
					/* Surface has no transport. */
				}
				if (midi.selectedInput === null || midi.selectedInput === VIRTUAL_GUITAR_AUDIO) {
					midi.selectVirtualInput(VIRTUAL_COMPUTER_KEYBOARD);
				}
				if (!engine.isRunning && midi.selectedInput !== null) {
					await engine.start(midi.selectedInput, midi.selectedOutputs);
				}
				if (!cancelled) initialized = true;
			} catch (error) {
				if (!cancelled) initError = `Could not initialize Contrapunk: ${error}`;
			}
		})();
		return () => {
			cancelled = true;
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
		await adapter.panicAllNotesOff();
		panicSent = true;
		window.setTimeout(() => (panicSent = false), 700);
	}

	function selectTab(tab: Tab) {
		activeTab = tab;
	}

	function handleTabKey(event: KeyboardEvent, current: Tab) {
		const currentIndex = TABS.findIndex((tab) => tab.id === current);
		let next = currentIndex;
		if (event.key === 'ArrowRight') next = (currentIndex + 1) % TABS.length;
		else if (event.key === 'ArrowLeft') next = (currentIndex - 1 + TABS.length) % TABS.length;
		else if (event.key === 'Home') next = 0;
		else if (event.key === 'End') next = TABS.length - 1;
		else return;
		event.preventDefault();
		activeTab = TABS[next].id;
		requestAnimationFrame(() => document.getElementById(`prototype-tab-${activeTab}`)?.focus());
	}
</script>

<div class="prototype-shell">
	<SettingsModal />
	<header class="app-header">
		<div class="identity">
			<strong>CONTRAPUNK</strong>
			<span>NATIVE PROTOTYPE</span>
		</div>
		<div class="signal-path" aria-label="Current signal path">
			<span class:live={inputLive}><i class="input"></i>{sourceName}</span>
			<b aria-hidden="true">›</b>
			<span class:live={ensembleLive}><i class="ensemble"></i>Ensemble</span>
			<b aria-hidden="true">›</b>
			<span class:live={outputLive}><i class="output"></i>Output</span>
		</div>
		<div class="header-actions">
			{#if adapter.capabilities.transportControl}<TransportBar />{/if}
			<button class="route-button" class:running={engine.isRunning} disabled={!engine.isRunning && midi.selectedInput === null} onclick={toggleRouting}>
				{engine.isRunning ? 'Stop routing' : 'Start routing'}
			</button>
			<button class="quiet-button" class:confirmed={panicSent} onclick={panic}>{panicSent ? 'Cleared' : 'Panic'}</button>
			<button class="quiet-button" onclick={() => ui.openSettings()}>Settings</button>
		</div>
	</header>

	<div class="body">
		<div class="primary-nav" role="tablist" aria-label="Prototype sections">
			{#each TABS as tab}
				<button
					id={`prototype-tab-${tab.id}`}
					role="tab"
					aria-selected={activeTab === tab.id}
					aria-controls={`prototype-panel-${tab.id}`}
					tabindex={activeTab === tab.id ? 0 : -1}
					class:active={activeTab === tab.id}
					onclick={() => selectTab(tab.id)}
					onkeydown={(event) => handleTabKey(event, tab.id)}
				>
					<strong>{tab.label}</strong>
					<span>{tab.description}</span>
				</button>
			{/each}
			<div class="session-summary">
				<span>KEY</span><strong>{KEY_DISPLAY[engine.key]}</strong>
				<span>SCALE</span><strong>{engine.scaleMode}</strong>
				<span>MODE</span><strong>{engine.mode}</strong>
			</div>
		</div>

		<main class="workspace">
			{#if initError}
				<div class="notice error" role="alert">{initError}</div>
			{:else if !initialized}
				<div class="notice">Initializing the real Contrapunk engine…</div>
			{:else if activeTab === 'perform'}
				<div id="prototype-panel-perform" role="tabpanel" aria-labelledby="prototype-tab-perform" class="perform-page">
					<ExpressionRoll />
					<div class="active-strip"><ActiveNotes /></div>
					<div class="section-toolbar">
						<div><p class="eyebrow">INSTRUMENT VIEW</p><h2>Where the sound lands</h2></div>
						<div class="segmented" role="group" aria-label="Instrument visualization">
							<button class:active={performanceAnchor === 'piano'} onclick={() => (performanceAnchor = 'piano')}>Piano</button>
							<button class:active={performanceAnchor === 'fretboard'} onclick={() => (performanceAnchor = 'fretboard')}>Fretboard</button>
							<button class:active={performanceAnchor === 'history'} onclick={() => (performanceAnchor = 'history')}>History</button>
						</div>
					</div>
					<div class="instrument-stage">
						{#if performanceAnchor === 'piano'}<Piano />
						{:else if performanceAnchor === 'fretboard'}<Fretboard />
						{:else}<HistoryStrip />{/if}
					</div>
				</div>
			{:else if activeTab === 'harmony'}
				<div id="prototype-panel-harmony" role="tabpanel" aria-labelledby="prototype-tab-harmony" class="configuration harmony-page">
					<div class="page-heading">
						<div><p class="eyebrow">MUSICAL RULES</p><h1>Harmony</h1><p>Choose the tonal context and how supporting voices move.</p></div>
						<div class="segmented" role="group" aria-label="Harmony control depth">
							<button class:active={harmonyDepth === 'simple'} onclick={() => (harmonyDepth = 'simple')}>Performance</button>
							<button class:active={harmonyDepth === 'advanced'} onclick={() => (harmonyDepth = 'advanced')}>Detailed</button>
						</div>
					</div>
					<div class="harmony-grid">
						<div class="surface-card controls-card">
							{#if harmonyDepth === 'simple'}<PerformanceView />{:else}<ControlPanel />{/if}
						</div>
						<aside class="surface-card presets-card"><PresetManager /></aside>
					</div>
				</div>
			{:else if activeTab === 'ensemble'}
				<div id="prototype-panel-ensemble" role="tabpanel" aria-labelledby="prototype-tab-ensemble" class="configuration ensemble-page">
					<div class="page-heading">
						<div><p class="eyebrow">GENERATED PARTS</p><h1>Ensemble</h1><p>Arrange harmonic support, imitation and independent counterpoint.</p></div>
						<div class="segmented" role="group" aria-label="Ensemble section">
							<button class:active={ensembleView === 'arrangement'} onclick={() => (ensembleView = 'arrangement')}>Arrangement</button>
							<button class:active={ensembleView === 'groups'} onclick={() => (ensembleView = 'groups')}>Counterpoint</button>
							<button class:active={ensembleView === 'voices'} onclick={() => (ensembleView = 'voices')}>Voice Library</button>
						</div>
					</div>
					<EnsemblePresetBar />
					<div class="surface-card ensemble-surface">
						{#if ensembleView === 'arrangement'}
							<VoiceGenerationChain />
							<div class="arrangement-summary">
								<div><span>MELODY</span><strong>Channel 1</strong></div>
								<div><span>HARMONY</span><strong>Channels 2–5</strong></div>
								<div><span>CANON</span><strong>Channel 6</strong></div>
								<div><span>COUNTERPOINT</span><strong>Channel 7</strong></div>
							</div>
						{:else if ensembleView === 'groups'}
							{#if adapter.capabilities.companionLanes}<CompanionPanel />{:else}<div class="notice">Companion lanes are unavailable on this surface.</div>{/if}
						{:else}<VoicesPanel />{/if}
					</div>
				</div>
			{:else}
				<div id="prototype-panel-io" role="tabpanel" aria-labelledby="prototype-tab-io" class="configuration io-page">
					<div class="page-heading">
						<div><p class="eyebrow">SIGNAL CONFIGURATION</p><h1>Input & output</h1><p>Select what you play, where each voice goes, and how it sounds.</p></div>
					</div>
					<div class="io-grid">
						<section class="surface-card io-column"><h2>Input</h2>{#if adapter.capabilities.inputSourcePicker}<InputPanel />{:else}<div class="notice">The host owns input selection.</div>{/if}</section>
						<section class="surface-card io-column"><h2>Output & sound</h2><OutputPanel /></section>
					</div>
				</div>
			{/if}
		</main>
	</div>
</div>

<style>
	.prototype-shell {
		--proto-bg: #0d0d0f;
		--proto-panel: #131316;
		--proto-raised: #18181b;
		--proto-line: #2c2c31;
		--proto-line-strong: #494950;
		--proto-text: #f1f1f2;
		--proto-muted: #929299;
		--proto-dim: #5b5b63;
		--color-bg-deep: var(--proto-bg);
		--color-bg-panel: var(--proto-panel);
		--color-bg-card: var(--proto-panel);
		--color-widget-bg: var(--proto-raised);
		--color-widget-inactive: #242428;
		--color-border: var(--proto-line);
		--color-border-active: var(--proto-line-strong);
		--color-text-primary: var(--proto-text);
		--color-text-secondary: var(--proto-muted);
		--color-text-dim: var(--proto-dim);
		--color-accent-cyan: #c6c6ca;
		--color-accent-cyan-dim: #707078;
		--color-accent-magenta: #e0e0e2;
		--color-accent-magenta-dim: #55555d;
		--color-accent-teal: #8d8d94;
		--color-accent-gold: #d0d0d3;
		--color-accent-amber: #a4a4aa;
		--color-piano-input: #42e8c4;
		--color-piano-harmony: #ec6f9e;
		--color-piano-borrowed: #a98eea;
		--glow-cyan: none;
		--glow-teal: none;
		--glow-magenta: none;
		display: grid;
		grid-template-rows: 52px minmax(0, 1fr);
		height: 100vh;
		width: 100vw;
		overflow: hidden;
		background: var(--proto-bg);
		color: var(--proto-text);
		font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
		-webkit-font-smoothing: antialiased;
		text-rendering: optimizeLegibility;
	}
	.app-header { display: grid; grid-template-columns: auto minmax(220px, 1fr) auto; align-items: center; gap: 18px; padding: 0 14px; border-bottom: 1px solid var(--proto-line); background: #101012; }
	.identity { display: flex; align-items: baseline; gap: 9px; white-space: nowrap; }
	.identity strong { font-size: 11px; letter-spacing: .18em; }
	.identity span { color: var(--proto-muted); font-size: 9px; letter-spacing: .09em; }
	.signal-path { display: flex; align-items: center; justify-content: center; gap: 9px; min-width: 0; color: var(--proto-dim); font-size: 10px; }
	.signal-path span { display: inline-flex; align-items: center; gap: 5px; max-width: 150px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
	.signal-path b { color: var(--proto-dim); font-weight: 400; }
	.signal-path i { width: 6px; height: 6px; background: var(--proto-dim); border-radius: 50%; }
	.signal-path span.live { color: var(--proto-text); }
	.signal-path span.live i.input { background: #42e8c4; }
	.signal-path span.live i.ensemble { background: #ec6f9e; }
	.signal-path span.live i.output { background: #f1c75b; }
	.header-actions { display: flex; align-items: center; justify-content: flex-end; gap: 6px; }
	.header-actions :global(.transport-bar) { filter: grayscale(1); }
	button { min-height: 28px; border: 1px solid var(--proto-line-strong); border-radius: 0; background: var(--proto-raised); color: var(--proto-text); font: 600 10px/1 -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif; cursor: pointer; }
	button:hover { border-color: #7c7c84; }
	button:focus-visible { outline: 2px solid #f1f1f2; outline-offset: 2px; }
	button:disabled { cursor: not-allowed; opacity: .42; }
	.route-button, .quiet-button { padding: 7px 10px; }
	.route-button.running { background: var(--proto-text); color: var(--proto-bg); border-color: var(--proto-text); }
	.quiet-button.confirmed { background: var(--proto-text); color: var(--proto-bg); }
	.body { min-height: 0; display: grid; grid-template-columns: 188px minmax(0, 1fr); }
	.primary-nav { min-height: 0; display: flex; flex-direction: column; padding: 10px 8px; border-right: 1px solid var(--proto-line); background: #101012; }
	.primary-nav > button { display: grid; gap: 4px; min-height: 56px; padding: 9px 10px; border-color: transparent; background: transparent; text-align: left; }
	.primary-nav > button strong { font-size: 12px; font-weight: 650; }
	.primary-nav > button span { overflow: hidden; color: var(--proto-muted); font-size: 9px; font-weight: 450; text-overflow: ellipsis; white-space: nowrap; }
	.primary-nav > button.active { border-color: var(--proto-line-strong); background: var(--proto-raised); }
	.primary-nav > button.active::before { position: absolute; }
	.session-summary { display: grid; grid-template-columns: auto 1fr; gap: 7px 8px; margin-top: auto; padding: 11px 9px; border-top: 1px solid var(--proto-line); }
	.session-summary span { color: var(--proto-muted); font-size: 8px; letter-spacing: .1em; }
	.session-summary strong { overflow: hidden; color: var(--proto-text); font: 500 9px/1.2 ui-monospace, SFMono-Regular, Menlo, monospace; text-align: right; text-overflow: ellipsis; white-space: nowrap; }
	.workspace { min-width: 0; min-height: 0; overflow: auto; padding: 12px; background: var(--proto-bg); }
	.perform-page, .configuration { display: grid; gap: 10px; max-width: 1440px; margin: 0 auto; }
	.notice { padding: 18px; border: 1px solid var(--proto-line); color: var(--proto-muted); background: var(--proto-panel); font-size: 12px; }
	.notice.error { color: var(--proto-text); border-style: dashed; }
	.active-strip { border: 1px solid var(--proto-line); background: var(--proto-panel); }
	.section-toolbar, .page-heading { display: flex; align-items: center; justify-content: space-between; gap: 16px; padding: 7px 2px; }
	.eyebrow { margin: 0 0 3px; color: var(--proto-muted); font-size: 9px; font-weight: 700; letter-spacing: .16em; }
	h1, h2 { margin: 0; color: var(--proto-text); font-weight: 650; }
	h1 { font-size: 21px; letter-spacing: -.02em; }
	h2 { font-size: 14px; }
	.page-heading p:last-child { margin: 5px 0 0; color: var(--proto-muted); font-size: 11px; }
	.segmented { display: flex; gap: 0; }
	.segmented button { padding: 7px 11px; color: var(--proto-muted); border-color: var(--proto-line); background: transparent; }
	.segmented button + button { border-left: 0; }
	.segmented button.active { color: var(--proto-bg); background: var(--proto-text); border-color: var(--proto-text); }
	.instrument-stage, .surface-card { border: 1px solid var(--proto-line); background: var(--proto-panel); }
	.instrument-stage { min-height: 120px; overflow: hidden; }
	.harmony-grid { display: grid; grid-template-columns: minmax(0, 1.5fr) minmax(260px, .7fr); gap: 10px; align-items: start; }
	.controls-card, .presets-card { min-width: 0; padding: 9px; }
	.presets-card { position: sticky; top: 0; }
	.ensemble-surface { min-height: 280px; padding: 9px; overflow: auto; }
	.arrangement-summary { display: grid; grid-template-columns: repeat(4, minmax(0, 1fr)); margin-top: 9px; border: 1px solid var(--proto-line); }
	.arrangement-summary > div { padding: 9px; border-right: 1px solid var(--proto-line); }
	.arrangement-summary > div:last-child { border-right: 0; }
	.arrangement-summary span { display: block; margin-bottom: 4px; color: var(--proto-muted); font-size: 8px; letter-spacing: .1em; }
	.arrangement-summary strong { font: 500 10px ui-monospace, SFMono-Regular, Menlo, monospace; }
	.io-grid { display: grid; grid-template-columns: minmax(320px, .8fr) minmax(420px, 1.2fr); gap: 10px; align-items: start; }
	.io-column { min-width: 0; overflow: hidden; }
	.io-column > h2 { padding: 10px 12px; border-bottom: 1px solid var(--proto-line); }
	.configuration {
		--color-piano-input: #bcbcc1;
		--color-piano-harmony: #c6c6cb;
		--color-piano-borrowed: #aaaab0;
		filter: grayscale(1);
	}
	.prototype-shell :global(.particles-canvas) { display: none !important; }
	.prototype-shell :global(.scale-overlay) { display: none !important; }
	.prototype-shell :global(.pixel-card),
	.prototype-shell :global(.card) { box-shadow: none !important; }
	.prototype-shell :global(.font-ui) { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif !important; font-weight: 500; }
	.prototype-shell :global(.font-code) { font-family: ui-monospace, SFMono-Regular, Menlo, monospace !important; }
	@media (max-width: 1020px) {
		.app-header { grid-template-columns: 1fr auto; }
		.signal-path { display: none; }
		.header-actions :global(.transport-bar) { display: none; }
		.harmony-grid, .io-grid { grid-template-columns: 1fr; }
		.presets-card { position: static; }
	}
	@media (max-width: 760px) {
		.prototype-shell { grid-template-rows: auto minmax(0, 1fr); }
		.app-header { padding: 8px; }
		.identity span, .quiet-button:last-child { display: none; }
		.body { grid-template-columns: 1fr; }
		.primary-nav { display: grid; grid-template-columns: repeat(4, 1fr); padding: 4px; border-right: 0; border-bottom: 1px solid var(--proto-line); }
		.primary-nav > button { min-height: 40px; padding: 6px; text-align: center; }
		.primary-nav > button span, .session-summary { display: none; }
		.workspace { padding: 8px; }
		.page-heading, .section-toolbar { align-items: flex-start; flex-direction: column; }
		.arrangement-summary { grid-template-columns: repeat(2, 1fr); }
	}
</style>
