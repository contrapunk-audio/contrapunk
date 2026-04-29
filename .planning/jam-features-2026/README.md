# Contrapunk Jam Series — Feature Roadmap (May & June 2026)

The 9-week music jam runs May 1 → Jun 28. Each week ships a new feature in `app.contrapunk.com` timed to enable that week's themed composer's style.

> Marketing / event-ops / cover artwork / community lives in the **website repo** at `website/.planning/marketing/jam-2026/` (gitignored). **This directory is just the code work.**

---

## Status (last updated 2026-04-30)

| Item | State |
|---|---|
| Feature slate | ✅ locked (9 features) |
| Sequencing | ✅ locked (alternates hard / easy weeks) |
| Discipline rules | ✅ defined (see below) |
| Wk 1 Looper brief | ✅ scoped → `01-looper.md` |
| Wk 2-9 feature briefs | 🟡 stubs — fill out 1 week ahead of ship |
| **Website infra (CF Pages + CI)** | ✅ shipped 2026-04-30 — feature work is unblocked from the marketing side |
| **Wk 1 Looper merge + deploy** | ⏳ TODAY — must ship Apr 30 EOD for May 1 jam launch |

---

## Shipping calendar

| Wk | Ship by (Thu EOD) | Live for jam | Feature | Brief | Risk |
|---|---|---|---|---|---|
| 1 | Apr 30 | May 1 | **Looper / phrase recorder** | [`01-looper.md`](01-looper.md) | Med |
| 2 | May 7 | May 8 | **Chord progression sequencer** | [`02-chord-seq.md`](02-chord-seq.md) | Med |
| 3 | May 14 | May 15 | **Drone layer + Bitcrusher** | [`03-drone-bitcrusher.md`](03-drone-bitcrusher.md) | **Low** ✅ |
| 4 | May 21 | May 22 | **Reverse delay + Shimmer reverb** | [`04-reverse-shimmer.md`](04-reverse-shimmer.md) | Low-med |
| 5 | May 28 | May 29 | **Exotic scale pack** | [`05-exotic-scales.md`](05-exotic-scales.md) | **Low** ✅ |
| 6 | Jun 4 | Jun 5 | **Motif transposer (loop+transpose)** | [`06-motif-transposer.md`](06-motif-transposer.md) | **HIGH** ⚠ |
| 7 | Jun 11 | Jun 12 | **Arpeggiator + sustain/legato** | [`07-arpeggiator-sustain.md`](07-arpeggiator-sustain.md) | Med |
| 8 | Jun 18 | Jun 19 | **Distortion chain + power-chord + drop-tune** | [`08-distortion-power.md`](08-distortion-power.md) | Med-high |
| 9 | Jun 25 | Jun 26 | **Ambient pad generator** | [`09-ambient-pad.md`](09-ambient-pad.md) | Med |

**~38-43 dev days across 9 weeks** if scoped tight. Roughly 5 dev days/week dedicated to feature work, separate from jam ops + ongoing maintenance.

Sequence rationale: alternate hard and easy. Wk 3 (drone+bitcrusher) and Wk 5 (exotic scales) are intentional easy weeks that buy back time after harder weeks. The single highest-risk feature (Wk 6 Motif Transposer) is mid-series so we have momentum + the option to descope to "loop + transpose-as-unit" if the algorithm turns out hairy.

---

## Discipline rules (mandatory — these are what make 9-in-9 actually possible)

