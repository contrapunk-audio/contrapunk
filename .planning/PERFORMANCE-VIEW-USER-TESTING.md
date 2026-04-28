# Performance View — Pre-Build User Testing Plan

A 30-minute session to validate (or invalidate) the design assumptions in `PERFORMANCE-VIEW.md` against a real non-musician *before* committing engineering effort. Run with the **current** Contrapunk UI (not the future Performance view) — the goal is to find out where the existing UI breaks down for the target audience.

## Goal

Diagnose where non-musicians actually stall in the current ~40-control UI. Validate that the brutal-critic findings reflect real user behavior rather than reviewer-grade nitpicks. Pin down which problems are *naming* (knobs make sense, labels don't), which are *count* (too many controls), and which are *defaults* (initial state hides the value prop).

## Recruit profile

Ideal: someone who likes music, can identify a song, has tried a music-making app once, but can't tell you what a "diatonic third" is. A hobbyist guitarist or singer who plays for fun. Not a producer, not a music-theory student. One person is enough for v1; this is qualitative, not statistical.

## Setup (before they arrive)

- Laptop running the current Contrapunk app
- App in default cold-start state (close it and reopen so they get the genuine first-launch experience)
- Audio output working through laptop speakers or headphones
- A MIDI keyboard plugged in if available; if not, the Computer Keyboard input works
- Screen recording on (with their consent) so you can review pause/hesitation moments later
- A timer visible to you (not them)

## Session protocol

**Don't coach. Don't explain music theory. Don't narrate the UI. Just observe.** If they ask "what does this do?" — write it down, then say *"What do you think it does?"* — you're trying to find out where the UI fails to teach itself.

### Phase 1: Cold open (5 min)

Hand them the laptop, app already running. Say:

> *"This app generates backing harmonies when you play or sing a note. Try to make it produce some harmony."*

Then stop talking. Watch.

**Observations to capture:**
- Time to first audible harmony note (or give-up)
- Whether they find Start before trying to play
- What they tried first (which control did they touch?)
- What they said out loud
- Whether they discovered that the default mode (`PassThrough`) is silent
- Whether they hear the Internal Synth at all (it's pre-routed but not visible)

### Phase 2: Tasks (15 min)

Give them these tasks one at a time. Don't move on until they either complete or explicitly give up. Write down time-to-complete for each.

| # | Task | What it actually exercises |
|---|---|---|
| 1 | "Make the harmony thicker — more voices." | `voiceCount` discoverability |
| 2 | "Change the style of the harmony — make it sound different." | `mode` picker + label comprehensibility |
| 3 | "Try to make it sound sad or moody." | scale-mode + key-mode selection |
| 4 | "Set this to play in the key of A." | Key picker + auto-key interaction |
| 5 | "Save this exact sound so you can come back to it later." | Save/recall (currently impossible — observe how they look for it) |
| 6 | "Make the harmony come from above your note instead of below." | `voicePosition` (the "You play" picker) |
| 7 | "Make this sound less random and more controlled." | Voice-leading toggle + style selection |

For each task, capture:
- Did they complete it? (yes / partial / gave up)
- What was their first attempt?
- What labels did they search for that didn't exist?
- What jargon stopped them cold?
- How many controls did they touch on the way?

### Phase 3: Reactions (5 min)

After all tasks, ask these in order. Write down their words verbatim where possible.

1. *"What was the most confusing thing about using this?"*
2. *"What did you expect to find that wasn't there?"*
3. *"Were there any words on the screen you didn't understand?"* (point if needed)
4. *"If you were going to use this in a band, what's missing?"*
5. *"What sound did you make that you actually liked? Could you make it again?"*

### Phase 4: Brief reveal (5 min)

Show them the `PERFORMANCE-VIEW.md` 8-knob mock-up (verbally describe if no mock exists yet):
> *"We're considering replacing this with 8 knobs: Mode, Voices, Tightness, Adventurous, Key, Scale, You-play, Spread. Plus the harmony you just made would be remembered when you reopen the app, but there's no explicit Save button. Does that sound better, worse, or the same?"*

Capture their gut reaction. Probe specifically:
- Is "Tightness" a word they'd intuit?
- Does "Adventurous" parse as a music control?
- Are they upset about no Save button, or relieved by fewer controls?

## Specific moments to watch for

- **The cold-start silence** (default mode = PassThrough). Do they think the app is broken? How long before they realize they need to change something to hear anything?
- **The Auto-key dropdown bug** (clicking Key while AUTO is on silently disables it). Do they hit it? Do they notice?
- **The 4-vs-8 voice-count mismatch** (UI clamps at 4 even though engine supports 8). Do they want more voices than the picker allows?
- **The Counterpoint Species card** (appears when they pick Strict Counterpoint). What's their face when they see "Species 2 (2:1)" and "Strictness: Relaxed/Strict"?
- **The PassThrough bypass moment** — first time they choose a mode that produces audible harmony. How did they find it? What was their reaction?
- **The "You play" picker** — do they understand "Soprano (1)" vs "Bass (4)" vocabulary, or do they ignore the control entirely?

## After the session

Within 24 hours, write a 1-page observation summary. Include:

- 5 things they got stuck on, ranked by how much time they lost
- 3 specific labels / words they couldn't parse
- 1 task they fully gave up on (likely Task 5: save/recall)
- Their gut reaction to the 8-knob redesign reveal
- Any design assumptions in `PERFORMANCE-VIEW.md` that this session disproved

If their behavior matches what `PERFORMANCE-VIEW.md` predicts, build it. If they got stuck somewhere we didn't anticipate, revise the doc before building.

## Anti-patterns

- Don't run this with a music producer. They'll succeed at every task and tell you the UI is fine.
- Don't run this with the developer team. They know where everything is.
- Don't ship the brutal-critic findings to the test subject before the session. You want their fresh reactions, not validations of someone else's analysis.
- Don't watch one user, conclude "the redesign is right," and stop. One user invalidates assumptions; multiple users (3-5) validate them. v1 of this plan asks for one because going from zero validation to one is the highest-leverage step; further runs only matter if the first one's findings surprise you.
