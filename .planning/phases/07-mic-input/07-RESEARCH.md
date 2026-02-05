# Phase 7: Mic Input - Research

**Researched:** 2026-02-05
**Domain:** Audio Capture, Pitch Detection, Audio-to-MIDI Conversion
**Confidence:** HIGH

## Summary

This phase implements microphone audio capture with real-time pitch detection to convert monophonic audio into MIDI notes for the harmony engine. The research covers three domains: (1) audio capture using the `cpal` crate, which is the standard Rust audio I/O library with cross-platform support including WASM; (2) pitch detection using the `pitch-detection` crate's YIN algorithm, which provides the best balance of accuracy and performance for monophonic voice/melody detection; and (3) the architecture pattern for integrating audio processing with the existing egui/eframe GUI using lock-free ring buffers for thread communication.

The key challenges are achieving low latency (target 30-40ms) while maintaining accurate pitch detection, handling vibrato smoothing without false note changes, and providing clear visual feedback for detected pitch and confidence levels. The existing codebase architecture (router thread pattern with `Arc<Mutex<GUIRouterState>>`) provides a proven template for this integration.

**Primary recommendation:** Use `cpal` 0.15+ for audio capture and `pitch-detection` crate's YIN detector with 1024-2048 sample buffer at 44.1kHz for optimal latency/accuracy tradeoff. Communicate between audio thread and main thread using `rtrb` lock-free ring buffer.

## Standard Stack

The established libraries/tools for this domain:

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| cpal | 0.15+ | Cross-platform audio I/O | De facto Rust audio library, supports all platforms including WASM via web-sys |
| pitch-detection | 0.3+ | Pitch detection algorithms | Provides YIN, McLeod, and autocorrelation detectors with clarity scores |
| rtrb | 0.3+ | Lock-free ring buffer | Wait-free SPSC buffer designed specifically for real-time audio |
| wmidi | 4.0 | MIDI note conversion | Already used in project, provides Note::from_u8_lossy() |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| web-sys | 0.3+ | WASM audio APIs | Already in project, add features for AudioContext, MediaStream |
| dasp | 0.11+ | Digital audio signal processing | If sample format conversion needed (f32<->i16) |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| pitch-detection | pyin | pYIN gives voiced probability but heavier, better for offline analysis |
| rtrb | ringbuf/direct_ring_buffer | rtrb specifically designed for audio real-time constraints |
| cpal | rodio | rodio is higher-level, less control over buffer sizes needed for low latency |

**Installation:**
```bash
cargo add cpal pitch-detection rtrb
# For WASM, add web-sys features in Cargo.toml
```

**Cargo.toml additions:**
```toml
[dependencies]
cpal = "0.15"
pitch-detection = "0.3"
rtrb = "0.3"

[target.'cfg(target_arch = "wasm32")'.dependencies.web-sys]
version = "0.3"
features = [
    # Existing features plus:
    "AudioContext", "MediaStream", "MediaStreamAudioSourceNode",
    "MediaDevices", "MediaStreamConstraints", "AnalyserNode",
    "GainNode", "ScriptProcessorNode"
]
```

## Architecture Patterns

### Recommended Project Structure
```
src/
├── audio/
│   ├── mod.rs           # Module exports, feature gates
│   ├── capture.rs       # Native cpal audio capture (cfg not wasm32)
│   ├── web.rs           # WASM getUserMedia audio capture
│   ├── pitch.rs         # Pitch detection wrapper (YIN + freq-to-MIDI)
│   ├── config.rs        # MicConfig struct (thresholds, buffer size, etc)
│   └── profiles.rs      # MicProfile save/load
├── midi/                # Existing MIDI module
├── harmony/             # Existing harmony engine
└── app.rs               # Add MicState to ContrapunkApp
```

### Pattern 1: Audio Thread to GUI Communication
**What:** Audio callback runs on high-priority thread, pushes samples to lock-free ring buffer, main thread drains and processes.
**When to use:** Always for audio capture - audio callbacks must be non-blocking.
**Example:**
```rust
// Source: Adapted from rtrb documentation + existing router.rs pattern
use rtrb::{RingBuffer, Producer, Consumer};

// In audio module
pub struct AudioCapture {
    producer: Producer<f32>,
    stream: cpal::Stream,
}

// In main app
pub struct MicState {
    consumer: Consumer<f32>,
    pitch_detector: PitchDetector,
    detected_pitch: Option<DetectedPitch>,
    current_note: Option<wmidi::Note>,
}

// Audio callback (runs on audio thread)
fn audio_callback(data: &[f32], producer: &mut Producer<f32>) {
    // MUST be non-blocking - just copy samples
    let chunk = producer.write_chunk(data.len()).unwrap_or_default();
    chunk.fill_from_iter(data.iter().copied());
}

// Main thread processing (in update())
fn process_audio_samples(mic_state: &mut MicState) {
    let mut buffer = vec![0.0f32; 2048];
    while let Ok(chunk) = mic_state.consumer.read_chunk(buffer.len()) {
        chunk.read_into(&mut buffer);
        if let Some(pitch) = mic_state.pitch_detector.detect(&buffer) {
            mic_state.detected_pitch = Some(pitch);
        }
    }
}
```

