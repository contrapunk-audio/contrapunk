# Task for researcher

Produce the independent Role A history/primary-source report for Phase 10.2 preset 07 “Stretto Engine,” reference J. S. Bach fugues. Read the accepted preset catalog and Role A research template; do not read sibling reports. Scope specific fugues/sources and periods rather than “Bach fugue” generically—choose a defensible corpus spanning relevant career/work contexts only where source dating supports it (for example selected Well-Tempered Clavier fugues and/or Art of Fugue). Define stretto historically, distinguish autograph/early copies/editions and composition/publication chronology, trace relevant evolution without forcing a simple ladder, separate persistent contrapuntal traits from work-specific stretto procedures, distinguish shared fugue vocabulary from characteristic choices, and identify myths/caricatures. Judge exactly what “tonic, fifth, and octave answers enter progressively closer” can honestly reference and what it must not claim. Prefer primary scores/manuscripts, Bach Digital, Neue Bach-Ausgabe or institutional/peer-reviewed score analysis; include stable citations, work/measure/entry references where responsible, retrieval date, claim-level confidence, disagreements, and gaps. Abstract descriptions only; no copied themes. Do not modify project/source files; return a complete standalone Markdown artifact.

---
**Output:**
Write your findings to exactly this path: /tmp/contrapunk-stretto-engine-history.md
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