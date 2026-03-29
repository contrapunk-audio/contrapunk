# Contrapunk Market Analysis & Competitive Landscape

## The One-Slide Summary

**No existing product combines real-time counterpoint generation against live MIDI input with configurable historical voice-leading styles in a standalone usable instrument.**

The individual pieces exist scattered across academic prototypes, hardware pedals, and software plugins. Contrapunk is the first to assemble them into a single tool for live performers.

---

## The Market Today: Three Categories, One Gap

### Category 1: Hardware Pedals & Vocal Harmonizers
**What they do:** Transpose your note by a fixed interval within a chosen scale.
**What they DON'T do:** Generate independent melodic lines. The harmony moves in parallel with you — the opposite of counterpoint.

| Product | Price | Voices | Voice Leading | Independent Lines |
|---------|-------|--------|---------------|-------------------|
| Boss Harmonist HR-2 | ~$150 | 2 | None | No — parallel transposition |
| TC Helicon Harmony Singer 2 | ~$200 | 2 | None — follows guitar chords | No — parallel |
| Eventide H9 / PitchFactor | ~$500 | 2 | None | No — parallel |
| Antares Harmony Engine | ~$200 | 4 | None | No — parallel |
| Waves Harmony | ~$30 | 8 | None | No — parallel |
| iZotope Nectar 4 | ~$250 | 4 | None — snaps to scale | No — parallel |

**The problem:** Every one of these is a "diatonic interval transposer." Set the key, set the interval (3rd, 5th, octave), and the harmony mechanically follows your melody in lockstep. Palestrina would call this "bad counterpoint" — voices moving in parallel destroy their independence.

### Category 2: MIDI Chord & Progression Tools
**What they do:** Help you build chord progressions, trigger chord voicings, suggest harmonic ideas.
**What they DON'T do:** Generate counterpoint against your live playing.

| Product | Price | Voice Leading | Real-Time vs Live Input |
|---------|-------|---------------|------------------------|
| Scaler 3 | ~$60 | Auto voice leading between PRE-SELECTED chords | No — you pick chords first, it voices them smoothly |
| Cthulhu (Xfer) | ~$40 | None | Chord triggering from single notes, not counterpoint |
| Captain Chords | ~$80 | None | Chord progression generator |
| Harmony Bloom | ~$50 | None | Polyrhythmic MIDI pattern generator |
| MIDI Agent | ~$30 | Claims it via LLM | Prompt-based generation, not real-time |

**Scaler 3 is the closest** — its "Auto Voice Leading" moves smoothly between chords, the way a trained pianist would. But it works on chord progressions you've already selected, not on a live note-by-note input stream. It doesn't generate an independent contrapuntal line.

### Category 3: Academic & Research Systems
**What they do:** Actually generate counterpoint, sometimes in real-time. But none are products.

| System | Year | Real-Time? | Live MIDI? | Styles | Available? |
|--------|------|-----------|-----------|--------|-----------|
| **BachDuet** (U. Rochester) | 2020 | Yes | Yes | Bach only (neural network) | Web app, open source |
| **CountGen** (Stanford CCRMA) | 2010s | Yes | Audio only, not MIDI | Palestrina only | Academic prototype |
| **Strasheela** (T. Anders) | 2000s | Yes (35ms) | Via SuperCollider+OSC | Configurable (requires Oz programming) | Unix only, 4+ tool setup |
| **Bell's MAX patch** (ICMC) | 1995 | Yes | Yes | Configurable per-composer profiles | Not publicly available |
| **Tonica Fugata** (Capella) | Current | No — offline | No | Bach, Reger, Jazz/Pop | Commercial, ~$80 |
| **Coconet** (Google Magenta) | 2019 | Semi | No — draw melody first | Bach (neural) | Open source |
| **DeepBach** | 2016 | Interactive | No — MuseScore plugin | Bach (neural) | Open source |
| **Contrapunctus** (academic) | 2010s | No — batch | No | Fux, Jeppesen, Salzer | Paper only |
| **Ebcioglu CHORAL** (IBM) | 1990 | No — batch | No | Bach (350+ expert rules) | Research artifact |

**BachDuet is the closest true competitor:** It generates genuinely independent counterpoint in real-time against live MIDI input. But it's neural-only (no explicit rules, can't guarantee "no parallel fifths"), 2 voices only, Bach style only, web-only, and has no DAW integration.

---

## The Gap

