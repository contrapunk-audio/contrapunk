# Phase 6: Humanization - Research

**Researched:** 2026-01-29
**Domain:** MIDI real-time humanization (timing jitter, velocity variation, groove)
**Confidence:** HIGH

## Summary

Humanization applies small random variations to MIDI note parameters (timing, velocity, duration) and rhythmic shifts (swing/groove) to make computer-generated notes feel more human. This is a well-understood domain in MIDI processing with established patterns.

The existing codebase already has `rand 0.8` for random harmony modes, and the harmony engine produces notes synchronously in the router loop. The main architectural challenge is **timing jitter** (HUM-01), which requires delaying note sends by 5-30ms. Currently, harmony notes are sent immediately in `handle_note_on_gui()`. Delayed sends require either a scheduled event queue or `thread::sleep` on a background thread.

**Primary recommendation:** Create a `Humanizer` struct that sits between the harmony engine output and MIDI output. It transforms velocity/duration immediately and queues timing-delayed notes via a priority queue drained in the router loop.

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| rand | 0.8 (already in Cargo.toml) | Random number generation for jitter/variation | Already used by harmony engine |
| std::collections::BinaryHeap | stdlib | Priority queue for scheduled delayed notes | No external dep needed |
| std::time::Instant | stdlib | Timestamps for delayed note scheduling | No external dep needed |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| rand_distr | 0.4 | Normal/Gaussian distributions for more natural randomness | Optional: if uniform random sounds too mechanical |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| BinaryHeap scheduler | crossbeam channel with timer | More complex, unnecessary for <30ms delays |
| rand uniform | rand_distr Normal | Gaussian feels more natural but uniform is fine for 5-30ms range |

**Installation:**
```bash
# No new dependencies required. rand 0.8 already present.
# Optional: cargo add rand_distr@0.4
```

## Architecture Patterns

### Recommended Project Structure
```
src/
├── humanize/
│   ├── mod.rs           # pub mod, re-exports
│   ├── config.rs        # HumanizeConfig, HumanizeParams enums
│   ├── engine.rs        # Humanizer struct - transforms notes
│   └── scheduler.rs     # DelayQueue for timing jitter
├── harmony/             # existing
├── midi/                # existing
└── router.rs            # modified to integrate Humanizer
```

### Pattern 1: Post-Harmony Transform Pipeline
**What:** Humanizer sits after HarmonyEngine, before MIDI output. It receives `(Note, Velocity, Channel)` tuples and returns `HumanizedNote` with modified velocity, timing offset, and duration delta.
**When to use:** Always -- this is the core pattern.
**Example:**
```rust
pub struct HumanizedNote {
    pub note: Note,
    pub channel: Channel,
    pub velocity: Velocity,
    /// Delay before sending this note (0 = immediate)
    pub delay_ms: u16,
    /// Duration adjustment in ms (positive = longer, negative = shorter)
    pub duration_delta_ms: i16,
    /// Port index for output routing
    pub port: usize,
}

pub struct HumanizeConfig {
    pub enabled: bool,
    /// Timing jitter range in ms (e.g., 5..30)
    pub jitter_range: (u16, u16),
    pub jitter_enabled: bool,
    /// Velocity variation range (e.g., 10..20)
    pub velocity_variation: u8,
    pub velocity_enabled: bool,
    /// Duration variation in ms
    pub duration_variation_ms: u16,
    pub duration_enabled: bool,
    /// Swing amount 0.0 (straight) to 1.0 (full triplet swing)
    pub swing_amount: f32,
    pub swing_enabled: bool,
}
```

### Pattern 2: Delayed Note Queue in Router Loop
**What:** The router loop checks a time-ordered queue each iteration and sends notes whose scheduled time has passed. Jittered notes get pushed with `Instant::now() + delay`.
**When to use:** For timing jitter (HUM-01).
**Example:**
```rust
use std::collections::BinaryHeap;
use std::cmp::Reverse;
use std::time::Instant;

struct ScheduledNote {
    send_at: Instant,
    note: HumanizedNote,
}

// In BinaryHeap<Reverse<ScheduledNote>> ordered by send_at
// Router loop drains ready notes each iteration:
while let Some(Reverse(scheduled)) = queue.peek() {
    if scheduled.send_at <= Instant::now() {
        let scheduled = queue.pop().unwrap().0;
        // send to output
    } else {
        break;
    }
}
```

