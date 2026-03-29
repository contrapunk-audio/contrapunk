---
phase: 07-mic-input
plan: 05b
type: execute
wave: 3
depends_on: ["07-04", "07-05"]
files_modified:
  - src/app.rs
autonomous: true

must_haves:
  truths:
    - "Mic input works in WASM builds in the browser"
    - "getUserMedia permission request triggers on mic selection"
    - "WebAudioCapture processes audio samples each frame"
    - "Detected pitch notes flow through harmony engine in WASM"
  artifacts:
    - path: "src/app.rs"
      provides: "WASM mic capture and processing"
      contains: "WebAudioCapture"
  key_links:
    - from: "src/app.rs"
      to: "src/audio/web.rs"
      via: "WebAudioCapture for WASM capture"
      pattern: "WebAudioCapture|request_mic_access"
    - from: "src/app.rs"
      to: "harmony engine"
      via: "NoteEvent triggers harmonize in WASM"
      pattern: "handle_wasm_note_on|handle_wasm_note_off"
---

<objective>
Wire microphone input for WASM builds using WebAudioCapture, request_mic_access, and browser-specific frame processing.

Purpose: Completes the mic input feature for browser users. After this plan, users can select microphone in the web version and hear harmonized notes.

Output: Working mic-to-harmony pipeline for WASM builds.
</objective>

<execution_context>
@/Users/vibhavbobade/.claude/get-shit-done/workflows/execute-plan.md
@/Users/vibhavbobade/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/phases/07-mic-input/07-CONTEXT.md
@.planning/phases/07-mic-input/07-04-SUMMARY.md
@.planning/phases/07-mic-input/07-05-SUMMARY.md
@src/app.rs
@src/audio/web.rs
</context>

<tasks>

<task type="auto">
  <name>Task 1: Add WASM mic state fields and imports</name>
  <files>src/app.rs</files>
  <action>
Add WASM-specific fields and imports for mic capture:

1. Add WASM-specific imports at top of app.rs (with existing WASM imports):
```rust
#[cfg(target_arch = "wasm32")]
use crate::audio::{WebAudioCapture, request_mic_access, calculate_rms_web};
```

2. Add WASM-specific fields to MicState struct (if not already present):
```rust
/// WASM audio capture (WASM only)
#[cfg(target_arch = "wasm32")]
pub wasm_capture: Option<WebAudioCapture>,
/// Whether mic permission has been requested (WASM)
#[cfg(target_arch = "wasm32")]
pub permission_requested: bool,
/// Pending mic permission future (WASM)
#[cfg(target_arch = "wasm32")]
pub pending_permission: Option<std::pin::Pin<Box<dyn std::future::Future<Output = Result<web_sys::MediaStream, wasm_bindgen::JsValue>>>>>,
```

3. Initialize WASM fields in MicState::default():
```rust
#[cfg(target_arch = "wasm32")]
wasm_capture: None,
#[cfg(target_arch = "wasm32")]
permission_requested: false,
#[cfg(target_arch = "wasm32")]
pending_permission: None,
```
  </action>
  <verify>
`cargo check --features wasm` compiles.
  </verify>
  <done>
WASM-specific imports and MicState fields exist for WebAudioCapture.
  </done>
</task>

<task type="auto">
  <name>Task 2: Implement WASM mic capture start and frame processing</name>
  <files>src/app.rs</files>
  <action>
Add WASM-specific mic capture methods:

1. Add start_mic_capture_wasm() method:
```rust
#[cfg(target_arch = "wasm32")]
fn start_mic_capture_wasm(&mut self) {
    use wasm_bindgen_futures::spawn_local;

    // Request mic permission (this spawns an async task)
    self.mic_state.permission_status = Some("Requesting mic permission...".to_string());

    // We need to request mic access - this must be triggered by user gesture
    // The actual permission request happens asynchronously
    // Store a flag that we've requested permission
    self.mic_state.permission_requested = true;
}
```

2. Add the async permission handling in update() for WASM. Find the WASM update section and add:
```rust
#[cfg(target_arch = "wasm32")]
{
    // Handle mic permission request (must be in user gesture context)
    if self.mic_state.permission_requested && self.mic_state.wasm_capture.is_none() {
        // Check if we have a pending button click to trigger the request
        // The actual request is triggered by the Start button in try_start_wasm
    }
}
```

3. Update try_start for WASM to handle mic input - find the WASM try_start method and add mic handling:
```rust
// In try_start for WASM, add mic handling:
let is_mic_input = self.state.input_port == Some(INPUT_MICROPHONE);

if is_mic_input {
    // Spawn async task to request mic permission and start capture
    use wasm_bindgen_futures::spawn_local;
    use std::rc::Rc;
    use std::cell::RefCell;

    // Clone what we need for the async closure
    let mic_state = Rc::new(RefCell::new(std::mem::take(&mut self.mic_state)));
    let mic_state_clone = mic_state.clone();

    spawn_local(async move {
        match request_mic_access().await {
            Ok(stream) => {
                let buffer_size = mic_state.borrow().config.buffer_size;
                match WebAudioCapture::new(&stream, buffer_size) {
                    Ok(capture) => {
                        // Resume audio context (required after user gesture)
                        if let Err(e) = capture.resume().await {
                            mic_state.borrow_mut().permission_status =
                                Some(format!("Failed to start audio: {:?}", e));
                            return;
                        }

                        let sample_rate = capture.sample_rate() as usize;
                        let config = mic_state.borrow().config.clone();

                        let mut state = mic_state.borrow_mut();
                        state.pitch_detector = Some(PitchDetector::new(sample_rate, &config));
                        state.note_tracker = NoteTracker::new(
                            config.hysteresis_cents,
                            config.min_note_duration_ms,
                        );
                        state.wasm_capture = Some(capture);
                        state.is_active = true;
                        state.permission_status = Some("Mic active".to_string());
                    }
                    Err(e) => {
                        mic_state.borrow_mut().permission_status =
                            Some(format!("Failed to create capture: {:?}", e));
                    }
                }
            }
            Err(e) => {
                mic_state.borrow_mut().permission_status =
                    Some(format!("Mic permission denied: {:?}", e));
            }
        }
    });

    // Restore mic_state after spawn
    self.mic_state = Rc::try_unwrap(mic_state_clone).ok().unwrap().into_inner();
    return; // Don't proceed with normal MIDI setup for mic input
}
```

