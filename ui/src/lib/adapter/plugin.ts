/**
 * Platform Adapter — Plugin (WebView) Implementation
 *
 * Implements ContrapunkAdapter by communicating with the nih-plug host
 * via window.plugin.send/listen (provided by nih-plug-webview).
 *
 * Used when the Svelte UI runs embedded inside a VST3/CLAP/AU plugin.
 */

import type {
	ContrapunkAdapter,
	EngineState,
	GuitarConfig,
	HarmonicLimit,
	HoldMode,
	MidiDevice,
	MidiPermissionState,
	NoteState,
	PhraseState,
	PluginInputMode,
	PluginMidiOutputMode,
	Preset,
	SlideConfig,
	SlideVoiceState,
	SynthRolePatch,
	TransportState,
	TuningStyle,
	VoiceOutputAssignment,
	VoiceOutputTarget,
	VoiceRouteId
} from './types';
import {
	cloneRolePatch,
	defaultRolePatch,
	rolePatchFromWire,
	rolePatchToWire
} from '$lib/elixir/patch';
import { defaultSlideConfig } from '$lib/slide/config';
import { mapPhraseState } from './phrase';

declare global {
	interface Window {
		plugin: {
			send: (message: string) => void;
			listen: (callback: (message: string) => void) => () => void;
			resize: (width: number, height: number) => void;
		};
	}
}

/** Current state from the plugin host, updated via listen() */
let currentParams: Record<string, unknown> = {};

const FLAT_TO_SHARP: Record<string, string> = {
	Db: 'C#',
	Eb: 'D#',
	Gb: 'F#',
	Ab: 'G#',
	Bb: 'A#'
};

export class PluginAdapter implements ContrapunkAdapter {
	readonly capabilities = {
		// DAW hosts the audio path entirely; the delay FX in the
		// plugin's process() isn't tempo-sync-aware (would need
		// host-tempo plumbing through nih-plug).
		delayTempoSync: false,
		// Chain editor is Tauri-only — plugin chain is fixed by the
		// nih-plug pipeline.
		chainEditor: false,
		// editor.rs frame loop pushes input/harmony/canon/counterpoint
		// notes via the `noteUpdate` message — Companion lanes are
		// wired through the audio thread now.
		noteUpdates: true,
		// DAW is the master clock in plugin mode — our TransportBar
		// would just be a frozen mirror of the host. Hide it.
		transportControl: false,
		// DAW routes MIDI to/from the plugin via its own bus / track
		// pickers; ours would be inert.
		midiDevicePicker: false,
		// Contrapunk's plugin renders its fixed Elixir monitor and exposes
		// real enabled/master parameters. Legacy delay/reverb stay hidden.
		audioFx: true,
		builtInFx: false,
		// Companion lanes (canon + counterpoint) are wired in the
		// plugin's process() via Companion::tick_tagged. CompanionPanel
		// IPC handlers in editor.rs route HoldMode / canon_configure /
		// counterpoint_configure into the running plugin instance.
		companionLanes: true,
		intervalMaps: false,
		patternLanes: false,
		phraseContext: true,
		// DAW owns audio + MIDI routing into the plugin. The source
		// radio (MIDI / Guitar / Voice) doesn't apply — the host bus
		// is the source. Hide the InputPanel entirely in plugin mode.
		inputSourcePicker: false,
		// DAW assigns plugin MIDI output channels; our per-voice port
		// picker would conflict. Plugin emits ch1 melody, ch2-ch5 harmony,
		// ch6 canon, ch7 counterpoint.
		perVoicePortRouting: false,
		// Plugin-only host MIDI mode selector.
		pluginMidiOutputMode: true,
		// The internal Elixir monitor exposes all four role gain buses.
		roleMix: true,
		nativeTuning: true,
		performanceReset: false,
		// Plugin guitar path runs through the DAW; calibration profile
		// persistence isn't wired (would need host file access).
		calibrationFlow: false
	} as const;

	private noteUpdateCallback: ((state: NoteState) => void) | null = null;
	private pluginParamsCallbacks = new Set<() => void>();
	private _detuneCents = 0;
	private _isReady = false;

