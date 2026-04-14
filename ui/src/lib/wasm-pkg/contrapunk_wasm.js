/* @ts-self-types="./contrapunk_wasm.d.ts" */

export class Engine {
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        EngineFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_engine_free(ptr, 0);
    }
    /**
     * Current beat-phase position within the bar (for UI beat indicator).
     * @returns {number}
     */
    beat_position() {
        const ret = wasm.engine_beat_position(this.__wbg_ptr);
        return ret;
    }
    /**
     * Current tempo in BPM.
     * @returns {number}
     */
    bpm() {
        const ret = wasm.engine_bpm(this.__wbg_ptr);
        return ret;
    }
    /**
     * Clear tracked note state (call after config changes that invalidate active harmonies).
     */
    clear_notes() {
        wasm.engine_clear_notes(this.__wbg_ptr);
    }
    /**
     * Returns the current key as a string (for UI to update after auto-detection).
     * @returns {string}
     */
    current_key() {
        let deferred1_0;
        let deferred1_1;
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.engine_current_key(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            deferred1_0 = r0;
            deferred1_1 = r1;
            return getStringFromWasm0(r0, r1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_export4(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * Delete a custom preset by name.
     * @param {string} name
     */
    delete_preset(name) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            const ptr0 = passStringToWasm0(name, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len0 = WASM_VECTOR_LEN;
            wasm.engine_delete_preset(retptr, this.__wbg_ptr, ptr0, len0);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            if (r1) {
                throw takeObject(r0);
            }
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Full humanize configuration as a JS object (for UI round-tripping).
     * @returns {any}
     */
    get_humanize_config() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.engine_get_humanize_config(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            if (r2) {
                throw takeObject(r1);
            }
            return takeObject(r0);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Get the current note state as a JS object.
     * @returns {any}
     */
    get_note_state() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.engine_get_note_state(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            if (r2) {
                throw takeObject(r1);
            }
            return takeObject(r0);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Get the current engine state as a JS object.
     * @returns {any}
     */
    get_state() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.engine_get_state(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            if (r2) {
                throw takeObject(r1);
            }
            return takeObject(r0);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Get all current suggestion weights as a JSON object.
     * @returns {any}
     */
    get_suggestion_weights() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.engine_get_suggestion_weights(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            if (r2) {
                throw takeObject(r1);
            }
            return takeObject(r0);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Compute ranked note suggestions based on current engine state.
     *
     * Returns a JSON-serialized array of `{note: u8, score: f32}` objects,
     * limited to the top 12 suggestions. This is a visual overlay -- the
     * suggestions are never played audibly.
     * @returns {any}
     */
    get_suggestions() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.engine_get_suggestions(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            if (r2) {
                throw takeObject(r1);
            }
            return takeObject(r0);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Harmonize a single MIDI note number.
     * Returns a JS array of MIDI note numbers (u8).
     * @param {number} note
     * @returns {Uint8Array}
     */
    harmonize(note) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.engine_harmonize(retptr, this.__wbg_ptr, note);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            var r3 = getDataViewMemory0().getInt32(retptr + 4 * 3, true);
            if (r3) {
                throw takeObject(r2);
            }
            var v1 = getArrayU8FromWasm0(r0, r1).slice();
            wasm.__wbindgen_export4(r0, r1 * 1, 1);
            return v1;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Process a MIDI Note-Off with humanization applied to harmony voices.
     *
     * See [`humanized_note_on`] for the result shape. Note-Off timings
     * mirror the Note-On humanization record (so a swung 8th note stays
     * swung on release).
     * @param {number} note
     * @returns {any}
     */
    humanized_note_off(note) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.engine_humanized_note_off(retptr, this.__wbg_ptr, note);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            if (r2) {
                throw takeObject(r1);
            }
            return takeObject(r0);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
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
     * @param {number} note
     * @param {number} velocity
     * @returns {any}
     */
    humanized_note_on(note, velocity) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.engine_humanized_note_on(retptr, this.__wbg_ptr, note, velocity);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            if (r2) {
                throw takeObject(r1);
            }
            return takeObject(r0);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Whether humanization is currently enabled (master flag).
     * @returns {boolean}
     */
    is_humanize_enabled() {
        const ret = wasm.engine_is_humanize_enabled(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Whether the metronome click is currently enabled.
     * @returns {boolean}
     */
    is_metronome_enabled() {
        const ret = wasm.engine_is_metronome_enabled(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * List all available presets (builtins + custom).
     * @returns {any}
     */
    list_presets() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.engine_list_presets(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            if (r2) {
                throw takeObject(r1);
            }
            return takeObject(r0);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Load a preset by name, applying its settings to the engine.
     * @param {string} name
     */
    load_preset(name) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            const ptr0 = passStringToWasm0(name, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len0 = WASM_VECTOR_LEN;
            wasm.engine_load_preset(retptr, this.__wbg_ptr, ptr0, len0);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            if (r1) {
                throw takeObject(r0);
            }
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Create a new Engine with default settings (C major, PassThrough).
     */
    constructor() {
        const ret = wasm.engine_new();
        this.__wbg_ptr = ret >>> 0;
        EngineFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * Process a MIDI Note-Off event.
     * Returns a JS array of MIDI note numbers to release.
     * @param {number} note
     * @returns {Uint8Array}
     */
    note_off(note) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.engine_note_off(retptr, this.__wbg_ptr, note);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            var r3 = getDataViewMemory0().getInt32(retptr + 4 * 3, true);
            if (r3) {
                throw takeObject(r2);
            }
            var v1 = getArrayU8FromWasm0(r0, r1).slice();
            wasm.__wbindgen_export4(r0, r1 * 1, 1);
            return v1;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Process a MIDI Note-On event.
     * Returns a JS array of MIDI note numbers to sound.
     * @param {number} note
     * @returns {Uint8Array}
     */
    note_on(note) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.engine_note_on(retptr, this.__wbg_ptr, note);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            var r3 = getDataViewMemory0().getInt32(retptr + 4 * 3, true);
            if (r3) {
                throw takeObject(r2);
            }
            var v1 = getArrayU8FromWasm0(r0, r1).slice();
            wasm.__wbindgen_export4(r0, r1 * 1, 1);
            return v1;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Reset suggestion weights to Bach chorale calibrated defaults.
     */
    reset_suggestion_weights() {
        wasm.engine_reset_suggestion_weights(this.__wbg_ptr);
    }
    /**
     * Save current engine settings as a custom preset.
     * @param {string} name
     */
    save_preset(name) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            const ptr0 = passStringToWasm0(name, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len0 = WASM_VECTOR_LEN;
            wasm.engine_save_preset(retptr, this.__wbg_ptr, ptr0, len0);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            if (r1) {
                throw takeObject(r0);
            }
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Enable or disable auto-key detection.
     * @param {boolean} enabled
     */
    set_auto_key(enabled) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.engine_set_auto_key(retptr, this.__wbg_ptr, enabled);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            if (r1) {
                throw takeObject(r0);
            }
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Set the tempo in BPM. Updates the beat clock without resetting
     * beat position (so the metronome doesn't stutter on tempo changes).
     * @param {number} bpm
     */
    set_bpm(bpm) {
        wasm.engine_set_bpm(this.__wbg_ptr, bpm);
    }
    /**
     * Set the counterpoint beat-phase position within the bar
     * (`0.0 .. beats_per_bar`). Pass `None` (via JS `undefined`/`null` from
     * the optional setter) to disable beat awareness and fall back to
     * Species 1 behavior.
     * @param {number | null} [phase]
     */
    set_counterpoint_beat_phase(phase) {
        wasm.engine_set_counterpoint_beat_phase(this.__wbg_ptr, !isLikeNone(phase), isLikeNone(phase) ? 0 : phase);
    }
    /**
     * Set the counterpoint species (`"Species1"` through `"Species4"`).
     *
     * Only active when the harmony mode is `StrictCounterpoint`. Species 2-4
     * require a beat-phase clock; since the WASM build has no internal
     * metronome, these species currently behave like Species 1 unless the
     * host explicitly calls `set_counterpoint_beat_phase` each frame.
     * @param {string} species
     */
    set_counterpoint_species(species) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            const ptr0 = passStringToWasm0(species, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len0 = WASM_VECTOR_LEN;
            wasm.engine_set_counterpoint_species(retptr, this.__wbg_ptr, ptr0, len0);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            if (r1) {
                throw takeObject(r0);
            }
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Set the counterpoint strictness (`"Relaxed"` or `"Strict"`).
     * @param {string} strictness
     */
    set_counterpoint_strictness(strictness) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            const ptr0 = passStringToWasm0(strictness, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len0 = WASM_VECTOR_LEN;
            wasm.engine_set_counterpoint_strictness(retptr, this.__wbg_ptr, ptr0, len0);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            if (r1) {
                throw takeObject(r0);
            }
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Enable/disable duration extension on harmony Note-Offs.
     * @param {boolean} enabled
     */
    set_duration_enabled(enabled) {
        wasm.engine_set_duration_enabled(this.__wbg_ptr, enabled);
    }
    /**
     * Set the max duration extension in ms (applied to Note-Off timing).
     * @param {number} ms
     */
    set_duration_variation(ms) {
        wasm.engine_set_duration_variation(this.__wbg_ptr, ms);
    }
    /**
     * Bulk-update the humanize configuration from a JS object.
     *
     * Accepts the same snake_case shape as `get_humanize_config` returns.
     * Fields omitted fall back to their current values.
     * @param {any} config
     */
    set_humanize_config(config) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.engine_set_humanize_config(retptr, this.__wbg_ptr, addHeapObject(config));
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            if (r1) {
                throw takeObject(r0);
            }
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Master toggle for all humanization effects.
     *
     * When `false`, Note-On/Note-Off pass through unchanged even if
     * swing/jitter/velocity-variation sub-toggles are on.
     * @param {boolean} enabled
     */
    set_humanize_enabled(enabled) {
        wasm.engine_set_humanize_enabled(this.__wbg_ptr, enabled);
    }
    /**
     * Configure modal interchange (enabled flag + borrowing range 1-5).
     * @param {boolean} enabled
     * @param {number} range
     */
    set_interchange(enabled, range) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.engine_set_interchange(retptr, this.__wbg_ptr, enabled, range);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            if (r1) {
                throw takeObject(r0);
            }
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Enable/disable timing jitter on harmony notes.
     * @param {boolean} enabled
     */
    set_jitter_enabled(enabled) {
        wasm.engine_set_jitter_enabled(this.__wbg_ptr, enabled);
    }
    /**
     * Set the musical key (e.g. "C", "Db", "F#").
     * @param {string} key
     */
    set_key(key) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            const ptr0 = passStringToWasm0(key, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len0 = WASM_VECTOR_LEN;
            wasm.engine_set_key(retptr, this.__wbg_ptr, ptr0, len0);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            if (r1) {
                throw takeObject(r0);
            }
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Enable/disable the metronome click track.
     * @param {boolean} enabled
     */
    set_metronome_enabled(enabled) {
        wasm.engine_set_metronome_enabled(this.__wbg_ptr, enabled);
    }
    /**
     * Set the harmony mode (e.g. "PassThrough", "DiatonicThirds").
     * @param {string} mode
     */
    set_mode(mode) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            const ptr0 = passStringToWasm0(mode, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len0 = WASM_VECTOR_LEN;
            wasm.engine_set_mode(retptr, this.__wbg_ptr, ptr0, len0);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            if (r1) {
                throw takeObject(r0);
            }
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Set the octave mode (e.g. "None", "Spread", "Mirror").
     * @param {string} mode
     */
    set_octave_mode(mode) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            const ptr0 = passStringToWasm0(mode, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len0 = WASM_VECTOR_LEN;
            wasm.engine_set_octave_mode(retptr, this.__wbg_ptr, ptr0, len0);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            if (r1) {
                throw takeObject(r0);
            }
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Set the scale mode (e.g. "Ionian", "Dorian", "HarmonicMinor").
     * @param {string} mode
     */
    set_scale_mode(mode) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            const ptr0 = passStringToWasm0(mode, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len0 = WASM_VECTOR_LEN;
            wasm.engine_set_scale_mode(retptr, this.__wbg_ptr, ptr0, len0);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            if (r1) {
                throw takeObject(r0);
            }
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Set a single suggestion weight by term name.
     *
     * Valid term names: chord_tone, scale_tone, dissonance, proximity,
     * contour, leap_recovery, repetition, next_chord_prep, leading_tone,
     * narmour, tessitura.
     * @param {string} term
     * @param {number} value
     */
    set_suggestion_weight(term, value) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            const ptr0 = passStringToWasm0(term, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len0 = WASM_VECTOR_LEN;
            wasm.engine_set_suggestion_weight(retptr, this.__wbg_ptr, ptr0, len0, value);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            if (r1) {
                throw takeObject(r0);
            }
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Set the swing amount. 0.0 = straight, 0.3 = light, 0.5 = jazz.
     * @param {number} amount
     */
    set_swing(amount) {
        wasm.engine_set_swing(this.__wbg_ptr, amount);
    }
    /**
     * Enable/disable swing feel on off-beats.
     * @param {boolean} enabled
     */
    set_swing_enabled(enabled) {
        wasm.engine_set_swing_enabled(this.__wbg_ptr, enabled);
    }
    /**
     * Set the time signature (e.g. 4, 4 for 4/4 time).
     * @param {number} beats_per_bar
     * @param {number} beat_unit
     */
    set_time_signature(beats_per_bar, beat_unit) {
        wasm.engine_set_time_signature(this.__wbg_ptr, beats_per_bar, beat_unit);
    }
    /**
     * Set the upper bound for per-note timing jitter (milliseconds).
     * The lower bound tracks 1ms or `max.min(1)` whichever is smaller.
     * @param {number} max_ms
     */
    set_timing_jitter(max_ms) {
        wasm.engine_set_timing_jitter(this.__wbg_ptr, max_ms);
    }
    /**
     * Enable/disable random velocity variation on harmony notes.
     * @param {boolean} enabled
     */
    set_velocity_enabled(enabled) {
        wasm.engine_set_velocity_enabled(this.__wbg_ptr, enabled);
    }
    /**
     * Set the +/- range for random velocity jitter (0..127 scale).
     * @param {number} variation
     */
    set_velocity_jitter(variation) {
        wasm.engine_set_velocity_jitter(this.__wbg_ptr, variation);
    }
    /**
     * Set the number of output voices (1 = melody only, 2+ = melody + harmonies).
     * @param {number} count
     */
    set_voice_count(count) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.engine_set_voice_count(retptr, this.__wbg_ptr, count);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            if (r1) {
                throw takeObject(r0);
            }
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Configure voice leading (enabled flag + style string).
     * @param {boolean} enabled
     * @param {string} style
     */
    set_voice_leading(enabled, style) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            const ptr0 = passStringToWasm0(style, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len0 = WASM_VECTOR_LEN;
            wasm.engine_set_voice_leading(retptr, this.__wbg_ptr, enabled, ptr0, len0);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            if (r1) {
                throw takeObject(r0);
            }
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Set the voice position (which output slot carries the melody).
     * @param {number} position
     */
    set_voice_position(position) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.engine_set_voice_position(retptr, this.__wbg_ptr, position);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            if (r1) {
                throw takeObject(r0);
            }
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
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
     * @param {number} now_ms
     * @returns {any}
     */
    tick(now_ms) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.engine_tick(retptr, this.__wbg_ptr, now_ms);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            if (r2) {
                throw takeObject(r1);
            }
            return takeObject(r0);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
}
if (Symbol.dispose) Engine.prototype[Symbol.dispose] = Engine.prototype.free;

export class WasmGuitarInput {
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        WasmGuitarInputFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_wasmguitarinput_free(ptr, 0);
    }
    /**
     * Free resources.
     */
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.wasmguitarinput_free(ptr);
    }
    /**
     * Create a new GuitarInput DSP pipeline with the given sample rate and buffer size.
     * @param {number} sample_rate
     * @param {number} buffer_size
     */
    constructor(sample_rate, buffer_size) {
        const ret = wasm.wasmguitarinput_new(sample_rate, buffer_size);
        this.__wbg_ptr = ret >>> 0;
        WasmGuitarInputFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * Process an audio block and return MIDI events as a JSON string.
     * Input: Float32Array of mono audio samples.
     * Output: JSON array of event objects.
     * @param {Float32Array} samples
     * @returns {string}
     */
    process_block(samples) {
        let deferred2_0;
        let deferred2_1;
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            const ptr0 = passArrayF32ToWasm0(samples, wasm.__wbindgen_export);
            const len0 = WASM_VECTOR_LEN;
            wasm.wasmguitarinput_process_block(retptr, this.__wbg_ptr, ptr0, len0);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            deferred2_0 = r0;
            deferred2_1 = r1;
            return getStringFromWasm0(r0, r1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_export4(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * Enable/disable pitch bend detection.
     * @param {boolean} val
     */
    set_bends_enabled(val) {
        wasm.wasmguitarinput_set_bends_enabled(this.__wbg_ptr, val);
    }
    /**
     * Set input gain (default 1.0).
     * @param {number} val
     */
    set_input_gain(val) {
        wasm.wasmguitarinput_set_input_gain(this.__wbg_ptr, val);
    }
    /**
     * Enable/disable legato detection.
     * @param {boolean} val
     */
    set_legato_enabled(val) {
        wasm.wasmguitarinput_set_legato_enabled(this.__wbg_ptr, val);
    }
    /**
     * Set onset threshold (default 0.015).
     * @param {number} val
     */
    set_onset_threshold(val) {
        wasm.wasmguitarinput_set_onset_threshold(this.__wbg_ptr, val);
    }
    /**
     * Enable/disable slide detection.
     * @param {boolean} val
     */
    set_slides_enabled(val) {
        wasm.wasmguitarinput_set_slides_enabled(this.__wbg_ptr, val);
    }
    /**
     * Set string confidence minimum (default 0.4).
     * @param {number} val
     */
    set_string_confidence(val) {
        wasm.wasmguitarinput_set_string_confidence(this.__wbg_ptr, val);
    }
    /**
     * Enable/disable vibrato detection.
     * @param {boolean} val
     */
    set_vibrato_enabled(val) {
        wasm.wasmguitarinput_set_vibrato_enabled(this.__wbg_ptr, val);
    }
}
if (Symbol.dispose) WasmGuitarInput.prototype[Symbol.dispose] = WasmGuitarInput.prototype.free;

export function init_panic_hook() {
    wasm.init_panic_hook();
}

/**
 * Convert a MIDI note number (0-127) to its note name (e.g. 60 -> "C4").
 * @param {number} midi
 * @returns {string}
 */
export function midi_to_name(midi) {
    let deferred1_0;
    let deferred1_1;
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        wasm.midi_to_name(retptr, midi);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        deferred1_0 = r0;
        deferred1_1 = r1;
        return getStringFromWasm0(r0, r1);
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
        wasm.__wbindgen_export4(deferred1_0, deferred1_1, 1);
    }
}

function __wbg_get_imports() {
    const import0 = {
        __proto__: null,
        __wbg_Error_8c4e43fe74559d73: function(arg0, arg1) {
            const ret = Error(getStringFromWasm0(arg0, arg1));
            return addHeapObject(ret);
        },
        __wbg_Number_04624de7d0e8332d: function(arg0) {
            const ret = Number(getObject(arg0));
            return ret;
        },
        __wbg_String_8f0eb39a4a4c2f66: function(arg0, arg1) {
            const ret = String(getObject(arg1));
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg___wbindgen_bigint_get_as_i64_8fcf4ce7f1ca72a2: function(arg0, arg1) {
            const v = getObject(arg1);
            const ret = typeof(v) === 'bigint' ? v : undefined;
            getDataViewMemory0().setBigInt64(arg0 + 8 * 1, isLikeNone(ret) ? BigInt(0) : ret, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, !isLikeNone(ret), true);
        },
        __wbg___wbindgen_boolean_get_bbbb1c18aa2f5e25: function(arg0) {
            const v = getObject(arg0);
            const ret = typeof(v) === 'boolean' ? v : undefined;
            return isLikeNone(ret) ? 0xFFFFFF : ret ? 1 : 0;
        },
        __wbg___wbindgen_debug_string_0bc8482c6e3508ae: function(arg0, arg1) {
            const ret = debugString(getObject(arg1));
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg___wbindgen_in_47fa6863be6f2f25: function(arg0, arg1) {
            const ret = getObject(arg0) in getObject(arg1);
            return ret;
        },
        __wbg___wbindgen_is_bigint_31b12575b56f32fc: function(arg0) {
            const ret = typeof(getObject(arg0)) === 'bigint';
            return ret;
        },
        __wbg___wbindgen_is_function_0095a73b8b156f76: function(arg0) {
            const ret = typeof(getObject(arg0)) === 'function';
            return ret;
        },
        __wbg___wbindgen_is_object_5ae8e5880f2c1fbd: function(arg0) {
            const val = getObject(arg0);
            const ret = typeof(val) === 'object' && val !== null;
            return ret;
        },
        __wbg___wbindgen_is_string_cd444516edc5b180: function(arg0) {
            const ret = typeof(getObject(arg0)) === 'string';
            return ret;
        },
        __wbg___wbindgen_is_undefined_9e4d92534c42d778: function(arg0) {
            const ret = getObject(arg0) === undefined;
            return ret;
        },
        __wbg___wbindgen_jsval_eq_11888390b0186270: function(arg0, arg1) {
            const ret = getObject(arg0) === getObject(arg1);
            return ret;
        },
        __wbg___wbindgen_jsval_loose_eq_9dd77d8cd6671811: function(arg0, arg1) {
            const ret = getObject(arg0) == getObject(arg1);
            return ret;
        },
        __wbg___wbindgen_number_get_8ff4255516ccad3e: function(arg0, arg1) {
            const obj = getObject(arg1);
            const ret = typeof(obj) === 'number' ? obj : undefined;
            getDataViewMemory0().setFloat64(arg0 + 8 * 1, isLikeNone(ret) ? 0 : ret, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, !isLikeNone(ret), true);
        },
        __wbg___wbindgen_string_get_72fb696202c56729: function(arg0, arg1) {
            const obj = getObject(arg1);
            const ret = typeof(obj) === 'string' ? obj : undefined;
            var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            var len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg___wbindgen_throw_be289d5034ed271b: function(arg0, arg1) {
            throw new Error(getStringFromWasm0(arg0, arg1));
        },
        __wbg_call_389efe28435a9388: function() { return handleError(function (arg0, arg1) {
            const ret = getObject(arg0).call(getObject(arg1));
            return addHeapObject(ret);
        }, arguments); },
        __wbg_call_4708e0c13bdc8e95: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = getObject(arg0).call(getObject(arg1), getObject(arg2));
            return addHeapObject(ret);
        }, arguments); },
        __wbg_crypto_86f2631e91b51511: function(arg0) {
            const ret = getObject(arg0).crypto;
            return addHeapObject(ret);
        },
        __wbg_error_7534b8e9a36f1ab4: function(arg0, arg1) {
            let deferred0_0;
            let deferred0_1;
            try {
                deferred0_0 = arg0;
                deferred0_1 = arg1;
                console.error(getStringFromWasm0(arg0, arg1));
            } finally {
                wasm.__wbindgen_export4(deferred0_0, deferred0_1, 1);
            }
        },
        __wbg_getRandomValues_b3f15fcbfabb0f8b: function() { return handleError(function (arg0, arg1) {
            getObject(arg0).getRandomValues(getObject(arg1));
        }, arguments); },
        __wbg_get_with_ref_key_1dc361bd10053bfe: function(arg0, arg1) {
            const ret = getObject(arg0)[getObject(arg1)];
            return addHeapObject(ret);
        },
        __wbg_instanceof_ArrayBuffer_c367199e2fa2aa04: function(arg0) {
            let result;
            try {
                result = getObject(arg0) instanceof ArrayBuffer;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_Uint8Array_9b9075935c74707c: function(arg0) {
            let result;
            try {
                result = getObject(arg0) instanceof Uint8Array;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_isSafeInteger_bfbc7332a9768d2a: function(arg0) {
            const ret = Number.isSafeInteger(getObject(arg0));
            return ret;
        },
        __wbg_length_32ed9a279acd054c: function(arg0) {
            const ret = getObject(arg0).length;
            return ret;
        },
        __wbg_log_6b5ca2e6124b2808: function(arg0) {
            console.log(getObject(arg0));
        },
        __wbg_msCrypto_d562bbe83e0d4b91: function(arg0) {
            const ret = getObject(arg0).msCrypto;
            return addHeapObject(ret);
        },
        __wbg_new_361308b2356cecd0: function() {
            const ret = new Object();
            return addHeapObject(ret);
        },
        __wbg_new_3eb36ae241fe6f44: function() {
            const ret = new Array();
            return addHeapObject(ret);
        },
        __wbg_new_8a6f238a6ece86ea: function() {
            const ret = new Error();
            return addHeapObject(ret);
        },
        __wbg_new_dd2b680c8bf6ae29: function(arg0) {
            const ret = new Uint8Array(getObject(arg0));
            return addHeapObject(ret);
        },
        __wbg_new_no_args_1c7c842f08d00ebb: function(arg0, arg1) {
            const ret = new Function(getStringFromWasm0(arg0, arg1));
            return addHeapObject(ret);
        },
        __wbg_new_with_length_a2c39cbe88fd8ff1: function(arg0) {
            const ret = new Uint8Array(arg0 >>> 0);
            return addHeapObject(ret);
        },
        __wbg_node_e1f24f89a7336c2e: function(arg0) {
            const ret = getObject(arg0).node;
            return addHeapObject(ret);
        },
        __wbg_process_3975fd6c72f520aa: function(arg0) {
            const ret = getObject(arg0).process;
            return addHeapObject(ret);
        },
        __wbg_prototypesetcall_bdcdcc5842e4d77d: function(arg0, arg1, arg2) {
            Uint8Array.prototype.set.call(getArrayU8FromWasm0(arg0, arg1), getObject(arg2));
        },
        __wbg_randomFillSync_f8c153b79f285817: function() { return handleError(function (arg0, arg1) {
            getObject(arg0).randomFillSync(takeObject(arg1));
        }, arguments); },
        __wbg_require_b74f47fc2d022fd6: function() { return handleError(function () {
            const ret = module.require;
            return addHeapObject(ret);
        }, arguments); },
        __wbg_set_3f1d0b984ed272ed: function(arg0, arg1, arg2) {
            getObject(arg0)[takeObject(arg1)] = takeObject(arg2);
        },
        __wbg_set_f43e577aea94465b: function(arg0, arg1, arg2) {
            getObject(arg0)[arg1 >>> 0] = takeObject(arg2);
        },
        __wbg_stack_0ed75d68575b0f3c: function(arg0, arg1) {
            const ret = getObject(arg1).stack;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_static_accessor_GLOBAL_12837167ad935116: function() {
            const ret = typeof global === 'undefined' ? null : global;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        },
        __wbg_static_accessor_GLOBAL_THIS_e628e89ab3b1c95f: function() {
            const ret = typeof globalThis === 'undefined' ? null : globalThis;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        },
        __wbg_static_accessor_SELF_a621d3dfbb60d0ce: function() {
            const ret = typeof self === 'undefined' ? null : self;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        },
        __wbg_static_accessor_WINDOW_f8727f0cf888e0bd: function() {
            const ret = typeof window === 'undefined' ? null : window;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        },
        __wbg_subarray_a96e1fef17ed23cb: function(arg0, arg1, arg2) {
            const ret = getObject(arg0).subarray(arg1 >>> 0, arg2 >>> 0);
            return addHeapObject(ret);
        },
        __wbg_versions_4e31226f5e8dc909: function(arg0) {
            const ret = getObject(arg0).versions;
            return addHeapObject(ret);
        },
        __wbindgen_cast_0000000000000001: function(arg0) {
            // Cast intrinsic for `F64 -> Externref`.
            const ret = arg0;
            return addHeapObject(ret);
        },
        __wbindgen_cast_0000000000000002: function(arg0, arg1) {
            // Cast intrinsic for `Ref(Slice(U8)) -> NamedExternref("Uint8Array")`.
            const ret = getArrayU8FromWasm0(arg0, arg1);
            return addHeapObject(ret);
        },
        __wbindgen_cast_0000000000000003: function(arg0, arg1) {
            // Cast intrinsic for `Ref(String) -> Externref`.
            const ret = getStringFromWasm0(arg0, arg1);
            return addHeapObject(ret);
        },
        __wbindgen_cast_0000000000000004: function(arg0) {
            // Cast intrinsic for `U64 -> Externref`.
            const ret = BigInt.asUintN(64, arg0);
            return addHeapObject(ret);
        },
        __wbindgen_object_clone_ref: function(arg0) {
            const ret = getObject(arg0);
            return addHeapObject(ret);
        },
        __wbindgen_object_drop_ref: function(arg0) {
            takeObject(arg0);
        },
    };
    return {
        __proto__: null,
        "./contrapunk_wasm_bg.js": import0,
    };
}

const EngineFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_engine_free(ptr >>> 0, 1));
const WasmGuitarInputFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_wasmguitarinput_free(ptr >>> 0, 1));

function addHeapObject(obj) {
    if (heap_next === heap.length) heap.push(heap.length + 1);
    const idx = heap_next;
    heap_next = heap[idx];

    heap[idx] = obj;
    return idx;
}

function debugString(val) {
    // primitive types
    const type = typeof val;
    if (type == 'number' || type == 'boolean' || val == null) {
        return  `${val}`;
    }
    if (type == 'string') {
        return `"${val}"`;
    }
    if (type == 'symbol') {
        const description = val.description;
        if (description == null) {
            return 'Symbol';
        } else {
            return `Symbol(${description})`;
        }
    }
    if (type == 'function') {
        const name = val.name;
        if (typeof name == 'string' && name.length > 0) {
            return `Function(${name})`;
        } else {
            return 'Function';
        }
    }
    // objects
    if (Array.isArray(val)) {
        const length = val.length;
        let debug = '[';
        if (length > 0) {
            debug += debugString(val[0]);
        }
        for(let i = 1; i < length; i++) {
            debug += ', ' + debugString(val[i]);
        }
        debug += ']';
        return debug;
    }
    // Test for built-in
    const builtInMatches = /\[object ([^\]]+)\]/.exec(toString.call(val));
    let className;
    if (builtInMatches && builtInMatches.length > 1) {
        className = builtInMatches[1];
    } else {
        // Failed to match the standard '[object ClassName]'
        return toString.call(val);
    }
    if (className == 'Object') {
        // we're a user defined class or Object
        // JSON.stringify avoids problems with cycles, and is generally much
        // easier than looping through ownProperties of `val`.
        try {
            return 'Object(' + JSON.stringify(val) + ')';
        } catch (_) {
            return 'Object';
        }
    }
    // errors
    if (val instanceof Error) {
        return `${val.name}: ${val.message}\n${val.stack}`;
    }
    // TODO we could test for more things here, like `Set`s and `Map`s.
    return className;
}

function dropObject(idx) {
    if (idx < 132) return;
    heap[idx] = heap_next;
    heap_next = idx;
}

function getArrayU8FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getUint8ArrayMemory0().subarray(ptr / 1, ptr / 1 + len);
}

let cachedDataViewMemory0 = null;
function getDataViewMemory0() {
    if (cachedDataViewMemory0 === null || cachedDataViewMemory0.buffer.detached === true || (cachedDataViewMemory0.buffer.detached === undefined && cachedDataViewMemory0.buffer !== wasm.memory.buffer)) {
        cachedDataViewMemory0 = new DataView(wasm.memory.buffer);
    }
    return cachedDataViewMemory0;
}

let cachedFloat32ArrayMemory0 = null;
function getFloat32ArrayMemory0() {
    if (cachedFloat32ArrayMemory0 === null || cachedFloat32ArrayMemory0.byteLength === 0) {
        cachedFloat32ArrayMemory0 = new Float32Array(wasm.memory.buffer);
    }
    return cachedFloat32ArrayMemory0;
}

function getStringFromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return decodeText(ptr, len);
}

let cachedUint8ArrayMemory0 = null;
function getUint8ArrayMemory0() {
    if (cachedUint8ArrayMemory0 === null || cachedUint8ArrayMemory0.byteLength === 0) {
        cachedUint8ArrayMemory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachedUint8ArrayMemory0;
}

function getObject(idx) { return heap[idx]; }

function handleError(f, args) {
    try {
        return f.apply(this, args);
    } catch (e) {
        wasm.__wbindgen_export3(addHeapObject(e));
    }
}

let heap = new Array(128).fill(undefined);
heap.push(undefined, null, true, false);

let heap_next = heap.length;

function isLikeNone(x) {
    return x === undefined || x === null;
}

function passArrayF32ToWasm0(arg, malloc) {
    const ptr = malloc(arg.length * 4, 4) >>> 0;
    getFloat32ArrayMemory0().set(arg, ptr / 4);
    WASM_VECTOR_LEN = arg.length;
    return ptr;
}

function passStringToWasm0(arg, malloc, realloc) {
    if (realloc === undefined) {
        const buf = cachedTextEncoder.encode(arg);
        const ptr = malloc(buf.length, 1) >>> 0;
        getUint8ArrayMemory0().subarray(ptr, ptr + buf.length).set(buf);
        WASM_VECTOR_LEN = buf.length;
        return ptr;
    }

    let len = arg.length;
    let ptr = malloc(len, 1) >>> 0;

    const mem = getUint8ArrayMemory0();

    let offset = 0;

    for (; offset < len; offset++) {
        const code = arg.charCodeAt(offset);
        if (code > 0x7F) break;
        mem[ptr + offset] = code;
    }
    if (offset !== len) {
        if (offset !== 0) {
            arg = arg.slice(offset);
        }
        ptr = realloc(ptr, len, len = offset + arg.length * 3, 1) >>> 0;
        const view = getUint8ArrayMemory0().subarray(ptr + offset, ptr + len);
        const ret = cachedTextEncoder.encodeInto(arg, view);

        offset += ret.written;
        ptr = realloc(ptr, len, offset, 1) >>> 0;
    }

    WASM_VECTOR_LEN = offset;
    return ptr;
}

function takeObject(idx) {
    const ret = getObject(idx);
    dropObject(idx);
    return ret;
}

let cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
cachedTextDecoder.decode();
const MAX_SAFARI_DECODE_BYTES = 2146435072;
let numBytesDecoded = 0;
function decodeText(ptr, len) {
    numBytesDecoded += len;
    if (numBytesDecoded >= MAX_SAFARI_DECODE_BYTES) {
        cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
        cachedTextDecoder.decode();
        numBytesDecoded = len;
    }
    return cachedTextDecoder.decode(getUint8ArrayMemory0().subarray(ptr, ptr + len));
}

const cachedTextEncoder = new TextEncoder();

if (!('encodeInto' in cachedTextEncoder)) {
    cachedTextEncoder.encodeInto = function (arg, view) {
        const buf = cachedTextEncoder.encode(arg);
        view.set(buf);
        return {
            read: arg.length,
            written: buf.length
        };
    };
}

let WASM_VECTOR_LEN = 0;

let wasmModule, wasm;
function __wbg_finalize_init(instance, module) {
    wasm = instance.exports;
    wasmModule = module;
    cachedDataViewMemory0 = null;
    cachedFloat32ArrayMemory0 = null;
    cachedUint8ArrayMemory0 = null;
    wasm.__wbindgen_start();
    return wasm;
}

async function __wbg_load(module, imports) {
    if (typeof Response === 'function' && module instanceof Response) {
        if (typeof WebAssembly.instantiateStreaming === 'function') {
            try {
                return await WebAssembly.instantiateStreaming(module, imports);
            } catch (e) {
                const validResponse = module.ok && expectedResponseType(module.type);

                if (validResponse && module.headers.get('Content-Type') !== 'application/wasm') {
                    console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve Wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);

                } else { throw e; }
            }
        }

        const bytes = await module.arrayBuffer();
        return await WebAssembly.instantiate(bytes, imports);
    } else {
        const instance = await WebAssembly.instantiate(module, imports);

        if (instance instanceof WebAssembly.Instance) {
            return { instance, module };
        } else {
            return instance;
        }
    }

    function expectedResponseType(type) {
        switch (type) {
            case 'basic': case 'cors': case 'default': return true;
        }
        return false;
    }
}

function initSync(module) {
    if (wasm !== undefined) return wasm;


    if (module !== undefined) {
        if (Object.getPrototypeOf(module) === Object.prototype) {
            ({module} = module)
        } else {
            console.warn('using deprecated parameters for `initSync()`; pass a single object instead')
        }
    }

    const imports = __wbg_get_imports();
    if (!(module instanceof WebAssembly.Module)) {
        module = new WebAssembly.Module(module);
    }
    const instance = new WebAssembly.Instance(module, imports);
    return __wbg_finalize_init(instance, module);
}

async function __wbg_init(module_or_path) {
    if (wasm !== undefined) return wasm;


    if (module_or_path !== undefined) {
        if (Object.getPrototypeOf(module_or_path) === Object.prototype) {
            ({module_or_path} = module_or_path)
        } else {
            console.warn('using deprecated parameters for the initialization function; pass a single object instead')
        }
    }

    if (module_or_path === undefined) {
        module_or_path = new URL('contrapunk_wasm_bg.wasm', import.meta.url);
    }
    const imports = __wbg_get_imports();

    if (typeof module_or_path === 'string' || (typeof Request === 'function' && module_or_path instanceof Request) || (typeof URL === 'function' && module_or_path instanceof URL)) {
        module_or_path = fetch(module_or_path);
    }

    const { instance, module } = await __wbg_load(await module_or_path, imports);

    return __wbg_finalize_init(instance, module);
}

export { initSync, __wbg_init as default };
