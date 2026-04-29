# Wk 7 Feature — Arpeggiator + sustain/legato mode

**Ship by**: Thu Jun 11, 2026 EOD
**Live for jam**: Fri Jun 12
**Risk**: Medium
**Pairs with**: Shimomura themed jam — see `website/.planning/jam-2026/composers/07-*.md`

---

## Minimum shippable definition (lock Monday of the build week)

Arpeggiator = takes current chord + plays notes in pattern (up/down/up-down/random). Sustain mode = MIDI input sustain pedal extends note durations until pedal release.

---

## Files to touch

`crates/contrapunk-midi/src/lib.rs` (sustain), new arpeggiator in transport

(Detailed file list + diffs to be filled in when this week's brief is fleshed out — 1 week before ship.)

---

## Day-by-day plan

To be filled in 1 week before ship. See `01-looper.md` for the template structure (Mon spec lock → Tue impl → Wed checkpoint → Thu ship → Fri jam).

---

## Wednesday descope option

Ship just arpeggiator; sustain mode pushes to a later patch week

---

## Acceptance criteria

- [ ] Feature works in both desktop (Tauri) and web (`app.contrapunk.com`) builds
- [ ] No regressions in existing functionality
- [ ] Demo video recorded
- [ ] Documented in 1 paragraph for the website's `/jam/7` page

---

## Demo for the jam (30-60 sec video)

Arpeggiator playing slow piano-style up-down on Cmaj7; melody improvised on top in Bach Chorale mode.

Save as `cover/demos/07-arpeggiator.mp4` in website repo.

---

## Risks

To be filled in 1 week before ship.

---

## Cross-feature notes

See `README.md` "Cross-feature dependencies" section.
