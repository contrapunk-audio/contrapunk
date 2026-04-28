/**
 * Pattern Store — Beat-Aligned Chord Trigger Pattern (Svelte 5 Runes)
 *
 * Owns the rhythmic pattern that decides which beats trigger harmony
 * playback when the BPM-aware Performance feature is active. The store
 * exposes the pattern shape (subdivision, length, cells) and the
 * input-quantization mode (live / quantized / gated). The router thread
 * polls this state per beat tick to decide whether to fire a NoteOn for
 * the user's currently-held inputs.
 *
 * State persists to localStorage so a user's pattern survives reload.
 *
 * Cell-count math:
 *   cellCount = subdivision × beatsPerBar × length
 * where `subdivision` is cells-per-beat (1=quarter, 2=eighth, 4=16th,
 * 8=32nd), `beatsPerBar` comes from the transport's time signature, and
 * `length` is bars per pattern loop. Default 4 × 4 × 1 = 16 cells (one
 * bar of 16th notes in 4/4).
 */

const STORAGE_KEY = 'contrapunk-pattern';

/** Cells per beat. 1=quarter, 2=eighth, 4=16th, 8=32nd. */
export type Subdivision = 1 | 2 | 4 | 8;

/** Bars per pattern loop. */
export type Length = 1 | 2 | 4 | 8;

/** How input notes interact with the pattern.
 *
 * - `live`: input plays as performed; harmony fires only on pattern-active beats.
 * - `quantized`: input + harmony both snap to the next pattern beat.
 * - `gated`: input + harmony continuous, NoteOff harmony on pattern-off cells.
 */
export type InputMode = 'live' | 'quantized' | 'gated';

export const SUBDIVISION_OPTIONS: { value: Subdivision; label: string }[] = [
	{ value: 1, label: 'Quarter' },
	{ value: 2, label: 'Eighth' },
	{ value: 4, label: '16th' },
	{ value: 8, label: '32nd' }
];

export const LENGTH_OPTIONS: { value: Length; label: string }[] = [
	{ value: 1, label: '1 bar' },
	{ value: 2, label: '2 bars' },
	{ value: 4, label: '4 bars' },
	{ value: 8, label: '8 bars' }
];

export const INPUT_MODE_OPTIONS: { value: InputMode; label: string }[] = [
	{ value: 'live', label: 'Live' },
	{ value: 'quantized', label: 'Quantized' },
	{ value: 'gated', label: 'Gated' }
];

interface PersistedPattern {
	cells: boolean[];
	subdivision: number;
	length: number;
	inputMode: string;
}

const VALID_SUBDIVISIONS: Set<number> = new Set([1, 2, 4, 8]);
const VALID_LENGTHS: Set<number> = new Set([1, 2, 4, 8]);
const VALID_INPUT_MODES: Set<string> = new Set(['live', 'quantized', 'gated']);

class PatternStore {
	cells = $state<boolean[]>([]);
	subdivision = $state<Subdivision>(4);
	length = $state<Length>(1);
	inputMode = $state<InputMode>('live');

	/** Time-signature numerator. Updated from transport via `setBeatsPerBar`
	 *  on app init / time-sig change. Default 4 covers the common case. */
	beatsPerBar = $state(4);

	constructor() {
		this.cells = new Array(this.cellCount).fill(true);
	}

	/** Total number of cells in the pattern loop. */
	get cellCount(): number {
		return this.subdivision * this.beatsPerBar * this.length;
	}

	setSubdivision(s: Subdivision) {
		if (s === this.subdivision) return;
		this.subdivision = s;
		this.resizeCells();
		this.persist();
	}

	setLength(l: Length) {
		if (l === this.length) return;
		this.length = l;
		this.resizeCells();
		this.persist();
	}

	setInputMode(m: InputMode) {
		if (m === this.inputMode) return;
		this.inputMode = m;
		this.persist();
	}