### Pattern 3: Swing via Beat Position Detection
**What:** Swing shifts off-beat notes later in time. Requires knowing the beat position, which in a real-time MIDI processor without a clock means using note onset timing to infer beat grid, OR simply applying swing as an additional delay to every other note.
**When to use:** For HUM-04 (swing/groove).
**Example:**
```rust
/// Simple swing: alternate notes get delayed.
/// Track a note counter; odd-numbered notes get swing delay.
pub struct SwingState {
    note_counter: u64,
    /// Swing delay in ms for off-beat notes (e.g., 20-50ms)
    swing_delay_ms: u16,
}

impl SwingState {
    pub fn get_swing_delay(&mut self) -> u16 {
        self.note_counter += 1;
        if self.note_counter % 2 == 0 {
            self.swing_delay_ms
        } else {
            0
        }
    }
}
```

### Anti-Patterns to Avoid
- **thread::sleep in the router loop:** Never block the router thread. Use a scheduled queue instead.
- **Applying humanization to the melody (input note):** Only humanize harmony notes (indices 1+). The original note should pass through unchanged.
- **Humanizing Note-Off independently from Note-On:** Note-Off velocity/timing should match what was decided at Note-On time. Store the humanization decisions.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Random number generation | Custom PRNG | `rand::thread_rng()` / `rand::Rng::gen_range()` | Already in the project, well-tested |
| MIDI velocity clamping | Manual if/else chains | `velocity.saturating_add()` / `.min(127).max(1)` | MIDI velocity is 1-127, not 0-127 (0 = note-off) |
| Priority queue | Sorted Vec | `BinaryHeap<Reverse<_>>` | O(log n) insert vs O(n) for sorted vec |

**Key insight:** The humanization logic itself is simple arithmetic. The complexity is in the scheduling (delayed sends) and state tracking (matching Note-Off to humanized Note-On).

## Common Pitfalls

### Pitfall 1: Velocity 0 Means Note-Off
**What goes wrong:** Randomizing velocity to 0 sends a Note-Off instead of a quiet Note-On.
**Why it happens:** MIDI spec: NoteOn with velocity 0 = NoteOff.
**How to avoid:** Clamp humanized velocity to range 1..=127. Never allow 0.
**Warning signs:** Notes cutting off randomly when velocity variation is enabled.

### Pitfall 2: Note-Off Timing Must Account for Jitter
**What goes wrong:** Note-On is delayed by 20ms but Note-Off fires at original time, shortening the note.
**Why it happens:** Jitter applied to Note-On but not propagated to Note-Off.
**How to avoid:** Store the jitter delay applied to each Note-On. Apply the same delay to the corresponding Note-Off. The existing `active_notes` HashMap in HarmonyEngine is the model -- humanization needs a similar `active_humanization` map.
**Warning signs:** Notes sound clipped or shortened when jitter is enabled.

### Pitfall 3: Duration Variation on Note-Off
**What goes wrong:** Duration variation requires delaying Note-Off relative to Note-On, but the router only processes messages as they arrive.
**Why it happens:** MIDI Note-Off is triggered by the player releasing the key. To extend/shorten duration, the app must intercept Note-Off and delay or advance it.
**How to avoid:** For duration extension: delay the Note-Off by `duration_delta_ms`. For shortening: send Note-Off early (schedule it at Note-On time + shortened duration). The delay queue handles both cases.
**Warning signs:** Stuck notes if Note-Off delay is lost.

### Pitfall 4: WASM Has No std::time::Instant
**What goes wrong:** `Instant::now()` is not available on `wasm32` target (or behaves differently).
**Why it happens:** WASM doesn't have monotonic clock access the same way.
**How to avoid:** Use `web_sys::window().performance().now()` on WASM, `Instant::now()` on native. Abstract behind a `fn now_ms() -> f64` helper with `#[cfg]` branches. Alternatively, since the WASM build uses frame-based processing already, track frame timestamps.
**Warning signs:** Compilation failure on wasm32 target.

