<script lang="ts">
	import { midi } from '$lib/stores/midi.svelte';

	// Virtual input sentinel values (matching existing INPUT_NOTE_GENERATOR / INPUT_COMPUTER_KEYBOARD from app.rs)
	const VIRTUAL_NOTE_GENERATOR = Number.MAX_SAFE_INTEGER;
	const VIRTUAL_COMPUTER_KEYBOARD = Number.MAX_SAFE_INTEGER - 1;

	const MAX_OUTPUT_SLOTS = 8;

	// Derived: is Computer Keyboard selected as input?
	let isComputerKeyboard = $derived(midi.selectedInput === VIRTUAL_COMPUTER_KEYBOARD);

	function handleInputChange(event: Event) {
		const value = (event.target as HTMLSelectElement).value;
		if (value === '') {
			midi.clearInput();
		} else {
			const idx = parseInt(value, 10);
			if (idx === VIRTUAL_NOTE_GENERATOR || idx === VIRTUAL_COMPUTER_KEYBOARD) {
				// Virtual inputs: set directly (they bypass physical MIDI)
				midi.selectedInput = idx;
			} else {
				midi.selectInput(idx);
			}
		}
	}

	function handleOutputChange(slotIndex: number, event: Event) {
		const value = (event.target as HTMLSelectElement).value;
		const newOutputs = [...midi.selectedOutputs];

		if (value === '') {
			// Set slot to None -- remove this slot's value
			if (slotIndex < newOutputs.length) {
				newOutputs.splice(slotIndex, 1);
			}
		} else {
			const idx = parseInt(value, 10);
			// Ensure the array is long enough
			while (newOutputs.length <= slotIndex) {
				newOutputs.push(-1);
			}
			newOutputs[slotIndex] = idx;
		}

		// Filter out invalid entries
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
		<select
			class="midi-select font-pixel"
			value={midi.selectedInput !== null ? String(midi.selectedInput) : ''}
			onchange={handleInputChange}
		>
			<option value="">Select...</option>

			<!-- Physical MIDI inputs -->
			{#each midi.inputs as device}
				<option value={String(device.index)}>{device.name}</option>
			{/each}

			<!-- Virtual inputs -->
			<option value={String(VIRTUAL_NOTE_GENERATOR)}>Note Generator</option>
			<option value={String(VIRTUAL_COMPUTER_KEYBOARD)}>Computer Keyboard</option>
		</select>

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
				<select
					class="midi-select midi-select-sm font-pixel"
					value={getSlotValue(slotIdx)}
					onchange={(e) => handleOutputChange(slotIdx, e)}
				>
					<option value="">None</option>
					{#each midi.outputs as device}
						<option value={String(device.index)}>{device.name}</option>
					{/each}
				</select>
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

	.midi-select {
		flex: 1;
		background: var(--color-widget-bg);
		border: 1px solid var(--color-border);
		color: var(--color-text-primary);
		font-size: 7px;
		padding: 3px 4px;
		border-radius: 0;
		outline: none;
		-webkit-font-smoothing: none;
		text-rendering: optimizeSpeed;
		cursor: pointer;
		appearance: none;
		-webkit-appearance: none;
	}

	.midi-select:focus {
		border-color: var(--color-accent-cyan);
	}

	.midi-select option {
		background: var(--color-widget-bg);
		color: var(--color-text-primary);
	}

	.midi-select-sm {
		font-size: 6px;
		padding: 2px 3px;
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
