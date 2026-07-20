//! Realtime procedural drummer engine.

use std::f32::consts::TAU;

use crate::dynamics::{AdaptiveDynamics, DrummerIntent};
use crate::events::{Articulation, DrumHit, DrumPiece};
use crate::follow::FollowInput;
use crate::params::{EngineParams, SharedParams};
use crate::style::Style;

const MAX_EVENTS_PER_BLOCK: usize = 96;
const MAX_VOICES: usize = 32;

/// Clock snapshot supplied by the host once per audio block.
#[derive(Clone, Copy, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct ClockSnapshot {
    pub sample_pos: u64,
    pub sample_rate: u32,
    pub bpm: f64,
    pub playing: bool,
}

/// Audio-native drummer engine. v0.1 uses procedural drum synthesis so
/// the standalone app can be run before sample-kit assets exist.
pub struct Engine {
    sample_rate: u32,
    max_block: usize,
    voices: [Voice; MAX_VOICES],
    params: EngineParams,
    dynamics: AdaptiveDynamics,
    age_counter: u64,
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}

impl Engine {
    pub fn new() -> Self {
        Self {
            sample_rate: 48_000,
            max_block: 512,
            voices: [Voice::idle(); MAX_VOICES],
            params: EngineParams::default(),
            dynamics: AdaptiveDynamics::new(),
            age_counter: 0,
        }
    }

    pub fn prepare(&mut self, sample_rate: u32, max_block: usize) {
        self.sample_rate = sample_rate.max(1);
        self.max_block = max_block.max(1);
        for voice in self.voices.iter_mut() {
            voice.sample_rate = self.sample_rate as f32;
        }
    }

    pub fn set_params(&mut self, params: EngineParams) {
        self.params = params;
    }

    /// Snapshot shared params and process one block.
    pub fn process_with_shared_params(
        &mut self,
        shared: &SharedParams,
        mut clock: ClockSnapshot,
        follow: FollowInput,
        output: &mut [f32],
        channels: usize,
    ) {
        let params = shared.snapshot();
        clock.bpm = params.bpm as f64;
        self.set_params(params);
        self.process(clock, follow, output, channels);
    }

    /// Add drum audio into `output`. The host decides whether to clear
    /// the buffer first. No heap allocation, locks, or I/O.
    pub fn process(
        &mut self,
        clock: ClockSnapshot,
        follow: FollowInput,
        output: &mut [f32],
        channels: usize,
    ) {
        if channels == 0 || output.is_empty() {
            return;
        }

        let frames = output.len() / channels;
        if frames == 0 {
            return;
        }

        if clock.sample_rate != 0 && clock.sample_rate != self.sample_rate {
            self.prepare(clock.sample_rate, self.max_block.max(frames));
        }

        let mut events = [DrumHit::default(); MAX_EVENTS_PER_BLOCK];
        let mut event_count = 0usize;

        if clock.playing {
            let intent = self.dynamics.update(follow, frames, self.sample_rate);
            event_count = self.schedule(clock, intent, frames, &mut events);
            events[..event_count].sort_unstable_by_key(|hit| hit.offset_frames);
        }

        let mut next_event = 0usize;
        let master = self.params.master_gain;

        for frame_idx in 0..frames {
            while next_event < event_count && events[next_event].offset_frames as usize == frame_idx
            {
                self.trigger(events[next_event]);
                next_event += 1;
            }

            let mut left = 0.0f32;
            let mut right = 0.0f32;
            for voice in self.voices.iter_mut() {
                if !voice.active {
                    continue;
                }
                let sample = voice.next_sample();
                let pan = voice.pan.clamp(-1.0, 1.0);
                let l_gain = (1.0 - pan).sqrt() * 0.7071;
                let r_gain = (1.0 + pan).sqrt() * 0.7071;
                left += sample * l_gain;
                right += sample * r_gain;
            }

            let base = frame_idx * channels;
            let l = soft_clip(left * master);
            let r = soft_clip(right * master);
            if channels == 1 {
                output[base] += (l + r) * 0.5;
            } else {
                output[base] += l;
                output[base + 1] += r;
                // Leave additional hardware outputs silent. Some interfaces
                // expose loopback/aux channels that can sound unstable if we
                // mirror the main mix into every output bus.
            }
        }
    }

