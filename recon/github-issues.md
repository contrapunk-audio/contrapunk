# GitHub Issues Recon: contrapunk-audio/contrapunk

Date: 2026-06-29  
Repo: https://github.com/contrapunk-audio/contrapunk

## Summary

Using `/tmp/contrapunk_issues.json` as the issue snapshot, the repo has **63 issues total: 41 open, 22 closed**. The open backlog has shifted from completed v1.1 website/demo readiness toward larger performance-companion work: lanes, sampler, DAW sync, audio intelligence, guitar-input rewrite, and deeper theory support.

## Counts

| State | Count |
|---|---:|
| Open issues | 41 |
| Closed issues | 22 |
| Total issues | 63 |

Notes:
- The GitHub `issues` API is PR-inclusive; the open API endpoint showed **49 open issue-like items**, including 8 open PRs (`#96`, `#107`, `#109`, `#110`, `#123`, `#125`, `#126`, `#127`). Pure issue count remains **41 open**.
- The v1.1 website-launch readiness milestone entries in the snapshot are all closed: `#36`, `#37`, `#38`, `#40`, `#41`, `#43`, `#44`, `#46`, `#49`, `#51`, `#53`, `#55`, `#57`.
- Label coverage is incomplete: **25/63 issues are unlabeled**, especially newer May roadmap/features (`#97`-`#121`), so the taxonomy below is manual.

## Issue taxonomy

Primary bucket; each issue counted once.

| Bucket | Open | Closed | Total | Issues / notes |
|---|---:|---:|---:|---|
| Harmony engine, theory, analysis | 6 | 6 | 12 | Open: `#4`, `#12`, `#81`, `#100`, `#112`, `#115`. Closed: `#1`, `#2`, `#3`, `#33`, `#113`, `#114`. Core themes: auto-key, historical rule sets, analysis overlays, counterpoint/canon. |
| UI, FTUX, website/demo surface | 4 | 14 | 18 | Open: `#42`, `#65`, `#66`, `#116`. Closed: website-launch batch incl. piano/fretboard, Web MIDI UX, demo mode, typography, WASM transport, output routing. |
| Guitar/audio-to-MIDI, DSP/ML input | 7 | 0 | 7 | `#6`, `#8`, `#27`, `#28`, `#29`, `#79`, `#82`. Heavy emphasis on latency, real guitar fixtures, polyphonic detection, and rewrite with native/WASM lockstep. |
| Companion lanes, sampler, FX, audio intelligence | 11 | 1 | 12 | Open: `#97`, `#101`, `#102`, `#104`, `#105`, `#106`, `#117`, `#118`, `#119`, `#120`, `#121`. Closed: `#103`. This is the biggest active product direction. |
| DAW/plugin/platform/cloud integrations | 8 | 0 | 8 | `#9`, `#10`, `#11`, `#15`, `#30`, `#31`, `#98`, `#99`. DAW/plugin, openDAW, SonoBus, TouchDesigner, Windows, Ableton Link/IAC. |
| Docs / demos / marketing proof | 2 | 0 | 2 | `#5`, `#13`. Audio samples and walkthrough remain open. |
| Runtime reliability, bug triage, testability | 3 | 1 | 4 | Open: `#14`, `#90`, `#91`. Closed: `#7`. Router state, stale note recovery, MIDI-out report, DMG issue. |

## Recurring themes

