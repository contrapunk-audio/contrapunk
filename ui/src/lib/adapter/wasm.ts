/**
 * Platform Adapter — WASM Implementation
 *
 * Implements ContrapunkAdapter by calling wasm-bindgen exported functions
 * directly. Used when running in the browser without Tauri.
 */

import type {
	ContrapunkAdapter,
	EngineState,
	GuitarConfig,
	HoldMode,
	MidiDevice,
	MidiPermissionState,
	NoteState,
	Preset,
	TransportState,
	VoiceOutputTarget
} from './types';
import { MAX_VOICES } from './types';
import { GuitarAudioCapture } from '$lib/audio/guitarCapture';
import { guitar } from '$lib/stores/guitar.svelte';
import { transport } from '$lib/stores/transport.svelte';
import * as embedAudio from '$lib/embed/audio';

/**
 * Dynamically imported WASM module.
 * Resolved during init() to avoid top-level import failures
 * when the wasm-pkg hasn't been built yet.
 */
// eslint-disable-next-line @typescript-eslint/no-explicit-any
let wasmModule: any = null;
// eslint-disable-next-line @typescript-eslint/no-explicit-any
let engine: any = null;
// CompanionWasm instance — hosts the Canon + Counterpoint Lanes that
// the v1.2.0 native build ships. Constructed during init().
// eslint-disable-next-line @typescript-eslint/no-explicit-any
let companion: any = null;
// Animation-frame loop that advances the Companion's transport and
// drains pending lane emissions. Started on first injectNoteOn.
let companionTickHandle: number | null = null;
// MIDI numbers the Companion currently holds on. dispatchOpsJson
// updates this set on every note_on / note_off so getNoteState can
// merge these pitches into harmonyNotes — that's what feeds the
// ActiveNotes strip + the Piano / Fretboard highlights. Without
// this, the Tauri build shows the canon + counterpoint emissions
// in the UI but the WASM build only showed the player's harmony.
const activeCompanionNotes: Set<number> = new Set();

/** Debug logger — set `window.__cpDebug = true` in DevTools to
 *  print a snapshot of the Companion + per-voice state on every
 *  player NoteOn / NoteOff and the dispatch ops that came back.
 *  Off by default so production users don't see console noise. */
function isCompanionDebug(): boolean {
	try {
		return Boolean((globalThis as unknown as { __cpDebug?: boolean }).__cpDebug);
	} catch {
		return false;
	}
}

function logCompanionEvent(label: string, payload: Record<string, unknown>): void {
	if (!isCompanionDebug() || !companion) return;
	try {
		const snap = JSON.parse(companion.debug_snapshot());
		// eslint-disable-next-line no-console
		console.log(`[cp:${label}]`, { ...payload, ...snap });
	} catch {
		/* serialization failure — ignore */
	}
}

/**
 * Sentinel input index used to signal "guitar audio input" rather than MIDI.
 * Must match the value used in the UI when starting guitar routing.
 */
const GUITAR_AUDIO_SENTINEL = 999_997;

export class WasmAdapter implements ContrapunkAdapter {
	private initialized = false;
	private _isRunning = false;
	private noteUpdateCallback: ((state: NoteState) => void) | null = null;
	private pollingHandle: number | null = null;
	private midiAccess: MIDIAccess | null = null;
	private _midiPermissionState: MidiPermissionState = 'idle';
	private activeInput: MIDIInput | null = null;
	private activeOutputs: MIDIOutput[] = [];
	private _detuneCents = 0;
	/** Pitch bend range in semitones (standard MIDI default). */
	private pitchBendRangeSemitones = 2;
	/** Guitar audio capture instance for browser-based pitch detection. */
	private guitarCapture: GuitarAudioCapture | null = null;
	/** Currently selected guitar device ID (for audio capture). */
	private _guitarDeviceId = '';
	/** Currently selected guitar channel (0-based). */
	private _guitarChannel = 0;

	async init(): Promise<void> {
		if (this.initialized) return;

		try {
			// Dynamic import of wasm-pack output from $lib/wasm-pkg
			// wasm-pack builds to ui/src/lib/wasm-pkg/ which is under $lib
			wasmModule = await import('$lib/wasm-pkg');
			if (wasmModule.default && typeof wasmModule.default === 'function') {
				await wasmModule.default();
			}
			engine = new wasmModule.Engine();
			// Companion + Lanes. Mirrors what Tauri's AppState registers
			// at boot (CanonLane + CounterpointLane under one master
			// Companion). Construction defaults to enabled = ON.
			try {
				companion = new wasmModule.CompanionWasm();
			} catch (e) {
				console.warn('[wasm] CompanionWasm init failed:', e);
				companion = null;
			}
			this.initialized = true;
			this.startCompanionTick();
		} catch (e) {
			throw new Error(`Failed to initialize WASM: ${e}`);
		}
	}

	/** Run a requestAnimationFrame loop that advances the Companion's
	 *  transport by ~one buffer per frame and drains any lane emissions
	 *  whose scheduled fire_at has elapsed. Frames-per-tick is computed
	 *  to keep the transport roughly in real time at 48 kHz. */
	private startCompanionTick(): void {
		if (companionTickHandle !== null || !companion) return;
		let last = performance.now();
		const loop = () => {
			if (!companion) {
				companionTickHandle = null;
				return;
			}
			const now = performance.now();
			const dtMs = Math.max(0, now - last);
			last = now;
			// Frames at 48 kHz that match wallclock delta.
			const frames = Math.max(1, Math.round(dtMs * 48));
			try {
				companion.advance(frames);
				const json = companion.tick();
				this.dispatchOpsJson(json);
			} catch {
				/* keep the loop alive */
			}
			companionTickHandle = requestAnimationFrame(loop);
		};
		companionTickHandle = requestAnimationFrame(loop);
	}