    fn schedule(
        &mut self,
        clock: ClockSnapshot,
        intent: DrummerIntent,
        frames: usize,
        events: &mut [DrumHit; MAX_EVENTS_PER_BLOCK],
    ) -> usize {
        let bpm = clock.bpm.clamp(40.0, 240.0);
        let sample_rate = self.sample_rate as f64;
        let samples_per_beat = sample_rate * 60.0 / bpm;
        let samples_per_step = samples_per_beat / 4.0; // 16th notes in 4/4
        let block_start = clock.sample_pos as f64;
        let block_end = block_start + frames as f64;

        let first_step = (block_start / samples_per_step).floor() as i64 - 2;
        let last_step = (block_end / samples_per_step).ceil() as i64 + 2;
        let mut count = 0usize;

        for step in first_step..=last_step {
            if step < 0 {
                continue;
            }
            let step_in_bar = (step as u64 % 16) as u8;
            let step_in_phrase = (step as u64 % 64) as u8;
            let swing_delay = if step_in_bar % 2 == 1 {
                samples_per_step * self.params.swing as f64 * 0.55
            } else {
                0.0
            };
            let jitter = jitter_samples(step as u64, self.params.complexity, self.sample_rate);
            let event_sample = step as f64 * samples_per_step + swing_delay + jitter as f64;

            if event_sample < block_start || event_sample >= block_end {
                continue;
            }

            let offset = (event_sample - block_start).round().max(0.0) as u32;
            self.schedule_step(
                step as u64,
                step_in_bar,
                step_in_phrase,
                offset,
                intent,
                events,
                &mut count,
            );
        }

        count
    }

    #[allow(clippy::too_many_arguments)]
    fn schedule_step(
        &mut self,
        absolute_step: u64,
        step: u8,
        phrase_step: u8,
        offset: u32,
        intent: DrummerIntent,
        events: &mut [DrumHit; MAX_EVENTS_PER_BLOCK],
        count: &mut usize,
    ) {
        let follow = self.params.follow_amount.clamp(0.0, 1.0);
        let energy = (self.params.intensity
            + follow
                * (intent.energy * 0.42 + intent.section_lift * 0.22 + intent.accent * 0.12
                    - intent.restraint * 0.16))
            .clamp(0.05, 1.0);
        let complexity = (self.params.complexity
            + follow
                * (intent.density * 0.42
                    + intent.fill_tension * 0.18
                    + intent.section_lift * 0.16
                    - intent.restraint * 0.22))
            .clamp(0.0, 1.0);
        let onset_boost = intent.accent * follow;
        let velocity_drive = (energy * 0.72 + intent.velocity * follow * 0.28).clamp(0.0, 1.0);

        let fill_zone = phrase_step >= 56;
        let fill_eagerness = (self.params.fill_amount
            * (0.45 + intent.fill_tension * 0.75 + intent.section_lift * 0.35))
            .clamp(0.0, 1.0);
        let fill_active = fill_zone && fill_eagerness > 0.18 && energy > 0.25;

        if phrase_step == 0
            && absolute_step > 0
            && (intent.section_lift > 0.24 || intent.fill_tension > 0.45 || energy > 0.72)
        {
            push_hit(
                events,
                count,
                offset,
                DrumPiece::Crash,
                0.30 + velocity_drive * 0.50,
            );
        }

        if fill_active {
            self.schedule_fill_step(step, offset, velocity_drive, complexity, events, count);
            self.dynamics.consume_fill(0.45);
            return;
        }

        match self.params.style {
            Style::Rock => schedule_rock(
                step,
                offset,
                velocity_drive,
                complexity,
                onset_boost,
                events,
                count,
            ),
            Style::HalfTime => schedule_half_time(
                step,
                offset,
                velocity_drive,
                complexity,
                onset_boost,
                events,
                count,
            ),
            Style::FourOnFloor => schedule_four_on_floor(
                step,
                offset,
                velocity_drive,
                complexity,
                onset_boost,
                events,
                count,
            ),
        }
    }

    fn schedule_fill_step(
        &self,
        step: u8,
        offset: u32,
        energy: f32,
        complexity: f32,
        events: &mut [DrumHit; MAX_EVENTS_PER_BLOCK],
        count: &mut usize,
    ) {
        if step % 2 == 0 || complexity > 0.55 {
            let piece = match step % 8 {
                0 | 2 => DrumPiece::Snare,
                4 => DrumPiece::TomHigh,
                5 | 6 => DrumPiece::TomMid,
                _ => DrumPiece::TomLow,
            };
            push_hit(events, count, offset, piece, 0.35 + energy * 0.55);
        }

        if step % 4 == 0 {
            push_hit(events, count, offset, DrumPiece::Kick, 0.45 + energy * 0.35);
        }

        if complexity > 0.35 {
            push_hit(
                events,
                count,
                offset,
                DrumPiece::ClosedHat,
                0.18 + energy * 0.18,
            );
        }
    }

