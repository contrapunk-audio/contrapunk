# Technology Stack

**Analysis Date:** 2026-03-31

## Languages

**Primary:**
- Rust (edition 2021) - Core harmony engine, WASM bridge, Tauri desktop backend, server/client CLI
- TypeScript (^5.0.0) - SvelteKit UI, adapter layer, Svelte stores

**Secondary:**
- Python (≥3.x) - ML data pipeline: audio feature extraction, dataset loading, classifier training
- Shell - WASM build script (`ui/scripts/build-wasm.sh`)

## Runtime

**Environment:**
- Browser (WASM target: `wasm32-unknown-unknown`) - primary end-user deployment
- Desktop via Tauri v2 webview (macOS, Windows, Linux)
- Native CLI binary (server/client modes, audio capture)
- Node.js 22 - frontend build tooling only

**Package Manager:**
- npm (with `package-lock.json`)
- Lockfile: present (`ui/package-lock.json`)
- Cargo - Rust dependency manager
- Lockfile: present (`Cargo.lock`)

## Frameworks

**Core:**
- SvelteKit ^2.0.0 - Frontend application framework (`ui/`)
- Svelte ^5.0.0 - Component framework (uses Svelte 5 runes: `$state`, `$effect`)
- Tauri v2 - Desktop app shell wrapping the SvelteKit UI (`src-tauri/`)
- eframe / egui - Legacy desktop GUI (referenced in `target/doc/`; no longer active entrypoint)

**Build/Dev:**
- Vite ^6.0.0 - Frontend bundler (`ui/vite.config.ts`)
- vite-plugin-wasm ^3.0.0 - Loads `.wasm` modules in Vite
- vite-plugin-top-level-await ^1.0.0 - Enables top-level await needed for WASM init
- wasm-pack - Compiles `wasm/` crate to browser-ready `.wasm` + JS bindings
- `@sveltejs/adapter-static` ^3.0.0 - Outputs static SPA (`ui/svelte.config.js`); SPA fallback configured

**Styling:**
- Tailwind CSS ^4.0.0 - Utility-first CSS (Vite plugin: `@tailwindcss/vite`)

**ML Pipeline:**
- librosa ≥0.10 - Audio feature extraction
- scikit-learn ≥1.3 - ML model training
- numpy ≥1.24 - Numerical arrays
- soundfile ≥0.12 - Audio I/O
- matplotlib ≥3.7, seaborn ≥0.12 - Visualization
- umap-learn ≥0.5 - Dimensionality reduction

## Key Dependencies

**Critical (Rust core):**
- `wmidi` 4.0 - MIDI message parsing/encoding (used in both native and WASM targets)
- `midir` 0.10 - Native MIDI I/O (conditional: `cfg(not(target_arch = "wasm32"))`)
- `cpal` 0.15 - Cross-platform audio capture (native only)
- `pitch-detection` 0.3 - Audio pitch detection algorithms (native only)
- `wasm-bindgen` 0.2 - Rust-to-JS FFI bridge (`wasm/` crate)
- `serde-wasm-bindgen` 0.6 - Serde serialization over wasm-bindgen
- `web-sys` 0.3 - Browser Web MIDI API bindings (features: MidiAccess, MidiInput, MidiOutput, MidiMessageEvent, etc.)
- `js-sys` 0.3 - JavaScript primitive bindings
- `getrandom` 0.2 with `js` feature - Random number generation in WASM
- `rand` 0.8 - Humanization randomness
- `rmp-serde` 1.3 - MessagePack serialization for ML training dataset (native only)
- `anyhow` 1.0 - Error propagation throughout the codebase

**Infrastructure (Rust):**
- `clap` 4 with `derive` feature - CLI argument parsing (`src/main.rs`)
- `console_error_panic_hook` 0.1 - Better WASM panic messages in browser console
- `serde` 1.0 + `serde_json` 1.0 - JSON serialization for Tauri IPC and presets

**Frontend:**
- `@tauri-apps/api` ^2.0.0 - `invoke()` and `listen()` for Tauri IPC (imported only when `window.__TAURI__` is present)

## Configuration

**Environment:**
- No `.env` files detected - no runtime secrets required
- Fly.io deployment requires `FLY_API_TOKEN` as a GitHub Actions secret (`secrets.FLY_API_TOKEN`)

**Build:**
- `ui/vite.config.ts` - Vite config; excludes `$lib/wasm-pkg` from dep optimization
- `ui/svelte.config.js` - SvelteKit static adapter with SPA fallback (`index.html`)
- `ui/tsconfig.json` - TypeScript strict mode, `moduleResolution: bundler`, excludes `src/lib/wasm-pkg/**`
- `Cargo.toml` (workspace root) - Workspace members: `.` (core), `src-tauri`, `wasm`; release profile: `opt-level = "s"`, `strip = true`, `panic = "abort"`
- `wasm/Cargo.toml` - Release profile: `lto = true`; crate-type `["cdylib", "rlib"]`

## Platform Requirements

**Development:**
- Rust stable toolchain + `wasm32-unknown-unknown` target
- wasm-pack (for local builds; falls back to JS stub if absent; see `ui/scripts/build-wasm.sh`)
- Node.js 22 + npm
- Python ≥3.x + venv (for `ml/` pipeline only)
- Native MIDI: `libasound2-dev` (Linux); CoreMIDI (macOS, bundled)
- Audio capture: `cpal` requires ALSA dev headers on Linux

**Production:**
- Docker multi-stage build (`deploy/Dockerfile`): stage 1 uses `rust:1.88` + wasm-pack, stage 2 uses `node:22`, stage 3 uses `nginx:alpine`
- Deployed as a static SPA served by nginx on port 8080
- Tauri desktop: platform-specific bundles built separately via `cargo tauri build`

---

*Stack analysis: 2026-03-31*
