#!/bin/sh
# Build WASM package using wasm-pack, or create a stub for development.
# Called by npm run build:wasm from ui/package.json.

set -e

WASM_PKG_DIR="$(dirname "$0")/../src/lib/wasm-pkg"

if command -v wasm-pack >/dev/null 2>&1; then
  echo "[build-wasm] Building with wasm-pack..."
  cd "$(dirname "$0")/../.."
  wasm-pack build wasm/ --target web --out-dir ../ui/src/lib/wasm-pkg
  echo "[build-wasm] WASM build complete."
else
  echo "[build-wasm] wasm-pack not found. Creating development stub..."
  mkdir -p "$WASM_PKG_DIR"

  # Only create stub if no real wasm-pack output exists
  if [ ! -f "$WASM_PKG_DIR/contrapunk_wasm_bg.wasm" ]; then
    cat > "$WASM_PKG_DIR/index.js" << 'STUB'
// @ts-nocheck
/**
 * Stub WASM package for development without wasm-pack.
 * This file is replaced by wasm-pack output during production builds.
 */
export default function init() { return Promise.resolve(); }

export class Engine {
  constructor() { console.warn('[contrapunk-wasm] Using stub Engine. Run wasm-pack for real WASM.'); }
  set_key() {} set_mode() {} set_scale_mode() {} set_octave_mode() {}
  set_voice_leading() {} set_interchange() {} set_voice_position() {}
  harmonize(n) { return [n]; } note_on(n) { return [n]; } note_on_channel(n) { return [n]; }
  note_off(n) { return [n]; } note_off_channel(n) { return [n]; }
  get_state() { return { key:'C', mode:'PassThrough', mode_number:1, scale_mode:'Ionian', octave_mode:'None', voice_leading_enabled:false, voice_leading_style:'Free', interchange_enabled:false, borrowing_range:3, voice_position:0, voice_count:2 }; }
  get_note_state() { return { input_notes:[], harmony_notes:[], borrowed_notes:[], chord_name:'', last_borrowed_from:'' }; }
  list_presets() { return []; } load_preset() {} save_preset() {} delete_preset() {}
}

export function midi_to_name(midi) {
  const names = ['C','C#','D','D#','E','F','F#','G','G#','A','A#','B'];
  return names[midi % 12] + (Math.floor(midi / 12) - 1);
}
STUB
    echo "[build-wasm] Stub created at $WASM_PKG_DIR/index.js"
  else
    echo "[build-wasm] Real WASM output already exists, skipping stub."
  fi
fi
