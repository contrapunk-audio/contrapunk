# Task for researcher

Phase 10.2 preset 14 Color-Mode Windows — independent PERFORMER/INTERACTION report. Read `.planning/phases/10.2-arrangement-presets/10.2-RESEARCH-TEMPLATE.md`, the preset-14 catalog entry, and prior report structure under `research/12-planed-cathedral/performance.md`. Determine how a live clean-monophonic guitarist or keyboardist can evoke a narrowly scoped Messiaen-derived modes-of-limited-transposition color practice without expecting Contrapunk to compose or auto-rotate forms. Cover all performer sections: best gesture, register/tempo/note length/articulation/dynamics/density/silence, transport dependency, phrase/section development, listening/responding, guitar constraints, keyboard opportunities/chord limits, failure gestures, concise and expanded 'Play it like' guidance, and a concrete 30–60 second acceptance exercise. Ground advice in authoritative citations and distinguish selected works/periods from whole-career claims. State what current symmetric-scale mapping can and cannot honestly do. Do not design the final config or modify project/source files; returning the report through the configured output artifact is allowed.

---
Update progress at: /Users/vibhavbobade/go/src/github.com/contrapunk-audio/contrapunk/.pi-subagents/artifacts/progress/972144df-80fc-4267-add5-8df66fb3d4ff/progress.md

---
**Output:**
Write your findings to exactly this path: /Users/vibhavbobade/go/src/github.com/contrapunk-audio/contrapunk/.pi-subagents/artifacts/outputs/972144df-80fc-4267-add5-8df66fb3d4ff/.planning/phases/10.2-arrangement-presets/research/14-color-mode-windows/performance.md
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