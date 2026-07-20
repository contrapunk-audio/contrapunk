## Review
- **Medium:** `ui/src/lib/components/PerformanceView.svelte:188-189,277-285` — supported surfaces append `ExplicitIntervals` as mode index 5, but both MIDI-learn scaling and the knob remain capped at index 4. Consequently, Tauri/WASM users cannot select Interval Map from this control. Derive both from `MODE_TO_ENGINE.length - 1`.

Residual risk: no browser E2E test exercises edited-map NoteOn/NoteOff behavior; core lifecycle tests and generated WASM artifacts passed verification.