	/** Play (or release) the lane-emitted dispatch ops on the browser
	 *  WebAudio synth + any active external MIDI output. */
	private dispatchOpsJson(json: string): void {
		if (!json || json === '[]') return;
		let ops: Array<{ kind: string; note?: number; velocity?: number; channel?: number }>;
		try {
			ops = JSON.parse(json);
		} catch {
			return;
		}
		for (const op of ops) {
			if (op.kind === 'note_on' && typeof op.note === 'number') {
				const v = op.velocity ?? 100;
				embedAudio.noteOn(op.note, v);
				activeCompanionNotes.add(op.note);
				if (this.activeOutputs.length > 0) {
					this.activeOutputs[0].send([0x90, op.note, v]);
				}
			} else if (op.kind === 'note_off' && typeof op.note === 'number') {
				embedAudio.noteOff(op.note);
				activeCompanionNotes.delete(op.note);
				if (this.activeOutputs.length > 0) {
					this.activeOutputs[0].send([0x80, op.note, 0]);
				}
			}
		}
	}

	/**
	 * Tear down background loops and resources. Called from the root
	 * layout `onDestroy` hook so hot-reload and page navigation don't
	 * leak lingering guitar captures.
	 */
	destroy(): void {
		if (this.guitarCapture) {
			try {
				this.guitarCapture.stop();
			} catch {
				// best-effort
			}
			this.guitarCapture = null;
		}
	}

	private ensureInit(): void {
		if (!this.initialized || !engine) {
			throw new Error('WASM adapter not initialized. Call init() first.');
		}
	}

	async getEngineState(): Promise<EngineState> {
		this.ensureInit();
		try {
			const raw = engine.get_state();
			return {
				key: raw.key ?? 'C',
				mode: raw.mode ?? 'PassThrough',
				modeNumber: raw.mode_number ?? 1,
				scaleMode: raw.scale_mode ?? 'Ionian',
				octaveMode: raw.octave_mode ?? 'None',
				voiceLeadingEnabled: raw.voice_leading_enabled ?? false,
				voiceLeadingStyle: raw.voice_leading_style ?? 'Free',
				interchangeEnabled: raw.interchange_enabled ?? false,
				interchangeRange: raw.borrowing_range ?? 3,
				voicePosition: raw.voice_position ?? 0,
				voiceCount: raw.voice_count ?? 2,
				autoKey: raw.auto_key ?? false,
				isRunning: this._isRunning,
				counterpointSpecies: raw.counterpoint_species ?? 'Species1',
				counterpointStrictness: raw.counterpoint_strictness ?? 'Strict'
			};
		} catch (e) {
			throw new Error(`Failed to get engine state: ${e}`);
		}
	}

	/** Mirror the wasm Engine's global state (key / mode / scale_mode
	 *  / voice_count / voice_position) into the CompanionWasm's
	 *  internal engine_snapshot so the canon mini-engines see the
	 *  same key/scale as the player. The Tauri build shares one
	 *  HarmonyEngine across both; in WASM they're separate objects. */
	private syncCompanionGlobal(): void {
		if (!companion || !engine) return;
		try {
			const raw = engine.get_state();
			companion.set_global_state(
				raw.key ?? 'C',
				raw.mode ?? 'PassThrough',
				raw.scale_mode ?? 'Ionian',
				raw.voice_count ?? 2,
				raw.voice_position ?? 0
			);
		} catch {
			/* best-effort */
		}
	}

	async setKey(key: string): Promise<void> {
		this.ensureInit();
		try {
			engine.set_key(key);
			this.syncCompanionGlobal();
		} catch (e) {
			throw new Error(`Failed to set key: ${e}`);
		}
	}

	async setMode(mode: string): Promise<void> {
		this.ensureInit();
		try {
			engine.set_mode(mode);
			this.syncCompanionGlobal();
		} catch (e) {
			throw new Error(`Failed to set mode: ${e}`);
		}
	}

	async setScaleMode(mode: string): Promise<void> {
		this.ensureInit();
		try {
			engine.set_scale_mode(mode);
			this.syncCompanionGlobal();
		} catch (e) {
			throw new Error(`Failed to set scale mode: ${e}`);
		}
	}

	async setOctaveMode(mode: string): Promise<void> {
		this.ensureInit();
		try {
			engine.set_octave_mode(mode);
		} catch (e) {
			throw new Error(`Failed to set octave mode: ${e}`);
		}
	}

	async setOctaveIntensity(amount: number): Promise<void> {
		this.ensureInit();
		try {
			engine.set_octave_intensity(amount);
		} catch (e) {
			throw new Error(`Failed to set octave intensity: ${e}`);
		}
	}

	async setVoiceLeading(enabled: boolean, style: string): Promise<void> {
		this.ensureInit();
		try {
			engine.set_voice_leading(enabled, style);
		} catch (e) {
			throw new Error(`Failed to set voice leading: ${e}`);
		}
	}

	async setInterchange(enabled: boolean, range: number): Promise<void> {
		this.ensureInit();
		try {
			engine.set_interchange(enabled, range);
		} catch (e) {
			throw new Error(`Failed to set interchange: ${e}`);
		}
	}

