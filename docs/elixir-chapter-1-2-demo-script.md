# Elixir Chapter 1 and 2 Demo Capture Contract

These clips use only project-authored MIDI, Elixir audio, and the deterministic states exposed by the Synth view. Record after the feature branch is merged and audio QA is complete.

## Shared Capture Settings

- Sample rate: 48 kHz
- Master gain: 0.25
- Delay and reverb: off unless a clip explicitly explains output processing
- Input: computer keyboard or a project-authored MIDI phrase
- Capture the Elixir Synth view at 16:9 with the edited role visible
- Open `/?elixir-example=<id>` to enter the Synth view with a deterministic example already loaded
- Begin and end every clip at silence
- Preserve relative level for phase, addition, ring, and articulation comparisons
- Export captions and a separate text description for the website

## Chapter 1

### Clip 1: One pitch, five colours

1. Hold A3 at 220 Hz.
2. Select Sine, Three harmonics, Odd only, Saw-like, then Dark.
3. Pause on each recipe long enough to show the waveform and spectrum.
4. Keep pitch, velocity, role, and master gain fixed.

Website placement: beside “Build a tone by adding sine waves.”

### Clip 2: Timbre organizes the ensemble

1. Load **Ensemble colours** from Factory patch.
2. Play one project-authored phrase with Input, Harmony, Canon, and Counterpoint entering in that order.
3. Solo each role once, then restore the ensemble.
4. Keep notes and rhythm fixed while the role recipes provide the identity.

Website placement: beside “Musical application 1: harmonic call and response.”

## Chapter 2

### Clip 3: Phase becomes level

1. Use Sine with Add mode and matched operator tuning.
2. Play 0°, 90°, and 180° without changing either level.
3. Show the component readout and waveform for each state.
4. Do not normalize the three recordings.

Website placement: beside “Constructive and destructive summation.”

### Clip 4: Addition versus multiplication

1. Hold A5 at 880 Hz.
2. Set operator B one octave down at 440 Hz.
3. Play Add, then Ring.
4. Identify 440 Hz difference and 1320 Hz sum components in captions.

Website placement: beside “Heterodyning: make a small difference audible.”

### Clip 5: Ring-down versus maintained oscillation

1. Load **Passive ring-down** from Factory patch and excite one note.
2. Let it reach silence.
3. Load **Maintained vibrato** and hold the same note.
4. Release it cleanly after the sustained comparison.

Website placement: beside “Sustained LC oscillation.” State clearly that the envelope is the digital product analogue, not an LC simulation.

### Clip 6: Independent pitch and amplitude trajectories

1. Use one fixed harmonic recipe.
2. Play a continuous Slide with sustained articulation.
3. Repeat the same pitch path with short detached notes.
4. Play a held note with 18-cent, 5 Hz vibrato.
5. Keep the scope on-screen and label which controls alter pitch versus amplitude.

Website placement: beside “Amplitude as a trajectory” and “Pitch as a trajectory.”

## Publication Checklist

- Use the matching `FACTORY_PATCHES` identifier in production notes and website deep links.
- Verify the recorded patch against `elixir-preset::factory_presets()`.
- Include captions, alt text, and a long description.
- State exact note frequencies, harmonic coefficients, phase, interval, envelope, and vibrato values.
- Credit audio and video as project-authored.
- Do not introduce filters, a modulation matrix, wavetable morphing, or other later-chapter concepts in these clips.
