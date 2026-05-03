# Performance View — Design Notes

A simplified, switchable UI view alongside the existing detailed UI. Targets musicians (especially non-experts) who currently find Contrapunk's ~40-control surface unusable. The existing UI stays as the "Advanced" view; users toggle between them. The previous view-mode infrastructure was deleted in favor of per-panel pips, so the switcher needs to be built fresh.

## Design principle

Every knob in this view passes the test: *"Is this something the user touches WHILE playing?"* If not, it's a pad / setting / advanced-only.

## The 8 knobs

| # | Knob | Type | Maps to |
|---|---|---|---|
| 1 | **Mode** | 5 detents (ordered spectrum) | `HarmonyMode` enum |
| 2 | **Voices** | 1-8 detents | `voice_count` |
| 3 | **Tightness** | continuous 0..1 | composite: `voice_leading_*` + `counterpoint_*` |
| 4 | **Adventurous** | continuous 0..1 | composite: `interchange_enabled` + `borrowing_range` |
| 5 | **Key** + Auto toggle | 12 detents + on/off | `key`, `auto_key` |
| 6 | **Scale** | 5-7 category detents | `scale_mode` (with default sub-mode per category) |
| 7 | **You play** | 1-N detents (N = voice count) | `voice_position` |
| 8 | **Spread** | continuous 0..1 | quantized `OctaveMode` (v1) → continuous (v2) |

**Detune dropped from Performance view** — pure texture with no live-performance value. Moves to Setup pane / synth stage.

## Mode spectrum

```
Off  →  Parallel 3rds  →  Parallel 4ths  →  Contrary  →  Functional
 0          1                  2                3              4
```

5 detents, ordered by harmonic sophistication / voice independence. User can sweep through this live for dynamic effect.

Engine mapping:

| Detent | `HarmonyMode` |
|---|---|
| 0 | `PassThrough` |
| 1 | `DiatonicThirds` |
| 2 | `DiatonicFourths` |
| 3 | `ContraryMotion` |
| 4 | `FunctionalHarmony` |

### Dropped from this view

- **`RandomBelow`** — nondeterministic, untrustworthy live. Variation moves to Adventurous knob (interchange-based, deterministic).
- **`RandomBelowNoSeconds`** — same reason.
- **`BarryHarris`** — scale-specific (only sensible in BH 6th-diminished scales). Stays in advanced view; could re-enable in Performance view conditionally when the active scale family is `BarryHarris`.
- **`BachChorale`** — promoted to an emergent unlock (see below).
- **`StrictCounterpoint`** (Species 1-4) — folded into Tightness=high. Stays in advanced view with full species/strictness controls.

## Composite-knob mappings

### Tightness

| Range | Underlying behavior |
|---|---|
| 0.0–0.3 | `voice_leading_enabled = false` |
| 0.3–0.7 | `voice_leading_enabled = true`, `voice_leading_style = Free` |
| 0.7–0.9 | `voice_leading_style = Smooth` (or Jazz / BachChorale-flavored, depending on Mode) |
| 0.9–1.0 | When Mode = Functional → BachChorale voicing rules. When Mode = Contrary → StrictCounterpoint, Species 1, Strictness=Strict. Otherwise → strictest applicable voice-leading style. |

Exact ranges/curves TBD by sound testing.

### Adventurous

| Range | Underlying behavior |
|---|---|
| 0.0 | `interchange_enabled = false` |
| 0.0–1.0 | `interchange_enabled = true`, `borrowing_range = clamp(round(value × 7), 1, 7)` |

### Spread (continuous OctaveMode)

**v1** (cheap path) — quantize knob into 4 zones mapping to existing `OctaveMode` enum:

| Range | OctaveMode |
|---|---|
| 0.0–0.25 | `None` |
| 0.25–0.5 | `Spread` |
| 0.5–0.75 | `BassTrebleSplit` |
| 0.75–1.0 | `Mirror` |

**v2** (proper) — true continuous spread coefficient applied per-voice in the engine. Larger refactor; not required for v1.

### Scale (categorical, 8 detents)

The engine's `scale_mode` enum has ~30 sub-modes. Performance view collapses these to 8 category detents; sub-mode within a category is set by per-category default (or last-used), with full picker available in Advanced view.

