<script lang="ts">
	import { onMount } from 'svelte';
	import { adapter } from '$lib/adapter';
	import type { PluginInputMode, PluginMidiOutputMode } from '$lib/adapter/types';
	import { synth } from '$lib/stores/synth.svelte';
	import PixelSelect from './PixelSelect.svelte';
	import ToneSourcePanel from './ToneSourcePanel.svelte';

	const inputOptions = [
		{ value: 'midi', label: 'Host MIDI' },
		{ value: 'audio', label: 'Guitar audio' }
	];
	const outputOptions = [
		{ value: 'full', label: 'Input + Contrapunk' },
		{ value: 'pass_through', label: 'Input only' }
	];

	let inputMode = $state<PluginInputMode>('midi');
	let outputMode = $state<PluginMidiOutputMode>('full');

	async function refresh() {
		const [nextInputMode, nextOutputMode] = await Promise.all([
			adapter.getPluginInputMode(),
			adapter.getPluginMidiOutputMode()
		]);
		inputMode = nextInputMode;
		outputMode = nextOutputMode;
	}

	onMount(() => {
		void Promise.all([refresh(), synth.syncFromBackend()]);
		return adapter.onPluginParamsUpdate(() => void refresh());
	});

	async function setInput(value: string) {
		inputMode = value === 'audio' ? 'audio' : 'midi';
		await adapter.setPluginInputMode(inputMode);
	}

	async function setOutput(value: string) {
		outputMode = value === 'pass_through' ? 'pass_through' : 'full';
		await adapter.setPluginMidiOutputMode(outputMode);
	}
</script>

<div class="plugin-routing" aria-label="DAW plugin routing">
	<div>
		<span>INPUT</span>
		<PixelSelect options={inputOptions} value={inputMode} small={true} onchange={setInput} />
	</div>
	<div>
		<span>MIDI OUTPUT</span>
		<PixelSelect options={outputOptions} value={outputMode} small={true} onchange={setOutput} />
	</div>
	<div class="monitor-status" aria-label={`Internal monitor ${synth.enabled ? `on at ${Math.round(synth.masterGain * 100)} percent` : 'off'}; controlled in Synth`}>
		<span>INTERNAL MONITOR</span>
		<strong>{synth.enabled ? `On · ${Math.round(synth.masterGain * 100)}%` : 'Off'}</strong>
		<small>Controlled in Synth</small>
	</div>
	<p><strong>FL Studio:</strong> set Contrapunk's wrapper <strong>Output port</strong> and the destination synth's wrapper <strong>Input port</strong> to the same number. Mute Internal Monitor in Synth when listening through Serum. If wrapper port forwarding is unavailable, place both plug-ins in Patcher and connect Contrapunk's MIDI output to the synth.</p>
	<p>For Logic guitar input, insert <strong>Contrapunk Guitar</strong> as an Audio FX, choose <strong>Guitar audio</strong>, then select <strong>Contrapunk Guitar MIDI Out</strong> on the Analog Lab instrument track.</p>
</div>
<ToneSourcePanel />

<style>
	.plugin-routing {
		display: grid;
		grid-template-columns: 1fr 1fr minmax(140px, auto);
		gap: 12px;
		margin-bottom: 14px;
		padding: 14px;
		border: 1px solid var(--proto-line, var(--color-border));
		background: var(--proto-panel, var(--color-bg-panel));
	}
	.plugin-routing > div { display: grid; gap: 6px; }
	span { color: var(--proto-muted, var(--color-text-secondary)); font: 700 8px var(--font-code); letter-spacing: .12em; }
	.monitor-status { align-content: center; padding: 6px 8px; border: 1px solid var(--proto-line-strong, var(--color-border)); }
	.monitor-status strong, .monitor-status small { display: block; }
	.monitor-status strong { color: var(--proto-text, var(--color-text-primary)); font: 10px var(--font-code); }
	.monitor-status small { color: var(--proto-dim, var(--color-text-dim)); font: 8px var(--font-code); }
	p { grid-column: 1 / -1; margin: 0; color: var(--proto-dim, var(--color-text-dim)); font: 9px/1.5 var(--font-code); }
	strong { color: var(--proto-text, var(--color-text-primary)); }
	@media (max-width: 760px) { .plugin-routing { grid-template-columns: 1fr; } p { grid-column: auto; } }
</style>
