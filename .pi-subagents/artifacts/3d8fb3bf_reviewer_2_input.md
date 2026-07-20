# Task for reviewer

[Read from: /Users/vibhavbobade/go/src/github.com/contrapunk-audio/contrapunk/plan.md, /Users/vibhavbobade/go/src/github.com/contrapunk-audio/contrapunk/progress.md]

Read-only architecture/design review for the requested separate native functional prototype. Use the approved rules: monochrome idle UI, color only encodes sound, controlled density, all app features, piano roll retained, continuous guitar expression trace vs emitted MIDI. Inspect current routes/components and recommend the smallest isolation boundary that runs as a native Tauri entry while reusing real adapters/stores/backend and not changing production behavior. Compare route/config/query alternatives, recommend one, propose top-level navigation, accessibility/performance constraints, and smallest testable vertical slice. Do not modify files.

## Acceptance Contract
Acceptance level: reviewed
Completion is not accepted from prose alone. End with a structured acceptance report.

Criteria:
- criterion-1: Return concrete findings with file paths and severity when applicable

Required evidence: changed-files, tests-added, commands-run, validation-output, residual-risks, no-staged-files

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