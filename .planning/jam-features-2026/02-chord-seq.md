# Wk 2 Feature — Chord progression sequencer

**Ship by**: Thu May 7, 2026 EOD
**Live for jam**: Fri May 8
**Risk**: Medium
**Pairs with**: Uematsu themed jam — see `website/.planning/jam-2026/composers/02-*.md`

---

## Minimum shippable definition (lock Monday of the build week)

Type a chord progression as text (e.g. `Am F C G`); transport plays through it; user improvises melody on top while engine harmonizes both.

---

## Files to touch

`crates/contrapunk-chord/`, `crates/contrapunk-transport/`, `src/router.rs`, new Svelte panel

(Detailed file list + diffs to be filled in when this week's brief is fleshed out — 1 week before ship.)

---

## Day-by-day plan

To be filled in 1 week before ship. See `01-looper.md` for the template structure (Mon spec lock → Tue impl → Wed checkpoint → Thu ship → Fri jam).

---

## Wednesday descope option

Drop multi-chord input — just let user dial in 4 chord pads and click them in sequence (manual cycling, no auto-cycle)

---

## Acceptance criteria

- [ ] Feature works in both desktop (Tauri) and web (`app.contrapunk.com`) builds
- [ ] No regressions in existing functionality
- [ ] Demo video recorded
- [ ] Documented in 1 paragraph for the website's `/jam/2` page

---

## Demo for the jam (30-60 sec video)

Type `Am F C G`; engine cycles through; user plays melody on top in Bach Chorale mode.

Save as `cover/demos/02-chord.mp4` in website repo.

---

## Risks

To be filled in 1 week before ship.

---

## Cross-feature notes

See `README.md` "Cross-feature dependencies" section.
