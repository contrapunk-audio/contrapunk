import { adapter } from '$lib/adapter';
import type { CounterpointSpeciesName } from './engine.svelte';
import { engine } from './engine.svelte';
import {
	validateArrangementConfig,
	type ArrangementCapability,
	type ArrangementConfig,
	type ArrangementPatternConfig,
	type ArrangementPatternLaneConfig,
	type ArrangementPatternLaneId
} from '$lib/arrangement/presets';

export interface CounterpointLaneState {
	enabled: boolean;
	species: CounterpointSpeciesName;
	transposeDegrees: number;
	preferAbove: boolean;
}

const DEFAULT_COUNTERPOINT: CounterpointLaneState = {
	enabled: false,
	species: 'Species1',
	transposeDegrees: 2,
	preferAbove: true
};

const EMPTY_PATTERN_LANE: ArrangementPatternLaneConfig = {
	enabled: false,
	cycleBeats: 4,
	tailBeats: 4,
	events: []
};

function emptyPatterns(): ArrangementPatternConfig {
	return {
		lowSupport: { ...EMPTY_PATTERN_LANE, events: [] },
		counterline: { ...EMPTY_PATTERN_LANE, events: [] }
	};
}

class ArrangementStore {
	counterpoint = $state<CounterpointLaneState>({ ...DEFAULT_COUNTERPOINT });
	patterns = $state<ArrangementPatternConfig>(emptyPatterns());
	mixLevels = $state([1, 1, 1, 1]);
	mixLoaded = $state(false);
	applying = $state(false);

	get availableCapabilities(): ReadonlySet<ArrangementCapability> {
		const capabilities = new Set<ArrangementCapability>(['harmony', 'voice_leading']);
		if (adapter.capabilities.companionLanes) {
			capabilities.add('strict_canon');
			capabilities.add('free_imitation');
			capabilities.add('species_counterpoint');
		}
		if (adapter.capabilities.patternLanes) {
			capabilities.add('pattern_lane');
			capabilities.add('stable_lane_groups');
		}
		if (adapter.capabilities.roleMix) capabilities.add('role_mix');
		return capabilities;
	}

	async syncFromBackend() {
		try {
			const state = await adapter.counterpointState();
			if (state) {
				this.counterpoint = {
					enabled: state.enabled,
					species: state.species as CounterpointSpeciesName,
					transposeDegrees: state.transpose_degrees,
					preferAbove: state.prefer_above
				};
			}
		} catch {
			// Surface does not expose the dedicated Counterpoint lane.
		}

		for (const [role, laneId] of [
			['lowSupport', 'pattern_low'],
			['counterline', 'pattern_counter']
		] as const) {
			try {
				const state = await adapter.patternState(laneId);
				if (state) this.patterns[role] = patternFromWire(state);
			} catch {
				// Surface does not expose stable pattern roles.
			}
		}

		if (adapter.capabilities.roleMix) {
			try {
				const state = await adapter.getSynthState();
				if (state.mixGains?.length === 4) {
					this.mixLevels = state.mixGains.map((value) => clamp01(value));
				}
			} catch {
				// Keep unity defaults when the synth is not ready yet.
			}
		}
		this.mixLoaded = true;
	}

	async setPattern(
		role: keyof ArrangementPatternConfig,
		laneId: ArrangementPatternLaneId,
		config: ArrangementPatternLaneConfig
	) {
		const previous = this.patterns[role];
		this.patterns[role] = clonePattern(config);
		try {
			await adapter.patternConfigure(laneId, {
				enabled: config.enabled,
				cycle_beats: config.cycleBeats,
				tail_beats: config.tailBeats,
				events: config.events.map((event) => ({
					beat: event.beat,
					degree: event.degree,
					octave: event.octave,
					duration_beats: event.durationBeats,
					velocity: event.velocity
				}))
			});
		} catch (error) {
			this.patterns[role] = previous;
			throw error;
		}
	}

	async setCounterpoint(patch: Partial<CounterpointLaneState>) {
		const previous = this.counterpoint;
		const next = { ...previous, ...patch };
		this.counterpoint = next;
		try {
			await adapter.counterpointSetConfig({
				enabled: patch.enabled,
				species: patch.species,
				transpose_degrees: patch.transposeDegrees,
				prefer_above: patch.preferAbove
			});
		} catch (error) {
			this.counterpoint = previous;
			throw error;
		}
	}

