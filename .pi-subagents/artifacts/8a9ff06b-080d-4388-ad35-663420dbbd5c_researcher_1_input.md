# Task for researcher

Produce the independent Role B music-theory and temporal-behavior report for Phase 10.2 preset 04 “Mensuration Web,” reference Johannes Ockeghem. Read `.planning/phases/10.2-arrangement-presets/10.2-PRESET-CATALOG.md`, Role B in `.planning/phases/10.2-arrangement-presets/10.2-RESEARCH-TEMPLATE.md`, and minimum relevant local HarmonyEngine/CanonLane/transport code; do not read sibling reports. Scope specific Ockeghem works/periods, especially the proportional/mensuration canons of *Missa prolationum*, without treating one Mass as his entire style. Analyze tonal/modal organization, consonance/dissonance, melodic and motivic behavior, voice entry/independence, mensuration/prolation and proportional duration relationships, cadence, phrase/section density, register/texture, within-phrase evolution, across-section evolution, and which traits are work-specific versus persistent. Deliver testable invariants, variable parameters, misleading behaviors, a temporal state sketch with triggers, at least three novel abstract scale-degree/beat examples, exact mapping to current Free Imitation `CanonLane` capabilities and named gaps, lifecycle/transport requirements, citations, claim-level confidence, competing interpretations, and retrieval date. Explicitly distinguish mensural reinterpretation of one notated line from naive audio time-stretch or three metronomes. Do not modify project/source files; return a complete standalone Markdown artifact only.

---
**Output:**
Write your findings to exactly this path: /tmp/contrapunk-mensuration-web-theory.md
This path is authoritative for this run.
Ignore any other output filename or output path mentioned elsewhere, including output destinations in the base agent prompt, system prompt, or task instructions.

## Acceptance Contract
Acceptance level: reviewed
Completion is not accepted from prose alone. End with a structured acceptance report.

Criteria:
- criterion-1: Implement the requested change without widening scope
- criterion-2: Return evidence sufficient for an independent acceptance review

Required evidence: changed-files, tests-added, commands-run, validation-output, residual-risks, no-staged-files

Review gate: optional by reviewer.

Finish with a fenced JSON block tagged `acceptance-report` in this shape:
Use empty arrays when no items apply; array fields contain strings unless object entries are shown.
```acceptance-report
{
  "criteriaSatisfied": [
    {
      "id": "criterion-1",
      "status": "satisfied",
      "evidence": "specific proof"
    }
  ],
  "changedFiles": [
    "src/file.ts"
  ],
  "testsAddedOrUpdated": [
    "test/file.test.ts"
  ],
  "commandsRun": [
    {
      "command": "command",
      "result": "passed",
      "summary": "short result"
    }
  ],
  "validationOutput": [
    "validation output or concise summary"
  ],
  "residualRisks": [
    "none"
  ],
  "noStagedFiles": true,
  "diffSummary": "short description of the diff",
  "reviewFindings": [
    "blocker: file.ts:12 - issue found, or no blockers"
  ],
  "manualNotes": "anything else the parent should know"
}
```