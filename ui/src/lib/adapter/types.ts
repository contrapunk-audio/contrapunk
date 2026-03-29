/**
 * Platform Adapter Layer - Shared Type Definitions
 *
 * Defines the ContrapunkAdapter interface and all related types.
 * Components import from this module and never call Tauri or WASM directly.
 */

/** Represents a MIDI device (input or output). */
export interface MidiDevice {
	index: number;
	name: string;
}

/** Snapshot of the harmony engine configuration. */
export interface EngineState {
	key: string;
	mode: string;
	modeNumber: number;
	scaleMode: string;
	octaveMode: string;
	voiceLeadingEnabled: boolean;
	voiceLeadingStyle: string;
	interchangeEnabled: boolean;
	interchangeRange: number;
	voicePosition: number;
	voiceCount: number;
	isRunning: boolean;
}

/** Real-time note state emitted during MIDI routing. */
export interface NoteState {
	inputNotes: number[];
	harmonyNotes: number[];
	borrowedNotes: number[];
	chordName: string;
	lastBorrowedFrom: string;
}

/** Humanization engine configuration. */
export interface HumanizeState {
	enabled: boolean;
	jitterEnabled: boolean;
	jitterMinMs: number;
	jitterMaxMs: number;
	velocityEnabled: boolean;
	velocityVariation: number;
	durationEnabled: boolean;
	durationVariationMs: number;
	swingEnabled: boolean;
	swingAmount: number;
	bpm: number;
	metronomeEnabled: boolean;
}

/** Preset metadata. */
export interface Preset {
	index: number;
	name: string;
	persona: string;
	genre: string;
	isBuiltin: boolean;
}

/**
 * Unified adapter interface for communicating with the Rust backend.
 *
 * Svelte components call these methods without knowing whether the
 * backend is reached via Tauri IPC (desktop) or WASM direct calls (browser).
 */
export interface ContrapunkAdapter {
	// -- Initialization --

	/** Initialize the backend (WASM init or Tauri readiness check). */
	init(): Promise<void>;

	// -- Engine state --

	/** Get a snapshot of the current engine configuration. */
	getEngineState(): Promise<EngineState>;

	/** Set the musical key (e.g. "C", "Db", "F#"). */
	setKey(key: string): Promise<void>;

	/** Set the harmony mode (e.g. "PassThrough", "DiatonicThirds"). */
	setMode(mode: string): Promise<void>;

	/** Set the scale mode (e.g. "Ionian", "Dorian", "HarmonicMinor"). */
	setScaleMode(mode: string): Promise<void>;

	/** Set the octave mode (e.g. "None", "Spread", "Mirror"). */
	setOctaveMode(mode: string): Promise<void>;

	/** Configure voice leading (enabled flag + style). */
	setVoiceLeading(enabled: boolean, style: string): Promise<void>;

	/** Configure modal interchange (enabled flag + borrowing range). */
	setInterchange(enabled: boolean, range: number): Promise<void>;

	/** Set the voice position (which output slot carries the melody). */
	setVoicePosition(position: number): Promise<void>;

	/** Set the number of output voices (1 = melody only, 2+ = melody + harmonies). */
	setVoiceCount(count: number): Promise<void>;

	// -- Humanization --

	/** Get the current humanization configuration. */
	getHumanizeState(): Promise<HumanizeState>;

	/** Update humanization configuration (partial update). */
	setHumanizeConfig(config: Partial<HumanizeState>): Promise<void>;

	// -- MIDI devices --

	/** List available MIDI input devices. */
	listMidiInputs(): Promise<MidiDevice[]>;

	/** List available MIDI output devices. */
	listMidiOutputs(): Promise<MidiDevice[]>;

	/** Re-enumerate MIDI devices (forces a fresh scan). */
	refreshMidiDevices(): Promise<void>;

	// -- Routing --

	/** Start MIDI routing from the given input to the given outputs. */
	startRouting(inputIdx: number, outputIndices: number[]): Promise<void>;

	/** Stop the currently active MIDI routing. */
	stopRouting(): Promise<void>;

	// -- Real-time state --

	/** Get the current note state snapshot. */
	getNoteState(): Promise<NoteState>;

	/**
	 * Subscribe to real-time note updates.
	 * Returns an unsubscribe function.
	 */
	onNoteUpdate(callback: (state: NoteState) => void): () => void;

	// -- Virtual Input (keyboard, generator) --

	/** Inject a Note On event directly (for virtual inputs like computer keyboard). */
	injectNoteOn(note: number, velocity?: number): Promise<number[]>;

	/** Inject a Note Off event directly (for virtual inputs). */
	injectNoteOff(note: number): Promise<number[]>;

	// -- Presets --

	/** List all available presets (builtins + custom). */
	listPresets(): Promise<Preset[]>;

	/** Load a preset by name and apply it to the engine. */
	loadPreset(name: string): Promise<void>;

	/** Save the current engine config as a custom preset. */
	savePreset(name: string): Promise<void>;

	/** Delete a custom preset by name. */
	deletePreset(name: string): Promise<void>;

	// -- Detune --

	/** Set global detune in cents (sends pitch bend to all outputs). */
	setDetune(cents: number): void;

	/** Get the current detune value in cents. */
	getDetune(): number;
}
