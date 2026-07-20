# Task for researcher

Produce the missing independent performer/interaction report for Phase 10.2 preset 02 “Modal Linework,” reference Palestrina. Read only `.planning/phases/10.2-arrangement-presets/10.2-PRESET-CATALOG.md`, role C in `.planning/phases/10.2-arrangement-presets/10.2-RESEARCH-TEMPLATE.md`, and the minimum relevant local input/transport/arrangement code needed to keep guidance operational; do NOT read the existing theory report, so this remains an independent perspective. Research how a live clean-monophonic guitarist or keyboardist can supply gestures that support a bounded late-Renaissance vocal-polyphony arrangement: source gesture, register, tempo, lengths, articulation, dynamics/velocity, density, rests, transport precision, phrase/section development, listening/responding, guitar limits, keyboard opportunities and chord-density limit, and failure gestures. Provide a concise plain-language “Play it like…” line, expanded guidance, and one explicit 30–60 second acceptance exercise with observable pass/fail evidence including note cleanup. Use authoritative performance-practice, score, institutional, or scholarly sources with stable URLs, retrieval date, claim-level confidence, and gaps; distinguish historical evidence from interaction-design inference. Avoid exact-imitation claims and copyrighted melody. Do not modify project/source files; return the complete standalone Markdown report through the configured output artifact.

---
**Output:**
Write your findings to exactly this path: /tmp/contrapunk-modal-linework-performance.md
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