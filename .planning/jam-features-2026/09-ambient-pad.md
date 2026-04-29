# Wk 9 Feature — Ambient pad generator

**Ship by**: Thu Jun 25, 2026 EOD
**Live for jam**: Fri Jun 26
**Risk**: Medium
**Pairs with**: Lena Raine themed jam — see `website/.planning/jam-2026/composers/09-*.md`

---

## Minimum shippable definition (lock Monday of the build week)

Pad generator = slowly-evolving polyphonic synth voice that morphs between 2-3 pad presets over 8-16 bars. Builds on `src/synth/` and Wk 3 drone layer infra.

---

## Files to touch

`src/synth/pad.rs` (new), reuse drone trigger model

(Detailed file list + diffs to be filled in when this week's brief is fleshed out — 1 week before ship.)

---

## Day-by-day plan

To be filled in 1 week before ship. See `01-looper.md` for the template structure (Mon spec lock → Tue impl → Wed checkpoint → Thu ship → Fri jam).

---

## Wednesday descope option

Ship one fixed pad preset that loops; no morphing between multiple presets

---

## Acceptance criteria

- [ ] Feature works in both desktop (Tauri) and web (`app.contrapunk.com`) builds
- [ ] No regressions in existing functionality
- [ ] Demo video recorded
- [ ] Documented in 1 paragraph for the website's `/jam/9` page

---

## Demo for the jam (30-60 sec video)

Pad slowly evolving underneath; soft major-7th melody on top in Bach Chorale; series finale vibes.

Save as `cover/demos/09-ambient.mp4` in website repo.

---

## Risks

To be filled in 1 week before ship.

---

## Cross-feature notes

See `README.md` "Cross-feature dependencies" section.