	setMixLevel(group: number, value: number) {
		if (group < 0 || group >= 4) return;
		this.mixLevels[group] = clamp01(value);
		this.mixLevels = [...this.mixLevels];
	}

	async pushMixLevel(group: number, value = this.mixLevels[group] ?? 1) {
		if (!adapter.capabilities.roleMix || group < 0 || group >= 4) return;
		await adapter.setSynthMixGain(group, clamp01(value));
	}

	async apply(config: ArrangementConfig) {
		if (this.applying) throw new Error('An arrangement is already being applied');
		const errors = validateArrangementConfig(config);
		if (errors.length) throw new Error(errors.join('; '));

		await this.syncFromBackend();
		const previous = this.snapshot();
		this.applying = true;
		try {
			await adapter.panicAllNotesOff();
			await this.applyUnchecked(config);
		} catch (cause) {
			await adapter.panicAllNotesOff();
			try {
				await this.applyUnchecked(previous);
			} catch (rollbackCause) {
				throw new Error(`Arrangement apply failed: ${cause}; rollback failed: ${rollbackCause}`);
			}
			throw cause;
		} finally {
			this.applying = false;
		}
	}

	private async applyUnchecked(config: ArrangementConfig) {
		const harmony = config.harmony;
		await engine.setScaleMode(harmony.scaleMode);
		await engine.setMode(harmony.mode);
		await engine.setVoiceCount(harmony.voiceCount);
		await engine.setVoicePosition(harmony.voicePosition);
		await engine.setVoiceLeading(harmony.voiceLeadingEnabled, harmony.voiceLeadingStyle);
		await engine.setOctaveMode(harmony.octaveMode);
		await engine.setOctaveIntensity(harmony.octaveIntensity);
		await engine.setInterchange(harmony.interchangeEnabled, harmony.interchangeRange);
		await engine.setCounterpointSpecies(harmony.counterpointSpecies);
		await engine.setCounterpointStrictness(harmony.counterpointStrictness);

		const companion = config.companion;
		await engine.setCompanionHoldMode(companion.globalHoldMode);
		await engine.setImitativeForm(companion.canon.form);
		await engine.setCanonVoices(
			companion.canon.voices.map((voice) => ({
				delay_beats: voice.delayBeats,
				transpose_degrees: voice.transposeDegrees,
				time_ratio: voice.timeRatio,
				harmony_mode: voice.harmonyMode,
				reference_voice: voice.referenceVoice,
				voice_count: voice.voiceCount,
				voice_position: voice.voicePosition,
				voice_leading_enabled: voice.voiceLeadingEnabled,
				voice_leading_style: voice.voiceLeadingStyle,
				octave_mode: voice.octaveMode,
				counterpoint_species: voice.counterpointSpecies,
				counterpoint_strictness: voice.counterpointStrictness,
				hold_mode: voice.holdMode
			}))
		);
		await engine.setCanonLaneHoldMode(companion.canon.holdMode);
		await engine.setCanonEnabled(companion.canon.enabled);
		await this.setCounterpoint({
			enabled: companion.counterpoint.enabled,
			species: companion.counterpoint.species,
			transposeDegrees: companion.counterpoint.transposeDegrees,
			preferAbove: companion.counterpoint.preferAbove
		});
		await engine.setCounterpointLaneHoldMode(companion.counterpoint.holdMode);
		const patterns = companion.patterns ?? emptyPatterns();
		await this.setPattern('lowSupport', 'pattern_low', patterns.lowSupport);
		await this.setPattern('counterline', 'pattern_counter', patterns.counterline);
		await engine.setCompanionEnabled(companion.enabled);

		const mix = [config.mix.input, config.mix.harmony, config.mix.canon, config.mix.counterpoint];
		for (let group = 0; group < mix.length; group++) {
			this.setMixLevel(group, mix[group]);
			await this.pushMixLevel(group);
		}
	}

