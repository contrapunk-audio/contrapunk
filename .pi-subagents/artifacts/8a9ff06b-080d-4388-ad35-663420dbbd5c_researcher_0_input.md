# Task for researcher

Produce the independent Role A artist/style-history and primary-source report for Phase 10.2 preset 04 “Mensuration Web,” reference Johannes Ockeghem. Read `.planning/phases/10.2-arrangement-presets/10.2-PRESET-CATALOG.md` and Role A in `.planning/phases/10.2-arrangement-presets/10.2-RESEARCH-TEMPLATE.md`; do not read sibling reports. Scope named works, sources, and periods—especially but not automatically limited to *Missa prolationum*—and distinguish composition chronology from manuscript/publication chronology. Cover historical/aesthetic context, career/style evolution, persistent versus work-specific traits, relation to peers and shared late-medieval/Renaissance mensural/canonic vocabulary, unusually characteristic choices, myths/caricatures to reject, primary evidence, stable citations with folio/section/measure references where responsible, retrieval date, claim-level confidence, disagreements, and gaps. Judge exactly what “one motif at simultaneous proportional speeds” can honestly reference and where that product phrase is misleading (mensuration, prolation, notation, canon, voice entry, and performed tempo are not interchangeable). Use abstract descriptions only. Do not modify project/source files; return a complete standalone Markdown artifact only.

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