	async init(): Promise<void> {
		if (this._isReady) return;
		let resolveInitialParams: (() => void) | null = null;
		const initialParams = new Promise<void>((resolve) => {
			resolveInitialParams = resolve;
		});

		// Register listener before sending ready so the first host snapshot
		// cannot race the UI's initial getEngineState() call.
		window.plugin.listen((msg: string) => {
			try {
				const data = JSON.parse(msg);
				if (data.type === 'paramsUpdate') {
					currentParams = data;
					resolveInitialParams?.();
					resolveInitialParams = null;
					for (const callback of this.pluginParamsCallbacks) callback();
				} else if (data.type === 'slideUpdate') {
					currentParams = { ...currentParams, slideVoices: data.voices ?? [] };
				} else if (data.type === 'noteUpdate' && this.noteUpdateCallback) {
					this.noteUpdateCallback({
						inputNotes: data.inputNotes ?? [],
						harmonyNotes: data.harmonyNotes ?? [],
						borrowedNotes: data.borrowedNotes ?? [],
						// Companion lanes arrive as independent ownership sets
						// so source-aware views do not infer attribution by pitch.
						canonNotes: data.canonNotes ?? [],
						counterpointNotes: data.counterpointNotes ?? [],
						chordName: data.chordName ?? '',
						lastBorrowedFrom: data.lastBorrowedFrom ?? '',
						currentKey: data.currentKey ?? 'C',
						phrase: mapPhraseState(data.phrase)
					});
				}
			} catch {
				/* ignore parse errors */
			}
		});

		// Signal ready — the Rust side responds with product + parameter state.
		window.plugin.send(JSON.stringify({ type: 'ready' }));
		await initialParams;
		window.plugin.resize?.(currentParams.product === 'elixir' ? 760 : 1200, currentParams.product === 'elixir' ? 720 : 800);
		this._isReady = true;
	}

	private send(type: string, value?: unknown, extra: Record<string, unknown> = {}): void {
		window.plugin.send(JSON.stringify({ ...extra, type, value }));
	}

	async getEngineState(): Promise<EngineState> {
		return {
			key: FLAT_TO_SHARP[(currentParams.key as string) ?? 'C'] ?? ((currentParams.key as string) ?? 'C'),
			mode: (currentParams.mode as string) ?? 'DiatonicThirds',
			modeNumber: 0,
			scaleMode: 'Ionian',
			octaveMode: (currentParams.octaveMode as string) ?? 'None',
			octaveIntensity: (currentParams.octaveIntensity as number) ?? 1,
			voiceLeadingEnabled: (currentParams.voiceLeading as boolean) ?? false,
			voiceLeadingStyle: (currentParams.voiceLeadingStyle as string) ?? 'Free',
			interchangeEnabled: false,
			interchangeRange: 3,
			voicePosition: (currentParams.voicePosition as number) ?? 0,
			voiceCount: (currentParams.voiceCount as number) ?? 2,
			autoKey: (currentParams.autoKey as boolean) ?? false,
			tuningStyle: currentParams.tuningStyle === 'Pure' ? 'pure' : 'standard',
			tuningDepth: (currentParams.tuningDepth as number) ?? 0.6,
			harmonicLimit: currentParams.harmonicLimit === 'Seven' ? 'seven' : 'five',
			isRunning: true, // Plugin is always "running" — DAW handles routing
			counterpointSpecies:
				(currentParams.counterpointSpecies as string) ?? 'Species1',
			counterpointStrictness:
				(currentParams.counterpointStrictness as string) ?? 'Strict'
		};
	}

	async setKey(key: string): Promise<void> {
		this.send('setKey', key);
	}

	async setMode(mode: string): Promise<void> {
		this.send('setMode', mode);
	}

	async setScaleMode(_mode: string): Promise<void> {
		// Scale mode not exposed as a plugin parameter yet
	}

	async setOctaveMode(mode: string): Promise<void> {
		this.send('setOctaveMode', mode);
	}

	async setOctaveIntensity(amount: number): Promise<void> {
		this.send('setOctaveIntensity', amount);
	}

	async setVoiceLeading(enabled: boolean, style: string): Promise<void> {
		this.send('setVoiceLeading', enabled);
		this.send('setVoiceLeadingStyle', style);
	}

