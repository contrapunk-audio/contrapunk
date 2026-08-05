<script lang="ts">
	import { onMount } from 'svelte';
	import { adapter } from '$lib/adapter';
	import type { PluginInputMode, SlideRole, VoiceOutputTarget, VoiceRouteId } from '$lib/adapter/types';
	import EnsemblePresetBar from '$lib/components/EnsemblePresetBar.svelte';
	import { engine } from '$lib/stores/engine.svelte';
	import { arrangement } from '$lib/stores/arrangement.svelte';
	import { midi } from '$lib/stores/midi.svelte';
	import { phrase } from '$lib/stores/phrase.svelte';
	import { synth } from '$lib/stores/synth.svelte';
	import { slide } from '$lib/stores/slide.svelte';
	import { SLIDE_PRESETS, SLIDE_ROLES } from '$lib/slide/config';

	let { openSetup }: { openSetup: (section: string, focus?: string) => void } = $props();

	const VIRTUAL_TONE_SOURCE = 999_996;
	type Role = {
		name: string;
		shortName: string;
		subtitle: string;
		section: string;
		group: number;
		active: boolean;
		route: string;
		color: string;
		slide: string;
	};

	let pluginInputMode = $state<PluginInputMode>('midi');

	function routeName(target: VoiceOutputTarget | undefined): string {
		if (!adapter.capabilities.perVoicePortRouting) {
			return adapter.capabilities.pluginMidiOutputMode ? 'DAW host' : 'Browser output';
		}
		if (!target || target.kind === 'synth') return 'Synth';
		if (target.kind === 'off') return 'Off';
		return midi.outputs.find((device) => device.index === target.port)?.name ?? 'MIDI unavailable';
	}

	let inputRoute = $derived(routeName(midi.getVoiceOutput('input')));
	function routesName(routes: VoiceRouteId[]): string {
		const names = new Set(routes.map((route) => routeName(midi.getVoiceOutput(route))));
		return names.size === 1 ? [...names][0] : names.size ? 'Mixed' : '—';
	}
	let harmonyRoute = $derived.by(() =>
		routesName(
			engine.mode === 'PassThrough'
				? []
				: Array.from({ length: engine.voiceCount }, (_, slot) => slot)
						.filter((slot) => slot !== engine.voicePosition)
						.map((slot) => `harmony:${slot}` as VoiceRouteId)
		)
	);
	let canonRoute = $derived.by(() => {
		const routes = engine.canonEnabled
			? engine.canonVoices.map((_, index) => `canon:${index}` as VoiceRouteId)
			: [];
		if (arrangement.patterns.lowSupport.enabled) routes.push('pattern_low');
		return routesName(routes);
	});
	let counterpointRoute = $derived.by(() => {
		const routes: VoiceRouteId[] = arrangement.counterpoint.enabled
			? arrangement.counterpoint.phraseAware
				? ['counterpoint:0', 'counterpoint:1']
				: ['counterpoint:0']
			: [];
		if (arrangement.patterns.counterline.enabled) routes.push('pattern_counter');
		return routesName(routes);
	});
	let sourceName = $derived.by(() => {
		if (adapter.capabilities.pluginMidiOutputMode) {
			return pluginInputMode === 'audio' ? 'Guitar Audio' : 'Host MIDI';
		}
		if (midi.selectedInput === 999_997) return 'Guitar Audio';
		if (midi.selectedInput === 999_998) return 'Computer Keys';
		if (midi.selectedInput === VIRTUAL_TONE_SOURCE) return 'Tone';
		return midi.inputs.find((device) => device.index === midi.selectedInput)?.name ?? 'Choose source';
	});
	let sourceType = $derived.by(() => {
		if (adapter.capabilities.pluginMidiOutputMode) return pluginInputMode === 'audio' ? 'MONOPHONIC AUDIO' : 'DAW EVENTS';
		if (midi.selectedInput === VIRTUAL_TONE_SOURCE) return 'DIAGNOSTIC MIDI';
		if (midi.selectedInput === 999_997) return 'MONOPHONIC AUDIO';
		if (midi.selectedInput === 999_998) return 'TYPING KEYBOARD';
		return midi.selectedInput === null ? 'NOT CONNECTED' : 'MIDI INPUT';
	});
	let sourceDetail = $derived.by(() => {
		if (adapter.capabilities.pluginMidiOutputMode) return pluginInputMode === 'audio' ? 'Pitch tracking into MIDI' : 'Notes from the DAW host';
		if (midi.selectedInput === 999_997) return 'Clean single-note pitch tracking';
		if (midi.selectedInput === 999_998) return 'Computer keys become MIDI notes';
		if (midi.selectedInput === VIRTUAL_TONE_SOURCE) return 'Test MIDI through the arrangement';
		if (midi.selectedInput === null) return 'Choose an input in Setup';
		return 'Controller notes enter the arrangement';
	});
	function slideLabel(role: SlideRole): string {
		const index = SLIDE_ROLES.indexOf(role);
		const travel = slide.config.roles[index].travel;
		const mixed = slide.config.voices[index].some(
			(voice) => voice.travel !== null || voice.trigger !== null || voice.curve !== null
		);
		if (mixed) return 'Mixed';
		return travel.kind === 'off'
			? 'Off'
			: travel.kind === 'time'
				? `${Math.round(travel.milliseconds)} ms`
				: `${travel.semitones_per_second} st/s`;
	}

	let roles = $derived<Role[]>([
		{
			name: 'Your Voice',
			shortName: 'You',
			subtitle: engine.inputNotes.length ? `${engine.inputNotes.length} sounding` : 'Performed',
			section: 'harmony',
			group: 0,
			active: engine.inputNotes.length > 0,
			route: inputRoute,
			color: '#4fe8c3',
			slide: slideLabel('input')
		},
		{
			name: 'Harmonic Support',
			shortName: 'Harmony',
			subtitle: engine.mode === 'PassThrough'
				? 'Off'
				: `${Math.max(0, engine.voiceCount - 1)} ${engine.voiceCount === 2 ? 'voice' : 'voices'}`,
			section: 'harmony',
			group: 1,
			active: engine.harmonyNotes.length > 0,
			route: harmonyRoute,
			color: '#ff2e88',
			slide: slideLabel('harmony')
		},
		{
			name: engine.imitativeForm === 'strict_canon' ? 'Strict Canon' : 'Free Imitation',
			shortName: 'Canon',
			subtitle: engine.canonEnabled ? `${engine.canonVoices.length} ${engine.canonVoices.length === 1 ? 'voice' : 'voices'}` : 'Off',
			section: 'canon',
			group: 2,
			active: engine.canonNotes.length > 0,
			route: canonRoute,
			color: '#ffdd44',
			slide: slideLabel('canon')
		},
		{
			name: arrangement.counterpoint.phraseAware ? 'Suspension Pair' : 'Species Counterpoint',
			shortName: arrangement.counterpoint.phraseAware ? 'Suspension' : 'Counterpoint',
			subtitle: arrangement.counterpoint.phraseAware
				? 'Bass + tied voice'
				: engine.counterpointSpecies.replace('Species', 'Sp. '),
			section: 'counterpoint',
			group: 3,
			active: engine.counterpointNotes.length > 0,
			route: counterpointRoute,
			color: '#a3e635',
			slide: slideLabel('counterpoint')
		}
	]);

	async function setLevel(group: number, value: number) {
		await arrangement.setAndPushMixLevel(group, value);
	}

	onMount(() => {
		if (!arrangement.mixLoaded) void arrangement.syncFromBackend();
		if (!adapter.capabilities.pluginMidiOutputMode) return;
		const refreshInputMode = () => {
			void adapter.getPluginInputMode().then((mode) => (pluginInputMode = mode));
		};
		refreshInputMode();
		return adapter.onPluginParamsUpdate(refreshInputMode);
	});
