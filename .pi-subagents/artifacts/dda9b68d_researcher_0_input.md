# Task for researcher

Final retry 3/3 for missing Role C only. Produce independent performer/HCI research for Phase 10.2 preset 12 Planed Cathedral (Debussy). Read exact inputs `.planning/phases/10.2-arrangement-presets/10.2-PRESET-CATALOG.md` and `.planning/phases/10.2-arrangement-presets/10.2-RESEARCH-TEMPLATE.md`, plus relevant public adapter/config surfaces if needed. Do not read sibling preset research, Role A/B, or another Planed Cathedral report. Use reputable performance, score, interview, institutional, or scholarly evidence. Define the best clean-monophonic guitar and keyboard source gesture; register; slow/moderate tempo; note lengths; articulation; velocity/dynamic arc; density/silence; transport dependency; phrase/section evolution supplied by the player; listening/responding instructions; failure gestures; guitar limits; keyboard opportunities; and a plain-language Play prompt. Include a deterministic 30–60 second acceptance exercise using abstract degrees/intervals only, expected chord-plane relations and evolution, normal NoteOff completion, Panic/Stop cleanup, and preservation of tonic/BPM/meter/devices/routing/sound/mix/plugins/transport state. Explicitly distinguish static generated planing from player-shaped emergence/submergence and flag any honesty blocker. Include citations/URLs/publications, retrieval date, confidence, and gaps. Write `/tmp/contrapunk-planed-cathedral-performance.md`; do not modify repository files.

## Acceptance Contract
Acceptance level: checked
Completion is not accepted from prose alone. End with a structured acceptance report.

Criteria:
- criterion-1: Implement the requested change without widening scope

Required evidence: changed-files, tests-added, commands-run, residual-risks, no-staged-files

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