	async setInterchange(_enabled: boolean, _range: number): Promise<void> {
		// Interchange not exposed as a plugin parameter yet
	}

	async setVoicePosition(position: number): Promise<void> {
		this.send('setVoicePosition', position);
	}

	async setVoiceCount(count: number): Promise<void> {
		this.send('setVoiceCount', count);
	}

	async setAutoKey(enabled: boolean): Promise<void> {
		this.send('setAutoKey', enabled);
	}

	async setTuningStyle(style: TuningStyle): Promise<void> {
		currentParams = { ...currentParams, tuningStyle: style === 'pure' ? 'Pure' : 'Standard' };
		this.send('setTuningStyle', style);
	}

	async setTuningDepth(depth: number): Promise<void> {
		const value = Math.max(0, Math.min(1, depth));
		currentParams = { ...currentParams, tuningDepth: value };
		this.send('setTuningDepth', value);
	}

	async setHarmonicLimit(limit: HarmonicLimit): Promise<void> {
		currentParams = { ...currentParams, harmonicLimit: limit === 'seven' ? 'Seven' : 'Five' };
		this.send('setHarmonicLimit', limit);
	}

	async setTuningCompare(enabled: boolean): Promise<void> {
		currentParams = { ...currentParams, tuningCompare: enabled };
		this.send('setTuningCompare', enabled);
	}

	async getSlideConfig(): Promise<SlideConfig> {
		return (currentParams.slideConfig as SlideConfig | undefined) ?? defaultSlideConfig();
	}

	async setSlideConfig(config: SlideConfig): Promise<void> {
		currentParams = { ...currentParams, slideConfig: config };
		this.send('setSlideConfig', config);
	}

	async getSlideVoices(): Promise<SlideVoiceState[]> {
		return ((currentParams.slideVoices as Array<Record<string, unknown>> | undefined) ?? []).map(
			(voice) => ({
				voiceId: String(voice.voice_id),
				slot: voice.slot as SlideVoiceState['slot'],
				currentFrequencyHz: Number(voice.current_frequency_hz),
				targetFrequencyHz: Number(voice.target_frequency_hz),
				progress: Number(voice.progress),
				curve: voice.curve as SlideVoiceState['curve'],
				durationMs: Number(voice.duration_ms)
			})
		);
	}

	async setCounterpointSpecies(species: string): Promise<void> {
		// nih-plug parameters are fixed at compile time; pass through so the
		// host can route to a parameter if it eventually exposes one.
		this.send('setCounterpointSpecies', species);
	}

	async setCounterpointStrictness(strictness: string): Promise<void> {
		// nih-plug parameters are fixed at compile time; pass through so the
		// host can route to a parameter if it eventually exposes one.
		this.send('setCounterpointStrictness', strictness);
	}

	async setExplicitIntervalMap(_degreeOffsets: number[][], _fallbackOffsets: number[]): Promise<void> {
		throw new Error('Explicit interval maps are unavailable in the plugin');
	}

	async companionSetEnabled(enabled: boolean): Promise<void> {
		this.send('companionSetEnabled', enabled);
	}
	async companionIsEnabled(): Promise<boolean> {
		return false;
	}
	async companionSetGlobalHoldMode(holdMode: HoldMode): Promise<void> {
		this.send('companionSetGlobalHoldMode', holdMode);
	}
	async setPhraseGapBeats(beats: number): Promise<void> {
		const previous = currentParams.phrase;
		const phrase = previous !== null && typeof previous === 'object' ? previous : {};
		currentParams = { ...currentParams, phrase: { ...phrase, gap_beats: beats } };
		this.send('companionSetPhraseGap', beats);
	}
	async getPhraseState(): Promise<PhraseState> {
		return mapPhraseState(currentParams.phrase);
	}
	async canonSetEnabled(enabled: boolean): Promise<void> {
		this.send('canonConfigure', { enabled });
	}
	async canonSetDelay(beats: number): Promise<void> {
		this.send('canonConfigure', { delay_beats: beats });
	}
	async canonSetTranspose(degrees: number): Promise<void> {
		this.send('canonConfigure', { transpose_degrees: degrees });
	}
	async canonSetVoices(
		voices: Array<{
			delay_beats: number;
			transpose_degrees: number;
			time_ratio?: number;
			harmony_mode?: string | null;
			reference_voice?: number | null;
			voice_count?: number | null;
			voice_position?: number | null;
			voice_leading_enabled?: boolean | null;
			voice_leading_style?: string | null;
			octave_mode?: string | null;
			counterpoint_species?: string | null;
			counterpoint_strictness?: string | null;
		}>
	): Promise<void> {
		this.send('canonSetVoices', voices);
	}
	async counterpointSetConfig(config: {
		enabled?: boolean;
		species?: string;
		transpose_degrees?: number;
		prefer_above?: boolean;
		phrase_aware?: boolean;
	}): Promise<void> {
		this.send('counterpointConfigure', config);
	}

