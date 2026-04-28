/**
 * Performance-view MIDI Learn store.
 *
 * Owns the mapping from MIDI CC numbers (0-127) to Performance-view
 * software-knob indices (0-7). The mapping is persisted in localStorage
 * under `contrapunk-knob-cc-map`. On first run we seed the MPK Mini
 * defaults (knobs 1-8 → CC 70-77).
 *
 * The store also tracks which knob (if any) is currently in "learning"
 * state — when set, the next CC observed via `adapter.onKnobCcRaw`
 * binds to that knob and clears the learning flag.
 *
 * KnobIndex layout (matches PerformanceView grid):
 *   0 = Mode, 1 = Voices, 2 = Tightness, 3 = Adventurous,
 *   4 = Key,  5 = Scale,  6 = You play,  7 = Spread.
 */

export type KnobIndex = 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7;
export type KnobCcMap = Record<KnobIndex, number | null>;

const STORAGE_KEY = 'contrapunk-knob-cc-map';

/** MPK Mini MK3 default — knobs 1-8 send CC 70-77 in grid order. */
export const MPK_MINI_DEFAULT: KnobCcMap = {
	0: 70,
	1: 71,
	2: 72,
	3: 73,
	4: 74,
	5: 75,
	6: 76,
	7: 77
};

function cloneDefault(): KnobCcMap {
	return { ...MPK_MINI_DEFAULT };
}

function loadFromStorage(): KnobCcMap {
	if (typeof localStorage === 'undefined') return cloneDefault();
	try {
		const raw = localStorage.getItem(STORAGE_KEY);
		if (!raw) return cloneDefault();
		const parsed = JSON.parse(raw) as Partial<Record<string, number | null>>;
		const out = cloneDefault();
		for (let i = 0 as KnobIndex; i < 8; i = (i + 1) as KnobIndex) {
			const v = parsed[String(i)];
			if (v === null) {
				out[i] = null;
			} else if (typeof v === 'number' && v >= 0 && v <= 127) {
				out[i] = Math.round(v);
			}
			// otherwise keep default
		}
		return out;
	} catch {
		return cloneDefault();
	}
}

function saveToStorage(map: KnobCcMap): void {
	if (typeof localStorage === 'undefined') return;
	try {
		localStorage.setItem(STORAGE_KEY, JSON.stringify(map));
	} catch {
		// best-effort — quota / private mode
	}
}

class KnobMidiMapStore {
	/** CC mapping per knob (or null if unbound). */
	map = $state<KnobCcMap>(loadFromStorage());
	/** Knob index currently in MIDI Learn mode, or null. */
	learning = $state<KnobIndex | null>(null);

	/** Begin learning for a specific knob. Cancels any other in-progress
	 *  learn first. */
	startLearn(idx: KnobIndex): void {
		this.learning = idx;
	}

	/** Toggle learning state for a knob — clicking the same affordance
	 *  twice cancels. */
	toggleLearn(idx: KnobIndex): void {
		this.learning = this.learning === idx ? null : idx;
	}

	/** Cancel any in-progress learn (Escape key, click elsewhere). */
	cancelLearn(): void {
		this.learning = null;
	}

	/** Bind a CC number to a knob. If the same CC is already bound to
	 *  another knob, that other knob is unbound to keep the mapping
	 *  one-to-one. */
	bind(idx: KnobIndex, cc: number): void {
		const next: KnobCcMap = { ...this.map };
		for (let i = 0 as KnobIndex; i < 8; i = (i + 1) as KnobIndex) {
			if (next[i] === cc && i !== idx) next[i] = null;
		}
		next[idx] = cc;
		this.map = next;
		saveToStorage(next);
	}

	/** Clear a knob's binding. */
	unbind(idx: KnobIndex): void {
		const next: KnobCcMap = { ...this.map, [idx]: null };
		this.map = next;
		saveToStorage(next);
	}

	/** Resolve a CC number to its bound knob index, or null if unbound. */
	knobForCc(cc: number): KnobIndex | null {
		for (let i = 0 as KnobIndex; i < 8; i = (i + 1) as KnobIndex) {
			if (this.map[i] === cc) return i;
		}
		return null;
	}

	/** Restore the MPK Mini defaults — used by the "Reset to MPK Mini
	 *  defaults" button in Settings. */
	resetToMpkDefaults(): void {
		this.map = cloneDefault();
		this.learning = null;
		saveToStorage(this.map);
	}
}

export const knobMidiMap = new KnobMidiMapStore();
