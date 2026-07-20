# Task for researcher

Retry 2/3: produce missing Role B theory/temporal report for preset 08 Suspension Garland. Exact inputs: `.planning/phases/10.2-arrangement-presets/10.2-PRESET-CATALOG.md` and `.planning/phases/10.2-arrangement-presets/10.2-RESEARCH-TEMPLATE.md`; read minimum CounterpointState Species4, CounterpointLane, HarmonyEngine beat phase, transport, lifecycle code; do not read siblings. Separate Fux fourth-species pedagogy from Palestrina corpus practice and state intersection. Analyze preparation, tie/hold, accented dissonance classes, metric placement, downward-step resolution/perfection, voice motion, cadence, density/register and phrase evolution. Deliver invariants, variables, rejected shortcuts, FSM/triggers, three abstract examples, exact current Species IV mapping/gaps, transport/Hold/lifecycle, citations, date, confidence/disagreements/gaps. Decide whether current Species4 needs a correctness fix before activation. Do not modify project/source files; write `/tmp/contrapunk-suspension-garland-theory.md`, skip blocked sources, record gaps, finish.

---
**Output:**
Write your findings to exactly this path: /tmp/contrapunk-suspension-garland-theory.md
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