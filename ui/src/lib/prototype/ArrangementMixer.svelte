<script lang="ts">
	import { onMount } from 'svelte';
	import { engine } from '$lib/stores/engine.svelte';
	import { arrangement } from '$lib/stores/arrangement.svelte';
	import { midi } from '$lib/stores/midi.svelte';
	import { synth } from '$lib/stores/synth.svelte';
	import type { VoiceOutputTarget } from '$lib/adapter';

	let { openSetup }: { openSetup: (section: string, focus?: string) => void } = $props();

	type Role = {
		name: string;
		subtitle: string;
		section: string;
		group: number;
		active: boolean;
		route: string;
		color: string;
	};

	let muted = $state([false, false, false, false]);
	let solo = $state<number | null>(null);

	function routeName(target: VoiceOutputTarget | undefined): string {
		if (!target || target.kind === 'synth') return 'Synth';
		if (target.kind === 'off') return 'Off';
		const deviceIndex = midi.selectedOutputs[target.port];
		return midi.outputs.find((device) => device.index === deviceIndex)?.name ?? `MIDI ${target.port + 1}`;
	}

	let inputRoute = $derived(routeName(midi.voiceOutputs[engine.voicePosition]));
	let harmonyRoute = $derived.by(() => {
		const names = new Set(
			midi.voiceOutputs
				.slice(0, engine.voiceCount)
				.filter((_, index) => index !== engine.voicePosition)
				.map(routeName)
		);
		return names.size === 1 ? [...names][0] : names.size ? 'Mixed routes' : '—';
	});
	let sourceName = $derived.by(() => {
		if (midi.selectedInput === 999_997) return 'Guitar Audio';
		if (midi.selectedInput === 999_998) return 'Computer Keys';
		return midi.inputs.find((device) => device.index === midi.selectedInput)?.name ?? 'Choose source';
	});
	let roles = $derived<Role[]>([
		{
			name: 'Your Voice',
			subtitle: engine.inputNotes.length ? `${engine.inputNotes.length} sounding` : 'Performed line',
			section: 'harmony',
			group: 0,
			active: engine.inputNotes.length > 0,
			route: inputRoute,
			color: '#4fe8c3'
		},
		{
			name: 'Harmonic Support',
			subtitle: `${Math.max(0, engine.voiceCount - 1)} generated ${engine.voiceCount === 2 ? 'voice' : 'voices'}`,
			section: 'harmony',
			group: 1,
			active: engine.harmonyNotes.length > 0,
			route: harmonyRoute,
			color: '#ff2e88'
		},
		{
			name: engine.imitativeForm === 'strict_canon' ? 'Strict Canon' : 'Free Imitation',
			subtitle: engine.canonEnabled ? `${engine.canonVoices.length} active ${engine.canonVoices.length === 1 ? 'voice' : 'voices'}` : 'Disabled',
			section: 'canon',
			group: 2,
			active: engine.canonNotes.length > 0,
			route: 'Companion',
			color: '#ffdd44'
		},
		{
			name: 'Species Counterpoint',
			subtitle: engine.counterpointSpecies.replace('Species', 'Species '),
			section: 'counterpoint',
			group: 3,
			active: engine.counterpointNotes.length > 0,
			route: 'Companion',
			color: '#a3e635'
		}
	]);

	function appliedLevel(group: number): number {
		if (muted[group]) return 0;
		if (solo !== null && solo !== group) return 0;
		return arrangement.mixLevels[group] ?? 1;
	}

	async function pushLevel(group: number) {
		await arrangement.pushMixLevel(group, appliedLevel(group));
	}

	async function pushAll() {
		await Promise.all([0, 1, 2, 3].map(pushLevel));
	}

	async function setLevel(group: number, value: number) {
		arrangement.setMixLevel(group, value);
		await pushLevel(group);
	}

	async function toggleMute(group: number) {
		muted[group] = !muted[group];
		muted = [...muted];
		await pushLevel(group);
	}

	async function toggleSolo(group: number) {
		solo = solo === group ? null : group;
		await pushAll();
	}

	onMount(() => {
		void arrangement.syncFromBackend();
	});
</script>

