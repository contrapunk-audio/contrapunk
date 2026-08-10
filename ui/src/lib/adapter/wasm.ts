/**
 * Platform Adapter — WASM Implementation
 *
 * Implements ContrapunkAdapter by calling wasm-bindgen exported functions
 * directly. Used when running in the browser without Tauri.
 */

import type {
	ContrapunkAdapter,
	EngineState,
	GuitarConfig,
	HarmonicLimit,
	HoldMode,
	MidiDevice,
	MidiPermissionState,
	NoteState,
	PhraseState,
	PluginInputMode,
	PluginMidiOutputMode,
	Preset,
	SlideConfig,
	SlideRole,
	SlideVoiceState,
	TransportState,
	TuningStyle,
	VoiceOutputAssignment,
	VoiceOutputTarget,
	VoiceRouteId
} from './types';
import {
	GuitarAudioCapture,
	serializeGuitarCaptureOperation
} from '$lib/audio/guitarCapture';
import { guitar } from '$lib/stores/guitar.svelte';
import { transport } from '$lib/stores/transport.svelte';
import * as embedAudio from '$lib/embed/audio';
import { defaultSlideConfig, resolveSlide } from '$lib/slide/config';
import { mapPhraseState } from './phrase';

/**
 * Dynamically imported WASM module.
 * Resolved during init() to avoid top-level import failures
 * when the wasm-pkg hasn't been built yet.
 */
// eslint-disable-next-line @typescript-eslint/no-explicit-any
let wasmModule: any = null;
// eslint-disable-next-line @typescript-eslint/no-explicit-any
let engine: any = null;
// CompanionWasm instance — hosts the Canon + Counterpoint Lanes that
// the v1.2.0 native build ships. Constructed during init().
// eslint-disable-next-line @typescript-eslint/no-explicit-any
let companion: any = null;
// Animation-frame loop that advances the Companion's transport and
// drains pending lane emissions. Started on first injectNoteOn.
let companionTickHandle: number | null = null;
// Per-source ownership counts preserve overlapping voices at the same
// pitch. A Set incorrectly removed the pitch when the first of several
// Canon/Counterpoint owners released it.
type NoteCounts = Map<number, number>;
const activeCompanionNotes: NoteCounts = new Map();
const activeCanonNotes: NoteCounts = new Map();
const activeCounterpointNotes: NoteCounts = new Map();

function countNoteOn(notes: NoteCounts, note: number): boolean {
	const firstOwner = !notes.has(note);
	notes.set(note, (notes.get(note) ?? 0) + 1);
	return firstOwner;
}

function countNoteOff(notes: NoteCounts, note: number): boolean {
	const count = (notes.get(note) ?? 0) - 1;
	if (count > 0) {
		notes.set(note, count);
		return false;
	}
	const hadOwner = notes.delete(note);
	return hadOwner;
}

function activeNotes(notes: NoteCounts): number[] {
	return [...notes.keys()].sort((a, b) => a - b);
}

function clearCompanionNotes(): void {
	activeCompanionNotes.clear();
	activeCanonNotes.clear();
	activeCounterpointNotes.clear();
}

/** Debug logger — set `window.__cpDebug = true` in DevTools to
 *  print a snapshot of the Companion + per-voice state on every
 *  player NoteOn / NoteOff and the dispatch ops that came back.
 *  Off by default so production users don't see console noise. */
function isCompanionDebug(): boolean {
	try {
		return Boolean((globalThis as unknown as { __cpDebug?: boolean }).__cpDebug);
	} catch {
		return false;
	}
}

function logCompanionEvent(label: string, payload: Record<string, unknown>): void {
	if (!isCompanionDebug() || !companion) return;
	try {
		const snap = JSON.parse(companion.debug_snapshot());
		// eslint-disable-next-line no-console
		console.log(`[cp:${label}]`, { ...payload, ...snap });
	} catch {
		/* serialization failure — ignore */
	}
}

/**
 * Sentinel input index used to signal "guitar audio input" rather than MIDI.
 * Must match the value used in the UI when starting guitar routing.
 */
const GUITAR_AUDIO_SENTINEL = 999_997;

export class WasmAdapter implements ContrapunkAdapter {
	readonly capabilities = {
		// embedAudio has no tempo-sync surface yet (see setDelaySync*
		// stubs below) — hide the SYNC toggle on web to avoid a
		// visible-lie until that ships.
		delayTempoSync: false,
		// listChainBlocks / removeChainBlock are no-ops on WASM.
		chainEditor: false,
		// dispatchOpsJson populates canon/counterpoint sets; piano
		// colors work on WASM.
		noteUpdates: true,
		// WASM runs its own Web Audio transport — the BPM/play UI
		// drives the embedded audio context, so transportControl is
		// authoritative on this surface.
		transportControl: true,
		// Web MIDI device pickers work in browsers that support the
		// Web MIDI API; the UI gracefully degrades when not granted.
		midiDevicePicker: true,
		// WASM has its own Elixir AudioWorklet synth, without legacy FX.
		audioFx: true,
		builtInFx: false,
		// WASM has Companion lanes via the WasmCompanion bridge.
		companionLanes: true,
		intervalMaps: true,
		patternLanes: true,
		phraseContext: true,
		// WASM exposes MIDI + guitar-audio via Web MIDI + WebAudio
		// (guitarCapture.ts). Voice option is disabled like everywhere.
		inputSourcePicker: true,
		// Per-voice port routing is NOT honored on WASM today: the MIDI
		// dispatch loop (search for `outs[i % outs.length]`) round-robins
		// regardless of what `setVoiceOutput` was passed. Flip back to
		// true once dispatch resolves the stable route map for kind/port.
		// Brutal-critic #12 caveat — was advertising a
		// capability we didn't have.
		perVoicePortRouting: false,
		// Plugin-only host MIDI mode selector.
		pluginMidiOutputMode: false,
		// Elixir AudioWorklet exposes the same four role buses as native.
		roleMix: true,
		nativeTuning: true,
		// No persistence layer for the calibration profile on web yet
		// — hide the Calibrate button + status badge.
		calibrationFlow: false
	} as const;

