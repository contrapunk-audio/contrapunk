/**
 * MIDI Store -- Reactive MIDI Device State (Svelte 5 Runes)
 *
 * Tracks available MIDI input/output devices, selected devices,
 * and connection state. All device operations delegate to the
 * platform adapter (Tauri IPC or WASM Web MIDI).
 */

import { adapter } from '$lib/adapter';
import type { MidiDevice } from '$lib/adapter';

// === MIDI Store (Svelte 5 runes) ===

class MidiStore {
	// -- Available devices --
	inputs = $state<MidiDevice[]>([]);
	outputs = $state<MidiDevice[]>([]);

	// -- Selection state --
	selectedInput = $state<number | null>(null);
	selectedOutputs = $state<number[]>([]);

	// -- Loading / error state --
	isLoading = $state(false);
	error = $state<string | null>(null);

	/**
	 * Refresh the list of available MIDI devices from the backend.
	 * Resets error state on success.
	 */
	async refresh() {
		this.isLoading = true;
		this.error = null;

		try {
			await adapter.refreshMidiDevices();

			const [newInputs, newOutputs] = await Promise.all([
				adapter.listMidiInputs(),
				adapter.listMidiOutputs()
			]);

			this.inputs = newInputs;
			this.outputs = newOutputs;

			// Clear selections if the previously selected device no longer exists
			if (
				this.selectedInput !== null &&
				!newInputs.some((d) => d.index === this.selectedInput)
			) {
				this.selectedInput = null;
			}

			this.selectedOutputs = this.selectedOutputs.filter((idx) =>
				newOutputs.some((d) => d.index === idx)
			);
		} catch (e) {
			this.error = `Failed to refresh MIDI devices: ${e}`;
		} finally {
			this.isLoading = false;
		}
	}

	/**
	 * Select a MIDI input device by index.
	 */
	selectInput(index: number) {
		if (this.inputs.some((d) => d.index === index)) {
			this.selectedInput = index;
		}
	}

	/**
	 * Clear the input selection.
	 */
	clearInput() {
		this.selectedInput = null;
	}

	/**
	 * Toggle a MIDI output device selection.
	 * If already selected, deselects it. Otherwise, adds it.
	 */
	toggleOutput(index: number) {
		if (!this.outputs.some((d) => d.index === index)) return;

		const idx = this.selectedOutputs.indexOf(index);
		if (idx >= 0) {
			this.selectedOutputs = this.selectedOutputs.filter((i) => i !== index);
		} else {
			this.selectedOutputs = [...this.selectedOutputs, index];
		}
	}

	/**
	 * Set outputs to a specific list of indices.
	 */
	setOutputs(indices: number[]) {
		this.selectedOutputs = indices.filter((idx) => this.outputs.some((d) => d.index === idx));
	}

	/**
	 * Check whether a valid input and at least one output are selected.
	 */
	get isReady(): boolean {
		return this.selectedInput !== null && this.selectedOutputs.length > 0;
	}

	/**
	 * Get the name of the selected input device, or null.
	 */
	get selectedInputName(): string | null {
		if (this.selectedInput === null) return null;
		return this.inputs.find((d) => d.index === this.selectedInput)?.name ?? null;
	}

	/**
	 * Get the names of the selected output devices.
	 */
	get selectedOutputNames(): string[] {
		return this.selectedOutputs
			.map((idx) => this.outputs.find((d) => d.index === idx)?.name)
			.filter((name): name is string => name !== undefined);
	}
}

export const midi = new MidiStore();
