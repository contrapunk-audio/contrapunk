/**
 * Shared QWERTY keyboard input for the harmony engine.
 *
 * Same key map used by the desktop app's "Computer Keyboard" virtual
 * input AND the website embed's `<ContrapunkEngine />` bootstrap.
 * One source of truth so we don't drift between contexts.
 *
 *   Z X C V B N M  + S D G H J  → lower octave (white + black)
 *   Q W E R T Y U I + 2 3 5 6 7 → upper octave
 *   + / -                        → shift octave up / down
 *
 * Skips when focus is in <input>, <textarea>, or contenteditable so
 * pages with form fields keep working. Releases held notes on detach
 * so the synth doesn't ring forever.
 */

const LOWER_KEYS: Record<string, number> = {
	z: 0, s: 1, x: 2, d: 3, c: 4,
	v: 5, g: 6, b: 7, h: 8, n: 9,
	j: 10, m: 11
};
const UPPER_KEYS: Record<string, number> = {
	q: 0, '2': 1, w: 2, '3': 3, e: 4,
	r: 5, '5': 6, t: 7, '6': 8, y: 9,
	'7': 10, u: 11, i: 12
};
const MIN_OCTAVE = 1;
const MAX_OCTAVE = 7;

export interface KeyboardInputOptions {
	/** Initial octave (lower-row C lives at this octave). Default 3 → C3. */
	baseOctave?: number;
	/** Called whenever the user presses a mapped key for the first time
	 *  (key-repeat events are filtered). */
	onNoteOn: (midi: number) => void;
	/** Called when the user releases a previously-pressed key. */
	onNoteOff: (midi: number) => void;
	/** Optional: called whenever the octave shifts via +/- so the host
	 *  can show the current octave. */
	onOctaveChange?: (octave: number) => void;
	/** Optional gate. Return false to ignore the keydown — useful for
	 *  the desktop app where input is gated on the user picking the
	 *  Computer-Keyboard virtual MIDI input. */
	enabled?: () => boolean;
}

/**
 * Wire keydown/keyup listeners on `window`. Returns a detach function;
 * call it on unmount to drop the listeners and release any sustained
 * notes. Safe to call repeatedly — each call attaches a fresh pair.
 */
export function attachKeyboardInput(opts: KeyboardInputOptions): () => void {
	if (typeof window === 'undefined') return () => {};

	let octave = opts.baseOctave ?? 3;
	const heldKeys = new Map<string, number>();

	function keyToMidi(key: string): number | null {
		let midi: number;
		if (key in LOWER_KEYS) midi = (octave + 1) * 12 + LOWER_KEYS[key];
		else if (key in UPPER_KEYS) midi = (octave + 2) * 12 + UPPER_KEYS[key];
		else return null;
		if (midi < 0 || midi > 127) return null;
		return midi;
	}

	function isFormField(t: EventTarget | null): boolean {
		return (
			t instanceof HTMLInputElement ||
			t instanceof HTMLTextAreaElement ||
			(t instanceof HTMLElement && t.isContentEditable)
		);
	}

	function onKeyDown(e: KeyboardEvent) {
		if (opts.enabled && !opts.enabled()) return;
		if (isFormField(e.target)) return;
		const key = e.key.toLowerCase();
		if (key === '=' || key === '+') {
			if (octave < MAX_OCTAVE) {
				octave += 1;
				opts.onOctaveChange?.(octave);
			}
			e.preventDefault();
			return;
		}
		if (key === '-') {
			if (octave > MIN_OCTAVE) {
				octave -= 1;
				opts.onOctaveChange?.(octave);
			}
			e.preventDefault();
			return;
		}
		const midi = keyToMidi(key);
		if (midi === null) return;
		if (heldKeys.has(key)) return; // ignore key repeat
		heldKeys.set(key, midi);
		opts.onNoteOn(midi);
		e.preventDefault();
	}

	function onKeyUp(e: KeyboardEvent) {
		const key = e.key.toLowerCase();
		const midi = heldKeys.get(key);
		if (midi === undefined) return;
		heldKeys.delete(key);
		opts.onNoteOff(midi);
	}

	window.addEventListener('keydown', onKeyDown);
	window.addEventListener('keyup', onKeyUp);

	return () => {
		window.removeEventListener('keydown', onKeyDown);
		window.removeEventListener('keyup', onKeyUp);
		for (const midi of heldKeys.values()) opts.onNoteOff(midi);
		heldKeys.clear();
	};
}
