# Technology Stack

**Analysis Date:** 2026-02-04

## Languages

**Primary:**
- Rust (Edition 2021) - Core application logic, MIDI processing, harmony generation

**Secondary:**
- HTML/CSS - WebAssembly UI container (`index.html`)

## Runtime

**Environment:**
- Rust 1.93.0 (native compilation)
- WebAssembly (wasm32-unknown-unknown target for browser deployment)

**Package Manager:**
- Cargo 1.93.0
- Lockfile: present (`Cargo.lock`, 113KB)

## Frameworks

**Core:**
- eframe 0.33 - GUI framework for both native and web builds, features: persistence
- wmidi 4.0 - MIDI message parsing and generation

**Testing:**
- Built-in Rust test framework - Unit tests in `src/server/protocol.rs`
- cargo test - Test runner (no external framework detected)

**Build/Dev:**
- Trunk - WASM bundler and dev server (configured in `Trunk.toml`)
- wasm-bindgen 0.2 - Rust/JavaScript interop for WebAssembly
- cargo (native builds)

## Key Dependencies

**Critical:**
- wmidi 4.0 - MIDI message encoding/decoding, core to all MIDI operations
- eframe 0.33 - GUI framework for desktop and web, enables cross-platform UI
- anyhow 1.0 - Error handling throughout the application

**Infrastructure:**
- midir 0.10 - Native MIDI device I/O (non-WASM only), used in `src/midi/ports.rs`, `src/midi/input.rs`, `src/midi/output.rs`
- clap 4.x - CLI argument parsing for native builds (features: derive), used in `src/main.rs`
- serde 1.0 + serde_json 1.0 - Configuration serialization/deserialization, preset management in `src/preset/storage.rs`
- rand 0.8 - Randomization for humanization effects in `src/humanize/`

**WASM-Specific:**
- web-sys 0.3 - Browser API access (Web MIDI API, Canvas, Navigator), used in `src/midi/web.rs`, `src/lib.rs`
- js-sys 0.3 - JavaScript type bindings
- wasm-bindgen-futures 0.4 - Async support in WASM
- console_error_panic_hook 0.1 - Better panic messages in browser console
- getrandom 0.2 - WASM-compatible random number generation (features: js)

## Configuration

**Environment:**
- No environment variables required
- All configuration is runtime-selected (MIDI ports, harmony settings, keys)
- Server mode uses CLI flags: `--server`, `--client`, `--port` (default: 9900)

**Build:**
- `Cargo.toml` - Rust package manifest with conditional dependencies
- `Trunk.toml` - WASM build configuration (target: index.html, dist: dist)
- `index.html` - WASM entry point with Trunk directives
- Conditional compilation via feature flags: `gui`, `wasm`
- Target-specific dependencies: `cfg(target_arch = "wasm32")` vs `cfg(not(target_arch = "wasm32"))`

**Release Profile:**
- Optimization: size (`opt-level = "s"`)
- LTO: disabled
- Symbols: stripped
- Panic strategy: abort

## Platform Requirements

**Development:**
- Rust toolchain 1.93.0+ with stable channel
- ALSA development libraries (Linux: libasound2-dev) for native MIDI
- Trunk for WASM builds: `cargo install trunk`
- wasm32-unknown-unknown target: `rustup target add wasm32-unknown-unknown`

**Production:**
- Native: No external dependencies (standalone binary)
- Web: Static file hosting (Nginx, Fly.io)
- WASM deployment: `trunk build --release` generates static files in `dist/`

---

*Stack analysis: 2026-02-04*