	async canonConfigure(partial: Record<string, unknown>): Promise<void> {
		// editor.rs's `canonConfigure` handler forwards `value` to
		// `Companion::configure_lane("canon", _)`. UI uses this for
		// per-lane HoldMode + voice-cascade config.
		this.send('canonConfigure', partial);
	}

	async counterpointConfigure(partial: Record<string, unknown>): Promise<void> {
		this.send('counterpointConfigure', partial);
	}

	async patternConfigure(
		_laneId: 'pattern_low' | 'pattern_counter',
		_partial: Record<string, unknown>
	): Promise<void> {}

	async patternState(
		_laneId: 'pattern_low' | 'pattern_counter'
	): Promise<Record<string, unknown> | null> {
		return null;
	}

	async counterpointState(): Promise<{
		enabled: boolean;
		species: string;
		transpose_degrees: number;
		prefer_above: boolean;
		phrase_aware?: boolean;
	} | null> {
		return null;
	}

	async canonState(): Promise<{
		enabled: boolean;
		delay_beats: number;
		transpose_degrees: number;
		voices?: Array<{
			delay_beats: number;
			transpose_degrees: number;
			time_ratio?: number;
			harmony_mode?: string | null;
			reference_voice?: number | null;
		}>;
	} | null> {
		return null;
	}

	// -- MIDI devices (DAW handles routing in plugin mode) --

	/** DAW manages MIDI for the plugin; no browser permission needed. */
	readonly midiPermissionState: MidiPermissionState = 'granted';

	/** No-op in plugin mode — host owns MIDI routing. */
	async requestMidiPermission(): Promise<MidiPermissionState> {
		return 'granted';
	}

	async listMidiInputs(): Promise<MidiDevice[]> {
		return [];
	}

	async listMidiOutputs(): Promise<MidiDevice[]> {
		return [];
	}

	async refreshMidiDevices(): Promise<void> {}

	// -- Routing (DAW handles this) --

	async startRouting(_inputIdx: number, _outputIndices: number[]): Promise<void> {}
	async stopRouting(): Promise<void> {}

	// Per-voice output routing — the plugin host handles MIDI routing
	// outside the UI, so these are no-ops that just track the user's
	// selection in memory for UI restoration.
	private _voiceOutputs = new Map<VoiceRouteId, VoiceOutputTarget>();
	async setVoiceOutput(route: VoiceRouteId, target: VoiceOutputTarget): Promise<void> {
		if (target.kind === 'synth') this._voiceOutputs.delete(route);
		else this._voiceOutputs.set(route, target);
	}
	async setAllVoiceOutputsToSynth(_enabled: boolean): Promise<void> {}
	async getVoiceOutputs(): Promise<VoiceOutputAssignment[]> {
		return [...this._voiceOutputs].map(([route, target]) => ({ route, target }));
	}

	async getPluginInputMode(): Promise<PluginInputMode> {
		return currentParams.inputMode === 'Audio' ? 'audio' : 'midi';
	}

	async setPluginInputMode(mode: PluginInputMode): Promise<void> {
		const value = mode === 'audio' ? 'Audio' : 'Midi';
		currentParams = { ...currentParams, inputMode: value };
		this.send('setInputMode', value);
	}

