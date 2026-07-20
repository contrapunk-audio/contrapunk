# Task for researcher

Produce independent Role C performer/HCI research for preset 08 “Suspension Garland,” references Fux/Palestrina. Exact inputs: `.planning/phases/10.2-arrangement-presets/10.2-PRESET-CATALOG.md` and `.planning/phases/10.2-arrangement-presets/10.2-RESEARCH-TEMPLATE.md`; also read minimum relevant Species IV/transport/input lifecycle code. Do not read sibling reports. Define how a live clean-monophonic guitarist or keyboardist should play long notes across beats/barlines so generated preparation–suspension–resolution remains audible and safe. Cover source gesture, register, tempo, note lengths, articulation, velocity, density, silence, transport precision, phrase/section development, listening cues, guitar constraints, keyboard chord/pedal limits, and failure gestures. Provide concise Play-it-like copy, expanded guidance, and a 30–60 second observable acceptance exercise with metric phase, normal NoteOn/NoteOff parity, Hold behavior, and hard cleanup. Prefer primary/institutional/scholarly/performance sources, distinguish history from HCI inference, include stable citations, retrieval date, confidence, gaps, and no copied melody. Do not modify project/source files; write `/tmp/contrapunk-suspension-garland-performance.md`, skip blocked sources, record gaps, and finish.

---
**Output:**
Write your findings to exactly this path: /tmp/contrapunk-suspension-garland-performance.md
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