# Contrapunk organic reach + SEO content-pillar strategy

Date: 2026-06-29

## Summary

Contrapunk should win organic reach by owning a narrow, useful lane: **"play a melody, hear real counterpoint now"** for musicians who already search for counterpoint, guitar harmony, DAW routing, voice leading, and composer workflow help. Publish practical lessons with audible A/B demos and a browser-first CTA, not generic AI-music thought pieces.

Grounding: local docs position Contrapunk as a real-time counterpoint harmony generator for guitar/MIDI with explicit Palestrina/Bach/Jazz/Free voice-leading styles, browser/native surfaces, CLAP hosting, and open source (`README.md`, `docs/MARKET_ANALYSIS.md`). Current recon also says demos/walkthroughs and first-sound onboarding remain important (`recon/github-issues.md`).

## Positioning to repeat

> Contrapunk is a free, open-source real-time counterpoint instrument: plug in MIDI or guitar, choose a style, and hear independent harmony voices instead of parallel pitch-shifted copies.

Use this contrast everywhere:

- **Not just a harmonizer pedal:** those usually transpose in parallel.
- **Not just a chord generator:** those start from chosen chords/progressions.
- **Not just an academic demo:** Contrapunk is playable in a browser/native app.
- **Not a generic AI composer:** the hook is explicit, inspectable voice-leading rules.

## Content pillars

### 1. Counterpoint you can hear

**Audience:** music theory learners, composers, classical/jazz-curious musicians.  
**Search intent:** "teach me the rule, then let me hear it."  
**Why this fits:** public counterpoint exercises already have search demand, but they are mostly notation-first; Contrapunk can make rules audible instantly.

**Keyword clusters**

- species counterpoint, first species counterpoint, second species counterpoint, cantus firmus exercises
- counterpoint generator, online counterpoint tool, counterpoint app
- voice leading rules, parallel fifths, contrary motion, oblique motion, consonance/dissonance
- Palestrina counterpoint, Bach chorale harmony, Fux counterpoint

**Content formats**

- "First species counterpoint: the 5 rules you can hear immediately"
- "Parallel thirds vs real counterpoint: why one sounds like a pedal and one sounds like two players"
- "No parallel fifths: 60-second before/after demo"
- Interactive glossary pages: `parallel fifths`, `contrary motion`, `cantus firmus`, `voice leading`

**CTA:** open browser demo with a preset matching the lesson.

### 2. Guitar into harmony

**Audience:** guitarists, guitar teachers, loopers/live performers.  
**Search intent:** "make my guitar sound bigger without writing four parts."  
**Why this fits:** guitar counterpoint is considered rewarding but awkward on the fretboard; Contrapunk removes some mechanical friction while keeping the musical concept.

**Keyword clusters**

- guitar counterpoint exercises, counterpoint on guitar, contrary motion guitar
- guitar harmonizer, guitar harmony generator, guitar to MIDI harmony
- harmonize guitar melody, guitar double stops thirds sixths, guitar voice leading
- real-time guitar MIDI, low latency guitar to MIDI, guitar MIDI app

**Content formats**

- "Counterpoint exercises for guitarists: parallel, oblique, contrary motion"
- "Guitar harmonizer pedal vs counterpoint generator: A/B audio"
- "Turn one clean guitar line into 3-part harmony in the browser"
- Shorts/Reels: single dry riff → pedal-style thirds → Contrapunk counterpoint

**CTA:** browser demo for MIDI users; native app for lowest-latency guitar input.

### 3. DAW and plugin workflow help

**Audience:** Ableton/Logic/GarageBand/Reaper users, producers, plugin users.  
**Search intent:** "how do I route this into my actual setup?"  
**Why this fits:** docs/issues show first-sound and routing friction. Helpful setup pages convert better than launch posts.

**Keyword clusters**

