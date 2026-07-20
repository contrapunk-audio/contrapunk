# Task for researcher

Produce independent Role A history/primary-source research for preset 08 “Suspension Garland,” references Johann Joseph Fux and Giovanni Pierluigi da Palestrina. Exact inputs: `.planning/phases/10.2-arrangement-presets/10.2-PRESET-CATALOG.md` and `.planning/phases/10.2-arrangement-presets/10.2-RESEARCH-TEMPLATE.md`; do not read sibling reports. This is a multiple-reference preset: identify each reference's relevant work/period, the documented intersection, and excluded differences. Scope Fux's *Gradus ad Parnassum* (1725), its species pedagogy/reception, and selected Palestrina four-voice sacred sources/corpus evidence; distinguish later pedagogical reconstruction from sixteenth-century composition. Cover history/context, career/project evolution where defensible, persistent vs work-specific traits, peers/shared vocabulary, characteristic choices, myths/caricatures, primary evidence, stable citations with book/page/score/measure references where possible, retrieval date, claim confidence, disagreements, and gaps. Judge what “prepared suspensions resolve across strong beats” can honestly reference. Abstract descriptions only. Do not modify project/source files; write `/tmp/contrapunk-suspension-garland-history.md`, skip blocked sources, record gaps, and finish.

---
**Output:**
Write your findings to exactly this path: /tmp/contrapunk-suspension-garland-history.md
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