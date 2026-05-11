/**
 * Platform Adapter Layer - Shared Type Definitions
 *
 * Defines the ContrapunkAdapter interface and all related types.
 * Components import from this module and never call Tauri or WASM directly.
 */

/** Represents a MIDI device (input or output). */
/** Maximum number of voices the app exposes. Must match MAX_VOICES in
 *  `src-tauri/src/state.rs`. */
export const MAX_VOICES = 8;

/** Per-voice output destination. Shape mirrors the Rust `VoiceOutputTarget`
 *  enum with `#[serde(tag = "kind", rename_all = "snake_case")]` — see
 *  `src-tauri/src/state.rs` and the `voice_output_target_json_shape_is_tagged`
 *  test in `src-tauri/src/commands/routing.rs`.
 *
 *    - `synth`     — internal synth only, skip external MIDI (DEFAULT)
 *    - `midi_port` — specific MIDI port only, skip the internal synth
 *    - `off`       — voice is silent
 */
export type VoiceOutputTarget =
	| { kind: 'synth' }
	| { kind: 'midi_port'; port: number }
	| { kind: 'off' };

export interface MidiDevice {
	index: number;
	name: string;
}

/**
 * Browser MIDI permission state.
 *
 * `idle`        — never requested. Triggering a request requires a user gesture.
 * `granted`     — user accepted; device enumeration works.
 * `denied`      — user rejected. Recoverable only via browser settings; UI
 *                 should offer a "Try again" affordance.
 * `unsupported` — browser does not implement Web MIDI (Firefox historically).
 *                 Suggest the desktop app.
 */
export type MidiPermissionState = 'idle' | 'granted' | 'denied' | 'unsupported';

/** Snapshot of the harmony engine configuration. */
export interface EngineState {
	key: string;
	mode: string;
	modeNumber: number;
	scaleMode: string;
	octaveMode: string;
	voiceLeadingEnabled: boolean;
	voiceLeadingStyle: string;
	interchangeEnabled: boolean;
	interchangeRange: number;
	voicePosition: number;
	voiceCount: number;
	autoKey: boolean;
	isRunning: boolean;
	counterpointSpecies: string;
	counterpointStrictness: string;
}

/** Real-time note state emitted during MIDI routing. */
export interface NoteState {
	inputNotes: number[];
	harmonyNotes: number[];
	borrowedNotes: number[];
	chordName: string;
	lastBorrowedFrom: string;
	currentKey: string;
}

/** Guitar DSP pipeline configuration sent to the backend. */
export interface GuitarConfig {
	latencyMs: number;
	gain: number;
	stringConfidence: number;
	bends: boolean;
	legato: boolean;
	slides: boolean;
	vibrato: boolean;
}

/** Preset metadata. */
export interface Preset {
	index: number;
	name: string;
	persona: string;
	genre: string;
	isBuiltin: boolean;
}

/** Snapshot of built-in synth parameters. */
export interface SynthState {
	enabled: boolean;
	waveform: number;
	attackMs: number;
	decayMs: number;
	sustain: number;
	releaseMs: number;
	cutoffHz: number;
	resonance: number;
	masterGain: number;
}

/** Snapshot of built-in reverb parameters. */
export interface ReverbState {
	enabled: boolean;
	mix: number;
	roomSize: number;
	damping: number;
}

/** Snapshot of built-in delay parameters. */
export interface DelayState {
	enabled: boolean;
	mix: number;
	timeMs: number;
	feedback: number;
}

/** One block in the live audio chain. */
export interface ChainBlock {
	typeId: string;
	name: string;
}

/** Descriptor for a discovered CLAP plugin on disk. */
export interface ClapPluginDescriptor {
	id: string;
	name: string;
	vendor: string;
	version: string;
	path: string;
}

/** Returned from addClapPluginToChain — plugin is live, has a stable id for GUI control. */
export interface AddedPlugin {
	pluginId: number;
	name: string;
	path: string;
	hasGui: boolean;
}

/** Snapshot of transport / clock state. */
export interface TransportState {
	running: boolean;
	bpm: number;
	beatsPerBar: number;
	beatUnit: number;
	sampleRate: number;
	samplePos: number;
	beatPosition: number;
	bar: number;
	metronomeEnabled: boolean;
}

/**
 * Unified adapter interface for communicating with the Rust backend.
 *
 * Svelte components call these methods without knowing whether the
 * backend is reached via Tauri IPC (desktop) or WASM direct calls (browser).
 */
export interface ContrapunkAdapter {
	// -- Initialization --

	/** Initialize the backend (WASM init or Tauri readiness check). */
	init(): Promise<void>;

	// -- Engine state --

	/** Get a snapshot of the current engine configuration. */
	getEngineState(): Promise<EngineState>;

