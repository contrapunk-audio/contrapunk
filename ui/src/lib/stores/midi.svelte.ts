/**
 * MIDI Store -- Reactive MIDI Device State (Svelte 5 Runes)
 *
 * Tracks available MIDI input/output devices, selected devices,
 * and connection state. Persists device selections by NAME to
 * localStorage so they survive page reloads.
 */

import { adapter, isVoiceRouteId, platformName } from '$lib/adapter';
import type {
	MidiDevice,
	MidiPermissionState,
	VoiceOutputTarget,
	VoiceRouteId
} from '$lib/adapter';
import { MAX_VOICES } from '$lib/adapter/types';

const MIDI_SETTINGS_KEY = 'contrapunk-midi';
const VOICE_OUTPUTS_KEY = 'contrapunk-voice-outputs';
const VOICE_OUTPUT_OVERRIDE_KEY = 'contrapunk-all-voice-outputs-to-synth';
const MIDI_PERMISSION_KEY = 'contrapunk-midi-permission';

interface MidiSettings {
	inputName: string | null;
	outputNames: string[];
}

type VoiceOutputMap = Partial<Record<VoiceRouteId, VoiceOutputTarget>>;
type StoredVoiceOutputTarget =
	| { kind: 'off' }
	| { kind: 'midi_port'; deviceName: string };
type StoredVoiceOutputs = {
	version: 2;
	routes: Partial<Record<VoiceRouteId, StoredVoiceOutputTarget>>;
};
type LoadedVoiceOutputs = { routes: VoiceOutputMap; migrated: boolean };

function readTarget(
	value: unknown,
	outputs: MidiDevice[],
	selectedOutputs: number[],
	legacyConnectionSlot: boolean
): VoiceOutputTarget | null {
	if (!value || typeof value !== 'object') return null;
	const stored = value as { kind?: unknown; port?: unknown; deviceName?: unknown };
	if (stored.kind === 'synth' || stored.kind === 'off') return { kind: stored.kind };
	if (stored.kind !== 'midi_port') return null;
	if (typeof stored.deviceName === 'string') {
		const device = outputs.find((output) => output.name === stored.deviceName);
		return device ? { kind: 'midi_port', port: device.index } : null;
	}
	if (!Number.isInteger(stored.port)) return null;
	const savedPort = stored.port as number;
	const deviceIndex = legacyConnectionSlot ? selectedOutputs[savedPort] : savedPort;
	return Number.isInteger(deviceIndex) ? { kind: 'midi_port', port: deviceIndex } : null;
}

function browserStorage(): Storage | null {
	return typeof localStorage === 'undefined' ? null : localStorage;
}

function loadVoiceOutputs(
	outputs: MidiDevice[],
	selectedOutputs: number[],
	voicePosition: number
): LoadedVoiceOutputs | null {
	const storage = browserStorage();
	if (!storage) return null;
	try {
		const raw = storage.getItem(VOICE_OUTPUTS_KEY);
		if (!raw) return null;
		const parsed: unknown = JSON.parse(raw);
		const routes: VoiceOutputMap = {};

		// Old builds addressed a mutable connection-array slot. Preserve each
		// SATB destination and seed the performed-input route from its current slot.
		if (Array.isArray(parsed)) {
			parsed.slice(0, MAX_VOICES).forEach((entry, index) => {
				const target = readTarget(entry, outputs, selectedOutputs, true);
				if (!target || target.kind === 'synth') return;
				routes[`harmony:${index}`] = target;
				if (index === voicePosition) routes.input = target;
			});
			return { routes, migrated: true };
		}
		if (!parsed || typeof parsed !== 'object') return null;
		const versioned = parsed as Partial<StoredVoiceOutputs>;
		const currentVersion = versioned.version === 2 && !!versioned.routes;
		const source = currentVersion ? versioned.routes! : parsed;
		for (const [route, value] of Object.entries(source)) {
			const target = readTarget(value, outputs, selectedOutputs, !currentVersion);
			if (isVoiceRouteId(route) && target && target.kind !== 'synth') routes[route] = target;
		}
		return { routes, migrated: !currentVersion };
	} catch (error) {
		console.warn('[contrapunk] Could not load saved voice outputs:', error);
		return null;
	}
}

function saveVoiceOutputs(targets: VoiceOutputMap, outputs: MidiDevice[]) {
	const storage = browserStorage();
	if (!storage) return;
	const routes: StoredVoiceOutputs['routes'] = {};
	for (const [route, target] of Object.entries(targets)) {
		if (!isVoiceRouteId(route) || !target || target.kind === 'synth') continue;
		if (target.kind === 'off') {
			routes[route] = target;
			continue;
		}
		const device = outputs.find((output) => output.index === target.port);
		if (device) routes[route] = { kind: 'midi_port', deviceName: device.name };
	}
	try {
		storage.setItem(
			VOICE_OUTPUTS_KEY,
			JSON.stringify({ version: 2, routes } satisfies StoredVoiceOutputs)
		);
	} catch (error) {
		console.warn('[contrapunk] Could not save voice outputs:', error);
	}
}

function loadAllVoiceOutputsToSynth(): boolean {
	const storage = browserStorage();
	if (!storage) return false;
	try {
		return storage.getItem(VOICE_OUTPUT_OVERRIDE_KEY) === 'true';
	} catch (error) {
		console.warn('[contrapunk] Could not load the global voice-output override:', error);
		return false;
	}
}

function saveAllVoiceOutputsToSynth(enabled: boolean) {
	const storage = browserStorage();
	if (!storage) return;
	try {
		storage.setItem(VOICE_OUTPUT_OVERRIDE_KEY, String(enabled));
	} catch (error) {
		console.warn('[contrapunk] Could not save the global voice-output override:', error);
	}
}

