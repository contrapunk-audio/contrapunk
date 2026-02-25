<script lang="ts">
	import { midi } from '$lib/stores/midi.svelte';
	import PixelSelect from './PixelSelect.svelte';

	// Virtual input sentinel values (matching existing INPUT_NOTE_GENERATOR / INPUT_COMPUTER_KEYBOARD from app.rs)
	const VIRTUAL_NOTE_GENERATOR = Number.MAX_SAFE_INTEGER;
	const VIRTUAL_COMPUTER_KEYBOARD = Number.MAX_SAFE_INTEGER - 1;

	const MAX_OUTPUT_SLOTS = 8;

	// Derived: is Computer Keyboard selected as input?
	let isComputerKeyboard = $derived(midi.selectedInput === VIRTUAL_COMPUTER_KEYBOARD);

	let inputOptions = $derived([
		...midi.inputs.map((d) => ({ value: String(d.index), label: d.name })),
		{ value: String(VIRTUAL_NOTE_GENERATOR), label: 'Note Generator' },
		{ value: String(VIRTUAL_COMPUTER_KEYBOARD), label: 'Computer Keyboard' }
	]);

	let outputOptions = $derived(
		midi.outputs.map((d) => ({ value: String(d.index), label: d.name }))
	);

	function handleInputChange(value: string) {
		if (value === '') {
			midi.clearInput();
		} else {
			const idx = parseInt(value, 10);
			if (idx === VIRTUAL_NOTE_GENERATOR || idx === VIRTUAL_COMPUTER_KEYBOARD) {
				midi.selectedInput = idx;
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
		font-size: 7px !important;
		min-width: 20px;
		text-align: center;
	}

	.keyboard-hint {
		color: var(--color-accent-cyan);
		font-size: 6px;
		margin-top: 4px;
		padding: 2px 4px;
		background: var(--color-bg-panel);
		border: 1px solid var(--color-border);
		-webkit-font-smoothing: none;
		text-rendering: optimizeSpeed;
	}

	.error-text {
		color: #ff4466;
		font-size: 6px;
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
		font-size: 6px;
		white-space: nowrap;
		min-width: 56px;
		-webkit-font-smoothing: none;
		text-rendering: optimizeSpeed;
	}
</style>