- MIDI harmony plugin, MIDI harmonizer plugin, harmony generator plugin
- Ableton MIDI harmonizer, Logic MIDI harmonizer, GarageBand MIDI routing
- IAC Driver Logic Ableton, loopMIDI setup, Web MIDI to DAW
- CLAP plugin host, route MIDI to synth, MIDI output to DAW

**Content formats**

- "How to send Contrapunk MIDI into Logic with IAC Driver"
- "Ableton Live setup: Web MIDI/browser or native app → instrument track"
- "Why don't I hear sound? MIDI input/output checklist"
- "CLAP plugin hosting for musicians: what it means, no jargon"

**CTA:** 5-minute setup checklist + app link.

### 4. Composer workflows and writer's-block prompts

**Audience:** composers, songwriters, media/scoring users, jazz harmony learners.  
**Search intent:** "give me a musical idea I can finish."  
**Why this fits:** Scaler and similar tools own chord-progressions; Contrapunk should own live melody-to-independent-lines.

**Keyword clusters**

- harmonize a melody, melody harmonizer, accompaniment generator
- voice leading plugin, composition tool, counterpoint composition software
- modal interchange, mode mixture, Barry Harris sixth diminished, jazz harmonization
- Bach style harmonizer, Palestrina style harmony, chorale harmonizer

**Content formats**

- "Harmonize one melody 5 ways: Palestrina, Bach, Jazz, Free, Barry Harris"
- "From one motif to a cue: live counterpoint sketching in 10 minutes"
- "Modal interchange you can hear: borrow a color without losing the line"
- Downloadable MIDI/stem packs from each article

**CTA:** download the MIDI + try the same settings live.

### 5. Comparison and category-definition pages

**Audience:** problem-aware buyers/users comparing tools.  
**Search intent:** "which tool solves my harmony problem?"  
**Why this fits:** market docs identify a clean gap: harmonizers do parallel pitch copies, chord tools voice preselected chords, academic systems are not broad products.

**Keyword clusters**

- Scaler alternative, harmony plugin alternative, MIDI chord generator alternative
- guitar harmonizer pedal alternative, vocal harmonizer vs counterpoint
- counterpoint generator vs chord progression generator
- BachDuet alternative, Tonica Fugata alternative, real-time counterpoint software

**Content formats**

- One honest hub: "Harmony tools compared: harmonizer pedals, chord plugins, counterpoint generators"
- "Scaler-style chord tools vs Contrapunk: chords first vs melody first"
- "Vocal harmony plugins vs instrumental counterpoint"
- A/B audio grid: fixed thirds, chord block, Contrapunk independent line

**CTA:** "If you want live melody-following counterpoint, try Contrapunk. If you want chord progression libraries, use Scaler/Captain/etc."

### 6. Open-source audio engineering

**Audience:** audio devs, Rust/WASM/Tauri/plugin developers, GitHub users.  
**Search intent:** "how was this built, can I inspect/fork it?"  
**Why this fits:** open source is a differentiator, but this should be a secondary lane so musician messaging stays clear.

**Keyword clusters**

- Rust audio plugin, Rust MIDI app, real-time audio Rust
- WASM MIDI, Web MIDI Rust, Tauri audio app
- CLAP plugin host Rust, low latency pitch detection, guitar pitch detection
- open source music theory software, counterpoint engine

**Content formats**

- "How Contrapunk rejects parallel fifths in real time"
- "Rust core, browser app, native app: one harmony engine"
- "CLAP hosting in a standalone music app"
- Dev log posts tied to real shipped features only

**CTA:** GitHub repo, issues, build-from-source, developer newsletter.

## Keyword priority map

