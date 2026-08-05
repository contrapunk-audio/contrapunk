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

/** Per-part output destination. Shape mirrors the Rust `VoiceOutputTarget`
 *  enum with `#[serde(tag = "kind", rename_all = "snake_case")]` — see
 *  `src-tauri/src/state.rs` and the `voice_output_target_json_shape_is_tagged`
 *  test in `src-tauri/src/commands/routing.rs`.
 *
 *    - `synth`     — internal synth only, skip external MIDI (DEFAULT)
 *    - `midi_port` — specific MIDI port only; Tauri uses its system device index
 *    - `off`       — voice is silent
 */
export type VoiceOutputTarget =
	| { kind: 'synth' }
	| { kind: 'midi_port'; port: number }
	| { kind: 'off' };

/** Stable musical identity used by desktop output routing. Live and loop
 * playback intentionally share these IDs. */
export type VoiceRouteId =
	| 'input'
	| `harmony:${number}`
	| `canon:${number}`
	| `counterpoint:${number}`
	| 'pattern_low'
	| 'pattern_counter';

export interface VoiceOutputAssignment {
	route: VoiceRouteId;
	target: VoiceOutputTarget;
}

export function isVoiceRouteId(value: string): value is VoiceRouteId {
	if (value === 'input' || value === 'pattern_low' || value === 'pattern_counter') return true;
	const match = /^(harmony|canon|counterpoint):(\d+)$/.exec(value);
	return !!match && Number(match[2]) < MAX_VOICES;
}

export type PluginInputMode = 'midi' | 'audio';
export type PluginMidiOutputMode = 'full' | 'pass_through';
export type TuningStyle = 'standard' | 'pure';
export type HarmonicLimit = 'five' | 'seven';

export type SlideRole = 'input' | 'harmony' | 'canon' | 'counterpoint';
export type SlideTrigger = 'legato' | 'always';
export type SlideCurve = 'linear' | 'exponential' | 'inverse_exponential';
export type SlideTravel =
	| { kind: 'off' }
	| { kind: 'time'; milliseconds: number }
	| { kind: 'rate'; semitones_per_second: number };
export interface SlideSettings {
	travel: SlideTravel;
	trigger: SlideTrigger;
	curve: SlideCurve;
}
export interface SlideOverride {
	travel: SlideTravel | null;
	trigger: SlideTrigger | null;
	curve: SlideCurve | null;
}
export interface SlideSlot {
	role: SlideRole;
	voice: number;
}
export interface SlideConfig {
	roles: [SlideSettings, SlideSettings, SlideSettings, SlideSettings];
	voices: [SlideOverride[], SlideOverride[], SlideOverride[], SlideOverride[]];
}
export interface SlideVoiceState {
	voiceId: string;
	slot: SlideSlot;
	currentFrequencyHz: number;
	targetFrequencyHz: number;
	progress: number;
	curve: SlideCurve;
	durationMs: number;
}

/** What lanes do with pending emissions when the player releases the
 *  NoteOn that seeded them (#11). Shape mirrors the Rust `HoldMode`
 *  enum's JSON serialization (`hold_mode_to_json` in
 *  `crates/contrapunk-companion/src/lane.rs`).
 *
 *    - `cancel`       — kill all pending derived from the released note.
 *    - `near_future`  — let pending within `tail_beats` of now fire;
 *                       cancel anything beyond.
 *    - `phrase_end`   — let pending within the current phrase
 *                       (anchor + beats_per_bar) fire; cancel beyond.
 *    - `forever`      — never cancel (pre-v1.2 behavior, preserves
 *                       canon-as-canon semantics).
 */
export type HoldMode =
	| { kind: 'cancel' }
	| { kind: 'near_future'; tail_beats: number }
	| { kind: 'phrase_end' }
	| { kind: 'forever' };

