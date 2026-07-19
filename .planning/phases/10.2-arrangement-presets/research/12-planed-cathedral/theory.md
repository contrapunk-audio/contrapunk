# Research: Preset 12 — Planed Cathedral (Debussy), music-theory and temporal behavior

## Summary

A defensible preset can represent one bounded slice of Debussy's 1901–1910 piano language: slow, register-separated, non-functional chord planes drawn from a single whole-tone **or** pentatonic collection, articulated by a sparse monophonic source. It must not imply that *La cathédrale engloutie* is simply “whole-tone chords”: score-based scholarship describes that work as moving among five-, six-, and seven-note regions, often with a perceptible center, motivic links, pedal/common tones, and large formal/dynamic changes.

**Decision: `needs_shared_capability`.** No preset-specific HarmonyEngine branch is justified. The existing `DiatonicThirds + WholeTone` path can honestly produce a narrow three-note augmented-triad plane, and `DiatonicThirds` or `DiatonicFourths` over a pentatonic scale can produce collection-preserving *diatonic* planing. But the current preset record cannot enforce voice count/position, and the engine cannot switch collection/planing type, preserve a pedal, or evolve density/register across phrase states. Add only shared preset/configuration fields if the operational acceptance requires consistent three-note planes; defer multi-state evolution rather than faking it.

## Scope and selected-period limit

