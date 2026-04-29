# Wk 4 Feature — Reverse delay + Shimmer reverb

**Ship by**: Thu May 21, 2026 EOD
**Live for jam**: Fri May 22
**Risk**: Low-med
**Pairs with**: Yamaoka themed jam — see `website/.planning/jam-2026/composers/04-*.md`

---

## Minimum shippable definition (lock Monday of the build week)

Reverse delay = extend `src/fx/delay.rs` with a buffer-reverse mode. Shimmer = pitch-shift +12 semitones in reverb feedback path in `src/fx/reverb.rs`.

---

## Files to touch

`src/fx/delay.rs`, `src/fx/reverb.rs`, UI controls

(Detailed file list + diffs to be filled in when this week's brief is fleshed out — 1 week before ship.)

---

## Day-by-day plan

To be filled in 1 week before ship. See `01-looper.md` for the template structure (Mon spec lock → Tue impl → Wed checkpoint → Thu ship → Fri jam).

---

## Wednesday descope option

Ship just shimmer (Yamaoka also uses long-tail reverb); reverse delay pushes to Wk 5

---

## Acceptance criteria

- [ ] Feature works in both desktop (Tauri) and web (`app.contrapunk.com`) builds
- [ ] No regressions in existing functionality
- [ ] Demo video recorded
- [ ] Documented in 1 paragraph for the website's `/jam/4` page

---

## Demo for the jam (30-60 sec video)

Slow dissonant cluster played, reverse-delayed, shimmer-reverbed — Silent Hill atmosphere in 30 sec.

Save as `cover/demos/04-reverse.mp4` in website repo.

---

## Risks

To be filled in 1 week before ship.

---

## Cross-feature notes

See `README.md` "Cross-feature dependencies" section.
