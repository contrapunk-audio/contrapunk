<script lang="ts">
	/**
	 * AUTOPLAY scale walker (#42). Pure UI — no backend changes.
	 *
	 * Toggle button that, while active, walks up the current scale's
	 * notes at a fixed tempo using the existing
	 * `adapter.injectNoteOn` / `adapter.injectNoteOff` MIDI-injection
	 * commands. The user hears the engine harmonize each note exactly
	 * as if they were playing it on a controller.
	 *
	 * Source of truth for which notes to walk is
	 * `engine.inScaleNotes` (derived from key + scale mode in the
	 * engine store). We restrict the walk to one octave starting at
	 * the configured `octaveBase` (default 60 = C4) so the autoplay
	 * sits in a musically useful register regardless of the engine's
	 * voice_position / octave_mode.
	 *
	 * Tempo is fixed at 4 notes/sec for v0 — research scoped this as
	 * "XS, no Rust changes". Tempo and direction controls are a
	 * follow-up.
	 */
	import { onDestroy } from 'svelte';
	import { adapter } from '$lib/adapter';
	import { engine } from '$lib/stores/engine.svelte';

	let { octaveBase = 60 }: { octaveBase?: number } = $props();

	const NOTE_INTERVAL_MS = 250; // 4 notes/sec
	const NOTE_HOLD_MS = 200; // shorter than interval so legato-ish gaps stay open

	let running = $state(false);
	let intervalId: ReturnType<typeof setInterval> | null = null;
	let walkIndex = $state(0);
	let lastPlayedNote: number | null = null;

	/** Notes from the current scale that sit in [octaveBase, octaveBase+12). */
	function notesInWalkRange(): number[] {
		return engine.inScaleNotes.filter((n) => n >= octaveBase && n < octaveBase + 12);
	}

	async function tick() {
		const notes = notesInWalkRange();
		if (notes.length === 0) {
			return;
		}
		// Release the previously-played note (if any) before sounding
		// the next one. Without this, sustained pile-up would render
		// the chord engine output as polyphonic stack.
		if (lastPlayedNote !== null) {
			try {
				await adapter.injectNoteOff(lastPlayedNote);
			} catch (err) {
				// Routing may have stopped between ticks; swallow.
				console.warn('[autoplay] noteOff failed', err);
			}
			lastPlayedNote = null;
		}
		const next = notes[walkIndex % notes.length];
		walkIndex = (walkIndex + 1) % notes.length;
		try {
			await adapter.injectNoteOn(next, 100);
			lastPlayedNote = next;
			// Release after NOTE_HOLD_MS for a non-legato feel.
			setTimeout(async () => {
				if (lastPlayedNote === next) {
					try {
						await adapter.injectNoteOff(next);
					} catch {
						/* same swallow */
					}
					lastPlayedNote = null;
				}
			}, NOTE_HOLD_MS);
		} catch (err) {
			console.warn('[autoplay] noteOn failed', err);
		}
	}

	function start() {
		if (running) return;
		running = true;
		walkIndex = 0;
		// Fire immediately so the user hears something on click,
		// then settle into the interval cadence.
		tick();
		intervalId = setInterval(tick, NOTE_INTERVAL_MS);
	}

	function stop() {
		running = false;
		if (intervalId !== null) {
			clearInterval(intervalId);
			intervalId = null;
		}
		if (lastPlayedNote !== null) {
			adapter.injectNoteOff(lastPlayedNote).catch(() => {
				/* swallow */
			});
			lastPlayedNote = null;
		}
	}

	function toggle() {
		if (running) {
			stop();
		} else {
			start();
		}
	}

	// Stop on unmount so the interval doesn't outlive the component.
	onDestroy(() => {
		stop();
	});
</script>

<button
	type="button"
	class="autoplay-btn"
	class:running
	onclick={toggle}
	data-testid="autoplay-toggle"
	aria-pressed={running}
	title="Walk the current scale ascending at 4 notes/sec — routing must be running"
>
	{running ? 'AUTOPLAY ◼' : 'AUTOPLAY ▶'}
</button>

<style>
	.autoplay-btn {
		font-family: var(--font-ui), sans-serif;
		font-size: var(--font-size-xs);
		padding: 4px 12px;
		background: var(--color-widget-bg);
		border: 1px solid var(--color-border);
		color: var(--color-text-primary);
		cursor: pointer;
		border-radius: 0;
		min-width: 90px;
		text-align: center;
	}

	.autoplay-btn:hover {
		background: var(--color-widget-hover, var(--color-widget-bg));
	}

	.autoplay-btn.running {
		background: var(--color-accent-teal);
		border-color: var(--color-accent-cyan);
		box-shadow: var(--glow-teal);
		color: #ffffff;
	}
</style>
