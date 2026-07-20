# Synthesis: Preset 36 — Pixel Trio

**Decision:** `keep_locked`

**Blocking capability:** a reusable transport-aware three-role scheduler with stable role ownership, independent bass/counterline onset and rest patterns, register bounds, density ceiling, and exact lifecycle cleanup

**Reference scope:** selected 1985–87 Famicom/NES practice in Koji Kondo’s *Super Mario Bros.* and Manami Matsumae’s original *Mega Man*, limited to economical pitched-role allocation, hook clarity, repetition, contrast, and space

## 1. Historical claim boundary

“Three-channel” is an arrangement abstraction, not the hardware specification. The stock NES APU has five unequal sources: two pulse channels, triangle, noise, and DMC sample playback. Kondo’s “three sounds” and Matsumae’s “three notes/channels plus noise” support a bounded three-**pitched**-role concept while also proving that effects, noise, samples, drivers, memory, and game-specific channel allocation complicate it.

The shared intersection is:

- a memorable foreground hook;
- clear, scarce pitched roles;
- register separation;
- repetition with controlled variation;
- purposeful rests and temporary channel absence;
- short loop or cue identity tied to play context.

The named composers remain meaningfully different. Kondo’s selected evidence emphasizes prototype-tested movement, groove, location contrast, and music/effect permeability. Matsumae’s emphasizes heroic singable melody, chord implication, contrapuntal training, pop/rock rhythmic study, stage/character differentiation, and music/effect negotiation. The preset must not average these into a universal “8-bit style.”

## 2. Report agreement and conflict

All reports agree that the current engine cannot provide literal chip synthesis, percussion, looping, channel stealing, independent game-state adaptation, or either composer’s identity.

The history and theory reports further agree that “melody, bass, and counterline” are behavioral roles, not labels for three simultaneous pitches. A credible bass must sometimes move, sustain, or rest independently of the melody. A credible counterline must sometimes answer during a lead gap, use contrary/oblique motion, or remain silent. Full three-role density must be a peak rather than the default.

The performance report proposes a narrower current product: three total pitches from each clean input note, explicitly described as event-coupled shadows rather than independent roles. That could be a useful generic **Pixel Stack** exercise, but it does not satisfy the catalog identity of Pixel Trio. Renaming synchronous harmony outputs “bass” and “counterline” would be metadata-only role fiction.

The parent therefore keeps Pixel Trio locked rather than weakening its defining promise.

## 3. Why current shared strategies are insufficient

`HarmonyEngine` can cap output at three total pitches and place the source at an arrangement slot. `DiatonicThirds`, `DiatonicFourths`, `ContraryMotion`, and `StrictCounterpoint` can vary pitches or retain pitch history. However, every generated pitch still attacks and releases from the same source event.

Rejected current mappings:

- **Three DiatonicThirds voices:** deterministic close/medium diatonic planing; the supposed bass and counterline copy every lead onset, release, and contour transformation.
- **Three DiatonicFourths voices:** clearer open spacing, but still synchronous quartal planing and not a shared Kondo/Matsumae invariant.
- **ContraryMotion or StrictCounterpoint:** more independent pitch direction, but no independent rhythm, rests, role duration, loop, or bass ownership.
- **One harmony voice plus delayed Canon:** a fixed echo is not a counterline and still does not create a selectively moving bass.
- **Species IV Counterpoint plus harmony:** creates one bounded suspension behavior, not the repeating low support and complementary counterline contract.

Voice leading can revoice pitch classes; it cannot create independent role onsets, rests, articulation, density states, or loop seams. Role mix names do not change ownership semantics.

## 4. Required reusable capability

Do not add a preset-specific algorithm. Pixel Trio can unlock only after a shared limited-role arrangement capability provides:

