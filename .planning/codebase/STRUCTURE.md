# Structure — Contrapunk
**Generated:** 2026-04-04
## Layout
src/ — Core Rust library (harmony/, audio/, humanize/, generator/, chord.rs, midi/, router.rs, server/)
src-tauri/ — Tauri v2 backend (commands/, state.rs, guitar_bridge.rs)
wasm/ — WASM bindings (WasmHarmonyEngine, WasmGuitarInput)
ui/ — SvelteKit frontend (adapter/, stores/, components/, audio/, theme/)
tests/ — Integration tests (audio_pipeline.rs)
examples/ — Guitar demos (guitar_input_demo.rs, etc.)
deploy/ — Dockerfile, nginx.conf
.planning/ — Roadmap, phases, codebase docs
## Key Files
src/harmony/engine.rs — HarmonyEngine
src/audio/guitar_input.rs — Guitar DSP (2100+ lines)
src/chord.rs — Chord detection (40+ patterns)
src-tauri/src/commands/engine.rs — MIDI routing + note state
ui/src/lib/adapter/index.ts — Platform detection
ui/src/lib/stores/ — Reactive state
## Naming
Rust: snake_case files, PascalCase types, SCREAMING_SNAKE constants
Svelte: PascalCase.svelte components, camelCase.svelte.ts stores
Tauri commands: snake_case