    fn trigger(&mut self, hit: DrumHit) {
        if hit.velocity <= 0.0 {
            return;
        }

        if matches!(hit.piece, DrumPiece::ClosedHat | DrumPiece::OpenHat) {
            for voice in self.voices.iter_mut() {
                if matches!(voice.piece, DrumPiece::ClosedHat | DrumPiece::OpenHat) {
                    voice.release_fast();
                }
            }
        }

        self.age_counter = self.age_counter.wrapping_add(1);
        let idx = self
            .voices
            .iter()
            .position(|v| !v.active)
            .unwrap_or_else(|| {
                self.voices
                    .iter()
                    .enumerate()
                    .min_by_key(|(_, v)| v.age)
                    .map(|(i, _)| i)
                    .unwrap_or(0)
            });
        self.voices[idx].trigger(hit, self.sample_rate as f32, self.age_counter);
    }
}

fn schedule_rock(
    step: u8,
    offset: u32,
    energy: f32,
    complexity: f32,
    onset_boost: f32,
    events: &mut [DrumHit; MAX_EVENTS_PER_BLOCK],
    count: &mut usize,
) {
    schedule_hats(step, offset, energy, complexity, events, count);

    if matches!(step, 0 | 8) || (complexity > 0.45 && matches!(step, 6 | 10)) {
        push_hit(
            events,
            count,
            offset,
            DrumPiece::Kick,
            0.55 + energy * 0.40 + onset_boost * 0.2,
        );
    }
    if matches!(step, 4 | 12) {
        push_hit(
            events,
            count,
            offset,
            DrumPiece::Snare,
            0.58 + energy * 0.35,
        );
    } else if complexity > 0.62 && matches!(step, 3 | 11 | 15) {
        push_hit(
            events,
            count,
            offset,
            DrumPiece::Snare,
            0.14 + energy * 0.16,
        );
    }
    if onset_boost > 0.22 && !matches!(step, 4 | 12) && step % 2 == 0 {
        push_hit(
            events,
            count,
            offset,
            DrumPiece::Kick,
            0.34 + onset_boost * 0.35,
        );
    }
}

fn schedule_half_time(
    step: u8,
    offset: u32,
    energy: f32,
    complexity: f32,
    onset_boost: f32,
    events: &mut [DrumHit; MAX_EVENTS_PER_BLOCK],
    count: &mut usize,
) {
    schedule_hats(step, offset, energy * 0.9, complexity, events, count);

    if matches!(step, 0 | 6 | 14) || (complexity > 0.55 && step == 10) {
        push_hit(
            events,
            count,
            offset,
            DrumPiece::Kick,
            0.55 + energy * 0.38 + onset_boost * 0.2,
        );
    }
    if step == 8 {
        push_hit(
            events,
            count,
            offset,
            DrumPiece::Snare,
            0.62 + energy * 0.35,
        );
    } else if complexity > 0.70 && matches!(step, 7 | 15) {
        push_hit(
            events,
            count,
            offset,
            DrumPiece::Snare,
            0.13 + energy * 0.15,
        );
    }
}

fn schedule_four_on_floor(
    step: u8,
    offset: u32,
    energy: f32,
    complexity: f32,
    _onset_boost: f32,
    events: &mut [DrumHit; MAX_EVENTS_PER_BLOCK],
    count: &mut usize,
) {
    schedule_hats(step, offset, energy, complexity.max(0.45), events, count);

    if step % 4 == 0 {
        push_hit(events, count, offset, DrumPiece::Kick, 0.62 + energy * 0.34);
    }
    if matches!(step, 4 | 12) {
        push_hit(
            events,
            count,
            offset,
            DrumPiece::Snare,
            0.52 + energy * 0.35,
        );
    }
    if complexity > 0.68 && matches!(step, 2 | 10) {
        push_hit(
            events,
            count,
            offset,
            DrumPiece::OpenHat,
            0.22 + energy * 0.24,
        );
    }
}

