# Task for researcher

Retry 2/3 for the missing independent Role A history/primary-source report for preset 04 “Mensuration Web” (Johannes Ockeghem). Read only the accepted preset catalog and Role A research template; do not read existing sibling reports. Scope named works/sources/periods, especially but not exclusively *Missa prolationum*; distinguish chronology, mensuration, prolation, notation, canon, and performed tempo. Cover context, career/style evolution, persistent versus work-specific traits, peers/shared vocabulary versus characteristic choices, myths/caricatures, primary manuscript/score evidence, stable citations with folio/section references where possible, retrieval date, claim-level confidence, disagreements, and gaps. Judge exactly what “one motif at simultaneous proportional speeds” can and cannot honestly mean. Abstract description only, no copied melody. Do not modify project/source files; write the complete standalone report to the configured artifact.

---
**Output:**
Write your findings to exactly this path: /tmp/contrapunk-mensuration-web-history.md
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