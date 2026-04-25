/**
 * MIDI Store -- Reactive MIDI Device State (Svelte 5 Runes)
 *
 * Tracks available MIDI input/output devices, selected devices,
 * and connection state. Persists device selections by NAME to
 * localStorage so they survive page reloads.
 */

import { adapter, platformName } from '$lib/adapter';
import type { MidiDevice, MidiPermissionState, VoiceOutputTarget } from '$lib/adapter';
import { MAX_VOICES } from '$lib/adapter/types';

const MIDI_SETTINGS_KEY = 'contrapunk-midi';
const VOICE_OUTPUTS_KEY = 'contrapunk-voice-outputs';
const MIDI_PERMISSION_KEY = 'contrapunk-midi-permission';

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
		// Validate each entry against the current schema. Old installs
		// may have `use_default` kinds left over — those got removed
		// when routing collapsed to Synth | MidiPort | Off. Drop any
		// invalid entry so the loader's `?? { kind: 'synth' }` fallback
		// fills it in.
		return parsed.slice(0, MAX_VOICES).map((entry: unknown) => {
			if (!entry || typeof entry !== 'object') return { kind: 'synth' as const };
			const kind = (entry as { kind?: unknown }).kind;
			if (kind === 'synth' || kind === 'off') return entry as VoiceOutputTarget;
			if (kind === 'midi_port') {
				const port = (entry as { port?: unknown }).port;
				if (typeof port === 'number') {
					return { kind: 'midi_port', port } as VoiceOutputTarget;
				}
			}
			return { kind: 'synth' as const };
		});
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

function loadPermissionState(): MidiPermissionState | null {
	try {
		const raw = localStorage.getItem(MIDI_PERMISSION_KEY);
		if (raw === 'granted' || raw === 'denied' || raw === 'idle' || raw === 'unsupported') {
			return raw;
		}
		return null;
	} catch {
		return null;
	}
}

function savePermissionState(state: MidiPermissionState) {
	try {
		localStorage.setItem(MIDI_PERMISSION_KEY, state);
	} catch {
		// localStorage unavailable
	}
}

// === MIDI Store (Svelte 5 runes) ===

/** Initial permission state for store construction. Native paths
 *  (Tauri / Plugin) report `'granted'` from the adapter and don't need a
 *  user gesture; the browser path hydrates from localStorage so a returning
 *  user with prior consent skips the prompt UI. */
function initialPermissionState(): MidiPermissionState {
	if (platformName !== 'browser') return 'granted';
	return loadPermissionState() ?? 'idle';
}

class MidiStore {
	// -- Available devices --
	inputs = $state<MidiDevice[]>([]);
	outputs = $state<MidiDevice[]>([]);

	// -- Selection state --
	selectedInput = $state<number | null>(null);
	selectedOutputs = $state<number[]>([]);

	/** Web MIDI permission state. Native paths report `'granted'`; the
	 *  browser path drives the "Enable MIDI" UI. */
	permissionState = $state<MidiPermissionState>(initialPermissionState());

	// -- Per-voice output routing (issue #36 / backed by #57) --
	// Length is always MAX_VOICES. Default is `synth` — first run plays
	// audio without the user needing to add a MIDI port first.
	voiceOutputs = $state<VoiceOutputTarget[]>(
		Array.from({ length: MAX_VOICES }, () => ({ kind: 'synth' as const }))
	);

	// -- Loading / error state --
	isLoading = $state(false);
	error = $state<string | null>(null);

	/**
	 * Trigger the Web MIDI permission prompt. Browser-only; safe to call
	 * on Tauri / Plugin (resolves immediately to `'granted'`).
	 *
	 * Must be invoked from inside a user gesture (click, keydown). On grant,
	 * automatically populates the device list. Persists the resulting state
	 * to localStorage so subsequent visits skip the prompt UI.
	 */
	async requestPermission(): Promise<MidiPermissionState> {
		const next = await adapter.requestMidiPermission();
		this.permissionState = next;
		if (platformName === 'browser') savePermissionState(next);
		if (next === 'granted') {
			await this.refresh();
		}
		return next;
	}

	/**
	 * Re-acquire MIDI access on app startup if the user previously granted it.
	 *
	 * Browser permission persists across reloads, so calling
	 * `navigator.requestMIDIAccess()` again resolves silently without a
	 * prompt — but we still need to actually call it so the WasmAdapter
	 * caches a fresh `MIDIAccess` instance for this page lifetime.
	 *
	 * If the user revoked permission via browser settings since the last
	 * grant, falls back to `'denied'` and the UI re-renders the button.
	 */
	async hydratePermission(): Promise<void> {
		if (this.permissionState !== 'granted') return;
		const next = await adapter.requestMidiPermission();
		this.permissionState = next;
		if (platformName === 'browser') savePermissionState(next);
		if (next === 'granted') {
			await this.refresh();
		}
	}

	/**
	 * Refresh the list of available MIDI devices from the backend.
	 * After refreshing, restores previously selected devices by name.
	 *
	 * No-op when permission has not been granted on the browser path —
	 * `navigator.requestMIDIAccess()` requires a user gesture, so a refresh
	 * triggered at page load would silently leave the device list empty
	 * and confuse the user. The "Enable MIDI" button in MidiDevices.svelte
	 * provides the gesture and calls `requestPermission()` instead.
	 */
	async refresh() {
		if (this.permissionState !== 'granted') return;
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
				saved[i] ?? { kind: 'synth' }
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