fn schedule_hats(
    step: u8,
    offset: u32,
    energy: f32,
    complexity: f32,
    events: &mut [DrumHit; MAX_EVENTS_PER_BLOCK],
    count: &mut usize,
) {
    let dense = complexity > 0.52;
    if dense || step % 2 == 0 {
        let open = complexity > 0.78 && matches!(step, 6 | 14);
        push_hit(
            events,
            count,
            offset,
            if open {
                DrumPiece::OpenHat
            } else {
                DrumPiece::ClosedHat
            },
            0.16 + energy * if dense { 0.22 } else { 0.18 },
        );
    }
}

fn push_hit(
    events: &mut [DrumHit; MAX_EVENTS_PER_BLOCK],
    count: &mut usize,
    offset: u32,
    piece: DrumPiece,
    velocity: f32,
) {
    if *count >= MAX_EVENTS_PER_BLOCK {
        return;
    }
    events[*count] = DrumHit {
        piece,
        articulation: Articulation::Center,
        velocity: velocity.clamp(0.0, 1.0),
        offset_frames: offset,
    };
    *count += 1;
}

fn jitter_samples(_step: u64, _complexity: f32, _sample_rate: u32) -> i32 {
    // Keep v0.1 sample-accurate and locked. Humanized microtiming can come
    // back later behind an explicit feel control; random timing variance made
    // the first playable build feel unstable.
    0
}

fn soft_clip(x: f32) -> f32 {
    (x * 1.4).tanh() * 0.8
}

#[derive(Clone, Copy)]
struct Voice {
    active: bool,
    piece: DrumPiece,
    velocity: f32,
    age_samples: u32,
    max_samples: u32,
    phase: f32,
    noise_state: u32,
    filter_state: f32,
    pan: f32,
    sample_rate: f32,
    age: u64,
    fast_release: bool,
}

impl Voice {
    const fn idle() -> Self {
        Self {
            active: false,
            piece: DrumPiece::Kick,
            velocity: 0.0,
            age_samples: 0,
            max_samples: 1,
            phase: 0.0,
            noise_state: 0x1234_5678,
            filter_state: 0.0,
            pan: 0.0,
            sample_rate: 48_000.0,
            age: 0,
            fast_release: false,
        }
    }

    fn trigger(&mut self, hit: DrumHit, sample_rate: f32, age: u64) {
        self.active = true;
        self.piece = hit.piece;
        self.velocity = hit.velocity.clamp(0.0, 1.0);
        self.age_samples = 0;
        self.max_samples = duration_samples(hit.piece, sample_rate);
        self.phase = 0.0;
        self.noise_state = (age as u32)
            .wrapping_mul(747_796_405)
            .wrapping_add(2_891_336_453);
        self.filter_state = 0.0;
        self.pan = pan_for_piece(hit.piece);
        self.sample_rate = sample_rate;
        self.age = age;
        self.fast_release = false;
    }

    fn release_fast(&mut self) {
        self.fast_release = true;
        self.max_samples = self
            .age_samples
            .saturating_add((self.sample_rate * 0.018) as u32);
    }

    fn next_sample(&mut self) -> f32 {
        if !self.active {
            return 0.0;
        }

        let t = self.age_samples as f32 / self.sample_rate;
        let life = self.age_samples as f32 / self.max_samples.max(1) as f32;
        if life >= 1.0 {
            self.active = false;
            return 0.0;
        }

        let out = match self.piece {
            DrumPiece::Kick => self.render_kick(t),
            DrumPiece::Snare => self.render_snare(t, life),
            DrumPiece::ClosedHat => self.render_hat(life, false),
            DrumPiece::OpenHat => self.render_hat(life, true),
            DrumPiece::Ride => self.render_hat(life, true) * 0.7,
            DrumPiece::Crash => self.render_crash(life),
            DrumPiece::TomLow => self.render_tom(t, life, 105.0),
            DrumPiece::TomMid => self.render_tom(t, life, 145.0),
            DrumPiece::TomHigh => self.render_tom(t, life, 190.0),
        };

        self.age_samples = self.age_samples.saturating_add(1);
        out * self.velocity
    }

