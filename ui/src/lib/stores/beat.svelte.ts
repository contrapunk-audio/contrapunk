/**
 * Beat-clock store driven by the WASM Engine's tick() loop.
 *
 * The WasmAdapter runs a `requestAnimationFrame` loop that calls
 * `engine.tick(performance.now())` every frame and funnels the result
 * into this store. Components that need to visualize the beat (the
 * status-bar `BeatIndicator`, future metronome LEDs, etc.) subscribe
 * here instead of owning their own interval timer.
 *
 * On native (Tauri) the same data is emitted via Rust events; a future
 * bridge in `TauriAdapter` can populate the same fields so UI code
 * stays platform-agnostic.
 */
export const beat = $state({
	/** Beat-phase position within the bar (0.0 .. beats_per_bar). */
	beatPosition: 0,
	/** Integer beat number (0-indexed) currently active. */
	beatNumber: 0,
	/** Beats per bar (e.g. 4 for 4/4). */
	beatsPerBar: 4,
	/** Flipped each time a beat boundary is crossed — components can
	 *  `$effect` on `pulse` to trigger flashes without manual timers. */
	pulse: 0,
	/** Monotonic timestamp (performance.now()) of the last beat crossing. */
	lastCrossedAt: 0,
	/** Whether the WASM beat clock is currently running. */
	running: false,
	/** Current tempo in BPM. */
	bpm: 120
});