| Priority | Cluster | Primary page type | Why now |
|---|---|---|---|
| P0 | counterpoint generator / real-time counterpoint / voice leading rules | Evergreen pillar + glossary | Highest fit to differentiation |
| P0 | guitar harmonizer alternative / guitar counterpoint exercises | Demo lesson + A/B audio | Strong musician hook; visual/audio friendly |
| P0 | MIDI harmony plugin / Ableton/Logic MIDI routing / IAC Driver | Setup tutorials | Converts searches into successful first sound |
| P1 | harmonize a melody / melody harmonizer / accompaniment generator | Workflow posts | Captures composers/songwriters without generic AI positioning |
| P1 | Scaler alternative / chord generator alternative | Honest comparison hub | Useful, but avoid thin competitor pages |
| P1 | Barry Harris / modal interchange / Bach chorale harmonizer | Deep-dive demos | Great niche authority; lower volume, high relevance |
| P2 | Rust audio / WASM MIDI / CLAP host Rust | Dev logs | Good backlinks/GitHub stars; secondary to musicians |

## 8-week publishing plan

Cadence: **1 useful long-form page + 2 short demo clips + 1 community post per week**. Keep every article demo-first: short audio/video at top, lesson below, browser/native CTA, FAQ at bottom.

| Week | Main publish | Demo clips | Distribution | CTA / lead magnet |
|---|---|---|---|---|
| 1 | **What is a counterpoint generator?** Hear parallel harmony vs independent lines | 30s A/B: fixed thirds vs Contrapunk; 30s "no parallel fifths" | YouTube, TikTok/Reels, r/musictheory answer threads, Discord | Try browser demo preset |
| 2 | **First species counterpoint: rules you can hear** | consonance-only demo; bad parallel fifths rejected | Music theory communities, teacher newsletter outreach | PDF: first-species cheat sheet + MIDI cantus firmus |
| 3 | **Counterpoint exercises for guitarists** | dry guitar → counterpoint; contrary-motion riff | Guitar forums, YouTube Shorts, Instagram guitar tags | PDF/TAB: guitar counterpoint workout |
| 4 | **How to route Contrapunk into Logic/Ableton/GarageBand** | IAC setup in 45s; "why no sound?" checklist | DAW subreddits/forums, support docs, Discord | DAW routing checklist |
| 5 | **Guitar harmonizer pedal vs counterpoint generator** | pedal-style parallel third vs independent voice; clean guitar latency proof | Guitar/productivity communities, Hacker News only if technical angle | Audio A/B gallery |
| 6 | **Harmonize one melody 5 ways**: Palestrina, Bach, Jazz, Free, Barry Harris | same melody in 5 styles; Barry Harris color clip | Composer/scoring groups, YouTube | MIDI + preset pack |
| 7 | **MIDI harmony plugin workflow: melody first, not chord first** | Contrapunk into synth; Contrapunk + CLAP instrument | Producer communities, plugin forums | "5 melody prompts" pack |
| 8 | **How Contrapunk works under the hood**: rules, latency, Rust/WASM | code-to-sound clip; browser vs native clip | GitHub, Rust/audio-dev communities, HN if launch-quality | Build-from-source/dev signup |

After week 8: update winners, not the calendar. If week 3 guitar demos outperform theory, run another 4-week guitar series. If DAW routing gets conversions, expand by DAW one at a time.

## Demo and lead-magnet ideas

1. **Counterpoint cheat sheet** — one-page rules: consonances, contrary motion, parallel fifths/octaves, species basics.
2. **Guitar counterpoint workout** — TAB + standard notation + MIDI for parallel, oblique, contrary, and similar motion.
3. **DAW routing checklist** — macOS IAC, Windows loopMIDI, Linux JACK/ALSA, plus "why don't I hear sound?" checks.
4. **A/B audio gallery** — fixed interval harmonizer vs chord block vs Contrapunk; short embeddable clips for every comparison page.
5. **One melody, five styles pack** — MIDI/stems showing Palestrina/Bach/Jazz/Free/Barry Harris settings.
6. **Browser demo presets** — links or screenshots for "strict counterpoint," "jazz color," "guitar trio," "chorale sketch."
7. **Open-source dev note** — tiny annotated rule example, not a full architecture essay.

Do **not** gate the browser demo. Gate only PDFs/MIDI packs if email capture is needed.

## Lightweight metrics

Use Search Console, basic web analytics, app events, and UTMs. No heavy attribution stack yet.

