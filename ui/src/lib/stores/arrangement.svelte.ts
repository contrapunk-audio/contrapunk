import { adapter } from '$lib/adapter';
import type { CounterpointSpeciesName } from './engine.svelte';
import { DEFAULT_EXPLICIT_INTERVAL_MAP, engine } from './engine.svelte';
import { tone } from './tone.svelte';
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
	phraseAware: boolean;
}

const DEFAULT_COUNTERPOINT: CounterpointLaneState = {
	enabled: false,
	species: 'Species1',
	transposeDegrees: 2,
	preferAbove: true,
	phraseAware: false
};

const EMPTY_PATTERN_LANE: ArrangementPatternLaneConfig = {
	enabled: false,
	cycleBeats: 4,
	tailBeats: 4,
	pitchAnchor: 'key',
	onlyWhenInputIdle: false,
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
	muted = $state([false, false, false, false]);
	solo = $state<number | null>(null);
	mixLoaded = $state(false);
	mixError = $state<string | null>(null);
	applying = $state(false);

	get availableCapabilities(): ReadonlySet<ArrangementCapability> {
		const capabilities = new Set<ArrangementCapability>(['harmony', 'voice_leading']);
		if (adapter.capabilities.companionLanes) {
			capabilities.add('strict_canon');
			capabilities.add('free_imitation');
			capabilities.add('species_counterpoint');
		}
		if (adapter.capabilities.intervalMaps) capabilities.add('interval_stacks');
		if (adapter.capabilities.patternLanes) {
			capabilities.add('pattern_lane');
			capabilities.add('stable_lane_groups');
		}
		if (adapter.capabilities.phraseContext) capabilities.add('phrase_context');
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
					preferAbove: state.prefer_above,
					phraseAware: state.phrase_aware === true
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
				if (state.mixGains?.length === 4 && this.solo === null && !this.muted.some(Boolean)) {
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
				pitch_anchor: config.pitchAnchor ?? 'key',
				only_when_input_idle: config.onlyWhenInputIdle ?? false,
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
				prefer_above: patch.preferAbove,
				phrase_aware: patch.phraseAware
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

	appliedMixLevel(group: number) {
		if (this.muted[group]) return 0;
		if (this.solo !== null && this.solo !== group) return 0;
		return this.mixLevels[group] ?? 1;
	}

	async pushMixLevel(group: number, value = this.appliedMixLevel(group)) {
		if (!adapter.capabilities.roleMix || group < 0 || group >= 4) return;
		await adapter.setSynthMixGain(group, clamp01(value));
	}

	async setAndPushMixLevel(group: number, value: number) {
		if (!adapter.capabilities.roleMix || group < 0 || group >= 4) return;
		const previous = this.mixLevels[group] ?? 1;
		this.setMixLevel(group, value);
		try {
			await this.pushMixLevel(group);
			this.mixError = null;
		} catch (error) {
			this.setMixLevel(group, previous);
			try { await this.pushMixLevel(group); } catch { /* Preserve the original error. */ }
			this.mixError = `Could not change mix level: ${error}`;
		}
	}

	async toggleMute(group: number) {
		if (!adapter.capabilities.roleMix || group < 0 || group >= 4) return;
		const previous = [...this.muted];
		this.muted[group] = !this.muted[group];
		this.muted = [...this.muted];
		try {
			await this.pushMixLevel(group);
			this.mixError = null;
		} catch (error) {
			this.muted = previous;
			try { await this.pushMixLevel(group); } catch { /* Preserve the original error. */ }
			this.mixError = `Could not change mute: ${error}`;
		}
	}

	async toggleSolo(group: number) {
		if (!adapter.capabilities.roleMix || group < 0 || group >= 4) return;
		const previous = this.solo;
		this.solo = this.solo === group ? null : group;
		try {
			await this.pushAllMixLevels();
			this.mixError = null;
		} catch (error) {
			this.solo = previous;
			try { await this.pushAllMixLevels(); } catch { /* Preserve the original error. */ }
			this.mixError = `Could not change solo: ${error}`;
		}
	}

	private async pushAllMixLevels() {
		for (let group = 0; group < 4; group++) await this.pushMixLevel(group);
	}

	async apply(config: ArrangementConfig) {
		if (this.applying) throw new Error('An arrangement is already being applied');
		const errors = validateArrangementConfig(config);
		if (errors.length) throw new Error(errors.join('; '));

		await this.syncFromBackend();
		const previous = this.snapshot();
		this.applying = true;
		try {
			await tone.stop();
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
		if (adapter.capabilities.intervalMaps) {
			await engine.setExplicitIntervalMap(
				harmony.explicitIntervalMap ?? DEFAULT_EXPLICIT_INTERVAL_MAP
			);
		}
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
			preferAbove: companion.counterpoint.preferAbove,
			phraseAware: companion.counterpoint.phraseAware ?? false
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
				counterpointStrictness: engine.counterpointStrictness,
				explicitIntervalMap: {
					degreeOffsets: engine.explicitIntervalMap.degreeOffsets.map((offsets) => [...offsets]),
					fallbackOffsets: [...engine.explicitIntervalMap.fallbackOffsets]
				}
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
					phraseAware: this.counterpoint.phraseAware,
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
		pitchAnchor:
			state.pitch_anchor === 'phrase_start' || state.pitch_anchor === 'latest_input'
				? state.pitch_anchor
				: 'key',
		onlyWhenInputIdle: state.only_when_input_idle === true,
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
