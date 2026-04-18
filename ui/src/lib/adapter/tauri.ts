/**
 * Platform Adapter — Tauri IPC Implementation
 *
 * Implements ContrapunkAdapter by calling Tauri commands via invoke()
 * and subscribing to events via listen().
 */

import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { guitar } from '$lib/stores/guitar.svelte';
import { beat } from '$lib/stores/beat.svelte';
import type {
	ContrapunkAdapter,
	EngineState,
	GuitarConfig,
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
		autoKey: raw.auto_key as boolean,
		isRunning,
		counterpointSpecies: (raw.counterpoint_species as string) ?? 'Species1',
		counterpointStrictness: (raw.counterpoint_strictness as string) ?? 'Strict'
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
		lastBorrowedFrom: raw.last_borrowed_from as string,
		currentKey: (raw.current_key as string) ?? 'C'
	};
}

export class TauriAdapter implements ContrapunkAdapter {
	private _isRunning = false;
	private _guitarSignalUnsub: UnlistenFn | null = null;
	/** Unsubscribe handle for the Rust-driven beat-update event. */
	private _beatUpdateUnsub: UnlistenFn | null = null;

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

	async setAutoKey(enabled: boolean): Promise<void> {
		try {
			await invoke('set_auto_key', { enabled });
		} catch (e) {
			throw new Error(`Failed to set auto key: ${e}`);
		}
	}

	async setCounterpointSpecies(species: string): Promise<void> {
		try {
			await invoke('set_counterpoint_species', { species });
		} catch (e) {
			throw new Error(`Failed to set counterpoint species: ${e}`);
		}
	}

	async setCounterpointStrictness(strictness: string): Promise<void> {
		try {
			await invoke('set_counterpoint_strictness', { strictness });
		} catch (e) {
			throw new Error(`Failed to set counterpoint strictness: ${e}`);
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
			if (config.metronomeSubdivision !== undefined)
				snakeConfig.metronome_subdivision = config.metronomeSubdivision;
			if (config.metronomeSound !== undefined)
				snakeConfig.metronome_sound = config.metronomeSound;
			if (config.metronomeVolume !== undefined)
				snakeConfig.metronome_volume = config.metronomeVolume;
			if (config.metronomeCountInBars !== undefined)
				snakeConfig.metronome_count_in_bars = config.metronomeCountInBars;
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
			this.startBeatListener();

			// If guitar audio mode, listen for signal events and feed guitar store
			const GUITAR_AUDIO_SENTINEL = 999_997;
			if (inputIdx === GUITAR_AUDIO_SENTINEL) {
				guitar.detecting = true;

				// Throttle note display updates to ~10fps to reduce UI jitter
				let lastNoteUpdate = 0;
				const NOTE_UPDATE_INTERVAL = 100; // ms

				this._guitarSignalUnsub = await listen<Record<string, unknown>>('guitar-signal', (event) => {
					const p = event.payload;
					const rms = p.rms as number;
					const clarity = p.clarity as number;
					const noteName = p.note_name as string;

					// Signal graph data — push every frame (canvas renders independently)
					guitar.pushSignalFrame(rms, clarity);

					// Throttle reactive state updates to reduce re-renders
					const now = performance.now();
					if (now - lastNoteUpdate > NOTE_UPDATE_INTERVAL) {
						lastNoteUpdate = now;
						if (noteName) {
							guitar.currentNote = noteName;
							guitar.confidence = Math.round(clarity * 100);
							guitar.velocity = Math.round(rms * 800);
						} else {
							guitar.currentNote = '';
							guitar.confidence = 0;
						}
					}
				});
			}
		} catch (e) {
			throw new Error(`Failed to start routing: ${e}`);
		}
	}