	async setVoicePosition(position: number): Promise<void> {
		this.ensureInit();
		try {
			engine.set_voice_position(position);
			this.syncCompanionGlobal();
		} catch (e) {
			throw new Error(`Failed to set voice position: ${e}`);
		}
	}

	async setVoiceCount(count: number): Promise<void> {
		this.ensureInit();
		try {
			engine.set_voice_count(count);
			this.syncCompanionGlobal();
		} catch (e) {
			throw new Error(`Failed to set voice count: ${e}`);
		}
	}

	async setAutoKey(enabled: boolean): Promise<void> {
		this.ensureInit();
		try {
			engine.set_auto_key(enabled);
		} catch (e) {
			throw new Error(`Failed to set auto key: ${e}`);
		}
	}

	async setCounterpointSpecies(species: string): Promise<void> {
		this.ensureInit();
		try {
			engine.set_counterpoint_species(species);
		} catch (e) {
			throw new Error(`Failed to set counterpoint species: ${e}`);
		}
	}

	async setCounterpointStrictness(strictness: string): Promise<void> {
		this.ensureInit();
		try {
			engine.set_counterpoint_strictness(strictness);
		} catch (e) {
			throw new Error(`Failed to set counterpoint strictness: ${e}`);
		}
	}

	// -- Companion / CanonLane (#3) -- WASM surface stub
	// Companion lives in the Tauri router thread; the browser/WASM
	// surface has no companion infrastructure yet. These methods are
	// no-ops or return defaults so the shared store doesn't blow up
	// when running in browser mode.

	async companionSetEnabled(enabled: boolean): Promise<void> {
		if (companion) companion.set_enabled(enabled);
	}

	async companionIsEnabled(): Promise<boolean> {
		if (!companion) return false;
		try {
			return Boolean(companion.is_enabled());
		} catch {
			return false;
		}
	}

	async companionSetGlobalHoldMode(holdMode: HoldMode): Promise<void> {
		if (!companion) return;
		try {
			companion.set_global_hold_mode(JSON.stringify(holdMode));
		} catch (e) {
			console.warn('[wasm-adapter] companionSetGlobalHoldMode failed:', e);
		}
	}

	async canonSetEnabled(enabled: boolean): Promise<void> {
		if (companion) companion.configure_canon(JSON.stringify({ enabled }));
	}

	async canonSetDelay(_beats: number): Promise<void> {
		// Single-voice setters are unused in the v1.2.0 multi-voice UI;
		// keep as no-op (canonSetVoices is the authoritative path).
	}

	async canonSetTranspose(_degrees: number): Promise<void> {
		// Same as canonSetDelay — unused in v1.2.0.
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
		if (!companion) return;
		try {
			companion.configure_canon(JSON.stringify({ voices }));
		} catch (e) {
			console.warn('[wasm] canonSetVoices failed:', e);
		}
	}

	async counterpointSetConfig(config: {
		enabled?: boolean;
		species?: string;
		transpose_degrees?: number;
		prefer_above?: boolean;
	}): Promise<void> {
		if (!companion) return;
		try {
			companion.configure_counterpoint(JSON.stringify(config));
		} catch (e) {
			console.warn('[wasm] counterpointSetConfig failed:', e);
		}
	}

	async counterpointState(): Promise<{
		enabled: boolean;
		species: string;
		transpose_degrees: number;
		prefer_above: boolean;
	} | null> {
		// CompanionWasm doesn't expose a state getter in v1.2.0 — the
		// canonical state lives in the JS engine store + localStorage,
		// and gets pushed back via configure_counterpoint on mount.
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
		// Same as counterpointState — state is JS-side authoritative.
		return null;
	}

	/** Current MIDI permission state. */
	get midiPermissionState(): MidiPermissionState {
		return this._midiPermissionState;
	}

	/**
	 * Return cached `MIDIAccess` without prompting the user.
	 *
	 * `navigator.requestMIDIAccess()` is gated behind a user gesture in
	 * Chromium-based browsers — calling it at page load fails silently and
	 * leaves the device list empty. The actual prompt lives in
	 * `requestMidiPermission()`, invoked from a click handler.
	 */
	private ensureMidiAccess(): MIDIAccess | null {
		return this.midiAccess;
	}

	/**
	 * Trigger the Web MIDI permission prompt. Must be called from inside a
	 * user-gesture handler (click, keydown). Idempotent — safe to call after
	 * grant or denial; reports the resulting state via the return value.
	 */
	async requestMidiPermission(): Promise<MidiPermissionState> {
		if (typeof navigator === 'undefined' || !('requestMIDIAccess' in navigator)) {
			this._midiPermissionState = 'unsupported';
			return this._midiPermissionState;
		}
		if (this.midiAccess) {
			this._midiPermissionState = 'granted';
			return this._midiPermissionState;
		}
		try {
			this.midiAccess = await navigator.requestMIDIAccess();
			this._midiPermissionState = 'granted';
		} catch {
			this._midiPermissionState = 'denied';
		}
		return this._midiPermissionState;
	}

	async listMidiInputs(): Promise<MidiDevice[]> {
		const access = this.ensureMidiAccess();
		if (!access) return [];
		const devices: MidiDevice[] = [];
		let index = 0;
		access.inputs.forEach((input) => {
			devices.push({ index: index++, name: input.name ?? `Input ${index}` });
		});
		return devices;
	}

	async listMidiOutputs(): Promise<MidiDevice[]> {
		const access = this.ensureMidiAccess();
		if (!access) return [];
		const devices: MidiDevice[] = [];
		let index = 0;
		access.outputs.forEach((output) => {
			devices.push({ index: index++, name: output.name ?? `Output ${index}` });
		});
		return devices;
	}

