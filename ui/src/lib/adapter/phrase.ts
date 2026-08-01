import type { PhraseState } from './types';

export function mapPhraseState(raw: unknown): PhraseState {
	const state = raw !== null && typeof raw === 'object' ? raw as Record<string, unknown> : {};
	const nullableNumber = (value: unknown) => typeof value === 'number' ? value : null;
	return {
		id: nullableNumber(state.id),
		phase: state.phase === 'opening' || state.phase === 'active' || state.phase === 'releasing'
			? state.phase
			: 'idle',
		gapBeats: typeof state.gap_beats === 'number' ? state.gap_beats : 2,
		startedAt: nullableNumber(state.started_at),
		releaseStartedAt: nullableNumber(state.release_started_at),
		attackCount: typeof state.attack_count === 'number' ? state.attack_count : 0,
		openingNote: nullableNumber(state.opening_note),
		previousNote: nullableNumber(state.previous_note),
		latestNote: nullableNumber(state.latest_note),
		latestVelocity: nullableNumber(state.latest_velocity),
		latestChannel: nullableNumber(state.latest_channel),
		inputIdle: state.input_idle !== false
	};
}
