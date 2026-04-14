/* tslint:disable */
/* eslint-disable */

export class Engine {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Current beat-phase position within the bar (for UI beat indicator).
     */
    beat_position(): number;
    /**
     * Current tempo in BPM.
     */
    bpm(): number;
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
     * Full humanize configuration as a JS object (for UI round-tripping).
     */
    get_humanize_config(): any;
    /**
     * Get the current note state as a JS object.
     */
    get_note_state(): any;
    /**
     * Get the current engine state as a JS object.
     */
    get_state(): any;
    /**
     * Get all current suggestion weights as a JSON object.
     */
    get_suggestion_weights(): any;
    /**
     * Compute ranked note suggestions based on current engine state.
     *
     * Returns a JSON-serialized array of `{note: u8, score: f32}` objects,
     * limited to the top 12 suggestions. This is a visual overlay -- the
     * suggestions are never played audibly.
     */
    get_suggestions(): any;
    /**
     * Harmonize a single MIDI note number.
     * Returns a JS array of MIDI note numbers (u8).
     */
    harmonize(note: number): Uint8Array;
    /**
     * Process a MIDI Note-Off with humanization applied to harmony voices.
     *
     * See [`humanized_note_on`] for the result shape. Note-Off timings
     * mirror the Note-On humanization record (so a swung 8th note stays
     * swung on release).
     */
    humanized_note_off(note: number): any;
    /**
     * Process a MIDI Note-On with humanization applied to harmony voices.
     *
     * The melody note (voice 0) is always returned with zero delay so the
     * player hears their input in realtime. Harmony notes (voice 1+) pass
     * through the humanizer: they get random velocity/jitter/swing, and
     * any note with a non-zero delay is pushed into the internal delay
     * queue to be released on a later `tick()`.
     *
     * Returns a JS object shaped like:
     * ```ignore
     * {
     *   immediate: [{ port, bytes }, ...],   // send this frame
     *   deferred_count: number,              // queued for later tick()s
     *   input_note: u8,
     * }
     * ```
     */
    humanized_note_on(note: number, velocity: number): any;
    /**
     * Whether humanization is currently enabled (master flag).
     */
    is_humanize_enabled(): boolean;
    /**
     * Whether the metronome click is currently enabled.
     */
    is_metronome_enabled(): boolean;
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
     * Reset suggestion weights to Bach chorale calibrated defaults.
     */
    reset_suggestion_weights(): void;
    /**
     * Save current engine settings as a custom preset.
     */
    save_preset(name: string): void;
    /**
     * Enable or disable auto-key detection.
     */
    set_auto_key(enabled: boolean): void;
    /**
     * Set the tempo in BPM. Updates the beat clock without resetting
     * beat position (so the metronome doesn't stutter on tempo changes).
     */
    set_bpm(bpm: number): void;
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
     * require a beat-phase clock; since the WASM build has no internal
     * metronome, these species currently behave like Species 1 unless the
     * host explicitly calls `set_counterpoint_beat_phase` each frame.
     */
    set_counterpoint_species(species: string): void;
    /**
     * Set the counterpoint strictness (`"Relaxed"` or `"Strict"`).
     */
    set_counterpoint_strictness(strictness: string): void;
    /**
     * Enable/disable duration extension on harmony Note-Offs.
     */
    set_duration_enabled(enabled: boolean): void;
    /**
     * Set the max duration extension in ms (applied to Note-Off timing).
     */
    set_duration_variation(ms: number): void;
    /**
     * Bulk-update the humanize configuration from a JS object.
     *
     * Accepts the same snake_case shape as `get_humanize_config` returns.
     * Fields omitted fall back to their current values.
     */
    set_humanize_config(config: any): void;
    /**
     * Master toggle for all humanization effects.
     *
     * When `false`, Note-On/Note-Off pass through unchanged even if
     * swing/jitter/velocity-variation sub-toggles are on.
     */
    set_humanize_enabled(enabled: boolean): void;
    /**
     * Configure modal interchange (enabled flag + borrowing range 1-5).
     */
    set_interchange(enabled: boolean, range: number): void;
    /**
     * Enable/disable timing jitter on harmony notes.
     */
    set_jitter_enabled(enabled: boolean): void;
    /**
     * Set the musical key (e.g. "C", "Db", "F#").
     */
    set_key(key: string): void;
    /**
     * Enable/disable the metronome click track.
     */
    set_metronome_enabled(enabled: boolean): void;
    /**
     * Set the harmony mode (e.g. "PassThrough", "DiatonicThirds").
     */
    set_mode(mode: string): void;
    /**
     * Set the octave mode (e.g. "None", "Spread", "Mirror").
     */
    set_octave_mode(mode: string): void;
    /**
     * Set the scale mode (e.g. "Ionian", "Dorian", "HarmonicMinor").
     */
    set_scale_mode(mode: string): void;
    /**
     * Set a single suggestion weight by term name.
     *
     * Valid term names: chord_tone, scale_tone, dissonance, proximity,
     * contour, leap_recovery, repetition, next_chord_prep, leading_tone,
     * narmour, tessitura.
     */
    set_suggestion_weight(term: string, value: number): void;
    /**
     * Set the swing amount. 0.0 = straight, 0.3 = light, 0.5 = jazz.
     */
    set_swing(amount: number): void;
    /**
     * Enable/disable swing feel on off-beats.
     */
    set_swing_enabled(enabled: boolean): void;
    /**
     * Set the time signature (e.g. 4, 4 for 4/4 time).
     */
    set_time_signature(beats_per_bar: number, beat_unit: number): void;
    /**
     * Set the upper bound for per-note timing jitter (milliseconds).
     * The lower bound tracks 1ms or `max.min(1)` whichever is smaller.
     */
    set_timing_jitter(max_ms: number): void;
    /**
     * Enable/disable random velocity variation on harmony notes.
     */
    set_velocity_enabled(enabled: boolean): void;
    /**
     * Set the +/- range for random velocity jitter (0..127 scale).
     */
    set_velocity_jitter(variation: number): void;
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
     * Drive the beat clock + metronome + humanize delay queue forward.
     *
     * Must be called on every animation frame with `performance.now()`.
     * Returns a JSON-serializable object describing what happened this
     * frame: beat position, any metronome click bytes, and any delayed
     * humanized notes that are now due to be sent.
     *
     * The returned object shape (`TickResultJs`) is:
     * ```ignore
     * {
     *   beat_position: f64,
     *   beat_number: u8,
     *   beat_crossed: u8 | null,
     *   metronome_on: u8[] | null,
     *   metronome_off: u8[] | null,
     *   scheduled_notes: [{ port: number, bytes: u8[] }, ...],
     *   humanize_enabled: bool,
     *   running: bool,
     *   bpm: f64,
     * }
     * ```
     */
    tick(now_ms: number): any;
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
    readonly engine_beat_position: (a: number) => number;
    readonly engine_bpm: (a: number) => number;
    readonly engine_clear_notes: (a: number) => void;
    readonly engine_current_key: (a: number, b: number) => void;
    readonly engine_delete_preset: (a: number, b: number, c: number, d: number) => void;
    readonly engine_get_humanize_config: (a: number, b: number) => void;
    readonly engine_get_note_state: (a: number, b: number) => void;
    readonly engine_get_state: (a: number, b: number) => void;
    readonly engine_get_suggestion_weights: (a: number, b: number) => void;
    readonly engine_get_suggestions: (a: number, b: number) => void;
    readonly engine_harmonize: (a: number, b: number, c: number) => void;
    readonly engine_humanized_note_off: (a: number, b: number, c: number) => void;
    readonly engine_humanized_note_on: (a: number, b: number, c: number, d: number) => void;
    readonly engine_is_humanize_enabled: (a: number) => number;
    readonly engine_is_metronome_enabled: (a: number) => number;
    readonly engine_list_presets: (a: number, b: number) => void;
    readonly engine_load_preset: (a: number, b: number, c: number, d: number) => void;
    readonly engine_new: () => number;
    readonly engine_note_off: (a: number, b: number, c: number) => void;
    readonly engine_note_on: (a: number, b: number, c: number) => void;
    readonly engine_reset_suggestion_weights: (a: number) => void;
    readonly engine_save_preset: (a: number, b: number, c: number, d: number) => void;
    readonly engine_set_auto_key: (a: number, b: number, c: number) => void;
    readonly engine_set_bpm: (a: number, b: number) => void;
    readonly engine_set_counterpoint_beat_phase: (a: number, b: number, c: number) => void;
    readonly engine_set_counterpoint_species: (a: number, b: number, c: number, d: number) => void;
    readonly engine_set_counterpoint_strictness: (a: number, b: number, c: number, d: number) => void;
    readonly engine_set_duration_enabled: (a: number, b: number) => void;
    readonly engine_set_duration_variation: (a: number, b: number) => void;
    readonly engine_set_humanize_config: (a: number, b: number, c: number) => void;
    readonly engine_set_humanize_enabled: (a: number, b: number) => void;
    readonly engine_set_interchange: (a: number, b: number, c: number, d: number) => void;
    readonly engine_set_jitter_enabled: (a: number, b: number) => void;
    readonly engine_set_key: (a: number, b: number, c: number, d: number) => void;
    readonly engine_set_metronome_enabled: (a: number, b: number) => void;
    readonly engine_set_mode: (a: number, b: number, c: number, d: number) => void;
    readonly engine_set_octave_mode: (a: number, b: number, c: number, d: number) => void;
    readonly engine_set_scale_mode: (a: number, b: number, c: number, d: number) => void;
    readonly engine_set_suggestion_weight: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly engine_set_swing: (a: number, b: number) => void;
    readonly engine_set_swing_enabled: (a: number, b: number) => void;
    readonly engine_set_time_signature: (a: number, b: number, c: number) => void;
    readonly engine_set_timing_jitter: (a: number, b: number) => void;
    readonly engine_set_velocity_enabled: (a: number, b: number) => void;
    readonly engine_set_velocity_jitter: (a: number, b: number) => void;
    readonly engine_set_voice_count: (a: number, b: number, c: number) => void;
    readonly engine_set_voice_leading: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly engine_set_voice_position: (a: number, b: number, c: number) => void;
    readonly engine_tick: (a: number, b: number, c: number) => void;
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
    readonly __wbindgen_export: (a: number, b: number) => number;
    readonly __wbindgen_export2: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_export3: (a: number) => void;
    readonly __wbindgen_export4: (a: number, b: number, c: number) => void;
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
