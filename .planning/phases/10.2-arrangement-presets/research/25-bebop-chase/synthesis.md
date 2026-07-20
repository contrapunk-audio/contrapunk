# Synthesis: Preset 25 — Bebop Chase

**Decision:** `ready_for_implementation`

**Reference scope:** Charlie Parker and Dizzy Gillespie's shared 1945–46 small-group bebop language, bounded by the turn-taking evidence of Parker/Gillespie exchanges on “Ko Ko” (1945) and “Leap Frog” (1950)

**Operational claim:** one clean four-to-six-note in-scale burst returns four transport beats later as a complete monophonic octave answer; the player supplies the call, gap, harmonic sense, swing, accents, development, and ending

## 1. Referenced scope

The bounded corpus is the pair's shared 1945–46 small-group work—especially “Groovin' High,” “Salt Peanuts,” “Shaw 'Nuff,” “Hot House,” “Ko Ko,” and the Town Hall period—plus the unusually explicit four-bar exchanges on the 1950 reunion recording “Leap Frog.” The preset references compatible intent and bounded turn-taking inside a shared rhythmic-harmonic language. It does not merge the artists into one pseudo-soloist.

The intersection supported by the reports includes:

- stable swing pulse and form awareness;
- fast, clean, harmonically directed monophonic lines;
- chromatic approaches serving tonal destinations rather than random outside color;
- compact coordinated statements followed by distinct individual turns;
- recurring material flexibly displaced, extended, and answered.

“Leap Frog” is the strongest direct evidence for equal phrase exchanges. The reports do not support a claim that Parker and Gillespie characteristically answered one another with automatically shortened copies.

## 2. Agreements and disagreements

All three reports agree that the musical center is active exchange, not layered canonic texture. They also agree that credible bebop requires phrase boundaries, chord-scale targeting, chromatic approach logic, swing/accent detail, and role-aware response—capabilities the current engine does not possess.

The reports disagree only on how much current Free Imitation can honestly stand in for that exchange:

- **History:** a delayed reduced answer can be a disclosed abstraction, but literal copying is weak evidence for the historical practice.
- **Theory:** delayed note-stream imitation is not phrase recognition; approve only after narrowing the claim or keep the original claim gated.
- **Performance:** a complete delayed echo can be playable if source and answer occupy separate windows; its proposed `timeRatio < 1` is only valid with a bounded captured phrase or enough causal look-ahead.

The parent decision resolves this conservatively: remove “shortened,” remove motif selection, retain every attack, use `timeRatio: 1`, and constrain the performer to a short call that ends before the four-beat answer begins.

## 3. What Contrapunk can operationalize now

Use the existing shared Free Imitation lane as a one-voice, fixed-delay response:

```json
{
  "harmony": {
    "mode": "pass_through",
    "voiceCount": 1
  },
  "companion": {
    "enabled": true,
    "holdMode": { "near_future": { "tailBeats": 4.0 } },
    "canon": {
      "enabled": true,
      "kind": "free_imitation",
      "voices": [
        {
          "delayBeats": 4.0,
          "transposeDegrees": 7,
          "timeRatio": 1.0,
          "holdMode": null,
          "enabled": true
        }
      ]
    },
    "counterpoint": { "enabled": false }
  }
}
```

Exact field casing may follow the catalog serializer, but the musical contract must not change.

### User-facing contract

- Use transport in 4/4 at roughly 160 BPM; preserve the user's current tonic, BPM, transport, devices, routing, sound, master, mute/solo, and plugins.
- Play four to six legato or detached eighth-note attacks within at most three beats.
- Stop playing and leave the next four-beat window open.
- The companion repeats every attack four beats later, one scale octave above, with the original velocity, inter-onset timing, and gate lengths.
- Begin the next call only after the answer clears if a strict call-and-response effect is desired.

The performer creates rhythm, swing, rests, accents, contour, chord targeting, phrase boundaries, and any later variation. Contrapunk supplies only delayed, octave-displaced pursuit.

