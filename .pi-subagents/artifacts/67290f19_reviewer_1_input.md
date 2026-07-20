# Task for reviewer

[Read from: /Users/vibhavbobade/go/src/github.com/contrapunk-audio/contrapunk/plan.md, /Users/vibhavbobade/go/src/github.com/contrapunk-audio/contrapunk/progress.md]

Trace the CURRENT origin/main guitar-audio-to-MIDI sustain lifecycle end to end (detector state machine -> bridge -> router -> synth/MIDI). Identify why a physically sustained guitar note could get early NoteOff or repeated NoteOn/Off. Inspect existing corpus/regression tests and recommend the smallest runnable check that distinguishes detector degradation from downstream panic/reset. Do not edit. Ignore uncommitted work by reading origin/main versions where relevant. Report exact paths/lines and commands.

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