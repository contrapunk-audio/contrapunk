# Synthesis: Preset 48 — Crystal Chorale

**Decision:** `ready_for_implementation`

**Reference scope:** melody-first, limited-voice, thematically recurring arrangement practice in a bounded 1987–94 Nobuo Uematsu / *Final Fantasy* console corpus, with later PlayStation evidence used only to explain expansion, collaboration, and why choir/orchestration claims require separate attribution

**Operational claim:** each clean harmonic-minor source note receives a reactive four-part SATB-style chord and one complete octave echo two transport beats later; the player authors the melody, recurrence, cadence, dynamics, phrase form, development, and silence

## 1. Historical boundary

The strongest documented through-line is memorable melody surviving changing hardware and arrangement. Early work used severe voice/data limits; SNES production allowed sampled color and greater thematic/linear development; later PlayStation production introduced larger recorded and collaborative forces. Recurring material such as the series Prelude changes arrangement across projects rather than representing one fixed “crystal harmony.”

Neither harmonic minor nor octave call-and-response is established as Uematsu’s universal method. They are bounded product choices compatible with melody-first, limited-voice reinforcement. The preset must not claim an authentic Uematsu scale, a learned theme, a franchise Prelude mechanism, or reproduction of a named score.

Attribution remains distributed. Sound programmers, development teams, text writers, orchestrators such as Shiro Hamaguchi, and named performers contributed to cited productions. A MIDI preset cannot collapse those roles into “Uematsu orchestration.”

## 2. Report agreement and parent resolution

All reports agree that:

- the player must provide the memorable original melody;
- recognizable recurrence and variation are authored formal behaviors, not a local harmony rule;
- harmonic minor is defensible as a bounded fantasy/cadential palette but not an artist fingerprint;
- a current delayed octave can be implemented only as one echo per note, not phrase capture or a learned answer;
- choir, orchestra, crystal/Prelude identity, game state, narrative function, thematic families, modulation, and long-range form remain unsupported;
- public copy must use new abstract material and reject soundtrack/artist imitation.

The history report proposes sparse opening, selective reinforcement, and cadential widening. Current HarmonyEngine and Canon cannot detect those phrase states. The parent therefore removes automatic density growth: four-part harmony is reactive on every source event, and the octave echo is fixed on every in-scale event. Sparse-to-full shape remains performer-authored through note density, register, duration, dynamics, and rests.

## 3. Exact current mapping

Use the shared HarmonyEngine plus one shared Free Imitation voice:

```json
{
  "harmony": {
    "scaleMode": "HarmonicMinor",
    "mode": "BachChorale",
    "voiceCount": 4,
    "voicePosition": 0,
    "voiceLeadingEnabled": false,
    "voiceLeadingStyle": "Free",
    "octaveMode": "None",
    "octaveIntensity": 1,
    "interchangeEnabled": false,
    "interchangeRange": 3,
    "counterpointSpecies": "Species1",
    "counterpointStrictness": "Strict"
  },
  "companion": {
    "enabled": true,
    "globalHoldMode": { "kind": "cancel" },
    "canon": {
      "enabled": true,
      "form": "free_imitation",
      "holdMode": { "kind": "forever" },
      "voices": [
        {
          "delayBeats": 2.0,
          "transposeDegrees": 7,
          "timeRatio": 1.0,
          "harmonyMode": "PassThrough",
          "referenceVoice": null,
          "voiceCount": 1,
          "voicePosition": 0,
          "voiceLeadingEnabled": false,
          "voiceLeadingStyle": "Free",
          "octaveMode": "None",
          "counterpointSpecies": "Species1",
          "counterpointStrictness": "Strict",
          "holdMode": null
        }
      ]
    },
    "counterpoint": {
      "enabled": false,
      "species": "Species1",
      "transposeDegrees": 2,
      "preferAbove": true,
      "holdMode": null
    }
  },
  "mix": { "input": 1, "harmony": 1, "canon": 1, "counterpoint": 1 }
}
```

**Requirements:** `['harmony', 'free_imitation']`. Transport is required for the delayed echo and otherwise preserved.

The main path emits exactly four immediate notes, including the exact source once as soprano. The Canon path emits exactly one delayed PassThrough subject. For an in-scale seven-note collection input with MIDI headroom, `transposeDegrees: 7` must equal source `+12` semitones. The honest peak contract is five logical notes per source event, not four total.

`Forever` means that releasing the source does not cancel its already-scheduled echo. It does not mean infinite sustain: the delayed NoteOff still follows the source duration at unity time.