	/** Set the musical key (e.g. "C", "Db", "F#"). */
	setKey(key: string): Promise<void>;

	/** Set the harmony mode (e.g. "PassThrough", "DiatonicThirds"). */
	setMode(mode: string): Promise<void>;

	/** Set the scale mode (e.g. "Ionian", "Dorian", "HarmonicMinor"). */
	setScaleMode(mode: string): Promise<void>;

	/** Set the octave mode (e.g. "None", "Spread", "Mirror"). */
	setOctaveMode(mode: string): Promise<void>;

	/** Continuous octave-spread coefficient applied to Spread / Split modes.
	 *  Range [0.0, 1.0]; 0 = no displacement, 1 = full-octave (legacy)
	 *  displacement. Used by the Performance view's Spread knob for
	 *  smooth audible morphs. */
	setOctaveIntensity(amount: number): Promise<void>;

	/** Configure voice leading (enabled flag + style). */
	setVoiceLeading(enabled: boolean, style: string): Promise<void>;

	/** Configure modal interchange (enabled flag + borrowing range). */
	setInterchange(enabled: boolean, range: number): Promise<void>;

	/** Set the voice position (which output slot carries the melody). */
	setVoicePosition(position: number): Promise<void>;

	/** Set the number of output voices (1 = melody only, 2+ = melody + harmonies). */
	setVoiceCount(count: number): Promise<void>;

	/** Enable or disable auto-key detection. */
	setAutoKey(enabled: boolean): Promise<void>;

	/**
	 * Set the counterpoint species (1-4) used when the harmony mode is
	 * `StrictCounterpoint`. Values: `"Species1"`, `"Species2"`, `"Species3"`,
	 * `"Species4"`.
	 */
	setCounterpointSpecies(species: string): Promise<void>;

	/**
	 * Set the counterpoint strictness (`"Relaxed"` or `"Strict"`) applied
	 * to the counterpoint scoring weights.
	 */
	setCounterpointStrictness(strictness: string): Promise<void>;

	/**
	 * Get per-harmony-voice phase offsets in beats. Length = voiceCount - 1.
	 * Index 0 = first harmony voice above the melody. Issue #8b.
	 */
	voiceOffsets(): Promise<number[]>;

	/**
	 * Set the phase offset (in beats, clamped to [-0.5, 0.5]) for a
	 * specific harmony-voice index. Out-of-range indices are a no-op.
	 * Only Species 2-4 use the offset; Species 1 ignores phase. Issue #8b.
	 */
	setVoiceOffset(voiceIndex: number, offset: number): Promise<void>;

	// -- MIDI devices --

	/**
	 * Browser-only: request Web MIDI permission via a user gesture.
	 *
	 * Adapters that don't gate MIDI behind a gesture (Tauri, Plugin) return
	 * `'granted'` immediately. The browser adapter calls
	 * `navigator.requestMIDIAccess()` and resolves with the resulting state.
	 *
	 * Callers should gate device enumeration on this returning `'granted'`.
	 */
	requestMidiPermission(): Promise<MidiPermissionState>;

	/** Current MIDI permission state. `'granted'` for native paths. */
	readonly midiPermissionState: MidiPermissionState;

	/** List available MIDI input devices. */
	listMidiInputs(): Promise<MidiDevice[]>;

	/** List available MIDI output devices. */
	listMidiOutputs(): Promise<MidiDevice[]>;

	/** Re-enumerate MIDI devices (forces a fresh scan). */
	refreshMidiDevices(): Promise<void>;

	// -- Routing --

	/** Start MIDI routing from the given input to the given outputs. */
	startRouting(inputIdx: number, outputIndices: number[]): Promise<void>;

	/** Stop the currently active MIDI routing. */
	stopRouting(): Promise<void>;

	// -- Per-voice output routing --

	/**
	 * Set the output destination for a single voice by index (0..7).
	 * Each voice can go independently to the internal synth, a specific
	 * external MIDI port, nowhere (off), or fall back to the global
	 * routing mode (use_default).
	 */
	setVoiceOutput(voiceIdx: number, target: VoiceOutputTarget): Promise<void>;

	/**
	 * Get the current voice-output routing table. Array length = MAX_VOICES
	 * (8). Each entry is the destination for that voice index.
	 */
	getVoiceOutputs(): Promise<VoiceOutputTarget[]>;

	// -- Real-time state --

	/**
	 * Subscribe to real-time note updates.
	 * Returns an unsubscribe function.
	 */
	onNoteUpdate(callback: (state: NoteState) => void): () => void;

	/**
	 * Subscribe to raw MIDI CC events. Every Control Change message
	 * received on the active input is forwarded with its raw CC number
	 * and normalized 0..1 value. The Performance view uses this to
	 * implement MIDI Learn — it maps CC numbers to software-knob
	 * indices via a user-configurable table persisted in localStorage.
	 * Browser/wasm/plugin platforms that don't deliver raw MIDI CC
	 * may return a no-op.
	 *
	 * Returns an unsubscribe function.
	 */
	onKnobCcRaw(callback: (cc: number, value: number) => void): () => void;

