# Contrapunk Diagram Snippets

> Compact Mermaid snippets synthesized from `README.md`, `recon/github-issues.md`, `recon/planning-roadmap.md`, `Cargo.toml`, and `graphify-out/GRAPH_REPORT.md`. Roadmap order is inferred where the sources only expose backlog signals.

## Architecture

```mermaid
flowchart LR
    subgraph Inputs["Inputs"]
        Guitar["Guitar or mic audio"]
        MidiIn["MIDI controller"]
        WebKeys["Browser piano or demo"]
        DawClock["DAW clock, IAC, Link"]
    end

    subgraph Surfaces["Distribution surfaces"]
        Desktop["Tauri desktop"]
        Browser["WASM browser"]
        Cli["Native CLI"]
        Plugin["VST3 or CLAP plugin"]
        Adapter["Surface glue and adapters<br/>TauriAdapter, WasmAdapter, PluginAdapter"]
    end

    subgraph Core["Shared Rust core"]
        Pitch["Pitch detection<br/>McLeod, onset, calibration"]
        Transport["Transport<br/>beat phase and tempo"]
        Midi["MIDI and routing"]
        Harmony["HarmonyEngine<br/>scales, modes, voice leading"]
        Companion["Companion lanes<br/>drum, drone, looper, arp"]
        Presets["Preset and state model"]
        Analysis["Analysis overlays<br/>chords, intervals, function"]
    end

    subgraph Audio["Audio and output chain"]
        Synth["Built-in synth<br/>legacy or Elixir"]
        Fx["Delay, reverb, DSP"]
        Clap["Runtime CLAP hosting"]
        Output["Speakers, MIDI out, DAW"]
    end

    Desktop --> Adapter
    Browser --> Adapter
    Cli --> Adapter
    Plugin --> Adapter

    Guitar --> Pitch --> Adapter
    MidiIn --> Adapter
    WebKeys --> Adapter
    DawClock --> Transport

    Adapter --> Midi
    Adapter --> Presets
    Adapter --> Transport
    Presets --> Harmony
    Midi --> Harmony
    Transport --> Harmony
    Harmony --> Analysis --> Adapter
    Harmony --> Companion --> Synth
    Harmony --> Synth
    Synth --> Fx --> Clap --> Output
    Adapter --> Output
```

## Event Flow

```mermaid
sequenceDiagram
    autonumber
    participant Player as Player
    participant Surface as Surface adapter
    participant GuitarDsp as Guitar DSP
    participant Transport as Transport
    participant Engine as HarmonyEngine
    participant Chain as Audio and MIDI chain
    participant Out as Speakers or DAW

    Player->>Surface: Play MIDI, guitar audio, or browser piano
    alt audio input
        Surface->>GuitarDsp: detect onset, pitch, string or fret
        GuitarDsp-->>Surface: note event plus confidence
    else MIDI or UI input
        Surface-->>Surface: normalize note on or off
    end

    Surface->>Transport: read beat phase and tempo when available
    Transport-->>Surface: timing context
    Surface->>Engine: note event plus key, mode, style, voice config
    Engine-->>Surface: harmony notes plus analysis
    Surface->>Chain: route internal synth, Elixir, FX, CLAP, or MIDI out
    Chain-->>Out: sound and note events

    opt settings change while notes ring
        Player->>Surface: change key, style, routing, preset
        Surface->>Engine: update state
        Engine-->>Surface: all notes off or reharmonize diff
        Surface->>Chain: drain stale notes before new harmony
    end
```

## Crate and Surface Map

