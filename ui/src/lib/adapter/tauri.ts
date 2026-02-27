/**
 * Platform Adapter — Tauri IPC Implementation
 *
 * Implements ContrapunkAdapter by calling Tauri commands via invoke()
 * and subscribing to events via listen().
 */

import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type {
	ContrapunkAdapter,
	EngineState,
	HumanizeState,
	MidiDevice,
	NoteState,
	Preset
} from './types';

/**
 * Maps the Tauri backend's snake_case response to our camelCase EngineState.
 */
function mapEngineState(raw: Record<string, unknown>, isRunning: boolean): EngineState {
	return {
		key: raw.key as string,
		mode: raw.mode as string,
		modeNumber: raw.mode_number as number,
		scaleMode: raw.scale_mode as string,
		octaveMode: raw.octave_mode as string,
		voiceLeadingEnabled: raw.voice_leading_enabled as boolean,
		voiceLeadingStyle: raw.voice_leading_style as string,
		interchangeEnabled: raw.interchange_enabled as boolean,
		interchangeRange: raw.borrowing_range as number,
		voicePosition: raw.voice_position as number,
		voiceCount: raw.voice_count as number,
		isRunning
	};
}

/**
 * Maps the Tauri backend's snake_case NoteUpdatePayload to our camelCase NoteState.
 */
function mapNoteState(raw: Record<string, unknown>): NoteState {
	return {
		inputNotes: raw.input_notes as number[],
		harmonyNotes: raw.harmony_notes as number[],
		borrowedNotes: raw.borrowed_notes as number[],
		chordName: raw.chord_name as string,
		lastBorrowedFrom: raw.last_borrowed_from as string
	};
}

export class TauriAdapter implements ContrapunkAdapter {
	private _isRunning = false;

	async init(): Promise<void> {
		// Tauri is ready when this code runs in the webview.
		// Fetch initial state to confirm backend is available.
		await this.getEngineState();
	}

	async getEngineState(): Promise<EngineState> {
		try {
			const raw = (await invoke('get_engine_state')) as Record<string, unknown>;
			return mapEngineState(raw, this._isRunning);
		} catch (e) {
			throw new Error(`Failed to get engine state: ${e}`);
		}
	}

	async setKey(key: string): Promise<void> {
		try {
			await invoke('set_key', { key });
		} catch (e) {
			throw new Error(`Failed to set key: ${e}`);
		}
	}

	async setMode(mode: string): Promise<void> {
		try {
			await invoke('set_mode', { mode });
		} catch (e) {
			throw new Error(`Failed to set mode: ${e}`);
		}
	}

	async setScaleMode(mode: string): Promise<void> {
		try {
			await invoke('set_scale_mode', { mode });
		} catch (e) {
			throw new Error(`Failed to set scale mode: ${e}`);
		}
	}

	async setOctaveMode(mode: string): Promise<void> {
		try {
			await invoke('set_octave_mode', { mode });
		} catch (e) {
			throw new Error(`Failed to set octave mode: ${e}`);
		}
	}

	async setVoiceLeading(enabled: boolean, style: string): Promise<void> {
		try {
			await invoke('set_voice_leading', { enabled, style });
		} catch (e) {
			throw new Error(`Failed to set voice leading: ${e}`);
		}
	}

	async setInterchange(enabled: boolean, range: number): Promise<void> {
		try {
			await invoke('set_interchange', { enabled, range });
		} catch (e) {
			throw new Error(`Failed to set interchange: ${e}`);
		}
	}

	async setVoicePosition(position: number): Promise<void> {
		try {
			await invoke('set_voice_position', { position });
		} catch (e) {
			throw new Error(`Failed to set voice position: ${e}`);
		}
	}

	async setVoiceCount(count: number): Promise<void> {
		try {
			await invoke('set_voice_count', { count });
		} catch (e) {
			throw new Error(`Failed to set voice count: ${e}`);
		}
	}

	async getHumanizeState(): Promise<HumanizeState> {
		try {
			const raw = (await invoke('get_humanize_state')) as Record<string, unknown>;
			return {
				enabled: raw.enabled as boolean,
				jitterEnabled: raw.jitter_enabled as boolean,
				jitterMinMs: raw.jitter_min_ms as number,
				jitterMaxMs: raw.jitter_max_ms as number,
				velocityEnabled: raw.velocity_enabled as boolean,
				velocityVariation: raw.velocity_variation as number,
				durationEnabled: raw.duration_enabled as boolean,
				durationVariationMs: raw.duration_variation_ms as number,
				swingEnabled: raw.swing_enabled as boolean,
				swingAmount: raw.swing_amount as number,
				bpm: raw.bpm as number,
				metronomeEnabled: raw.metronome_enabled as boolean
			};
		} catch (e) {
			throw new Error(`Failed to get humanize state: ${e}`);
		}
	}

