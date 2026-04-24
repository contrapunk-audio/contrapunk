/**
 * MIDI Store -- Reactive MIDI Device State (Svelte 5 Runes)
 *
 * Tracks available MIDI input/output devices, selected devices,
 * and connection state. Persists device selections by NAME to
 * localStorage so they survive page reloads.
 */

import { adapter } from '$lib/adapter';
import type { MidiDevice, VoiceOutputTarget } from '$lib/adapter';
import { MAX_VOICES } from '$lib/adapter/types';

const MIDI_SETTINGS_KEY = 'contrapunk-midi';
const VOICE_OUTPUTS_KEY = 'contrapunk-voice-outputs';

interface MidiSettings {
	inputName: string | null;
	outputNames: string[];
}

function loadVoiceOutputs(): VoiceOutputTarget[] | null {
	try {
		const raw = localStorage.getItem(VOICE_OUTPUTS_KEY);
		if (!raw) return null;
		const parsed = JSON.parse(raw);
		if (!Array.isArray(parsed)) return null;
		return parsed.slice(0, MAX_VOICES);
	} catch {
		return null;
	}
}

function saveVoiceOutputs(targets: VoiceOutputTarget[]) {
	try {
		localStorage.setItem(VOICE_OUTPUTS_KEY, JSON.stringify(targets));
	} catch {
		// localStorage unavailable
	}
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

	// -- Per-voice output routing (issue #36 / backed by #57) --
	// Length is always MAX_VOICES. UseDefault preserves legacy behavior.
	voiceOutputs = $state<VoiceOutputTarget[]>(
		Array.from({ length: MAX_VOICES }, () => ({ kind: 'use_default' as const }))
	);

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
				// Restore input by name (check virtual inputs first)
				if (saved.inputName) {
					const virtualId = MidiStore.VIRTUAL_IDS[saved.inputName];
					if (virtualId !== undefined) {
						this.selectedInput = virtualId;
					} else {
						const match = newInputs.find((d) => d.name === saved.inputName);
						if (match) {
							this.selectedInput = match.index;
						} else {
							this.selectedInput = null;
						}
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

	/** Map virtual sentinel values to persistent names. */
	private static readonly VIRTUAL_NAMES: Record<number, string> = {
		999_998: '__virtual_computer_keyboard__',
		999_997: '__virtual_guitar_audio__',
	};

	/** Reverse lookup: persistent name → sentinel value. */
	private static readonly VIRTUAL_IDS: Record<string, number> = {
		'__virtual_computer_keyboard__': 999_998,
		'__virtual_guitar_audio__': 999_997,
	};

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
	 * Select a virtual input (Guitar Audio, Computer Keyboard).
	 */
	selectVirtualInput(index: number) {
		this.selectedInput = index;
		this.persist();
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
	 * Set the output destination for a single voice.
	 * Updates the reactive store, pushes to the backend via the adapter,
	 * and persists to localStorage so the selection survives reloads.
	 */
	async setVoiceOutput(voiceIdx: number, target: VoiceOutputTarget) {
		if (voiceIdx < 0 || voiceIdx >= MAX_VOICES) return;
		const next = this.voiceOutputs.slice();
		next[voiceIdx] = target;
		this.voiceOutputs = next;
		saveVoiceOutputs(next);
		try {
			await adapter.setVoiceOutput(voiceIdx, target);
		} catch (e) {
			this.error = `Failed to set voice output: ${e}`;
		}
	}

	/**
	 * Hydrate voiceOutputs from localStorage and push to the backend.
	 * Falls back to the backend's current state if no saved selection
	 * exists (first run, fresh install). Call once at app init.
	 */
	async hydrateVoiceOutputs() {
		const saved = loadVoiceOutputs();
		if (saved && saved.length > 0) {
			const padded: VoiceOutputTarget[] = Array.from({ length: MAX_VOICES }, (_, i) =>
				saved[i] ?? { kind: 'use_default' }
			);
			this.voiceOutputs = padded;
			try {
				await Promise.all(padded.map((t, i) => adapter.setVoiceOutput(i, t)));
			} catch {
				// Non-fatal — UI still reflects user's prior choice
			}
			return;
		}
		try {
			const current = await adapter.getVoiceOutputs();
			if (Array.isArray(current) && current.length === MAX_VOICES) {
				this.voiceOutputs = current;
			}
		} catch {
			// Adapter may be stubbed (browser / plugin host)
		}
	}

	/**
	 * Check whether a valid input and at least one output are selected.
	 */
	get isReady(): boolean {
		return this.selectedInput !== null && this.selectedOutputs.length > 0;
	}

	/**
	 * Get the name of the selected input device, or null.
	 * Returns virtual names for sentinel values (e.g. "__virtual_guitar_audio__").
	 */
	get selectedInputName(): string | null {
		if (this.selectedInput === null) return null;
		const virtualName = MidiStore.VIRTUAL_NAMES[this.selectedInput];
		if (virtualName) return virtualName;
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