function loadMidiSettings(): MidiSettings | null {
	const storage = browserStorage();
	if (!storage) return null;
	try {
		const raw = storage.getItem(MIDI_SETTINGS_KEY);
		if (!raw) return null;
		return JSON.parse(raw);
	} catch (error) {
		console.warn('[contrapunk] Could not load MIDI settings:', error);
		return null;
	}
}

function saveMidiSettings(settings: MidiSettings) {
	const storage = browserStorage();
	if (!storage) return;
	try {
		storage.setItem(MIDI_SETTINGS_KEY, JSON.stringify(settings));
	} catch (error) {
		console.warn('[contrapunk] Could not save MIDI settings:', error);
	}
}

function loadPermissionState(): MidiPermissionState | null {
	const storage = browserStorage();
	if (!storage) return null;
	try {
		const raw = storage.getItem(MIDI_PERMISSION_KEY);
		if (raw === 'granted' || raw === 'denied' || raw === 'idle' || raw === 'unsupported') {
			return raw;
		}
		return null;
	} catch (error) {
		console.warn('[contrapunk] Could not load MIDI permission state:', error);
		return null;
	}
}

function savePermissionState(state: MidiPermissionState) {
	const storage = browserStorage();
	if (!storage) return;
	try {
		storage.setItem(MIDI_PERMISSION_KEY, state);
	} catch (error) {
		console.warn('[contrapunk] Could not save MIDI permission state:', error);
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

	// -- Stable musical-part output routing --
	// Missing entries mean Synth, so first run produces audio without setup.
	voiceOutputs = $state<VoiceOutputMap>({});
	allVoiceOutputsToSynth = $state(loadAllVoiceOutputsToSynth());

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
	 * and confuse the user. The "Enable MIDI" button in
	 * MidiPermissionCard.svelte provides the gesture and calls
	 * `requestPermission()` instead.
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
		999_996: '__virtual_tone_source__',
	};

	/** Reverse lookup: persistent name → sentinel value. */
	private static readonly VIRTUAL_IDS: Record<string, number> = {
		'__virtual_computer_keyboard__': 999_998,
		'__virtual_guitar_audio__': 999_997,
		'__virtual_tone_source__': 999_996,
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

	/** Ensure a system MIDI output is connected and return its stable device index. */
	ensureOutputPort(deviceIdx: number): number {
		if (!this.selectedOutputs.includes(deviceIdx)) {
			this.selectedOutputs = [...this.selectedOutputs, deviceIdx];
			this.persist();
		}
		return deviceIdx;
	}

	getVoiceOutput(route: VoiceRouteId): VoiceOutputTarget {
		return this.voiceOutputs[route] ?? { kind: 'synth' };
	}

	/** Update one musical part, push it to the backend, and persist it. */
	async setVoiceOutput(route: VoiceRouteId, target: VoiceOutputTarget) {
		const previous = this.voiceOutputs;
		const next: VoiceOutputMap = { ...previous };
		if (target.kind === 'synth') delete next[route];
		else next[route] = target;
		this.voiceOutputs = next;
		saveVoiceOutputs(next, this.outputs);
		try {
			await adapter.setVoiceOutput(route, target);
			this.error = null;
		} catch (error) {
			this.voiceOutputs = previous;
			saveVoiceOutputs(previous, this.outputs);
			this.error = `Failed to set voice output: ${error}`;
			throw error;
		}
	}

	/** Temporarily override every route with the synth while preserving per-part choices. */
	async setAllVoiceOutputsToSynth(enabled: boolean) {
		if (enabled === this.allVoiceOutputsToSynth) return;
		const previous = this.allVoiceOutputsToSynth;
		this.allVoiceOutputsToSynth = enabled;
		saveAllVoiceOutputsToSynth(enabled);
		try {
			await adapter.setAllVoiceOutputsToSynth(enabled);
			this.error = null;
		} catch (error) {
			this.allVoiceOutputsToSynth = previous;
			saveAllVoiceOutputsToSynth(previous);
			this.error = `Failed to change global voice output: ${error}`;
			throw error;
		}
	}

	/** Hydrate stable routes after device-name restoration, migrating old slot-based settings. */
	async hydrateVoiceOutputs(voicePosition: number) {
		if (!adapter.capabilities.perVoicePortRouting) return;
		if (this.outputs.length === 0) await this.refresh();
		const saved = loadVoiceOutputs(this.outputs, this.selectedOutputs, voicePosition);
		if (saved) {
			this.voiceOutputs = saved.routes;
			for (const target of Object.values(saved.routes)) {
				if (target?.kind === 'midi_port' && !this.selectedOutputs.includes(target.port)) {
					this.selectedOutputs = [...this.selectedOutputs, target.port];
				}
			}
			this.persist();
			if (saved.migrated) saveVoiceOutputs(saved.routes, this.outputs);
			await Promise.all(
				Object.entries(saved.routes).flatMap(([route, target]) =>
					target ? [adapter.setVoiceOutput(route as VoiceRouteId, target)] : []
				)
			);
		} else {
			try {
				const current = await adapter.getVoiceOutputs();
				this.voiceOutputs = Object.fromEntries(
					current.map(({ route, target }) => [route, target])
				) as VoiceOutputMap;
			} catch (error) {
				console.warn('[contrapunk] Could not hydrate voice outputs from the adapter:', error);
			}
		}
		await adapter.setAllVoiceOutputsToSynth(this.allVoiceOutputsToSynth);
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
