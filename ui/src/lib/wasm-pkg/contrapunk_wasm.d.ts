/* tslint:disable */
/* eslint-disable */

export class CompanionWasm {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Advance the transport by `frames` audio frames. JS calls this
     * from its animation-frame / WebAudio loop so per-lane scheduling
     * (canon delays, counterpoint subdivisions) sees forward time.
     * At the default 48 kHz sample rate, 768 frames ≈ 16 ms.
     */
    advance(frames: number): void;
    /**
     * Apply a partial JSON state blob to the canon lane (same shape
     * the Tauri command consumes). See CanonLane::deserialize_state.
     */
    configure_canon(json: string): void;
    configure_counterpoint(json: string): void;
    is_enabled(): boolean;
    constructor();
    on_note_off(note: number, channel: number): string;
    /**
     * Feed a player NoteOn into the Companion. Returns a JSON array
     * of dispatch ops emitted by lanes that fired immediately
     * (Species 1 canon-onset emissions, etc.).
     */
    on_note_on(note: number, velocity: number, channel: number): string;
    set_enabled(enabled: boolean): void;
    /**
     * Mirror the global engine's key/scale/mode/voice_count etc.
     * into the snapshot the Companion's mini-engines read. Called by
     * JS whenever the Harmony tab changes these.
     */
    set_global_state(key: string, mode: string, scale_mode: string, voice_count: number, voice_position: number): void;
    /**
     * Tick the lanes. Drains pending emissions whose fire_at has
     * elapsed. Returns a JSON array of dispatch ops to schedule on
     * the WebAudio synth.
     */
    tick(): string;
}

export class Engine {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Returns the bass-register threshold MIDI note number.
     */
    bass_register_threshold(): number;
    /**
     * Clear tracked note state (call after config changes that invalidate active harmonies).
     */
    clear_notes(): void;
    /**
     * Returns the current key as a string (for UI to update after auto-detection).
     */
    current_key(): string;
    /**
     * Delete a custom preset by name.
     */
    delete_preset(name: string): void;
    /**
     * Get the current note state as a JS object.
     */
    get_note_state(): any;
    /**
     * Get the current engine state as a JS object.
     */
    get_state(): any;
    /**
     * Harmonize a single MIDI note number.
     * Returns a JS array of MIDI note numbers (u8).
     */
    harmonize(note: number): Uint8Array;
    /**
     * List all available presets (builtins + custom).
     */
    list_presets(): any;
    /**
     * Load a preset by name, applying its settings to the engine.
     */
    load_preset(name: string): void;
    /**
     * Create a new Engine with default settings (C major, PassThrough).
     */
    constructor();
    /**
     * Process a MIDI Note-Off event.
     * Returns a JS array of MIDI note numbers to release.
     */
    note_off(note: number): Uint8Array;
    /**
     * Process a MIDI Note-On event.
     * Returns a JS array of MIDI note numbers to sound.
     */
    note_on(note: number): Uint8Array;
    /**
     * Save current engine settings as a custom preset.
     */
    save_preset(name: string): void;
    /**
     * Enable or disable auto-key detection.
     */
    set_auto_key(enabled: boolean): void;
    /**
     * Sets the bass-register threshold MIDI note (notes below pass
     * through when `suppress_bass_register` is true). Clamped to
     * 0..=127.
     */
    set_bass_register_threshold(midi: number): void;
    /**
     * Set the counterpoint beat-phase position within the bar
     * (`0.0 .. beats_per_bar`). Pass `None` (via JS `undefined`/`null` from
     * the optional setter) to disable beat awareness and fall back to
     * Species 1 behavior.
     */
    set_counterpoint_beat_phase(phase?: number | null): void;
    /**
     * Set the counterpoint species (`"Species1"` through `"Species4"`).
     *
     * Only active when the harmony mode is `StrictCounterpoint`. Species 2-4
     * require beat-phase input via `set_counterpoint_beat_phase`; without
     * it they fall back to Species 1 behavior.
     */
    set_counterpoint_species(species: string): void;
    /**
     * Set the counterpoint strictness (`"Relaxed"` or `"Strict"`).
     */
    set_counterpoint_strictness(strictness: string): void;
    /**
     * Configure modal interchange (enabled flag + borrowing range 1-5).
     */
    set_interchange(enabled: boolean, range: number): void;
    /**
     * Set the musical key (e.g. "C", "Db", "F#").
     */
    set_key(key: string): void;
    /**
     * Set the harmony mode (e.g. "PassThrough", "DiatonicThirds").
     */
    set_mode(mode: string): void;
    /**
     * Continuous octave-spread coefficient. Range [0.0, 1.0]; 0 = no
     * displacement, 1 = full-octave (legacy) per-voice displacement.
     */
    set_octave_intensity(amount: number): void;
    /**
     * Set the octave mode (e.g. "None", "Spread", "Mirror").
     */
    set_octave_mode(mode: string): void;
    /**
     * Set the scale mode (e.g. "Ionian", "Dorian", "HarmonicMinor").
     */
    set_scale_mode(mode: string): void;
    /**
     * Enable or disable bass-register suppression. When on, input
     * notes below the threshold pass through without producing
     * harmony — for users who play the bass line themselves.
     */
    set_suppress_bass_register(enabled: boolean): void;
    /**
     * Set the number of output voices (1 = melody only, 2+ = melody + harmonies).
     */
    set_voice_count(count: number): void;
    /**
     * Configure voice leading (enabled flag + style string).
     */
    set_voice_leading(enabled: boolean, style: string): void;
    /**
     * Set the voice position (which output slot carries the melody).
     */
    set_voice_position(position: number): void;
    /**
     * Returns whether bass-register suppression is active. Issue #100.
     */
    suppress_bass_register(): boolean;
}

