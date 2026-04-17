//! Polyphonic sine synth used as a test tone generator for the audio
//! pipeline before VST3 plugin hosting lands (sub-project 2).

use std::f32::consts::TAU;

/// ADSR envelope stage.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum EnvelopeStage {
    Idle,
    Attack,
    Decay,
    Sustain,
    Release,
}

const ATTACK_SECONDS: f32 = 0.005;
const DECAY_SECONDS: f32 = 0.05;
const SUSTAIN_LEVEL: f32 = 0.8;
const RELEASE_SECONDS: f32 = 0.15;

/// A single sine oscillator with an ADSR amplitude envelope.
#[derive(Clone, Debug)]
pub struct SineVoice {
    sample_rate: f32,
    phase: f32,
    freq: f32,
    amp: f32,
    env: f32,
    stage: EnvelopeStage,
    /// Note number this voice was triggered with (for voice matching on Note-Off).
    note: Option<u8>,
}

impl SineVoice {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            sample_rate,
            phase: 0.0,
            freq: 440.0,
            amp: 0.0,
            env: 0.0,
            stage: EnvelopeStage::Idle,
            note: None,
        }
    }

    /// Start a note at the given frequency and amplitude (0.0–1.0).
    pub fn note_on(&mut self, freq: f32, amp: f32) {
        self.freq = freq;
        self.amp = amp.clamp(0.0, 1.0);
        self.stage = EnvelopeStage::Attack;
    }

    /// Release the note. Envelope transitions to Release stage.
    pub fn note_off(&mut self) {
        if self.stage != EnvelopeStage::Idle {
            self.stage = EnvelopeStage::Release;
        }
    }

    /// True if the voice is currently producing audio (any non-idle stage).
    pub fn is_active(&self) -> bool {
        self.stage != EnvelopeStage::Idle
    }

    /// Record which MIDI note this voice is playing (for Note-Off matching).
    pub fn set_note(&mut self, note: u8) {
        self.note = Some(note);
    }

    /// The note this voice is playing, if any.
    pub fn note(&self) -> Option<u8> {
        self.note
    }

    /// Jump the envelope straight to sustain. Testing aid only.
    #[cfg(test)]
    pub fn skip_attack(&mut self) {
        self.env = SUSTAIN_LEVEL;
        self.stage = EnvelopeStage::Sustain;
    }

    /// Process one buffer, ADDING the voice's output into `output` (mono).
    pub fn process(&mut self, output: &mut [f32]) {
        if self.stage == EnvelopeStage::Idle {
            return;
        }
        let phase_inc = self.freq * TAU / self.sample_rate;
        let attack_step = 1.0 / (ATTACK_SECONDS * self.sample_rate);
        let decay_step = (1.0 - SUSTAIN_LEVEL) / (DECAY_SECONDS * self.sample_rate);
        let release_step = SUSTAIN_LEVEL / (RELEASE_SECONDS * self.sample_rate);

        for sample in output.iter_mut() {
            // Advance envelope.
            match self.stage {
                EnvelopeStage::Attack => {
                    self.env += attack_step;
                    if self.env >= 1.0 {
                        self.env = 1.0;
                        self.stage = EnvelopeStage::Decay;
                    }
                }
                EnvelopeStage::Decay => {
                    self.env -= decay_step;
                    if self.env <= SUSTAIN_LEVEL {
                        self.env = SUSTAIN_LEVEL;
                        self.stage = EnvelopeStage::Sustain;
                    }
                }
                EnvelopeStage::Sustain => {}
                EnvelopeStage::Release => {
                    self.env -= release_step;
                    if self.env <= 0.0 {
                        self.env = 0.0;
                        self.stage = EnvelopeStage::Idle;
                        self.note = None;
                        break;
                    }
                }
                EnvelopeStage::Idle => break,
            }

            *sample += self.phase.sin() * self.env * self.amp;
            self.phase += phase_inc;
            if self.phase >= TAU {
                self.phase -= TAU;
            }
        }
    }
}

use crate::audio_out::midi_queue::MidiEvent;

/// Convert a MIDI note number to a frequency in Hz (equal temperament, A4=440).
pub fn midi_note_to_freq(note: u8) -> f32 {
    440.0 * 2.0_f32.powf((note as f32 - 69.0) / 12.0)
}

/// Polyphonic wrapper around `SineVoice`. Dispatches MIDI events to free
/// voices, steals the oldest voice when all are busy, and mixes all voices
/// into a stereo output buffer.
#[derive(Debug)]
pub struct PolySynth {
    voices: Vec<SineVoice>,
    /// Pre-allocated mono scratch buffer, reused every callback to avoid
    /// heap allocation on the audio thread.
    scratch: Vec<f32>,
}

impl PolySynth {
    /// Create a new PolySynth with `max_polyphony` voices pre-allocated.
    pub fn new(sample_rate: f32, max_polyphony: usize) -> Self {
        Self {
            voices: (0..max_polyphony)
                .map(|_| SineVoice::new(sample_rate))
                .collect(),
            scratch: vec![0.0_f32; 2048],
        }
    }

    /// Handle a single MIDI event: allocate/release voices.
    pub fn handle_event(&mut self, event: MidiEvent) {
        match event {
            MidiEvent::NoteOn { note, velocity, .. } => {
                // Find an idle voice first; otherwise steal the first active voice.
                let idx = self.voices.iter().position(|v| !v.is_active()).unwrap_or(0);
                let amp = (velocity as f32 / 127.0).clamp(0.0, 1.0);
                self.voices[idx].note_on(midi_note_to_freq(note), amp);
                self.voices[idx].set_note(note);
            }
            MidiEvent::NoteOff { note, .. } => {
                // Release every active voice holding this note.
                for v in self.voices.iter_mut() {
                    if v.note() == Some(note) && v.is_active() {
                        v.note_off();
                    }
                }
            }
        }
    }

