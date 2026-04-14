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

#[cfg(test)]
mod tests {
    use super::*;

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
}
