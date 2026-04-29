# Wk 6 Feature — Motif transposer (loop + transpose)

**Ship by**: Thu Jun 4, 2026 EOD
**Live for jam**: Fri Jun 5
**Risk**: **HIGH ⚠**
**Pairs with**: Toby Fox themed jam — see `website/.planning/jam-2026/composers/06-*.md`

---

## Minimum shippable definition (lock Monday of the build week)

Build on Wk 1 Looper. Add a 'transpose' control that pitch-shifts the entire loop buffer up/down in semitones. Optional advanced: auto-fit loop to current chord (descopable).

---

## Files to touch

`crates/contrapunk-transport/src/lib.rs` (LoopBuffer extension), UI

(Detailed file list + diffs to be filled in when this week's brief is fleshed out — 1 week before ship.)

---

## Day-by-day plan

To be filled in 1 week before ship. See `01-looper.md` for the template structure (Mon spec lock → Tue impl → Wed checkpoint → Thu ship → Fri jam).

---

## Wednesday descope option

Ship just transpose-as-unit (manual semitone slider). DROP auto-fit-to-chord — that's the high-risk algorithm. If even unit transpose isn't ready, fallback to plugin-curation only.

---

## Acceptance criteria

- [ ] Feature works in both desktop (Tauri) and web (`app.contrapunk.com`) builds
- [ ] No regressions in existing functionality
- [ ] Demo video recorded
- [ ] Documented in 1 paragraph for the website's `/jam/6` page

---

## Demo for the jam (30-60 sec video)

Loop a 4-sec motif; click +5 semitones; same motif now in different key; improvise transition.

Save as `cover/demos/06-motif.mp4` in website repo.

---

## Risks

To be filled in 1 week before ship.

---

## Cross-feature notes

See `README.md` "Cross-feature dependencies" section.
