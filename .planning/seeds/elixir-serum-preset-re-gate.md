---
title: Elixir — Serum preset reverse-engineering gate
trigger_condition: After Elixir's wavetable engine and modulation matrix are stable (post-MVP, likely v0.3+), AND user is ready to invest in Serum preset import as a feature.
planted_date: 2026-04-28
---

# Serum preset RE — gated future work

The user has expressed intent to reverse-engineer Serum's `.SerumPreset` format for full bidirectional preset compatibility. This is **gated work** — do not start until the trigger conditions are met AND the gate below is fully passed.

## Why gated

- **Format is closed and unreversed.** Per `.planning/research/elixir/serum-features.md`, `.SerumPreset` = XferJson header + zlib-compressed undocumented binary blob. No public spec exists.
- **Legal risk varies by jurisdiction.** Xfer's EULA almost certainly prohibits reverse engineering. Some jurisdictions (EU under the Software Directive, US under DMCA §1201(f) for interoperability) carve out RE-for-interoperability with limits; others don't. Litigation risk is non-zero even where legally defensible.
- **Effort is significant.** Solo, undocumented binary format, brittle long-term as Serum updates. Easy to spend months for marginal user value (most users care more about wavetable compat — already covered).
- **Premature work.** Until the engine itself can faithfully *reproduce* what a Serum preset describes (all 5 engines, full mod matrix, 3-bus FX graph), an importer has nothing to import *into*.

## Trigger conditions (all must be met)

- [ ] Elixir wavetable engine is stable and producing audio comparable to Serum.
- [ ] Modulation matrix supports the full Serum feature surface (10 LFOs, 4 envs, 8 macros, mod slot remap curves).
- [ ] FX graph supports the 3-bus parallel topology with splitter modules.
- [ ] User explicitly decides preset import is the next priority over other backlog items.

## Gate (must complete BEFORE any RE work begins)

1. **Clean-room methodology document.** Specify how RE will be performed:
   - No disassembly of Serum binaries.
   - No use of debuggers attached to running Serum.
   - File-format-only analysis: hex dumps, statistical analysis, comparison of preset files saved with controlled parameter changes.
   - One person observes Serum behavior and writes specs in English; a separate person (or separate work session, with a deliberate gap) implements against those specs only. Document this separation.
2. **Jurisdictional analysis.** Document the user's jurisdiction (USA / EU / other) and the applicable RE-for-interop carve-outs. Capture the specific legal basis being relied on.
3. **EULA review.** Read Serum's current EULA. Document the specific clauses relevant to RE and the user's interpretation of them.
4. **Legal review (recommended).** Have an IP attorney review the methodology document, jurisdictional analysis, and EULA review before any RE begins. Document their feedback in writing.
5. **Risk acceptance.** User explicitly accepts residual risk in writing (commit to this seed file or a successor doc) before work starts.

## Implementation guardrails (when work begins)

- All RE work happens in a **separate, clearly-marked branch and crate** (e.g. `crates/elixir-serum-import`) so the import code can be removed/disabled if legal posture changes.
- Importer is **read-only** — Elixir does not write `.SerumPreset` files (that would be authoring proprietary format, not interop).
- Importer is **lossy and warns** — clearly tell users when params can't be mapped. Don't pretend to fully reproduce a Serum preset; produce an Elixir preset that approximates one.
- Distribution decision: shipping importer in default builds vs. as opt-in feature flag is its own gate. Default to feature-flagged.

## Alternative if gate cannot be passed

Stay on wavetable-compat-only path. Users can re-create presets manually using Elixir's native format. The "open & inspectable" ambition arguably wins more hearts than risky preset import anyway.

## Pointers

- Format research: `.planning/research/elixir/serum-features.md` (preset format section)
- Wavetable format (already permitted, not gated): `.planning/research/elixir/serum-features.md` (wavetable section)
- Original decision context: `.planning/notes/elixir-design-decisions.md` (decision #5)
