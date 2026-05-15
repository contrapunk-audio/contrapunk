<script lang="ts">
	import { onMount } from 'svelte';
	import { midi } from '$lib/stores/midi.svelte';
	import { engine } from '$lib/stores/engine.svelte';
	import { adapter } from '$lib/adapter';
	import VoiceGenerationChain from './VoiceGenerationChain.svelte';
	import ChainPanel from './ChainPanel.svelte';
	import PixelSelect from './PixelSelect.svelte';

	const SYNTH_VALUE = '__synth__';
	const OFF_VALUE = '__off__';

	onMount(() => {
		midi.hydrateVoiceOutputs();
		if (midi.outputs.length === 0 && !midi.isLoading) {
			midi.refresh();
		}
	});

	const slotCount = $derived(Math.max(1, Math.min(engine.voiceCount, 8)));

	const voiceCountOptions = [1, 2, 3, 4].map((c) => ({
		value: String(c),
		label: c === 1 ? '1 voice' : `${c} voices`
	}));

	function onVoiceCountChange(value: string) {
		const n = parseInt(value, 10);
		if (Number.isFinite(n)) engine.setVoiceCount(n);
	}

	let outputOptions = $derived([
		{ value: SYNTH_VALUE, label: 'Internal Synth' },
		...midi.outputs.map((d) => ({ value: String(d.index), label: d.name })),
		{ value: OFF_VALUE, label: 'Off' }
	]);

	function handleOutputChange(slotIndex: number, value: string) {
		if (value === SYNTH_VALUE) {
			midi.setVoiceOutput(slotIndex, { kind: 'synth' });
			return;
		}
		if (value === OFF_VALUE) {
			midi.setVoiceOutput(slotIndex, { kind: 'off' });
			return;
		}
		const deviceIdx = parseInt(value, 10);
		if (Number.isNaN(deviceIdx)) return;
		const port = midi.ensureOutputPort(deviceIdx);
		midi.setVoiceOutput(slotIndex, { kind: 'midi_port', port });
	}

	function getSlotValue(slotIndex: number): string {
		const t = midi.voiceOutputs[slotIndex];
		if (!t || t.kind === 'synth') return SYNTH_VALUE;
		if (t.kind === 'off') return OFF_VALUE;
		if (t.kind === 'midi_port') {
			const dev = midi.selectedOutputs[t.port];
			if (typeof dev === 'number') return String(dev);
		}
		return SYNTH_VALUE;
	}

	function slotLabel(index: number): string {
		if (index === 0) return 'Voice 1 (melody)';
		return `Voice ${index + 1}`;
	}

	function slotIsActive(index: number): boolean {
		const t = midi.voiceOutputs[index];
		return !!t && t.kind !== 'off';
	}
</script>

<div class="output-panel">
	<!-- PRIMARY: Voice Generation Chain visualization -->
	<VoiceGenerationChain />

	{#if adapter.capabilities.perVoicePortRouting}
		<!-- Per-voice MIDI port routing — replicates the OUTPUTS section
		     that lived in MidiDevices.svelte (Play tab). Hidden in plugin
		     mode where the DAW owns routing. -->
		<div class="routing-section pixel-card">
			<div class="output-header-row">
				<span class="section-header font-ui">Per-voice routing</span>
				<div title="Number of voices the engine generates">
					<PixelSelect
						options={voiceCountOptions}
						value={String(engine.voiceCount)}
						small={true}
						onchange={onVoiceCountChange}
					/>
				</div>
			</div>
			<div class="output-slots">
				{#each Array.from({ length: slotCount }, (_, i) => i) as slotIdx}
					<div class="output-slot" class:slot-off={!slotIsActive(slotIdx)}>
						<span
							class="slot-status"
							aria-hidden="true"
							title={slotIsActive(slotIdx) ? 'Will produce audio' : 'Silent'}
						>
							{slotIsActive(slotIdx) ? '●' : '○'}
						</span>
						<span class="slot-label font-ui">{slotLabel(slotIdx)}</span>
						<PixelSelect
							options={outputOptions}
							value={getSlotValue(slotIdx)}
							placeholder="None"
							small={true}
							onchange={(val) => handleOutputChange(slotIdx, val)}
						/>
					</div>
				{/each}
			</div>
		</div>
	{/if}

	{#if adapter.capabilities.audioFx || adapter.capabilities.chainEditor}
		<!-- Synth + FX racks (absorbed from the legacy Chain tab).
		     ChainPanel handles its own audioFx capability gating
		     internally; we render it here whenever either flag is set. -->
		<ChainPanel />
	{/if}
</div>

<style>
	.output-panel {
		display: flex;
		flex-direction: column;
		gap: 8px;
		padding: 8px 10px 14px;
		height: 100%;
		overflow-y: auto;
	}

	.routing-section {
		padding: 6px 8px;
	}

	.section-header {
		color: var(--color-accent-gold);
		font-size: var(--font-size-xs);
		letter-spacing: 1.5px;
		text-transform: uppercase;
	}

	.output-header-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 8px;
		margin-bottom: 6px;
	}

	.output-slots {
		display: flex;
		flex-direction: column;
		gap: 2px;
	}

	.output-slot {
		display: flex;
		align-items: center;
		gap: 6px;
	}

	.slot-status {
		font-size: var(--font-size-xs);
		line-height: 1;
		width: 10px;
		text-align: center;
		color: var(--color-accent-cyan);
		text-shadow: 0 0 4px var(--color-accent-cyan);
	}
	.output-slot.slot-off .slot-status {
		color: var(--color-text-dim);
		text-shadow: none;
	}
	.output-slot.slot-off {
		opacity: 0.55;
	}
	.output-slot.slot-off .slot-label {
		color: var(--color-text-dim);
	}

	.slot-label {
		color: var(--color-text-secondary);
		font-size: var(--font-size-xs);
		white-space: nowrap;
		min-width: 96px;
	}
</style>