	/** Update the time-signature numerator. Called from a transport-watching
	 *  effect at app init or when the user changes time signature. Resizes
	 *  cells preserving as much of the user's pattern as fits. */
	setBeatsPerBar(bpb: number) {
		if (bpb === this.beatsPerBar || !Number.isFinite(bpb) || bpb < 1) return;
		this.beatsPerBar = Math.max(1, Math.floor(bpb));
		this.resizeCells();
	}

	toggleCell(idx: number) {
		if (idx < 0 || idx >= this.cells.length) return;
		const next = this.cells.slice();
		next[idx] = !next[idx];
		this.cells = next;
		this.persist();
	}

	setCell(idx: number, value: boolean) {
		if (idx < 0 || idx >= this.cells.length) return;
		if (this.cells[idx] === value) return;
		const next = this.cells.slice();
		next[idx] = value;
		this.cells = next;
		this.persist();
	}

	clear() {
		this.cells = new Array(this.cellCount).fill(false);
		this.persist();
	}

	fillAll() {
		this.cells = new Array(this.cellCount).fill(true);
		this.persist();
	}

	/** Reset to a sensible default. Currently "all cells on", which matches
	 *  pre-pattern engine behavior (chord fires on every beat). */
	resetDefault() {
		this.fillAll();
	}

	/** Compute the cell index playing at a given total-beats position
	 *  (monotonic global counter from the transport). Wraps at the
	 *  pattern's loop length. Used by the UI to highlight the
	 *  currently-playing cell, and by the router to decide whether
	 *  to trigger a chord. */
	cellIndexAt(totalBeats: number): number {
		const beatsPerLoop = this.beatsPerBar * this.length;
		if (beatsPerLoop <= 0) return 0;
		const positionInLoop = ((totalBeats % beatsPerLoop) + beatsPerLoop) % beatsPerLoop;
		const idx = Math.floor(positionInLoop * this.subdivision);
		const total = this.cellCount;
		if (total <= 0) return 0;
		return ((idx % total) + total) % total;
	}

	/** Whether the pattern triggers at the given total-beats position. */
	isActiveAt(totalBeats: number): boolean {
		const idx = this.cellIndexAt(totalBeats);
		return !!this.cells[idx];
	}

	private resizeCells() {
		const target = this.cellCount;
		if (target === this.cells.length) return;
		const next = new Array(target).fill(true);
		const copyLen = Math.min(this.cells.length, target);
		for (let i = 0; i < copyLen; i++) next[i] = this.cells[i];
		this.cells = next;
	}

	private persist() {
		try {
			localStorage.setItem(
				STORAGE_KEY,
				JSON.stringify({
					cells: this.cells,
					subdivision: this.subdivision,
					length: this.length,
					inputMode: this.inputMode
				})
			);
		} catch {
			/* localStorage unavailable */
		}
	}

	/** Hydrate from localStorage. Call on app init. */
	restore() {
		if (typeof window === 'undefined') return;
		try {
			const raw = localStorage.getItem(STORAGE_KEY);
			if (!raw) return;
			const parsed = JSON.parse(raw) as PersistedPattern;
			if (typeof parsed !== 'object' || parsed === null) return;

			if (typeof parsed.subdivision === 'number' && VALID_SUBDIVISIONS.has(parsed.subdivision)) {
				this.subdivision = parsed.subdivision as Subdivision;
			}
			if (typeof parsed.length === 'number' && VALID_LENGTHS.has(parsed.length)) {
				this.length = parsed.length as Length;
			}
			if (typeof parsed.inputMode === 'string' && VALID_INPUT_MODES.has(parsed.inputMode)) {
				this.inputMode = parsed.inputMode as InputMode;
			}

			// Resize to match restored subdivision/length/beatsPerBar before
			// applying persisted cells, so cellCount is right when we copy.
			this.resizeCells();
			if (Array.isArray(parsed.cells) && parsed.cells.length === this.cellCount) {
				this.cells = parsed.cells.map((c) => !!c);
			}
		} catch {
			/* localStorage unavailable or corrupt */
		}
	}
}

export const pattern = new PatternStore();
