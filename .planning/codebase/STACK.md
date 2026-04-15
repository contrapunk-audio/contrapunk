# Technology Stack

**Analysis Date:** 2026-04-15

## Languages

**Primary:**
- Rust 1.93.0 — Core harmony engine, DSP/audio pipeline, WASM bridge, Tauri backend, plugin
- TypeScript 5.x — SvelteKit frontend (UI layer), adapter abstraction, stores
- Python 3.x — ML training pipeline only (`ml/` directory, not shipped)

**Secondary:**
- CSS (Tailwind v4) — Frontend styling
- JavaScript (generated) — wasm-pack output at `ui/src/lib/wasm-pkg/`
- Shell — Build orchestration (`ui/scripts/build-wasm.sh`, `scripts/pre-commit`)

## Runtime

**Environment:**
- Node.js v22.9.0 — SvelteKit dev server and build tooling
- Rust/Cargo 1.93.0 — All Rust crates, no toolchain pin file present
- Python (venv) — ML training only, `ml/venv/` present locally

**Package Manager:**
- npm 10.8.3 — Frontend (`ui/package-lock.json` present, lockfile committed)
- Cargo — Rust workspace (`Cargo.lock` committed)
- pip — ML Python dependencies (`ml/requirements.txt`)
- Lockfiles: `Cargo.lock` and `ui/package-lock.json` both committed; `ml/` uses `requirements.txt` pinned versions

## Workspace / Crates Structure

Cargo workspace defined in root `Cargo.toml` with these members:

| Crate | Path | Purpose |
|---|---|---|
| `contrapunk` | `.` (root) | Core library: harmony engine, DSP, audio, MIDI, presets |
| `contrapunk-tauri` | `src-tauri/` | Tauri v2 desktop app backend |
| `contrapunk-wasm` | `wasm/` | WebAssembly bridge (`cdylib + rlib`) |
| `contrapunk_plugin` | `plugin/` | VST3/CLAP plugin via nih-plug (`cdylib`) |
| `xtask` | `xtask/` | Build task runner for plugin bundling |

## Frameworks

**Core (Rust):**
- `tauri` 2.10.2 — Desktop app shell with Tauri v2 IPC (`src-tauri/Cargo.toml`)
- `nih_plug` (git fork `contrapunk-audio/nih-plug`) — VST3/CLAP plugin framework (`plugin/Cargo.toml`)
- `nih_plug_webview` (git fork `contrapunk-audio/nih-plug-webview`) — Embedded web UI inside the plugin

**Frontend:**
- SvelteKit 2.x with Svelte 5.x (runes mode) — `ui/package.json`
- Vite 6.x — Build tool (`ui/vite.config.ts`)
- Tailwind CSS v4 — Styling via `@tailwindcss/vite` Vite plugin
- `@sveltejs/adapter-static` 3.x — Outputs static SPA (`ui/svelte.config.js`)

**ML (Python, training only):**
- PyTorch 2.11.0 — CNN training (`ml/requirements.txt`)
- librosa 0.11.0 — Audio feature extraction (mel-spectrograms)
- scikit-learn 1.8.0 — Evaluation utilities
- numpy 2.4.3 — Array operations
- umap-learn 0.5.11 — Dimensionality reduction for visualisation
- sounddevice 0.5.5 / soundfile 0.13.1 — Audio I/O during training

## Key Dependencies

**Audio (Rust, native only — excluded on `wasm32`):**
- `cpal` 0.15.3 — Cross-platform audio I/O (mic capture + audio output)
- `midir` 0.10.3 — MIDI device enumeration and I/O
- `ringbuf` 0.4.8 — Lock-free SPSC ring buffer for audio→router→synth pipeline

**MIDI (Rust, all platforms):**
- `wmidi` 4.0.10 — MIDI message parsing and serialisation

**DSP / Pitch Detection (Rust):**
- `pitch-detection` 0.3.0 — McLeod pitch method (MPM) and BACF implementations
- Custom implementations in `src/audio/`: onset detection (HFC, spectral flux, autocorrelation), Goertzel filter bank, AMDF, single-cycle detector, inharmonicity-based DSP string identification

