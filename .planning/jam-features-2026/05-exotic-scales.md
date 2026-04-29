# Wk 5 Feature — Exotic scale pack

**Ship by**: Thu May 28, 2026 EOD
**Live for jam**: Fri May 29
**Risk**: Low ✅
**Pairs with**: Mitsuda themed jam — see `website/.planning/jam-2026/composers/05-*.md`

---

## Minimum shippable definition (lock Monday of the build week)

Add 8 new scales to the existing scale data: Hijaz, Hungarian Minor, Phrygian Dominant, Romanian Minor, Persian, Bhairav, Yo, In. Verify voice-leading rules still produce sane harmonies in each.

---

## Files to touch

`crates/contrapunk-harmony/src/scales/` (or wherever scales live)

(Detailed file list + diffs to be filled in when this week's brief is fleshed out — 1 week before ship.)

---

## Day-by-day plan

To be filled in 1 week before ship. See `01-looper.md` for the template structure (Mon spec lock → Tue impl → Wed checkpoint → Thu ship → Fri jam).

---

## Wednesday descope option

Ship 4 scales instead of 8; pick the most stylistically distinct (Hijaz, Hungarian Minor, Phrygian Dominant, Yo)

---

## Acceptance criteria

- [ ] Feature works in both desktop (Tauri) and web (`app.contrapunk.com`) builds
- [ ] No regressions in existing functionality
- [ ] Demo video recorded
- [ ] Documented in 1 paragraph for the website's `/jam/5` page

---

## Demo for the jam (30-60 sec video)

Mode = Free, scale = Hijaz; play modal melody over a drone — instant Mediterranean.

Save as `cover/demos/05-exotic.mp4` in website repo.

---

## Risks

To be filled in 1 week before ship.

---

## Cross-feature notes

See `README.md` "Cross-feature dependencies" section.
