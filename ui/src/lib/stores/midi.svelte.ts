/**
 * MIDI Store -- Reactive MIDI Device State (Svelte 5 Runes)
 *
 * Tracks available MIDI input/output devices, selected devices,
 * and connection state. Persists device selections by NAME to
 * localStorage so they survive page reloads.
 */

import { adapter } from '$lib/adapter';
import type { MidiDevice } from '$lib/adapter';

const MIDI_SETTINGS_KEY = 'contrapunk-midi';

interface MidiSettings {
	inputName: string | null;
	outputNames: string[];
}

function loadMidiSettings(): MidiSettings | null {
	try {
		const raw = localStorage.getItem(MIDI_SETTINGS_KEY);
		if (!raw) return null;
		return JSON.parse(raw);
	} catch {
		return null;
	}
}

function saveMidiSettings(settings: MidiSettings) {
	try {
		localStorage.setItem(MIDI_SETTINGS_KEY, JSON.stringify(settings));
	} catch {
		// localStorage unavailable
	}
}

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
	 * After refreshing, restores previously selected devices by name.
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

			// Try to restore saved selections by name
			const saved = loadMidiSettings();
			if (saved) {
				// Restore input by name
				if (saved.inputName) {
					const match = newInputs.find((d) => d.name === saved.inputName);
					if (match) {
						this.selectedInput = match.index;
					} else {
						this.selectedInput = null;
					}
				}

				// Restore outputs by name
				if (saved.outputNames.length > 0) {
					this.selectedOutputs = saved.outputNames
						.map((name) => newOutputs.find((d) => d.name === name))
						.filter((d): d is MidiDevice => d !== undefined)
						.map((d) => d.index);
				}
			} else {
				// No saved settings — clear selections if devices changed
				if (
					this.selectedInput !== null &&
					!newInputs.some((d) => d.index === this.selectedInput)
				) {
					this.selectedInput = null;
				}
				this.selectedOutputs = this.selectedOutputs.filter((idx) =>
					newOutputs.some((d) => d.index === idx)
				);
			}
		} catch (e) {
			this.error = `Failed to refresh MIDI devices: ${e}`;
		} finally {
			this.isLoading = false;
		}
	}

	/** Persist current selections by device name. */
	private persist() {
		saveMidiSettings({
			inputName: this.selectedInputName,
			outputNames: this.selectedOutputNames,
		});
	}

	/**
	 * Select a MIDI input device by index.
	 */
	selectInput(index: number) {
		if (this.inputs.some((d) => d.index === index)) {
			this.selectedInput = index;
			this.persist();
		}
	}

	/**
	 * Clear the input selection.
	 */
	clearInput() {
		this.selectedInput = null;
		this.persist();
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
		this.persist();
	}

	/**
	 * Set outputs to a specific list of indices.
	 */
	setOutputs(indices: number[]) {
		this.selectedOutputs = indices.filter((idx) => this.outputs.some((d) => d.index === idx));
		this.persist();
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
