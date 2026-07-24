/**
 * Platform Adapter — Tauri IPC Implementation
 *
 * Implements ContrapunkAdapter by calling Tauri commands via invoke()
 * and subscribing to events via listen().
 */

import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { guitar } from '$lib/stores/guitar.svelte';
import { transport } from '$lib/stores/transport.svelte';
import type {
	AddedPlugin,
	CalibrationStatus,
	ChainBlock,
	ClapPluginDescriptor,
	ContrapunkAdapter,
	DelayState,
	DelaySubdivision,
	EngineState,
	GuitarConfig,
	HoldMode,
	MidiDevice,
	MidiPermissionState,
	NoteState,
	PluginInputMode,
	PluginMidiOutputMode,
	Preset,
	ReverbState,
	SynthState,
	TransportState,
	VoiceOutputTarget
} from './types';

/** Map snake_case CalibrationStatus from the Tauri backend to camelCase. */
function mapCalibrationStatus(raw: Record<string, unknown>): CalibrationStatus {
	return {
		existsOnDisk: Boolean(raw.exists_on_disk),
		path: String(raw.path ?? ''),
		version: Number(raw.version ?? 0),
		sampleCounts: Array.isArray(raw.sample_counts)
			? (raw.sample_counts as number[])
			: []
	};
}

/**
 * Maps the Tauri backend's snake_case response to our camelCase EngineState.
 */
function mapEngineState(raw: Record<string, unknown>, isRunning: boolean): EngineState {
	const explicitMap = (raw.explicit_interval_map ?? {}) as Record<string, unknown>;
	return {
		key: normalizeKey(raw.key as string),
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
		counterpointStrictness: (raw.counterpoint_strictness as string) ?? 'Strict',
		explicitIntervalMap: {
			degreeOffsets: Array.isArray(explicitMap.degree_offsets)
				? (explicitMap.degree_offsets as number[][])
				: Array.from({ length: 7 }, () => [7]),
			fallbackOffsets: Array.isArray(explicitMap.fallback_offsets)
				? (explicitMap.fallback_offsets as number[])
				: [7]
		}
	};
}

/**
 * The Rust `Key` enum prints accidentals as flats (Db, Eb, Gb, Ab, Bb)
 * but the UI's `KeyName` type uses sharps. Normalize so auto-key
 * detector results map cleanly back to KeyName.
 */
const FLAT_TO_SHARP: Record<string, string> = {
	Db: 'C#',
	Eb: 'D#',
	Gb: 'F#',
	Ab: 'G#',
	Bb: 'A#'
};

function normalizeKey(key: string): string {
	return FLAT_TO_SHARP[key] ?? key;
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
		currentKey: normalizeKey((raw.current_key as string) ?? 'C'),
		canonNotes: (raw.canon_notes as number[]) ?? [],
		counterpointNotes: (raw.counterpoint_notes as number[]) ?? []
	};
}

export class TauriAdapter implements ContrapunkAdapter {
	readonly capabilities = {
		delayTempoSync: true,
		chainEditor: true,
		noteUpdates: true,
		transportControl: true,
		midiDevicePicker: true,
		audioFx: true,
		builtInFx: true,
		companionLanes: true,
		intervalMaps: true,
		patternLanes: true,
		// Tauri renders the full I/O Input subtab source radio —
		// MIDI / Guitar Audio (cpal backend) / Voice (disabled).
		inputSourcePicker: true,
		// Tauri owns MIDI routing via midir; per-voice port pickers
		// drive setVoiceOutput end-to-end.
		perVoicePortRouting: true,
		// Plugin-only host MIDI mode selector.
		pluginMidiOutputMode: false,
		roleMix: true,
		// Calibration profile persists to app_data_dir() and applies on
		// next routing start via GuitarBridge -> GuitarInput.
		calibrationFlow: true
	} as const;

	private _isRunning = false;
	private _guitarSignalUnsub: UnlistenFn | null = null;
	private _beatUpdateUnsub: UnlistenFn | null = null;