	async getPluginMidiOutputMode(): Promise<PluginMidiOutputMode> {
		return currentParams.midiOutputMode === 'PassThrough' ? 'pass_through' : 'full';
	}

	async setPluginMidiOutputMode(mode: PluginMidiOutputMode): Promise<void> {
		const value = mode === 'pass_through' ? 'PassThrough' : 'Full';
		currentParams = { ...currentParams, midiOutputMode: value };
		this.send('setMidiOutputMode', value);
	}

	async getPluginSynthEnabled(): Promise<boolean> {
		return (currentParams.synthEnabled as boolean) ?? true;
	}

	async setPluginSynthEnabled(enabled: boolean): Promise<void> {
		currentParams = { ...currentParams, synthEnabled: enabled };
		this.send('setSynthEnabled', enabled);
	}

	async panicAllNotesOff(): Promise<void> {
		this.send('panic', true);
	}

	async resetPerformance(): Promise<void> {
		throw new Error('Performance Reset is owned by the DAW in plugin mode');
	}

	onPluginParamsUpdate(callback: () => void): () => void {
		this.pluginParamsCallbacks.add(callback);
		return () => this.pluginParamsCallbacks.delete(callback);
	}

	// -- Real-time state --

	onNoteUpdate(callback: (state: NoteState) => void): () => void {
		this.noteUpdateCallback = callback;
		return () => {
			this.noteUpdateCallback = null;
		};
	}

	onKnobCcRaw(_callback: (cc: number, value: number) => void): () => void {
		// Plugin path: DAW host owns MIDI routing. Knob CC mapping
		// would happen via DAW automation rather than direct CC
		// interception. No-op for now.
		return () => {};
	}

	// -- Virtual Input (not applicable in plugin mode) --

	async injectNoteOn(note: number, velocity = 100): Promise<number[]> {
		this.send('injectNoteOn', undefined, { note, velocity });
		return [note];
	}

	async injectNoteOff(note: number): Promise<number[]> {
		this.send('injectNoteOff', undefined, { note });
		return [note];
	}

	// -- Presets (not available in plugin mode yet) --

	async listPresets(): Promise<Preset[]> {
		return [];
	}

	async loadPreset(_name: string): Promise<void> {}
	async savePreset(_name: string): Promise<void> {}
	async deletePreset(_name: string): Promise<void> {}

	// -- Guitar input (plugin gets audio from DAW) --

	async listAudioDevices(): Promise<string[]> {
		return [];
	}

	async setGuitarDevice(_deviceName: string, _channel: number): Promise<void> {}
	async setGuitarConfig(_config: GuitarConfig): Promise<void> {}

	async getCalibrationStatus() {
		return { existsOnDisk: false, path: '', version: 1, sampleCounts: [0, 0, 0, 0, 0, 0] };
	}
	async loadCalibrationProfile() {
		return this.getCalibrationStatus();
	}
	async saveCalibrationProfile(_profileJson: string) {
		return this.getCalibrationStatus();
	}

	async deleteCalibrationProfile() {
		return this.getCalibrationStatus();
	}

	async getLastPortMap(): Promise<number[]> {
		// Plugin host owns routing; per-voice port mapping doesn't apply.
		return [];
	}

	// -- Detune --

	setDetune(cents: number): void {
		this._detuneCents = cents;
	}

	getDetune(): number {
		return this._detuneCents;
	}

	// -- Transport (plugin mode — DAW owns transport) --

	async getTransportState(): Promise<TransportState> {
		return {
			running: false,
			bpm: 120,
			beatsPerBar: 4,
			beatUnit: 4,
			sampleRate: 48_000,
			samplePos: 0,
			beatPosition: 0,
			bar: 0,
			metronomeEnabled: false
		};
	}

	async transportPlay(): Promise<void> {}
	async transportStop(): Promise<void> {}
	async transportReset(): Promise<void> {}
	async setBpm(_bpm: number): Promise<void> {}
	async setTimeSignature(_beatsPerBar: number, _beatUnit: number): Promise<void> {}
	async setMetronomeEnabled(_enabled: boolean): Promise<void> {}

	// -- Synth (DAW hosts audio in plugin mode) --

