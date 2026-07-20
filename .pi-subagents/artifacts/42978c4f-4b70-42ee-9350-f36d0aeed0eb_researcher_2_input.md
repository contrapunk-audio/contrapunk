# Task for researcher

Phase 10.2 preset 02 Modal Linework — PERFORMER AND INTERACTION GUIDANCE. Read .planning/phases/10.2-arrangement-presets/10.2-RESEARCH-TEMPLATE.md and preset 02 in 10.2-PRESET-CATALOG.md. Independently research how a live monophonic guitarist or keyboardist should seed a real-time arrangement informed by Palestrina/late-Renaissance vocal polyphony. Ground guidance in credible performance/theory sources rather than generic UX intuition. Specify best gestures, register, tempo, note lengths, articulation, dynamics, density, silence, transport needs, phrase/section development, listening/responding cues, guitar constraints, keyboard opportunities, and failure gestures that create mud or contradict the style. Explain how the input and generated lines should evolve over 30-60 seconds. Deliver a short Play-it prompt, expanded guidance, and a concrete 30-60 second acceptance exercise. Cite sources, confidence, gaps, and state what the preset cannot authentically reproduce. Do not modify project/source files; return only the report artifact.

---
**Output:**
Write your findings to exactly this path: /Users/vibhavbobade/go/src/github.com/contrapunk-audio/contrapunk/.pi-subagents/artifacts/outputs/42978c4f-4b70-42ee-9350-f36d0aeed0eb/research/02-modal-linework/performance.md
This path is authoritative for this run.
Ignore any other output filename or output path mentioned elsewhere, including output destinations in the base agent prompt, system prompt, or task instructions.

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