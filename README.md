<p align="center">
  <img src="ui/static/logo.svg" alt="Contrapunk" width="200" />
</p>

<h1 align="center">Contrapunk</h1>

<p align="center">
  <a href="https://contrapunk.com">Website</a> &middot;
  <a href="https://app.contrapunk.com">Try in Browser</a> &middot;
  <a href="https://github.com/contrapunk-audio/contrapunk/releases">Download for Mac</a> &middot;
  <a href="CONTRIBUTING.md">Contributing</a>
</p>

Real-time counterpoint harmony generator + OSS plugin host. Plug in a guitar or MIDI controller, pick a key and voice leading style, and Contrapunk generates harmony voices that follow actual counterpoint rules -- Palestrina, Bach Chorale, Jazz, or Free. Route the output through built-in FX or any CLAP plugin you have installed. Built in Rust.

Counterpoint is the art of combining independent melodic lines. Contrapunk implements these rules so you play one note and hear harmonically correct accompaniment in real-time.

## Try It

**Browser** (no install): [app.contrapunk.com](https://app.contrapunk.com)

**macOS DMG**: [GitHub Releases](https://github.com/contrapunk-audio/contrapunk/releases)

## Build from Source

Prerequisites: [Rust](https://rustup.rs/), Node.js, [Tauri CLI](https://v2.tauri.app/start/prerequisites/)

```bash
git clone https://github.com/contrapunk-audio/contrapunk.git
cd contrapunk
cargo tauri dev
```

For WASM only:
```bash
cd ui && npm install && npm run build:wasm && npm run dev
```

## How It Works

```
Guitar/MIDI Input
       |
  Pitch Detection (McLeod + single-cycle, 128-sample buffer, ~2.7ms)
       |
  Harmony Engine (scales, modes, voice leading rules)
       |
  Audio Chain: Synth -> Delay -> Reverb -> CLAP plugins...
       |
  Speakers / MIDI Output
```

**Harmony**: 8 modes (diatonic thirds, contrary motion, strict counterpoint, Barry Harris, etc.), 28 scales, 4 voice leading styles. Voice position is configurable -- play as soprano, alto, tenor, or bass.

**Guitar Input**: audio-to-MIDI with onset detection, pitch voting, auto-calibration, and string/fret identification. Sub-10ms pluck-to-note-on on M-series Macs.

**Audio chain**: 8-voice built-in synth, Freeverb reverb, stereo delay, and third-party CLAP plugins (FabFilter, sforzando, u-he Diva, Surge XT, Vital, etc.) -- all loaded at runtime, GUIs embedded inside the app window or detached. Port-layout-aware so sidechain effects work correctly. VST3 / AU / AAX on the roadmap.

**Runs everywhere**: native desktop (Tauri v2), browser (WebAssembly), same Rust core.

## Examples

```bash
cargo run --release --example guitar_tuner      # Live tuner with auto-calibration
cargo run --release --example guitar_harmony    # Guitar audio -> harmony -> MIDI out
cargo run --release --example guitar_capture    # ML training data capture
```

## Tests

```bash
cargo test                                      # Full suite
cargo test --test audio_pipeline -- --nocapture # Audio pipeline integration
cargo test -p contrapunk guitar                 # Guitar module
```

Pre-commit hook (fmt, clippy, test, wasm check):
```bash
cp scripts/pre-commit .git/hooks/pre-commit && chmod +x .git/hooks/pre-commit
```

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for build setup, architecture overview, and guidelines.

Issues and PRs welcome. If you hit a bug or have a feature idea, [open an issue](https://github.com/contrapunk-audio/contrapunk/issues).

## Docs

- [Engine Deep Dive](docs/ENGINE_DEEP_DIVE.md) -- technical internals with diagrams
- [Project Journey](docs/PROJECT_JOURNEY.md) -- the story of building Contrapunk
- [ML Concepts](ml/CONCEPTS.md) -- educational guide to the ML/audio pipeline

## License

MIT. See [LICENSE](LICENSE).
