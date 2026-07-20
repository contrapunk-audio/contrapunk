<script lang="ts">
	import { onMount } from 'svelte';
	import { adapter } from '$lib/adapter';
	import type { PluginInputMode, PluginMidiOutputMode } from '$lib/adapter/types';
	import PixelSelect from './PixelSelect.svelte';

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
	let synthEnabled = $state(true);

	async function refresh() {
		[inputMode, outputMode, synthEnabled] = await Promise.all([
			adapter.getPluginInputMode(),
			adapter.getPluginMidiOutputMode(),
			adapter.getPluginSynthEnabled()
		]);
	}

	onMount(() => {
		void refresh();
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

	async function toggleSynth() {
		synthEnabled = !synthEnabled;
		await adapter.setPluginSynthEnabled(synthEnabled);
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
	<button type="button" class:enabled={synthEnabled} onclick={toggleSynth}>
		INTERNAL MONITOR {synthEnabled ? 'ON' : 'OFF'}
	</button>
	<p>To drive Analog Lab V, choose <strong>Guitar audio</strong> for live guitar or <strong>Host MIDI</strong> for a controller, select <strong>Input + Contrapunk</strong>, turn the internal monitor off, and place Analog Lab V after Contrapunk in the DAW chain.</p>
</div>

<style>
	.plugin-routing {
		display: grid;
		grid-template-columns: 1fr 1fr auto;
		gap: 12px;
		margin-bottom: 14px;
		padding: 14px;
		border: 1px solid var(--proto-line, var(--color-border));
		background: var(--proto-panel, var(--color-bg-panel));
	}
	.plugin-routing > div { display: grid; gap: 6px; }
	span { color: var(--proto-muted, var(--color-text-secondary)); font: 700 8px var(--font-code); letter-spacing: .12em; }
	button { align-self: end; min-height: 32px; border: 1px solid var(--proto-line-strong, var(--color-border)); background: transparent; color: var(--proto-muted, var(--color-text-secondary)); font: 700 9px var(--font-code); }
	button.enabled { color: var(--proto-text, var(--color-text-primary)); }
	p { grid-column: 1 / -1; margin: 0; color: var(--proto-dim, var(--color-text-dim)); font: 9px/1.5 var(--font-code); }
	strong { color: var(--proto-text, var(--color-text-primary)); }
	@media (max-width: 760px) { .plugin-routing { grid-template-columns: 1fr; } p { grid-column: auto; } }
</style>
