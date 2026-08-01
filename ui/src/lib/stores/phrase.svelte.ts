import { adapter } from '$lib/adapter';
import type { PhraseState } from '$lib/adapter/types';

const STORAGE_KEY = 'contrapunk-phrase-gap-beats';
const DEFAULT_GAP = 2;
const MIN_GAP = 0.5;
const MAX_GAP = 16;

const IDLE: PhraseState = {
	id: null,
	phase: 'idle',
	gapBeats: DEFAULT_GAP,
	startedAt: null,
	releaseStartedAt: null,
	attackCount: 0,
	openingNote: null,
	previousNote: null,
	latestNote: null,
	latestVelocity: null,
	latestChannel: null,
	inputIdle: true
};

class PhraseStore {
	state = $state<PhraseState>({ ...IDLE });
	loaded = $state(false);
	error = $state<string | null>(null);

	get gapBeats() { return this.state.gapBeats; }
	get phase() { return this.state.phase; }
	get id() { return this.state.id; }
	get statusLabel() {
		return {
			idle: 'Waiting',
			opening: 'Listening',
			active: 'Playing',
			releasing: 'Phrase gap'
		}[this.state.phase];
	}

	async init() {
		if (!adapter.capabilities.phraseContext) return;
		const saved = this.savedGap();
		try {
			if (saved !== null) await adapter.setPhraseGapBeats(saved);
			this.applySnapshot(await adapter.getPhraseState());
			this.loaded = true;
			this.error = null;
		} catch (error) {
			this.loaded = true;
			this.error = `Could not load phrase settings: ${error}`;
		}
	}

	applySnapshot(snapshot: PhraseState) {
		this.state = snapshot;
	}

	async setGapBeats(beats: number) {
		const next = clampGap(beats);
		const previous = this.state;
		this.state = { ...previous, gapBeats: next };
		try {
			await adapter.setPhraseGapBeats(next);
			try { localStorage.setItem(STORAGE_KEY, String(next)); } catch { /* persistence unavailable */ }
			this.error = null;
		} catch (error) {
			this.state = previous;
			this.error = `Could not change phrase gap: ${error}`;
		}
	}

	private savedGap(): number | null {
		try {
			const raw = Number(localStorage.getItem(STORAGE_KEY));
			return Number.isFinite(raw) && raw >= MIN_GAP && raw <= MAX_GAP ? raw : null;
		} catch {
			return null;
		}
	}
}

function clampGap(beats: number) {
	return Math.max(MIN_GAP, Math.min(MAX_GAP, Number.isFinite(beats) ? beats : DEFAULT_GAP));
}

export const phrase = new PhraseStore();
