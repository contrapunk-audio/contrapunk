<p align="center">
  <img src="ui/static/logo.svg" alt="Contrapunk" width="200" />
</p>

<h1 align="center">Contrapunk</h1>

<p align="center">
  <a href="https://contrapunk.com">Website</a> &middot;
  <a href="https://app.contrapunk.com">Web App</a> &middot;
  <a href="https://contrapunk.com/docs/">User Guide</a> &middot;
  <a href="https://github.com/contrapunk-audio/contrapunk/releases">Downloads</a> &middot;
  <a href="CONTRIBUTING.md">Contributing</a>
</p>

Contrapunk is a real-time MIDI harmony and arrangement generator. Play a MIDI controller or monophonic guitar line, choose a harmonic approach, and route the generated voices to the built-in sound engine, hardware MIDI, or a DAW instrument.

The same Rust core powers the native app, browser app, and DAW plug-ins. Arrangement presets are technique-named, research-backed configurations; unfinished built-ins stay hidden from production menus.

## Install and use

- **Desktop:** download the latest published macOS or Windows installer from [GitHub Releases](https://github.com/contrapunk-audio/contrapunk/releases).
- **Browser:** open [app.contrapunk.com](https://app.contrapunk.com). Web MIDI requires a compatible browser and permission.
- **DAW plug-ins:** published releases may include VST3, CLAP, and macOS Audio Unit assets; check the release asset list.
- **Full setup guide:** [contrapunk.com/docs/start/install](https://contrapunk.com/docs/start/install/).

Desktop custom harmony-style presets persist as versioned `.cpk` files in the app-data directory. They do not yet include synth, plug-in, routing, Slide, or Companion state.

### Logic Pro

Contrapunk provides two Audio Units under **Contrapunk Audio**:

- **MIDI FX → Contrapunk:** harmonizes incoming track MIDI before the instrument.
- **Audio FX → Contrapunk Guitar:** accepts mono or stereo guitar audio, detects pitch, and publishes generated MIDI through the CoreMIDI source **Contrapunk Guitar MIDI Out**. Route that source to a separate software-instrument track.

See the [Logic routing guide](https://contrapunk.com/docs/surfaces/logic/). Logic does not send track audio into MIDI FX, so guitar conversion requires the Audio FX component.

## Signal flow

```text
MIDI controller ──────────────────────────────┐
                                              v
Monophonic guitar → pitch/onset detection → HarmonyEngine → Companion lanes
                                                        │
                                                        └→ generated MIDI
                                                            ├→ built-in sound/FX
                                                            ├→ hardware or virtual MIDI
                                                            └→ DAW instrument
```

The harmony core includes diatonic, contrary-motion, species-counterpoint, Barry Harris, functional-harmony, chorale, and explicit-interval strategies. Companion lanes add bounded canon, counterpoint, and pattern timing with exact NoteOn/NoteOff ownership.

The desktop app can host CLAP instruments and effects. Generic VST3 instrument hosting is not part of the current release; Contrapunk's own VST3/CLAP/AU builds are distribution formats, not embedded third-party hosts.

## Product surfaces

| Surface | Entry point | Notes |
|---|---|---|
| Native CLI | `src/main.rs` | Headless development and TCP client/server modes |
| Desktop | `src-tauri/` | Tauri app for macOS, Windows, and Linux builds |
| Browser | `wasm/` + `ui/` | WebAssembly core with Web MIDI/Web Audio adapters |
| VST3 / CLAP | `plugin/` | DAW MIDI/audio processing with the production webview UI |
| Audio Unit | `au-wrapper/` | Logic MIDI FX plus mono/stereo Guitar Audio FX |

## Build from source

Prerequisites: [Rust](https://rustup.rs/), Node.js 22, npm, and the [Tauri platform prerequisites](https://v2.tauri.app/start/prerequisites/).

```bash
git clone https://github.com/contrapunk-audio/contrapunk.git
cd contrapunk
npm --prefix ui ci

# Native CLI
cargo build --release

# Desktop development
(cd src-tauri && cargo tauri dev)

# Browser development
npm --prefix ui run build:wasm
npm --prefix ui run dev
```

Build the DAW plug-ins:

```bash
npm --prefix ui run build
CONTRAPUNK_PLUGIN_UI_DIR="$PWD/ui/build" \
  cargo xtask bundle contrapunk_plugin --release --features embed-ui

# macOS universal VST3/CLAP and Audio Units
CONTRAPUNK_PLUGIN_UI_DIR="$PWD/ui/build" \
  cargo xtask bundle-universal contrapunk_plugin --release --features embed-ui
./au-wrapper/build.sh
```

## Checks

```bash
cargo check --workspace --message-format=short
cargo test -p contrapunk-harmony --lib
cargo test -p contrapunk-audio --lib
npm --prefix ui run check
```

See [CONTRIBUTING.md](CONTRIBUTING.md) for architecture, focused test commands, and release-surface guidance.

## Documentation

- [User guide](https://contrapunk.com/docs/)
- [DAW routing](docs/DAW_SIDE_BY_SIDE.md)
- [Standalone instrument routing](docs/IAC_PLUGIN_SETUP.md)
- [Engine deep dive](docs/ENGINE_DEEP_DIVE.md)
- [Release secrets](docs/RELEASE_SECRETS.md)

## License

MIT. See [LICENSE](LICENSE).
