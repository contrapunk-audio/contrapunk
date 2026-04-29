# Wk 8 Feature — Distortion chain + power-chord mode + drop-tune preset

**Ship by**: Thu Jun 18, 2026 EOD
**Live for jam**: Fri Jun 19
**Risk**: Med-high
**Pairs with**: Mick Gordon themed jam — see `website/.planning/jam-2026/composers/08-*.md`

---

## Minimum shippable definition (lock Monday of the build week)

Distortion = multi-stage saturation in new `src/fx/distortion.rs`. Power-chord mode = harmony engine option to drop the third (root + 5th only). Drop-tune preset = guitar input calibration preset for Drop D / Drop C / Drop A.

---

## Files to touch

`src/fx/distortion.rs` (new), `crates/contrapunk-harmony/src/voice_leading/`, `crates/contrapunk-audio/src/guitar_pipeline/`

(Detailed file list + diffs to be filled in when this week's brief is fleshed out — 1 week before ship.)

---

## Day-by-day plan

To be filled in 1 week before ship. See `01-looper.md` for the template structure (Mon spec lock → Tue impl → Wed checkpoint → Thu ship → Fri jam).

---

## Wednesday descope option

Ship distortion + power-chord; drop-tune pushes to W9 (or stays in plugin-curation)

---

## Acceptance criteria

- [ ] Feature works in both desktop (Tauri) and web (`app.contrapunk.com`) builds
- [ ] No regressions in existing functionality
- [ ] Demo video recorded
- [ ] Documented in 1 paragraph for the website's `/jam/8` page

---

## Demo for the jam (30-60 sec video)

Drop-tuned guitar input → power-chord mode → distortion chain → Phrygian metal riff in 30 sec.

Save as `cover/demos/08-distortion.mp4` in website repo.

---

## Risks

To be filled in 1 week before ship.

---

## Cross-feature notes

See `README.md` "Cross-feature dependencies" section.
