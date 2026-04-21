<script lang="ts">
	import { onMount } from 'svelte';
	import { midi } from '$lib/stores/midi.svelte';
	import PixelSelect from './PixelSelect.svelte';
	import type { Snippet } from 'svelte';

	let { children }: { children?: Snippet } = $props();

	// Virtual input sentinel values — must match src-tauri/src/commands/engine.rs
	const VIRTUAL_COMPUTER_KEYBOARD = 999_998;
	const VIRTUAL_GUITAR_AUDIO = 999_997;

	// Auto-refresh on mount if device lists are empty (defensive — page init should
	// already call midi.refresh(), but this ensures devices show even if that failed).
	onMount(() => {
		if (midi.inputs.length === 0 && midi.outputs.length === 0 && !midi.isLoading) {
			midi.refresh();
		}
	});

	const MAX_OUTPUT_SLOTS = 8;

	// Derived: is Computer Keyboard selected as input?
	let isComputerKeyboard = $derived(midi.selectedInput === VIRTUAL_COMPUTER_KEYBOARD);

	let inputOptions = $derived([
		...midi.inputs.map((d) => ({ value: String(d.index), label: d.name })),
		{ value: String(VIRTUAL_COMPUTER_KEYBOARD), label: 'Computer Keyboard' },
		{ value: String(VIRTUAL_GUITAR_AUDIO), label: 'Guitar Audio' }
	]);

	let outputOptions = $derived(
		midi.outputs.map((d) => ({ value: String(d.index), label: d.name }))
	);

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
		const newOutputs = [...midi.selectedOutputs];

		if (value === '') {
			if (slotIndex < newOutputs.length) {
				newOutputs.splice(slotIndex, 1);
			}
		} else {
			const idx = parseInt(value, 10);
			while (newOutputs.length <= slotIndex) {
				newOutputs.push(-1);
			}
			newOutputs[slotIndex] = idx;
		}

		midi.selectedOutputs = newOutputs.filter((v) => v >= 0);
	}

	function getSlotValue(slotIndex: number): string {
		if (slotIndex < midi.selectedOutputs.length) {
			return String(midi.selectedOutputs[slotIndex]);
		}
		return '';
	}

	function slotLabel(index: number): string {
		if (index === 0) return 'Voice 1 (melody)';
		return `Voice ${index + 1}`;
	}
</script>

<!-- Input Device Section -->
<div class="midi-section pixel-card">
	<div class="section-header font-pixel">INPUT</div>

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
		<div class="keyboard-hint font-pixel">Z-M: C3-B3, Q-U: C4-C5</div>
	{/if}

	{#if midi.error}
		<div class="error-text font-pixel">{midi.error}</div>
	{/if}
</div>

<!-- Slot for Guitar Input panel (rendered between INPUT and OUTPUTS) -->
{@render children?.()}

<!-- Output Device Section -->
<div class="midi-section pixel-card">
	<div class="section-header font-pixel">OUTPUTS</div>

	<div class="output-slots">
		{#each Array.from({ length: MAX_OUTPUT_SLOTS }, (_, i) => i) as slotIdx}
			<div class="output-slot">
				<span class="slot-label font-pixel">{slotLabel(slotIdx)}</span>
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