export type PhrasePhase = 'idle' | 'opening' | 'active' | 'releasing';
export interface PhraseState {
	id: number | null;
	phase: PhrasePhase;
	gapBeats: number;
	startedAt: number | null;
	releaseStartedAt: number | null;
	attackCount: number;
	openingNote: number | null;
	previousNote: number | null;
	latestNote: number | null;
	latestVelocity: number | null;
	latestChannel: number | null;
	inputIdle: boolean;
}

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
	octaveIntensity?: number;
	voiceLeadingEnabled: boolean;
	voiceLeadingStyle: string;
	interchangeEnabled: boolean;
	interchangeRange: number;
	voicePosition: number;
	voiceCount: number;
	autoKey: boolean;
	tuningStyle: TuningStyle;
	tuningDepth: number;
	harmonicLimit: HarmonicLimit;
	isRunning: boolean;
	counterpointSpecies: string;
	counterpointStrictness: string;
	explicitIntervalMap?: {
		degreeOffsets: number[][];
		fallbackOffsets: number[];
	};
}

/** Real-time note state emitted during MIDI routing. */
export interface NoteState {
	inputNotes: number[];
	harmonyNotes: number[];
	borrowedNotes: number[];
	chordName: string;
	lastBorrowedFrom: string;
	currentKey: string;
	/** Companion-emitted notes attributed by originating lane. Harmony,
	 *  Canon, and Counterpoint are separate ownership sets so same-pitch
	 *  voices remain visible independently across all surfaces. */
	canonNotes?: number[];
	counterpointNotes?: number[];
	phrase?: PhraseState;
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
	masterGain: number;
	/** Native performance-mixer base gains: input, harmony, canon, counterpoint. */
	mixGains?: number[];
}

/** Snapshot of built-in reverb parameters. */
export interface ReverbState {
	enabled: boolean;
	mix: number;
	roomSize: number;
	damping: number;
}

/** Canonical subdivision tags accepted by `setDelaySubdivision`. */
export type DelaySubdivision = '1/4' | '1/8d' | '1/8' | '1/8t' | '1/16' | '1/16t';

/** Snapshot of built-in delay parameters. */
export interface DelayState {
	enabled: boolean;
	mix: number;
	timeMs: number;
	feedback: number;
	/** When true, the delay tap length is derived from the transport
	 *  BPM and `subdivision`; otherwise `timeMs` is the free tap. */
	syncEnabled: boolean;
	subdivision: DelaySubdivision;
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
/** Feature gates exposed by an adapter so the UI can hide controls
 *  that would otherwise no-op silently on a given surface. Add a
 *  flag here when a feature ships on one surface before the others;
 *  remove it once parity catches up. */
export interface AdapterCapabilities {
	/** Whether the delay FX honors `setDelaySyncEnabled` /
	 *  `setDelaySubdivision`. False on WASM + plugin today — they
	 *  stub the setters because their audio paths don't read the
	 *  transport BPM. */
	delayTempoSync: boolean;
	/** Whether the audio chain topology (chain blocks) is editable
	 *  from this surface. False on WASM + plugin. */
	chainEditor: boolean;
	/** Whether the surface emits per-note state updates so the Piano
	 *  / Fretboard can light keys. True on Tauri + WASM + plugin
	 *  (plugin emits via editor on_frame). */
	noteUpdates: boolean;
	/** Whether this surface owns the transport (play/stop, BPM, time
	 *  signature). False in plugin mode — the DAW is the master
	 *  clock and exposes its own transport UI; ours would be a lie. */
	transportControl: boolean;
	/** Whether this surface routes MIDI itself, so device pickers and
	 *  the start/stop routing flow make sense. False in plugin mode
	 *  — the DAW handles MIDI routing. */
	midiDevicePicker: boolean;
	/** Whether the surface drives the built-in synth. */
	audioFx: boolean;
	/** Whether the surface also implements the legacy delay and reverb racks. */
	builtInFx: boolean;
	/** Whether the surface supports Companion lanes (canon +
	 *  counterpoint). False in plugin mode for v1.3.0 — plugin core
	 *  doesn't wire Companion yet (deferred to v1.4). The
	 *  CompanionPanel and master HoldMode toggle hide when false. */
	companionLanes: boolean;
	/** Whether source-degree explicit interval maps and their live editor work. */
	intervalMaps: boolean;
	/** Whether the reusable declarative PatternLane roles are wired on
	 *  this surface. Kept separate from legacy Canon/Counterpoint support. */
	patternLanes: boolean;
	/** Whether exact shared phrase tracking, configuration, and telemetry work. */
	phraseContext: boolean;
	/** Whether the InputPanel renders the three-way source picker
	 *  (MIDI / Guitar Audio / Voice). False in plugin mode — the DAW
	 *  routes audio in and chooses the input bus itself, so the picker
	 *  would be inert. */
	inputSourcePicker: boolean;
	/** Whether the OutputPanel exposes enforceable per-part routing.
	 *  Currently true only on Tauri; browser dispatch and plugin-host
	 *  routing do not honor these destinations yet. */
	perVoicePortRouting: boolean;
	/** Whether plugin mode exposes a host-side MIDI output mode selector
	 *  (Full Contrapunk vs pass-through-only). False outside plugin mode. */
	pluginMidiOutputMode: boolean;
	/** Whether per-role Input/Harmony/Canon/Counterpoint synth gains are real. */
	roleMix: boolean;
	/** Whether Contrapunk-owned audio accepts exact Adaptive Pure frequencies. */
	nativeTuning: boolean;
	/** Whether the surface supports loading + saving the per-string
	 *  calibration profile to disk and applying it to the live guitar
	 *  pipeline. Tauri only for v1 — uses `app_data_dir()`. WASM /
	 *  plugin would need their own persistence story. */
	calibrationFlow: boolean;
}

/** Status returned from `getCalibrationStatus`. `sampleCounts[i]` is the
 *  total number of soft+strong calibration samples captured for string i
 *  (index 0 = Low E, 5 = High E). */
export interface CalibrationStatus {
	existsOnDisk: boolean;
	path: string;
	version: number;
	sampleCounts: number[];
}

export interface ContrapunkAdapter {
	/** Per-surface capability flags. Read in the UI to gate controls
	 *  that aren't wired on this adapter. */
	readonly capabilities: AdapterCapabilities;

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