	async getSynthState() {
		return {
			enabled: (currentParams.synthEnabled as boolean) ?? true,
			masterGain: (currentParams.synthGain as number) ?? 0.25,
			mixGains: Array.isArray(currentParams.mixGains)
				? (currentParams.mixGains as number[])
				: undefined,
			rolePatches: Array.isArray(currentParams.rolePatches)
				? (currentParams.rolePatches as Record<string, any>[]).map(rolePatchFromWire)
				: Array.from({ length: 4 }, defaultRolePatch)
		};
	}
	async setSynthEnabled(enabled: boolean): Promise<void> {
		currentParams = { ...currentParams, synthEnabled: enabled };
		this.send('setSynthEnabled', enabled);
	}
	async setSynthMasterGain(value: number): Promise<void> {
		const gain = Math.max(0, Math.min(1, value));
		currentParams = { ...currentParams, synthGain: gain };
		this.send('setSynthGain', gain);
	}
	async setSynthMixGain(group: number, value: number): Promise<void> {
		if (group < 0 || group > 3) return;
		const gains = Array.isArray(currentParams.mixGains)
			? [...(currentParams.mixGains as number[])]
			: [1, 1, 1, 1];
		gains[group] = Math.max(0, Math.min(1, value));
		currentParams = { ...currentParams, mixGains: gains };
		window.plugin.send(JSON.stringify({ type: 'setSynthMixGain', group, value: gains[group] }));
	}
	async setSynthRolePatch(group: number, patch: SynthRolePatch): Promise<void> {
		if (group < 0 || group > 3) return;
		const patches = Array.isArray(currentParams.rolePatches)
			? (currentParams.rolePatches as Record<string, any>[]).map(rolePatchFromWire)
			: Array.from({ length: 4 }, defaultRolePatch);
		patches[group] = cloneRolePatch(patch);
		currentParams = { ...currentParams, rolePatches: patches.map(rolePatchToWire) };
		window.plugin.send(JSON.stringify({
			type: 'setSynthRolePatch',
			group,
			patch: rolePatchToWire(patch)
		}));
	}

	// -- FX (DAW hosts FX in plugin mode) --

	async getReverbState() {
		return { enabled: false, mix: 0.3, roomSize: 0.7, damping: 0.5 };
	}
	async setReverbEnabled(_enabled: boolean): Promise<void> {}
	async setReverbMix(_value: number): Promise<void> {}
	async setReverbRoomSize(_value: number): Promise<void> {}
	async setReverbDamping(_value: number): Promise<void> {}

	async getDelayState() {
		return {
			enabled: false,
			mix: 0.3,
			timeMs: 375,
			feedback: 0.35,
			syncEnabled: false,
			subdivision: '1/8d' as const
		};
	}
	async setDelayEnabled(_enabled: boolean): Promise<void> {}
	async setDelayMix(_value: number): Promise<void> {}
	async setDelayTimeMs(_ms: number): Promise<void> {}
	async setDelayFeedback(_value: number): Promise<void> {}
	async setDelaySyncEnabled(_enabled: boolean): Promise<void> {}
	async setDelaySubdivision(_subdivision: string): Promise<void> {}

	// -- Chain topology + CLAP (DAW handles plugin hosting; stubs) --

	async listChainBlocks() {
		return [];
	}
	async removeChainBlock(_index: number): Promise<void> {}
	async clearChain(): Promise<void> {}
	async listClapPlugins() {
		return [];
	}
	async addClapPluginToChain(_path: string) {
		return { pluginId: 0, name: '', path: '', hasGui: false };
	}
	async openPluginGui(_pluginId: number): Promise<void> {}
	async getPluginGuiSize(_pluginId: number): Promise<{ width: number; height: number } | null> {
		return null;
	}
	async openPluginGuiEmbedded(
		_pluginId: number,
		_x: number,
		_y: number,
		_w: number,
		_h: number
	): Promise<void> {}
	async setPluginGuiFrame(
		_pluginId: number,
		_x: number,
		_y: number,
		_w: number,
		_h: number
	): Promise<void> {}
	async closePluginGui(_pluginId: number): Promise<void> {}
	async removePlugin(_pluginId: number): Promise<void> {}
}