	async refreshMidiDevices(): Promise<void> {
		if (this._midiPermissionState !== 'granted') return;
		// Permission was already granted, so the browser will not re-prompt.
		// Re-acquire so newly connected devices show up; if the user revoked
		// permission via browser settings since the last grant, fall back.
		try {
			this.midiAccess = await navigator.requestMIDIAccess();
		} catch {
			this.midiAccess = null;
			this._midiPermissionState = 'denied';
		}
	}

	async startRouting(inputIdx: number, outputIndices: number[]): Promise<void> {
		this.ensureInit();

		// --- Guitar audio input mode ---
		if (inputIdx === GUITAR_AUDIO_SENTINEL) {
			await this.startGuitarCapture(outputIndices);
			return;
		}

		const access = this.ensureMidiAccess();
		if (!access) {
			// No Web MIDI (permission not granted, or unsupported browser):
			// still allow "running" for virtual/keyboard input.
			this._isRunning = true;
			return;
		}

		// Resolve input device
		const inputs = Array.from(access.inputs.values());
		if (inputIdx >= 0 && inputIdx < inputs.length) {
			this.activeInput = inputs[inputIdx];
		}

		// Resolve output devices
		const outputs = Array.from(access.outputs.values());
		this.activeOutputs = outputIndices
			.filter((i) => i >= 0 && i < outputs.length)
			.map((i) => outputs[i]);

		// Wire up MIDI message handler
		if (this.activeInput) {
			const self = this;
			const outs = this.activeOutputs;
			this.activeInput.onmidimessage = (event: MIDIMessageEvent) => {
				if (!event.data || event.data.length < 2) return;
				const status = event.data[0] & 0xf0;
				const note = event.data[1];
				const velocity = event.data.length > 2 ? event.data[2] : 0;

				let resultNotes: number[] = [];

				if (status === 0x90 && velocity > 0) {
					try {
						resultNotes = engine.note_on(note);
						const sorted = self.sortVoices(resultNotes);
						for (let i = 0; i < sorted.length; i++) {
							if (outs.length > 0) {
								outs[i % outs.length].send([0x90, sorted[i], velocity]);
							}
							// WebAudio synth so the engine harmonies are
							// audible even without an external MIDI out.
							embedAudio.noteOn(sorted[i], velocity);
						}
						// Feed the player input to the Companion so canon +
						// counterpoint lanes fire delayed / subdivided
						// emissions. Without this hook the v1.2.0 web build
						// would only fire companion voices for the
						// Computer-Keyboard virtual input path, not real
						// MIDI hardware.
						if (companion) {
							try {
								const opsJson = companion.on_note_on(note, velocity, 0);
								self.dispatchOpsJson(opsJson);
								logCompanionEvent('note_on (midi)', {
									player_note: note,
									velocity,
									immediate_ops: self.safeParse(opsJson)
								});
							} catch {
								/* best-effort */
							}
						}
					} catch {
						/* give up on this note */
					}
				} else if (status === 0x80 || (status === 0x90 && velocity === 0)) {
					try {
						resultNotes = engine.note_off(note);
						const sorted = self.sortVoices(resultNotes);
						for (let i = 0; i < sorted.length; i++) {
							if (outs.length > 0) {
								outs[i % outs.length].send([0x80, sorted[i], 0]);
							}
							embedAudio.noteOff(sorted[i]);
						}
						if (companion) {
							try {
								const opsJson = companion.on_note_off(note, 0);
								self.dispatchOpsJson(opsJson);
								logCompanionEvent('note_off (midi)', {
									player_note: note,
									immediate_ops: self.safeParse(opsJson)
								});
							} catch {
								/* best-effort */
							}
						}
					} catch {
						/* give up */
					}
				} else {
					// Pass through other MIDI messages (CC, pitch bend, etc.)
					for (const output of outs) {
						output.send(Array.from(event.data));
					}
				}
			};
		}

		this._isRunning = true;
		// Apply current detune on start
		if (this._detuneCents !== 0) {
			this.sendPitchBend();
		}
	}

	/**
	 * Start guitar audio capture with pitch detection.
	 * Routes detected notes through the harmony engine via injectNoteOn/Off.
	 */
	private async startGuitarCapture(outputIndices: number[]): Promise<void> {
		// Resolve MIDI outputs for sending harmony notes
		const access = this.ensureMidiAccess();
		if (access) {
			const outputs = Array.from(access.outputs.values());
			this.activeOutputs = outputIndices
				.filter((i) => i >= 0 && i < outputs.length)
				.map((i) => outputs[i]);
		}

		this.guitarCapture = new GuitarAudioCapture();
		const self = this;

		guitar.detecting = true;

		// Set initial gate values from store (noise gate only)
		this.guitarCapture.noiseGateThreshold = guitar.noiseGateThreshold;
		this.guitarCapture.noiseGateEnabled = guitar.noiseGateEnabled;
		this.guitarCapture.clarityGateEnabled = false;

		// Read device/channel directly from guitar store (not stale adapter properties)
		// because the user may have changed settings after the last syncDevice() call
		const deviceId = guitar.selectedDeviceId || this._guitarDeviceId;
		const channelIndex = Math.max(0, guitar.selectedChannel - 1); // 1-indexed → 0-indexed
		console.log(`[wasm] startGuitarCapture: device='${deviceId}' channel=${channelIndex} (store.selectedChannel=${guitar.selectedChannel})`);

		await this.guitarCapture.start(
			deviceId,
			channelIndex,
			{
				onNoteOn(note: number, velocity: number) {
					self.injectNoteOn(note, velocity).catch(() => {});
				},
				onNoteOff(note: number) {
					self.injectNoteOff(note).catch(() => {});
				},
				onDetection(info) {
					// Sync noise gate + actual channel from capture every frame
					if (self.guitarCapture) {
						self.guitarCapture.noiseGateThreshold = guitar.noiseGateThreshold;
						self.guitarCapture.noiseGateEnabled = guitar.noiseGateEnabled;
						guitar.activeChannel = self.guitarCapture.actualChannel + 1;
					}

					// Push signal data for graphs
					guitar.pushSignalFrame(info.rms, info.clarity);

					if (info.frequency !== null) {
						guitar.currentNote = info.noteName;
						guitar.confidence = Math.round(info.clarity * 100);
						guitar.velocity = Math.round(info.rms * 800);
					} else {
						guitar.currentNote = '';
						guitar.confidence = 0;
					}
				}
			}
		);

		// Update UI with the actual channel being used (may differ if device has fewer channels)
		guitar.activeChannel = this.guitarCapture.actualChannel + 1; // 1-indexed for display
		this._isRunning = true;
	}