```mermaid
flowchart TB
    subgraph Ship["Shipped surfaces"]
        CliSurface["contrapunk CLI<br/>src/main.rs"]
        TauriSurface["Tauri desktop<br/>src-tauri"]
        WasmSurface["Browser WASM<br/>wasm"]
        UiSurface["Svelte UI<br/>ui"]
        PluginSurface["nih-plug plugin<br/>plugin"]
    end

    subgraph Contrapunk["Contrapunk workspace crates"]
        Root["contrapunk root crate"]
        TransportCrate["contrapunk-transport"]
        MidiCrate["contrapunk-midi"]
        ChordCrate["contrapunk-chord"]
        HarmonyCrate["contrapunk-harmony"]
        AudioCrate["contrapunk-audio"]
        DspCrate["contrapunk-dsp"]
        PresetCrate["contrapunk-preset"]
        CompanionCrate["contrapunk-companion"]
    end

    subgraph Elixir["Elixir synth line"]
        ElixirCore["elixir-core"]
        ElixirStandalone["elixir-standalone"]
        ElixirPlugin["elixir-plugin"]
        ElixirPreset["elixir-preset"]
    end

    subgraph Golem["Golem drummer line"]
        GolemApp["apps/golem/src-tauri"]
        GolemCore["golem-core"]
    end

    UiSurface --> TauriAdapter["TauriAdapter"]
    UiSurface --> WasmAdapter["WasmAdapter"]
    UiSurface --> PluginAdapter["PluginAdapter"]
    TauriAdapter --> TauriSurface
    WasmAdapter --> WasmSurface
    PluginAdapter --> PluginSurface

    CliSurface --> Root
    TauriSurface --> Root
    WasmSurface --> Root
    PluginSurface --> Root

    Root --> TransportCrate
    Root --> MidiCrate
    Root --> ChordCrate
    Root --> HarmonyCrate
    Root --> AudioCrate
    Root --> DspCrate
    Root --> PresetCrate
    Root -.-> CompanionCrate
    Root -.-> ElixirCore

    ElixirStandalone --> ElixirCore
    ElixirPlugin --> ElixirCore
    ElixirPreset --> ElixirCore
    GolemApp --> GolemCore
```

## Roadmap

```mermaid
flowchart LR
    Done["Baseline shipped<br/>desktop, browser, plugin path<br/>v1.1 web demo readiness mostly closed"]
    Backlog["Open backlog<br/>41 issues<br/>companion, integrations, guitar, theory lead"]
    Triage["Trim stale work<br/>close or rescope superseded items"]
    Ftux["Activation fix<br/>default sound path and visible routing<br/>issue 116"]
    ElixirNow["Current focus<br/>Elixir v1.5 / elixir-v0.1.0"]
    B5["Next named slice<br/>plugin preset serialization and load"]
    B7["B7 wavetable parity"]
    Release["B8/B9 render and public Elixir release"]
    CompanionMvp["Companion MVP<br/>DrumLane, DroneLane, LooperLane, ArpLane"]
    GuitarRewrite["Guitar rewrite<br/>fixtures, A/B stages, native/WASM parity"]
    DawBase["DAW foundation<br/>sync, IAC, BlackHole, openDAW"]
    Intelligence["Sampler and audio intelligence<br/>ListenLane, sidechain, slicing"]
    Theory["Theory depth<br/>historical rules and live analysis"]
    Deferred["Known pending<br/>modal UX review, note generator WASM parity,<br/>logo icons, DMG polish"]

    Done --> Backlog --> Triage --> Ftux
    Ftux --> ElixirNow --> B5 --> B7 --> Release
    Ftux --> CompanionMvp --> GuitarRewrite --> DawBase --> Intelligence --> Theory
    Done -.-> Deferred
    Deferred -.-> Triage
```

## Marketing Funnel

```mermaid
flowchart TB
    Awareness["Awareness<br/>website, README, GitHub, docs"]
    Trial["Trial<br/>app.contrapunk.com browser demo"]
    Activation{"Activation<br/>hear harmony immediately?"}
    Fix["FTUX fix<br/>internal synth default and inline routing<br/>issue 116"]
    Aha["Aha moment<br/>one note becomes counterpoint"]
    Convert["Conversion<br/>Mac DMG, desktop app, plugin"]
    Retain["Retention<br/>presets, DAW sync, lanes, guitar input"]
    Advocate["Advocacy<br/>OSS issues, PRs, examples"]
    Proof["Proof gaps<br/>audio samples issue 5<br/>walkthrough issue 13"]

    Awareness --> Trial --> Activation
    Activation -->|yes| Aha
    Activation -->|risk| Fix --> Aha
    Aha --> Convert --> Retain --> Advocate
    Proof -.-> Awareness
    Proof -.-> Trial
```
