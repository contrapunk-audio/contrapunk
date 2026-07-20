# Task for researcher

Produce the independent Role B theory/temporal report for Phase 10.2 preset 07 “Stretto Engine,” reference selected J. S. Bach fugues. Read the accepted preset catalog, Role B research template, and minimum relevant Strict Canon/CanonLane/HarmonyEngine/transport code; do not read sibling reports. Scope named works/periods and distinguish stretto as a work-specific contrapuntal process from generic echo/canon. Analyze subject/answer interval handling, tonal versus real answer, melodic/motivic identity, invertible counterpoint/countersubject, entry order and progressively reduced entry distance, overlap, harmony/dissonance, cadence, density, register, within-phrase and across-section evolution, and limits of career-style claims. Deliver testable invariants, variable parameters, rejected shortcuts, a temporal state sketch/triggers, at least three novel scale-degree/beat examples, exact current Strict Canon mapping and named gaps, transport/Hold/lifecycle requirements, citations, retrieval date, claim-level confidence, disagreements, and gaps. Determine whether the baseline can honestly use fixed full-subject delays or only a bounded overlapping-answer approximation. Do not modify project/source files; return a complete standalone Markdown artifact.

---
**Output:**
Write your findings to exactly this path: /tmp/contrapunk-stretto-engine-theory.md
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