	private initialized = false;
	private _isRunning = false;
	private noteUpdateCallback: ((state: NoteState) => void) | null = null;
	private pollingHandle: number | null = null;
	private midiAccess: MIDIAccess | null = null;
	private _midiPermissionState: MidiPermissionState = 'idle';
	private activeInput: MIDIInput | null = null;
	private activeOutputs: MIDIOutput[] = [];
	private _detuneCents = 0;
	private _synthEnabled = true;
	private _synthMasterGain = 0.25;
	private _synthMixGains = [1, 1, 1, 1];
	private _slideConfig = defaultSlideConfig();
	/** Pitch bend range in semitones (standard MIDI default). */
	private pitchBendRangeSemitones = 2;
	/** Guitar audio capture instance for browser-based pitch detection. */
	private guitarCapture: GuitarAudioCapture | null = null;
	private guitarCaptureOperation: Promise<void> = Promise.resolve();
	/** Currently selected guitar device ID (for audio capture). */
	private _guitarDeviceId = '';
	/** Currently selected guitar channel (0-based). */
	private _guitarChannel = 0;
	private _guitarOutputIndices: number[] = [];
	private _guitarConfig: GuitarConfig = {
		latencyMs: 21,
		gain: 1,
		stringConfidence: 0.4,
		bends: true,
		legato: true,
		slides: true,
		vibrato: false
	};

	private applyGuitarConfig(capture: GuitarAudioCapture): void {
		capture.setConfig({
			bends: this._guitarConfig.bends,
			legato: this._guitarConfig.legato,
			slides: this._guitarConfig.slides,
			vibrato: this._guitarConfig.vibrato,
			gain: this._guitarConfig.gain,
			stringConfidence: this._guitarConfig.stringConfidence
		});
	}

	private sendGuitarMidi(message: number[]): void {
		for (const output of this.activeOutputs) {
			try {
				output.send(message);
			} catch {
				/* disconnected output */
			}
		}
	}

	private enqueueGuitarCaptureOperation(operation: () => Promise<void>): Promise<void> {
		const { result, tail } = serializeGuitarCaptureOperation(
			this.guitarCaptureOperation,
			operation
		);
		this.guitarCaptureOperation = tail;
		return result;
	}

	async init(): Promise<void> {
		if (this.initialized) return;

		try {
			// Dynamic import of wasm-pack output from $lib/wasm-pkg
			// wasm-pack builds to ui/src/lib/wasm-pkg/ which is under $lib
			wasmModule = await import('$lib/wasm-pkg');
			if (wasmModule.default && typeof wasmModule.default === 'function') {
				await wasmModule.default();
			}
			engine = new wasmModule.Engine();
			// Companion + Lanes. Mirrors what Tauri's AppState registers
			// at boot (CanonLane + CounterpointLane under one master
			// Companion). Construction defaults to enabled = ON.
			try {
				companion = new wasmModule.CompanionWasm();
			} catch (e) {
				console.warn('[wasm] CompanionWasm init failed:', e);
				companion = null;
			}
			this.initialized = true;
			this.startCompanionTick();
		} catch (e) {
			throw new Error(`Failed to initialize WASM: ${e}`);
		}
	}

	/** Run a requestAnimationFrame loop that advances the Companion's
	 *  transport by ~one buffer per frame and drains any lane emissions
	 *  whose scheduled fire_at has elapsed. Frames-per-tick is computed
	 *  to keep the transport roughly in real time at 48 kHz. */
	private startCompanionTick(): void {
		if (companionTickHandle !== null || !companion) return;
		let last = performance.now();
		const loop = () => {
			if (!companion) {
				companionTickHandle = null;
				return;
			}
			const now = performance.now();
			const dtMs = Math.max(0, now - last);
			last = now;
			// Frames at 48 kHz that match wallclock delta.
			const frames = Math.max(1, Math.round(dtMs * 48));
			try {
				companion.advance(frames);
				const json = companion.tick();
				this.dispatchOpsJson(json);
			} catch {
				/* keep the loop alive */
			}
			companionTickHandle = requestAnimationFrame(loop);
		};
		companionTickHandle = requestAnimationFrame(loop);
	}

	/** Play (or release) the lane-emitted dispatch ops on the browser
	 *  WebAudio synth + any active external MIDI output. Updates the
	 *  per-lane note-attribution sets keyed by the op's `lane` field
	 *  so the piano UI can color each lane's notes distinctly. */
	private dispatchOpsJson(json: string): void {
		if (!json || json === '[]') return;
		let ops: Array<{
			kind: string;
			note?: number;
			velocity?: number;
			channel?: number;
			lane?: string;
			voice_slot?: number;
		}>;
		try {
			ops = JSON.parse(json);
		} catch {
			return;
		}
		for (const op of ops) {
			const laneNotes =
				op.lane === 'canon' || op.lane === 'pattern_low'
					? activeCanonNotes
					: op.lane === 'counterpoint' || op.lane === 'pattern_counter'
						? activeCounterpointNotes
						: null;
			if (op.kind === 'note_on' && typeof op.note === 'number') {
				const velocity = op.velocity ?? 100;
				const firstOwner = countNoteOn(activeCompanionNotes, op.note);
				if (laneNotes) countNoteOn(laneNotes, op.note);
				const role =
					op.lane === 'canon' || op.lane === 'pattern_low'
						? 2
						: op.lane === 'counterpoint' || op.lane === 'pattern_counter'
							? 3
							: 1;
				const slideRole: SlideRole = role === 2 ? 'canon' : role === 3 ? 'counterpoint' : 'harmony';
				const voiceSlot = Math.max(0, Math.min(7, Math.trunc(op.voice_slot ?? 0)));
				embedAudio.noteOn(
					op.note,
					velocity,
					undefined,
					role,
					undefined,
					voiceSlot,
					resolveSlide(this._slideConfig, { role: slideRole, voice: voiceSlot })
				);
				if (firstOwner) {
					if (this.activeOutputs.length > 0) {
						this.activeOutputs[0].send([0x90, op.note, velocity]);
					}
				}
			} else if (op.kind === 'note_off' && typeof op.note === 'number') {
				const lastOwner = countNoteOff(activeCompanionNotes, op.note);
				if (laneNotes) countNoteOff(laneNotes, op.note);
				const role =
					op.lane === 'canon' || op.lane === 'pattern_low'
						? 2
						: op.lane === 'counterpoint' || op.lane === 'pattern_counter'
							? 3
							: 1;
				embedAudio.noteOff(op.note, undefined, role, op.voice_slot ?? 0);
				if (lastOwner) {
					if (this.activeOutputs.length > 0) {
						this.activeOutputs[0].send([0x80, op.note, 0]);
					}
				}
			} else if (op.kind === 'all_notes_off') {
				embedAudio.allNotesOff();
				clearCompanionNotes();
			}
		}
	}