## 4. Performer contract

- Use one clean harmonic-minor note at a time around MIDI 57–74.
- Play an original four-bar phrase at roughly 72–88 BPM, mostly quarters/eighths, three to six attacks per bar.
- Repeat the opening rhythm once, changing one or two middle pitches or the final destination.
- Hold the cadence for two to four beats, release cleanly, then leave at least one full bar for the delayed tail and silence.
- Keep velocities roughly 60–92 and reserve one modest peak.
- Guitar: clean DI, tuned, muted strings, discrete fretted notes, no bends, slides, harmonics, ringing strings, distortion, delay, or reverb before detection.
- Keyboard: one key at a time, sustain pedal up, no chord mode, arpeggiator, bend, or overlapping keys.
- Stay below MIDI 116 for the +12 echo; the recommended range provides ample headroom.

The player creates `A statement → cadence → breath → A′ variation → longer cadence → silence`. The engine neither recognizes nor remembers A/A′.

## 5. Honest public copy

Use copy equivalent to:

- **Name:** Crystal Chorale
- **Result:** Each clean harmonic-minor melody note receives a four-part SATB-style chord and one complete octave echo two beats later.
- **Play:** Perform an original four-bar single-note phrase, repeat its opening with one small change, hold the cadence for two to four beats, then release and leave a full bar for the echo tail.
- **Approximation:** A bounded melody-first fantasy-RPG arrangement study informed by selected 1987–94 console practice. Harmonic minor and the per-note octave echo are product choices, not Nobuo Uematsu signatures. The preset does not compose or recognize themes, replay phrases, reproduce the Prelude or any score, orchestrate a choir, follow game/narrative state, develop form, or simulate Uematsu.

Call the delayed layer a **per-note octave echo**, not a phrase answer, octave response, learned theme, Prelude, crystal theme, or independent orchestration.

## 6. Explicitly out of scope

The current preset does not provide:

- motif or phrase recognition, capture, replay, variation, thematic networking, or leitmotivic meaning;
- automatic four/eight-bar form, sparse-to-full growth, cadence detection, modulation, harmonic timeline, section return, or game-state adaptation;
- authored bass/counterline rhythm, instrument groups, crystal timbre, arpeggiated Prelude identity, choir, orchestra, sound programming, or performance;
- exact score harmony, melody, rhythm, samples, MIDI, or copyrighted franchise assets;
- artist identity, endorsement, or a whole-career style model.

Future upgrades must use reusable phrase, timeline, stable-group, sound-role, and adaptive-scene capabilities, never preset-specific branches.

## 7. Implementation acceptance checks

1. `HarmonicMinor` is accepted by the current BachChorale functional path and produces exactly four immediate notes with source once as soprano.
2. A focused main-engine fixture pins one safe in-scale pitch set and matching NoteOff ownership with empty active state.
3. The Canon voice is explicitly PassThrough, one voice, +7 scale degrees, unity time, two-beat delay, and emits exactly source +12 in the approved range.
4. One short source note released before beat 2 still produces exactly one delayed NoteOn and one delayed NoteOff; queues and held state end empty.
5. The combined documented peak is four immediate plus one delayed note per source event; no delayed chord is emitted.
6. Chords, overlapping input, chromatic acceptance, and near-range-overflow input are outside the approved performance corpus.
7. Stop, panic, preset replacement, and Canon disable leave zero active, held, or pending owners through existing shared lifecycle contracts.
8. Applying the preset preserves tonic, BPM, transport, devices, routing, sound, master, mute/solo, and plugins.
9. UI copy says per-note octave echo and contains no phrase-answer, learned-theme, Prelude, choir/orchestra, narrative/game-state, soundtrack, or artist-imitation claim.
10. Shared harmony, Companion, catalog/persistence, and UI checks pass with no new warnings.

## 8. Evidence trail

This synthesis depends on the three independent cited reports in this directory:

- `history.md` for hardware/project evolution, melody-first evidence, recurring arrangement/leitmotif context, collaborators, attribution, and copyright boundaries;
- `theory.md` for HarmonicMinor/BachChorale/Canon capability audit, the five-note contract, Hold semantics, ownership tests, and unsupported formal behaviors;
- `performance.md` for the original-phrase contract, register, tempo, articulation, dynamics, guitar/keyboard limits, failure gestures, lifecycle, and 36-second acceptance exercise.

Where research describes selective reinforcement or phrase-level answers, this synthesis adopts the smaller deterministic per-note behavior the present shared engine can prove and states the difference in public copy.