**Ordering principle: simplest / hardest-to-misuse first.** Detent 1 should be the safest landing spot for a non-musician — the scale where "no notes are wrong." That's Pentatonic. Each subsequent detent introduces more harmonic complexity that the user needs to know how to handle.

| Detent | Category | Default sub-mode | Tonal? |
|---|---|---|---|
| 1 | **Pentatonic** | Major Pentatonic | partial — no 4 or 7, can't really hit a wrong note |
| 2 | **Major** | Ionian | yes |
| 3 | **Natural Minor** | Aeolian | yes |
| 4 | **Harmonic Minor** | Harmonic Minor | yes (strong dom-V via raised 7) |
| 5 | **Melodic Minor** | Melodic Minor | yes |
| 6 | **Modal** | Dorian (sub-cycle to Phrygian / Lydian / Mixolydian / Locrian in Advanced) | yes |
| 7 | **World** | Pelog | no |
| 8 | **Symmetric** | Whole Tone | no |

**Default on first launch:** Pentatonic. Cold-start user plays anything and the harmony sounds good — that's the value proposition the brutal-critic UI review said was hidden behind 3 setting changes today.

Harmonic Minor and Melodic Minor are split out from "Minor" because they sound fundamentally different — the raised 7 in Harmonic Minor produces strong V-i resolutions and the iconic augmented 2nd from b6 to 7; Melodic Minor (raised 6 and 7 ascending) is the basis of much modern jazz harmony. Lumping them under one category loses too much.

The "Tonal?" column gates which Mode + Tightness + Adventurous unlocks can fire (see updated tree below). Non-tonal scales (World, Symmetric) work fine with Parallel/Contrary modes; Functional/Chorale unlocks suppress because there's no functional harmony to chord-tone against.