| Capability | Pedals | Plugins | BachDuet | Tonica Fugata | Strasheela | **Contrapunk** |
|-----------|--------|---------|----------|---------------|------------|---------------|
| Real-time | Yes | Yes | Yes | No | Yes (35ms) | **Yes (<10ms)** |
| Live MIDI input | Some | Some | Yes | No | Via toolchain | **Yes** |
| Independent melodic lines | No | No | Yes | Yes | Yes | **Yes** |
| Explicit voice-leading rules | No | No | No (learned) | Yes | Yes | **Yes** |
| Parallel fifth avoidance | No | No | Implicit | Yes | Yes | **Yes (hard reject)** |
| Configurable styles | No | No | No (Bach only) | Yes (3 styles) | Yes (programming) | **Yes (4 styles, dropdown)** |
| Multi-voice (3+) | Some (up to 8) | Some (up to 8) | No (2 only) | Yes (4) | No (2) | **Yes (2-8)** |
| Modal interchange | No | No | No | No | No | **Yes (5 range levels)** |
| 28+ scale modes | No | Limited | No | Limited | No | **Yes** |
| Standalone instrument | Yes | Yes (VST) | Web only | Desktop | No (4+ tools) | **Yes (native + web)** |
| Open source | No | No | Yes | No | Yes | **Yes (MIT)** |
| No programming required | Yes | Yes | Yes | Yes | No (Oz lang) | **Yes** |

**What the gap looks like in one sentence:**

Every existing tool is either (a) a parallel interval transposer with no voice-leading awareness, (b) an offline composition tool with proper rules but no live performance, or (c) an academic prototype that requires assembling a complex toolchain. No product lets a musician plug in a MIDI keyboard, select "Palestrina style," and immediately get real-time counterpoint that follows 500 years of voice-leading rules.

---

## The Claim (for the talk)

> "To our knowledge, Contrapunk is the first standalone instrument that combines real-time counterpoint generation against live MIDI input with configurable historical voice-leading styles — from Palestrina through Jazz — in a single usable tool for live performers."

**This claim is defensible because:**
1. BachDuet does real-time counterpoint but is neural-only, single-style, 2-voice, web-only
2. Tonica Fugata has multi-style rules but is entirely offline
3. Strasheela has configurable constraints but requires a Unix toolchain and programming
4. No commercial plugin (VST/AU) generates independent contrapuntal lines with voice-leading rules
5. No DAW (Logic, Ableton, Bitwig, Reason) has built-in counterpoint generation
6. No modular environment (VCV Rack, Reaktor, The Grid) has counterpoint-aware modules

**Caveat:** There may be obscure Max/MSP patches or custom SuperCollider scripts in individual performers' setups. The claim applies to publicly available, documented tools.

---

## Why This Gap Exists

1. **Counterpoint is hard to compute in real-time.** The cartesian product of voice placements grows exponentially with voice count. Most systems punt to offline batch processing.

2. **Rules vary by era.** Palestrina's rules contradict jazz practice (parallel fifths: forbidden vs. fine). Building a configurable system means encoding multiple contradictory rule sets.

3. **The market is split.** Hardware pedal buyers want simple "add a third above." Academic researchers want correct species counterpoint. Live performers want both — correct AND real-time — and nobody built it.

4. **Neural approaches avoid rules.** BachDuet and DeepBach use neural networks, which learn implicit rules from data. This avoids the hard work of encoding explicit rules — but sacrifices configurability and guarantees. You can't tell a neural network "use Palestrina rules, not Jazz rules."

5. **Rust + WASM didn't exist when earlier systems were built.** Bell's 1995 Max patch, Strasheela's Oz implementation, and Ebcioglu's CHORAL were limited by the performance of their era. Rust's zero-cost abstractions make exhaustive cartesian scoring feasible in <1ms — the core enabler for Contrapunk's approach.

---

## Sources

- BachDuet: labsites.rochester.edu/air/projects/BachDuet.html
- CountGen: ccrma.stanford.edu/~srsmith/projects/CountGen.html
- Strasheela: strasheela.sourceforge.net
- Bell ICMC 1995: quod.lib.umich.edu/i/icmc/bbp2372.1995.143
- Tonica Fugata: capella-software.com
- Scaler 3: scalermusic.com
- Google Coconet: magenta.tensorflow.org/coconet
- DeepBach: arxiv.org/pdf/1612.01010
- Ebcioglu CHORAL: quod.lib.umich.edu/i/icmc/bbp2372.1986.086
- Music Mouse: eventideaudio.com/software/music-mouse
- Contrapunctus: academia.edu/25995307