	async stopRouting(): Promise<void> {
		// Stop guitar audio capture if active
		if (this.guitarCapture) {
			await this.guitarCapture.stop();
			this.guitarCapture = null;
			guitar.detecting = false;
			guitar.currentNote = '';
			guitar.confidence = 0;
			guitar.velocity = 0;
		}

		// Send All-Notes-Off (CC 123) to every active output to prevent stuck notes
		for (const output of this.activeOutputs) {
			try {
				output.send([0xb0, 123, 0]); // CC#123 = All Notes Off
			} catch {
				// Output may already be disconnected
			}
		}

		// Clear engine's tracked note state
		if (engine) {
			try {
				engine.clear_notes();
			} catch {
				// Engine may not be initialized
			}
		}

		// Disconnect MIDI input handler
		if (this.activeInput) {
			this.activeInput.onmidimessage = null;
			this.activeInput = null;
		}
		this.activeOutputs = [];
		this._isRunning = false;
		this.stopNotePolling();
	}

	// Per-voice output routing is native-only for now. The browser path
	// has no backend to route to — tracked in the in-memory table below
	// so UI reads/writes round-trip consistently until the WASM engine
	// gains a synth + MIDI dispatcher of its own.
	private _voiceOutputs: VoiceOutputTarget[] = Array.from(
		{ length: MAX_VOICES },
		() => ({ kind: 'synth' })
	);

	async setVoiceOutput(voiceIdx: number, target: VoiceOutputTarget): Promise<void> {
		if (voiceIdx < 0 || voiceIdx >= MAX_VOICES) {
			throw new Error(`voiceIdx ${voiceIdx} out of range (0..${MAX_VOICES - 1})`);
		}
		this._voiceOutputs[voiceIdx] = target;
	}

	async getVoiceOutputs(): Promise<VoiceOutputTarget[]> {
		return this._voiceOutputs.slice();
	}

	async injectNoteOn(note: number, velocity?: number): Promise<number[]> {
		this.ensureInit();
		try {
			const result = engine.note_on(note);
			const sorted = this.sortVoices(result ?? [note]);
			const vel = velocity ?? 100;
			for (let i = 0; i < sorted.length; i++) {
				if (this.activeOutputs.length > 0) {
					this.activeOutputs[i % this.activeOutputs.length].send([0x90, sorted[i], vel]);
				}
				// Web Audio synth — gives the browser path audible
				// playback even without a connected MIDI output device
				// (the desktop Tauri side has its own Rust synth).
				embedAudio.noteOn(sorted[i], vel);
			}
			// Feed the player input to the Companion so the Canon and
			// Counterpoint lanes can fire delayed / subdivided
			// emissions. Immediate-fire ops (Species 1 at delay 0) come
			// back here; longer-delayed ones drain via the tick loop.
			if (companion) {
				try {
					const opsJson = companion.on_note_on(note, vel, 0);
					this.dispatchOpsJson(opsJson);
					logCompanionEvent('note_on (virtual)', {
						player_note: note,
						velocity: vel,
						immediate_ops: this.safeParse(opsJson)
					});
				} catch {
					/* ignore — companion is best-effort in browser */
				}
			}
			return sorted;
		} catch {
			return [note];
		}
	}

	private safeParse(json: string): unknown {
		try {
			return JSON.parse(json);
		} catch {
			return json;
		}
	}

	async injectNoteOff(note: number): Promise<number[]> {
		this.ensureInit();
		try {
			const result = engine.note_off(note);
			const sorted = this.sortVoices(result ?? [note]);
			for (let i = 0; i < sorted.length; i++) {
				if (this.activeOutputs.length > 0) {
					this.activeOutputs[i % this.activeOutputs.length].send([0x80, sorted[i], 0]);
				}
				embedAudio.noteOff(sorted[i]);
			}
			if (companion) {
				try {
					const opsJson = companion.on_note_off(note, 0);
					this.dispatchOpsJson(opsJson);
					logCompanionEvent('note_off (virtual)', {
						player_note: note,
						immediate_ops: this.safeParse(opsJson)
					});
				} catch {
					/* ignore */
				}
			}
			return sorted;
		} catch {
			return [note];
		}
	}

