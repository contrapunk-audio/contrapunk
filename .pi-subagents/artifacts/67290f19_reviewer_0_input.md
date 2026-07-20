# Task for reviewer

[Read from: /Users/vibhavbobade/go/src/github.com/contrapunk-audio/contrapunk/plan.md, /Users/vibhavbobade/go/src/github.com/contrapunk-audio/contrapunk/progress.md]

Audit every commit on origin/main from the last 24 hours for any plausible regression causing a held/sustained guitar note to produce prematurely ending or repeatedly retriggered MIDI. Use git log/diffs. Focus guitar detector, Tauri guitar bridge, router, harmony parameter panic/reconfiguration, Companion enable/disable, UI effects and preset application. Do not edit. Return: exact culprit commit(s) if any, causal path with file/line evidence, and commits ruled out. Current working tree is dirty; inspect committed origin/main only and ignore uncommitted changes.

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