**ML Inference (Rust, pure — no external ML framework):**
- Hand-written CNN forward pass in `src/audio/inference.rs` — 4-layer Conv2D + BN + pooling network, weights loaded from binary file (`guitar_calibration_profile.json` / `guitar_training_data.msgpack`)
- `rmp-serde` 1.3.1 — MessagePack deserialization for training data (native only)

**WASM:**
- `wasm-bindgen` 0.2.108 — Rust↔JS bridge (`wasm/`)
- `serde-wasm-bindgen` 0.6 — Serde integration for WASM
- `wasm-pack` 0.14.0 — Build tool (`wasm-pack build wasm/ --target web`)

**Serialisation:**
- `serde` 1.0 + `serde_json` 1.0 — JSON everywhere (Tauri IPC payloads, preset files)
- `rmp-serde` 1.3.1 — MessagePack for training dataset blobs (native only)

**Error Handling:**
- `anyhow` 1.0 — All Rust crates use anyhow for error propagation

**Frontend (runtime):**
- `@tauri-apps/api` 2.x — Tauri IPC `invoke()` + `listen()` for desktop adapter
- `posthog-js` 1.364.7 — Product analytics (browser-only, opt-in via env var)

**Build / Dev (frontend):**
- `vite-plugin-wasm` 3.x — Vite WASM module loading
- `vite-plugin-top-level-await` 1.x — Required for WASM async init
- `svelte-check` 4.x — TypeScript + Svelte type checking

## Features / Conditional Compilation

**Rust feature flags (root `Cargo.toml`):**
```toml
[features]
default = []
web-midi = ["wasm-bindgen", "wasm-bindgen-futures", "js-sys", "web-sys", "getrandom"]
```

**`cfg(not(target_arch = "wasm32"))` gates:**
- `midir`, `cpal`, `clap`, `rmp-serde` — all excluded from WASM builds
- `src/midi/input.rs`, `src/midi/output.rs`, `src/server/` — native only
- `audio_out::engine` (cpal stream lifecycle) — native only

**Platform adapter (TypeScript):**
Three runtime implementations at `ui/src/lib/adapter/`:
- `TauriAdapter` — used when `__TAURI_INTERNALS__` detected in window
- `WasmAdapter` — used in browser (uses Web MIDI API + WASM engine)
- `PluginAdapter` — used when `window.plugin.send` detected (nih-plug-webview)

## Configuration

**Environment Variables:**
- `PUBLIC_POSTHOG_KEY` — PostHog analytics key; loaded via `import.meta.env` in `ui/src/routes/+layout.ts`; analytics silently disabled if absent
- No `.env` files present in repo; no other documented env vars

**Build:**
- `src-tauri/tauri.conf.json` — Tauri app config, window sizes, bundle targets (dmg/app/nsis), dev server URL
- `ui/vite.config.ts` — Vite plugins, `esnext` build target
- `ui/svelte.config.js` — Static adapter, Svelte 5 runes enabled, prerender entries
- `ui/tsconfig.json` — `strict: true`, `moduleResolution: bundler`
- `deploy/Dockerfile` — Multi-stage: Rust (wasm-pack) → Node (vite build) → nginx

**Tauri Capabilities:**
- `src-tauri/capabilities/default.json` — Minimal: `core:default` + `core:event:default`; CSP disabled (`"csp": null`)

## Platform Requirements

**Development:**
- Rust 1.93+ with `wasm32-unknown-unknown` target for WASM builds
- `wasm-pack` 0.14.0 (`ui/scripts/build-wasm.sh` generates stub if absent)
- Node.js 22+, npm 10+
- macOS recommended (Tauri with CoreAudio + CoreMIDI); Linux/Windows supported via cpal/midir

**Production / Distribution:**
- Desktop: Tauri bundles to `.dmg` (macOS), `.app` (macOS), `.nsis` installer (Windows)
- Web: Static SPA deployed on Fly.io (`fly.toml`, region `bom`), served via nginx
- Plugin: VST3/CLAP built with nih-plug, bundled via `cargo xtask bundle`

---

*Stack analysis: 2026-04-15*