	async setHumanizeConfig(config: Partial<HumanizeState>): Promise<void> {
		try {
			// Convert camelCase keys to snake_case for Rust backend
			const snakeConfig: Record<string, unknown> = {};
			if (config.enabled !== undefined) snakeConfig.enabled = config.enabled;
			if (config.jitterEnabled !== undefined) snakeConfig.jitter_enabled = config.jitterEnabled;
			if (config.jitterMinMs !== undefined) snakeConfig.jitter_min_ms = config.jitterMinMs;
			if (config.jitterMaxMs !== undefined) snakeConfig.jitter_max_ms = config.jitterMaxMs;
			if (config.velocityEnabled !== undefined)
				snakeConfig.velocity_enabled = config.velocityEnabled;
			if (config.velocityVariation !== undefined)
				snakeConfig.velocity_variation = config.velocityVariation;
			if (config.durationEnabled !== undefined)
				snakeConfig.duration_enabled = config.durationEnabled;
			if (config.durationVariationMs !== undefined)
				snakeConfig.duration_variation_ms = config.durationVariationMs;
			if (config.swingEnabled !== undefined) snakeConfig.swing_enabled = config.swingEnabled;
			if (config.swingAmount !== undefined) snakeConfig.swing_amount = config.swingAmount;
			if (config.bpm !== undefined) snakeConfig.bpm = config.bpm;
			if (config.metronomeEnabled !== undefined)
				snakeConfig.metronome_enabled = config.metronomeEnabled;
			await invoke('set_humanize_config', { config: snakeConfig });
		} catch (e) {
			throw new Error(`Failed to set humanize config: ${e}`);
		}
	}

	async listMidiInputs(): Promise<MidiDevice[]> {
		try {
			const raw = (await invoke('list_midi_inputs')) as Array<Record<string, unknown>>;
			return raw.map((d) => ({ index: d.index as number, name: d.name as string }));
		} catch (e) {
			throw new Error(`Failed to list MIDI inputs: ${e}`);
		}
	}

	async listMidiOutputs(): Promise<MidiDevice[]> {
		try {
			const raw = (await invoke('list_midi_outputs')) as Array<Record<string, unknown>>;
			return raw.map((d) => ({ index: d.index as number, name: d.name as string }));
		} catch (e) {
			throw new Error(`Failed to list MIDI outputs: ${e}`);
		}
	}

	async refreshMidiDevices(): Promise<void> {
		try {
			await invoke('refresh_midi_devices');
		} catch (e) {
			throw new Error(`Failed to refresh MIDI devices: ${e}`);
		}
	}

	async startRouting(inputIdx: number, outputIndices: number[]): Promise<void> {
		try {
			await invoke('start_routing', { inputIdx, outputIndices });
			this._isRunning = true;
		} catch (e) {
			throw new Error(`Failed to start routing: ${e}`);
		}
	}

	async stopRouting(): Promise<void> {
		try {
			await invoke('stop_routing');
			this._isRunning = false;
		} catch (e) {
			throw new Error(`Failed to stop routing: ${e}`);
		}
	}

	async injectNoteOn(note: number, velocity?: number): Promise<number[]> {
		try {
			return (await invoke('inject_note_on', { note, velocity: velocity ?? 100 })) as number[];
		} catch {
			return [note];
		}
	}

	async injectNoteOff(note: number): Promise<number[]> {
		try {
			return (await invoke('inject_note_off', { note })) as number[];
		} catch {
			return [note];
		}
	}

	async getNoteState(): Promise<NoteState> {
		try {
			const raw = (await invoke('get_note_state')) as Record<string, unknown>;
			return mapNoteState(raw);
		} catch (e) {
			throw new Error(`Failed to get note state: ${e}`);
		}
	}

	onNoteUpdate(callback: (state: NoteState) => void): () => void {
		let unlisten: UnlistenFn | undefined;
		let cancelled = false;

		listen<Record<string, unknown>>('note-update', (event) => {
			if (!cancelled) {
				callback(mapNoteState(event.payload));
			}
		}).then((fn) => {
			if (cancelled) {
				// If already cancelled before listen resolved, immediately unlisten
				fn();
			} else {
				unlisten = fn;
			}
		});

		return () => {
			cancelled = true;
			unlisten?.();
		};
	}

	async listPresets(): Promise<Preset[]> {
		try {
			const raw = (await invoke('list_presets')) as Array<Record<string, unknown>>;
			return raw.map((p) => ({
				index: p.index as number,
				name: p.name as string,
				persona: p.persona as string,
				genre: p.genre as string,
				isBuiltin: p.is_builtin as boolean
			}));
		} catch (e) {
			throw new Error(`Failed to list presets: ${e}`);
		}
	}

	async loadPreset(name: string): Promise<void> {
		try {
			await invoke('load_preset', { name });
		} catch (e) {
			throw new Error(`Failed to load preset: ${e}`);
		}
	}

	async savePreset(name: string): Promise<void> {
		try {
			await invoke('save_preset', { name });
		} catch (e) {
			throw new Error(`Failed to save preset: ${e}`);
		}
	}

	async deletePreset(name: string): Promise<void> {
		try {
			await invoke('delete_preset', { name });
		} catch (e) {
			throw new Error(`Failed to delete preset: ${e}`);
		}
	}
}