	async getNoteState(): Promise<NoteState> {
		this.ensureInit();
		try {
			const raw = engine.get_note_state();
			const engineHarmony: number[] = raw?.harmony_notes ?? [];
			// Merge companion-emitted notes (Canon + Counterpoint
			// lanes) into harmonyNotes so the ActiveNotes strip + the
			// Piano + Fretboard highlight them. The Tauri build gets
			// these from the router thread's note-state aggregation;
			// in WASM we accumulate them in `activeCompanionNotes` via
			// dispatchOpsJson. Set-merge dedups overlap with the
			// engine's own harmony output.
			const merged = new Set<number>(engineHarmony);
			for (const n of activeCompanionNotes) merged.add(n);
			return {
				inputNotes: raw?.input_notes ?? [],
				harmonyNotes: Array.from(merged),
				borrowedNotes: raw?.borrowed_notes ?? [],
				chordName: raw?.chord_name ?? '',
				lastBorrowedFrom: raw?.last_borrowed_from ?? '',
				currentKey: engine.current_key?.() ?? 'C'
			};
		} catch {
			return {
				inputNotes: [],
				harmonyNotes: Array.from(activeCompanionNotes),
				borrowedNotes: [],
				chordName: '',
				lastBorrowedFrom: '',
				currentKey: 'C'
			};
		}
	}

	onNoteUpdate(callback: (state: NoteState) => void): () => void {
		this.noteUpdateCallback = callback;
		this.startNotePolling();

		return () => {
			this.noteUpdateCallback = null;
			this.stopNotePolling();
		};
	}

	onKnobCcRaw(_callback: (cc: number, value: number) => void): () => void {
		// Browser/wasm path: raw MIDI CCs aren't currently forwarded
		// from the Web MIDI input handler. Could be wired up in a
		// future iteration to enable MIDI Learn in the browser; for
		// now, no-op.
		return () => {};
	}

	/**
	 * Poll for note state changes using requestAnimationFrame.
	 * WASM doesn't push events, so we pull at frame rate.
	 */
	private startNotePolling(): void {
		if (this.pollingHandle !== null) return;

		const poll = () => {
			if (!this.noteUpdateCallback || !this._isRunning) {
				this.pollingHandle = null;
				return;
			}

			this.getNoteState()
				.then((state) => {
					this.noteUpdateCallback?.(state);
				})
				.catch(() => {
					// Silently ignore polling errors
				});

			this.pollingHandle = requestAnimationFrame(poll);
		};

		this.pollingHandle = requestAnimationFrame(poll);
	}

	/**
	 * Sort notes ascending so lowest note always routes to output 0 (bass),
	 * and each higher voice stays on its consistent output slot.
	 */
	private sortVoices(notes: number[]): number[] {
		return [...notes].sort((a, b) => a - b);
	}

	/**
	 * Convert cents offset to a 14-bit MIDI pitch bend value.
	 * Center = 8192, range depends on pitchBendRangeSemitones (default ±2 = 200 cents).
	 */
	private centsToPitchBend(cents: number): number {
		const maxCents = this.pitchBendRangeSemitones * 100;
		const normalized = Math.max(-1, Math.min(1, cents / maxCents));
		return Math.round(8192 + normalized * 8191);
	}

	/**
	 * Send pitch bend to all active outputs on channel 0.
	 */
	private sendPitchBend(): void {
		const bend = this.centsToPitchBend(this._detuneCents);
		const lsb = bend & 0x7f;
		const msb = (bend >> 7) & 0x7f;
		for (const output of this.activeOutputs) {
			output.send([0xe0, lsb, msb]);
		}
	}

	async listAudioDevices(): Promise<string[]> {
		if (typeof navigator === 'undefined' || !navigator.mediaDevices) {
			return [];
		}
		try {
			// Request permission so labels are populated
			const stream = await navigator.mediaDevices.getUserMedia({ audio: true });
			stream.getTracks().forEach((t) => t.stop());

			const allDevices = await navigator.mediaDevices.enumerateDevices();
			return allDevices
				.filter((d) => d.kind === 'audioinput')
				.map((d) => d.label || `Audio Input ${d.deviceId.slice(0, 8)}`);
		} catch {
			return [];
		}
	}

	async setGuitarDevice(deviceName: string, channel: number): Promise<void> {
		// Store for use when startRouting is called with guitar sentinel
		// Channel arrives 0-indexed from syncDevice()
		this._guitarDeviceId = deviceName;
		this._guitarChannel = Math.max(0, channel);
	}

	async setGuitarConfig(_config: GuitarConfig): Promise<void> {
		// Browser-side pitch detection doesn't use these DSP params,
		// but accept silently so shared UI code doesn't error.
	}

	/**
	 * Set global detune in cents and send pitch bend to all outputs.
	 * Assumes standard ±2 semitone pitch bend range in your DAW.
	 */
	setDetune(cents: number): void {
		this._detuneCents = cents;
		if (this._isRunning) {
			this.sendPitchBend();
		}
	}

	getDetune(): number {
		return this._detuneCents;
	}

	// -- Transport + metronome (browser implementation, issue #55) --
	//
	// JS-side clock + Web Audio click. Native parity isn't sample-accurate
	// — setInterval drift over long playback isn't corrected — but it's
	// fine for a demo metronome and matches what users expect from the
	// /app embed on the marketing site.
	//
	// Click frequencies + duration match Rust's audio_clock.rs constants
	// so native + web sound identical.

	private _transport = {
		running: false,
		bpm: 120,
		beatsPerBar: 4,
		beatUnit: 4,
		beatInBar: 0,
		totalBeat: 0,
		bar: 0,
		metronomeEnabled: false
	};
	private _audioCtx: AudioContext | null = null;
	private _clockTimer: ReturnType<typeof setInterval> | null = null;