### Pattern 2: Frequency to MIDI Note Conversion
**What:** Convert detected frequency (Hz) to nearest MIDI note number with cents deviation.
**When to use:** After pitch detection returns a frequency.
**Example:**
```rust
// Source: MIDI tuning standard (Wikipedia)
pub struct DetectedPitch {
    pub frequency: f32,
    pub midi_note: u8,
    pub cents_deviation: i8,  // -50 to +50
    pub clarity: f32,         // 0.0 to 1.0
}

pub fn freq_to_midi_note(freq: f32) -> DetectedPitch {
    // MIDI note = 69 + 12 * log2(freq / 440)
    let midi_float = 69.0 + 12.0 * (freq / 440.0).log2();
    let midi_note = midi_float.round() as u8;
    let cents_deviation = ((midi_float - midi_note as f32) * 100.0) as i8;

    DetectedPitch {
        frequency: freq,
        midi_note: midi_note.clamp(0, 127),
        cents_deviation: cents_deviation.clamp(-50, 50),
        clarity: 0.0, // Set by caller from pitch detector
    }
}

// Voice range filtering (C2=36 to C6=84)
pub fn is_in_voice_range(midi_note: u8) -> bool {
    midi_note >= 36 && midi_note <= 84
}
```

### Pattern 3: Vibrato Smoothing with Hysteresis
**What:** Track center pitch, ignore small oscillations, require sustained deviation to trigger note change.
**When to use:** After pitch detection to prevent rapid note flickering during vibrato.
**Example:**
```rust
// Source: Cycfi Research pitch detection articles
pub struct NoteTracker {
    current_note: Option<u8>,
    note_start_time: f64,
    stability_window_ms: f64,  // e.g., 50ms
    hysteresis_cents: i8,      // e.g., 40 cents
}

impl NoteTracker {
    pub fn update(&mut self, pitch: &DetectedPitch, now_ms: f64) -> Option<NoteEvent> {
        match self.current_note {
            None => {
                // No current note - start tracking
                self.current_note = Some(pitch.midi_note);
                self.note_start_time = now_ms;
                Some(NoteEvent::NoteOn(pitch.midi_note))
            }
            Some(current) if current != pitch.midi_note => {
                // Different note detected
                let cents_diff = ((pitch.midi_note as i16 - current as i16) * 100
                    + pitch.cents_deviation as i16).abs();

                // Only change if beyond hysteresis threshold
                if cents_diff > self.hysteresis_cents as i16 * 2 {
                    let old = current;
                    self.current_note = Some(pitch.midi_note);
                    self.note_start_time = now_ms;
                    Some(NoteEvent::NoteChange(old, pitch.midi_note))
                } else {
                    None // Within hysteresis, ignore
                }
            }
            _ => None, // Same note, no change
        }
    }
}
```

### Pattern 4: Raw Audio Buffer for Vocoder (Phase 8)
**What:** Maintain a circular buffer of raw audio samples accessible by future vocoder phase.
**When to use:** When capturing audio, store both for pitch detection AND raw passthrough.
**Example:**
```rust
// Source: Design for Phase 8 integration
pub struct AudioBuffer {
    samples: Vec<f32>,
    write_pos: usize,
    capacity: usize,  // e.g., 4096 samples = ~93ms at 44.1kHz
}

impl AudioBuffer {
    pub fn push_samples(&mut self, data: &[f32]) {
        for &sample in data {
            self.samples[self.write_pos] = sample;
            self.write_pos = (self.write_pos + 1) % self.capacity;
        }
    }

    /// Get last N samples for vocoder (Phase 8)
    pub fn get_recent(&self, count: usize) -> Vec<f32> {
        let mut result = Vec::with_capacity(count);
        let start = (self.write_pos + self.capacity - count) % self.capacity;
        for i in 0..count {
            result.push(self.samples[(start + i) % self.capacity]);
        }
        result
    }
}
```

