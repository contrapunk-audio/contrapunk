<script lang="ts">
	import { onMount } from 'svelte';
	import { midi } from '$lib/stores/midi.svelte';
	import { engine } from '$lib/stores/engine.svelte';
	import PixelSelect from './PixelSelect.svelte';
	import MidiPermissionCard from './MidiPermissionCard.svelte';
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

	// Voice-count picker lives next to the OUTPUTS header (was in
	// ControlPanel; consolidated here so the count is visible alongside
	// the per-voice routing it controls).
	const voiceCountOptions = [1, 2, 3, 4].map((c) => ({
		value: String(c),
		label: c === 1 ? '1 voice' : `${c} voices`
	}));

	function onVoiceCountChange(value: string) {
		const n = parseInt(value, 10);
		if (Number.isFinite(n)) {
			engine.setVoiceCount(n);
		}
	}

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
			midi.setVoiceOutput(slotIndex, { kind: 'synth' });
			return;
		}
		if (value === OFF_VALUE) {
			midi.setVoiceOutput(slotIndex, { kind: 'off' });
			return;
		}
		// MIDI device picked. Make sure the device is in the router's
		// port pool (selectedOutputs) and route this voice directly to
		// that port via MidiPort. No more positional / use_default
		// indirection — voice_outputs is the only routing source now.
		const deviceIdx = parseInt(value, 10);
		if (Number.isNaN(deviceIdx)) return;
		const port = midi.ensureOutputPort(deviceIdx);
		midi.setVoiceOutput(slotIndex, { kind: 'midi_port', port });
	}

	function getSlotValue(slotIndex: number): string {
		const t = midi.voiceOutputs[slotIndex];
		if (!t || t.kind === 'synth') return SYNTH_VALUE;
		if (t.kind === 'off') return OFF_VALUE;
		// MidiPort → resolve back to the absolute device index so the
		// dropdown highlights the correct device.
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

	// Whether a voice slot is actually producing sound (vs `off`). Drives
	// the status dot + dimming on the slot row.
	function slotIsActive(index: number): boolean {
		const t = midi.voiceOutputs[index];
		return !!t && t.kind !== 'off';
	}

	// "You play" — voice the user's input occupies in the harmony. Lives
	// next to INPUT because it answers "where in the chord is what I'm
	// playing", which is a property of the input source, not a harmony
	// setting. Hidden when voiceCount === 1 (only choice = melody).
	function voiceLabel(index: number, count: number): string {
		const names = ['Soprano', 'Alto', 'Tenor', 'Bass'];
		if (count <= 4) return `${names[index] || 'Voice'} (${index + 1})`;
		return `Voice (${index + 1})`;
	}
	const voicePositionOptions = $derived(
		Array.from({ length: engine.voiceCount }, (_, i) => ({
			value: String(i),
			label: voiceLabel(i, engine.voiceCount)
		}))
	);
	function onVoicePositionChange(val: string) {
		engine.setVoicePosition(parseInt(val, 10));
	}
</script>

<!-- Shared permission card. The component gates itself on
     platformName === 'browser' AND midi.permissionState !== 'granted'. -->
<MidiPermissionCard />

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

	{#if engine.voiceCount > 1}
		<!-- "You play" — which voice the input note becomes in the harmony.
		     Lives next to the INPUT picker because it's a property of the
		     input, not a harmony setting. -->
		<div class="you-play-row">
			<span class="you-play-label font-ui">You play</span>
			<PixelSelect
				options={voicePositionOptions}
				value={String(engine.voicePosition)}
				placeholder="Voice"
				small={true}
				onchange={onVoicePositionChange}
			/>
		</div>
	{/if}
</div>

<!-- Slot for Guitar Input panel (rendered between INPUT and OUTPUTS) -->
{@render children?.()}

<!-- Output Device Section -->
<div class="midi-section pixel-card">
	<div class="output-header-row">
		<span class="section-header font-ui">OUTPUTS</span>
		<div class="voice-count-control" title="Number of voices the engine generates">
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

<style>
	.midi-section {
		padding: 4px 6px;
		margin-bottom: 2px;
	}

	.section-header {
		color: var(--color-accent-gold);
		font-size: var(--font-size-xs);
		margin-bottom: 4px;
		-webkit-font-smoothing: none;
		text-rendering: optimizeSpeed;
	}

	/* OUTPUTS header gets the voice-count picker inline next to the
	   label so the count is visible alongside the per-voice slots. */
	.output-header-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 8px;
		margin-bottom: 4px;
	}
	.output-header-row .section-header {
		margin-bottom: 0;
	}
	.voice-count-control {
		display: flex;
		align-items: center;
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

	/* Active slots get a glowing accent dot; off slots get a hollow ring
	   and the whole row dims so the user can scan-eye which voices will
	   actually produce sound. */
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
		min-width: 56px;
		-webkit-font-smoothing: none;
		text-rendering: optimizeSpeed;
	}

	/* "You play" sits inline with the INPUT card. Same horizontal layout
	   as a slot row so it visually rhymes with the OUTPUTS slots below. */
	.you-play-row {
		display: flex;
		align-items: center;
		gap: 6px;
		margin-top: 4px;
	}
	.you-play-label {
		color: var(--color-accent-gold);
		font-size: var(--font-size-xs);
		white-space: nowrap;
		-webkit-font-smoothing: none;
		text-rendering: optimizeSpeed;
	}

	.midi-permission-card {
		display: flex;
		flex-direction: column;
		gap: 6px;
	}

	.permission-text {
		color: var(--color-text-secondary);
		font-size: var(--font-size-xs);
		line-height: 1.5;
		margin: 0;
		-webkit-font-smoothing: none;
		text-rendering: optimizeSpeed;
	}

	.permission-text.error {
		color: #ff4466;
	}

	.permission-btn {
		display: inline-block;
		text-align: center;
		padding: 6px 10px;
		font-size: var(--font-size-xs);
		text-decoration: none;
		color: var(--color-accent-cyan);
		cursor: pointer;
	}

	.permission-btn[disabled] {
		opacity: 0.5;
		cursor: not-allowed;
	}
</style>