	private ensureAudioContext(): AudioContext | null {
		if (this._audioCtx) return this._audioCtx;
		try {
			const Ctor =
				window.AudioContext ??
				(window as unknown as { webkitAudioContext?: typeof AudioContext })
					.webkitAudioContext;
			if (!Ctor) return null;
			this._audioCtx = new Ctor();
			return this._audioCtx;
		} catch {
			return null;
		}
	}

	private playClick(downbeat: boolean) {
		const ctx = this.ensureAudioContext();
		if (!ctx) return;
		// Match Rust constants: CLICK_FREQ_DOWNBEAT=900, CLICK_FREQ_OFFBEAT=600,
		// CLICK_SECS=0.015, CLICK_GAIN=0.30 (audio_clock.rs).
		const freq = downbeat ? 900 : 600;
		const dur = 0.015;
		const now = ctx.currentTime;
		const osc = ctx.createOscillator();
		const gain = ctx.createGain();
		osc.type = 'sine';
		osc.frequency.value = freq;
		// Fast attack + exponential decay over CLICK_SECS.
		gain.gain.setValueAtTime(0, now);
		gain.gain.linearRampToValueAtTime(0.3, now + 0.001);
		gain.gain.exponentialRampToValueAtTime(0.0001, now + dur);
		osc.connect(gain).connect(ctx.destination);
		osc.start(now);
		osc.stop(now + dur + 0.005);
	}

	private tick() {
		// Increment counters first so beat 0 fires at start.
		const t = this._transport;
		// Notify the store + render the pip / label first, then synthesize
		// the click. Order doesn't strictly matter for ear-correctness
		// since the click is short, but doing the UI update first means
		// visual + audio land together on the same frame.
		transport.applyBeatUpdate({
			totalBeat: t.totalBeat,
			beatInBar: t.beatInBar,
			bar: t.bar,
			bpm: t.bpm,
			running: t.running
		});
		if (t.metronomeEnabled) {
			this.playClick(t.beatInBar === 0);
		}
		// Advance.
		t.totalBeat += 1;
		t.beatInBar += 1;
		if (t.beatInBar >= t.beatsPerBar) {
			t.beatInBar = 0;
			t.bar += 1;
		}
	}

	private startClock() {
		this.stopClock();
		const intervalMs = 60_000 / this._transport.bpm;
		this._clockTimer = setInterval(() => this.tick(), intervalMs);
		// Fire one beat immediately so the UI shows a pip on the first
		// beat without waiting for the interval to elapse.
		this.tick();
	}

	private stopClock() {
		if (this._clockTimer !== null) {
			clearInterval(this._clockTimer);
			this._clockTimer = null;
		}
	}

	async getTransportState(): Promise<TransportState> {
		return {
			running: this._transport.running,
			bpm: this._transport.bpm,
			beatsPerBar: this._transport.beatsPerBar,
			beatUnit: this._transport.beatUnit,
			sampleRate: 48_000,
			samplePos: 0,
			beatPosition: 0,
			bar: this._transport.bar,
			metronomeEnabled: this._transport.metronomeEnabled
		};
	}

	async transportPlay(): Promise<void> {
		if (this._transport.running) return;
		// User gesture is implicit — play() is called from a button click,
		// which satisfies the browser autoplay policy.
		this.ensureAudioContext();
		const ctx = this._audioCtx;
		if (ctx && ctx.state === 'suspended') {
			try {
				await ctx.resume();
			} catch {
				// suspended → resume can fail if not in a user-gesture
			}
		}
		this._transport.running = true;
		this._transport.beatInBar = 0;
		this._transport.totalBeat = 0;
		this._transport.bar = 0;
		this.startClock();
	}

	async transportStop(): Promise<void> {
		this._transport.running = false;
		this.stopClock();
		// Push a final state update so pips drop the active state.
		transport.applyBeatUpdate({
			totalBeat: this._transport.totalBeat,
			beatInBar: this._transport.beatInBar,
			bar: this._transport.bar,
			bpm: this._transport.bpm,
			running: false
		});
	}

	async transportReset(): Promise<void> {
		this._transport.beatInBar = 0;
		this._transport.totalBeat = 0;
		this._transport.bar = 0;
		transport.applyBeatUpdate({
			totalBeat: 0,
			beatInBar: 0,
			bar: 0,
			bpm: this._transport.bpm,
			running: this._transport.running
		});
	}

	async setBpm(bpm: number): Promise<void> {
		const clamped = Math.max(20, Math.min(400, bpm));
		this._transport.bpm = clamped;
		// If the clock is running, restart it so the new interval takes
		// effect on the next tick.
		if (this._transport.running) {
			this.startClock();
		}
	}

	async setTimeSignature(beatsPerBar: number, beatUnit: number): Promise<void> {
		this._transport.beatsPerBar = Math.max(1, Math.min(16, beatsPerBar));
		this._transport.beatUnit = beatUnit;
		// Reset beatInBar so we don't desync the pip animation.
		this._transport.beatInBar = 0;
	}

	async setMetronomeEnabled(enabled: boolean): Promise<void> {
		this._transport.metronomeEnabled = enabled;
		// If user enables the click without first hitting Play, we still
		// need an AudioContext primed so the next tick can fire a click.
		if (enabled) this.ensureAudioContext();
	}

	// -- Synth (no built-in synth in browser; stubs) --