	/** Capture only musical arrangement state; performance environment is absent by contract. */
	snapshot(): ArrangementConfig {
		return {
			harmony: {
				scaleMode: engine.scaleMode,
				mode: engine.mode,
				voiceCount: engine.voiceCount,
				voicePosition: engine.voicePosition,
				voiceLeadingEnabled: engine.voiceLeadingEnabled,
				voiceLeadingStyle: engine.voiceLeadingStyle,
				octaveMode: engine.octaveMode,
				octaveIntensity: engine.octaveIntensity,
				interchangeEnabled: engine.interchangeEnabled,
				interchangeRange: engine.interchangeRange,
				counterpointSpecies: engine.counterpointSpecies,
				counterpointStrictness: engine.counterpointStrictness
			},
			companion: {
				enabled: engine.companionEnabled,
				globalHoldMode: engine.companionHoldMode,
				canon: {
					enabled: engine.canonEnabled,
					form: engine.imitativeForm,
					holdMode: engine.canonLaneHoldMode,
					voices: engine.canonVoices.map((voice) => ({
						delayBeats: voice.delay_beats,
						transposeDegrees: voice.transpose_degrees,
						timeRatio: voice.time_ratio,
						harmonyMode: (voice.harmony_mode as ArrangementConfig['harmony']['mode'] | null | undefined) ?? null,
						referenceVoice: voice.reference_voice ?? null,
						voiceCount: voice.voice_count ?? null,
						voicePosition: voice.voice_position ?? null,
						voiceLeadingEnabled: voice.voice_leading_enabled ?? null,
						voiceLeadingStyle: (voice.voice_leading_style as ArrangementConfig['harmony']['voiceLeadingStyle'] | null | undefined) ?? null,
						octaveMode: (voice.octave_mode as ArrangementConfig['harmony']['octaveMode'] | null | undefined) ?? null,
						counterpointSpecies: (voice.counterpoint_species as ArrangementConfig['harmony']['counterpointSpecies'] | null | undefined) ?? null,
						counterpointStrictness: (voice.counterpoint_strictness as ArrangementConfig['harmony']['counterpointStrictness'] | null | undefined) ?? null,
						holdMode: voice.hold_mode ?? null
					}))
				},
				counterpoint: {
					enabled: this.counterpoint.enabled,
					species: this.counterpoint.species,
					transposeDegrees: this.counterpoint.transposeDegrees,
					preferAbove: this.counterpoint.preferAbove,
					holdMode: engine.counterpointLaneHoldMode
				},
				patterns: {
					lowSupport: clonePattern(this.patterns.lowSupport),
					counterline: clonePattern(this.patterns.counterline)
				}
			},
			mix: {
				input: this.mixLevels[0] ?? 1,
				harmony: this.mixLevels[1] ?? 1,
				canon: this.mixLevels[2] ?? 1,
				counterpoint: this.mixLevels[3] ?? 1
			}
		};
	}
}

function clonePattern(config: ArrangementPatternLaneConfig): ArrangementPatternLaneConfig {
	return { ...config, events: config.events.map((event) => ({ ...event })) };
}

function patternFromWire(state: Record<string, unknown>): ArrangementPatternLaneConfig {
	return {
		enabled: state.enabled === true,
		cycleBeats: typeof state.cycle_beats === 'number' ? state.cycle_beats : 4,
		tailBeats: typeof state.tail_beats === 'number' ? state.tail_beats : 4,
		events: Array.isArray(state.events)
			? state.events.flatMap((value) => {
					if (typeof value !== 'object' || value === null) return [];
					const event = value as Record<string, unknown>;
					if (
						typeof event.beat !== 'number' ||
						typeof event.degree !== 'number' ||
						typeof event.octave !== 'number' ||
						typeof event.duration_beats !== 'number' ||
						typeof event.velocity !== 'number'
					) return [];
					return [{
						beat: event.beat,
						degree: event.degree,
						octave: event.octave,
						durationBeats: event.duration_beats,
						velocity: event.velocity
					}];
				})
			: []
	};
}

function clamp01(value: number): number {
	return Math.max(0, Math.min(1, Number.isFinite(value) ? value : 1));
}

export const arrangement = new ArrangementStore();