	// -- Virtual Input (keyboard) --

	/** Inject a Note On event directly (for virtual inputs like computer keyboard). */
	injectNoteOn(note: number, velocity?: number): Promise<number[]>;

	/** Inject a Note Off event directly (for virtual inputs). */
	injectNoteOff(note: number): Promise<number[]>;

	// -- Presets --

	/** List all available presets (builtins + custom). */
	listPresets(): Promise<Preset[]>;

	/** Load a preset by name and apply it to the engine. */
	loadPreset(name: string): Promise<void>;

	/** Save the current engine config as a custom preset. */
	savePreset(name: string): Promise<void>;

	/** Delete a custom preset by name. */
	deletePreset(name: string): Promise<void>;

	// -- Guitar input --

	/** List available audio input devices (via cpal on desktop). */
	listAudioDevices(): Promise<string[]>;

	/** Set the guitar audio input device and channel. */
	setGuitarDevice(deviceName: string, channel: number): Promise<void>;

	/** Set the guitar DSP pipeline configuration. */
	setGuitarConfig(config: GuitarConfig): Promise<void>;

	// -- Detune --

	/** Set global detune in cents (sends pitch bend to all outputs). */
	setDetune(cents: number): void;

	/** Get the current detune value in cents. */
	getDetune(): number;

	// -- Transport (sample-accurate clock) --

	/** Get a snapshot of the current transport state. */
	getTransportState(): Promise<TransportState>;

	/** Start the transport (resume from current position). */
	transportPlay(): Promise<void>;

	/** Stop the transport (freeze at current position). */
	transportStop(): Promise<void>;

	/** Reset sample position to 0. */
	transportReset(): Promise<void>;

	/** Set the tempo in BPM (clamped to [20, 400] on the backend). */
	setBpm(bpm: number): Promise<void>;

	/** Set the time signature (beats per bar × beat unit). */
	setTimeSignature(beatsPerBar: number, beatUnit: number): Promise<void>;

	/** Toggle the audible metronome click. Off by default. */
	setMetronomeEnabled(enabled: boolean): Promise<void>;

	// -- Built-in synth (Tier 1 audio output) --

	getSynthState(): Promise<SynthState>;
	setSynthEnabled(enabled: boolean): Promise<void>;
	setSynthWaveform(value: number): Promise<void>;
	setSynthAttackMs(ms: number): Promise<void>;
	setSynthDecayMs(ms: number): Promise<void>;
	setSynthSustain(level: number): Promise<void>;
	setSynthReleaseMs(ms: number): Promise<void>;
	setSynthCutoffHz(hz: number): Promise<void>;
	setSynthResonance(value: number): Promise<void>;
	setSynthMasterGain(value: number): Promise<void>;

	// -- Built-in FX (reverb) --

	getReverbState(): Promise<ReverbState>;
	setReverbEnabled(enabled: boolean): Promise<void>;
	setReverbMix(value: number): Promise<void>;
	setReverbRoomSize(value: number): Promise<void>;
	setReverbDamping(value: number): Promise<void>;

	getDelayState(): Promise<DelayState>;
	setDelayEnabled(enabled: boolean): Promise<void>;
	setDelayMix(value: number): Promise<void>;
	setDelayTimeMs(ms: number): Promise<void>;
	setDelayFeedback(value: number): Promise<void>;

	// -- Audio chain topology --

	listChainBlocks(): Promise<ChainBlock[]>;
	removeChainBlock(index: number): Promise<void>;
	clearChain(): Promise<void>;

	// -- CLAP plugin host --

	listClapPlugins(): Promise<ClapPluginDescriptor[]>;
	addClapPluginToChain(path: string): Promise<AddedPlugin>;
	openPluginGui(pluginId: number): Promise<void>;
	getPluginGuiSize(pluginId: number): Promise<{ width: number; height: number } | null>;
	openPluginGuiEmbedded(
		pluginId: number,
		x: number,
		y: number,
		width: number,
		height: number
	): Promise<void>;
	setPluginGuiFrame(
		pluginId: number,
		x: number,
		y: number,
		width: number,
		height: number
	): Promise<void>;
	closePluginGui(pluginId: number): Promise<void>;
	removePlugin(pluginId: number): Promise<void>;

	// -- Lifecycle --

	/**
	 * Tear down any background loops, event listeners, or audio contexts
	 * owned by this adapter. Called from the root layout `onDestroy` hook
	 * so hot-reload and navigation don't leak requestAnimationFrame loops.
	 */
	destroy?(): void;
}
