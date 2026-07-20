# Task for researcher

Role B independent music-theory/temporal report for Phase 10.2 preset 12 Planed Cathedral (Debussy). Read only `.planning/phases/10.2-arrangement-presets/10.2-PRESET-CATALOG.md`, `.planning/phases/10.2-arrangement-presets/10.2-RESEARCH-TEMPLATE.md`, and relevant current harmony/scale/voice-leading/catalog code needed to map capability; do not read sibling preset research or other Planed Cathedral reports. Research authoritative score-based/scholarly sources. Cover tonal center, whole-tone/pentatonic/diatonic/chromatic interaction, chord vocabulary and planing types (exact/diatonic/mixed), spacing/inversion/doubling, melodic and voice motion, rhythm/phrase/form/register/dynamics, within-phrase and across-section evolution, and selected-period limits. Deliver testable invariants, variable parameters, misleading behaviors, temporal state sketch/triggers, at least three abstract degree/beat examples, exact mapping to current HarmonyEngine capabilities and named gaps, citations, confidence, competing interpretations, and an explicit `ready_for_implementation | needs_shared_capability | reject_as_misleading` decision. Determine whether any existing mode can honestly realize a bounded preset without a new preset-specific branch. Write `/tmp/contrapunk-planed-cathedral-theory.md`; do not modify repository files.

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