**Acquisition**

- Non-brand Google clicks/impressions for P0 clusters.
- SERP CTR by page title; rewrite titles under ~1% CTR after enough impressions.
- YouTube/Shorts retention through first 10 seconds.
- Community referral sessions from guitar/theory/DAW posts.
- Backlinks or embeds from educators/devs.

**Activation**

- Article → `app.contrapunk.com` click-through.
- App opened → input selected → output selected → Start clicked → first note/harmony generated.
- macOS DMG downloads from content pages.
- "No sound" support questions per 100 new users.

**Lead / trust**

- PDF/MIDI pack downloads.
- Email/Discord/GitHub star conversion from content pages.
- Comments/questions that become new FAQs.
- Repeat visits to tutorial pages.

**Review rhythm**

- Weekly: publish + check technical indexing, broken CTAs, demo playback.
- Every 4 weeks: keep/kill topics based on clicks, demo starts, and first-note success.
- Every 8 weeks: refresh the top 3 pages with better audio, FAQs, and internal links.

## On-page template

Use this for nearly every SEO/tutorial page:

1. 10-30 second audible demo above the fold.
2. One-sentence promise: "You will hear/write/route X by the end."
3. Minimal concept explanation.
4. Step-by-step walkthrough.
5. "Try it in Contrapunk" settings box.
6. Download: MIDI/PDF/stems if useful.
7. FAQ with real support/search questions.
8. Links to adjacent pillar pages.

This matches Google's guidance to make content helpful, original, and people-first rather than search-engine-first.

## What to skip

- Skip generic "AI music generator" SEO. Too broad, too competitive, weak fit.
- Skip lots of thin competitor pages. Make one honest comparison hub with audio proof.
- Skip daily generic social posting. Reuse strong demos; don't feed the treadmill.
- Skip a full music theory course. Short lessons that end in sound are enough.
- Skip podcast/webinar/influencer programs until demo pages already convert.
- Skip claims around unshipped or unstable surfaces: VST3/AU, cloud, Golem/adaptive drummer, future Elixir work. Mention only when released.
- Skip developer-first messaging on musician pages. "Rust/WASM/CLAP" belongs in the dev pillar, not the main hook.
- Skip hiding the product behind email capture. The browser demo is the lead magnet.

## Sources used

Local:

- `README.md` — current product promise, surfaces, guitar/MIDI, voice-leading styles.
- `docs/MARKET_ANALYSIS.md` — market gap and competitor framing.
- `recon/github-issues.md` — onboarding/demos/DAW/guitar/theory themes.
- `recon/planning-roadmap.md` — roadmap caveats; avoid marketing unshipped work.

External:

- [Google Search Central: Creating helpful, reliable, people-first content](https://developers.google.com/search/docs/fundamentals/creating-helpful-content) — content quality principles.
- [Google Search Central: SEO Starter Guide](https://developers.google.com/search/docs/fundamentals/seo-starter-guide) — crawlability, descriptive titles, useful content, promotion basics.
- [Contrapunk website](https://contrapunk.com/) and [Getting Started](https://contrapunk.com/tutorials/getting-started/) — current public messaging and setup flow.
- [Scaler 3 official page](https://scalermusic.com/products/scaler-3/) — chord/progression/voice-leading competitor positioning.
- [BachDuet project](https://labsites.rochester.edu/air/projects/BachDuet.html) — real-time human-machine counterpoint research context.
- [iZotope Nectar 4 Voices docs](https://docs.izotope.com/nectar4/en/voices/index.html) — vocal harmony/parallel harmony comparison context.
- [Puget Sound species counterpoint exercises](https://musictheory.pugetsound.edu/mt21c/IntroductionToCounterpointExercises.html) — theory search intent and exercise format.
- [Acoustic Guitar: counterpoint exercises on guitar](https://acousticguitar.com/fun-and-challenging-counterpoint-exercises-on-guitar/) — guitarist framing and practical exercise intent.
