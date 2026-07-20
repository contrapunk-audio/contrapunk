# Task for researcher

Role C independent performer/HCI report for Phase 10.2 preset 12 Planed Cathedral (Debussy). Read only `.planning/phases/10.2-arrangement-presets/10.2-PRESET-CATALOG.md`, `.planning/phases/10.2-arrangement-presets/10.2-RESEARCH-TEMPLATE.md`, and relevant public adapter/config surfaces if needed; do not read sibling preset research or other Planed Cathedral reports. Use reputable performance, score, interview, institutional, or scholarly evidence. Define the best live clean-monophonic guitar and keyboard source gestures, register, tempo, note lengths, articulation/dynamics, density/silence, transport dependency, phrase/section development, listening/responding instructions, failure gestures, guitar limitations, keyboard opportunities, and a plain-language Play it like prompt. Include a 30–60 second deterministic acceptance exercise with abstract degrees/intervals, expected output relationships/evolution, normal release, Panic/Stop cleanup, and environment-preservation checks. Distinguish what player input supplies from what current harmony can generate; surface any honesty blocker. Include citations, retrieval date, confidence and gaps. Write `/tmp/contrapunk-planed-cathedral-performance.md`; do not modify repository files.

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