/* tslint:disable */
/* eslint-disable */

export class Engine {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Clear tracked note state (call after config changes that invalidate active harmonies).
     */
    clear_notes(): void;
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
     * Set the octave mode (e.g. "None", "Spread", "Mirror").
     */
    set_octave_mode(mode: string): void;
    /**
     * Set the scale mode (e.g. "Ionian", "Dorian", "HarmonicMinor").
     */
    set_scale_mode(mode: string): void;
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
}

export class WasmGuitarInput {
    free(): void;
    [Symbol.dispose](): void;
    constructor(sample_rate: number, buffer_size: number);
    /**
     * Process audio block, returns JSON array of MIDI events.
     */
    process_block(audio: Float32Array): string;
    set_bends_enabled(v: boolean): void;
    set_input_gain(v: number): void;
    set_legato_enabled(v: boolean): void;
    set_onset_threshold(v: number): void;
    set_slides_enabled(v: boolean): void;
    set_string_confidence(v: number): void;
    set_vibrato_enabled(v: boolean): void;
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
    readonly engine_clear_notes: (a: number) => void;
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
    readonly engine_set_interchange: (a: number, b: number, c: number, d: number) => void;
    readonly engine_set_key: (a: number, b: number, c: number, d: number) => void;
    readonly engine_set_mode: (a: number, b: number, c: number, d: number) => void;
    readonly engine_set_octave_mode: (a: number, b: number, c: number, d: number) => void;
    readonly engine_set_scale_mode: (a: number, b: number, c: number, d: number) => void;
    readonly engine_set_voice_count: (a: number, b: number, c: number) => void;
    readonly engine_set_voice_leading: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly engine_set_voice_position: (a: number, b: number, c: number) => void;
    readonly init_panic_hook: () => void;
    readonly midi_to_name: (a: number, b: number) => void;
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
