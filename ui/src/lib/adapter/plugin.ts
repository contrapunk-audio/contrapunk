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
	MidiDevice,
	NoteState,
	Preset,
	TransportState
} from './types';

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

export class PluginAdapter implements ContrapunkAdapter {
	private noteUpdateCallback: ((state: NoteState) => void) | null = null;
	private _detuneCents = 0;
	private _isReady = false;

	async init(): Promise<void> {
		// Register listener for messages from the Rust plugin host
		window.plugin.listen((msg: string) => {
			try {
				const data = JSON.parse(msg);
				if (data.type === 'paramsUpdate') {
					currentParams = data;
				} else if (data.type === 'noteUpdate' && this.noteUpdateCallback) {
					this.noteUpdateCallback({
						inputNotes: data.inputNotes ?? [],
						harmonyNotes: data.harmonyNotes ?? [],
						borrowedNotes: data.borrowedNotes ?? [],
						chordName: data.chordName ?? '',
						lastBorrowedFrom: data.lastBorrowedFrom ?? '',
						currentKey: data.currentKey ?? 'C'
					});
				}
			} catch {
				/* ignore parse errors */
			}
		});

		// Signal ready — the Rust side will respond with initial params
		window.plugin.send(JSON.stringify({ type: 'ready' }));
		this._isReady = true;
	}

	private send(type: string, value: unknown): void {
		window.plugin.send(JSON.stringify({ type, value }));
	}

	async getEngineState(): Promise<EngineState> {
		return {
			key: (currentParams.key as string) ?? 'C',
			mode: (currentParams.mode as string) ?? 'DiatonicThirds',
			modeNumber: 0,
			scaleMode: 'Ionian',
			octaveMode: (currentParams.octaveMode as string) ?? 'None',
			voiceLeadingEnabled: (currentParams.voiceLeading as boolean) ?? false,
			voiceLeadingStyle: 'Free',
			interchangeEnabled: false,
			interchangeRange: 3,
			voicePosition: (currentParams.voicePosition as number) ?? 0,
			voiceCount: (currentParams.voiceCount as number) ?? 2,
			autoKey: (currentParams.autoKey as boolean) ?? false,
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

	async setVoiceLeading(enabled: boolean, _style: string): Promise<void> {
		this.send('setVoiceLeading', enabled);
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

	// -- MIDI devices (DAW handles routing in plugin mode) --

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

	// -- Real-time state --

	async getNoteState(): Promise<NoteState> {
		return {
			inputNotes: [],
			harmonyNotes: [],
			borrowedNotes: [],
			chordName: '',
			lastBorrowedFrom: '',
			currentKey: (currentParams.key as string) ?? 'C'
		};
	}

	onNoteUpdate(callback: (state: NoteState) => void): () => void {
		this.noteUpdateCallback = callback;
		return () => {
			this.noteUpdateCallback = null;
		};
	}

	// -- Virtual Input (not applicable in plugin mode) --

	async injectNoteOn(note: number, _velocity?: number): Promise<number[]> {
		return [note];
	}

	async injectNoteOff(note: number): Promise<number[]> {
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
			bar: 0
		};
	}

	async transportPlay(): Promise<void> {}
	async transportStop(): Promise<void> {}
	async transportReset(): Promise<void> {}
	async setBpm(_bpm: number): Promise<void> {}
	async setTimeSignature(_beatsPerBar: number, _beatUnit: number): Promise<void> {}
}
