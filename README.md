# Contrapunk

Real-time MIDI harmony generator built with Rust.

Connect a MIDI controller, pick a scale and harmony mode, and Contrapunk generates live harmonies alongside your playing. Runs as a native desktop app or in the browser via WebAssembly.

## Try It

Browser version: [contrapunk.fly.dev](https://contrapunk.fly.dev/)

## Features

- **9 harmony modes** -- Pass-through, Diatonic 3rds, Diatonic 5ths, Random Diatonic, Contrary Motion, Strict Counterpoint, Barry Harris, and more
- **28+ scale modes** organized by family (Church modes, Harmonic Minor, Melodic Minor, Exotic, Barry Harris 6th Diminished)
- **Modal interchange** with visual feedback for chromatic note handling
- **Chord detection** -- extended chords, slash chords, add chords, roman numeral analysis
- **Steampunk-themed GUI** with tabbed navigation (Play / Craft / Settings)
- **11+ musical style presets** with character personas
- **Humanization engine** -- timing jitter, velocity variation, groove patterns, beat clock
- **Mirror octaves** for multi-octave harmony spread
- **Native desktop app** (single binary) and **WASM browser version**
- **Server/client mode** for remote MIDI processing over the network
- **Persistent MIDI device selection** across sessions

## Installation

### From Source

```
git clone https://github.com/waveywaves/contrapunk.git
cd contrapunk
cargo build --release
./target/release/contrapunk
```

### WASM (Browser)

```
cargo install trunk
trunk serve
```

Then open http://localhost:8080

## Usage

1. Select a MIDI input device and one or more output devices
2. Choose a musical key and scale mode
3. Pick a harmony mode (Diatonic 3rds, Contrary Motion, etc.)
4. Play notes on your MIDI controller to hear generated harmonies
5. Adjust humanization, presets, and other settings via the tabbed GUI

## Development

```
cargo build          # Debug build
cargo test           # Run tests
cargo clippy         # Lint
trunk build          # WASM build
trunk serve          # WASM dev server
```

## Architecture

Key modules in `src/`:

- **harmony/** -- Harmony engine, scale definitions, modal interchange, chord detection
- **humanizer/** -- Timing jitter, velocity variation, groove patterns, beat clock
- **server/** -- TCP server/client for remote MIDI processing
- **gui/** -- egui/eframe interface with steampunk theme, piano keyboard, preset system
- **midi/** -- MIDI I/O via midir (native) and Web MIDI API (WASM)

## Deployment

Deployed to Fly.io at [contrapunk.fly.dev](https://contrapunk.fly.dev/). CI/CD via GitHub Actions (`.github/workflows/ci.yml`).