### Why `timeRatio: 1.0`

A ratio below one schedules later notes of a live line earlier relative to the first note. Without phrase capture or a declared maximum phrase duration, that is not generally causal and the validator correctly treats it as a phrase-capture requirement. Keeping unity timing preserves every attack and makes NoteOn/NoteOff ownership exact.

### Why `near_future: 4.0`

The preset must allow already-scheduled delayed attacks to finish after the player's source notes are released, while remaining bounded. A four-beat tail retains the promised response and avoids Forever-hold ambiguity. Normal stop/panic cleanup must still cancel all owned notes.

## 4. What must remain manual or explicitly out of scope

The current preset does **not** provide:

- phrase capture, phrase-end detection, or form awareness;
- motif extraction, salient-note selection, truncation, reduction, or continuation;
- shortened answers or proportional phrase compression;
- automatic trading fours, chorus tracking, or role switching;
- Parker- or Gillespie-specific vocabulary, identity, articulation, tone, or swing;
- chord-change following, guide-tone targeting, altered-dominant logic, or chromatic enclosure generation;
- automatic unisons, head arrangement, accompaniment, bass, or drums.

Public text must not use “AI improviser,” “bebop generator,” “virtual Parker/Gillespie,” “shortened answer,” or “trading fours.”

## 5. Preset catalog recommendation

Use copy equivalent to:

- **Name:** Bebop Chase
- **Summary:** Four-to-six-note bursts return four beats later as a complete octave answer.
- **Result:** A clean monophonic burst comes back one scale octave higher; every source attack is retained.
- **Play tip:** In 4/4, play four to six in-scale eighth notes within three beats, then leave space for the four-beat-delayed answer.
- **Approximation:** A disclosed turn-taking exercise inspired by Parker/Gillespie small-group exchanges; no phrase recognition, shortening, swing, chord logic, or artist imitation.
- **Requires:** transport and shared Free Imitation only.

This creates a useful fast-burst call-and-response study and remains distinct from Mensuration Web: one answer voice, unity time, fixed octave, and separated performance windows rather than three concurrent proportional layers.

## 6. Approval boundary

Approved now only under the narrowed operational claim above. The original catalog phrase “shortened answer” is rejected.

Do not add preset-specific phrase machinery. A future reusable phrase-response lane may upgrade the preset only when it can:

1. capture a bounded phrase and detect or receive its end;
2. choose a musically coherent fragment or continuation;
3. schedule a shorter response causally;
4. preserve exact NoteOn/NoteOff ownership through retrigger, stop, panic, preset change, and transport loss;
5. pass shared lifecycle and catalog validation tests.

Until then, every implementation and UI description must call the output a complete delayed octave answer, not a shortened bebop reply.

## 7. Implementation acceptance checks

1. Catalog validation accepts the preset as approved with only `free_imitation` and `transport` requirements.
2. Applying it preserves all non-arrangement state named above.
3. An in-scale four-note eighth-note source produces exactly four response NoteOns four beats later and exactly four matched NoteOffs.
4. The response pitches are one scale octave above, and source velocities, timing, and gates are unchanged.
5. Releasing source notes before response onset does not erase the scheduled response.
6. Stop, panic, preset change, and transport loss leave zero orphan notes.
7. UI copy contains no claim of shortening, phrase recognition, swing generation, trading fours, or artist simulation.
8. Shared harmony, catalog/persistence, and UI checks pass with no new warnings.

## 8. Evidence trail

This synthesis depends on the three independent cited reports in this directory:

- `history.md` for documented collaboration, bounded recording evidence, and attribution risk;
- `theory.md` for the grammar of bebop response and the engine capability gap;
- `performance.md` for playable windows, perceptual thresholds, lifecycle risks, and acceptance targets.

Those reports contain the full citations. Where their recommendations conflict, this synthesis adopts the least expansive claim the present shared engine can make truthfully.