1. **“Make sound immediately” onboarding is still a live pain.** `#116` reports a real Discord user hitting silence on first play because voice routing defaults are invisible; recommended fix is default all voices to internal synth plus show output target inline. `#42` autoplay was explicitly dropped from v1.1 as non-launch-critical, but it remains a related “hear the engine without setup” path. Sources: [#116](https://github.com/contrapunk-audio/contrapunk/issues/116), [#42 comment](https://github.com/contrapunk-audio/contrapunk/issues/42#issuecomment-4318563639)
2. **v1.1 website/demo work appears largely done.** Closed milestone issues cover piano/fretboard interactions, Web MIDI permission UX, demo mode, typography cleanup, UI perf, WASM transport/metronome, and output routing. Source milestone examples: [#36](https://github.com/contrapunk-audio/contrapunk/issues/36), [#55](https://github.com/contrapunk-audio/contrapunk/issues/55), [#57](https://github.com/contrapunk-audio/contrapunk/issues/57)
3. **The product is expanding from harmony generator to performance companion.** `#117` defines Drum/Drone/Looper/Arpeggiator lanes; `#121` expands that into SampleLane, DrumLane, UserConductorSignal, AudioIntelligence, smart slicing, and SidechainLane. Sources: [#117](https://github.com/contrapunk-audio/contrapunk/issues/117), [#117 DrumLane design comment](https://github.com/contrapunk-audio/contrapunk/issues/117#issuecomment-4429105217), [#121](https://github.com/contrapunk-audio/contrapunk/issues/121)
4. **DAW co-existence is a repeated blocker/enabler.** Ableton Link, bidirectional IAC, BlackHole/CoreAudio capture, sidechain triggers, and eventual AUv3/plugin paths recur across `#98`, `#99`, `#102`, `#119`, `#120`. The `#98` comment clarifies Ableton can get OSC control via AbletonOSC, while Logic is limited to IAC/Link/CoreAudio Tap unless Contrapunk ships as AUv3. Sources: [#98](https://github.com/contrapunk-audio/contrapunk/issues/98), [#98 comment](https://github.com/contrapunk-audio/contrapunk/issues/98#issuecomment-4429111076), [#99](https://github.com/contrapunk-audio/contrapunk/issues/99)
5. **Audio intelligence/ML ambition is high, but dependency risk is visible.** ListenLane, TimbreIntelligence, DDSP, Demucs, Basic Pitch, beat-this, drumsep, CLAP/Essentia, and GrooVAE appear across `#102`, `#104`, `#118`, `#121`; blockers include model size, WASM path, and licenses. Sources: [#102](https://github.com/contrapunk-audio/contrapunk/issues/102), [#102 comment](https://github.com/contrapunk-audio/contrapunk/issues/102#issuecomment-4429107517), [#118](https://github.com/contrapunk-audio/contrapunk/issues/118), [#121](https://github.com/contrapunk-audio/contrapunk/issues/121)
6. **Guitar input has enough drift that rewrite beat patching.** `#79` identifies the immediate pitch-bend wobble/debug-window work; `#82` explicitly supersedes it with a stage-by-stage, A/B-testable rewrite and fixture corpus. Sources: [#79](https://github.com/contrapunk-audio/contrapunk/issues/79), [#82](https://github.com/contrapunk-audio/contrapunk/issues/82)
7. **Theory roadmap is becoming test-first and historical.** `#115` lays out Renaissance → Baroque → Classical → Romantic → 20th-century milestones with testable rules; `#112` asks to expose interval/species/function analysis in real time. Sources: [#115](https://github.com/contrapunk-audio/contrapunk/issues/115), [#112](https://github.com/contrapunk-audio/contrapunk/issues/112)
8. **Some open issues are likely stale or superseded.** `#4` is implemented but awaiting user testing; `#14` has a user comment saying latest local build worked; `#30` has green Windows CI artifacts but awaits tester handoff; `#79` is superseded by `#82`; `#106` targeted a May 14 jam and is still open. Sources: [#4 comment](https://github.com/contrapunk-audio/contrapunk/issues/4#issuecomment-4329338455), [#14 comment](https://github.com/contrapunk-audio/contrapunk/issues/14#issuecomment-4269371805), [#30 comment](https://github.com/contrapunk-audio/contrapunk/issues/30#issuecomment-4244167291), [#82](https://github.com/contrapunk-audio/contrapunk/issues/82), [#106](https://github.com/contrapunk-audio/contrapunk/issues/106)

## Likely near-term roadmap implied by issues

Inferred, not official:

1. **Triage/close stale open items first.** Close or re-scope `#4`, `#14`, `#30`, `#42`, `#79`, `#106` where comments/supersession/deadlines indicate stale state.
2. **Fix FTUX sound path.** `#116` is the clearest active user pain: fresh install should route voices to internal synth and show output targets without opening Voice Routing.
3. **Ship a minimal Companion lane slice before the mega-roadmap.** `#117` is smaller and concrete: DrumLane, DroneLane, LooperLane, ArpeggiatorLane using the existing Lane trait. `#121` should probably decompose after that.
4. **Stabilize guitar input with fixtures.** `#82` plus `#27`/`#6` imply a near-term quality program: fixture corpus, A/B stages, native/WASM parity, latency docs.
5. **Build DAW sync/capture foundation.** `#98` and `#99` unblock tempo, bass suppression, sidechain, ListenLane, and smart sequencing. `#119` reduces BlackHole setup friction for macOS.
6. **Then layer ListenLane / sampler / sidechain / intelligence.** `#97`, `#102`, `#118`, `#120`, `#121` are connected; sampler and capture/runtime ML foundations come before smarter UI.
7. **Continue theory depth as rule-set work, not UI-only work.** `#115`, `#81`, `#112` suggest R1/Renaissance correctness, mode+tonic auto-key, and visible analysis are the next theory-facing threads.

## Source URLs

Kept:
- Local issue snapshot: `/tmp/contrapunk_issues.json` — pure issue list/counts used for open/closed totals.
- GitHub repo issues: https://github.com/contrapunk-audio/contrapunk/issues
- GitHub API open list: https://api.github.com/repos/contrapunk-audio/contrapunk/issues?state=open&per_page=100 — used to reconcile PR-inclusive count.
- Key issues/comments: [#116](https://github.com/contrapunk-audio/contrapunk/issues/116), [#117](https://github.com/contrapunk-audio/contrapunk/issues/117), [#117 comment](https://github.com/contrapunk-audio/contrapunk/issues/117#issuecomment-4429105217), [#121](https://github.com/contrapunk-audio/contrapunk/issues/121), [#82](https://github.com/contrapunk-audio/contrapunk/issues/82), [#98](https://github.com/contrapunk-audio/contrapunk/issues/98), [#98 comment](https://github.com/contrapunk-audio/contrapunk/issues/98#issuecomment-4429111076), [#102](https://github.com/contrapunk-audio/contrapunk/issues/102), [#102 comment](https://github.com/contrapunk-audio/contrapunk/issues/102#issuecomment-4429107517), [#115](https://github.com/contrapunk-audio/contrapunk/issues/115), [#112](https://github.com/contrapunk-audio/contrapunk/issues/112), [#30 comment](https://github.com/contrapunk-audio/contrapunk/issues/30#issuecomment-4244167291)

Dropped / caveats:
- Logged-out GitHub issues HTML fetch did not expose the full issue list reliably.
- Search-result repo card showing 49 open items was treated as PR-inclusive/stale for pure issue counts.
- Labels alone were not used for taxonomy because many recent issues are unlabeled.

## Gaps

- No project board or milestone ordering beyond the closed v1.1 milestone was visible, so roadmap ordering is inferred from issue text/comments.
- Assignees and milestones are mostly empty; ownership is unclear.
- I did not verify commits/PR closures beyond issue state/comments; stale-close recommendations need maintainer confirmation.
