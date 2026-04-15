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
	HumanizeState,
	MidiDevice,
	NoteState,
	Preset
} from './types';
import { GuitarAudioCapture } from '$lib/audio/guitarCapture';
import { guitar } from '$lib/stores/guitar.svelte';
import { beat } from '$lib/stores/beat.svelte';

/**
 * Dynamically imported WASM module.
 * Resolved during init() to avoid top-level import failures
 * when the wasm-pkg hasn't been built yet.
 */
// eslint-disable-next-line @typescript-eslint/no-explicit-any
let wasmModule: any = null;
// eslint-disable-next-line @typescript-eslint/no-explicit-any
let engine: any = null;

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
	/** requestAnimationFrame handle for the beat/humanize tick loop. */
	private tickRafHandle: number | null = null;

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
			this.initialized = true;
			// Kick off the frame-driven tick loop. Runs for the lifetime
			// of the adapter — beat clock + metronome stay live even when
			// no MIDI routing is active so the UI beat indicator works.
			this.startTickLoop();
		} catch (e) {
			throw new Error(`Failed to initialize WASM: ${e}`);
		}
	}

	/**
	 * Drive `engine.tick(performance.now())` on every animation frame and
	 * fan the result out to the beat store + any active MIDI outputs (for
	 * metronome clicks + delayed humanized notes).
	 */
	private startTickLoop(): void {
		if (this.tickRafHandle !== null) return;
		const loop = () => {
			if (!engine) {
				this.tickRafHandle = null;
				return;
			}
			try {
				const result = engine.tick(performance.now()) as {
					beat_position?: number;
					beat_number?: number;
					beat_crossed?: number | null;
					metronome_on?: number[] | null;
					metronome_off?: number[] | null;
					scheduled_notes?: { port: number; bytes: number[] }[];
					humanize_enabled?: boolean;
					running?: boolean;
					bpm?: number;
				};

				if (result) {
					beat.beatPosition = result.beat_position ?? 0;
					beat.beatNumber = result.beat_number ?? 0;
					beat.running = result.running ?? false;
					beat.bpm = result.bpm ?? 120;
					if (result.beat_crossed !== null && result.beat_crossed !== undefined) {
						beat.lastCrossedAt = performance.now();
						beat.pulse = beat.pulse + 1;
					}
					// Metronome click bytes → route to the first active MIDI output.
					// We use the first active output so the click rides along with
					// whatever synth the user has selected, matching Tauri behavior.
					const metroTarget = this.activeOutputs[0];
					if (metroTarget && Array.isArray(result.metronome_on)) {
						try {
							metroTarget.send(result.metronome_on);
						} catch {
							/* output may have disconnected */
						}
					}
					if (metroTarget && Array.isArray(result.metronome_off)) {
						try {
							metroTarget.send(result.metronome_off);
						} catch {
							/* ignore */
						}
					}
					// Drain any humanized harmony notes whose delay elapsed.
					if (Array.isArray(result.scheduled_notes)) {
						for (const s of result.scheduled_notes) {
							const out = this.activeOutputs[s.port % Math.max(1, this.activeOutputs.length)];
							if (out && Array.isArray(s.bytes)) {
								try {
									out.send(s.bytes);
								} catch {
									/* ignore */
								}
							}
						}
					}
				}
			} catch {
				// Never let a tick error kill the loop — the user can keep
				// using the UI even if one frame fails to serialize.
			}
			this.tickRafHandle = requestAnimationFrame(loop);
		};
		this.tickRafHandle = requestAnimationFrame(loop);
	}

	/** Stop the tick loop. Called from `destroy()` on teardown. */
	private stopTickLoop(): void {
		if (this.tickRafHandle !== null) {
			cancelAnimationFrame(this.tickRafHandle);
			this.tickRafHandle = null;
		}
	}

	/**
	 * Tear down background loops and resources. Called from the root
	 * layout `onDestroy` hook so hot-reload and page navigation don't
	 * leak the requestAnimationFrame loop or lingering guitar captures.
	 */
	destroy(): void {
		this.stopTickLoop();
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

	async setKey(key: string): Promise<void> {
		this.ensureInit();
		try {
			engine.set_key(key);
		} catch (e) {
			throw new Error(`Failed to set key: ${e}`);
		}
	}

	async setMode(mode: string): Promise<void> {
		this.ensureInit();
		try {
			engine.set_mode(mode);
		} catch (e) {
			throw new Error(`Failed to set mode: ${e}`);
		}
	}

	async setScaleMode(mode: string): Promise<void> {
		this.ensureInit();
		try {
			engine.set_scale_mode(mode);
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
		} catch (e) {
			throw new Error(`Failed to set voice position: ${e}`);
		}
	}

	async setVoiceCount(count: number): Promise<void> {
		this.ensureInit();
		try {
			engine.set_voice_count(count);
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

	async getHumanizeState(): Promise<HumanizeState> {
		this.ensureInit();
		try {
			const raw = engine.get_humanize_config() as Record<string, unknown>;
			return {
				enabled: (raw.enabled as boolean) ?? false,
				jitterEnabled: (raw.jitter_enabled as boolean) ?? false,
				jitterMinMs: (raw.jitter_min_ms as number) ?? 1,
				jitterMaxMs: (raw.jitter_max_ms as number) ?? 10,
				velocityEnabled: (raw.velocity_enabled as boolean) ?? false,
				velocityVariation: (raw.velocity_variation as number) ?? 15,
				durationEnabled: (raw.duration_enabled as boolean) ?? false,
				durationVariationMs: (raw.duration_variation_ms as number) ?? 20,
				swingEnabled: (raw.swing_enabled as boolean) ?? false,
				swingAmount: (raw.swing_amount as number) ?? 0.0,
				bpm: (raw.bpm as number) ?? 120.0,
				metronomeEnabled: (raw.metronome_enabled as boolean) ?? false
			};
		} catch {
			return {
				enabled: false,
				jitterEnabled: false,
				jitterMinMs: 1,
				jitterMaxMs: 10,
				velocityEnabled: false,
				velocityVariation: 15,
				durationEnabled: false,
				durationVariationMs: 20,
				swingEnabled: false,
				swingAmount: 0.0,
				bpm: 120.0,
				metronomeEnabled: false
			};
		}
	}

	async setHumanizeConfig(config: Partial<HumanizeState>): Promise<void> {
		this.ensureInit();
		// Merge onto the current WASM config then push the full object back.
		// The WASM `set_humanize_config` takes a full `HumanizeConfig`, so we
		// read → patch → write to avoid blowing away fields the caller omitted.
		try {
			const current = engine.get_humanize_config() as Record<string, unknown>;
			const merged = { ...current };
			if (config.enabled !== undefined) merged.enabled = config.enabled;
			if (config.jitterEnabled !== undefined) merged.jitter_enabled = config.jitterEnabled;
			if (config.jitterMinMs !== undefined) merged.jitter_min_ms = config.jitterMinMs;
			if (config.jitterMaxMs !== undefined) merged.jitter_max_ms = config.jitterMaxMs;
			if (config.velocityEnabled !== undefined) merged.velocity_enabled = config.velocityEnabled;
			if (config.velocityVariation !== undefined)
				merged.velocity_variation = config.velocityVariation;
			if (config.durationEnabled !== undefined) merged.duration_enabled = config.durationEnabled;
			if (config.durationVariationMs !== undefined)
				merged.duration_variation_ms = config.durationVariationMs;
			if (config.swingEnabled !== undefined) merged.swing_enabled = config.swingEnabled;
			if (config.swingAmount !== undefined) merged.swing_amount = config.swingAmount;
			if (config.bpm !== undefined) merged.bpm = config.bpm;
			if (config.metronomeEnabled !== undefined)
				merged.metronome_enabled = config.metronomeEnabled;
			engine.set_humanize_config(merged);
		} catch (e) {
			throw new Error(`Failed to set humanize config: ${e}`);
		}
	}

	/**
	 * Ensure Web MIDI Access is available. Caches the MIDIAccess instance.
	 */
	private async ensureMidiAccess(): Promise<MIDIAccess | null> {
		if (this.midiAccess) return this.midiAccess;
		if (typeof navigator === 'undefined' || !('requestMIDIAccess' in navigator)) {
			return null;
		}
		try {
			this.midiAccess = await navigator.requestMIDIAccess();
			return this.midiAccess;
		} catch {
			return null;
		}
	}

	async listMidiInputs(): Promise<MidiDevice[]> {
		const access = await this.ensureMidiAccess();
		if (!access) return [];
		const devices: MidiDevice[] = [];
		let index = 0;
		access.inputs.forEach((input) => {
			devices.push({ index: index++, name: input.name ?? `Input ${index}` });
		});
		return devices;
	}

	async listMidiOutputs(): Promise<MidiDevice[]> {
		const access = await this.ensureMidiAccess();
		if (!access) return [];
		const devices: MidiDevice[] = [];
		let index = 0;
		access.outputs.forEach((output) => {
			devices.push({ index: index++, name: output.name ?? `Output ${index}` });
		});
		return devices;
	}

	async refreshMidiDevices(): Promise<void> {
		// Clear cached access so next list call re-enumerates
		this.midiAccess = null;
	}

	async startRouting(inputIdx: number, outputIndices: number[]): Promise<void> {
		this.ensureInit();

		// --- Guitar audio input mode ---
		if (inputIdx === GUITAR_AUDIO_SENTINEL) {
			await this.startGuitarCapture(outputIndices);
			return;
		}

		const access = await this.ensureMidiAccess();
		if (!access) {
			// No Web MIDI: still allow "running" for virtual/keyboard input
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
					// Note On — humanized path: per-voice velocity jitter + delay queue.
					// Deferred harmony notes come out later via tick()'s scheduled_notes drain.
					try {
						const humanized = engine.humanized_note_on(note, velocity) as {
							immediate: { port: number; bytes: number[] }[];
							deferred_count: number;
							input_note: number;
						};
						for (const s of humanized.immediate) {
							const out = outs[s.port % Math.max(1, outs.length)];
							if (out) out.send(s.bytes);
						}
					} catch {
						// Fall back to plain note_on if humanization fails.
						try {
							resultNotes = engine.note_on(note);
							const sorted = self.sortVoices(resultNotes);
							for (let i = 0; i < sorted.length; i++) {
								if (outs.length > 0) {
									outs[i % outs.length].send([0x90, sorted[i], velocity]);
								}
							}
						} catch {
							/* give up on this note */
						}
					}
				} else if (status === 0x80 || (status === 0x90 && velocity === 0)) {
					// Note Off — humanized release path, matches humanized_note_on delays.
					try {
						const humanized = engine.humanized_note_off(note) as {
							immediate: { port: number; bytes: number[] }[];
							deferred_count: number;
							input_note: number;
						};
						for (const s of humanized.immediate) {
							const out = outs[s.port % Math.max(1, outs.length)];
							if (out) out.send(s.bytes);
						}
					} catch {
						try {
							resultNotes = engine.note_off(note);
							const sorted = self.sortVoices(resultNotes);
							for (let i = 0; i < sorted.length; i++) {
								if (outs.length > 0) {
									outs[i % outs.length].send([0x80, sorted[i], 0]);
								}
							}
						} catch {
							/* give up */
						}
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
		const access = await this.ensureMidiAccess();
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
			}
			return sorted;
		} catch {
			return [note];
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
			return {
				inputNotes: raw?.input_notes ?? [],
				harmonyNotes: raw?.harmony_notes ?? [],
				borrowedNotes: raw?.borrowed_notes ?? [],
				chordName: raw?.chord_name ?? '',
				lastBorrowedFrom: raw?.last_borrowed_from ?? '',
				currentKey: engine.current_key?.() ?? 'C'
			};
		} catch {
			return {
				inputNotes: [],
				harmonyNotes: [],
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

	async listAudioOutputDevices(): Promise<{ name: string; is_default: boolean }[]> {
		return [];
	}

	async startAudioOutput(_opts: { deviceId?: string; sampleRate?: number; bufferSize?: number }): Promise<void> {
		console.warn('[contrapunk] audio output not available in browser — use desktop app');
	}

	async stopAudioOutput(): Promise<void> {}

	async isAudioOutputRunning(): Promise<boolean> {
		return false;
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

	// -- Suggestions --

	getSuggestions(): { note: number; score: number }[] {
		if (!this.initialized || !engine) return [];
		try {
			return engine.get_suggestions() ?? [];
		} catch {
			return [];
		}
	}

	setSuggestionWeight(term: string, value: number): void {
		if (!this.initialized || !engine) return;
		try {
			engine.set_suggestion_weight(term, value);
		} catch {
			// silently ignore
		}
	}

	resetSuggestionWeights(): void {
		if (!this.initialized || !engine) return;
		try {
			engine.reset_suggestion_weights();
		} catch {
			// silently ignore
		}
	}
}