</script>

<section class="arrangement" aria-labelledby="arrangement-title">
	<aside class="source-pane" class:live={engine.inputNotes.length > 0}>
		<div class="source-heading">
			<span>SOURCE</span>
			<button type="button" title="Input setup" onclick={() => openSetup('input', 'input-source')}>CHANGE ↗</button>
		</div>
		<div class="source-identity">
			<span class="source-dot" aria-hidden="true"></span>
			<div><strong>{sourceName}</strong><small>{sourceType}</small></div>
		</div>
		<p class="source-detail">{sourceDetail}</p>
		<button class="source-slide" type="button" title="Configure Your Voice Slide" onclick={() => openSetup('harmony', 'slide-controls')}>
			<span>YOUR VOICE SLIDE</span><strong>{roles[0].slide}</strong>
		</button>
	</aside>

	<div class="arrangement-pane">
		<header>
			<h2 id="arrangement-title"><span aria-hidden="true">⎇</span>Arrangement</h2>
			{#if adapter.capabilities.phraseContext}
				<button class="phrase-status" type="button" title="Configure phrase gap" aria-live="polite" onclick={() => openSetup('canon')}>
					<i class:live={phrase.phase !== 'idle'}></i><span>PHRASE</span><strong>{phrase.statusLabel}</strong>
				</button>
			{/if}
			<EnsemblePresetBar compact />
			<label class="slide-preset">
				<span>SLIDE</span>
				<select
					aria-label="Slide preset"
					value={slide.selectedPreset}
					disabled={!slide.loaded}
					onchange={(event) => {
						if (event.currentTarget.value !== 'custom') slide.applyPreset(event.currentTarget.value);
					}}
				>
					{#if slide.selectedPreset === 'custom'}<option value="custom">Custom</option>{/if}
					{#each SLIDE_PRESETS as preset}<option value={preset.id}>{preset.name}</option>{/each}
				</select>
			</label>
			{#if arrangement.mixError}<span class="mix-error" role="alert" title={arrangement.mixError}>MIX ERROR</span>{/if}
			<button class="arrangement-setup" type="button" title="Open arrangement setup" aria-label="Open arrangement setup" onclick={() => openSetup('harmony')}>⚙</button>
		</header>

		<div class="flow">
			<div class="role player" style:--role-color={roles[0].color}>{@render roleNode(roles[0])}</div>
			<span class="arrow" aria-hidden="true">→</span>
			<div class="branch" aria-label="Parallel generated parts">
				<div class="branch-rail" aria-hidden="true"></div>
				{#each roles.slice(1) as role (role.group)}
					<div class="role" style:--role-color={role.color}>{@render roleNode(role)}</div>
				{/each}
			</div>
			<span class="arrow" aria-hidden="true">→</span>
			<div class="output-strip">
				<button class="node-title" type="button" title="Output setup" onclick={() => openSetup('output', 'output-routing')}>
					<span class="activity-dot" class:active={synth.enabled}></span>
					<strong>Master</strong><small>Built-in sine</small>
				</button>
				<div class="node-controls master-controls">
					<button class:on={!synth.enabled} type="button" title="Mute built-in synth" aria-label="Mute built-in synth" aria-pressed={!synth.enabled} onclick={() => synth.setEnabled(!synth.enabled)}>M</button>
					<label title="Built-in synth master level"><input aria-label="Master output level" type="range" min="0" max="1" step="0.01" value={synth.masterGain} oninput={(event) => synth.setMasterGain(Number(event.currentTarget.value))} /><output>{Math.round(synth.masterGain * 100)}</output></label>
				</div>
				<button class="node-foot output-foot" type="button" onclick={() => openSetup('output', 'output-routing')}>OUTPUT ↗</button>
			</div>
		</div>
	</div>
</section>

{#snippet roleNode(role: Role)}
	<button class="node-title" type="button" title={`Open ${role.name} setup`} onclick={() => openSetup(role.section, `${role.section}-controls`)}>
		<span class="activity-dot" class:active={role.active}></span>
		<strong>{role.shortName}</strong><small>{role.subtitle}</small>
	</button>
	<div class="node-controls">
		<div class="mix-buttons">
			<button class:on={arrangement.muted[role.group]} type="button" title={`Mute ${role.name}`} aria-label={`Mute ${role.name} in built-in synth`} aria-pressed={arrangement.muted[role.group]} disabled={!adapter.capabilities.roleMix} onclick={() => arrangement.toggleMute(role.group)}>M</button>
			<button class:on={arrangement.solo === role.group} type="button" title={`Solo ${role.name}`} aria-label={`Solo ${role.name} in built-in synth`} aria-pressed={arrangement.solo === role.group} disabled={!adapter.capabilities.roleMix} onclick={() => arrangement.toggleSolo(role.group)}>S</button>
		</div>
		<label class="gain" title={`${role.name} built-in synth level`}>
			<input aria-label={`${role.name} built-in synth level`} type="range" min="0" max="1" step="0.01" value={arrangement.mixLevels[role.group]} disabled={!arrangement.mixLoaded || !adapter.capabilities.roleMix} oninput={(event) => setLevel(role.group, Number(event.currentTarget.value))} />
			<output>{Math.round((arrangement.mixLevels[role.group] ?? 1) * 100)}</output>
		</label>
	</div>
	<div class="node-foot">
		<button type="button" title={`Configure ${role.name} Slide`} onclick={() => openSetup('harmony', 'slide-controls')}>↝ {role.slide}</button>
		{#if adapter.capabilities.perVoicePortRouting || adapter.capabilities.pluginMidiOutputMode}
			<button type="button" title={`Route ${role.name}`} onclick={() => openSetup('output', 'output-routing')}>↗ {role.route}</button>
		{:else}
			<span class="route-static" title="Routing is managed by this surface">↗ {role.route}</span>
		{/if}
	</div>
{/snippet}

<style>
	.arrangement { display: grid; height: 100%; min-height: 0; grid-template-columns: minmax(245px, 290px) minmax(0, 1fr); border: 1px solid var(--proto-line); background: var(--proto-panel); }
	.source-pane { display: grid; min-width: 0; grid-template-rows: 22px auto 1fr 28px; gap: 5px; padding: 7px; border-right: 1px solid var(--proto-line-strong); background: linear-gradient(135deg, #0d1413, var(--proto-panel) 58%); }
	.source-heading { display: flex; align-items: center; justify-content: space-between; color: var(--proto-muted); font: 700 8px var(--font-code); letter-spacing: .12em; }
	.source-heading button { min-height: 20px; padding: 0; border: 0; background: transparent; color: var(--proto-muted); font: 700 7px var(--font-code); }
	.source-heading button:hover { color: var(--proto-text); }
	.source-identity { display: grid; min-width: 0; grid-template-columns: 7px 1fr; align-items: center; gap: 7px; }
	.source-dot { width: 6px; height: 6px; border: 1px solid #4fe8c3; border-radius: 50%; }
	.source-pane.live .source-dot { background: #4fe8c3; box-shadow: 0 0 8px #4fe8c3; }
	.source-identity strong, .source-identity small { display: block; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
	.source-identity strong { font-size: 11px; }
	.source-identity small { color: var(--proto-muted); font: 7px var(--font-code); letter-spacing: .08em; }
	.source-detail { align-self: start; margin: 5px 0 0; color: var(--proto-muted); font: 8px/1.4 var(--font-code); }
	.source-slide { display: flex; min-height: 28px; align-items: center; justify-content: space-between; padding: 0 6px; border: 1px solid var(--proto-line); background: transparent; color: var(--proto-muted); font: 7px var(--font-code); }
	.source-slide strong { color: #4fe8c3; }
	.arrangement-pane { display: grid; min-width: 0; grid-template-rows: 36px minmax(0, 1fr); }
	header { display: flex; min-width: 0; align-items: center; gap: 8px; padding: 4px 6px 4px 9px; border-bottom: 1px solid var(--proto-line); }
	header h2 { display: flex; flex: none; align-items: center; gap: 6px; margin: 0; font-size: 11px; font-weight: 650; }
	header h2 span { color: var(--proto-muted); font: 10px var(--font-code); }
	.phrase-status { display: grid; grid-template-columns: 5px auto; grid-template-rows: 1fr 1fr; align-items: center; column-gap: 5px; min-width: 72px; min-height: 27px; padding: 2px 6px; border: 1px solid var(--proto-line); background: transparent; color: var(--proto-muted); text-align: left; }
	.phrase-status:hover { border-color: var(--proto-text); }
	.phrase-status i { grid-row: 1 / 3; width: 4px; height: 4px; border: 1px solid currentColor; border-radius: 50%; }
	.phrase-status i.live { border-color: #4fe8c3; background: #4fe8c3; box-shadow: 0 0 6px #4fe8c3; }
	.phrase-status span { align-self: end; font: 700 6px var(--font-code); letter-spacing: .08em; }
	.phrase-status strong { align-self: start; overflow: hidden; color: var(--proto-text); font: 7px var(--font-code); text-overflow: ellipsis; white-space: nowrap; }
	header :global(.preset-bar.compact) { max-width: 470px; margin-left: auto; }
	header :global(.preset-bar.compact .toolbar) { grid-template-columns: 42px minmax(150px, 1fr) auto; gap: 5px; }
	.slide-preset { display: grid; min-width: 150px; grid-template-columns: auto minmax(92px, 1fr); align-items: center; gap: 5px; color: var(--proto-muted); font: 700 7px var(--font-code); letter-spacing: .1em; }
	.slide-preset select { min-width: 0; height: 27px; border: 1px solid var(--proto-line-strong); background: var(--proto-surface); color: var(--proto-text); font: 9px var(--font-grotesk); }
	.mix-error { flex: none; color: #ff7a91; font: 700 7px var(--font-code); }
	.arrangement-setup { width: 28px; height: 28px; padding: 0; border: 1px solid var(--proto-line-strong); background: transparent; color: var(--proto-muted); }
	.arrangement-setup:hover { color: var(--proto-text); }
	.flow { display: grid; min-width: 0; min-height: 0; grid-template-columns: minmax(104px, .8fr) 14px minmax(330px, 2.5fr) 14px minmax(96px, .72fr); align-items: stretch; padding: 5px; overflow: hidden; }
	.arrow { display: grid; place-items: center; color: var(--proto-muted); font: 12px var(--font-code); }
	.branch { position: relative; display: grid; min-width: 0; grid-template-columns: repeat(3, minmax(100px, 1fr)); gap: 5px; }
	.branch-rail { position: absolute; z-index: 0; inset: 50% 0 auto; height: 1px; background: var(--proto-line-strong); }
	.role, .output-strip { position: relative; z-index: 1; display: grid; min-width: 0; grid-template-rows: 30px minmax(29px, 1fr) 28px; border: 1px solid var(--proto-line-strong); background: var(--proto-surface); }
	.node-title { display: grid; min-width: 0; grid-template-columns: 7px 1fr; grid-template-rows: 1fr 1fr; align-items: center; column-gap: 5px; padding: 3px 6px; border: 0; border-bottom: 1px solid var(--proto-line); background: transparent; color: var(--proto-text); text-align: left; }
	.node-title:hover, .node-foot button:hover, .output-foot:hover { background: var(--proto-hover); }
	.node-title strong, .node-title small { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
	.node-title strong { align-self: end; font-size: 9px; line-height: 1; }
	.node-title small { grid-column: 2; align-self: start; color: var(--proto-muted); font: 7px var(--font-code); }
	.activity-dot { grid-row: 1 / 3; width: 5px; height: 5px; border: 1px solid var(--proto-muted); border-radius: 50%; }
	.activity-dot.active { border-color: var(--role-color, var(--proto-text)); background: var(--role-color, var(--proto-text)); box-shadow: 0 0 6px var(--role-color, var(--proto-text)); }
	.node-controls { display: grid; min-width: 0; grid-template-columns: 38px 1fr; align-items: center; gap: 5px; padding: 3px 5px; }
	.mix-buttons { display: grid; grid-template-columns: 1fr 1fr; border: 1px solid var(--proto-line); }
	.mix-buttons button, .master-controls > button { min-width: 0; height: 28px; padding: 0; border: 0; border-right: 1px solid var(--proto-line); background: transparent; color: var(--proto-muted); font: 700 7px var(--font-code); }
	.mix-buttons button:last-child { border-right: 0; }
	.mix-buttons button.on, .master-controls > button.on { background: var(--proto-text); color: var(--proto-bg); }
	.mix-buttons button:disabled { opacity: .3; cursor: not-allowed; }
	.gain, .master-controls label { display: grid; min-width: 0; grid-template-columns: 1fr 22px; align-items: center; gap: 3px; }
	.gain input, .master-controls input { width: 100%; min-width: 0; accent-color: var(--role-color, var(--proto-text)); }
	.gain output, .master-controls output { color: var(--proto-text); font: 7px var(--font-code); text-align: right; }
	.node-foot { display: grid; min-width: 0; grid-template-columns: 1fr 1fr; border-top: 1px solid var(--proto-line); }
	.node-foot button, .node-foot .route-static, .output-foot { display: grid; min-width: 0; min-height: 28px; place-items: center; overflow: hidden; padding: 0 5px; border: 0; border-right: 1px solid var(--proto-line); background: transparent; color: var(--proto-muted); font: 7px var(--font-code); text-overflow: ellipsis; white-space: nowrap; }
	.node-foot button:last-child, .node-foot .route-static:last-child { border-right: 0; color: var(--proto-text); }
	.master-controls { grid-template-columns: 22px 1fr; }
	.master-controls > button { border: 1px solid var(--proto-line); }
	.output-foot { border-top: 1px solid var(--proto-line); border-right: 0; color: var(--proto-text); }
	@media (max-width: 1080px) {
		.arrangement { grid-template-columns: 225px minmax(0, 1fr); }
		header h2 { display: none; }
		header :global(.preset-bar.compact) { margin-left: 0; }
		.flow { overflow-x: auto; }
		.flow > * { min-width: 0; }
	}
	@media (max-width: 820px) {
		.arrangement { grid-template-columns: 1fr; grid-template-rows: auto minmax(120px, 1fr); overflow: auto; }
		.source-pane { grid-template-columns: minmax(90px, .8fr) minmax(210px, 1.5fr) minmax(110px, .8fr); grid-template-rows: 1fr; align-items: center; border-right: 0; border-bottom: 1px solid var(--proto-line-strong); }
		.source-heading { display: none; }
		.source-slide { min-height: 28px; }
		.arrangement-pane { min-width: 0; }
	}
	@media (max-width: 620px) {
		.flow { grid-template-columns: minmax(82px, .8fr) 8px minmax(255px, 2.5fr) 8px minmax(78px, .72fr); }
		.branch { grid-template-columns: repeat(3, minmax(80px, 1fr)); }
		.source-pane { grid-template-columns: minmax(80px, .8fr) minmax(180px, 1.5fr) minmax(95px, .8fr); }
		.slide-preset { min-width: 115px; }
		header :global(.preset-bar.compact .toolbar) { grid-template-columns: minmax(120px, 1fr) auto; }
		header :global(.preset-bar.compact .preset-label) { display: none; }
	}
</style>
