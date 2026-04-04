# Architecture — Contrapunk
**Generated:** 2026-04-04
## Pattern
Layered architecture with platform adapters. Core = pure Rust library.
Three frontends: Tauri desktop, WASM browser, CLI.
## Layers
1. Core Library (src/) — Harmony engine, audio DSP, humanizer, generator, chord, MIDI, router
2. Tauri Backend (src-tauri/) — IPC commands, AppState (Arc<Mutex<T>>)
3. WASM Bindings (wasm/) — wasm-bindgen exports
4. Frontend (ui/) — SvelteKit + Svelte 5 runes
5. Platform Adapter (ui/src/lib/adapter/) — Tauri vs WASM detection
## Data Flow: MIDI
Input -> mpsc::channel -> Router Thread -> HarmonyEngine -> Humanizer -> OutputRouter -> MIDI Out
## Data Flow: Guitar
cpal/getUserMedia -> GuitarInput::process_block() -> Onset -> Pitch -> State Machine -> MidiEvent -> Router
## Key Abstractions
HarmonyEngine (src/harmony/engine.rs), GuitarInput (src/audio/guitar_input.rs),
OutputRouter (src/midi/output.rs), Humanizer (src/humanize/mod.rs),
Scale (src/harmony/scale.rs), PlatformAdapter (ui/src/lib/adapter/)
