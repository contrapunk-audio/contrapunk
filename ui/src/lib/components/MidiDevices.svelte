<script lang="ts">
	import { onMount } from 'svelte';
	import { midi } from '$lib/stores/midi.svelte';
	import { engine } from '$lib/stores/engine.svelte';
	import PixelSelect from './PixelSelect.svelte';
	import type { Snippet } from 'svelte';

	let { children }: { children?: Snippet } = $props();

	// Virtual input sentinel values — must match src-tauri/src/commands/engine.rs
	const VIRTUAL_COMPUTER_KEYBOARD = 999_998;
	const VIRTUAL_GUITAR_AUDIO = 999_997;

	// Sentinel strings used as <select> values for the non-MIDI-device
	// slot options. MIDI device options use their numeric device index
	// stringified. We pick values that can never collide with an int.
	const SYNTH_VALUE = '__synth__';
	const OFF_VALUE = '__off__';

	// Auto-refresh on mount if device lists are empty (defensive — page init should
	// already call midi.refresh(), but this ensures devices show even if that failed).
	// Also hydrate per-voice routing from localStorage + adapter.
	onMount(() => {
		if (midi.inputs.length === 0 && midi.outputs.length === 0 && !midi.isLoading) {
			midi.refresh();
		}
		midi.hydrateVoiceOutputs();
	});

	// Slot count is driven by the configured voice count (1..8) so the
	// UI only shows slots that actually produce sound.
	const slotCount = $derived(Math.max(1, Math.min(engine.voiceCount, 8)));

	// Derived: is Computer Keyboard selected as input?
	let isComputerKeyboard = $derived(midi.selectedInput === VIRTUAL_COMPUTER_KEYBOARD);

	let inputOptions = $derived([
		...midi.inputs.map((d) => ({ value: String(d.index), label: d.name })),
		{ value: String(VIRTUAL_COMPUTER_KEYBOARD), label: 'Computer Keyboard' },
		{ value: String(VIRTUAL_GUITAR_AUDIO), label: 'Guitar Audio' }
	]);

	// Per-slot dropdown options: Internal Synth first (sensible default for
	// no-MIDI users), then each available MIDI device, then Off last.
	let outputOptions = $derived([
		{ value: SYNTH_VALUE, label: 'Internal Synth' },
		...midi.outputs.map((d) => ({ value: String(d.index), label: d.name })),
		{ value: OFF_VALUE, label: 'Off' }
	]);

	function handleInputChange(value: string) {
		if (value === '') {
			midi.clearInput();
		} else {
			const idx = parseInt(value, 10);
			if (idx === VIRTUAL_COMPUTER_KEYBOARD || idx === VIRTUAL_GUITAR_AUDIO) {
				midi.selectVirtualInput(idx);
			} else {
				midi.selectInput(idx);
			}
		}
	}

	function handleOutputChange(slotIndex: number, value: string) {
		if (value === SYNTH_VALUE) {
			// Per-voice synth: skip external MIDI for this voice.
			// Keep the positional MIDI device list unchanged so other
			// voices are not disturbed.
			midi.setVoiceOutput(slotIndex, { kind: 'synth' });
			return;
		}
		if (value === OFF_VALUE) {
			midi.setVoiceOutput(slotIndex, { kind: 'off' });
			return;
		}
		// MIDI device picked. Revert this voice to default routing (so
		// the engine's PortBased / ChannelBased logic applies), then
		// slot the device into the positional selectedOutputs list the
		// same way it worked before voice_outputs existed.
		midi.setVoiceOutput(slotIndex, { kind: 'use_default' });
		const deviceIdx = parseInt(value, 10);
		if (Number.isNaN(deviceIdx)) return;
		const newOutputs = [...midi.selectedOutputs];
		while (newOutputs.length <= slotIndex) {
			newOutputs.push(-1);
		}
		newOutputs[slotIndex] = deviceIdx;
		midi.selectedOutputs = newOutputs.filter((v) => v >= 0);
	}

	function getSlotValue(slotIndex: number): string {
		const t = midi.voiceOutputs[slotIndex];
		if (t?.kind === 'synth') return SYNTH_VALUE;
		if (t?.kind === 'off') return OFF_VALUE;
		// UseDefault or MidiPort → reflect the positional MIDI device
		// selection (preserves the pre-#36 slot-to-port mapping).
		if (slotIndex < midi.selectedOutputs.length) {
			return String(midi.selectedOutputs[slotIndex]);
		}
		return SYNTH_VALUE;
	}

	function slotLabel(index: number): string {
		if (index === 0) return 'Voice 1 (melody)';
		return `Voice ${index + 1}`;
	}
</script>

<!-- Input Device Section -->
<div class="midi-section pixel-card">
	<div class="section-header font-ui">INPUT</div>

	<div class="input-row">
		<PixelSelect
			options={inputOptions}
			value={midi.selectedInput !== null ? String(midi.selectedInput) : ''}
			placeholder="Select..."
			onchange={handleInputChange}
		/>

		<button
			class="refresh-btn pixel-btn"
			onclick={() => midi.refresh()}
			disabled={midi.isLoading}
			title="Refresh MIDI devices"
		>
			{midi.isLoading ? '...' : 'R'}
		</button>
	</div>

	{#if isComputerKeyboard}
		<div class="keyboard-hint font-code">Z-M: C3-B3, Q-U: C4-C5</div>
	{/if}

	{#if midi.error}
		<div class="error-text font-ui">{midi.error}</div>
	{/if}
</div>

<!-- Slot for Guitar Input panel (rendered between INPUT and OUTPUTS) -->
{@render children?.()}

<!-- Output Device Section -->
<div class="midi-section pixel-card">
	<div class="section-header font-ui">OUTPUTS</div>

	<div class="output-slots">
		{#each Array.from({ length: slotCount }, (_, i) => i) as slotIdx}
			<div class="output-slot">
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

<style>
	.midi-section {
		padding: 6px;
		margin-bottom: 4px;
	}

	.section-header {
		color: var(--color-accent-gold);
		font-size: var(--font-size-xs);
		margin-bottom: 4px;
		-webkit-font-smoothing: none;
		text-rendering: optimizeSpeed;
	}

	.input-row {
		display: flex;
		gap: 4px;
		align-items: center;
	}

	.refresh-btn {
		padding: 3px 6px !important;
		font-size: var(--font-size-xs) !important;
		min-width: 20px;
		text-align: center;
	}

	.keyboard-hint {
		color: var(--color-accent-cyan);
		font-size: var(--font-size-xs);
		margin-top: 4px;
		padding: 2px 4px;
		background: var(--color-bg-panel);
		border: 1px solid var(--color-border);
		-webkit-font-smoothing: none;
		text-rendering: optimizeSpeed;
	}

	.error-text {
		color: #ff4466;
		font-size: var(--font-size-xs);
		margin-top: 3px;
		-webkit-font-smoothing: none;
		text-rendering: optimizeSpeed;
	}

	.output-slots {
		display: flex;
		flex-direction: column;
		gap: 2px;
	}

	.output-slot {
		display: flex;
		align-items: center;
		gap: 4px;
	}

	.slot-label {
		color: var(--color-text-secondary);
		font-size: var(--font-size-xs);
		white-space: nowrap;
		min-width: 56px;
		-webkit-font-smoothing: none;
		text-rendering: optimizeSpeed;
	}
</style>
