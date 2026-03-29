# Phase 7: Mic Input - Context

**Gathered:** 2026-02-05
**Status:** Ready for planning

<domain>
## Phase Boundary

Capture audio from microphone for pitch-to-MIDI conversion and raw audio passthrough for vocoder. User can select an audio input device, detected pitch is converted to MIDI notes feeding the harmony engine, and GUI displays detected pitch and confidence level. Vocoder functionality itself is Phase 8.

</domain>

<decisions>
## Implementation Decisions

### Pitch Detection Behavior
- Monophonic only — single pitch at a time, simpler and more reliable for voice/melody
- Immediate trigger — send Note-On as soon as pitch detected with sufficient confidence
- Volume threshold gate + confidence cutoff combined — both user-adjustable in UI
- Voice range C2-C6 — optimized for vocals, ignores extreme frequencies
- Quantize to nearest chromatic note — no pitch bend messages, clean output for harmony engine
- Smooth vibrato — track center pitch, ignore small oscillations, one stable note
- Note-Off on silence OR pitch change — whichever comes first
- Trust algorithm for octave detection — add octave lock later if users report problems (incremental approach)

### Audio Device Selection UX
- Unified IN dropdown — mic appears alongside MIDI devices ("Mic: Built-in Microphone")
- Dedicated Mic section in Settings tab — all tuning controls (thresholds, latency) in one place
- Level meter visible in Play tab when mic is active — immediate feedback that audio is coming in
- Persist all mic settings — device selection, thresholds, latency all saved for next launch
- Mic profiles — ability to save/load different configurations ("Quiet Studio", "Noisy Room", "Fast Playing", etc.)

### Confidence & Feedback Display
- Full tuner-style display — note name + octave + cents deviation (e.g., "C4 +12¢")
- Rich visual confidence — color coding (green/yellow/red) + visual bar meter combined
- Distinct color for mic-detected notes — unique color on piano keyboard, different from MIDI input
- Dedicated pitch panel — like a virtual tuner pedal with room for full display

### Latency vs Accuracy
- User-adjustable buffer size — latency slider in Mic Settings for power users
- Default 30-40ms latency — balanced starting point
- Latency included in mic profiles — "Fast Playing" can have low latency, "Ballad" higher accuracy
- Show latency in ms — display actual value in Mic Settings

### Claude's Discretion
- Pitch detection algorithm choice (YIN, pYIN, or similar)
- Exact buffer sizes and FFT parameters
- Specific colors for mic input notes
- Audio capture library selection (cpal, rodio, or similar)
- Raw audio buffer design for Phase 8 vocoder integration

</decisions>

<specifics>
## Specific Ideas

- Mic profiles should work like musical style presets — save everything, load with one click
- The pitch panel should feel like a guitar tuner pedal — familiar interface for musicians
- Incremental approach: ship working pitch detection first, add octave lock only if needed

</specifics>

<deferred>
## Deferred Ideas

- Autotune/pitch correction → Phase 8 (Vocoder) — requires audio output, not just MIDI
- Octave lock option → future enhancement if octave jumping becomes a reported problem
- Polyphonic detection → Phase 9 (Guitar Input) handles this use case

</deferred>

---

*Phase: 07-mic-input*
*Context gathered: 2026-02-05*
