import { Y as derived, Z as ensure_array_like, _ as attr_class, $ as attr_style, a0 as stringify, a1 as attr, e as escape_html } from "../../chunks/index.js";
import { a as adapter, u as ui } from "../../chunks/ui.svelte.js";
import "clsx";
import "@tauri-apps/api/core";
import "@tauri-apps/api/event";
function sameNotes(a, b) {
  if (a.length !== b.length) return false;
  if (a.length === 0) return true;
  const sa = [...a].sort();
  const sb = [...b].sort();
  for (let i = 0; i < sa.length; i++) {
    if (sa[i] !== sb[i]) return false;
  }
  return true;
}
const ALL_KEYS = [
  "C",
  "C#",
  "D",
  "D#",
  "E",
  "F",
  "F#",
  "G",
  "G#",
  "A",
  "A#",
  "B"
];
const ALL_MODES = [
  {
    name: "PassThrough",
    label: "Pass Through",
    shortLabel: "Pass"
  },
  {
    name: "DiatonicThirds",
    label: "Diatonic Thirds",
    shortLabel: "3rds"
  },
  {
    name: "DiatonicFourths",
    label: "Diatonic Fourths",
    shortLabel: "4ths"
  },
  {
    name: "RandomBelow",
    label: "Random Below",
    shortLabel: "Rand"
  },
  {
    name: "RandomBelowNoSeconds",
    label: "Random No 2nds",
    shortLabel: "Rand-"
  },
  {
    name: "ContraryMotion",
    label: "Contrary Motion",
    shortLabel: "Contra"
  },
  {
    name: "StrictCounterpoint",
    label: "Strict Counterpoint",
    shortLabel: "Strict"
  },
  {
    name: "BarryHarris",
    label: "Barry Harris",
    shortLabel: "Barry"
  }
];
const SCALE_FAMILIES = [
  {
    family: "Diatonic",
    label: "Diatonic Modes",
    modes: [
      { name: "Ionian", label: "Ionian (Major)" },
      { name: "Dorian", label: "Dorian" },
      { name: "Phrygian", label: "Phrygian" },
      { name: "Lydian", label: "Lydian" },
      { name: "Mixolydian", label: "Mixolydian" },
      { name: "Aeolian", label: "Aeolian (Minor)" },
      { name: "Locrian", label: "Locrian" }
    ]
  },
  {
    family: "HarmonicMinor",
    label: "Harmonic Minor",
    modes: [
      { name: "HarmonicMinor", label: "Harmonic Minor" },
      { name: "LocrianNat6", label: "Locrian Nat 6" },
      { name: "IonianAug", label: "Ionian Aug" },
      { name: "DorianSharp4", label: "Dorian #4" },
      { name: "PhrygianDominant", label: "Phrygian Dom" },
      { name: "LydianSharp2", label: "Lydian #2" },
      { name: "SuperLocrianDim", label: "Super Locrian Dim" }
    ]
  },
  {
    family: "MelodicMinor",
    label: "Melodic Minor",
    modes: [
      { name: "MelodicMinor", label: "Melodic Minor" },
      { name: "DorianFlat2", label: "Dorian b2" },
      { name: "LydianAug", label: "Lydian Aug" },
      { name: "LydianDominant", label: "Lydian Dom" },
      { name: "MixolydianFlat6", label: "Mixolydian b6" },
      { name: "LocrianNat2", label: "Locrian Nat 2" },
      { name: "SuperLocrian", label: "Super Locrian" }
    ]
  },
  {
    family: "Exotic",
    label: "Exotic",
    modes: [
      { name: "DoubleHarmonic", label: "Double Harmonic" },
      { name: "HungarianMinor", label: "Hungarian Minor" },
      { name: "Enigmatic", label: "Enigmatic" },
      { name: "NeapolitanMinor", label: "Neapolitan Minor" },
      { name: "NeapolitanMajor", label: "Neapolitan Major" }
    ]
  },
  {
    family: "BarryHarris",
    label: "Barry Harris",
    modes: [
      { name: "BHMajor6thDim", label: "Major 6th Dim" },
      { name: "BHMinor6thDim", label: "Minor 6th Dim" }
    ]
  }
];
const OCTAVE_MODES = [
  { name: "None", label: "None" },
  { name: "Spread", label: "Spread" },
  { name: "BassTrebleSplit", label: "Split" },
  { name: "Mirror", label: "Mirror" }
];
const VOICE_LEADING_STYLES = [
  { name: "Free", label: "Free" },
  { name: "Palestrina", label: "Palestrina" },
  { name: "BachChorale", label: "Bach" },
  { name: "Jazz", label: "Jazz" }
];
const SCALE_INTERVALS = {
  // Church modes
  Ionian: [0, 2, 4, 5, 7, 9, 11],
  Dorian: [0, 2, 3, 5, 7, 9, 10],
  Phrygian: [0, 1, 3, 5, 7, 8, 10],
  Lydian: [0, 2, 4, 6, 7, 9, 11],
  Mixolydian: [0, 2, 4, 5, 7, 9, 10],
  Aeolian: [0, 2, 3, 5, 7, 8, 10],
  Locrian: [0, 1, 3, 5, 6, 8, 10],
  // Harmonic minor modes
  HarmonicMinor: [0, 2, 3, 5, 7, 8, 11],
  LocrianNat6: [0, 1, 3, 5, 6, 9, 10],
  IonianAug: [0, 2, 4, 5, 8, 9, 11],
  DorianSharp4: [0, 2, 3, 6, 7, 9, 10],
  PhrygianDominant: [0, 1, 4, 5, 7, 8, 10],
  LydianSharp2: [0, 3, 4, 6, 7, 9, 11],
  SuperLocrianDim: [0, 1, 3, 4, 6, 8, 9],
  // Melodic minor modes
  MelodicMinor: [0, 2, 3, 5, 7, 9, 11],
  DorianFlat2: [0, 1, 3, 5, 7, 9, 10],
  LydianAug: [0, 2, 4, 6, 8, 9, 11],
  LydianDominant: [0, 2, 4, 6, 7, 9, 10],
  MixolydianFlat6: [0, 2, 4, 5, 7, 8, 10],
  LocrianNat2: [0, 2, 3, 5, 6, 8, 10],
  SuperLocrian: [0, 1, 3, 4, 6, 8, 10],
  // Exotic
  DoubleHarmonic: [0, 1, 4, 5, 7, 8, 11],
  HungarianMinor: [0, 2, 3, 6, 7, 8, 11],
  Enigmatic: [0, 1, 4, 6, 8, 10, 11],
  NeapolitanMinor: [0, 1, 3, 5, 7, 8, 11],
  NeapolitanMajor: [0, 1, 3, 5, 7, 9, 11],
  // Barry Harris 6th Diminished (8-note)
  BHMajor6thDim: [0, 2, 4, 5, 7, 8, 9, 11],
  BHMinor6thDim: [0, 2, 3, 5, 7, 8, 9, 11]
};
const KEY_TO_PITCH_CLASS = {
  C: 0,
  "C#": 1,
  D: 2,
  "D#": 3,
  E: 4,
  F: 5,
  "F#": 6,
  G: 7,
  "G#": 8,
  A: 9,
  "A#": 10,
  B: 11
};
function computeScaleNotes(key, scaleMode) {
  const tonic = KEY_TO_PITCH_CLASS[key];
  const intervals = SCALE_INTERVALS[scaleMode];
  const pitchClasses = new Set(intervals.map((i) => (tonic + i) % 12));
  const notes = [];
  for (let midi2 = 0; midi2 <= 127; midi2++) {
    if (pitchClasses.has(midi2 % 12)) notes.push(midi2);
  }
  return notes;
}
const SETTINGS_KEY = "contrapunk-settings";
const SETTINGS_VERSION = 1;
const SETTINGS_DEFAULTS = {
  version: SETTINGS_VERSION,
  key: "C",
  mode: "PassThrough",
  scaleMode: "Ionian",
  octaveMode: "None",
  voiceLeadingEnabled: false,
  voiceLeadingStyle: "Free",
  interchangeEnabled: false,
  interchangeRange: 3,
  voicePosition: 0,
  voiceCount: 2,
  detuneCents: 0
};
const VALID_KEYS = new Set(ALL_KEYS);
const VALID_MODES = new Set(ALL_MODES.map((m) => m.name));
const VALID_SCALE_MODES = new Set(SCALE_FAMILIES.flatMap((f) => f.modes.map((m) => m.name)));
const VALID_OCTAVE_MODES = new Set(OCTAVE_MODES.map((m) => m.name));
const VALID_VL_STYLES = new Set(VOICE_LEADING_STYLES.map((s) => s.name));
function loadSettings() {
  try {
    const raw = localStorage.getItem(SETTINGS_KEY);
    if (!raw) return null;
    const parsed = JSON.parse(raw);
    if (!parsed || parsed.version !== SETTINGS_VERSION) {
      localStorage.removeItem(SETTINGS_KEY);
      return null;
    }
    return {
      version: SETTINGS_VERSION,
      key: VALID_KEYS.has(parsed.key) ? parsed.key : SETTINGS_DEFAULTS.key,
      mode: VALID_MODES.has(parsed.mode) ? parsed.mode : SETTINGS_DEFAULTS.mode,
      scaleMode: VALID_SCALE_MODES.has(parsed.scaleMode) ? parsed.scaleMode : SETTINGS_DEFAULTS.scaleMode,
      octaveMode: VALID_OCTAVE_MODES.has(parsed.octaveMode) ? parsed.octaveMode : SETTINGS_DEFAULTS.octaveMode,
      voiceLeadingEnabled: typeof parsed.voiceLeadingEnabled === "boolean" ? parsed.voiceLeadingEnabled : SETTINGS_DEFAULTS.voiceLeadingEnabled,
      voiceLeadingStyle: VALID_VL_STYLES.has(parsed.voiceLeadingStyle) ? parsed.voiceLeadingStyle : SETTINGS_DEFAULTS.voiceLeadingStyle,
      interchangeEnabled: typeof parsed.interchangeEnabled === "boolean" ? parsed.interchangeEnabled : SETTINGS_DEFAULTS.interchangeEnabled,
      interchangeRange: typeof parsed.interchangeRange === "number" && parsed.interchangeRange >= 1 && parsed.interchangeRange <= 5 ? parsed.interchangeRange : SETTINGS_DEFAULTS.interchangeRange,
      voicePosition: typeof parsed.voicePosition === "number" && parsed.voicePosition >= 0 ? parsed.voicePosition : SETTINGS_DEFAULTS.voicePosition,
      voiceCount: typeof parsed.voiceCount === "number" && parsed.voiceCount >= 1 && parsed.voiceCount <= 8 ? parsed.voiceCount : SETTINGS_DEFAULTS.voiceCount,
      detuneCents: typeof parsed.detuneCents === "number" && parsed.detuneCents >= -100 && parsed.detuneCents <= 100 ? parsed.detuneCents : SETTINGS_DEFAULTS.detuneCents
    };
  } catch {
    localStorage.removeItem(SETTINGS_KEY);
    return null;
  }
}
function saveSettings(s) {
  try {
    localStorage.setItem(SETTINGS_KEY, JSON.stringify({ ...s, version: SETTINGS_VERSION }));
  } catch {
  }
}
class EngineStore {
  // -- Harmony configuration --
  key = "C";
  mode = "PassThrough";
  modeNumber = 1;
  scaleMode = "Ionian";
  octaveMode = "None";
  // -- Voice leading --
  voiceLeadingEnabled = false;
  voiceLeadingStyle = "Free";
  // -- Modal interchange --
  interchangeEnabled = false;
  interchangeRange = 3;
  // -- Voice position --
  voicePosition = 0;
  voiceCount = 2;
  // -- Detune --
  detuneCents = 0;
  // -- Transport --
  isRunning = false;
  // -- Real-time note state (updated by adapter events) --
  inputNotes = [];
  harmonyNotes = [];
  borrowedNotes = [];
  generatorNotes = [];
  #inScaleNotes = derived(() => computeScaleNotes(this.key, this.scaleMode));
  get inScaleNotes() {
    return this.#inScaleNotes();
  }
  set inScaleNotes($$value) {
    return this.#inScaleNotes($$value);
  }
  chordName = "";
  lastBorrowedFrom = "";
  unsubNotes = null;
  persist() {
    saveSettings({
      key: this.key,
      mode: this.mode,
      scaleMode: this.scaleMode,
      octaveMode: this.octaveMode,
      voiceLeadingEnabled: this.voiceLeadingEnabled,
      voiceLeadingStyle: this.voiceLeadingStyle,
      interchangeEnabled: this.interchangeEnabled,
      interchangeRange: this.interchangeRange,
      voicePosition: this.voicePosition,
      voiceCount: this.voiceCount,
      detuneCents: this.detuneCents
    });
  }
  /**
   * Restore saved settings from localStorage and apply to backend.
   * Call after adapter.init() and syncFromBackend().
   */
  async restoreSettings() {
    const saved = loadSettings();
    if (!saved) return;
    const ops = [
      ["key", () => adapter.setKey(saved.key)],
      ["mode", () => adapter.setMode(saved.mode)],
      ["scaleMode", () => adapter.setScaleMode(saved.scaleMode)],
      ["octaveMode", () => adapter.setOctaveMode(saved.octaveMode)],
      [
        "voiceLeading",
        () => adapter.setVoiceLeading(saved.voiceLeadingEnabled, saved.voiceLeadingStyle)
      ],
      [
        "interchange",
        () => adapter.setInterchange(saved.interchangeEnabled, saved.interchangeRange)
      ],
      [
        "voicePosition",
        () => adapter.setVoicePosition(saved.voicePosition)
      ],
      ["voiceCount", () => adapter.setVoiceCount(saved.voiceCount)],
      [
        "detune",
        () => {
          adapter.setDetune(saved.detuneCents);
          return Promise.resolve();
        }
      ]
    ];
    for (const [name, op] of ops) {
      try {
        await op();
      } catch (e) {
        console.warn(`[contrapunk] Failed to restore ${name}:`, e);
      }
    }
    try {
      await this.syncFromBackend();
    } catch (e) {
      console.warn("[contrapunk] Failed to sync after restore:", e);
    }
  }
  // === Adapter-wired actions (optimistic update with rollback) ===
  async setKey(newKey) {
    const prev = this.key;
    this.key = newKey;
    try {
      await adapter.setKey(newKey);
      this.persist();
    } catch (e) {
      this.key = prev;
      throw e;
    }
  }
  async setMode(newMode) {
    const prev = this.mode;
    this.mode = newMode;
    try {
      await adapter.setMode(newMode);
      this.persist();
    } catch (e) {
      this.mode = prev;
      throw e;
    }
  }
  async setScaleMode(newMode) {
    const prev = this.scaleMode;
    this.scaleMode = newMode;
    try {
      await adapter.setScaleMode(newMode);
      this.persist();
    } catch (e) {
      this.scaleMode = prev;
      throw e;
    }
  }
  async setOctaveMode(newMode) {
    const prev = this.octaveMode;
    this.octaveMode = newMode;
    try {
      await adapter.setOctaveMode(newMode);
      this.persist();
    } catch (e) {
      this.octaveMode = prev;
      throw e;
    }
  }
  async setVoiceLeading(enabled, style) {
    const prevEnabled = this.voiceLeadingEnabled;
    const prevStyle = this.voiceLeadingStyle;
    this.voiceLeadingEnabled = enabled;
    if (style) this.voiceLeadingStyle = style;
    try {
      await adapter.setVoiceLeading(enabled, style ?? this.voiceLeadingStyle);
      this.persist();
    } catch (e) {
      this.voiceLeadingEnabled = prevEnabled;
      this.voiceLeadingStyle = prevStyle;
      throw e;
    }
  }
  async setInterchange(enabled, range) {
    const prevEnabled = this.interchangeEnabled;
    const prevRange = this.interchangeRange;
    this.interchangeEnabled = enabled;
    if (range !== void 0) this.interchangeRange = range;
    try {
      await adapter.setInterchange(enabled, range ?? this.interchangeRange);
      this.persist();
    } catch (e) {
      this.interchangeEnabled = prevEnabled;
      this.interchangeRange = prevRange;
      throw e;
    }
  }
  async setVoicePosition(position) {
    const prev = this.voicePosition;
    this.voicePosition = position;
    try {
      await adapter.setVoicePosition(position);
      this.persist();
    } catch (e) {
      this.voicePosition = prev;
      throw e;
    }
  }
  async setVoiceCount(count) {
    const prev = this.voiceCount;
    this.voiceCount = count;
    try {
      await adapter.setVoiceCount(count);
      this.persist();
    } catch (e) {
      this.voiceCount = prev;
      throw e;
    }
  }
  setDetune(cents) {
    this.detuneCents = cents;
    adapter.setDetune(cents);
    this.persist();
  }
  /**
   * Start MIDI routing from the given input to the given outputs.
   */
  async start(inputIdx, outputIndices) {
    await adapter.startRouting(inputIdx, outputIndices);
    this.isRunning = true;
    this.startNoteUpdates();
  }
  /**
   * Stop MIDI routing and clear note state.
   */
  async stop() {
    await adapter.stopRouting();
    this.isRunning = false;
    this.stopNoteUpdates();
    this.inputNotes = [];
    this.harmonyNotes = [];
    this.borrowedNotes = [];
    this.chordName = "";
    this.lastBorrowedFrom = "";
  }
  /** Toggle start/stop (requires MIDI store state for device indices). */
  toggle() {
    if (this.isRunning) {
      this.stop();
    }
  }
  /**
   * Pull full engine state from the backend and update local reactive state.
   * Call after init or preset load.
   */
  async syncFromBackend() {
    const state = await adapter.getEngineState();
    this.key = state.key;
    this.mode = state.mode;
    this.modeNumber = state.modeNumber;
    this.scaleMode = state.scaleMode;
    this.octaveMode = state.octaveMode;
    this.voiceLeadingEnabled = state.voiceLeadingEnabled;
    this.voiceLeadingStyle = state.voiceLeadingStyle;
    this.interchangeEnabled = state.interchangeEnabled;
    this.interchangeRange = state.interchangeRange;
    this.voicePosition = state.voicePosition;
    this.voiceCount = state.voiceCount;
    this.isRunning = state.isRunning;
  }
  /**
   * Subscribe to real-time note update events from the adapter.
   * Only assigns when values actually change to avoid Svelte re-renders.
   */
  startNoteUpdates() {
    if (this.unsubNotes) return;
    this.unsubNotes = adapter.onNoteUpdate((state) => {
      if (!sameNotes(this.inputNotes, state.inputNotes)) this.inputNotes = state.inputNotes;
      if (!sameNotes(this.harmonyNotes, state.harmonyNotes)) this.harmonyNotes = state.harmonyNotes;
      if (!sameNotes(this.borrowedNotes, state.borrowedNotes)) this.borrowedNotes = state.borrowedNotes;
      if (this.chordName !== state.chordName) this.chordName = state.chordName;
      if (this.lastBorrowedFrom !== state.lastBorrowedFrom) this.lastBorrowedFrom = state.lastBorrowedFrom;
    });
  }
  /**
   * Unsubscribe from note update events.
   */
  stopNoteUpdates() {
    this.unsubNotes?.();
    this.unsubNotes = null;
  }
}
const engine = new EngineStore();
const MIDI_SETTINGS_KEY = "contrapunk-midi";
function loadMidiSettings() {
  try {
    const raw = localStorage.getItem(MIDI_SETTINGS_KEY);
    if (!raw) return null;
    return JSON.parse(raw);
  } catch {
    return null;
  }
}
function saveMidiSettings(settings) {
  try {
    localStorage.setItem(MIDI_SETTINGS_KEY, JSON.stringify(settings));
  } catch {
  }
}
class MidiStore {
  // -- Available devices --
  inputs = [];
  outputs = [];
  // -- Selection state --
  selectedInput = null;
  selectedOutputs = [];
  // -- Loading / error state --
  isLoading = false;
  error = null;
  /**
   * Refresh the list of available MIDI devices from the backend.
   * After refreshing, restores previously selected devices by name.
   */
  async refresh() {
    this.isLoading = true;
    this.error = null;
    try {
      await adapter.refreshMidiDevices();
      const [newInputs, newOutputs] = await Promise.all([adapter.listMidiInputs(), adapter.listMidiOutputs()]);
      this.inputs = newInputs;
      this.outputs = newOutputs;
      const saved = loadMidiSettings();
      if (saved) {
        if (saved.inputName) {
          const virtualId = MidiStore.VIRTUAL_IDS[saved.inputName];
          if (virtualId !== void 0) {
            this.selectedInput = virtualId;
          } else {
            const match = newInputs.find((d) => d.name === saved.inputName);
            if (match) {
              this.selectedInput = match.index;
            } else {
              this.selectedInput = null;
            }
          }
        }
        if (saved.outputNames.length > 0) {
          this.selectedOutputs = saved.outputNames.map((name) => newOutputs.find((d) => d.name === name)).filter((d) => d !== void 0).map((d) => d.index);
        }
      } else {
        if (this.selectedInput !== null && !newInputs.some((d) => d.index === this.selectedInput)) {
          this.selectedInput = null;
        }
        this.selectedOutputs = this.selectedOutputs.filter((idx) => newOutputs.some((d) => d.index === idx));
      }
    } catch (e) {
      this.error = `Failed to refresh MIDI devices: ${e}`;
    } finally {
      this.isLoading = false;
    }
  }
  /** Persist current selections by device name. */
  persist() {
    saveMidiSettings({
      inputName: this.selectedInputName,
      outputNames: this.selectedOutputNames
    });
  }
  /** Map virtual sentinel values to persistent names. */
  static VIRTUAL_NAMES = {
    999999: "__virtual_note_generator__",
    999998: "__virtual_computer_keyboard__",
    999997: "__virtual_guitar_audio__"
  };
  /** Reverse lookup: persistent name → sentinel value. */
  static VIRTUAL_IDS = {
    "__virtual_note_generator__": 999999,
    "__virtual_computer_keyboard__": 999998,
    "__virtual_guitar_audio__": 999997
  };
  /**
   * Select a MIDI input device by index.
   */
  selectInput(index) {
    if (this.inputs.some((d) => d.index === index)) {
      this.selectedInput = index;
      this.persist();
    }
  }
  /**
   * Select a virtual input (Guitar Audio, Computer Keyboard, Note Generator).
   */
  selectVirtualInput(index) {
    this.selectedInput = index;
    this.persist();
  }
  /**
   * Clear the input selection.
   */
  clearInput() {
    this.selectedInput = null;
    this.persist();
  }
  /**
   * Toggle a MIDI output device selection.
   * If already selected, deselects it. Otherwise, adds it.
   */
  toggleOutput(index) {
    if (!this.outputs.some((d) => d.index === index)) return;
    const idx = this.selectedOutputs.indexOf(index);
    if (idx >= 0) {
      this.selectedOutputs = this.selectedOutputs.filter((i) => i !== index);
    } else {
      this.selectedOutputs = [...this.selectedOutputs, index];
    }
    this.persist();
  }
  /**
   * Set outputs to a specific list of indices.
   */
  setOutputs(indices) {
    this.selectedOutputs = indices.filter((idx) => this.outputs.some((d) => d.index === idx));
    this.persist();
  }
  /**
   * Check whether a valid input and at least one output are selected.
   */
  get isReady() {
    return this.selectedInput !== null && this.selectedOutputs.length > 0;
  }
  /**
   * Get the name of the selected input device, or null.
   * Returns virtual names for sentinel values (e.g. "__virtual_guitar_audio__").
   */
  get selectedInputName() {
    if (this.selectedInput === null) return null;
    const virtualName = MidiStore.VIRTUAL_NAMES[this.selectedInput];
    if (virtualName) return virtualName;
    return this.inputs.find((d) => d.index === this.selectedInput)?.name ?? null;
  }
  /**
   * Get the names of the selected output devices.
   */
  get selectedOutputNames() {
    return this.selectedOutputs.map((idx) => this.outputs.find((d) => d.index === idx)?.name).filter((name) => name !== void 0);
  }
}
const midi = new MidiStore();
function BeatIndicator($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    let bpm = derived(() => 120);
    let beatDuration = derived(() => 60 / bpm());
    let beatCount = 0;
    $$renderer2.push(`<div class="beat-indicator-container svelte-iompg9"><div class="beat-bar-row svelte-iompg9"><!--[-->`);
    const each_array = ensure_array_like([0, 1, 2, 3]);
    for (let $$index = 0, $$length = each_array.length; $$index < $$length; $$index++) {
      let beat = each_array[$$index];
      $$renderer2.push(`<div${attr_class("beat-pip svelte-iompg9", void 0, {
        "active": engine.isRunning && beatCount === beat,
        "downbeat": beat === 0 && engine.isRunning && beatCount === 0,
        "animate": ui.animationsEnabled && engine.isRunning
      })}${attr_style("", { "animation-duration": `${stringify(beatDuration())}s` })}></div>`);
    }
    $$renderer2.push(`<!--]--></div></div>`);
  });
}
function StatusBar($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    $$renderer2.push(`<div class="status-bar svelte-1piydef"><button${attr_class("transport-btn font-pixel svelte-1piydef", void 0, {
      "running": (
        // Restore FX preference from localStorage on mount
        // localStorage unavailable
        engine.isRunning
      )
    })}${attr("disabled", !engine.isRunning && midi.selectedInput === null, true)}${attr("title", !engine.isRunning && midi.selectedInput === null ? "Select an input device first" : "")}>${escape_html(engine.isRunning ? "Stop" : "Start")}</button> <span${attr_class("status-indicator font-pixel svelte-1piydef", void 0, { "active": engine.isRunning })}>${escape_html(engine.isRunning ? "ACTIVE" : "STOPPED")}</span> `);
    BeatIndicator($$renderer2);
    $$renderer2.push(`<!----> <div class="chord-info svelte-1piydef">`);
    if (engine.chordName) {
      $$renderer2.push("<!--[-->");
      $$renderer2.push(`<span class="chord-name font-pixel svelte-1piydef">${escape_html(engine.chordName)}</span>`);
    } else {
      $$renderer2.push("<!--[!-->");
      $$renderer2.push(`<span class="chord-name font-pixel dim svelte-1piydef">---</span>`);
    }
    $$renderer2.push(`<!--]--> `);
    if (engine.interchangeEnabled && engine.lastBorrowedFrom) {
      $$renderer2.push("<!--[-->");
      $$renderer2.push(`<span class="borrowed-label font-pixel svelte-1piydef">from ${escape_html(engine.lastBorrowedFrom)}</span>`);
    } else {
      $$renderer2.push("<!--[!-->");
    }
    $$renderer2.push(`<!--]--></div> <div class="spacer svelte-1piydef"></div> <button${attr_class("fx-btn pixel-btn font-pixel svelte-1piydef", void 0, { "fx-off": !ui.animationsEnabled })}${attr("title", ui.animationsEnabled ? "Disable visual effects" : "Enable visual effects")}>FX</button> <img src="/logo.svg" alt="Contrapunk" class="brand-logo svelte-1piydef"/> <span class="brand font-pixel svelte-1piydef">Contrapunk</span></div>`);
  });
}
function _page($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    let hasActiveNotes = derived(() => engine.inputNotes.length > 0 || engine.harmonyNotes.length > 0);
    let manyNotesActive = derived(() => engine.inputNotes.length + engine.harmonyNotes.length >= 4);
    $$renderer2.push(`<div class="app-layout svelte-1uha8ag">`);
    if (ui.animationsEnabled && hasActiveNotes()) {
      $$renderer2.push("<!--[-->");
      $$renderer2.push(`<div${attr_class("vignette-overlay svelte-1uha8ag", void 0, { "intense": manyNotesActive() })} aria-hidden="true"></div>`);
    } else {
      $$renderer2.push("<!--[!-->");
    }
    $$renderer2.push(`<!--]--> `);
    {
      $$renderer2.push("<!--[!-->");
    }
    $$renderer2.push(`<!--]--> `);
    StatusBar($$renderer2);
    $$renderer2.push(`<!----> `);
    {
      $$renderer2.push("<!--[1-->");
      $$renderer2.push(`<div class="init-loading font-pixel svelte-1uha8ag">Initializing engine...</div>`);
    }
    $$renderer2.push(`<!--]--></div>`);
  });
}
export {
  _page as default
};