1. **Monday: lock "minimum shippable" definition.** Written in 3 sentences. No scope additions Wed-Fri.
2. **Wednesday checkpoint.** If <60% complete, descope NOW (don't wait until Friday). Update brief with new scope.
3. **Thursday EOD = hard feature freeze.** If unmerged + deployed to `app.contrapunk.com`, the week pivots to **"plugin curation only"** for that composer. The jam still happens. The feature gets re-scoped and slotted into a later week.
4. **Plugin-curation panel always prepared by Wednesday**, regardless of feature status. Free safety net.
5. **No two HIGH-risk features back-to-back.** Currently only Wk 6 is high-risk → safe.

---

## Codebase context

| Area | Where | Notes |
|---|---|---|
| Harmony engine | `crates/contrapunk-harmony/` | modes, scales, voice leading |
| MIDI I/O | `crates/contrapunk-midi/` | input parsing, output routing |
| Transport / clock | `crates/contrapunk-transport/` | recently shipped — relevant for Wk 1 Looper |
| Pattern sequencer | `src/`, `crates/contrapunk-chord/` | recently shipped — also relevant for Wk 1 |
| FX framework | `src/fx/` | has `delay.rs`, `reverb.rs`, `mod.rs` — extends easily for Wk 3, 4, 8 |
| Synth | `src/synth/` | for Wk 9 ambient pad |
| Plugin host | `src/plugin_host/` | enables CLAP — used for plugin curation every week |
| Chain / routing | `src/chain/` | for Wk 8 distortion chain |
| Audio + guitar pipeline | `crates/contrapunk-audio/src/guitar_pipeline/` | for Wk 8 drop-tune detection |

---

## Per-week brief structure

Each `0X-name.md` in this directory contains:

1. **Feature scope** — what ships, in 3 sentences (the "minimum shippable definition" locked Monday)
2. **Files to touch** — concrete paths
3. **Day-by-day plan** — Mon/Tue/Wed/Thu work breakdown
4. **Wednesday descope option** — if 60% checkpoint fails, what gets cut
5. **Acceptance criteria** — how we know it's done
6. **Demo for the jam** — the 30-second video showing the feature working
7. **Risks** — known unknowns

Open the briefs in order — each one is fully self-contained.

---

## Cross-feature dependencies (worth tracking)

- Wk 1 Looper → uses transport/clock (already shipped) + pattern seq infra (already shipped)
- Wk 2 Chord Seq → uses transport/clock + `contrapunk-chord` crate
- Wk 3 Drone Layer → uses synth.rs + transport for sustained playback
- Wk 4 Reverse Delay + Shimmer → both extend `src/fx/delay.rs` and `reverb.rs`
- Wk 6 Motif Transposer → uses Looper from Wk 1 (so Wk 1 must ship cleanly)
- Wk 7 Arpeggiator → uses transport + pattern seq
- Wk 8 Distortion Chain → uses `src/chain/` + extends `src/fx/`
- Wk 9 Ambient Pad → uses synth.rs (could leverage Wk 3 drone infrastructure)

**Hot path: Wk 1 Looper → Wk 6 Motif Transposer.** If Wk 1 looper has rough edges, Wk 6 is in trouble. Build Looper as production-grade, not as MVP.

---

## "Definition of done" per week

Before Thursday EOD ship gate:

- [ ] Feature merged to `main` in `contrapunk` repo
- [ ] Deployed to `app.contrapunk.com` (web target builds + ships clean)
- [ ] Feature accessible without UI hunting (visible in default UI on first load)
- [ ] At least one short demo video (30-60 sec, screen capture) recorded for the Friday jam announcement
- [ ] Feature documented in 1 paragraph on the website's `/jam/[week]` page
- [ ] No regressions in existing functionality (CI green, manual smoke test passes)

If any of these slip past Thursday EOD → drop to "plugin curation only" for that week, push feature to a later week's slot.

---

## Risk register (code-side)

| Risk | Mitigation |
|---|---|
| Looper (Wk 1) is rough → blocks Motif Transposer (Wk 6) | Treat Wk 1 as production-grade, not MVP |
| Distortion chain (Wk 8) eats more time than budgeted | Pre-investigate `src/chain/` infra in Wk 5-6 (the easy weeks) |
| Web build breaks on a feature → can't ship to `app.contrapunk.com` | Web-first dev for jam features (test in browser before native) |
| Motif Transposer (Wk 6) algorithm proves intractable | Wednesday descope to "loop + transpose-as-unit" (no auto-fit-to-chord) |
| Pattern seq + transport refactor mid-series breaks earlier features | Freeze transport/pattern API surface from Wk 1 onward; refactor under the hood only |
| New CLAP host bugs surface during demo weekend | Freeze CLAP host changes from Wk 0; only fixes, no new behaviors |