Primary anchor: Debussy, *Préludes*, Book I (1910), no. 10, “La cathédrale engloutie,” checked against the public-domain Durand score listing. Comparative evidence is limited to the coloristic piano language surrounding *Pour le piano* (1901), “Pagodes” (1903), “Voiles” (1909), and Book I of the *Préludes* (1910). [Primary score listing](https://imslp.org/wiki/Pr%C3%A9ludes%2C_Livre_1%2C_CD_125_(Debussy%2C_Claude))

This is not a whole-career Debussy model. Later bitonal/polychordal works and the 1915 *Études* are excluded from the implementation target, though Uchida's historical survey is useful for showing why one static scale is not a career-wide description. [Uchida thesis](https://scholarsbank.uoregon.edu/server/api/core/bitstreams/02c02733-9c18-4b3d-a80e-7b6f23990771/content)

## Findings

### 1. Tonal center and collection interaction

1. **Center is often asserted without continuous functional syntax (high confidence).** In “La cathédrale,” the broad reference center is C, but local regions, pedals, open fifths, registral emphasis, and return carry more weight than repeated V–I motion. Waters specifically finds movement among 5-, 6-, and 7-note diatonic regions while avoiding functional progressions; mm. 16–19 move from B pentatonic to a six-note E-flat collection, joined by enharmonic common tone and motivic completion. [Waters, paras. 5–6](https://mtosmt.org/issues/mto.12.18.3/mto.12.18.3.waters.html)
2. **Whole-tone, pentatonic, diatonic/modal, and chromatic materials interact by juxtaposition and overlap, not by one permanent “Debussy scale” (high).** Uchida documents pentatonic material used alone and combined with diatonic/modal material; whole-tone passages can alternate with pentatonic passages, and pedal tones can restore center within symmetric collections. [Uchida, pp. 82–86](https://scholarsbank.uoregon.edu/server/api/core/bitstreams/02c02733-9c18-4b3d-a80e-7b6f23990771/content)
3. **Pentatonicism does not automatically erase tonality (high).** Day-O'Connell places Debussy's pentatonic practice in continuity with nineteenth-century tonal practice and identifies cadential interaction rather than treating pentatonicism as uniformly non-tonal. [Day-O’Connell](https://www.skidmore.edu/music/documents/Day-OConnell_Debussy_Pentatonicism.pdf)
4. **Whole-tone stasis is work-specific (high).** Waters reports that “Voiles,” unusually among Book I preludes, relies on fixed collections for extended spans and may be heard either as non-triadic stasis or, in Howat's competing reading, as an extended B-flat dominant leading to E-flat minor. It is therefore useful evidence for a bounded whole-tone state, not a template for all Debussy. [Waters, paras. 29–32](https://mtosmt.org/issues/mto.12.18.3/mto.12.18.3.waters.html)
5. **Chromatic tones frequently bridge rather than merely decorate (high).** In *Prélude à l’après-midi d’un faune*, Waters identifies chromatic lead-ins into thematic/whole-tone returns; in “La cathédrale,” the E-flat at m. 19 both completes a motive and forms an enharmonic common tone at a collection change. [Waters, paras. 6–10 and n.4](https://mtosmt.org/issues/mto.12.18.3/mto.12.18.3.waters.html)

### 2. Chord vocabulary, planing, spacing, inversion, doubling

6. **Planing has at least three behaviorally distinct types (high):**
   - **Exact/real/chromatic:** every chord member moves by the same semitone interval, preserving chord quality and inversion; e.g. parallel augmented triads within a whole-tone collection.
   - **Diatonic/tonal:** every member moves by the same number of scale degrees, so interval sizes and chord qualities can change while the collection is preserved.
   - **Mixed/anchored:** some voices plane while a bass pedal, common tone, contrary bass, octave displacement, or collection boundary does not.
   Uchida explicitly distinguishes diatonic/chromatic and tonal/real planing and notes that it may involve one interval or complex chords. [Uchida, p. 88](https://scholarsbank.uoregon.edu/server/api/core/bitstreams/02c02733-9c18-4b3d-a80e-7b6f23990771/content)
7. **Vocabulary is broader than triads (high).** Relevant sonorities include open fourths/fifths, quartal/quintal chords, augmented triads, added-sixth/added-second sonorities, dominant ninths treated non-functionally, and occasional conventional triads/cadential supports. Uchida's analytical survey lists these categories and gives score examples of augmented-triad, ninth-chord, quartal, and added-tone planing. [Uchida, pp. 11–13, 26–28, 88–90](https://scholarsbank.uoregon.edu/server/api/core/bitstreams/02c02733-9c18-4b3d-a80e-7b6f23990771/content)
8. **“La cathédrale” foregrounds open spacing and registral separation (medium-high).** Score-based analysis describes low, widely spaced fourth/fifth sonorities separated from higher material, later expanding into organ-like block texture. This is at least as important to the cathedral association as scale choice. [Rodríguez Alvira analysis](https://www.teoria.com/en/articles/2021/debussy-preludes/10/index.php)
9. **Inversion is not governed by classical root-position cadence rules (high).** In exact planes, inversion normally remains fixed because the entire pitch set moves intact. In collection-preserving diatonic planes, surface “root” and chord quality can shift by step. Mixed passages may keep an upper structure while the bass changes its harmonic meaning: Waters shows this explicitly in “Reflets dans l’eau,” mm. 39–47. [Waters, paras. 24–25](https://mtosmt.org/issues/mto.12.18.3/mto.12.18.3.waters.html)
10. **Doubling is coloristic and registral, not SATB-error avoidance (medium-high).** Octaves and fifths may be deliberately doubled, especially at dynamic/register peaks. The safe bounded preset should use 3 distinct pitch classes for a whole-tone augmented plane and may add octave doubling only as an optional intensity parameter; constant full “Mirror” tripling would overstate the texture.

### 3. Melody and voice motion

11. **Source melody should be short, scalar/gapped, and motivically recurrent (high).** “La cathédrale” is built from compact cells and returns rather than a continuously novel tune. Waters's mm. 16–19 example shows truncation, restart, and delayed completion of a five-note idea. [Waters, para. 6](https://mtosmt.org/issues/mto.12.18.3/mto.12.18.3.waters.html)
12. **Parallel motion is a foreground color, not a universal texture (high).** Exact or diatonic parallel blocks can alternate with pedals, oblique sustained tones, contrary bass, and unharmonized/pared-down gestures. A voice-leading postprocessor that rewards common tones or contrary motion destroys exact planing; one that spreads/re-octaves individual chord tones can also change fixed inversion.
13. **Melodic continuity can cross collection/section boundaries (high).** Pattern completion, common tones, augmentation, and overlap create continuity even where harmony changes abruptly. Waters's broader model of “additive variation” retains old material while adding/stripping other layers. [Waters, paras. 11–12](https://mtosmt.org/issues/mto.12.18.3/mto.12.18.3.waters.html)

### 4. Rhythm, phrase, form, register, dynamics

14. **Rhythm is spacious but not metronomically inert (medium-high).** The opening of “La cathédrale” is marked *Profondément calme*; long values, rests, fermata-like breadth, and block attacks matter. The piece later intensifies through denser repeated/chordal attacks and then withdraws. A preset should follow source note onsets rather than inventing busy subdivisions.
15. **Formal pacing is arch-like, but analytical boundaries compete (medium).** A commonly cited reading is ABCBA with divisions at mm. 28, 47, 72, and 84; other analysts divide the piece differently. The invariant is not exact bar counts but the audible trajectory from distant/sparse to present/massive and back to distant. [Performance/form survey quoting Howat](https://www.thefreelibrary.com/Solving+performance+problems+in+Debussy%27s%3A+La+Cathedrale+Engloutie.-a0149767135)
16. **Register and dynamics are formal parameters (high).** Low/high separation and soft attack create distance; convergence, thicker block sonority, stronger dynamics, and broader register create emergence; thinning, lower energy, and restored separation create submergence. They cannot be reduced to “select Whole Tone.”
17. **Across-section change is often continuous at the boundary (high).** Debussy may retain a motive, figuration, upper structure, bass note, or final chord across a boundary while another dimension changes. In “La cathédrale,” pattern completion joins remote collections; in “Reflets,” upper structure and figuration persist while bass/harmonic meaning changes. [Waters, paras. 5–6, 24–27](https://mtosmt.org/issues/mto.12.18.3/mto.12.18.3.waters.html)

## Testable stylistic invariants

1. For the bounded whole-tone realization, every accepted in-scale input produces only pitch classes from the selected whole-tone collection.
2. With exactly 3 output voices and `DiatonicThirds`, each in-scale attack produces three distinct pitch classes separated by 4 semitones modulo 12 (an augmented triad); successive input steps preserve the interval vector exactly.
3. Voice leading is disabled during exact planing; no generated voice may be independently re-octaved between consecutive plane attacks unless the optional global octave-doubling state is explicitly active.
4. Output onsets and releases track the source gesture exactly; no autonomous fast pattern is introduced.
5. Default texture is three voices or fewer, sustained and sparse; silence in the input remains silence.
6. A bounded pentatonic variant must be labeled **diatonic/collection-preserving planing**, not exact chord planing; chord interval content is allowed to change by degree.
7. Chromatic/out-of-collection input must not be advertised as exact planing: current fallback changes interval choice to land harmony in scale.
8. The preset copy must say “whole-tone chord plane inspired by a selected Debussy piano technique,” not “Debussy harmonization” or “the sound of La cathédrale.”
9. No classical prohibition on parallel fifths/octaves is applied; parallels are intentional.
10. At section/intensity peaks, any thickening should be by bounded octave doubling/register expansion, never random added scale degrees.

## Variable parameters that preserve identity

- Tonic/transposition of either whole-tone collection (only two distinct pitch-class sets).
- Whole-tone exact augmented plane **or** major-pentatonic diatonic plane, provided the UI names the active type.
- Two versus three sounding pitch classes; three is preferred for the whole-tone identity.
- Source tempo roughly slow to moderate (about 40–80 quarter-note pulse), with player-controlled rubato.
- Note duration, release, and inter-gesture silence.
- Register separation and optional single-octave doubling at the peak.
- Plane direction/voice position, if fixed for the gesture and range-safe.
- Dynamic contour controlled by performer velocity; soft default, stronger middle, withdrawal at end.

## Misleading behaviors

- **Blocker:** calling all output “La cathédrale” while using a permanent whole-tone collection; the source work has pentatonic, hexatonic, diatonic, modal/tonal, and chromatic interactions.
- **Blocker:** enabling Palestrina/Bach voice-leading rules; they penalize the defining parallel motion.
- **High:** `OctaveMode::Mirror` as a default; it triples every harmony voice and can create an unbounded, continuously huge sonority rather than a staged registral peak.
- **High:** modal interchange as a substitute for collection evolution; current interchange chooses the first parallel mode containing an out-of-scale note and is neither temporally directed nor whole-tone/pentatonic-aware.
- **High:** random-below modes; stochastic chord membership contradicts coherent planes and fixed inversion.
- **High:** `DiatonicFourths + WholeTone` advertised as quartal planing. In a six-note whole-tone scale, +3 degrees is a tritone, not a perfect fourth.
- **Medium:** continuous legato note streams with no rests; they make every melodic note a new block and produce harmonic churn.
- **Medium:** automatic key detection; a symmetric whole-tone collection has no unique tonic and key changes would be arbitrary/misleading.

## Temporal state sketch and triggers

`Distant outline → Rising plane → Emerged mass → Dissolve/bridge → Submerged return`

- **Distant outline:** trigger = preset load or ≥2 beats of silence. Soft velocity; 1–2 held/open voices; wide low/high separation; center suggested by register/pedal, not cadence.
- **Rising plane:** trigger = 2–4 connected in-collection notes at slow rate. Three-note exact whole-tone or diatonic pentatonic planes; retain one register layout; gradual velocity/register rise comes from performer.
- **Emerged mass:** trigger = sustained higher velocity and/or higher register for 2–4 attacks. Optional octave doubling and longer duration, bounded in count; no random chord tones.
- **Dissolve/bridge:** trigger = falling velocity, a long held note, or a chromatic boundary note. Thin to dyad/single note; a future shared implementation may retain one common/pedal tone while collection changes.
- **Submerged return:** trigger = ≥1 bar of sparse/soft input after peak or phrase rest. Return to original collection/register and 1–2 voices.

**Current-engine reality:** only player-driven versions of these states are available. The engine does not analyze velocity/density into scene transitions, preserve pedals independently, or schedule section changes. Therefore the first implementation should remain a static “whole-tone plane” and put the arch in performance guidance; it must not claim automatic evolution.

## Abstract degree/beat examples

Notation: `WTn` = degree n of a six-note whole-tone collection; `Pn` = degree n of a five-note major-pentatonic collection. Brackets are simultaneous pitch classes; apostrophe means next octave. These are synthetic examples, not quotations.

1. **Exact whole-tone plane, three voices (4/4):** input `b1 WT1 (half), b3 WT2 (half)` → output `b1 [WT1, WT3, WT5]`, `b3 [WT2, WT4, WT6]`. Semitone form from tonic: `[0,4,8] → [2,6,10]`; every voice moves +2 semitones and chord quality/inversion is fixed.
2. **Exact plane with space and return (4/4):** input `b1 WT4 (dotted half), b4 rest; next b1 WT3 (whole)` → `[WT4,WT6,WT2′]`, silence preserved, then `[WT3,WT5,WT1′]`. Expected: no autonomous fill during rest; fixed interval vector at return.
3. **Pentatonic diatonic plane (3/4):** input `b1 P1 (quarter), b2 P2 (quarter), b3 P3 (quarter)` → degree stacks `[P1,P3,P5] → [P2,P4,P1′] → [P3,P5,P2′]`. Expected: all tones remain pentatonic, but semitone interval content changes; label this diatonic, not exact, planing.
4. **Mixed/anchored target showing a current gap (4/4):** pedal `P1` held for the bar while upper plane attacks on b1 `[P3,P5]`, b3 `[P4,P1′]`. Expected future output `[P1 | P3,P5] → [P1 | P4,P1′]`; current HarmonyEngine cannot keep an independently sustained generated pedal while reharmonizing upper voices from monophonic input.
5. **Collection bridge target showing a current gap:** b1–2 `[WT1,WT3,WT5]`; chromatic lead-in on b4; next bar b1 `[P1,P3,P5]` with one enharmonic/common pitch retained. Current engine can only treat the chromatic input through consonant fallback/interchange; it cannot schedule the collection switch or require the common tone.

## Exact mapping to current capabilities

| Requirement | Current mapping | Honest result / named gap |
|---|---|---|
| Whole-tone pitch collection | `crates/contrapunk-harmony/src/config.rs`: `ScaleMode::WholeTone = [0,2,4,6,8,10]` | Exact collection exists. |
| Pentatonic collection | `config.rs`: `MajorPentatonic`, `MinorPentatonic`, others | Major pentatonic is the safest bounded comparative option; no automatic switching. |
| Exact augmented-triad plane | `modes.rs`: `diatonic_thirds_directed`; `engine.rs`: chained harmonies; choose WholeTone and 3 voices | Existing algorithm works for in-scale input. **Gap:** `StylePreset` cannot store/apply `voice_count` or `voice_position`. |
| Pentatonic diatonic planing | `DiatonicThirds` or `DiatonicFourths` over `MajorPentatonic` | Collection-preserving, changing intervals. Honest only if labeled diatonic planing. |
| Exact arbitrary chord/interval plane | None | **Named gap: fixed interval-stack/exact-planing strategy.** Do not add for the narrow WT augmented version. |
| Mixed planing with pedal/common tone | None in per-note engine | **Named gap: independent pedal/anchor voice lifecycle.** |
| Preserve parallel motion | Disable voice leading | `VoiceLeadingStyle::Free` still revoices if enabled; safest is `voice_leading_enabled=false`. |
| Register control | `voice_position`; `OctaveMode::{None,Spread,BassTrebleSplit,Mirror}`; optional VL register ranges | Coarse. Spread moves successive voices by increasing octave amounts and does not model one coherent plane. Use `None` initially. |
| Dynamics/articulation | MIDI input lifecycle passes through | Harmony pitch generation has no style-specific velocity/articulation transform. Performer must supply contour. |
| Temporal states/section arc | Beat phase exists mainly for counterpoint; no state graph | **Named gap: phrase/intensity scene evolution.** Not required for narrow static v1, but UI copy must say player-driven. |
| Collection changes | setters can change `scale_mode` and reharmonize held input | Manual/config change only; **no phrase trigger or crossfade/common-tone policy**. |
| Chromatic bridge | `scale.rs::harmonize_smart` uses interchange or consonant fallback | Not exact planing and not deterministic stylistic bridging. Disable interchange and constrain performer to in-scale notes for acceptance. |
| Preset storage/load | `crates/contrapunk-preset/src/lib.rs`; `src-tauri/src/commands/presets.rs` | Stores mode/key/VL/octave/scale/interchange only. **Blocker for invariant output density:** no voice count/position. |
| Current built-ins | `crates/contrapunk-preset/src/builtins.rs` | No Planed Cathedral record yet; existing records demonstrate only static config bundles. |

### Minimal honest operational configuration

- `harmony_mode = DiatonicThirds`
- `scale_mode = WholeTone`
- `voice_count = 3` (**requires shared preset-schema/application support, not an engine branch**)
- `voice_position = 2` (source as lowest of three) or another explicitly tested fixed position
- `voice_leading_enabled = false`
- `octave_mode = None`
- `interchange_enabled = false`
- auto-key off
- performer contract: in-scale monophonic notes, slow/held, rests between gestures; manually shape soft → strong/high → soft/low.

With those bounds, existing HarmonyEngine behavior honestly realizes a whole-tone augmented-triad plane. It does **not** realize the broader temporal/cross-collection “Planed Cathedral” thesis automatically.

## Concrete review findings

- **blocker — `crates/contrapunk-preset/src/lib.rs` (`StylePreset`) and `src-tauri/src/commands/presets.rs` (`apply_preset_inner`)**: preset data cannot declare/apply voice count or voice position. A user left at 2 or 4+ voices will not receive the required three-distinct-pitch augmented plane; 4 voices repeat an augmented-triad pitch class.
- **high — `crates/contrapunk-harmony/src/scale.rs` (`harmonize_smart` / `harmonize_chromatic`)**: out-of-scale notes invoke consonant fallback or modal borrowing rather than preserving an exact plane. Acceptance input must be constrained to the selected collection.
- **high — `crates/contrapunk-harmony/src/engine.rs` (`VoiceLeadingProcessor`, `apply_octave_mode`)**: enabling revoicing or coarse octave modes can alter fixed inversion/register continuity. Disable both for the bounded exact-plane version.
- **medium — `crates/contrapunk-harmony/src/config.rs` / `modes.rs`**: the name `DiatonicThirds` is mathematically correct over arbitrary cardinalities but can obscure that +2 degrees in WholeTone is a major third and +3 is a tritone. Tests/preset copy should assert semitone outcomes rather than rely on common-practice interval labels.
- **medium — `.planning/phases/10.2-arrangement-presets/10.2-PRESET-CATALOG.md`**: “whole-tone or pentatonic parallel chord planes” is implementable only as alternatives under current static configuration, not evolving interaction inside one preset. Treat “or” literally in v1.

## Competing interpretations and confidence

- **Tonal versus non-tonal Debussy:** Pomeroy/Waters characterize “Voiles” as a rare non-triadic fixed-collection experiment, while Howat hears an extended dominant-to-tonic relation. Confidence high that both hearings exist; low confidence in any claim that whole-tone usage is categorically atonal.
- **Form of “La cathédrale”:** Howat's ABCBA divisions are influential, but surveyed analyses disagree. Confidence high in the emergence/peak/withdrawal arc, medium in exact section labels, low in making bar counts an implementation invariant.
- **Planing taxonomy:** exact versus diatonic is robust and directly testable; “mixed planing” is an implementation umbrella for planed upper structures plus pedal/common/contrary layers rather than a single universally standardized category. Confidence high for the behaviors, medium for the label.
- **Cathedral timbre from open fifths/register:** score evidence is strong; the programmatic identification as bells/organ is interpretive. Confidence high in pitch/register facts, medium in the image attached to them.

## Sources

### Kept

- [Debussy, *Préludes*, Book I — IMSLP score record](https://imslp.org/wiki/Pr%C3%A9ludes%2C_Livre_1%2C_CD_125_(Debussy%2C_Claude)) — primary score access and work/date identity.
- [Keith Waters, “Other Good Bridges,” *Music Theory Online* 18.3](https://mtosmt.org/issues/mto.12.18.3/mto.12.18.3.waters.html) — peer-reviewed, measure-specific evidence for collection changes, motivic overlap, additive variation, temporality, and competing tonal readings.
- [Dmitri Tymoczko, “Scale Networks and Debussy,” *Journal of Music Theory* 48.2](https://doi.org/10.1215/00222909-48-2-219) — authoritative account of scalar adjacency and Debussy's expanded but organized pitch space.
- [Jeremy Day-O’Connell, “Debussy, Pentatonicism, and the Tonal Tradition”](https://www.skidmore.edu/music/documents/Day-OConnell_Debussy_Pentatonicism.pdf) — scholarly corrective to the claim that pentatonicism simply abolishes tonal function.
- [Rika Uchida, *Tonal Ambiguity in Debussy’s Piano Works*](https://scholarsbank.uoregon.edu/server/api/core/bitstreams/02c02733-9c18-4b3d-a80e-7b6f23990771/content) — university thesis with score-measure examples and explicit planing/collection taxonomy; used as supporting rather than sole authority.
- [José Rodríguez Alvira, score-based “La Cathédrale engloutie” analysis](https://www.teoria.com/en/articles/2021/debussy-preludes/10/index.php) — accessible score-linked evidence for open fourth/fifth spacing and registral separation.
- [Form/performance survey quoting Roy Howat](https://www.thefreelibrary.com/Solving+performance+problems+in+Debussy%27s%3A+La+Cathedrale+Engloutie.-a0149767135) — useful for competing form readings; treated as secondary.

### Dropped

- Wikipedia — redundant and less authoritative than the primary score and scholarly analyses.
- Generic “Debussy impressionism/whole-tone” teaching pages — dropped as caricature-prone and insufficiently measure-specific.
- Commercial sheet-music listings — metadata only, no analytical evidence.
- Academia.edu-only upload — provenance/access friction; stronger accessible sources covered the same issues.

## Gaps and residual risks

1. This report did not establish a single critical-edition fingering/pedaling interpretation; pedal behavior is therefore described abstractly, not encoded.
2. Exact dynamic/tempo performance practice varies, including evidence from Debussy-associated piano rolls; a performer-focused report should set the final interaction ranges.
3. The engine's exact MIDI output should be locked by tests for all six WholeTone degrees, both edge registers, and the selected `voice_position`; code inspection establishes the algorithm but not every range-wrap outcome.
4. A future pentatonic option needs separate acceptance tables because “+2 degrees” produces changing semitone structures.
5. Automated emergence/submergence remains intentionally out of scope until a shared intensity/scene capability exists.

## Implementation decision

`needs_shared_capability`

Reason: add shared `voice_count` (and preferably `voice_position`) fields to preset data/application so the existing mode produces a deterministic three-note plane. Do **not** add a Debussy/PlanedCathedral branch. Ship the first bounded variant as “whole-tone augmented chord plane; player shapes the arc.” Collection rotation, pedal retention, and automatic temporal evolution should remain unavailable rather than be simulated by interchange/randomness.
