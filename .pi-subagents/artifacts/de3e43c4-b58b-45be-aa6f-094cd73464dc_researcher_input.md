# Task for researcher

Retry 2/3: produce the missing independent Role B theory/temporal report for preset 07 “Stretto Engine,” selected J. S. Bach fugues. Exact inputs: `.planning/phases/10.2-arrangement-presets/10.2-PRESET-CATALOG.md` and `.planning/phases/10.2-arrangement-presets/10.2-RESEARCH-TEMPLATE.md`; also read minimum relevant Strict Canon/CanonLane/HarmonyEngine/transport code. Do not read sibling reports. Scope named works/periods; distinguish stretto from generic echo/canon. Analyze subject/answer interval handling, tonal versus real answer, motif identity, countersubject/invertible counterpoint, entry order and reduced entry distance, overlap, harmony/dissonance, cadence, density, register, within-phrase/across-section evolution, and career/work limits. Deliver testable invariants, variable parameters, rejected shortcuts, temporal state sketch/triggers, at least three novel scale-degree/beat examples, exact current Strict Canon mapping and gaps, transport/Hold/lifecycle requirements, citations, retrieval date, claim confidence, disagreements, and gaps. Decide whether fixed full-subject delays are only a bounded overlapping-answer approximation. Do not modify project/source files; write `/tmp/contrapunk-stretto-engine-theory.md` and finish, skipping blocked sources and recording gaps.

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