	setTuningStyle(style: TuningStyle): Promise<void>;
	setTuningDepth(depth: number): Promise<void>;
	setHarmonicLimit(limit: HarmonicLimit): Promise<void>;
	setTuningCompare(enabled: boolean): Promise<void>;
	getSlideConfig(): Promise<SlideConfig>;
	setSlideConfig(config: SlideConfig): Promise<void>;
	getSlideVoices(): Promise<SlideVoiceState[]>;

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

	/** Replace the seven-degree anchor-relative semitone map and chromatic fallback. */
	setExplicitIntervalMap(degreeOffsets: number[][], fallbackOffsets: number[]): Promise<void>;

	// -- Companion / CanonLane (#3) --

	/** Master enable for the Companion orchestrator (#91). Required for
	 *  any registered Lane (including CanonLane) to actually run. */
	companionSetEnabled(enabled: boolean): Promise<void>;

	/** Read whether the Companion is currently enabled. */
	companionIsEnabled(): Promise<boolean>;

	/** Set the Companion's global HoldMode default (#11). Lanes / voices
	 *  can still override via the existing canon / counterpoint
	 *  configure paths. JSON shape:
	 *    {"kind":"cancel"}
	 *    {"kind":"near_future","tail_beats":1.0}
	 *    {"kind":"phrase_end"}
	 *    {"kind":"forever"}
	 */
	companionSetGlobalHoldMode(holdMode: HoldMode): Promise<void>;
	setPhraseGapBeats(beats: number): Promise<void>;
	getPhraseState(): Promise<PhraseState>;

	/** Enable / disable the Canon voice (#3). Requires the Companion
	 *  master to be enabled too — this is the per-lane gate. */
	canonSetEnabled(enabled: boolean): Promise<void>;

	/** Set the canon delay in beats. Clamped engine-side to [0.0, 8.0]. */
	canonSetDelay(beats: number): Promise<void>;

	/** Set the canon diatonic transpose for voice 0 (backward-compat).
	 *  For multi-voice canons use canonSetVoices. Clamped to [-7, +7]. */
	canonSetTranspose(degrees: number): Promise<void>;

	/** Replace the canon voices array. Each voice has its own delay
	 *  and transpose so callers can configure multi-entry canons
	 *  (3-voice at +0/+2/+4 beats, etc). Clamped to 8 voices max. */
	canonSetVoices(
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
			hold_mode?: HoldMode | null;
		}>
	): Promise<void>;