Note: The above async pattern is complex for borrowing. Alternative simpler approach - use a separate flag and poll in update():

4. Simpler approach - Add flag-based permission flow:
```rust
// In MicState, add:
#[cfg(target_arch = "wasm32")]
pub mic_permission_pending: bool,
#[cfg(target_arch = "wasm32")]
pub mic_stream: Option<web_sys::MediaStream>,

// In try_start for WASM with mic input:
if is_mic_input {
    // Trigger permission request - the actual async work happens via wasm_bindgen_futures
    self.mic_state.mic_permission_pending = true;
    self.mic_state.permission_status = Some("Click Start to grant mic permission".to_string());

    // Spawn the permission request
    use wasm_bindgen_futures::spawn_local;
    let window = web_sys::window().expect("window");

    // Use a JS promise callback approach instead
    spawn_local(async {
        match request_mic_access().await {
            Ok(stream) => {
                // Store stream for later use
                // This requires interior mutability - we'll handle in update()
                web_sys::console::log_1(&"Mic permission granted".into());
            }
            Err(e) => {
                web_sys::console::log_1(&format!("Mic denied: {:?}", e).into());
            }
        }
    });
}
```

5. Actually, the cleanest approach is to handle mic setup entirely in the WASM update loop after permission is granted. Add process_mic_frame_wasm():
```rust
#[cfg(target_arch = "wasm32")]
fn process_mic_frame_wasm(&mut self) {
    if !self.mic_state.is_active {
        return;
    }

    let Some(ref mut capture) = self.mic_state.wasm_capture else { return };
    let Some(ref mut detector) = self.mic_state.pitch_detector else { return };

    // Read samples from WebAudioCapture
    let samples = capture.read_samples_f32();

    if samples.is_empty() {
        return;
    }

    // Add samples to detector
    detector.add_samples(samples);

    // Update RMS for level meter
    self.mic_state.current_rms = calculate_rms_web(samples);

    // Try to detect pitch
    let pitch = detector.detect();
    self.mic_state.last_pitch = pitch.clone();

    // Get current time
    let now_ms = web_sys::window()
        .and_then(|w| w.performance())
        .map(|p| p.now())
        .unwrap_or(0.0);

    // Update note tracker
    if let Some(event) = self.mic_state.note_tracker.update(pitch.as_ref(), now_ms) {
        self.handle_mic_note_event_wasm(event);
    }
}

#[cfg(target_arch = "wasm32")]
fn handle_mic_note_event_wasm(&mut self, event: NoteEvent) {
    use wmidi::{Note, Channel, Velocity};

    let channel = Channel::Ch1;
    let velocity = Velocity::try_from(100u8).unwrap();

    match event {
        NoteEvent::NoteOn(midi) => {
            let note = Note::from_u8_lossy(midi);
            // Reuse existing WASM note handling which does humanization + output
            self.handle_wasm_note_on(channel, note, velocity);
        }
        NoteEvent::NoteOff(midi) => {
            let note = Note::from_u8_lossy(midi);
            self.handle_wasm_note_off(channel, note, Velocity::MIN);
        }
        NoteEvent::NoteChange { old, new } => {
            let old_note = Note::from_u8_lossy(old);
            self.handle_wasm_note_off(channel, old_note, Velocity::MIN);

            let new_note = Note::from_u8_lossy(new);
            self.handle_wasm_note_on(channel, new_note, velocity);
        }
    }
}
```

6. Wire into WASM update() loop - find the WASM-specific update section and add:
```rust
// Process mic input each frame (WASM)
#[cfg(target_arch = "wasm32")]
if self.mic_state.is_active && self.state.is_running {
    self.process_mic_frame_wasm();
}
```
  </action>
  <verify>
`cargo check --features wasm` compiles.
`trunk serve` runs, mic can be selected in browser.
  </verify>
  <done>
WASM mic capture uses WebAudioCapture and request_mic_access. process_mic_frame_wasm() processes samples each frame. handle_mic_note_event_wasm() reuses existing handle_wasm_note_on/off for harmony processing and output.
  </done>
</task>

</tasks>

<verification>
1. `cargo check --features wasm` compiles without errors
2. `trunk serve` runs without errors
3. Selecting mic in browser triggers permission request
4. After permission granted, level meter shows activity
5. Detected notes produce harmonized output in browser
</verification>

<success_criteria>
- WASM imports include WebAudioCapture, request_mic_access
- MicState has wasm_capture field for WASM builds
- start_mic_capture triggers getUserMedia permission request
- process_mic_frame_wasm() reads samples from WebAudioCapture
- handle_mic_note_event_wasm() reuses handle_wasm_note_on/off
- Mic works end-to-end in browser
</success_criteria>

<output>
After completion, create `.planning/phases/07-mic-input/07-05b-SUMMARY.md`
</output>