	async init(): Promise<void> {
		// Tauri is ready when this code runs in the webview.
		// Fetch initial state to confirm backend is available.
		await this.getEngineState();

		// Subscribe once to transport beat-update events. The audio
		// clock emits one per beat crossing; the store updates and
		// components re-render via $effect on `pulse`. Guard against
		// re-init (HMR or programmatic re-call) leaking the previous
		// listener.
		if (this._beatUpdateUnsub) {
			try {
				this._beatUpdateUnsub();
			} catch {
				// best-effort
			}
			this._beatUpdateUnsub = null;
		}
		this._beatUpdateUnsub = await listen<{
			total_beat: number;
			beat_in_bar: number;
			bar: number;
			bpm: number;
			running: boolean;
		}>('beat-update', (event) => {
			const p = event.payload;
			transport.applyBeatUpdate({
				totalBeat: p.total_beat,
				beatInBar: p.beat_in_bar,
				bar: p.bar,
				bpm: p.bpm,
				running: p.running
			});
		});
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

	async setOctaveIntensity(amount: number): Promise<void> {
		try {
			await invoke('set_octave_intensity', { amount });
		} catch (e) {
			throw new Error(`Failed to set octave intensity: ${e}`);
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

	async setExplicitIntervalMap(degreeOffsets: number[][], fallbackOffsets: number[]): Promise<void> {
		try {
			await invoke('set_explicit_interval_map', {
				degreeOffsets,
				fallbackOffsets
			});
		} catch (e) {
			throw new Error(`Failed to set explicit interval map: ${e}`);
		}
	}

	async companionSetEnabled(enabled: boolean): Promise<void> {
		try {
			await invoke('companion_set_enabled', { enabled });
		} catch (e) {
			throw new Error(`Failed to set companion enabled: ${e}`);
		}
	}

	async companionIsEnabled(): Promise<boolean> {
		try {
			return (await invoke('companion_is_enabled')) as boolean;
		} catch (e) {
			throw new Error(`Failed to read companion enabled: ${e}`);
		}
	}

	async companionSetGlobalHoldMode(holdMode: HoldMode): Promise<void> {
		try {
			await invoke('companion_set_global_hold_mode', { holdModeJson: holdMode });
		} catch (e) {
			throw new Error(`Failed to set companion hold mode: ${e}`);
		}
	}

	async canonSetEnabled(enabled: boolean): Promise<void> {
		try {
			await invoke('canon_set_enabled', { enabled });
		} catch (e) {
			throw new Error(`Failed to set canon enabled: ${e}`);
		}
	}

	async canonSetDelay(beats: number): Promise<void> {
		try {
			await invoke('canon_set_delay', { beats });
		} catch (e) {
			throw new Error(`Failed to set canon delay: ${e}`);
		}
	}

	async canonSetTranspose(degrees: number): Promise<void> {
		try {
			await invoke('canon_set_transpose', { degrees });
		} catch (e) {
			throw new Error(`Failed to set canon transpose: ${e}`);
		}
	}

	async canonSetVoices(
		voices: Array<{
			delay_beats: number;
			transpose_degrees: number;
			time_ratio?: number;
			harmony_mode?: string | null;
			reference_voice?: number | null;
			voice_count?: number | null;
			voice_position?: number | null;
			voice_leading_enabled?: boolean | null;
			voice_leading_style?: string | null;
			octave_mode?: string | null;
			counterpoint_species?: string | null;
			counterpoint_strictness?: string | null;
		}>
	): Promise<void> {
		try {
			await invoke('canon_set_voices', { voices });
		} catch (e) {
			throw new Error(`Failed to set canon voices: ${e}`);
		}
	}

	async counterpointSetConfig(config: {
		enabled?: boolean;
		species?: string;
		transpose_degrees?: number;
		prefer_above?: boolean;
	}): Promise<void> {
		try {
			await invoke('counterpoint_set_config', config);
		} catch (e) {
			throw new Error(`Failed to set counterpoint config: ${e}`);
		}
	}

	async canonConfigure(partial: Record<string, unknown>): Promise<void> {
		try {
			await invoke('canon_configure', { partial });
		} catch (e) {
			throw new Error(`Failed to configure canon lane: ${e}`);
		}
	}

	async counterpointConfigure(partial: Record<string, unknown>): Promise<void> {
		try {
			await invoke('counterpoint_configure', { partial });
		} catch (e) {
			throw new Error(`Failed to configure counterpoint lane: ${e}`);
		}
	}

	async patternConfigure(
		laneId: 'pattern_low' | 'pattern_counter',
		partial: Record<string, unknown>
	): Promise<void> {
		try {
			await invoke('pattern_configure', { laneId, partial });
		} catch (e) {
			throw new Error(`Failed to configure ${laneId}: ${e}`);
		}
	}

	async patternState(
		laneId: 'pattern_low' | 'pattern_counter'
	): Promise<Record<string, unknown> | null> {
		try {
			const state = await invoke('pattern_state', { laneId });
			return state !== null && typeof state === 'object'
				? (state as Record<string, unknown>)
				: null;
		} catch (e) {
			throw new Error(`Failed to read ${laneId}: ${e}`);
		}
	}

	async counterpointState(): Promise<{
		enabled: boolean;
		species: string;
		transpose_degrees: number;
		prefer_above: boolean;
	} | null> {
		try {
			const s = await invoke('counterpoint_state');
			if (s === null || typeof s !== 'object') return null;
			return s as {
				enabled: boolean;
				species: string;
				transpose_degrees: number;
				prefer_above: boolean;
			};
		} catch (e) {
			throw new Error(`Failed to read counterpoint state: ${e}`);
		}
	}

	async canonState(): Promise<{
		enabled: boolean;
		delay_beats: number;
		transpose_degrees: number;
		voices?: Array<{
			delay_beats: number;
			transpose_degrees: number;
			time_ratio?: number;
			harmony_mode?: string | null;
			reference_voice?: number | null;
		}>;
	} | null> {
		try {
			const s = await invoke('canon_state');
			if (s === null || typeof s !== 'object') return null;
			return s as {
				enabled: boolean;
				delay_beats: number;
				transpose_degrees: number;
				voices?: Array<{
					delay_beats: number;
					transpose_degrees: number;
					time_ratio?: number;
					harmony_mode?: string | null;
					reference_voice?: number | null;
				}>;
			};
		} catch (e) {
			throw new Error(`Failed to read canon state: ${e}`);
		}
	}

	/** Tauri uses native OS MIDI APIs; no browser-style permission gate. */
	readonly midiPermissionState: MidiPermissionState = 'granted';

	/** No-op on Tauri — native MIDI is always available. */
	async requestMidiPermission(): Promise<MidiPermissionState> {
		return 'granted';
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
			const GUITAR_AUDIO_SENTINEL = 999_997;
			if (inputIdx === GUITAR_AUDIO_SENTINEL) {
				// Persisted UI selections are not guaranteed to have been touched
				// this session. Synchronize them at the exact start boundary so the
				// bridge cannot silently fall back to device/input zero.
				await guitar.syncDevice();
				await guitar.syncConfig();
			}

			await invoke('start_routing', { inputIdx, outputIndices });
			this._isRunning = true;

			// If guitar audio mode, listen for signal events and feed guitar store
			if (inputIdx === GUITAR_AUDIO_SENTINEL) {
				guitar.detecting = true;
				// A successful bridge start means cpal accepted this input.
				// The store is 1-indexed for display; the backend already received
				// the corresponding 0-indexed channel through syncDevice().
				guitar.activeChannel = guitar.selectedChannel;

				// Double-start guard: if a previous start left a
				// listener wired up (engine.start called twice without
				// an intervening stop), drop it before subscribing
				// fresh so the old handle doesn't leak forever.
				if (this._guitarSignalUnsub) {
					try {
						this._guitarSignalUnsub();
					} catch {
						// best-effort
					}
					this._guitarSignalUnsub = null;
				}

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

			// Clean up guitar signal listener
			if (this._guitarSignalUnsub) {
				this._guitarSignalUnsub();
				this._guitarSignalUnsub = null;
			}
			guitar.detecting = false;
			guitar.resetLiveState();
		} catch (e) {
			throw new Error(`Failed to stop routing: ${e}`);
		}
	}

	async setVoiceOutput(voiceIdx: number, target: VoiceOutputTarget): Promise<void> {
		try {
			await invoke('set_voice_output', { voiceIdx, target });
		} catch (e) {
			throw new Error(`Failed to set voice output: ${e}`);
		}
	}

	async getVoiceOutputs(): Promise<VoiceOutputTarget[]> {
		try {
			return (await invoke('get_voice_outputs')) as VoiceOutputTarget[];
		} catch (e) {
			throw new Error(`Failed to get voice outputs: ${e}`);
		}
	}

	async getPluginInputMode(): Promise<PluginInputMode> {
		return 'midi';
	}

	async setPluginInputMode(_mode: PluginInputMode): Promise<void> {}

	async getPluginMidiOutputMode(): Promise<PluginMidiOutputMode> {
		return 'full';
	}

	async setPluginMidiOutputMode(_mode: PluginMidiOutputMode): Promise<void> {}

	async getPluginSynthEnabled(): Promise<boolean> {
		return true;
	}

	async setPluginSynthEnabled(_enabled: boolean): Promise<void> {}

	async panicAllNotesOff(): Promise<void> {
		await invoke('panic_all_notes_off');
	}

	onPluginParamsUpdate(_callback: () => void): () => void {
		return () => {};
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

	onKnobCcRaw(callback: (cc: number, value: number) => void): () => void {
		let unlisten: UnlistenFn | undefined;
		let cancelled = false;

		listen<{ cc: number; value: number }>('knob-cc-raw', (event) => {
			if (!cancelled) {
				callback(event.payload.cc, event.payload.value);
			}
		}).then((fn) => {
			if (cancelled) fn();
			else unlisten = fn;
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

	async getCalibrationStatus(): Promise<CalibrationStatus> {
		try {
			const raw = (await invoke('get_calibration_status')) as Record<string, unknown>;
			return mapCalibrationStatus(raw);
		} catch (e) {
			throw new Error(`Failed to get calibration status: ${e}`);
		}
	}

	async loadCalibrationProfile(): Promise<CalibrationStatus> {
		// Backend returns CalibrationStatus directly (post brutal-critic
		// #13 — no longer the legacy two-roundtrip dance).
		try {
			const raw = (await invoke('load_calibration_profile')) as Record<string, unknown>;
			return mapCalibrationStatus(raw);
		} catch (e) {
			throw new Error(`Failed to load calibration profile: ${e}`);
		}
	}

	async saveCalibrationProfile(profileJson: string): Promise<CalibrationStatus> {
		try {
			const raw = (await invoke('save_calibration_profile', {
				profileJson
			})) as Record<string, unknown>;
			return mapCalibrationStatus(raw);
		} catch (e) {
			throw new Error(`Failed to save calibration profile: ${e}`);
		}
	}

	async deleteCalibrationProfile(): Promise<CalibrationStatus> {
		try {
			const raw = (await invoke('delete_calibration_profile')) as Record<string, unknown>;
			return mapCalibrationStatus(raw);
		} catch (e) {
			throw new Error(`Failed to delete calibration profile: ${e}`);
		}
	}

	async getLastPortMap(): Promise<number[]> {
		try {
			const raw = (await invoke('get_last_port_map')) as number[];
			return Array.isArray(raw) ? raw : [];
		} catch {
			return [];
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

	// -- Transport --

	async getTransportState(): Promise<TransportState> {
		const raw = (await invoke('get_transport_state')) as Record<string, unknown>;
		return {
			running: raw.running as boolean,
			bpm: raw.bpm as number,
			beatsPerBar: raw.beats_per_bar as number,
			beatUnit: raw.beat_unit as number,
			sampleRate: raw.sample_rate as number,
			samplePos: raw.sample_pos as number,
			beatPosition: raw.beat_position as number,
			bar: raw.bar as number,
			metronomeEnabled: raw.metronome_enabled as boolean
		};
	}

	async transportPlay(): Promise<void> {
		await invoke('transport_play');
	}

	async transportStop(): Promise<void> {
		await invoke('transport_stop');
	}

	async transportReset(): Promise<void> {
		await invoke('transport_reset');
	}

	async setBpm(bpm: number): Promise<void> {
		await invoke('set_bpm', { bpm });
	}

	async setTimeSignature(beatsPerBar: number, beatUnit: number): Promise<void> {
		await invoke('set_time_signature', { beatsPerBar, beatUnit });
	}

	async setMetronomeEnabled(enabled: boolean): Promise<void> {
		await invoke('set_metronome_enabled', { enabled });
	}

	// -- Built-in synth --

	async getSynthState(): Promise<SynthState> {
		const raw = (await invoke('get_synth_state')) as Record<string, unknown>;
		return {
			enabled: raw.enabled as boolean,
			masterGain: raw.master_gain as number,
			mixGains: Array.isArray(raw.mix_gains) ? (raw.mix_gains as number[]) : undefined
		};
	}

	async setSynthEnabled(enabled: boolean): Promise<void> {
		await invoke('set_synth_enabled', { enabled });
	}
	async setSynthMasterGain(value: number): Promise<void> {
		await invoke('set_synth_master_gain', { value });
	}
	async setSynthMixGain(group: number, value: number): Promise<void> {
		await invoke('set_synth_mix_gain', { group, value });
	}

	// -- Built-in FX (reverb) --

	async getReverbState(): Promise<ReverbState> {
		const raw = (await invoke('get_reverb_state')) as Record<string, unknown>;
		return {
			enabled: raw.enabled as boolean,
			mix: raw.mix as number,
			roomSize: raw.room_size as number,
			damping: raw.damping as number
		};
	}
	async setReverbEnabled(enabled: boolean): Promise<void> {
		await invoke('set_reverb_enabled', { enabled });
	}
	async setReverbMix(value: number): Promise<void> {
		await invoke('set_reverb_mix', { value });
	}
	async setReverbRoomSize(value: number): Promise<void> {
		await invoke('set_reverb_room_size', { value });
	}
	async setReverbDamping(value: number): Promise<void> {
		await invoke('set_reverb_damping', { value });
	}

	async getDelayState(): Promise<DelayState> {
		const raw = (await invoke('get_delay_state')) as Record<string, unknown>;
		return {
			enabled: raw.enabled as boolean,
			mix: raw.mix as number,
			timeMs: raw.time_ms as number,
			feedback: raw.feedback as number,
			syncEnabled: (raw.sync_enabled as boolean) ?? false,
			subdivision: ((raw.subdivision as DelaySubdivision) ?? '1/8d') as DelaySubdivision
		};
	}
	async setDelayEnabled(enabled: boolean): Promise<void> {
		await invoke('set_delay_enabled', { enabled });
	}
	async setDelayMix(value: number): Promise<void> {
		await invoke('set_delay_mix', { value });
	}
	async setDelayTimeMs(ms: number): Promise<void> {
		await invoke('set_delay_time_ms', { ms });
	}
	async setDelayFeedback(value: number): Promise<void> {
		await invoke('set_delay_feedback', { value });
	}
	async setDelaySyncEnabled(enabled: boolean): Promise<void> {
		await invoke('set_delay_sync_enabled', { enabled });
	}
	async setDelaySubdivision(subdivision: DelaySubdivision): Promise<void> {
		await invoke('set_delay_subdivision', { subdivision });
	}

	// -- Chain topology --

	async listChainBlocks(): Promise<ChainBlock[]> {
		const raw = (await invoke('list_chain_blocks')) as Array<Record<string, unknown>>;
		return raw.map((b) => ({
			typeId: b.type_id as string,
			name: b.name as string
		}));
	}
	async removeChainBlock(index: number): Promise<void> {
		await invoke('remove_chain_block', { index });
	}
	async clearChain(): Promise<void> {
		await invoke('clear_chain');
	}

	// -- CLAP plugin host --

	async listClapPlugins(): Promise<ClapPluginDescriptor[]> {
		const raw = (await invoke('list_clap_plugins')) as Array<Record<string, unknown>>;
		return raw.map((p) => ({
			id: p.id as string,
			name: p.name as string,
			vendor: (p.vendor as string) ?? '',
			version: (p.version as string) ?? '',
			path: p.path as string
		}));
	}
	async addClapPluginToChain(path: string): Promise<AddedPlugin> {
		const raw = (await invoke('add_clap_plugin_to_chain', { path })) as Record<string, unknown>;
		return {
			pluginId: raw.plugin_id as number,
			name: raw.name as string,
			path: raw.path as string,
			hasGui: raw.has_gui as boolean
		};
	}
	async openPluginGui(pluginId: number): Promise<void> {
		await invoke('open_plugin_gui', { pluginId });
	}
	async getPluginGuiSize(pluginId: number): Promise<{ width: number; height: number } | null> {
		const raw = (await invoke('get_plugin_gui_size', { pluginId })) as
			| { width: number; height: number }
			| null;
		return raw;
	}
	async openPluginGuiEmbedded(
		pluginId: number,
		x: number,
		y: number,
		width: number,
		height: number
	): Promise<void> {
		await invoke('open_plugin_gui_embedded', { pluginId, x, y, width, height });
	}
	async setPluginGuiFrame(
		pluginId: number,
		x: number,
		y: number,
		width: number,
		height: number
	): Promise<void> {
		await invoke('set_plugin_gui_frame', { pluginId, x, y, width, height });
	}
	async closePluginGui(pluginId: number): Promise<void> {
		await invoke('close_plugin_gui', { pluginId });
	}
	async removePlugin(pluginId: number): Promise<void> {
		await invoke('remove_plugin', { pluginId });
	}
}