    fn render_kick(&mut self, t: f32) -> f32 {
        let env = (-t * 10.5).exp();
        let pitch_env = (-t * 34.0).exp();
        let freq = 42.0 + 96.0 * pitch_env;
        self.phase += TAU * freq / self.sample_rate;
        if self.phase > TAU {
            self.phase -= TAU;
        }
        let click = if self.age_samples < 52 {
            (1.0 - self.age_samples as f32 / 52.0) * 0.35 * self.noise()
        } else {
            0.0
        };
        self.phase.sin() * env * 1.25 + click
    }

    fn render_snare(&mut self, t: f32, life: f32) -> f32 {
        let noise_env = (1.0 - life).powf(2.5);
        let body_env = (-t * 18.0).exp();
        self.phase += TAU * 185.0 / self.sample_rate;
        let body = self.phase.sin() * body_env * 0.48;
        let n = self.highpass_noise(0.72) * noise_env * 0.95;
        body + n
    }

    fn render_hat(&mut self, life: f32, open: bool) -> f32 {
        let env = if open {
            (1.0 - life).powf(1.7)
        } else {
            (1.0 - life).powf(5.5)
        };
        let metallic = self.highpass_noise(0.90) + self.square_partial(421.0) * 0.16;
        metallic * env * 0.55
    }

    fn render_crash(&mut self, life: f32) -> f32 {
        let env = (1.0 - life).powf(1.2);
        (self.highpass_noise(0.82) + self.square_partial(317.0) * 0.12) * env * 0.75
    }

    fn render_tom(&mut self, _t: f32, life: f32, freq: f32) -> f32 {
        let env = (1.0 - life).powf(2.2);
        self.phase += TAU * freq / self.sample_rate;
        if self.phase > TAU {
            self.phase -= TAU;
        }
        self.phase.sin() * env * 0.9 + self.noise() * env * 0.04
    }

    fn square_partial(&mut self, freq: f32) -> f32 {
        let p = (self.age_samples as f32 * freq / self.sample_rate).fract();
        if p < 0.5 {
            1.0
        } else {
            -1.0
        }
    }

    fn highpass_noise(&mut self, amount: f32) -> f32 {
        let n = self.noise();
        self.filter_state += (n - self.filter_state) * (1.0 - amount).clamp(0.01, 0.95);
        n - self.filter_state
    }

    fn noise(&mut self) -> f32 {
        self.noise_state = self
            .noise_state
            .wrapping_mul(1_664_525)
            .wrapping_add(1_013_904_223);
        let v = (self.noise_state >> 8) as f32 / 16_777_216.0;
        v * 2.0 - 1.0
    }
}

fn duration_samples(piece: DrumPiece, sample_rate: f32) -> u32 {
    let secs = match piece {
        DrumPiece::Kick => 0.72,
        DrumPiece::Snare => 0.42,
        DrumPiece::ClosedHat => 0.11,
        DrumPiece::OpenHat => 0.62,
        DrumPiece::Ride => 1.2,
        DrumPiece::Crash => 1.8,
        DrumPiece::TomLow => 0.75,
        DrumPiece::TomMid => 0.62,
        DrumPiece::TomHigh => 0.52,
    };
    (sample_rate * secs) as u32
}

fn pan_for_piece(piece: DrumPiece) -> f32 {
    match piece {
        DrumPiece::Kick => 0.0,
        DrumPiece::Snare => -0.06,
        DrumPiece::ClosedHat => 0.34,
        DrumPiece::OpenHat => 0.40,
        DrumPiece::Ride => 0.48,
        DrumPiece::Crash => -0.46,
        DrumPiece::TomLow => -0.24,
        DrumPiece::TomMid => 0.08,
        DrumPiece::TomHigh => 0.28,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn engine_renders_audio_when_playing() {
        let mut engine = Engine::new();
        engine.prepare(48_000, 512);
        let mut output = [0.0f32; 1024];
        engine.process(
            ClockSnapshot {
                sample_pos: 0,
                sample_rate: 48_000,
                bpm: 110.0,
                playing: true,
            },
            FollowInput::default(),
            &mut output,
            2,
        );
        assert!(output.iter().any(|s| s.abs() > 0.0001));
    }

    #[test]
    fn engine_is_silent_when_stopped_and_idle() {
        let mut engine = Engine::new();
        let mut output = [0.0f32; 512];
        engine.process(
            ClockSnapshot {
                sample_pos: 0,
                sample_rate: 48_000,
                bpm: 110.0,
                playing: false,
            },
            FollowInput::default(),
            &mut output,
            2,
        );
        assert!(output.iter().all(|s| *s == 0.0));
    }
}
