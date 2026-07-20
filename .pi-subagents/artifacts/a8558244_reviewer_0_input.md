# Task for reviewer

Review ONLY the current uncommitted Cloister Organum / ExplicitIntervals implementation diff in these paths: crates/contrapunk-harmony/src/{config.rs,engine.rs,lib.rs}, crates/contrapunk-companion/src/canon_lane.rs, src-tauri/src/{commands/harmony.rs,main.rs}, wasm/src/{enum_strings.rs,lib.rs}, ui/src/lib/{adapter,arrangement,components,prototype,stores} files touching ExplicitIntervals/interval map, tracked ui/src/lib/wasm-pkg generated artifacts, and research synthesis. Ignore all unrelated dirty work. Check correctness, note-on/off ownership and reconfiguration cleanup, octave/routing behavior, Tauri/WASM API symmetry, unsupported plugin capability gating, Apply/manual edit/Save As round-trip, preset requirements, and tests. Do not edit files. Report only actionable findings ranked by severity with exact paths/lines; if none, say no actionable findings and note residual risks.

## Acceptance Contract
Acceptance level: attested
Completion is not accepted from prose alone. End with a structured acceptance report.

Criteria:
- criterion-1: Return concrete findings with file paths and severity when applicable

Required evidence: review-findings, residual-risks

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