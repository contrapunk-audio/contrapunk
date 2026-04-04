# Stack — Contrapunk
**Generated:** 2026-04-04
## Languages
Rust (Edition 2021) — Core library, Tauri backend, WASM bindings
TypeScript (ES2022) — SvelteKit frontend
## Platforms
Desktop: Tauri v2 (WebView + Rust backend)
Browser: SvelteKit static site + WASM, deployed to Fly.io
CLI: Native Rust binary (server/client mode)
## Frameworks
Tauri 2.0.0, SvelteKit ^2.0.0, Svelte ^5.0.0, Vite ^6.0.0
## Key Rust Dependencies
wmidi 4.0 (MIDI types), midir 0.10 (native MIDI), cpal 0.15 (native audio),
pitch-detection 0.3 (McLeod, all targets), clap 4 (CLI), serde/serde_json 1.0,
rmp-serde 1.3 (wire protocol), wasm-bindgen 0.2, web-sys 0.3, rand 0.8, anyhow 1.0
## Workspace
members: ".", "src-tauri", "wasm" (resolver = "2")
## Feature Flags
web-midi: wasm-bindgen, js-sys, web-sys, getrandom (browser MIDI)
cfg(not(wasm32)): guards midir, cpal, clap, rmp-serde