### Pitfall 5: Swing Without Beat Clock
**What goes wrong:** Swing needs to know which notes are "on-beat" vs "off-beat" but there's no MIDI clock.
**Why it happens:** Real-time MIDI input has no inherent beat grid.
**How to avoid:** Use a simple alternating counter approach (every other harmony note gets swing delay). This is musically imperfect but functional. A more advanced approach would use inter-onset-interval detection, but that's over-engineering for this phase.
**Warning signs:** Swing sounds wrong when playing non-eighth-note patterns.

## Code Examples

### Velocity Variation
```rust
use rand::Rng;
use wmidi::Velocity;

fn humanize_velocity(velocity: Velocity, variation: u8) -> Velocity {
    let v = u8::from(velocity);
    let mut rng = rand::thread_rng();
    let delta: i16 = rng.gen_range(-(variation as i16)..=(variation as i16));
    let new_v = (v as i16 + delta).clamp(1, 127) as u8;
    // Safety: clamped to 1..=127, which is valid for Velocity
    Velocity::try_from(new_v).unwrap_or(velocity)
}
```

### Timing Jitter
```rust
use rand::Rng;

fn compute_jitter_ms(min_ms: u16, max_ms: u16) -> u16 {
    let mut rng = rand::thread_rng();
    rng.gen_range(min_ms..=max_ms)
}
```

### GUI Slider for Humanization Parameter
```rust
// In egui update():
ui.label("Timing Jitter (ms):");
ui.add(egui::Slider::new(&mut config.jitter_range.1, 0..=50).text("max"));

ui.label("Velocity Variation:");
ui.add(egui::Slider::new(&mut config.velocity_variation, 0..=30).text("range"));

ui.label("Swing:");
ui.add(egui::Slider::new(&mut config.swing_amount, 0.0..=1.0).text("amount"));
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Fixed random ranges | Per-parameter configurable ranges with GUI | Standard practice | Users can tune feel |
| Uniform distribution | Gaussian/triangular distribution | Always available in rand_distr | More natural clustering around center value |

**Note:** For this phase, uniform distribution is perfectly adequate. Gaussian is a nice-to-have that can be added later.

## Open Questions

1. **Should humanization apply to melody (input) notes too?**
   - What we know: Requirements say "harmony notes" but users might want global humanization
   - Recommendation: Only humanize harmony notes (skip index 0) for Phase 6. Add melody humanization as future option.

2. **Duration variation implementation complexity**
   - What we know: Shortening duration requires sending Note-Off early, which means scheduling a future Note-Off at Note-On time
   - What's unclear: How to handle the case where the player releases the key before the shortened duration expires
   - Recommendation: Only support duration extension (delay Note-Off), not shortening. Simpler and avoids race conditions.

3. **WASM compatibility for delayed sends**
   - What we know: WASM frame-based loop runs at ~60fps (16ms per frame). Jitter of 5-30ms means some delays are sub-frame.
   - Recommendation: On WASM, round delays to nearest frame. 16ms granularity is acceptable for humanization feel.

## Sources

### Primary (HIGH confidence)
- Existing codebase analysis (router.rs, engine.rs, app.rs, Cargo.toml)
- Rust std library documentation (BinaryHeap, Instant)
- wmidi 4.0 crate API (Velocity range 1-127)

### Secondary (MEDIUM confidence)
- MIDI specification: NoteOn velocity 0 = NoteOff (well-established standard)
- rand 0.8 API: `gen_range()`, `thread_rng()`

### Tertiary (LOW confidence)
- WASM Instant behavior -- needs verification during implementation

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - no new dependencies needed, all patterns use existing crate ecosystem
- Architecture: HIGH - clear insertion point in router between engine output and MIDI send
- Pitfalls: HIGH - based on direct code analysis of router.rs and MIDI spec knowledge

**Research date:** 2026-01-29
**Valid until:** 2026-03-01 (stable domain, no fast-moving dependencies)