Sub-mode access in Advanced view exposes each category's full mode list:
- Major → Ionian (only)
- Natural Minor → Aeolian (only)
- Harmonic Minor → Harmonic Minor + its 7 modes (Phrygian Dominant, Lydian #2, etc.)
- Melodic Minor → Melodic Minor + its 7 modes (Lydian Dominant, Altered, Locrian #2, etc.)
- Modal → Dorian / Phrygian / Lydian / Mixolydian / Locrian
- Pentatonic → Major / Minor / Hirajoshi / InSen / Iwato / Yo / Kumoi
- World → Pelog / Slendro / Hijaz / Hungarian / Persian / etc.
- Symmetric → Whole Tone / Diminished / Chromatic

### Key (with Auto-key toggle)

The Key knob has an attached Auto/Manual toggle pip:

- **Auto = on**: Key knob is read-only and *displays* the engine's currently-detected key with a "live" indicator. Engine drives `key` via `auto_key` detection from input.
- **Auto = off**: User sets key manually via the knob. `auto_key = false`, `key = user_choice`.
- **Grabbing the Key knob while Auto is on**: explicitly disables Auto with a visible state change (toggle pip flips, brief toast: *"Manual key set — auto-key disabled"*). Fixes the silent-toggle bug in `ControlPanel.svelte:97-104` flagged by the brutal-critic UI review.

## Continuity philosophy — knobs should morph, not step

The point of a knob is to allow smooth musical evolution while playing. If turning a knob produces audible "step" jumps between named zones, it's a multi-position selector pretending to be a knob. The Performance view targets two layers of continuity:

### Layer A — continuous Mode (v1, achievable)

Mode is a 0.0–4.0 continuous knob with snap-detents at integer positions. At fractional values the engine interpolates between the two adjacent modes:

| Transition | Engine interpolation |
|---|---|
| 0 ↔ 1  (Off → Parallel 3rds) | crossfade harmony output volume in/out |
| 1 ↔ 2  (Parallel 3rds → 4ths) | morph the diatonic interval: 3rd → tritone-ish blend → 4th |
| 2 ↔ 3  (Parallel 4ths → Contrary) | per-voice probabilistic motion: P(contrary) = fractional part |
| 3 ↔ 4  (Contrary → Functional) | chord-tone-awareness weight blend; partially functional, partially contrary |

Implementation: at fractional Mode value M, run `harmonize` for `floor(M)` and `ceil(M)`, then mix output per voice with weight `M - floor(M)`. Live sweeping produces audible morphs, not zone-jumps.

### Layer B — continuous composite knobs (v2, requires engine work)

Today, Tightness / Adventurous / Spread internally jump between zones because the underlying engine takes enums (`voice_leading_style`, `OctaveMode`) or stepped integers (`borrowing_range`). True continuity needs an engine refactor:

| Knob | Today (stepped) | v2 (continuous) |
|---|---|---|
| **Tightness** | enum: Free / Smooth / Palestrina / BachChorale | float coefficient applied to voice-leading rule weights (parallel-5th penalty × T, voice-distance penalty × T, etc.) |
| **Adventurous** | bool `interchange_enabled` + int `borrowing_range` (1-7) | float coefficient: `P(borrow_per_note) = A`, with borrowing depth determined per-note by sampling from a continuous distribution |
| **Spread** | enum `OctaveMode`: None / Spread / Split / Mirror | float spread coefficient applied per-voice in engine octave-distribution math; the four enum values become preset positions on the same continuum (≈ 0.0, 0.33, 0.66, 1.0) |

v2 work is bigger but eliminates the "stepped feel" entirely and makes every composite knob a true performance dial.

### Why this matters

- Live sweeping a knob produces a continuous timbral morph, not a series of discrete state changes — better for performance gestures (build-ups, drops, transitions).
- Eliminates the "wall of named zones" problem in composite knobs (the user doesn't think "Free vs Smooth vs Palestrina"; they think "looser vs tighter").
- Future-proofs the joystick / expression-pedal mapping: continuous external controllers want continuous parameters.

## Emergent unlocks

Recognizable musical idioms surfaced as small badges when the user lands on a particular knob configuration. Not separate modes — emergent labels that teach the user which knob combinations produce which named styles.

| Unlock | Trigger combination |
|---|---|
| **Chorale** | Mode=Functional + Tightness>0.9 + Voices=4 + Scale=tonal (Major/Minor/Modal) |
| **Strict Counterpoint** | Mode=Contrary + Tightness>0.9 + Scale=tonal |
| **Pop Choir** | Mode=Parallel 3rds + Voices∈{3,4} + Tightness∈[0.4, 0.7] |
| **Jazz Block** | Mode=Functional + Adventurous>0.5 + Voices=4 + Scale=tonal |
| **Drone** | Mode=Parallel 4ths + Voices=2 + Tightness<0.3 |
| **Bagpipe** | Mode=Parallel 4ths + Voices=2 + Spread<0.25 |
| **Modal Borrowing** | any + Adventurous=1.0 + Scale ∈ Major/Minor/Modal |

UI behavior:
- While the current configuration matches an unlock, display the badge label in the chord readout area.
- The unlock label functions as a *de facto* preset name — the user learns "Pop Choir = these three knob positions" by experiencing the badge, then can return to that combination deliberately. This replaces explicit save/recall.
- Gated by a Settings toggle "Show Advanced Idioms" (default off for v1, on for power users).

## Pads (MPK Mini hardware-mapping reference)

Not all need software equivalents in the Performance view, but the hardware mapping target is:

1. Panic / All-notes-off
2. Hold harmony (sustain past input)
3. A/B compare (snapshot vs. live)
4. Voice-position cycle (SATB next)
5. Octave-mode cycle (stepped through Spread quantization)
6. Mode jump → Parallel 3rds (quick reset to a safe default)
7. Mode jump → Functional (quick jump to chord-tone harmony)
8. Mode jump → Off (instant harmony bypass)

(No scene save/recall in v1 — see "Save/recall — explicitly out of scope" section. Pads 6/7/8 may become user-assignable in a later iteration.)

## Hardware-knob mapping — MIDI Learn

The 8 software knobs are bound to hardware MIDI CC numbers via a per-knob MIDI Learn flow (no preset library beyond the built-in MPK Mini default). The mapping is `Record<KnobIndex, number | null>` where each of the 8 knob indices maps to an optional CC number; persisted in localStorage under `contrapunk-knob-cc-map`.

**Cold-start default** — MPK Mini MK3 baseline preset, seeded on first run if no localStorage entry exists:

| Knob # | Knob | CC |
|---|---|---|
| 0 | Mode | 70 |
| 1 | Voices | 71 |
| 2 | Tightness | 72 |
| 3 | Adventurous | 73 |
| 4 | Key | 74 |
| 5 | Scale | 75 |
| 6 | You play | 76 |
| 7 | Spread | 77 |

**Learn flow:**
- Each knob has a small "Learn" pip displaying its current CC binding (e.g. `CC70`) or `—` if unbound.
- Clicking the pip arms that knob; the pip pulses amber and shows `LEARN…`.
- The next CC message received via the active MIDI input binds to that knob.
- Clicking the same pip again, or pressing Escape, cancels the learn.
- Bindings are one-to-one: if a CC is already bound to another knob, that other knob is unbound to keep the table unique.

**Reset to MPK Mini defaults** — a small button in the Performance view footer restores the seed mapping. No multi-preset system; this single preset plus user-configurable Learn covers the v1 surface.

**Backend contract** — the Tauri router thread emits a `knob-cc-raw` event with `{ cc: u8, value: f32 }` for every Control Change message it receives on the active input. CC → knob-index resolution lives entirely in the frontend; the Rust side has no hardcoded controller mapping. Browser/wasm/plugin transports do not currently forward raw CCs (no-op `onKnobCcRaw`); MIDI Learn in those transports is a future iteration.

**Out of scope for v1:**
- A multi-controller preset library (only the MPK Mini baseline ships).
- Pitch bend / aftertouch / NRPN learn — only standard 7-bit CC.
- Multi-channel filtering — every CC on every channel is observed; first-byte mask `0xB0` matches all channels.

## Save/recall — explicitly out of scope for v1

Decision: no preset manager, no named scenes, no save/recall UI. PresetManager (#65) stays unmounted. The brutal-critic UI review flagged this as the single biggest live-perf failure; we are accepting that trade-off because:

- Emergent unlock badges *are* the named-preset surface — they teach the user which knob combinations produce which idioms. "Pop Choir" is the preset name; the knob positions are the recipe.
- Last-knob-state is already persisted across sessions via localStorage. Users get back to their last-used config on relaunch.
- A formal preset library adds a save / browse / rename / delete UI surface that contradicts the "8 knobs, no menus" frame of the Performance view.

If save/recall comes back, it does so in a later milestone with a redesigned approach — not by re-mounting the existing PresetManager.

## Knob-value tree

How knob values combine, what each combination produces, and where unlocks emerge. Mode is the root because it's the most categorical knob; everything else modulates the chosen mode.

Legend: 🔓 = emergent unlock badge.

```
Mode = Off (PassThrough)
└── No harmony. Voices/Tightness/Adventurous/Spread/Detune all inert.
    Only useful for A/B-comparing dry signal vs. harmony-on.


Mode = Parallel 3rds                      (engine: DiatonicThirds)
│
├── Voices = 1
│   └── Effectively PassThrough (no harmony note generated).
│
├── Voices = 2 .. 8
│   │
│   ├── Tightness = 0.0 .. 0.3            voice-leading off
│   │   └── Raw stacked 3rds, parallel motion. Pop/folk default.
│   │
│   ├── Tightness = 0.3 .. 0.7            voice-leading on, Free style
│   │   └── Smooth parallel 3rds, common-tone retention.
│   │       └── Voices ∈ {3,4} → 🔓 Pop Choir
│   │
│   └── Tightness = 0.7 .. 1.0            stricter voice-leading
│       └── Tightly led parallel 3rds; close to 4-part chorale texture
│           when Voices = 4 (but not Bach — that needs Functional).
│
├── Adventurous = 0.0 .. 0.5              diatonic only
│   └── Pure in-key 3rds.
│
├── Adventurous = 0.5 .. 1.0              modal interchange enabled
│   └── Out-of-key inputs borrow from parallel modes.
│       └── Adventurous = 1.0 → 🔓 Modal Borrowing
│
├── Spread = 0.0 .. 0.25 (None)
│   └── Voices clustered around input.
├── Spread = 0.25 .. 0.5 (Spread)
│   └── Voices fanned out by octave.
├── Spread = 0.5 .. 0.75 (BassTrebleSplit)
│   └── Lower voices pushed down, upper voices pushed up.
└── Spread = 0.75 .. 1.0 (Mirror)
    └── Each harmony doubled at ±octave (voice count effectively 2×).


Mode = Parallel 4ths                      (engine: DiatonicFourths)
│
├── Voices = 2
│   │
│   ├── Tightness < 0.3 + held bass note  →  🔓 Drone
│   │   └── Sparse open-4ths drone texture.
│   │
│   ├── Spread < 0.25 (close voicing)     →  🔓 Bagpipe
│   │   └── Tight quartal voicing, drone-like.
│   │
│   └── (otherwise: stacked open 4ths, quartal jazz/modal sound)
│
├── Voices = 3 .. 4
│   └── Stacked 4ths (McCoy Tyner / quartal jazz texture).
│       Tightness behaves as Mode 1.
│
├── Voices = 5 .. 8
│   └── Wide quartal stack. Tends to blur tonality.
│
└── Adventurous = 1.0                     →  🔓 Modal Borrowing


Mode = Contrary                            (engine: ContraryMotion)
│
├── Voices = 1
│   └── Effectively PassThrough.
│
├── Voices = 2 .. 8
│   │
│   ├── Tightness = 0.0 .. 0.3
│   │   └── Loose contrary motion (harmony goes up when input goes down).
│   │
│   ├── Tightness = 0.3 .. 0.7
│   │   └── Smoothed contrary motion with voice-leading rules.
│   │
│   └── Tightness > 0.9                    →  🔓 Strict Counterpoint
│       └── Engine internally promotes to StrictCounterpoint mode,
│           Species 1, Strictness=Strict. Independent voices, Fux-style
│           rules applied.
│
├── Note: Contrary is stateful — it tracks input history. Repeated
│         identical input notes produce different harmonies on purpose.
│
└── Adventurous = 1.0                     →  🔓 Modal Borrowing


Mode = Functional                          (engine: FunctionalHarmony)
│
├── Voices = 1
│   └── Returns just the input (functional needs ≥ 2 voices to chord).
│
├── Voices = 2 .. 3
│   └── Reduced functional harmony — root + 3rd, or root + 3rd + 5th.
│
├── Voices = 4
│   │
│   ├── Tightness = 0.0 .. 0.5            loose chord-tone harmony
│   │   └── Standard chord-tone SATB; voice-leading on but permissive.
│   │
│   ├── Tightness = 0.5 .. 0.9            smooth voice-led SATB
│   │   └── Tighter resolution rules; sounds like good pop arranging.
│   │
│   ├── Tightness > 0.9 + Scale=tonal      →  🔓 Chorale
│   │   └── Engine internally uses BachChorale voicing rules.
│   │       No parallel 5ths/8ves, stepwise inner voices, classic hymn
│   │       texture. Suppressed on Pentatonic/World/Symmetric scales.
│   │
│   ├── Adventurous = 0.0 .. 0.5
│   │   └── Diatonic chord choices only.
│   │
│   └── Adventurous > 0.5 + Scale=tonal    →  🔓 Jazz Block
│       └── Modal interchange + extensions; produces 7th/9th/13th
│           voicings and parallel-minor borrows. Needs tonal scale.
│
├── Voices = 5 .. 8
│   └── Extended functional voicing (works but unconventional).
│
├── Scale = Pentatonic
│   └── Reduced functional harmony — fewer chord tones to land on.
│       Chorale / Jazz Block unlocks suppressed.
│
├── Scale = World / Symmetric
│   └── Functional harmony degrades — no I-IV-V relationships exist.
│       Engine falls back to chord-tone-stack of nearest in-scale notes.
│       All Functional unlocks suppressed; consider switching to
│       Parallel or Contrary modes for these scales.
│
└── Spread mostly cosmetic at Voices = 4 — chorale texture is dense by
    design. Spread > 0.75 (Mirror) produces choir-with-octave-doubling.
```

### Reverse index — unlock requirements

| 🔓 Unlock | Mode | Voices | Tightness | Adventurous | Spread | Scale |
|---|---|---|---|---|---|---|
| Pop Choir | Parallel 3rds | 3-4 | 0.3–0.7 | any | any | any |
| Drone | Parallel 4ths | 2 | < 0.3 | any | any | any |
| Bagpipe | Parallel 4ths | 2 | any | any | < 0.25 | any |
| Strict Counterpoint | Contrary | 2+ | > 0.9 | any | any | tonal |
| Chorale | Functional | 4 | > 0.9 | any | any | tonal |
| Jazz Block | Functional | 4 | any | > 0.5 | any | tonal |
| Modal Borrowing | any (except Off) | any | any | = 1.0 | any | Major / Minor / Modal |

"tonal" = Major / Minor / Modal categories. Pentatonic / World / Symmetric suppress these unlocks.

### Knobs that don't gate unlocks

- **Key** — transposes everything; doesn't change behavior or trigger unlocks. (Auto-key toggle also doesn't affect unlocks; it just changes how `key` is set.)
- **You play** — re-slots the user's note in the SATB arrangement; affects placement but not which unlock fires.

### Notes on edge interactions

- **Voices = 1** collapses Parallel/Contrary/Functional to PassThrough behavior in practice. Disable Tightness/Adventurous knobs in the UI when Voices=1 (or grey them out) to prevent confusion.
- **Voices ≥ 5** outside Functional mode produces walls of stacked intervals. Acceptable but verges on noise; consider a "dense" warning hint.
- **Adventurous = 1.0 in any non-Off mode** triggers Modal Borrowing — this is the only unlock that's mode-agnostic.
- Multiple unlocks can match simultaneously (e.g. Chorale + Modal Borrowing at Functional + V=4 + T>0.9 + A=1.0). UI should show the most-specific badge first; Modal Borrowing is the catch-all.

## First-time user experience (FTUX)

Most of the brutal-critic UI review's findings are FTUX failures: silent default mode, no save/recall, invisible internal synth, jargon-only labels, no demo path. This section lays out the FTUX target.

### Principles

1. **Immediate value, no setup ritual.** App opens, user plays, harmony comes out. No mandatory wizard, no "configure your inputs" gate.
2. **Progressive disclosure via interaction, not via hidden controls.** All 8 knobs visible from the start; tooltips and hints appear as the user touches each knob for the first time. Hidden-by-default controls always backfire.
3. **Pre-routed audio.** Internal Synth is the default output and is wired to play harmonies immediately on first note. External MIDI is opt-in.
4. **Education without lecture.** No modal walkthroughs, no multi-step "Next" flows. Tips appear contextually as the user explores.

### Cold-start defaults

The target: cold-start user presses any laptop key and hears consonant harmony in under 2 seconds.

| Setting | Default | Reason |
|---|---|---|
| View mode | Performance (8 knobs) | Avoid confronting the user with the Advanced wall on first launch |
| Mode | Parallel 3rds | Simplest audible harmony — not PassThrough (which is silent) |
| Voices | 2 | One harmony voice + user's input — minimum to demonstrate the product |
| Tightness | 0.5 (mid) | Smooth voice-leading, not strict |
| Adventurous | 0.0 | Diatonic only — no out-of-key surprises for first-time users |
| Key | C, Auto-key = ON | Auto-detect from input; falls back to C if no input yet |
| Scale | **Pentatonic** | "No wrong notes" — safest first-touch scale (see Scale section) |
| You play | Soprano (slot 0) | Default for most users |
| Spread | 0.0 (None) | Voices clustered; tightest, simplest texture |
| Input | Computer Keyboard | Universally available; no MIDI hardware required |
| Output | Internal Synth | Pre-routed; user doesn't need external MIDI to hear anything |

### In-context guidance (action-triggered, not timed)

Time-based tooltip parades annoy users. Tie hints to user actions instead:

| Trigger | Hint |
|---|---|
| First Note-On after launch | Toast: *"That's a Pentatonic harmony in 3rds. Try the knobs."* |
| First hover on Mode | *"Sweep this to morph between harmony styles: thirds → fourths → contrary motion → block chords."* |
| First hover on Voices | *"More voices = thicker harmony. Try 4 for SATB choir."* |
| First hover on Tightness | *"How smoothly the voices move. Higher = more refined voice-leading."* |
| First hover on Adventurous | *"How often the harmony borrows out-of-key chords. Higher = more colorful, less predictable."* |
| First hover on Spread | *"How widely the voices spread around your note."* |
| First emergent unlock fires | Badge appears with one-line description (e.g. *"Pop Choir — Parallel 3rds, 3-4 voices, smooth voice-leading"*). Doubles as the de facto preset name; user learns the recipe. |
| After 5+ knobs explored | Toast: *"Curious about deeper controls? Try Advanced view."* |

Each "first hover" hint is shown once per knob per device, persisted via localStorage flag.

### Empty state (no input yet)

If 10 seconds elapse after launch with no Note-On:

- Display copy: *"Press any key on your computer, plug in a MIDI device, or click ▶ to hear a demo."*
- ▶ button triggers the Demo (below).

### Demo mode

Always accessible from empty state and from a small "Demo" button in the StatusBar:

- Plays a 10-second held C-Pentatonic chord through the engine.
- Knobs visibly animate during playback: Mode sweeps left-to-right, Voices builds 1→2→3→4, Tightness rises, Adventurous nudges up partway through.
- User sees the parameter → sound mapping without having to play.
- "Skip" / "Stop" interrupts. "Apply this config" copies the demo's final knob state to the user's session.
- Useful for non-musicians who don't yet know what any of the controls *do*.

### What FTUX is not

- A modal wizard the user must dismiss before playing.
- A multi-step "configure your audio" gate.
- A click-Next tutorial.
- A skill-level questionnaire.
- Hidden controls revealed by milestones / achievements.

### Connection to other systems

- Cold-start defaults assume the **Quick wins** patches have shipped: default mode is no longer PassThrough, internal synth is auto-routed, Computer Keyboard input is the default selection.
- In-context hints persist their "shown once" flags in the same localStorage that holds settings.
- **Emergent unlocks are the long-term FTUX teacher** — each unlock that fires teaches the user a parameter-combo → named-idiom mapping. The badge system carries the educational load after the cold-start hints are exhausted.

## Quick wins (independent of Performance view)

These should ship before or alongside the redesign — they fix bugs the brutal-critic UI review flagged that aren't tied to the new view:

- Change default `mode: 'PassThrough'` → `'DiatonicThirds'` so first launch produces audible harmony
- Fix Auto-key + Key-dropdown silent toggle interaction
- Expand voiceCount picker from 1-4 to 1-8 (engine already supports it)
- Rename `Rng` → "Borrow depth" in the Advanced view
- Tempo-density guard for Counterpoint Species 3/4 (or demote them out of the top-level mode picker)

(PresetManager re-mount removed — preset/scene management is explicitly out of scope; see the "Save/recall" decision above.)

## Open questions

- Should the unlock badge persist while in zone, or flash once on entry?
- True continuous Spread vs. quantized-to-enum: v1 design call
- Touch target sizing — current UI uses sub-12px labels and 40px buttons; Performance view needs much larger hit targets for one-handed live use
- View-mode switcher placement — StatusBar pill / Settings toggle / dedicated chrome
- Scale category default sub-modes — should "last-used" override the per-category default once user has explored, or always reset to default on category change?
- Detune was dropped from the 8 knobs — does it deserve a slot in the Setup pane / Settings, or is it OK to retire entirely from the Performance-view audience?

## Out of scope for v1

- Reactive / auto-evolving harmony (fast playing → simpler, slow → richer)
- Per-scene tempo / beat-phase
- Programmable mode trajectories (verse=parallel, chorus=functional auto-transitions)
- Audio preview on Mode detent change
- Joystick-based morphing of Adventurous

## Implementation notes

- The existing `engine.svelte.ts` setters are the dispatch surface. Composite knobs (Tightness, Adventurous, Spread) read 0..1 and fire 1-3 `adapter.set*` calls per change.
- View-switching state lives in the same `ui.svelte.ts` store that owns the panel-pip toggles. The two systems are orthogonal: panel pips control which existing-UI panels are visible inside Advanced view; the Performance view is a separate top-level layout.
- When the user switches Advanced → Performance, composite knobs display their last-known position. No attempt to invert-map advanced state to composite-knob positions.
