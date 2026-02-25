/**
 * Platform Adapter — WASM Implementation
 *
 * Implements ContrapunkAdapter by calling wasm-bindgen exported functions
 * directly. Used when running in the browser without Tauri.
 */

import type {
	ContrapunkAdapter,
	EngineState,
	HumanizeState,
	MidiDevice,
	NoteState,
	Preset
} from './types';

/**
 * Dynamically imported WASM module.
 * Resolved during init() to avoid top-level import failures
 * when the wasm-pkg hasn't been built yet.
 */
// eslint-disable-next-line @typescript-eslint/no-explicit-any
let wasmModule: any = null;
// eslint-disable-next-line @typescript-eslint/no-explicit-any
let engine: any = null;

export class WasmAdapter implements ContrapunkAdapter {
	private initialized = false;
	private _isRunning = false;
	private noteUpdateCallback: ((state: NoteState) => void) | null = null;
	private pollingHandle: number | null = null;

	async init(): Promise<void> {
		if (this.initialized) return;

		try {
			// Dynamic import of wasm-pack output
			// The path will be resolved by Vite's WASM plugin
			wasmModule = await import('contrapunk-wasm');
			if (wasmModule.default && typeof wasmModule.default === 'function') {
				await wasmModule.default();
			}
			engine = new wasmModule.Engine();
			this.initialized = true;
		} catch (e) {
			throw new Error(`Failed to initialize WASM: ${e}`);
		}
	}

	private ensureInit(): void {
		if (!this.initialized || !engine) {
			throw new Error('WASM adapter not initialized. Call init() first.');
		}
	}

	async getEngineState(): Promise<EngineState> {
		this.ensureInit();
		try {
			const raw = engine.get_state();
			return {
				key: raw.key ?? 'C',
				mode: raw.mode ?? 'PassThrough',
				modeNumber: raw.mode_number ?? 1,
				scaleMode: raw.scale_mode ?? 'Ionian',
				octaveMode: raw.octave_mode ?? 'None',
				voiceLeadingEnabled: raw.voice_leading_enabled ?? false,
				voiceLeadingStyle: raw.voice_leading_style ?? 'Free',
				interchangeEnabled: raw.interchange_enabled ?? false,
				interchangeRange: raw.borrowing_range ?? 3,
				voicePosition: raw.voice_position ?? 0,
				voiceCount: raw.voice_count ?? 2,
				isRunning: this._isRunning
			};
		} catch (e) {
			throw new Error(`Failed to get engine state: ${e}`);
		}
	}

	async setKey(key: string): Promise<void> {
		this.ensureInit();
		try {
			engine.set_key(key);
		} catch (e) {
			throw new Error(`Failed to set key: ${e}`);
		}
	}

	async setMode(mode: string): Promise<void> {
		this.ensureInit();
		try {
			engine.set_mode(mode);
		} catch (e) {
			throw new Error(`Failed to set mode: ${e}`);
		}
	}

	async setScaleMode(mode: string): Promise<void> {
		this.ensureInit();
		try {
			engine.set_scale_mode(mode);
		} catch (e) {
			throw new Error(`Failed to set scale mode: ${e}`);
		}
	}

	async setOctaveMode(mode: string): Promise<void> {
		this.ensureInit();
		try {
			engine.set_octave_mode(mode);
		} catch (e) {
			throw new Error(`Failed to set octave mode: ${e}`);
		}
	}

	async setVoiceLeading(enabled: boolean, style: string): Promise<void> {
		this.ensureInit();
		try {
			engine.set_voice_leading(enabled, style);
		} catch (e) {
			throw new Error(`Failed to set voice leading: ${e}`);
		}
	}

	async setInterchange(enabled: boolean, range: number): Promise<void> {
		this.ensureInit();
		try {
			engine.set_interchange(enabled, range);
		} catch (e) {
			throw new Error(`Failed to set interchange: ${e}`);
		}
	}

	async setVoicePosition(position: number): Promise<void> {
		this.ensureInit();
		try {
			engine.set_voice_position(position);
		} catch (e) {
			throw new Error(`Failed to set voice position: ${e}`);
		}
	}

	async getHumanizeState(): Promise<HumanizeState> {
		// WASM mode does not yet support humanization
		// Return defaults matching HumanizeConfig::default()
		return {
			enabled: false,
			jitterEnabled: false,
			jitterMinMs: 1,
			jitterMaxMs: 10,
			velocityEnabled: false,
			velocityVariation: 10,
			durationEnabled: false,
			durationVariationMs: 0,
			swingEnabled: false,
			swingAmount: 0.0,
			bpm: 120.0,
			metronomeEnabled: false
		};
	}

