# Task for reviewer

[Read from: /Users/vibhavbobade/go/src/github.com/contrapunk-audio/contrapunk/plan.md, /Users/vibhavbobade/go/src/github.com/contrapunk-audio/contrapunk/progress.md]

Read-only guitar-control contract audit. Do not edit, stage, run holdout, or inspect `.pi-subagents`. Trace every user-facing control/status in `ui/src/lib/components/GuitarInputPanel.svelte`, `SignalGraphs.svelte`, `InputPanel.svelte`, `ui/src/lib/stores/guitar.svelte.ts`, and all adapter implementations through Tauri/WASM/plugin into the shipping `GuitarInputConfig` and runtime worker. Compare displayed/default/reset values, units, ranges, live-vs-restart behavior, and whether knobs actually control current Phase 10.1 implementation. Identify stale, misleading, no-op, or missing controls and exact smallest fixes with file/line evidence. Also inspect calibration/profile reset semantics. Return prioritized blockers, should-fix items, and verified-aligned controls.

## Acceptance Contract
Acceptance level: reviewed
Completion is not accepted from prose alone. End with a structured acceptance report.

Criteria:
- criterion-1: Implement the requested change without widening scope
- criterion-2: Return evidence sufficient for an independent acceptance review

Required evidence: changed-files, tests-added, commands-run, validation-output, residual-risks, no-staged-files

Review gate: required by reviewer.

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