	/**
	 * Tear down background loops and resources. Called from the root
	 * layout `onDestroy` hook so hot-reload and page navigation don't
	 * leak lingering guitar captures.
	 */
	destroy(): void {
		void this.enqueueGuitarCaptureOperation(() => this.stopRoutingNow());
		// Tick / polling / clock all start RAF or setInterval loops
		// that close over the *current* `engine` / `companion`. On
		// HMR (and now plugin/test teardown) these must be cancelled
		// or the loop keeps marching against stale references and
		// pins the WASM instance from GC.
		if (companionTickHandle !== null) {
			cancelAnimationFrame(companionTickHandle);
			companionTickHandle = null;
		}
		this.stopNotePolling();
		this.stopClock();
		embedAudio.destroy();
		clearCompanionNotes();
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
			const explicitMap = raw.explicit_interval_map ?? {};
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
				autoKey: raw.auto_key ?? false,
				tuningStyle: raw.tuning_style === 'pure' ? 'pure' : 'standard',
				tuningDepth: typeof raw.tuning_depth === 'number' ? raw.tuning_depth : 0.6,
				harmonicLimit: raw.harmonic_limit === 'seven' ? 'seven' : 'five',
				isRunning: this._isRunning,
				counterpointSpecies: raw.counterpoint_species ?? 'Species1',
				counterpointStrictness: raw.counterpoint_strictness ?? 'Strict',
				explicitIntervalMap: {
					degreeOffsets: Array.isArray(explicitMap.degree_offsets)
						? explicitMap.degree_offsets
						: Array.from({ length: 7 }, () => [7]),
					fallbackOffsets: Array.isArray(explicitMap.fallback_offsets)
						? explicitMap.fallback_offsets
						: [7]
				}
			};
		} catch (e) {
			throw new Error(`Failed to get engine state: ${e}`);
		}
	}

	/** Mirror the wasm Engine's global state (key / mode / scale_mode
	 *  / voice_count / voice_position) into the CompanionWasm's
	 *  internal engine_snapshot so the canon mini-engines see the
	 *  same key/scale as the player. The Tauri build shares one
	 *  HarmonyEngine across both; in WASM they're separate objects. */
	private syncCompanionGlobal(): void {
		if (!companion || !engine) return;
		try {
			const raw = engine.get_state();
			companion.set_global_state(
				raw.key ?? 'C',
				raw.mode ?? 'PassThrough',
				raw.scale_mode ?? 'Ionian',
				raw.voice_count ?? 2,
				raw.voice_position ?? 0
			);
			companion.set_global_interval_map(JSON.stringify(
				raw.explicit_interval_map ?? {
					degree_offsets: Array.from({ length: 7 }, () => [7]),
					fallback_offsets: [7]
				}
			));
		} catch {
			/* best-effort */
		}
	}

	async setKey(key: string): Promise<void> {
		this.ensureInit();
		try {
			engine.set_key(key);
			this.syncCompanionGlobal();
		} catch (e) {
			throw new Error(`Failed to set key: ${e}`);
		}
	}

	async setMode(mode: string): Promise<void> {
		this.ensureInit();
		try {
			engine.set_mode(mode);
			this.syncCompanionGlobal();
		} catch (e) {
			throw new Error(`Failed to set mode: ${e}`);
		}
	}

	async setScaleMode(mode: string): Promise<void> {
		this.ensureInit();
		try {
			engine.set_scale_mode(mode);
			this.syncCompanionGlobal();
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

	async setOctaveIntensity(amount: number): Promise<void> {
		this.ensureInit();
		try {
			engine.set_octave_intensity(amount);
		} catch (e) {
			throw new Error(`Failed to set octave intensity: ${e}`);
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
			this.syncCompanionGlobal();
		} catch (e) {
			throw new Error(`Failed to set voice position: ${e}`);
		}
	}

	async setVoiceCount(count: number): Promise<void> {
		this.ensureInit();
		try {
			engine.set_voice_count(count);
			this.syncCompanionGlobal();
		} catch (e) {
			throw new Error(`Failed to set voice count: ${e}`);
		}
	}

	async setAutoKey(enabled: boolean): Promise<void> {
		this.ensureInit();
		try {
			engine.set_auto_key(enabled);
		} catch (e) {
			throw new Error(`Failed to set auto key: ${e}`);
		}
	}

	async setTuningStyle(style: TuningStyle): Promise<void> {
		this.ensureInit();
		await this.panicAllNotesOff();
		engine.set_tuning_style(style);
	}

	async setTuningDepth(depth: number): Promise<void> {
		this.ensureInit();
		await this.panicAllNotesOff();
		engine.set_tuning_depth(depth);
	}

	async setHarmonicLimit(limit: HarmonicLimit): Promise<void> {
		this.ensureInit();
		await this.panicAllNotesOff();
		engine.set_harmonic_limit(limit);
	}

	async setTuningCompare(enabled: boolean): Promise<void> {
		embedAudio.setCompareStandard(enabled);
	}

	async getSlideConfig(): Promise<SlideConfig> {
		return this._slideConfig;
	}

	async setSlideConfig(config: SlideConfig): Promise<void> {
		this._slideConfig = config;
	}

	async getSlideVoices(): Promise<SlideVoiceState[]> {
		return embedAudio.getSlideVoices();
	}

	async setCounterpointSpecies(species: string): Promise<void> {
		this.ensureInit();
		try {
			engine.set_counterpoint_species(species);
		} catch (e) {
			throw new Error(`Failed to set counterpoint species: ${e}`);
		}
	}

	async setCounterpointStrictness(strictness: string): Promise<void> {
		this.ensureInit();
		try {
			engine.set_counterpoint_strictness(strictness);
		} catch (e) {
			throw new Error(`Failed to set counterpoint strictness: ${e}`);
		}
	}

	async setExplicitIntervalMap(degreeOffsets: number[][], fallbackOffsets: number[]): Promise<void> {
		this.ensureInit();
		try {
			await this.panicAllNotesOff();
			engine.set_explicit_interval_map(JSON.stringify({
				degree_offsets: degreeOffsets,
				fallback_offsets: fallbackOffsets
			}));
			this.syncCompanionGlobal();
		} catch (e) {
			throw new Error(`Failed to set explicit interval map: ${e}`);
		}
	}

	// -- Companion / CanonLane (#3) -- WASM surface stub
	// Companion lives in the Tauri router thread; the browser/WASM
	// surface has no companion infrastructure yet. These methods are
	// no-ops or return defaults so the shared store doesn't blow up
	// when running in browser mode.

	async companionSetEnabled(enabled: boolean): Promise<void> {
		if (!companion) return;
		if (!enabled) await this.panicAllNotesOff();
		companion.set_enabled(enabled);
	}

	async companionIsEnabled(): Promise<boolean> {
		if (!companion) return false;
		try {
			return Boolean(companion.is_enabled());
		} catch {
			return false;
		}
	}

	async companionSetGlobalHoldMode(holdMode: HoldMode): Promise<void> {
		if (!companion) return;
		try {
			companion.set_global_hold_mode(JSON.stringify(holdMode));
		} catch (e) {
			console.warn('[wasm-adapter] companionSetGlobalHoldMode failed:', e);
		}
	}

	async setPhraseGapBeats(beats: number): Promise<void> {
		if (!companion) return;
		companion.set_phrase_gap(beats);
	}

	async getPhraseState(): Promise<PhraseState> {
		if (!companion) return mapPhraseState(null);
		return mapPhraseState(JSON.parse(companion.phrase_state()));
	}

	async canonSetEnabled(enabled: boolean): Promise<void> {
		if (companion) companion.configure_canon(JSON.stringify({ enabled }));
	}

	async canonSetDelay(_beats: number): Promise<void> {
		// Single-voice setters are unused in the v1.2.0 multi-voice UI;
		// keep as no-op (canonSetVoices is the authoritative path).
	}

	async canonSetTranspose(_degrees: number): Promise<void> {
		// Same as canonSetDelay — unused in v1.2.0.
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
		if (!companion) return;
		try {
			companion.configure_canon(JSON.stringify({ voices }));
		} catch (e) {
			console.warn('[wasm] canonSetVoices failed:', e);
		}
	}

	async counterpointSetConfig(config: {
		enabled?: boolean;
		species?: string;
		transpose_degrees?: number;
		prefer_above?: boolean;
		phrase_aware?: boolean;
	}): Promise<void> {
		if (!companion) return;
		try {
			companion.configure_counterpoint(JSON.stringify(config));
		} catch (e) {
			console.warn('[wasm] counterpointSetConfig failed:', e);
		}
	}

	async canonConfigure(partial: Record<string, unknown>): Promise<void> {
		if (!companion) return;
		try {
			companion.configure_canon(JSON.stringify(partial));
		} catch (e) {
			console.warn('[wasm] canonConfigure failed:', e);
		}
	}

	async counterpointConfigure(partial: Record<string, unknown>): Promise<void> {
		if (!companion) return;
		try {
			companion.configure_counterpoint(JSON.stringify(partial));
		} catch (e) {
			console.warn('[wasm] counterpointConfigure failed:', e);
		}
	}

	async patternConfigure(
		laneId: 'pattern_low' | 'pattern_counter',
		partial: Record<string, unknown>
	): Promise<void> {
		if (!companion) return;
		try {
			companion.configure_pattern(laneId, JSON.stringify(partial));
		} catch (e) {
			console.warn(`[wasm] ${laneId} configuration failed:`, e);
			throw e;
		}
	}

	async patternState(
		laneId: 'pattern_low' | 'pattern_counter'
	): Promise<Record<string, unknown> | null> {
		if (!companion) return null;
		try {
			const state = JSON.parse(companion.pattern_state(laneId));
			return state !== null && typeof state === 'object'
				? (state as Record<string, unknown>)
				: null;
		} catch (e) {
			console.warn(`[wasm] ${laneId} state read failed:`, e);
			return null;
		}
	}

	async counterpointState(): Promise<{
		enabled: boolean;
		species: string;
		transpose_degrees: number;
		prefer_above: boolean;
		phrase_aware?: boolean;
	} | null> {
		if (!companion) return null;
		try {
			return JSON.parse(companion.counterpoint_state());
		} catch {
			return null;
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
		// Same as counterpointState — state is JS-side authoritative.
		return null;
	}

	/** Current MIDI permission state. */
	get midiPermissionState(): MidiPermissionState {
		return this._midiPermissionState;
	}

	/**
	 * Return cached `MIDIAccess` without prompting the user.
	 *
	 * `navigator.requestMIDIAccess()` is gated behind a user gesture in
	 * Chromium-based browsers — calling it at page load fails silently and
	 * leaves the device list empty. The actual prompt lives in
	 * `requestMidiPermission()`, invoked from a click handler.
	 */
	private ensureMidiAccess(): MIDIAccess | null {
		return this.midiAccess;
	}

	/**
	 * Trigger the Web MIDI permission prompt. Must be called from inside a
	 * user-gesture handler (click, keydown). Idempotent — safe to call after
	 * grant or denial; reports the resulting state via the return value.
	 */
	async requestMidiPermission(): Promise<MidiPermissionState> {
		if (typeof navigator === 'undefined' || !('requestMIDIAccess' in navigator)) {
			this._midiPermissionState = 'unsupported';
			return this._midiPermissionState;
		}
		if (this.midiAccess) {
			this._midiPermissionState = 'granted';
			return this._midiPermissionState;
		}
		try {
			this.midiAccess = await navigator.requestMIDIAccess();
			this._midiPermissionState = 'granted';
		} catch {
			this._midiPermissionState = 'denied';
		}
		return this._midiPermissionState;
	}

	async listMidiInputs(): Promise<MidiDevice[]> {
		const access = this.ensureMidiAccess();
		if (!access) return [];
		const devices: MidiDevice[] = [];
		let index = 0;
		access.inputs.forEach((input) => {
			devices.push({ index: index++, name: input.name ?? `Input ${index}` });
		});
		return devices;
	}

	async listMidiOutputs(): Promise<MidiDevice[]> {
		const access = this.ensureMidiAccess();
		if (!access) return [];
		const devices: MidiDevice[] = [];
		let index = 0;
		access.outputs.forEach((output) => {
			devices.push({ index: index++, name: output.name ?? `Output ${index}` });
		});
		return devices;
	}

	async refreshMidiDevices(): Promise<void> {
		if (this._midiPermissionState !== 'granted') return;
		// Permission was already granted, so the browser will not re-prompt.
		// Re-acquire so newly connected devices show up; if the user revoked
		// permission via browser settings since the last grant, fall back.
		try {
			this.midiAccess = await navigator.requestMIDIAccess();
		} catch {
			this.midiAccess = null;
			this._midiPermissionState = 'denied';
		}
	}

	async startRouting(inputIdx: number, outputIndices: number[]): Promise<void> {
		this.ensureInit();

		// --- Guitar audio input mode ---
		if (inputIdx === GUITAR_AUDIO_SENTINEL) {
			await this.enqueueGuitarCaptureOperation(() =>
				this.startGuitarCapture(outputIndices)
			);
			return;
		}

		const access = this.ensureMidiAccess();
		if (!access) {
			// No Web MIDI (permission not granted, or unsupported browser):
			// still allow "running" for virtual/keyboard input.
			this._isRunning = true;
			return;
		}

		// Resolve input device
		const inputs = Array.from(access.inputs.values());
		if (inputIdx >= 0 && inputIdx < inputs.length) {
			this.activeInput = inputs[inputIdx];
		}

		// Resolve output devices
		const outputs = Array.from(access.outputs.values());
		this.activeOutputs = outputIndices
			.filter((i) => i >= 0 && i < outputs.length)
			.map((i) => outputs[i]);

		// Wire up MIDI message handler
		if (this.activeInput) {
			const self = this;
			const outs = this.activeOutputs;
			this.activeInput.onmidimessage = (event: MIDIMessageEvent) => {
				if (!event.data || event.data.length < 2) return;
				const status = event.data[0] & 0xf0;
				const channel = event.data[0] & 0x0f;
				const note = event.data[1];
				const velocity = event.data.length > 2 ? event.data[2] : 0;

				let resultNotes: number[] = [];

				if (status === 0x90 && velocity > 0) {
					try {
						resultNotes = engine.note_on_channel(note, channel);
						const voices = self.tunedVoices(resultNotes);
						for (let i = 0; i < voices.length; i++) {
							const voice = voices[i];
							if (outs.length > 0) {
								outs[i % outs.length].send([0x90 | channel, voice.note, velocity]);
							}
							// External MIDI remains ordinary note bytes; Contrapunk's
							// AudioWorklet receives the exact native tuning frequency.
							embedAudio.noteOn(
								voice.note,
								velocity,
								undefined,
								voice.role === 'input' ? 0 : 1,
								voice.frequencyHz,
								voice.slot,
								resolveSlide(self._slideConfig, { role: voice.role, voice: voice.slot })
							);
						}
						// Feed the player input to the Companion so canon +
						// counterpoint lanes fire delayed / subdivided
						// emissions. Without this hook the v1.2.0 web build
						// would only fire companion voices for the
						// Computer-Keyboard virtual input path, not real
						// MIDI hardware.
						if (companion) {
							try {
								const opsJson = companion.on_note_on(note, velocity, channel);
								self.dispatchOpsJson(opsJson);
								logCompanionEvent('note_on (midi)', {
									player_note: note,
									velocity,
									immediate_ops: self.safeParse(opsJson)
								});
							} catch {
								/* best-effort */
							}
						}
					} catch {
						/* give up on this note */
					}
				} else if (status === 0x80 || (status === 0x90 && velocity === 0)) {
					try {
						resultNotes = engine.note_off_channel(note, channel);
						const voices = self.tunedVoices(resultNotes);
						for (let i = 0; i < voices.length; i++) {
							const voice = voices[i];
							if (outs.length > 0) {
								outs[i % outs.length].send([0x80 | channel, voice.note, 0]);
							}
							embedAudio.noteOff(
								voice.note,
								undefined,
								voice.role === 'input' ? 0 : 1,
								voice.slot
							);
						}
						if (companion) {
							try {
								const opsJson = companion.on_note_off(note, channel);
								self.dispatchOpsJson(opsJson);
								logCompanionEvent('note_off (midi)', {
									player_note: note,
									immediate_ops: self.safeParse(opsJson)
								});
							} catch {
								/* best-effort */
							}
						}
					} catch {
						/* give up */
					}
				} else {
					if (status === 0xb0 && companion) {
						try {
							self.dispatchOpsJson(companion.on_cc(note, velocity, channel));
							if (note === 120 || note === 123) companion.reset_runtime();
						} catch {
							/* best-effort */
						}
					}
					if (status === 0xb0 && note === 64) embedAudio.setSustainPedal(velocity >= 64);
					if (status === 0xb0 && (note === 120 || note === 123)) embedAudio.allNotesOff();
					// Pass through other MIDI messages (CC, pitch bend, etc.)
					for (const output of outs) {
						output.send(Array.from(event.data));
					}
				}
			};
		}

		this._isRunning = true;
		// Apply current detune on start
		if (this._detuneCents !== 0) {
			this.sendPitchBend();
		}
	}

	/**
	 * Start guitar audio capture with pitch detection.
	 * Routes detected notes through the harmony engine via injectNoteOn/Off.
	 */
	private async startGuitarCapture(outputIndices: number[]): Promise<void> {
		if (this.guitarCapture) await this.stopRoutingNow();
		this._guitarOutputIndices = outputIndices.slice();
		// Resolve MIDI outputs for sending harmony notes
		const access = this.ensureMidiAccess();
		if (access) {
			const outputs = Array.from(access.outputs.values());
			this.activeOutputs = outputIndices
				.filter((i) => i >= 0 && i < outputs.length)
				.map((i) => outputs[i]);
		}

		const capture = new GuitarAudioCapture();
		this.applyGuitarConfig(capture);
		const self = this;

		// Set initial gate values from store (noise gate only)
		capture.noiseGateThreshold = guitar.noiseGateThreshold;
		capture.noiseGateEnabled = guitar.noiseGateEnabled;
		capture.clarityGateEnabled = false;

		// Read device/channel directly from guitar store (not stale adapter properties)
		// because the user may have changed settings after the last syncDevice() call
		const deviceId = guitar.selectedDeviceId || this._guitarDeviceId;
		const channelIndex = Math.max(0, guitar.selectedChannel - 1); // 1-indexed → 0-indexed
		console.log(`[wasm] startGuitarCapture: device='${deviceId}' channel=${channelIndex} (store.selectedChannel=${guitar.selectedChannel})`);

		try {
			await capture.start(
			deviceId,
			channelIndex,
			{
				onNoteOn(note: number, velocity: number) {
					self.injectNoteOn(note, velocity).catch(() => {});
				},
				onNoteOff(note: number) {
					self.injectNoteOff(note).catch(() => {});
				},
				// Harmony injection emits browser MIDI on channel 0, so expression
				// follows that shipping channel rather than the detector's MPE channel.
				onPitchBend(_channel, cents) {
					const value = self.centsToPitchBend(cents);
					self.sendGuitarMidi([0xe0, value & 0x7f, (value >> 7) & 0x7f]);
				},
				onMidiPitchBend(_channel, value) {
					self.sendGuitarMidi([0xe0, value & 0x7f, (value >> 7) & 0x7f]);
				},
				onCC(_channel, controller, value) {
					self.sendGuitarMidi([0xb0, controller & 0x7f, value & 0x7f]);
				},
				onChannelPressure(_channel, pressure) {
					self.sendGuitarMidi([0xd0, pressure & 0x7f]);
				},
				onDetection(info) {
					// Sync noise gate + actual channel from capture every frame.
					capture.noiseGateThreshold = guitar.noiseGateThreshold;
					capture.noiseGateEnabled = guitar.noiseGateEnabled;
					guitar.activeChannel = capture.actualChannel + 1;

					// Push signal data for graphs
					guitar.pushSignalFrame(info.rms, info.clarity);

					if (info.frequency !== null) {
						guitar.currentNote = info.noteName;
						guitar.confidence = Math.round(info.clarity * 100);
						guitar.velocity = Math.round(info.rms * 800);
					} else {
						guitar.currentNote = '';
						guitar.confidence = 0;
					}
				}
			},
			this._guitarConfig.latencyMs
			);
		} catch (error) {
			await capture.stop();
			throw error;
		}

		// Publish only a capture whose AudioContext and stream started successfully.
		this.guitarCapture = capture;
		guitar.detecting = true;
		guitar.activeChannel = capture.actualChannel + 1; // 1-indexed for display
		this._isRunning = true;
		this.startNotePolling();
	}

	private async stopRoutingNow(): Promise<void> {
		// Stop guitar audio capture if active
		if (this.guitarCapture) {
			await this.guitarCapture.stop();
			this.guitarCapture = null;
			guitar.detecting = false;
			guitar.currentNote = '';
			guitar.confidence = 0;
			guitar.velocity = 0;
		}

		// Send All-Notes-Off (CC 123) to every active output to prevent stuck notes
		for (const output of this.activeOutputs) {
			try {
				output.send([0xb0, 123, 0]); // CC#123 = All Notes Off
			} catch {
				// Output may already be disconnected
			}
		}

		// Clear engine's tracked note state
		if (engine) {
			try {
				engine.clear_notes();
			} catch {
				// Engine may not be initialized
			}
		}

		embedAudio.allNotesOff();
		clearCompanionNotes();
		try {
			companion?.reset_runtime?.();
		} catch {
			/* Companion may not be initialized */
		}

		// Disconnect MIDI input handler
		if (this.activeInput) {
			this.activeInput.onmidimessage = null;
			this.activeInput = null;
		}
		this.activeOutputs = [];
		this._isRunning = false;
		this.stopNotePolling();
	}

	async stopRouting(): Promise<void> {
		await this.enqueueGuitarCaptureOperation(() => this.stopRoutingNow());
	}

	// Per-voice output routing is native-only for now. The browser path
	// has no backend to route to — tracked in the in-memory table below
	// so UI reads/writes round-trip consistently until the WASM engine
	// gains a synth + MIDI dispatcher of its own.
	private _voiceOutputs = new Map<VoiceRouteId, VoiceOutputTarget>();

	async setVoiceOutput(route: VoiceRouteId, target: VoiceOutputTarget): Promise<void> {
		if (target.kind === 'synth') this._voiceOutputs.delete(route);
		else this._voiceOutputs.set(route, target);
	}

	async setAllVoiceOutputsToSynth(_enabled: boolean): Promise<void> {}

	async getVoiceOutputs(): Promise<VoiceOutputAssignment[]> {
		return [...this._voiceOutputs].map(([route, target]) => ({ route, target }));
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
		for (const output of this.activeOutputs) {
			try {
				output.send([0xb0, 120, 0]);
				output.send([0xb0, 123, 0]);
			} catch {
				/* disconnected output */
			}
		}
		embedAudio.allNotesOff();
		clearCompanionNotes();
		try {
			engine?.clear_notes?.();
			companion?.reset_runtime?.();
		} catch {
			/* backend may not be initialized */
		}
	}

	onPluginParamsUpdate(_callback: () => void): () => void {
		return () => {};
	}

	async injectNoteOn(note: number, velocity?: number): Promise<number[]> {
		this.ensureInit();
		try {
			const result = engine.note_on(note);
			const voices = this.tunedVoices(result ?? [note]);
			const vel = velocity ?? 100;
			for (let i = 0; i < voices.length; i++) {
				const voice = voices[i];
				if (this.activeOutputs.length > 0) {
					this.activeOutputs[i % this.activeOutputs.length].send([0x90, voice.note, vel]);
				}
				embedAudio.noteOn(
					voice.note,
					vel,
					undefined,
					voice.role === 'input' ? 0 : 1,
					voice.frequencyHz,
					voice.slot,
					resolveSlide(this._slideConfig, { role: voice.role, voice: voice.slot })
				);
			}
			// Feed the player input to the Companion so the Canon and
			// Counterpoint lanes can fire delayed / subdivided
			// emissions. Immediate-fire ops (Species 1 at delay 0) come
			// back here; longer-delayed ones drain via the tick loop.
			if (companion) {
				try {
					const opsJson = companion.on_note_on(note, vel, 0);
					this.dispatchOpsJson(opsJson);
					logCompanionEvent('note_on (virtual)', {
						player_note: note,
						velocity: vel,
						immediate_ops: this.safeParse(opsJson)
					});
				} catch {
					/* ignore — companion is best-effort in browser */
				}
			}
			return voices.map((voice) => voice.note);
		} catch {
			return [note];
		}
	}

	private safeParse(json: string): unknown {
		try {
			return JSON.parse(json);
		} catch {
			return json;
		}
	}

	async injectNoteOff(note: number): Promise<number[]> {
		this.ensureInit();
		try {
			const result = engine.note_off(note);
			const voices = this.tunedVoices(result ?? [note]);
			for (let i = 0; i < voices.length; i++) {
				const voice = voices[i];
				if (this.activeOutputs.length > 0) {
					this.activeOutputs[i % this.activeOutputs.length].send([0x80, voice.note, 0]);
				}
				embedAudio.noteOff(
					voice.note,
					undefined,
					voice.role === 'input' ? 0 : 1,
					voice.slot
				);
			}
			if (companion) {
				try {
					const opsJson = companion.on_note_off(note, 0);
					this.dispatchOpsJson(opsJson);
					logCompanionEvent('note_off (virtual)', {
						player_note: note,
						immediate_ops: this.safeParse(opsJson)
					});
				} catch {
					/* ignore */
				}
			}
			return voices.map((voice) => voice.note);
		} catch {
			return [note];
		}
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
				lastBorrowedFrom: raw?.last_borrowed_from ?? '',
				currentKey: engine.current_key?.() ?? 'C',
				canonNotes: activeNotes(activeCanonNotes),
				counterpointNotes: activeNotes(activeCounterpointNotes),
				phrase: await this.getPhraseState()
			};
		} catch {
			return {
				inputNotes: [],
				harmonyNotes: [],
				borrowedNotes: [],
				chordName: '',
				lastBorrowedFrom: '',
				currentKey: 'C',
				canonNotes: activeNotes(activeCanonNotes),
				counterpointNotes: activeNotes(activeCounterpointNotes),
				phrase: mapPhraseState(null)
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

	onKnobCcRaw(_callback: (cc: number, value: number) => void): () => void {
		// Browser/wasm path: raw MIDI CCs aren't currently forwarded
		// from the Web MIDI input handler. Could be wired up in a
		// future iteration to enable MIDI Learn in the browser; for
		// now, no-op.
		return () => {};
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

	private tunedVoices(
		notes: ArrayLike<number>
	): Array<{ note: number; frequencyHz: number; role: SlideRole; slot: number }> {
		const midiNotes = Array.from(notes);
		let frequencies: number[] = [];
		let portMap: number[] = [];
		try {
			frequencies = Array.from(
				engine.tuned_frequencies(new Uint8Array(midiNotes)) as Float32Array
			);
			portMap = Array.from(engine.last_port_map?.() ?? []);
		} catch {
			// The Rust bridge validates every frame; Standard is the safe fallback.
		}
		return midiNotes
			.map((note, index) => ({
				note,
				frequencyHz: frequencies[index] ?? 440 * 2 ** ((note - 69) / 12),
				role: index === 0 ? ('input' as const) : ('harmony' as const),
				slot: index === 0 ? 0 : (portMap[index] ?? index)
			}))
			.sort((a, b) => a.note - b.note);
	}

	/**
	 * Sort notes ascending so lowest note always routes to output 0 (bass),
	 * and each higher voice stays on its consistent output slot.
	 */
	private sortVoices(notes: number[]): number[] {
		return [...notes].sort((a, b) => a - b);
	}

	/**
	 * Convert cents offset to a 14-bit MIDI pitch bend value.
	 * Center = 8192, range depends on pitchBendRangeSemitones (default ±2 = 200 cents).
	 */
	private centsToPitchBend(cents: number): number {
		const maxCents = this.pitchBendRangeSemitones * 100;
		const normalized = Math.max(-1, Math.min(1, cents / maxCents));
		return Math.round(8192 + normalized * 8191);
	}

	/**
	 * Send pitch bend to all active outputs on channel 0.
	 */
	private sendPitchBend(): void {
		const bend = this.centsToPitchBend(this._detuneCents);
		const lsb = bend & 0x7f;
		const msb = (bend >> 7) & 0x7f;
		for (const output of this.activeOutputs) {
			output.send([0xe0, lsb, msb]);
		}
	}

	async listAudioDevices(): Promise<string[]> {
		if (typeof navigator === 'undefined' || !navigator.mediaDevices) {
			return [];
		}
		try {
			// Request permission so labels are populated
			const stream = await navigator.mediaDevices.getUserMedia({ audio: true });
			stream.getTracks().forEach((t) => t.stop());

			const allDevices = await navigator.mediaDevices.enumerateDevices();
			return allDevices
				.filter((d) => d.kind === 'audioinput')
				.map((d) => d.label || `Audio Input ${d.deviceId.slice(0, 8)}`);
		} catch {
			return [];
		}
	}

	async setGuitarDevice(deviceName: string, channel: number): Promise<void> {
		// Store for use when startRouting is called with guitar sentinel
		// Channel arrives 0-indexed from syncDevice()
		this._guitarDeviceId = deviceName;
		this._guitarChannel = Math.max(0, channel);
	}

	async setGuitarConfig(config: GuitarConfig): Promise<void> {
		const latencyChanged = config.latencyMs !== this._guitarConfig.latencyMs;
		this._guitarConfig = { ...config };
		if (!this.guitarCapture) return;

		if (!latencyChanged) {
			this.applyGuitarConfig(this.guitarCapture);
			return;
		}

		await this.enqueueGuitarCaptureOperation(async () => {
			if (!this.guitarCapture) return;
			const outputIndices = this._guitarOutputIndices.slice();
			await this.stopRoutingNow();
			await this.startGuitarCapture(outputIndices);
		});
	}

	async getCalibrationStatus() {
		// No persistence layer in WASM yet; report a default-only profile.
		return { existsOnDisk: false, path: '', version: 1, sampleCounts: [0, 0, 0, 0, 0, 0] };
	}

	async loadCalibrationProfile() {
		return this.getCalibrationStatus();
	}

	async saveCalibrationProfile(_profileJson: string) {
		return this.getCalibrationStatus();
	}

	async deleteCalibrationProfile() {
		return this.getCalibrationStatus();
	}

	async getLastPortMap(): Promise<number[]> {
		this.ensureInit();
		return Array.from(engine.last_port_map?.() ?? []);
	}

	/**
	 * Set global detune in cents and send pitch bend to all outputs.
	 * Assumes standard ±2 semitone pitch bend range in your DAW.
	 */
	setDetune(cents: number): void {
		this._detuneCents = cents;
		if (this._isRunning) {
			this.sendPitchBend();
		}
	}

	getDetune(): number {
		return this._detuneCents;
	}

	// -- Transport + metronome (browser implementation, issue #55) --
	//
	// JS-side clock + Web Audio click. Native parity isn't sample-accurate
	// — setInterval drift over long playback isn't corrected — but it's
	// fine for a demo metronome and matches what users expect from the
	// /app embed on the marketing site.
	//
	// Click frequencies + duration match Rust's audio_clock.rs constants
	// so native + web sound identical.

	private _transport = {
		running: false,
		bpm: 120,
		beatsPerBar: 4,
		beatUnit: 4,
		beatInBar: 0,
		totalBeat: 0,
		bar: 0,
		metronomeEnabled: false
	};
	private _clockTimer: ReturnType<typeof setInterval> | null = null;

	private ensureAudioContext(): AudioContext | null {
		return embedAudio.getAudioContext();
	}

	private playClick(downbeat: boolean) {
		const ctx = this.ensureAudioContext();
		if (!ctx) return;
		const key = downbeat ? 255 : 254;
		embedAudio.noteOn(key, 38, ctx.currentTime, 0, downbeat ? 900 : 600);
		embedAudio.noteOff(key, ctx.currentTime + 0.015, 0);
	}

	private tick() {
		// Increment counters first so beat 0 fires at start.
		const t = this._transport;
		// Notify the store + render the pip / label first, then synthesize
		// the click. Order doesn't strictly matter for ear-correctness
		// since the click is short, but doing the UI update first means
		// visual + audio land together on the same frame.
		transport.applyBeatUpdate({
			totalBeat: t.totalBeat,
			beatInBar: t.beatInBar,
			bar: t.bar,
			bpm: t.bpm,
			running: t.running
		});
		if (t.metronomeEnabled) {
			this.playClick(t.beatInBar === 0);
		}
		// Advance.
		t.totalBeat += 1;
		t.beatInBar += 1;
		if (t.beatInBar >= t.beatsPerBar) {
			t.beatInBar = 0;
			t.bar += 1;
		}
	}

	private startClock() {
		this.stopClock();
		const intervalMs = 60_000 / this._transport.bpm;
		this._clockTimer = setInterval(() => this.tick(), intervalMs);
		// Fire one beat immediately so the UI shows a pip on the first
		// beat without waiting for the interval to elapse.
		this.tick();
	}

	private stopClock() {
		if (this._clockTimer !== null) {
			clearInterval(this._clockTimer);
			this._clockTimer = null;
		}
	}

	async getTransportState(): Promise<TransportState> {
		return {
			running: this._transport.running,
			bpm: this._transport.bpm,
			beatsPerBar: this._transport.beatsPerBar,
			beatUnit: this._transport.beatUnit,
			sampleRate: 48_000,
			samplePos: 0,
			beatPosition: 0,
			bar: this._transport.bar,
			metronomeEnabled: this._transport.metronomeEnabled
		};
	}

	async transportPlay(): Promise<void> {
		if (this._transport.running) return;
		// User gesture is implicit — play() is called from a button click,
		// which satisfies the browser autoplay policy.
		const ctx = this.ensureAudioContext();
		if (ctx && ctx.state === 'suspended') {
			try {
				await ctx.resume();
			} catch {
				// suspended → resume can fail if not in a user-gesture
			}
		}
		this._transport.running = true;
		this._transport.beatInBar = 0;
		this._transport.totalBeat = 0;
		this._transport.bar = 0;
		this.startClock();
	}

	async transportStop(): Promise<void> {
		this._transport.running = false;
		this.stopClock();
		// A frozen beat clock cannot mature future Canon NoteOffs.
		// Silence outputs and clear every Companion queue before stop.
		await this.panicAllNotesOff();
		// Push a final state update so pips drop the active state.
		transport.applyBeatUpdate({
			totalBeat: this._transport.totalBeat,
			beatInBar: this._transport.beatInBar,
			bar: this._transport.bar,
			bpm: this._transport.bpm,
			running: false
		});
	}

	async transportReset(): Promise<void> {
		await this.panicAllNotesOff();
		this._transport.beatInBar = 0;
		this._transport.totalBeat = 0;
		this._transport.bar = 0;
		transport.applyBeatUpdate({
			totalBeat: 0,
			beatInBar: 0,
			bar: 0,
			bpm: this._transport.bpm,
			running: this._transport.running
		});
	}

	async setBpm(bpm: number): Promise<void> {
		const clamped = Math.max(20, Math.min(400, bpm));
		this._transport.bpm = clamped;
		// If the clock is running, restart it so the new interval takes
		// effect on the next tick.
		if (this._transport.running) {
			this.startClock();
		}
	}

	async setTimeSignature(beatsPerBar: number, beatUnit: number): Promise<void> {
		this._transport.beatsPerBar = Math.max(1, Math.min(16, beatsPerBar));
		this._transport.beatUnit = beatUnit;
		// Reset beatInBar so we don't desync the pip animation.
		this._transport.beatInBar = 0;
	}

	async setMetronomeEnabled(enabled: boolean): Promise<void> {
		this._transport.metronomeEnabled = enabled;
		// If user enables the click without first hitting Play, we still
		// need an AudioContext primed so the next tick can fire a click.
		if (enabled) this.ensureAudioContext();
	}

	// -- Fixed Elixir sine synth --

	async getSynthState() {
		return {
			enabled: this._synthEnabled,
			masterGain: this._synthMasterGain,
			mixGains: [...this._synthMixGains]
		};
	}
	async setSynthEnabled(enabled: boolean): Promise<void> {
		this._synthEnabled = enabled;
		embedAudio.setEnabled(enabled);
	}
	async setSynthMasterGain(value: number): Promise<void> {
		this._synthMasterGain = Math.max(0, Math.min(1, value));
		embedAudio.setMasterGain(this._synthMasterGain);
	}
	async setSynthMixGain(group: number, value: number): Promise<void> {
		if (group < 0 || group >= this._synthMixGains.length) return;
		this._synthMixGains[group] = Math.max(0, Math.min(1, value));
		embedAudio.setRoleGain(group, this._synthMixGains[group]);
	}

	// Native-only FX remain in the shared adapter contract.
	async getReverbState() {
		return { enabled: false, mix: 0, roomSize: 0, damping: 0 };
	}
	async setReverbEnabled(_enabled: boolean): Promise<void> {}
	async setReverbMix(_value: number): Promise<void> {}
	async setReverbRoomSize(_value: number): Promise<void> {}
	async setReverbDamping(_value: number): Promise<void> {}
	async getDelayState() {
		return {
			enabled: false,
			mix: 0,
			timeMs: 0,
			feedback: 0,
			syncEnabled: false,
			subdivision: '1/8' as const
		};
	}
	async setDelayEnabled(_enabled: boolean): Promise<void> {}
	async setDelayMix(_value: number): Promise<void> {}
	async setDelayTimeMs(_ms: number): Promise<void> {}
	async setDelayFeedback(_value: number): Promise<void> {}
	async setDelaySyncEnabled(_enabled: boolean): Promise<void> {}
	async setDelaySubdivision(_subdivision: string): Promise<void> {}

	// -- Chain topology + CLAP (no native host in browser; stubs) --

	async listChainBlocks() {
		return [];
	}
	async removeChainBlock(_index: number): Promise<void> {}
	async clearChain(): Promise<void> {}
	async listClapPlugins() {
		return [];
	}
	async addClapPluginToChain(_path: string) {
		return { pluginId: 0, name: '', path: '', hasGui: false };
	}
	async openPluginGui(_pluginId: number): Promise<void> {}
	async getPluginGuiSize(_pluginId: number): Promise<{ width: number; height: number } | null> {
		return null;
	}
	async openPluginGuiEmbedded(
		_pluginId: number,
		_x: number,
		_y: number,
		_w: number,
		_h: number
	): Promise<void> {}
	async setPluginGuiFrame(
		_pluginId: number,
		_x: number,
		_y: number,
		_w: number,
		_h: number
	): Promise<void> {}
	async closePluginGui(_pluginId: number): Promise<void> {}
	async removePlugin(_pluginId: number): Promise<void> {}

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