### Anti-Patterns to Avoid
- **Blocking in audio callback:** Never use Mutex::lock(), channel send(), or allocations in the audio callback - use lock-free primitives only
- **Processing pitch in audio thread:** Move pitch detection to main thread; audio callback should only copy samples
- **Single-sample pitch detection:** YIN needs at least 1024 samples; accumulate in buffer before detection
- **Ignoring clarity threshold:** Always check pitch detector's clarity score before using result
- **Hard note transitions:** Use hysteresis to prevent rapid note changes during vibrato

## Don't Hand-Roll

Problems that look simple but have existing solutions:

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Pitch detection | Simple zero-crossing or autocorrelation | pitch-detection crate YIN | Octave errors, noise handling, edge cases |
| Audio capture | Raw CoreAudio/WASAPI bindings | cpal | Cross-platform complexity, buffer management |
| Ring buffer for audio | std::collections::VecDeque | rtrb | VecDeque isn't lock-free, will cause audio glitches |
| Sample rate conversion | Manual interpolation | dasp crate | Aliasing, quality issues |
| MIDI note names | Hardcoded string array | wmidi Note display | Already have it, handles edge cases |

**Key insight:** Audio programming has subtle timing requirements. Lock-free data structures, proper buffer sizing, and battle-tested algorithms are essential - custom implementations will have glitches that are hard to debug.

## Common Pitfalls

### Pitfall 1: Audio Callback Blocking
**What goes wrong:** Audio dropouts, clicks, and glitches when the callback takes too long.
**Why it happens:** Mutex locks, memory allocation, or complex processing in the audio callback.
**How to avoid:** Audio callback should ONLY copy samples to a lock-free buffer. All processing happens on main thread.
**Warning signs:** Intermittent audio crackling, especially under CPU load.

### Pitfall 2: Octave Errors in Pitch Detection
**What goes wrong:** Detected pitch jumps an octave up or down randomly.
**Why it happens:** YIN algorithm can lock onto harmonics instead of fundamental frequency.
**How to avoid:**
1. Filter to voice range (C2-C6) before converting to MIDI
2. Use clarity threshold (e.g., 0.7) to reject uncertain detections
3. Apply hysteresis to prevent sudden octave jumps
**Warning signs:** Notes randomly jumping 12 semitones while singing steady pitch.

### Pitfall 3: Vibrato Triggering Multiple Notes
**What goes wrong:** Sustained note with vibrato produces rapid Note-On/Off stream.
**Why it happens:** Pitch deviation during vibrato crosses note boundary repeatedly.
**How to avoid:**
1. Hysteresis threshold (e.g., 40 cents) before accepting note change
2. Minimum note duration before allowing change (e.g., 50ms)
3. Track "center pitch" over moving window
**Warning signs:** Watching the MIDI output shows rapid note changes during vibrato.

### Pitfall 4: Latency Accumulation
**What goes wrong:** Total latency exceeds 50ms target, making real-time play feel sluggish.
**Why it happens:** Multiple buffers in the chain each add latency.
**How to avoid:**
1. Use small buffer size (512-1024 samples = 11-23ms at 44.1kHz)
2. Process pitch detection every frame, not on buffer fill
3. Overlap-add for smoother detection
4. Display actual measured latency to user
**Warning signs:** Noticeable delay between singing and harmony response.

### Pitfall 5: WASM Audio Permission Denied
**What goes wrong:** getUserMedia fails silently or with cryptic error on web.
**Why it happens:** Browser requires user gesture before microphone access, HTTPS required.
**How to avoid:**
1. Request mic permission only after user clicks a button
2. Handle rejection gracefully with clear error message
3. Ensure site is served over HTTPS (localhost exempt)
**Warning signs:** WASM build works locally but fails on deployed site.

### Pitfall 6: Sample Format Mismatch
**What goes wrong:** Pitch detection returns garbage or crashes.
**Why it happens:** Audio device provides i16 samples but detector expects f32.
**How to avoid:** Check `SupportedStreamConfig::sample_format()` and convert as needed using dasp or manual conversion.
**Warning signs:** Detected frequencies are wildly wrong or NaN.

## Code Examples

Verified patterns from official sources:

### cpal Audio Input Setup
```rust
// Source: cpal documentation (docs.rs/cpal)
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

pub fn setup_audio_capture() -> Result<(cpal::Stream, Consumer<f32>)> {
    let host = cpal::default_host();
    let device = host.default_input_device()
        .ok_or_else(|| anyhow::anyhow!("No input device available"))?;

    // Get supported config
    let config = device.default_input_config()?;
    let sample_rate = config.sample_rate().0;

    // Create ring buffer (4096 samples = ~93ms at 44.1kHz)
    let (mut producer, consumer) = RingBuffer::<f32>::new(4096);

    // Build stream with appropriate sample format
    let stream = match config.sample_format() {
        cpal::SampleFormat::F32 => device.build_input_stream(
            &config.into(),
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                // Non-blocking push to ring buffer
                if let Ok(mut chunk) = producer.write_chunk(data.len()) {
                    chunk.fill_from_iter(data.iter().copied());
                }
            },
            |err| eprintln!("Audio error: {}", err),
            None,
        )?,
        cpal::SampleFormat::I16 => device.build_input_stream(
            &config.into(),
            move |data: &[i16], _: &cpal::InputCallbackInfo| {
                // Convert i16 to f32 and push
                let floats: Vec<f32> = data.iter()
                    .map(|&s| s as f32 / i16::MAX as f32)
                    .collect();
                if let Ok(mut chunk) = producer.write_chunk(floats.len()) {
                    chunk.fill_from_iter(floats.into_iter());
                }
            },
            |err| eprintln!("Audio error: {}", err),
            None,
        )?,
        _ => return Err(anyhow::anyhow!("Unsupported sample format")),
    };

    stream.play()?;
    Ok((stream, consumer))
}
```

### YIN Pitch Detection
```rust
// Source: pitch-detection documentation
use pitch_detection::detector::yin::YINDetector;
use pitch_detection::detector::PitchDetector;

pub struct PitchAnalyzer {
    detector: YINDetector<f32>,
    sample_rate: usize,
    buffer: Vec<f32>,
    power_threshold: f32,
    clarity_threshold: f32,
}

impl PitchAnalyzer {
    pub fn new(sample_rate: usize) -> Self {
        // Buffer size of 2048 at 44100Hz gives ~46ms window
        // Good for detecting down to ~43Hz (lowest voice)
        let size = 2048;
        let padding = size / 2;

        Self {
            detector: YINDetector::new(size, padding),
            sample_rate,
            buffer: Vec::with_capacity(size),
            power_threshold: 5.0,    // Minimum signal power
            clarity_threshold: 0.7,  // Confidence threshold
        }
    }

    pub fn add_samples(&mut self, samples: &[f32]) {
        self.buffer.extend_from_slice(samples);
    }

    pub fn detect(&mut self) -> Option<DetectedPitch> {
        if self.buffer.len() < 2048 {
            return None;
        }

        // Process oldest 2048 samples
        let analysis_buffer: Vec<f64> = self.buffer[..2048]
            .iter()
            .map(|&s| s as f64)
            .collect();

        // Remove processed samples (keep overlap for continuity)
        self.buffer.drain(..1024);

        // Run pitch detection
        let pitch = self.detector.get_pitch(
            &analysis_buffer,
            self.sample_rate,
            self.power_threshold as f64,
            self.clarity_threshold as f64,
        )?;

        // Convert to our struct
        let mut result = freq_to_midi_note(pitch.frequency as f32);
        result.clarity = pitch.clarity as f32;

        // Filter to voice range
        if !is_in_voice_range(result.midi_note) {
            return None;
        }

        Some(result)
    }
}
```

### WASM Audio Capture (getUserMedia)
```rust
// Source: web-sys documentation + Toptal WebAssembly tutorial
#[cfg(target_arch = "wasm32")]
pub async fn request_mic_access() -> Result<web_sys::MediaStream, JsValue> {
    let window = web_sys::window().ok_or("no window")?;
    let navigator = window.navigator();
    let media_devices = navigator.media_devices()?;

    // Create constraints for audio only
    let constraints = web_sys::MediaStreamConstraints::new();
    constraints.set_audio(&JsValue::TRUE);
    constraints.set_video(&JsValue::FALSE);

    let promise = media_devices.get_user_media_with_constraints(&constraints)?;
    let stream = wasm_bindgen_futures::JsFuture::from(promise).await?;

    stream.dyn_into::<web_sys::MediaStream>()
}

#[cfg(target_arch = "wasm32")]
pub fn setup_audio_processing(
    stream: &web_sys::MediaStream
) -> Result<(web_sys::AudioContext, web_sys::AnalyserNode), JsValue> {
    let ctx = web_sys::AudioContext::new()?;
    let source = ctx.create_media_stream_source(stream)?;

    // Create analyser for getting audio data
    let analyser = ctx.create_analyser()?;
    analyser.set_fft_size(2048);

    source.connect_with_audio_node(&analyser)?;

    Ok((ctx, analyser))
}
```

