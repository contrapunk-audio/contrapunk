# Task for researcher

Role A independent history/primary-source report for Phase 10.2 preset 12 Planed Cathedral (Debussy). Read only `.planning/phases/10.2-arrangement-presets/10.2-PRESET-CATALOG.md` and `.planning/phases/10.2-arrangement-presets/10.2-RESEARCH-TEMPLATE.md`; do not read any sibling preset research or other Planed Cathedral reports. Research Debussy with authoritative primary, institutional, scholarly, score-based, and composer-contemporary sources. Scope specific works and periods relevant to parallel chordal motion, whole-tone/pentatonic materials, spacing/register, orchestration/piano texture, phrase/section evolution, and silence. Separate persistent traits from period/work-specific traits and shared fin-de-siècle vocabulary; identify caricatures such as 'Debussy equals whole tone' or generic parallel chords. Include works/measure locations where verifiable, citations/URLs/publications, retrieval date, confidence, disagreements, and gaps. Translate only to research implications, not implementation. Use abstract degree/beat examples only; copy no melodies. Write the complete report to `/tmp/contrapunk-planed-cathedral-history.md`. Do not modify repository files.

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