    /// Count of currently active (non-idle) voices. Testing/metering aid.
    pub fn active_voice_count(&self) -> usize {
        self.voices.iter().filter(|v| v.is_active()).count()
    }

    /// Process a stereo buffer. Samples are interleaved L R L R ....
    /// Voices are summed in mono then duplicated to both channels.
    pub fn process_stereo(&mut self, output: &mut [f32]) {
        let frames = output.len() / 2;
        // Zero the output.
        for s in output.iter_mut() {
            *s = 0.0;
        }
        // Grow the scratch buffer if the host sends a larger buffer than
        // expected (rare — only on buffer-size change). This is the only
        // allocation path and it amortises to zero after the first callback.
        if self.scratch.len() < frames {
            self.scratch.resize(frames, 0.0);
        }
        // Zero the scratch region we'll actually use.
        let mono = &mut self.scratch[..frames];
        mono.iter_mut().for_each(|s| *s = 0.0);
        for voice in self.voices.iter_mut() {
            voice.process(mono);
        }
        // Interleave mono into stereo output.
        for (i, &s) in mono.iter().enumerate() {
            output[i * 2] = s;
            output[i * 2 + 1] = s;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio_out::midi_queue::MidiEvent;

    #[test]
    fn test_idle_voice_produces_silence() {
        let mut voice = SineVoice::new(48_000.0);
        let mut buf = [0.0_f32; 64];
        voice.process(&mut buf);
        assert!(buf.iter().all(|&s| s == 0.0));
    }

    #[test]
    fn test_note_on_produces_signal() {
        let mut voice = SineVoice::new(48_000.0);
        voice.note_on(440.0, 0.8);
        let mut buf = [0.0_f32; 512];
        voice.process(&mut buf);
        // After attack, the voice should have non-silent samples.
        let peak = buf.iter().map(|s| s.abs()).fold(0.0_f32, f32::max);
        assert!(peak > 0.1, "peak was {peak}");
    }

    #[test]
    fn test_frequency_is_correct() {
        // 1 cycle of 480 Hz at 48 kHz = 100 samples.
        let mut voice = SineVoice::new(48_000.0);
        voice.note_on(480.0, 1.0);
        voice.skip_attack(); // jump to sustain so we don't fight the envelope
        let mut buf = [0.0_f32; 200];
        voice.process(&mut buf);
        // Count zero crossings in the buffer. 200 samples = 2 cycles = 4 zero crossings.
        let crossings = buf
            .windows(2)
            .filter(|w| w[0].signum() != w[1].signum())
            .count();
        assert!(
            (3..=5).contains(&crossings),
            "expected ~4 zero crossings, got {crossings}"
        );
    }

    #[test]
    fn test_note_off_releases() {
        let mut voice = SineVoice::new(48_000.0);
        voice.note_on(440.0, 1.0);
        voice.skip_attack();
        voice.note_off();
        // After a long release, the voice should be silent and inactive.
        let mut buf = [0.0_f32; 48_000]; // 1 second
        voice.process(&mut buf);
        assert!(!voice.is_active(), "voice should be inactive after release");
        // Last samples should be silent.
        assert!(buf[buf.len() - 1].abs() < 0.001);
    }

    #[test]
    fn test_polysynth_note_on_allocates_voice() {
        let mut synth = PolySynth::new(48_000.0, 8);
        synth.handle_event(MidiEvent::NoteOn {
            voice: 0,
            note: 60,
            velocity: 100,
        });
        let active = synth.active_voice_count();
        assert_eq!(active, 1);
    }

    #[test]
    fn test_polysynth_note_off_releases_matching_voice() {
        let mut synth = PolySynth::new(48_000.0, 8);
        synth.handle_event(MidiEvent::NoteOn {
            voice: 0,
            note: 60,
            velocity: 100,
        });
        synth.handle_event(MidiEvent::NoteOn {
            voice: 0,
            note: 64,
            velocity: 100,
        });
        assert_eq!(synth.active_voice_count(), 2);
        synth.handle_event(MidiEvent::NoteOff { voice: 0, note: 60 });
        // Release is still "active" (envelope not yet at zero). Run a little audio
        // to let the 60's envelope reach zero.
        let mut stereo = [0.0_f32; 48_000 * 2]; // 1s stereo
        synth.process_stereo(&mut stereo);
        // Now voice 60 should be gone; voice 64 is still held (no note-off).
        assert_eq!(synth.active_voice_count(), 1);
    }

    #[test]
    fn test_polysynth_mixes_multiple_voices() {
        let mut synth = PolySynth::new(48_000.0, 8);
        synth.handle_event(MidiEvent::NoteOn {
            voice: 0,
            note: 60,
            velocity: 100,
        });
        synth.handle_event(MidiEvent::NoteOn {
            voice: 1,
            note: 64,
            velocity: 100,
        });
        let mut stereo = [0.0_f32; 1024];
        synth.process_stereo(&mut stereo);
        let peak = stereo.iter().map(|s| s.abs()).fold(0.0_f32, f32::max);
        assert!(peak > 0.0, "polysynth should produce non-silent mix");
    }

    #[test]
    fn test_midi_note_to_freq() {
        // A4 = MIDI 69 = 440 Hz.
        assert!((midi_note_to_freq(69) - 440.0).abs() < 0.001);
        // A5 = MIDI 81 = 880 Hz.
        assert!((midi_note_to_freq(81) - 880.0).abs() < 0.001);
    }
}