	/** Generic lane-level configure for CanonLane (#11 override path).
	 *  Sends an arbitrary partial JSON state blob to the lane's
	 *  `deserialize_state`. Use for fields not covered by the typed
	 *  setters above — currently the lane-level `hold_mode` field. */
	canonConfigure(partial: Record<string, unknown>): Promise<void>;

	/** Configure the dedicated CounterpointLane. Each field is
	 *  optional — only the ones you pass are applied. */
	counterpointSetConfig(config: {
		enabled?: boolean;
		species?: string;
		transpose_degrees?: number;
		prefer_above?: boolean;
		phrase_aware?: boolean;
	}): Promise<void>;

	/** Generic lane-level configure for CounterpointLane (#11 override
	 *  path). Same shape as `canonConfigure`. */
	counterpointConfigure(partial: Record<string, unknown>): Promise<void>;

	/** Configure/read one reusable stable pattern role. */
	patternConfigure(
		laneId: 'pattern_low' | 'pattern_counter',
		partial: Record<string, unknown>
	): Promise<void>;
	patternState(laneId: 'pattern_low' | 'pattern_counter'): Promise<Record<string, unknown> | null>;

	/** Read the dedicated CounterpointLane's current state. */
	counterpointState(): Promise<{
		enabled: boolean;
		species: string;
		transpose_degrees: number;
		prefer_above: boolean;
		phrase_aware?: boolean;
	} | null>;

	/** Read the canon lane's full config snapshot. */
	canonState(): Promise<{
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
	} | null>;

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

	/** Set one stable musical part's destination. */
	setVoiceOutput(route: VoiceRouteId, target: VoiceOutputTarget): Promise<void>;

	/** Return non-default route assignments; omitted routes use the synth. */
	getVoiceOutputs(): Promise<VoiceOutputAssignment[]>;

	// -- Plugin host routing controls --
	getPluginInputMode(): Promise<PluginInputMode>;
	setPluginInputMode(mode: PluginInputMode): Promise<void>;
	getPluginMidiOutputMode(): Promise<PluginMidiOutputMode>;
	setPluginMidiOutputMode(mode: PluginMidiOutputMode): Promise<void>;
	getPluginSynthEnabled(): Promise<boolean>;
	setPluginSynthEnabled(enabled: boolean): Promise<void>;
	panicAllNotesOff(): Promise<void>;
	/** Subscribe to DAW/plugin parameter changes (automation, controller assignments). */
	onPluginParamsUpdate(callback: () => void): () => void;

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

	/** Read the current calibration status (whether a profile exists on
	 *  disk, where, and per-string sample counts). Stubbed on adapters
	 *  whose `capabilities.calibrationFlow` is false. */
	getCalibrationStatus(): Promise<CalibrationStatus>;

	/** Force-reload the calibration profile from disk into AppState. */
	loadCalibrationProfile(): Promise<CalibrationStatus>;

	/** Save a JSON-serialized calibration profile to disk. */
	saveCalibrationProfile(profileJson: string): Promise<CalibrationStatus>;

	/** Delete the calibration profile from disk and reset to defaults.
	 *  Gives users a "Reset to Default" affordance — useful when a
	 *  bad calibration sweep poisons the file. */
	deleteCalibrationProfile(): Promise<CalibrationStatus>;

	/** Get the engine's current port-map: for each result-index `i`
	 *  (0 = the user's input/melody, 1..N = harmony voices), the
	 *  arrangement slot the engine will route that voice through.
	 *  Empty when the engine has not processed any notes yet; UI should
	 *  fall back to a config-derived mapping in that case. Used by the
	 *  Voice Generation Chain to reflect the engine's ACTUAL routing
	 *  for Species 2-4, drop voicings, and inversions rather than
	 *  assuming `voicePosition` directly. */
	getLastPortMap(): Promise<number[]>;

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
	setSynthMasterGain(value: number): Promise<void>;
	setSynthMixGain(group: number, value: number): Promise<void>;

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
	setDelaySyncEnabled(enabled: boolean): Promise<void>;
	setDelaySubdivision(subdivision: DelaySubdivision): Promise<void>;

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