### Level Meter Calculation
```rust
// Source: Standard audio RMS calculation
pub fn calculate_rms_level(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    let sum_squares: f32 = samples.iter().map(|&s| s * s).sum();
    (sum_squares / samples.len() as f32).sqrt()
}

pub fn rms_to_db(rms: f32) -> f32 {
    if rms > 0.0 {
        20.0 * rms.log10()
    } else {
        -60.0 // Floor
    }
}

// For UI level meter (0.0 to 1.0 range)
pub fn level_for_display(rms: f32, floor_db: f32, ceiling_db: f32) -> f32 {
    let db = rms_to_db(rms);
    ((db - floor_db) / (ceiling_db - floor_db)).clamp(0.0, 1.0)
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| FFT-based pitch | Time-domain YIN/pYIN | 2002 (YIN paper) | More accurate, fewer octave errors |
| Blocking audio APIs | Callback-based async | cpal 0.8+ | Required for real-time, no glitches |
| std::sync::mpsc for audio | Lock-free ring buffers | Always for real-time | Prevents audio dropout |
| Single buffer detection | Overlap-add | Standard practice | Smoother tracking, lower latency |

**Deprecated/outdated:**
- **FFT peak detection:** Simple but unreliable for voice; use YIN instead
- **Zero-crossing detection:** Only works for simple waveforms, not voice
- **ScriptProcessorNode (Web):** Deprecated in Web Audio API, use AudioWorklet for new projects (but ScriptProcessor still works and is simpler)

## Open Questions

Things that couldn't be fully resolved:

1. **cpal WASM audio input support**
   - What we know: cpal supports WASM output via wasm-bindgen feature, input is experimental
   - What's unclear: Whether cpal's WASM input is production-ready or if we need raw web-sys
   - Recommendation: Start with web-sys getUserMedia directly for WASM (proven pattern), cpal for native

2. **Optimal buffer size for voice**
   - What we know: Larger buffer = better accuracy, smaller = lower latency
   - What's unclear: Exact sweet spot for typical voice characteristics
   - Recommendation: Default 2048 samples (~46ms), make user-adjustable via latency slider

3. **Audio Worklet for WASM low-latency**
   - What we know: AudioWorklet provides lower latency than ScriptProcessorNode
   - What's unclear: Complexity of Rust/WASM AudioWorklet integration
   - Recommendation: Start with simpler approach (ScriptProcessorNode or AnalyserNode), optimize later if needed

## Sources

### Primary (HIGH confidence)
- [cpal GitHub](https://github.com/RustAudio/cpal) - Audio I/O library documentation
- [cpal docs.rs](https://docs.rs/cpal/latest/cpal/) - API reference for input streams, buffer configuration
- [pitch-detection docs.rs](https://docs.rs/pitch-detection) - YIN detector API, PitchDetector trait
- [rtrb GitHub](https://github.com/mgeier/rtrb) - Lock-free ring buffer for real-time audio
- [MIDI tuning standard (Wikipedia)](https://en.wikipedia.org/wiki/MIDI_tuning_standard) - Frequency to MIDI formula

### Secondary (MEDIUM confidence)
- [Toptal WebAssembly Rust Audio Tutorial](https://www.toptal.com/developers/webassembly/webassembly-rust-tutorial-web-audio) - WASM audio processing patterns
- [Cycfi Research - Fast Pitch Detection](https://www.cycfi.com/2017/10/fast-and-efficient-pitch-detection/) - Hysteresis and vibrato handling
- [cpal issue #970](https://github.com/RustAudio/cpal/issues/970) - Windows callback + ring buffer issue (now fixed)
- [egui Discussion #1428](https://github.com/emilk/egui/discussions/1428) - Thread/atomic usage with egui

### Tertiary (LOW confidence)
- WebSearch results for pitch detection latency tradeoffs - general guidance, verify with testing
- WebSearch results for WASM getUserMedia - browser APIs change frequently, verify current behavior

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - cpal and pitch-detection are well-established, active maintenance
- Architecture: HIGH - Pattern matches existing router.rs, verified with cpal/egui docs
- Pitfalls: MEDIUM - Based on documentation and forum discussions, needs validation in implementation

**Research date:** 2026-02-05
**Valid until:** 60 days (audio libraries are stable, patterns well-established)