1. exactly three pitched role owners: live lead, low support, counter/support;
2. transport-aware independent onset, duration, rest, and loop behavior for low and counter roles;
3. stable register policy with bass below the upper roles and bounded crossing;
4. density states containing one, two, and three active pitched roles, with three never continuously saturated;
5. deterministic hook-relative or harmony-state behavior without stored copyrighted melodies;
6. loop-seam thinning or cadence behavior;
7. optional effect/channel-yield policy only if explicitly implemented and tested;
8. ownership keys that distinguish lane instance, role, and source event;
9. exact NoteOn/NoteOff cleanup through stop, panic, preset replacement, transport loss, retrigger, and role reuse;
10. capability gating on every surface.

The minimum declared requirements are `pattern_lane` and `stable_lane_groups`. If the implementation derives replies from a captured user hook rather than from current scale/harmony state, it must additionally declare `motif_memory` or `phrase_capture`.

## 5. Target performer contract

When the reusable capability exists:

- Play a two-bar, four-to-eight-attack monophonic hook with a recognizable rhythm and at least one beat of rest.
- Repeat it exactly once, change one ending pitch or register, then leave a full-bar opening.
- Keep guitar input clean and single-string; avoid bends, slides, ringing strings, double-stops, and ambiguous releases.
- Keep keyboard sustain, arpeggiator, chord mode, and overlapping input off.
- Use a middle-register lead at roughly 100–150 BPM.
- The system may place a slower low support role and a complementary counterline, but must never exceed three pitched roles.
- Silence remains a positive arranged state; noise/DMC percussion and chip timbre remain separate sound capabilities.

The player still supplies the hook, accents, repetition, controlled variation, phrase return, cadence, and live intent.

## 6. Honest locked catalog copy

Use copy equivalent to:

- **Name:** Pixel Trio
- **Target result:** A compact live hook stays foreground while independent low support and one answering tonal line alternate under a three-pitched-role ceiling.
- **Play:** Perform a catchy two-bar single-note hook, repeat it exactly, change one ending, then leave a full bar for role separation and cleanup.
- **Approximation:** A future modern MIDI arrangement inspired by three-pitched-role economy in selected 1985–87 Famicom/NES practice. It will not emulate NES hardware, reproduce game music, simulate Kondo or Matsumae, or add unclaimed noise, DMC, percussion, chip timbre, sound effects, or game-state logic.
- **Locked reason:** Current HarmonyEngine outputs are synchronous shadows of each input note; Pixel Trio requires independent low-support and counterline timing with stable ownership.

Do not say “the NES had only three channels,” “authentic NES,” “hardware-accurate,” “Kondo preset,” “Matsumae style,” “automatic 8-bit music,” or name/quote any protected game melody.

## 7. Future acceptance checks

The preset may become operational only when one deterministic 4–8-bar fixture proves:

1. active pitched roles never exceed lead, low support, and counter/support;
2. bass and counter onset sets each differ from the lead onset set;
3. each generated role owns at least one rest while another role sounds;
4. low support is not a fixed interval shadow of every lead note;
5. at least one counterline event begins during a lead rest or sustains across a lead change;
6. full density is brief and the fixture contains one-, two-, and three-role states;
7. bass remains normally at least a perfect fifth below the nearest upper role;
8. repeated input yields deterministic role output without storing or quoting source-game material;
9. no notes are emitted during the final rest after the arrangement clears;
10. transport stop/loss, panic, preset replacement, retrigger, and repeated loops leave zero active or pending owners;
11. unsupported surfaces keep Apply disabled with a precise capability explanation;
12. product copy makes no chip synthesis, percussion, effect, loop, game-state, hardware-emulation, or artist-imitation claim.

## 8. Evidence trail

This synthesis depends on the three independent cited reports in this directory:

- `history.md` establishes the hardware truth, bounded Kondo/Matsumae corpora, shared economy, distinct vocabularies, career change, attribution, and copyright boundary;
- `theory.md` defines the independent-role invariant, rejects synchronous candidate mappings, audits current code, and specifies future ownership tests;
- `performance.md` supplies the clean hook contract, guitar/keyboard constraints, density and silence guidance, lifecycle behavior, and a viable generic three-pitch exercise.

The performance report’s synchronous three-pitch study remains a useful design idea but is not activated under Pixel Trio because it fails the historically and semantically central independent-role requirement.
