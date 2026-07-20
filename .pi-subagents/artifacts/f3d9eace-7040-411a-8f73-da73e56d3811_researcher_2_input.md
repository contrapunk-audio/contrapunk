# Task for researcher

Produce the independent Role C performer/interaction report for Phase 10.2 preset 07 “Stretto Engine,” reference selected J. S. Bach fugues. Read the accepted preset catalog, Role C research template, and minimum relevant Strict Canon/transport/input lifecycle code; do not read sibling reports. Research how a live clean-monophonic guitarist or keyboardist should provide a two-to-four-note subject with repeatable rhythm so delayed tonic/fifth/octave answers remain identifiable as entry spacing tightens. Cover source gesture, register, tempo, note lengths, articulation, velocity/dynamics, density, silence, transport and rhythmic precision, phrase/section development, listening/responding cues, guitar constraints, keyboard opportunities/chord limits, and failure gestures causing mud/note storms or false fugue claims. Provide concise “Play it like…” copy, expanded guidance, and a 30–60 second observable acceptance exercise with normal NoteOn/NoteOff parity and hard cleanup evidence. Prefer primary/institutional/scholarly/performance-practice sources; distinguish historical evidence from HCI inference; include stable citations, retrieval date, confidence, gaps, and no copied melody. Do not modify project/source files; return a complete standalone Markdown artifact.

---
**Output:**
Write your findings to exactly this path: /tmp/contrapunk-stretto-engine-performance.md
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