# Task for reviewer

[Read from: /Users/vibhavbobade/go/src/github.com/contrapunk-audio/contrapunk/plan.md, /Users/vibhavbobade/go/src/github.com/contrapunk-audio/contrapunk/progress.md]

Read-only review. Inspect the current working tree in /Users/vibhavbobade/go/src/github.com/contrapunk-audio/contrapunk and identify the exact minimal file/hunk set that forms the post-Phase-10.1 guitar reliability/UI truthfulness slice described in .planning/HANDOFF.json. Separate it from older Ensemble/plugin/Golem/Elixir/generated work. Report commit-ready files, mixed files requiring hunk staging, validation needed, and any correctness blockers. Do not modify files or stage anything.

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