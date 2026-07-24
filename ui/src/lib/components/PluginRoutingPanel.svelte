<script lang="ts">
	import { onMount } from 'svelte';
	import { adapter } from '$lib/adapter';
	import type { PluginInputMode, PluginMidiOutputMode } from '$lib/adapter/types';
	import PixelSelect from './PixelSelect.svelte';
	import Knob from './Knob.svelte';
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
	let synthEnabled = $state(true);
	let synthGain = $state(0.25);

	async function refresh() {
		const [nextInputMode, nextOutputMode, nextSynthEnabled, synthState] = await Promise.all([
			adapter.getPluginInputMode(),
			adapter.getPluginMidiOutputMode(),
			adapter.getPluginSynthEnabled(),
			adapter.getSynthState()
		]);
		inputMode = nextInputMode;
		outputMode = nextOutputMode;
		synthEnabled = nextSynthEnabled;
		synthGain = synthState.masterGain;
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

	async function setGain(gain: number) {
		synthGain = gain;
		await adapter.setSynthMasterGain(gain);
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
	<Knob
		label="Sine gain"
		help="Level of the built-in fixed-sine monitor."
		value={synthGain}
		min={0}
		max={1}
		step={0.01}
		defaultValue={0.25}
		size={48}
		format={(value) => `${Math.round(value * 100)}%`}
		onchange={setGain}
	/>
	<p><strong>FL Studio:</strong> set Contrapunk's wrapper <strong>Output port</strong> and the destination synth's wrapper <strong>Input port</strong> to the same number. Turn Internal Monitor off when listening through Serum. If wrapper port forwarding is unavailable, place both plug-ins in Patcher and connect Contrapunk's MIDI output to the synth.</p>
	<p>For Logic guitar input, insert <strong>Contrapunk Guitar</strong> as an Audio FX, choose <strong>Guitar audio</strong>, then select <strong>Contrapunk Guitar MIDI Out</strong> on the Analog Lab instrument track.</p>
</div>
<ToneSourcePanel />

<style>
	.plugin-routing {
		display: grid;
		grid-template-columns: 1fr 1fr auto auto;
		gap: 12px;
		margin-bottom: 14px;
		padding: 14px;
		border: 1px solid var(--proto-line, var(--color-border));
		background: var(--proto-panel, var(--color-bg-panel));
	}
	.plugin-routing > div { display: grid; gap: 6px; }
	.plugin-routing :global(.knob) { align-self: center; }
	span { color: var(--proto-muted, var(--color-text-secondary)); font: 700 8px var(--font-code); letter-spacing: .12em; }
	button { align-self: end; min-height: 32px; border: 1px solid var(--proto-line-strong, var(--color-border)); background: transparent; color: var(--proto-muted, var(--color-text-secondary)); font: 700 9px var(--font-code); }
	button.enabled { color: var(--proto-text, var(--color-text-primary)); }
	p { grid-column: 1 / -1; margin: 0; color: var(--proto-dim, var(--color-text-dim)); font: 9px/1.5 var(--font-code); }
	strong { color: var(--proto-text, var(--color-text-primary)); }
	@media (max-width: 760px) { .plugin-routing { grid-template-columns: 1fr; } p { grid-column: auto; } }
</style>