	async stopRouting(): Promise<void> {
		try {
			await invoke('stop_routing');
			this._isRunning = false;
			this.stopBeatListener();

			// Clean up guitar signal listener
			if (this._guitarSignalUnsub) {
				this._guitarSignalUnsub();
				this._guitarSignalUnsub = null;
			}
			guitar.detecting = false;
			guitar.currentNote = '';
			guitar.confidence = 0;
			guitar.velocity = 0;
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

	async listAudioOutputDevices(): Promise<{ name: string; is_default: boolean }[]> {
		try {
			return (await invoke('list_audio_output_devices')) as { name: string; is_default: boolean }[];
		} catch (e) {
			throw new Error(`Failed to list audio output devices: ${e}`);
		}
	}

	async startAudioOutput(opts: { deviceId?: string; sampleRate?: number; bufferSize?: number }): Promise<void> {
		try {
			await invoke('start_audio_output', {
				deviceId: opts.deviceId ?? null,
				sampleRate: opts.sampleRate ?? null,
				bufferSize: opts.bufferSize ?? null,
			});
		} catch (e) {
			throw new Error(`Failed to start audio output: ${e}`);
		}
	}

	async stopAudioOutput(): Promise<void> {
		try {
			await invoke('stop_audio_output');
		} catch (e) {
			throw new Error(`Failed to stop audio output: ${e}`);
		}
	}

	async isAudioOutputRunning(): Promise<boolean> {
		try {
			return (await invoke('is_audio_output_running')) as boolean;
		} catch (e) {
			throw new Error(`Failed to check audio output running: ${e}`);
		}
	}

	async listAudioDevices(): Promise<string[]> {
		try {
			return (await invoke('list_audio_devices')) as string[];
		} catch (e) {
			throw new Error(`Failed to list audio devices: ${e}`);
		}
	}

	async setGuitarDevice(deviceName: string, channel: number): Promise<void> {
		try {
			await invoke('set_guitar_device', { deviceName, channel });
		} catch (e) {
			throw new Error(`Failed to set guitar device: ${e}`);
		}
	}

	async setGuitarConfig(config: GuitarConfig): Promise<void> {
		try {
			await invoke('set_guitar_config', {
				latencyMs: config.latencyMs,
				gain: config.gain,
				stringConfidence: config.stringConfidence,
				bends: config.bends,
				legato: config.legato,
				slides: config.slides,
				vibrato: config.vibrato,
			});
		} catch (e) {
			throw new Error(`Failed to set guitar config: ${e}`);
		}
	}

	async setGeneratorMode(mode: string): Promise<void> {
		try {
			await invoke('set_generator_mode', { mode });
		} catch (e) {
			throw new Error(`Failed to set generator mode: ${e}`);
		}
	}

	async setGeneratorEnabled(enabled: boolean): Promise<void> {
		try {
			await invoke('set_generator_enabled', { enabled });
		} catch (e) {
			throw new Error(`Failed to set generator enabled: ${e}`);
		}
	}

	async setGeneratorNotes(notes: number[]): Promise<void> {
		try {
			await invoke('set_generator_notes', { notes });
		} catch (e) {
			throw new Error(`Failed to set generator notes: ${e}`);
		}
	}

	async setGeneratorChordType(chordType: string): Promise<void> {
		try {
			await invoke('set_generator_chord_type', { chordType });
		} catch (e) {
			throw new Error(`Failed to set generator chord type: ${e}`);
		}
	}

	private _detuneCents = 0;

	setDetune(cents: number): void {
		this._detuneCents = cents;
		invoke('set_detune', { cents: Math.round(cents) }).catch(() => {
			// best-effort — if routing isn't active, detune is applied on next start
		});
	}

	getDetune(): number {
		return this._detuneCents;
	}

	// -- Suggestions (stub — desktop path deferred to Task 8) --

	getSuggestions(): { note: number; score: number }[] {
		return [];
	}

	setSuggestionWeight(_term: string, _value: number): void {
		// Desktop suggestion scoring deferred
	}

	resetSuggestionWeights(): void {
		// Desktop suggestion scoring deferred
	}

	/**
	 * Subscribe to real `beat-update` events emitted by the Rust router
	 * thread on each BeatClock crossing. Replaces the old JS setInterval
	 * approximation that drifted from the actual humanizer timing.
	 */
	private startBeatListener(): void {
		this.stopBeatListener();
		beat.running = true;
		listen<{ beat_position: number; beat_number: number; bpm: number; running: boolean }>(
			'beat-update',
			(event) => {
				const p = event.payload;
				beat.beatPosition = p.beat_position;
				beat.beatNumber = p.beat_number;
				beat.bpm = p.bpm;
				beat.running = p.running;
				beat.pulse = beat.pulse + 1;
				beat.lastCrossedAt = performance.now();
			}
		).then((unsub) => {
			this._beatUpdateUnsub = unsub;
		});
	}

	private stopBeatListener(): void {
		if (this._beatUpdateUnsub) {
			this._beatUpdateUnsub();
			this._beatUpdateUnsub = null;
		}
		beat.running = false;
	}
}
