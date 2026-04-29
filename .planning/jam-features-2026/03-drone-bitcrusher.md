# Wk 3 Feature — Drone layer + Bitcrusher

**Ship by**: Thu May 14, 2026 EOD
**Live for jam**: Fri May 15
**Risk**: Low ✅
**Pairs with**: Disasterpeace themed jam — see `website/.planning/jam-2026/composers/03-*.md`

---

## Minimum shippable definition (lock Monday of the build week)

Drone layer = sustained note generator (pick a pitch + voice/timbre, holds indefinitely). Bitcrusher = new FX in `src/fx/bitcrusher.rs` exposed in chain.

---

## Files to touch

`src/synth/`, `src/fx/bitcrusher.rs` (new), `src/chain/`, UI panels

(Detailed file list + diffs to be filled in when this week's brief is fleshed out — 1 week before ship.)

---

## Day-by-day plan

To be filled in 1 week before ship. See `01-looper.md` for the template structure (Mon spec lock → Tue impl → Wed checkpoint → Thu ship → Fri jam).

---

## Wednesday descope option

Ship just the drone layer; bitcrusher pushes to Wk 4 alongside other FX

---

## Acceptance criteria

- [ ] Feature works in both desktop (Tauri) and web (`app.contrapunk.com`) builds
- [ ] No regressions in existing functionality
- [ ] Demo video recorded
- [ ] Documented in 1 paragraph for the website's `/jam/3` page

---

## Demo for the jam (30-60 sec video)

Sustain a low D drone; improvise Lydian melody on top; bitcrush the output.

Save as `cover/demos/03-drone.mp4` in website repo.

---

## Risks

To be filled in 1 week before ship.

---

## Cross-feature notes

See `README.md` "Cross-feature dependencies" section.