export class WasmGuitarInput {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Free resources.
     */
    free(): void;
    /**
     * Create a new GuitarInput DSP pipeline with the given sample rate and buffer size.
     */
    constructor(sample_rate: number, buffer_size: number);
    /**
     * Process an audio block and return MIDI events as a JSON string.
     * Input: Float32Array of mono audio samples.
     * Output: JSON array of event objects.
     */
    process_block(samples: Float32Array): string;
    /**
     * Enable/disable pitch bend detection.
     */
    set_bends_enabled(val: boolean): void;
    /**
     * Set input gain (default 1.0).
     */
    set_input_gain(val: number): void;
    /**
     * Enable/disable legato detection.
     */
    set_legato_enabled(val: boolean): void;
    /**
     * Set onset threshold (default 0.015).
     */
    set_onset_threshold(val: number): void;
    /**
     * Enable/disable slide detection.
     */
    set_slides_enabled(val: boolean): void;
    /**
     * Set string confidence minimum (default 0.4).
     */
    set_string_confidence(val: number): void;
    /**
     * Enable/disable vibrato detection.
     */
    set_vibrato_enabled(val: boolean): void;
}

export function init_panic_hook(): void;

/**
 * Convert a MIDI note number (0-127) to its note name (e.g. 60 -> "C4").
 */
export function midi_to_name(midi: number): string;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_engine_free: (a: number, b: number) => void;
    readonly __wbg_wasmguitarinput_free: (a: number, b: number) => void;
    readonly engine_bass_register_threshold: (a: number) => number;
    readonly engine_clear_notes: (a: number) => void;
    readonly engine_current_key: (a: number, b: number) => void;
    readonly engine_delete_preset: (a: number, b: number, c: number, d: number) => void;
    readonly engine_get_note_state: (a: number, b: number) => void;
    readonly engine_get_state: (a: number, b: number) => void;
    readonly engine_harmonize: (a: number, b: number, c: number) => void;
    readonly engine_list_presets: (a: number, b: number) => void;
    readonly engine_load_preset: (a: number, b: number, c: number, d: number) => void;
    readonly engine_new: () => number;
    readonly engine_note_off: (a: number, b: number, c: number) => void;
    readonly engine_note_on: (a: number, b: number, c: number) => void;
    readonly engine_save_preset: (a: number, b: number, c: number, d: number) => void;
    readonly engine_set_auto_key: (a: number, b: number, c: number) => void;
    readonly engine_set_bass_register_threshold: (a: number, b: number) => void;
    readonly engine_set_counterpoint_beat_phase: (a: number, b: number, c: number) => void;
    readonly engine_set_counterpoint_species: (a: number, b: number, c: number, d: number) => void;
    readonly engine_set_counterpoint_strictness: (a: number, b: number, c: number, d: number) => void;
    readonly engine_set_interchange: (a: number, b: number, c: number, d: number) => void;
    readonly engine_set_key: (a: number, b: number, c: number, d: number) => void;
    readonly engine_set_mode: (a: number, b: number, c: number, d: number) => void;
    readonly engine_set_octave_intensity: (a: number, b: number) => void;
    readonly engine_set_octave_mode: (a: number, b: number, c: number, d: number) => void;
    readonly engine_set_scale_mode: (a: number, b: number, c: number, d: number) => void;
    readonly engine_set_suppress_bass_register: (a: number, b: number) => void;
    readonly engine_set_voice_count: (a: number, b: number, c: number) => void;
    readonly engine_set_voice_leading: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly engine_set_voice_position: (a: number, b: number, c: number) => void;
    readonly engine_suppress_bass_register: (a: number) => number;
    readonly init_panic_hook: () => void;
    readonly midi_to_name: (a: number, b: number) => void;
    readonly wasmguitarinput_free: (a: number) => void;
    readonly wasmguitarinput_new: (a: number, b: number) => number;
    readonly wasmguitarinput_process_block: (a: number, b: number, c: number, d: number) => void;
    readonly wasmguitarinput_set_bends_enabled: (a: number, b: number) => void;
    readonly wasmguitarinput_set_input_gain: (a: number, b: number) => void;
    readonly wasmguitarinput_set_legato_enabled: (a: number, b: number) => void;
    readonly wasmguitarinput_set_onset_threshold: (a: number, b: number) => void;
    readonly wasmguitarinput_set_slides_enabled: (a: number, b: number) => void;
    readonly wasmguitarinput_set_string_confidence: (a: number, b: number) => void;
    readonly wasmguitarinput_set_vibrato_enabled: (a: number, b: number) => void;
    readonly __wbg_companionwasm_free: (a: number, b: number) => void;
    readonly companionwasm_advance: (a: number, b: number) => void;
    readonly companionwasm_configure_canon: (a: number, b: number, c: number, d: number) => void;
    readonly companionwasm_configure_counterpoint: (a: number, b: number, c: number, d: number) => void;
    readonly companionwasm_is_enabled: (a: number) => number;
    readonly companionwasm_new: () => number;
    readonly companionwasm_on_note_off: (a: number, b: number, c: number, d: number) => void;
    readonly companionwasm_on_note_on: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly companionwasm_set_enabled: (a: number, b: number) => void;
    readonly companionwasm_set_global_state: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number) => void;
    readonly companionwasm_tick: (a: number, b: number) => void;
    readonly __wbindgen_export: (a: number, b: number) => number;
    readonly __wbindgen_export2: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_export3: (a: number, b: number, c: number) => void;
    readonly __wbindgen_export4: (a: number) => void;
    readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
    readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