	async getSynthState() {
		return {
			enabled: false,
			waveform: 0,
			attackMs: 5,
			decayMs: 120,
			sustain: 0.7,
			releaseMs: 250,
			cutoffHz: 6000,
			resonance: 0.2,
			masterGain: 0.25
		};
	}
	async setSynthEnabled(_enabled: boolean): Promise<void> {
		// embedAudio is permanently enabled in the browser; toggling it
		// off would silence the user's own notes too. Tauri toggles a
		// dedicated Rust synth that runs alongside MIDI-out routing.
		// No-op until embedAudio grows a mute toggle.
	}
	async setSynthWaveform(value: number): Promise<void> {
		// EngineState.waveform: 0 = sine, 1 = triangle, 2 = sawtooth,
		// 3 = square — matches contrapunk-audio's Waveform enum.
		const map = ['sine', 'triangle', 'sawtooth', 'square'] as const;
		const w = map[Math.max(0, Math.min(3, Math.round(value)))];
		try {
			embedAudio.setWaveform(w);
		} catch {
			/* embedAudio may not be ready before init */
		}
	}
	async setSynthAttackMs(ms: number): Promise<void> {
		try {
			embedAudio.setAttackMs(ms);
		} catch {
			/* */
		}
	}
	async setSynthDecayMs(ms: number): Promise<void> {
		try {
			embedAudio.setDecayMs(ms);
		} catch {
			/* */
		}
	}
	async setSynthSustain(level: number): Promise<void> {
		try {
			embedAudio.setSustain(level);
		} catch {
			/* */
		}
	}
	async setSynthReleaseMs(ms: number): Promise<void> {
		try {
			embedAudio.setReleaseMs(ms);
		} catch {
			/* */
		}
	}
	async setSynthCutoffHz(hz: number): Promise<void> {
		try {
			embedAudio.setCutoffHz(hz);
		} catch {
			/* */
		}
	}
	async setSynthResonance(value: number): Promise<void> {
		try {
			embedAudio.setResonance(value);
		} catch {
			/* */
		}
	}
	async setSynthMasterGain(value: number): Promise<void> {
		try {
			embedAudio.setMasterGain(Math.max(0, Math.min(1, value)));
		} catch {
			/* embedAudio may not be ready before init */
		}
	}

	// -- FX (browser uses WebAudio chain in embedAudio) --

	async getReverbState() {
		return { enabled: false, mix: 0.3, roomSize: 0.7, damping: 0.5 };
	}
	async setReverbEnabled(enabled: boolean): Promise<void> {
		// embedAudio has no explicit enable; routing mix to 0 silences
		// the reverb send. Tauri's Rust reverb has a real bypass.
		if (!enabled) {
			try {
				embedAudio.setReverbMix(0);
			} catch {
				/* */
			}
		}
	}
	async setReverbMix(value: number): Promise<void> {
		try {
			embedAudio.setReverbMix(Math.max(0, Math.min(1, value)));
		} catch {
			/* */
		}
	}
	async setReverbRoomSize(value: number): Promise<void> {
		try {
			embedAudio.setReverbRoomSize(value);
		} catch {
			/* */
		}
	}
	async setReverbDamping(value: number): Promise<void> {
		try {
			embedAudio.setReverbDamping(value);
		} catch {
			/* */
		}
	}

	async getDelayState() {
		return { enabled: false, mix: 0.3, timeMs: 375, feedback: 0.35 };
	}
	async setDelayEnabled(enabled: boolean): Promise<void> {
		if (!enabled) {
			try {
				embedAudio.setDelayMix(0);
			} catch {
				/* */
			}
		}
	}
	async setDelayMix(value: number): Promise<void> {
		try {
			embedAudio.setDelayMix(Math.max(0, Math.min(1, value)));
		} catch {
			/* */
		}
	}
	async setDelayTimeMs(ms: number): Promise<void> {
		try {
			embedAudio.setDelayTime(Math.max(0, ms) / 1000);
		} catch {
			/* */
		}
	}
	async setDelayFeedback(value: number): Promise<void> {
		try {
			embedAudio.setDelayFeedback(Math.max(0, Math.min(0.99, value)));
		} catch {
			/* */
		}
	}

	// -- Chain topology + CLAP (no native host in browser; stubs) --

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

	private stopNotePolling(): void {
		if (this.pollingHandle !== null) {
			cancelAnimationFrame(this.pollingHandle);
			this.pollingHandle = null;
		}
	}

	async listPresets(): Promise<Preset[]> {
		this.ensureInit();
		try {
			const raw = engine.list_presets();
			return (raw ?? []).map(
				(p: Record<string, unknown>, i: number): Preset => ({
					index: i,
					name: (p.name as string) ?? '',
					persona: (p.persona as string) ?? '',
					genre: (p.genre as string) ?? '',
					isBuiltin: (p.is_builtin as boolean) ?? true
				})
			);
		} catch {
			return [];
		}
	}

	async loadPreset(name: string): Promise<void> {
		this.ensureInit();
		try {
			engine.load_preset(name);
		} catch (e) {
			throw new Error(`Failed to load preset: ${e}`);
		}
	}

	async savePreset(name: string): Promise<void> {
		this.ensureInit();
		try {
			engine.save_preset(name);
		} catch (e) {
			throw new Error(`Failed to save preset: ${e}`);
		}
	}

	async deletePreset(name: string): Promise<void> {
		this.ensureInit();
		try {
			engine.delete_preset(name);
		} catch (e) {
			throw new Error(`Failed to delete preset: ${e}`);
		}
	}

}
