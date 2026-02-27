// @ts-nocheck
/**
 * Stub WASM package for development without wasm-pack.
 * This file is replaced by wasm-pack output during production builds.
 * Tracks state so the UI is fully interactive in dev mode.
 */
export default function init() { return Promise.resolve(); }

export class Engine {
  constructor() {
    console.info('[contrapunk-wasm] Using stub Engine (dev mode).');
    this._key = 'C';
    this._mode = 'PassThrough';
    this._scale_mode = 'Ionian';
    this._octave_mode = 'None';
    this._voice_leading_enabled = false;
    this._voice_leading_style = 'Free';
    this._interchange_enabled = false;
    this._borrowing_range = 3;
    this._voice_position = 0;
    this._voice_count = 2;
    this._input_notes = new Set();
    this._harmony_notes = new Set();
  }

  set_key(k) { this._key = k; }
  set_mode(m) { this._mode = m; }
  set_scale_mode(m) { this._scale_mode = m; }
  set_octave_mode(m) { this._octave_mode = m; }
  set_voice_leading(enabled, style) {
    this._voice_leading_enabled = enabled;
    if (style) this._voice_leading_style = style;
  }
  set_interchange(enabled, range) {
    this._interchange_enabled = enabled;
    if (range !== undefined) this._borrowing_range = range;
  }
  set_voice_position(p) { this._voice_position = p; }

  note_on(n) {
    this._input_notes.add(n);
    // Stub harmony: add a third above (4 semitones) when not in PassThrough
    if (this._mode !== 'PassThrough') {
      const harmony = n + 4;
      if (harmony <= 127) this._harmony_notes.add(harmony);
      return [n, harmony];
    }
    return [n];
  }

  note_off(n) {
    this._input_notes.delete(n);
    const harmony = n + 4;
    this._harmony_notes.delete(harmony);
    if (this._mode !== 'PassThrough') {
      return [n, harmony];
    }
    return [n];
  }

  harmonize(n) { return this.note_on(n); }

  get_state() {
    return {
      key: this._key,
      mode: this._mode,
      mode_number: 1,
      scale_mode: this._scale_mode,
      octave_mode: this._octave_mode,
      voice_leading_enabled: this._voice_leading_enabled,
      voice_leading_style: this._voice_leading_style,
      interchange_enabled: this._interchange_enabled,
      borrowing_range: this._borrowing_range,
      voice_position: this._voice_position,
      voice_count: this._voice_count,
    };
  }

  get_note_state() {
    return {
      input_notes: Array.from(this._input_notes),
      harmony_notes: Array.from(this._harmony_notes),
      borrowed_notes: [],
      chord_name: this._input_notes.size > 0 ? this._detectChord() : '',
      last_borrowed_from: '',
    };
  }

  _detectChord() {
    const notes = Array.from(this._input_notes).map(n => n % 12).sort((a, b) => a - b);
    if (notes.length === 0) return '';
    const names = ['C','C#','D','D#','E','F','F#','G','G#','A','A#','B'];
    if (notes.length === 1) return names[notes[0]];
    return notes.map(n => names[n]).join('+');
  }

  list_presets() { return []; }
  load_preset() {}
  save_preset() {}
  delete_preset() {}
}

export function midi_to_name(midi) {
  const names = ['C','C#','D','D#','E','F','F#','G','G#','A','A#','B'];
  return names[midi % 12] + (Math.floor(midi / 12) - 1);
}