	async setHumanizeConfig(_config: Partial<HumanizeState>): Promise<void> {
		// WASM mode does not yet support humanization
		// Silently accept to avoid errors in shared UI code
	}

	async listMidiInputs(): Promise<MidiDevice[]> {
		// Browser MIDI: use Web MIDI API if available
		if (typeof navigator !== 'undefined' && 'requestMIDIAccess' in navigator) {
			try {
				const access = await navigator.requestMIDIAccess();
				const devices: MidiDevice[] = [];
				let index = 0;
				access.inputs.forEach((input) => {
					devices.push({ index: index++, name: input.name ?? `Input ${index}` });
				});
				return devices;
			} catch {
				return [];
			}
		}
		return [];
	}

	async listMidiOutputs(): Promise<MidiDevice[]> {
		if (typeof navigator !== 'undefined' && 'requestMIDIAccess' in navigator) {
			try {
				const access = await navigator.requestMIDIAccess();
				const devices: MidiDevice[] = [];
				let index = 0;
				access.outputs.forEach((output) => {
					devices.push({ index: index++, name: output.name ?? `Output ${index}` });
				});
				return devices;
			} catch {
				return [];
			}
		}
		return [];
	}

	async refreshMidiDevices(): Promise<void> {
		// Web MIDI devices are refreshed on each listMidiInputs/listMidiOutputs call
	}

	async startRouting(_inputIdx: number, _outputIndices: number[]): Promise<void> {
		this.ensureInit();
		// In WASM mode, routing is handled differently:
		// We connect Web MIDI input events to the WASM engine and output
		// the results to Web MIDI outputs. This is a simplified placeholder
		// that will be fully implemented when the MIDI routing bridge is built.
		this._isRunning = true;
	}

	async stopRouting(): Promise<void> {
		this._isRunning = false;
		this.stopNotePolling();
	}

	async getNoteState(): Promise<NoteState> {
		this.ensureInit();
		try {
			const raw = engine.get_note_state();
			return {
				inputNotes: raw?.input_notes ?? [],
				harmonyNotes: raw?.harmony_notes ?? [],
				borrowedNotes: raw?.borrowed_notes ?? [],
				chordName: raw?.chord_name ?? '',
				lastBorrowedFrom: raw?.last_borrowed_from ?? ''
			};
		} catch {
			return {
				inputNotes: [],
				harmonyNotes: [],
				borrowedNotes: [],
				chordName: '',
				lastBorrowedFrom: ''
			};
		}
	}

	onNoteUpdate(callback: (state: NoteState) => void): () => void {
		this.noteUpdateCallback = callback;
		this.startNotePolling();

		return () => {
			this.noteUpdateCallback = null;
			this.stopNotePolling();
		};
	}

	/**
	 * Poll for note state changes using requestAnimationFrame.
	 * WASM doesn't push events, so we pull at frame rate.
	 */
	private startNotePolling(): void {
		if (this.pollingHandle !== null) return;

		const poll = () => {
			if (!this.noteUpdateCallback || !this._isRunning) {
				this.pollingHandle = null;
				return;
			}

			this.getNoteState()
				.then((state) => {
					this.noteUpdateCallback?.(state);
				})
				.catch(() => {
					// Silently ignore polling errors
				});

			this.pollingHandle = requestAnimationFrame(poll);
		};

		this.pollingHandle = requestAnimationFrame(poll);
	}

	private stopNotePolling(): void {
		if (this.pollingHandle !== null) {
			cancelAnimationFrame(this.pollingHandle);
			this.pollingHandle = null;
		}
	}

	async listPresets(): Promise<Preset[]> {
		this.ensureInit();
		try {
			const raw = engine.list_presets();
			return (raw ?? []).map(
				(p: Record<string, unknown>, i: number): Preset => ({
					index: i,
					name: (p.name as string) ?? '',
					persona: (p.persona as string) ?? '',
					genre: (p.genre as string) ?? '',
					isBuiltin: (p.is_builtin as boolean) ?? true
				})
			);
		} catch {
			return [];
		}
	}

	async loadPreset(name: string): Promise<void> {
		this.ensureInit();
		try {
			engine.load_preset(name);
		} catch (e) {
			throw new Error(`Failed to load preset: ${e}`);
		}
	}

	async savePreset(name: string): Promise<void> {
		this.ensureInit();
		try {
			engine.save_preset(name);
		} catch (e) {
			throw new Error(`Failed to save preset: ${e}`);
		}
	}

	async deletePreset(name: string): Promise<void> {
		this.ensureInit();
		try {
			engine.delete_preset(name);
		} catch (e) {
			throw new Error(`Failed to delete preset: ${e}`);
		}
	}
}