<section class="arrangement" aria-labelledby="arrangement-title">
	<header>
		<h2 id="arrangement-title"><span aria-hidden="true">⎇</span>Arrangement</h2>
		<span class="header-icon" title="Built-in synth levels" aria-label="Built-in synth levels">◧</span>
	</header>

	<div class="flow">
		<button class="endpoint input" type="button" title="Input setup" aria-label={`Input setup: ${sourceName}`} onclick={() => openSetup('input', 'input-source')}>
			<span class="endpoint-icon" aria-hidden="true">⌁</span>
			<strong>{sourceName}</strong>
		</button>

		<span class="arrow" aria-hidden="true">→</span>

		<div class="role player" style:--role-color={roles[0].color}>
			{@render roleStrip(roles[0])}
		</div>

		<div class="branch" aria-label="Parallel generated parts">
			<div class="branch-rail" aria-hidden="true"></div>
			{#each roles.slice(1) as role (role.group)}
				<div class="role" style:--role-color={role.color}>
					{@render roleStrip(role)}
				</div>
			{/each}
		</div>

		<span class="arrow" aria-hidden="true">→</span>

		<div class="output-strip">
			<button class="strip-title" type="button" title="Output setup" onclick={() => openSetup('output', 'output-routing')}>
				<span class="kicker" aria-hidden="true">◉</span>
				<strong>Master</strong>
			</button>
			<div class="strip-body">
				<div class="meter" class:active={engine.inputNotes.length + engine.harmonyNotes.length + engine.canonNotes.length + engine.counterpointNotes.length > 0 && synth.enabled} aria-hidden="true">
					{#each Array(8) as _}<i></i>{/each}
				</div>
				<label class="fader" title="Built-in synth master level">
					<span aria-hidden="true">◒</span>
					<input aria-label="Master output level" type="range" min="0" max="1" step="0.01" value={synth.masterGain} oninput={(event) => synth.setMasterGain(Number(event.currentTarget.value))} />
					<output>{Math.round(synth.masterGain * 100)}</output>
				</label>
			</div>
			<button class="mix-button" class:on={!synth.enabled} type="button" title="Mute built-in synth" aria-label="Mute built-in synth" aria-pressed={!synth.enabled} onclick={() => synth.setEnabled(!synth.enabled)}>M</button>
		</div>
	</div>
</section>

{#snippet roleStrip(role: Role)}
	<button class="strip-title" type="button" title={`Open ${role.name} setup`} onclick={() => openSetup(role.section, `${role.section}-controls`)}>
		<span class="activity-dot" class:active={role.active}></span>
		<strong>{role.name}</strong>
		<small>{role.subtitle}</small>
	</button>
	<div class="strip-body">
		<div class="meter" class:active={role.active && appliedLevel(role.group) > 0} aria-hidden="true">
			{#each Array(8) as _}<i></i>{/each}
		</div>
		<label class="fader" title={`${role.name} built-in synth level`}>
			<span aria-hidden="true">◒</span>
			<input
				aria-label={`${role.name} built-in synth level`}
				type="range"
				min="0"
				max="1"
				step="0.01"
				value={arrangement.mixLevels[role.group]}
				disabled={!arrangement.mixLoaded}
				oninput={(event) => setLevel(role.group, Number(event.currentTarget.value))}
			/>
			<output>{Math.round((arrangement.mixLevels[role.group] ?? 1) * 100)}</output>
		</label>
	</div>
	<div class="mix-buttons">
		<button class:on={muted[role.group]} type="button" title={`Mute ${role.name}`} aria-label={`Mute ${role.name} in built-in synth`} aria-pressed={muted[role.group]} onclick={() => toggleMute(role.group)}>M</button>
		<button class:on={solo === role.group} type="button" title={`Solo ${role.name}`} aria-label={`Solo ${role.name} in built-in synth`} aria-pressed={solo === role.group} onclick={() => toggleSolo(role.group)}>S</button>
	</div>
	<button class="route" type="button" title={`Route ${role.name}`} onclick={() => openSetup('output', 'output-routing')}>
		<span aria-hidden="true">↗</span>{role.route}
	</button>
{/snippet}

<style>
	.arrangement { display: grid; height: 100%; min-height: 0; grid-template-rows: 40px minmax(0, 1fr); border: 1px solid var(--proto-line); background: var(--proto-panel); }
	header { display: flex; min-height: 0; align-items: center; justify-content: space-between; padding: 6px 10px; border-bottom: 1px solid var(--proto-line); }
	header h2 { display: flex; align-items: center; gap: 7px; margin: 0; font-size: 13px; font-weight: 650; }
	header h2 span, .header-icon { color: var(--proto-muted); font: 12px var(--font-code); }
	.flow { display: grid; min-height: 0; grid-template-columns: minmax(112px, .8fr) 18px minmax(128px, .9fr) minmax(420px, 3fr) 18px minmax(128px, .9fr); align-items: stretch; gap: 0; padding: 6px; overflow: hidden; }
	.arrow { display: grid; place-items: center; color: var(--proto-muted); font: 16px var(--font-code); }
	.endpoint, .role, .output-strip { min-width: 0; border: 1px solid var(--proto-line-strong); background: var(--proto-surface); }
	.endpoint { align-self: center; min-height: 76px; padding: 9px; color: var(--proto-text); text-align: left; }
	.endpoint-icon { color: var(--proto-muted); font: 18px var(--font-code); }
	.endpoint strong { display: block; margin-top: 6px; font-size: 12px; line-height: 1.2; }
	.kicker { color: var(--proto-muted); font: 700 9px var(--font-code); letter-spacing: .14em; }
	.branch { position: relative; display: grid; grid-template-columns: repeat(3, minmax(136px, 1fr)); gap: 6px; padding: 0 7px; }
	.branch-rail { position: absolute; z-index: 0; inset: 50% 0 auto; height: 1px; background: var(--proto-line-strong); }
	.role, .output-strip { position: relative; z-index: 1; display: flex; min-height: 0; flex-direction: column; }
	.strip-title { display: block; width: 100%; min-height: 46px; padding: 6px 8px; border: 0; border-bottom: 1px solid var(--proto-line); background: transparent; color: var(--proto-text); text-align: left; }
	.strip-title:hover, .route:hover, .endpoint:hover { background: var(--proto-hover); }
	.strip-title strong { display: block; min-height: 18px; font-size: 10px; line-height: 1.2; }
	.strip-title small { display: block; overflow: hidden; color: var(--proto-muted); font-size: 9px; text-overflow: ellipsis; white-space: nowrap; }
	.activity-dot { float: right; width: 6px; height: 6px; margin-top: 3px; border: 1px solid var(--proto-muted); border-radius: 50%; }
	.activity-dot.active { border-color: var(--role-color, var(--proto-text)); background: var(--role-color, var(--proto-text)); box-shadow: 0 0 0 2px color-mix(in srgb, var(--role-color, #fff) 25%, transparent); }
	.strip-body { display: grid; min-height: 0; grid-template-columns: 9px 1fr; flex: 1; align-items: center; justify-items: center; gap: 5px; padding: 3px 8px; }
	.meter { display: flex; height: 54px; flex-direction: column-reverse; gap: 2px; }
	.meter i { display: block; width: 5px; height: 5px; background: var(--proto-line-strong); }
	.meter.active i:nth-child(-n+6) { background: var(--role-color, var(--proto-text)); }
	.fader { display: grid; height: 64px; grid-template-rows: 9px 1fr 9px; place-items: center; color: var(--proto-muted); font: 8px var(--font-code); }
	.fader input { width: 48px; margin: 18px -15px; accent-color: var(--proto-text); transform: rotate(-90deg); }
	.fader output { color: var(--proto-text); }
	.mix-buttons { display: grid; grid-template-columns: 1fr 1fr; border-top: 1px solid var(--proto-line); }
	.mix-buttons button, .mix-button { min-height: 21px; border: 0; border-right: 1px solid var(--proto-line); background: transparent; color: var(--proto-muted); font: 700 8px var(--font-code); }
	.mix-buttons button:last-child { border-right: 0; }
	.mix-buttons button.on, .mix-button.on { background: var(--proto-text); color: var(--proto-bg); }
	.mix-button { border-top: 1px solid var(--proto-line); border-right: 0; }
	.route { display: flex; min-height: 21px; align-items: center; justify-content: space-between; gap: 5px; overflow: hidden; padding: 2px 7px; border: 0; border-top: 1px solid var(--proto-line); background: transparent; color: var(--proto-text); font: 8px var(--font-code); text-overflow: ellipsis; white-space: nowrap; }
	.route span { color: var(--proto-muted); font-size: 8px; }
	@media (max-width: 980px) {
		.flow { min-width: 900px; }
	}
</style>
