import "clsx";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
let adapter, ui;
let __tla = (async () => {
  let WasmGuitarInputClass = null;
  async function getWasmGuitarInput() {
    if (!WasmGuitarInputClass) {
      const mod = await import("./contrapunk_wasm.js");
      WasmGuitarInputClass = mod.WasmGuitarInput;
    }
    return WasmGuitarInputClass;
  }
  const DEFAULT_BUFFER_SIZE = 1024;
  class GuitarAudioCapture {
    audioContext = null;
    mediaStream = null;
    sourceNode = null;
    processorNode = null;
    callbacks = null;
    dsp = null;
    _isRunning = false;
    _actualChannel = 0;
    _frameCount = 0;
    overlapBuffer = null;
    overlapWritePos = 0;
    hopSize = 0;
    windowSize = 0;
    get isRunning() {
      return this._isRunning;
    }
    get actualChannel() {
      return this._actualChannel;
    }
    noiseGateThreshold = 0.01;
    noiseGateEnabled = true;
    clarityGateEnabled = false;
    clarityThreshold = 0.7;
    async start(deviceId, channelIndex, callbacks, bufferSize = DEFAULT_BUFFER_SIZE) {
      if (this._isRunning) await this.stop();
      this.callbacks = callbacks;
      const WasmGuitarInput = await getWasmGuitarInput();
      const constraints = {
        audio: {
          deviceId: deviceId ? {
            exact: deviceId
          } : void 0,
          echoCancellation: false,
          noiseSuppression: false,
          autoGainControl: false,
          channelCount: {
            ideal: 32
          }
        }
      };
      this.mediaStream = await navigator.mediaDevices.getUserMedia(constraints);
      this.audioContext = new AudioContext();
      this.sourceNode = this.audioContext.createMediaStreamSource(this.mediaStream);
      const actualChannels = this.sourceNode.channelCount;
      const sampleRate = this.audioContext.sampleRate;
      console.log(`[guitar] Device: ${actualChannels}ch @ ${sampleRate}Hz, want ch${channelIndex}, buffer=${bufferSize}`);
      this.windowSize = bufferSize;
      this.hopSize = Math.floor(bufferSize / 4);
      this.overlapBuffer = new Float32Array(bufferSize);
      this.overlapWritePos = 0;
      this.dsp = new WasmGuitarInput(sampleRate, bufferSize);
      this.dsp.set_onset_threshold(0.015);
      this.dsp.set_string_confidence(0.4);
      console.log(`[guitar] Overlap: window=${this.windowSize} hop=${this.hopSize} (75% overlap)`);
      const inputChannels = Math.max(channelIndex + 1, actualChannels);
      this.processorNode = this.audioContext.createScriptProcessor(bufferSize, inputChannels, 1);
      this.processorNode.channelCountMode = "explicit";
      this.processorNode.channelInterpretation = "discrete";
      const self = this;
      let actualChannelLogged = false;
      this._actualChannel = channelIndex;
      this.processorNode.onaudioprocess = (event) => {
        if (!self._isRunning || !self.callbacks || !self.dsp) return;
        const inputBuffer = event.inputBuffer;
        const availableChannels = inputBuffer.numberOfChannels;
        const useChannel = channelIndex < availableChannels ? channelIndex : 0;
        if (!actualChannelLogged) {
          console.log(`[guitar] Processing: requested ch=${channelIndex}, available=${availableChannels}, using ch=${useChannel}`);
          self._actualChannel = useChannel;
          actualChannelLogged = true;
        }
        const samples = inputBuffer.getChannelData(useChannel);
        let rmsSum = 0;
        for (let i = 0; i < samples.length; i++) rmsSum += samples[i] * samples[i];
        const rms = Math.sqrt(rmsSum / samples.length);
        self._frameCount++;
        if (self._frameCount % 25 === 0) {
          console.log(`[guitar] frame=${self._frameCount} rms=${rms.toFixed(4)} ch=${useChannel}`);
        }
        const eventsJson = self.dsp.process_block(samples);
        let allEvents;
        try {
          allEvents = JSON.parse(eventsJson);
        } catch (err) {
          console.error("[guitar] Failed to parse WASM events:", err);
          return;
        }
        if (allEvents.length > 0) {
          console.log(`[guitar] WASM events (${allEvents.length}):`, JSON.stringify(allEvents));
        }
        if (self.callbacks.onDetection) {
          const lastNoteOn = allEvents.findLast?.((e) => e.type === "note_on");
          if (lastNoteOn) {
            self.callbacks.onDetection({
              frequency: null,
              clarity: 1,
              noteName: midiToNoteName$1(lastNoteOn.note),
              midi: lastNoteOn.note,
              cents: 0,
              rms
            });
          } else {
            self.callbacks.onDetection({
              frequency: null,
              clarity: 0,
              noteName: "-",
              midi: 0,
              cents: 0,
              rms
            });
          }
        }
        for (const e of allEvents) {
          switch (e.type) {
            case "note_on":
              console.log(`[midi] NOTE ON: ${midiToNoteName$1(e.note)} (${e.note}) vel=${e.velocity} ch=${e.channel}`);
              self.callbacks.onNoteOn(e.note, e.velocity);
              break;
            case "note_off":
              console.log(`[midi] NOTE OFF: ${midiToNoteName$1(e.note)} (${e.note}) ch=${e.channel}`);
              self.callbacks.onNoteOff(e.note);
              break;
            case "pitch_bend":
              self.callbacks.onPitchBend?.(e.channel, e.cents);
              break;
            case "midi_pitch_bend":
              self.callbacks.onMidiPitchBend?.(e.channel, e.value);
              break;
            case "cc":
              self.callbacks.onCC?.(e.channel, e.controller, e.value);
              break;
            case "channel_pressure":
              self.callbacks.onChannelPressure?.(e.channel, e.pressure);
              break;
            case "vibrato":
              self.callbacks.onVibratoStatus?.(e.active, e.rate_hz, e.depth_cents);
              break;
          }
        }
      };
      this.sourceNode.connect(this.processorNode);
      this.processorNode.connect(this.audioContext.destination);
      this._isRunning = true;
    }
    async stop() {
      this._isRunning = false;
      if (this.processorNode) {
        this.processorNode.onaudioprocess = null;
        this.processorNode.disconnect();
        this.processorNode = null;
      }
      if (this.sourceNode) {
        this.sourceNode.disconnect();
        this.sourceNode = null;
      }
      if (this.audioContext) {
        try {
          await this.audioContext.close();
        } catch {
        }
        this.audioContext = null;
      }
      if (this.mediaStream) {
        this.mediaStream.getTracks().forEach((t) => t.stop());
        this.mediaStream = null;
      }
      if (this.dsp) {
        try {
          this.dsp.free();
        } catch {
        }
        this.dsp = null;
      }
      this.callbacks = null;
    }
    setConfig(opts) {
      if (!this.dsp) return;
      if (opts.bends !== void 0) this.dsp.set_bends_enabled(opts.bends);
      if (opts.legato !== void 0) this.dsp.set_legato_enabled(opts.legato);
      if (opts.slides !== void 0) this.dsp.set_slides_enabled(opts.slides);
      if (opts.vibrato !== void 0) this.dsp.set_vibrato_enabled(opts.vibrato);
      if (opts.gain !== void 0) this.dsp.set_input_gain(opts.gain);
      if (opts.onsetThreshold !== void 0) this.dsp.set_onset_threshold(opts.onsetThreshold);
      if (opts.stringConfidence !== void 0) this.dsp.set_string_confidence(opts.stringConfidence);
    }
    async measureNoiseFloor(deviceId, durationMs = 3e3, channelIndex = 0) {
      const stream = await navigator.mediaDevices.getUserMedia({
        audio: {
          deviceId: deviceId ? {
            exact: deviceId
          } : void 0,
          echoCancellation: false,
          noiseSuppression: false,
          autoGainControl: false,
          channelCount: {
            ideal: 32
          }
        }
      });
      const ctx = new AudioContext();
      const source = ctx.createMediaStreamSource(stream);
      const inputChannels = Math.max(channelIndex + 1, source.channelCount);
      const processor = ctx.createScriptProcessor(DEFAULT_BUFFER_SIZE, inputChannels, 1);
      processor.channelCountMode = "explicit";
      processor.channelInterpretation = "discrete";
      let totalRms = 0;
      let frameCount = 0;
      return new Promise((resolve) => {
        processor.onaudioprocess = (event) => {
          const ch = Math.min(channelIndex, event.inputBuffer.numberOfChannels - 1);
          const data = event.inputBuffer.getChannelData(ch);
          let sum = 0;
          for (let i = 0; i < data.length; i++) sum += data[i] * data[i];
          totalRms += Math.sqrt(sum / data.length);
          frameCount++;
        };
        source.connect(processor);
        processor.connect(ctx.destination);
        setTimeout(() => {
          processor.onaudioprocess = null;
          processor.disconnect();
          source.disconnect();
          ctx.close().catch(() => {
          });
          stream.getTracks().forEach((t) => t.stop());
          resolve(frameCount > 0 ? totalRms / frameCount : 0);
        }, durationMs);
      });
    }
  }
  function midiToNoteName$1(midi) {
    const names = [
      "C",
      "C#",
      "D",
      "Eb",
      "E",
      "F",
      "F#",
      "G",
      "Ab",
      "A",
      "Bb",
      "B"
    ];
    const noteIndex = (midi % 12 + 12) % 12;
    const octave = Math.floor(midi / 12) - 1;
    return `${names[noteIndex]}${octave}`;
  }
  const NOTE_NAMES = [
    "C",
    "C#",
    "D",
    "Eb",
    "E",
    "F",
    "F#",
    "G",
    "Ab",
    "A",
    "Bb",
    "B"
  ];
  function detectPitch(buffer, sampleRate, clarityThreshold = 0.7) {
    const size = buffer.length;
    let rmsSum = 0;
    for (let i = 0; i < size; i++) {
      rmsSum += buffer[i] * buffer[i];
    }
    const rms = Math.sqrt(rmsSum / size);
    if (rms < 0.01) return null;
    const minPeriod = Math.floor(sampleRate / 1400);
    const maxPeriod = Math.ceil(sampleRate / 60);
    const searchMax = Math.min(maxPeriod, size - 1);
    const nsdf = new Float32Array(searchMax + 1);
    for (let tau = minPeriod; tau <= searchMax; tau++) {
      let acf = 0;
      let m = 0;
      const windowSize = size - tau;
      for (let i = 0; i < windowSize; i++) {
        acf += buffer[i] * buffer[i + tau];
        m += buffer[i] * buffer[i] + buffer[i + tau] * buffer[i + tau];
      }
      nsdf[tau] = m > 0 ? 2 * acf / m : 0;
    }
    let bestPeriod = -1;
    let bestClarity = -1;
    let inPositiveLobe = false;
    let lobePeakVal = -1;
    let lobePeakIdx = -1;
    for (let tau = minPeriod; tau <= searchMax; tau++) {
      if (nsdf[tau] > 0) {
        if (!inPositiveLobe) {
          inPositiveLobe = true;
          lobePeakVal = nsdf[tau];
          lobePeakIdx = tau;
        } else if (nsdf[tau] > lobePeakVal) {
          lobePeakVal = nsdf[tau];
          lobePeakIdx = tau;
        }
      } else if (inPositiveLobe) {
        if (lobePeakVal >= clarityThreshold) {
          bestPeriod = lobePeakIdx;
          bestClarity = lobePeakVal;
          break;
        }
        inPositiveLobe = false;
      }
    }
    if (inPositiveLobe && bestPeriod < 0 && lobePeakVal >= clarityThreshold) {
      bestPeriod = lobePeakIdx;
      bestClarity = lobePeakVal;
    }
    if (bestPeriod < 0) return null;
    let refinedPeriod = bestPeriod;
    if (bestPeriod > minPeriod && bestPeriod < searchMax) {
      const prev = nsdf[bestPeriod - 1];
      const curr = nsdf[bestPeriod];
      const next = nsdf[bestPeriod + 1];
      const denom = 2 * (2 * curr - prev - next);
      if (Math.abs(denom) > 1e-10) {
        refinedPeriod = bestPeriod + (prev - next) / denom;
      }
    }
    return {
      frequency: sampleRate / refinedPeriod,
      clarity: bestClarity
    };
  }
  function frequencyToMidi(freq) {
    const midiFloat = 69 + 12 * Math.log2(freq / 440);
    const note = Math.round(midiFloat);
    const cents = Math.round((midiFloat - note) * 100);
    return {
      note: Math.max(0, Math.min(127, note)),
      cents
    };
  }
  function midiToNoteName(midi) {
    const noteIndex = (midi % 12 + 12) % 12;
    const octave = Math.floor(midi / 12) - 1;
    return `${NOTE_NAMES[noteIndex]}${octave}`;
  }
  const STORAGE_KEY = "contrapunk-guitar";
  function loadSaved() {
    try {
      const raw = localStorage.getItem(STORAGE_KEY);
      return raw ? JSON.parse(raw) : {};
    } catch {
      return {};
    }
  }
  class GuitarInputStore {
    latencyMs = 21;
    gain = 1;
    stringConfidence = 0.4;
    bendsEnabled = true;
    legatoEnabled = true;
    slidesEnabled = true;
    vibratoEnabled = false;
    audioDevices = [];
    selectedDeviceId = "";
    selectedChannel = 1;
    maxChannels = 2;
    constructor() {
      const saved = loadSaved();
      if (saved.latencyMs !== void 0) this.latencyMs = saved.latencyMs;
      if (saved.gain !== void 0) this.gain = saved.gain;
      if (saved.stringConfidence !== void 0) this.stringConfidence = saved.stringConfidence;
      if (saved.bendsEnabled !== void 0) this.bendsEnabled = saved.bendsEnabled;
      if (saved.legatoEnabled !== void 0) this.legatoEnabled = saved.legatoEnabled;
      if (saved.slidesEnabled !== void 0) this.slidesEnabled = saved.slidesEnabled;
      if (saved.vibratoEnabled !== void 0) this.vibratoEnabled = saved.vibratoEnabled;
      if (saved.selectedDeviceId) this.selectedDeviceId = saved.selectedDeviceId;
      if (saved.selectedChannel) this.selectedChannel = saved.selectedChannel;
      if (saved.calibrated) this.calibrated = saved.calibrated;
      if (saved.noiseGateEnabled !== void 0) this.noiseGateEnabled = saved.noiseGateEnabled;
      if (saved.noiseGateThreshold !== void 0) this.noiseGateThreshold = saved.noiseGateThreshold;
      if (saved.freqGateEnabled !== void 0) this.freqGateEnabled = saved.freqGateEnabled;
      if (saved.freqGateRange !== void 0) this.freqGateRange = saved.freqGateRange;
    }
    persist() {
      try {
        localStorage.setItem(STORAGE_KEY, JSON.stringify({
          latencyMs: this.latencyMs,
          gain: this.gain,
          stringConfidence: this.stringConfidence,
          bendsEnabled: this.bendsEnabled,
          legatoEnabled: this.legatoEnabled,
          slidesEnabled: this.slidesEnabled,
          vibratoEnabled: this.vibratoEnabled,
          selectedDeviceId: this.selectedDeviceId,
          selectedChannel: this.selectedChannel,
          calibrated: this.calibrated,
          noiseGateEnabled: this.noiseGateEnabled,
          noiseGateThreshold: this.noiseGateThreshold,
          freqGateEnabled: this.freqGateEnabled,
          freqGateRange: this.freqGateRange
        }));
      } catch {
      }
    }
    manualMaxChannels = null;
    audioDeviceError = "";
    detecting = false;
    activeChannel = 0;
    currentNote = "";
    currentString = "";
    currentFret = 0;
    confidence = 0;
    velocity = 0;
    signalLevel = 0;
    signalClarity = 0;
    noiseGateEnabled = true;
    noiseGateThreshold = 0.01;
    freqGateEnabled = true;
    freqGateRange = 1200;
    amplitudeHistory = [];
    clarityHistory = [];
    static HISTORY_SIZE = 128;
    pushSignalFrame(rms, clarity) {
      this.signalLevel = Math.min(1, rms * 5);
      this.signalClarity = clarity;
      this.amplitudeHistory.push(Math.min(1, rms * 5));
      this.clarityHistory.push(clarity);
      if (this.amplitudeHistory.length > GuitarInputStore.HISTORY_SIZE) {
        this.amplitudeHistory.shift();
      }
      if (this.clarityHistory.length > GuitarInputStore.HISTORY_SIZE) {
        this.clarityHistory.shift();
      }
    }
    calibrated = false;
    calibrating = false;
    tunerActive = false;
    tunerStringIndex = 0;
    tunerDetectedNote = "";
    tunerDetectedFreq = 0;
    tunerCents = 0;
    tunerClarity = 0;
    tunerStatus = "waiting";
    tunerHoldProgress = 0;
    tunerPhase = "noise-floor";
    tunerNoiseProgress = 0;
    toggleTechnique(technique) {
      switch (technique) {
        case "bends":
          this.bendsEnabled = !this.bendsEnabled;
          break;
        case "legato":
          this.legatoEnabled = !this.legatoEnabled;
          break;
        case "slides":
          this.slidesEnabled = !this.slidesEnabled;
          break;
        case "vibrato":
          this.vibratoEnabled = !this.vibratoEnabled;
          break;
      }
      this.persist();
    }
    noiseFloorRms = 0;
    calibrationStatus = "";
    static OPEN_STRINGS = [
      {
        name: "Low E",
        note: "E2",
        midi: 40,
        freq: 82.41
      },
      {
        name: "A",
        note: "A2",
        midi: 45,
        freq: 110
      },
      {
        name: "D",
        note: "D3",
        midi: 50,
        freq: 146.83
      },
      {
        name: "G",
        note: "G3",
        midi: 55,
        freq: 196
      },
      {
        name: "B",
        note: "B3",
        midi: 59,
        freq: 246.94
      },
      {
        name: "High E",
        note: "E4",
        midi: 64,
        freq: 329.63
      }
    ];
    tunerCapture = null;
    tunerAnimFrame = null;
    inTuneSince = null;
    static IN_TUNE_CENTS = 8;
    static IN_TUNE_HOLD_MS = 1500;
    async startCalibration() {
      if (this.calibrating) return;
      this.calibrating = true;
      this.calibrated = false;
      this.tunerActive = true;
      this.tunerPhase = "noise-floor";
      this.tunerStringIndex = 0;
      this.tunerNoiseProgress = 0;
      this.tunerStatus = "waiting";
      this.tunerHoldProgress = 0;
      this.calibrationStatus = "Measuring noise floor...";
      try {
        const capture = new GuitarAudioCapture();
        const noisePromise = capture.measureNoiseFloor(this.selectedDeviceId, 3e3, this.selectedChannel - 1);
        const noiseStart = Date.now();
        const noiseTimer = setInterval(() => {
          const elapsed = Date.now() - noiseStart;
          this.tunerNoiseProgress = Math.min(1, elapsed / 3e3);
        }, 50);
        const avgRms = await noisePromise;
        clearInterval(noiseTimer);
        this.tunerNoiseProgress = 1;
        this.noiseFloorRms = avgRms;
        this.calibrationStatus = `Noise floor: ${(avgRms * 1e3).toFixed(1)} mRMS`;
        this.tunerPhase = "tuning";
        this.tunerStringIndex = 0;
        await this.startTunerCapture();
      } catch (err) {
        this.calibrationStatus = err instanceof Error ? `Calibration failed: ${err.message}` : "Calibration failed";
        this.calibrated = false;
        this.tunerActive = false;
        this.calibrating = false;
      }
    }
    async startTunerCapture() {
      this.inTuneSince = null;
      const channelIndex = this.selectedChannel - 1;
      const constraints = {
        audio: {
          ...this.selectedDeviceId ? {
            deviceId: {
              exact: this.selectedDeviceId
            }
          } : {},
          echoCancellation: false,
          noiseSuppression: false,
          autoGainControl: false,
          channelCount: {
            ideal: 32
          }
        }
      };
      const stream = await navigator.mediaDevices.getUserMedia(constraints);
      this.tunerStream = stream;
      const ctx = new AudioContext({
        sampleRate: 48e3
      });
      this.tunerContext = ctx;
      const source = ctx.createMediaStreamSource(stream);
      const inputChannels = Math.max(channelIndex + 1, source.channelCount);
      const proc = ctx.createScriptProcessor(2048, inputChannels, 1);
      proc.channelCountMode = "explicit";
      proc.channelInterpretation = "discrete";
      this.tunerProcessor = proc;
      proc.onaudioprocess = (event) => {
        if (!this.tunerActive || this.tunerPhase !== "tuning") return;
        const input = event.inputBuffer;
        const ch = Math.min(channelIndex, input.numberOfChannels - 1);
        const samples = input.getChannelData(ch);
        let sum = 0;
        for (let i = 0; i < samples.length; i++) sum += samples[i] * samples[i];
        const rms = Math.sqrt(sum / samples.length);
        const result = detectPitch(samples, ctx.sampleRate);
        const target = GuitarInputStore.OPEN_STRINGS[this.tunerStringIndex];
        if (!target) return;
        if (!result || rms < this.noiseFloorRms * 2) {
          this.tunerDetectedNote = "";
          this.tunerDetectedFreq = 0;
          this.tunerCents = 0;
          this.tunerClarity = 0;
          this.tunerStatus = "waiting";
          this.tunerHoldProgress = 0;
          this.inTuneSince = null;
          return;
        }
        const midiInfo = frequencyToMidi(result.frequency);
        const noteName = midiToNoteName(midiInfo.note);
        this.tunerDetectedNote = noteName;
        this.tunerDetectedFreq = result.frequency;
        this.tunerClarity = result.clarity;
        const centsFromTarget = 1200 * Math.log2(result.frequency / target.freq);
        this.tunerCents = Math.round(centsFromTarget);
        const absCents = Math.abs(this.tunerCents);
        if (absCents > 2400) {
          this.tunerStatus = "waiting";
          this.tunerHoldProgress = 0;
          this.inTuneSince = null;
          return;
        }
        if (absCents <= GuitarInputStore.IN_TUNE_CENTS) {
          this.tunerStatus = "holding";
          if (this.inTuneSince === null) {
            this.inTuneSince = Date.now();
          }
          const held = Date.now() - this.inTuneSince;
          this.tunerHoldProgress = Math.min(1, held / GuitarInputStore.IN_TUNE_HOLD_MS);
          if (held >= GuitarInputStore.IN_TUNE_HOLD_MS) {
            this.tunerStatus = "in-tune";
            this.advanceTunerString();
          }
        } else {
          this.inTuneSince = null;
          this.tunerHoldProgress = 0;
          this.tunerStatus = this.tunerCents > 0 ? "sharp" : "flat";
        }
      };
      source.connect(proc);
      proc.connect(ctx.destination);
    }
    tunerStream = null;
    tunerContext = null;
    tunerProcessor = null;
    advanceTunerString() {
      this.inTuneSince = null;
      this.tunerHoldProgress = 0;
      if (this.tunerStringIndex >= 5) {
        this.tunerPhase = "complete";
        this.calibrated = true;
        this.calibrating = false;
        this.calibrationStatus = "Tuning complete! Ready to play.";
        this.stopTunerCapture();
        setTimeout(() => {
          this.tunerActive = false;
          this.tunerPhase = "noise-floor";
        }, 2e3);
        setTimeout(() => {
          if (this.tunerPhase === "complete") {
            this.tunerActive = false;
          }
        }, 2e3);
      } else {
        this.tunerStringIndex++;
        this.tunerStatus = "waiting";
        this.tunerDetectedNote = "";
        this.tunerCents = 0;
      }
    }
    skipTunerString() {
      if (!this.tunerActive || this.tunerPhase !== "tuning") return;
      this.advanceTunerString();
    }
    cancelTuner() {
      this.stopTunerCapture();
      this.tunerActive = false;
      this.tunerPhase = "noise-floor";
      this.calibrating = false;
      this.calibrationStatus = this.calibrated ? "Calibration preserved" : "";
    }
    async stopTunerCapture() {
      if (this.tunerProcessor) {
        this.tunerProcessor.disconnect();
        this.tunerProcessor = null;
      }
      if (this.tunerStream) {
        this.tunerStream.getTracks().forEach((t) => t.stop());
        this.tunerStream = null;
      }
      if (this.tunerContext) {
        await this.tunerContext.close();
        this.tunerContext = null;
      }
      if (this.tunerCapture) {
        await this.tunerCapture.stop();
        this.tunerCapture = null;
      }
      if (this.tunerAnimFrame !== null) {
        cancelAnimationFrame(this.tunerAnimFrame);
        this.tunerAnimFrame = null;
      }
    }
    setLatency(value) {
      this.latencyMs = Math.max(1, Math.min(50, value));
      this.syncConfig();
      this.persist();
    }
    setGain(value) {
      this.gain = Math.max(0.1, Math.min(2, Math.round(value * 20) / 20));
      this.syncConfig();
      this.persist();
    }
    setStringConfidence(value) {
      this.stringConfidence = Math.max(0.1, Math.min(1, Math.round(value * 20) / 20));
      this.syncConfig();
      this.persist();
    }
    async enumerateAudioDevices() {
      if (typeof navigator === "undefined" || !navigator.mediaDevices) {
        this.audioDeviceError = "Audio devices not available";
        return;
      }
      try {
        const stream = await navigator.mediaDevices.getUserMedia({
          audio: true
        });
        stream.getTracks().forEach((t) => t.stop());
        const allDevices = await navigator.mediaDevices.enumerateDevices();
        this.audioDevices = allDevices.filter((d) => d.kind === "audioinput");
        this.audioDeviceError = "";
        if (this.audioDevices.length > 0 && !this.selectedDeviceId) {
          const audient = this.audioDevices.find((d) => d.label.toLowerCase().includes("audient"));
          this.selectDevice(audient ? audient.deviceId : this.audioDevices[0].deviceId);
        }
      } catch (err) {
        this.audioDeviceError = err instanceof Error ? err.message : "Failed to enumerate audio devices";
      }
    }
    selectDevice(deviceId) {
      this.selectedDeviceId = deviceId;
      this.selectedChannel = 1;
      this.persist();
      this.probeChannelCount(deviceId);
    }
    selectChannel(channel) {
      this.selectedChannel = channel;
      this.persist();
    }
    setManualMaxChannels(count) {
      this.manualMaxChannels = count;
      if (count != null && count > 0) {
        this.maxChannels = count;
      } else if (this.selectedDeviceId) {
        this.probeChannelCount(this.selectedDeviceId);
      }
    }
    async probeChannelCount(deviceId) {
      if (this.manualMaxChannels != null && this.manualMaxChannels > 0) {
        this.maxChannels = this.manualMaxChannels;
        return;
      }
      if (typeof navigator === "undefined" || !navigator.mediaDevices) return;
      try {
        const stream = await navigator.mediaDevices.getUserMedia({
          audio: {
            deviceId: {
              exact: deviceId
            },
            channelCount: {
              ideal: 32
            }
          }
        });
        const track = stream.getAudioTracks()[0];
        if (track) {
          const settings = track.getSettings();
          this.maxChannels = settings.channelCount ?? 2;
        }
        stream.getTracks().forEach((t) => t.stop());
      } catch {
        this.maxChannels = 2;
      }
    }
    backendAudioDevices = [];
    async loadAudioDevices() {
      try {
        this.backendAudioDevices = await adapter.listAudioDevices();
      } catch {
        this.backendAudioDevices = [];
      }
    }
    async syncConfig() {
      try {
        await adapter.setGuitarConfig({
          latencyMs: this.latencyMs,
          gain: this.gain,
          stringConfidence: this.stringConfidence,
          bends: this.bendsEnabled,
          legato: this.legatoEnabled,
          slides: this.slidesEnabled,
          vibrato: this.vibratoEnabled
        });
      } catch {
      }
    }
    async syncDevice() {
      try {
        const device = this.audioDevices.find((d) => d.deviceId === this.selectedDeviceId);
        const deviceName = device?.label || this.selectedDeviceId;
        const channel0 = Math.max(0, this.selectedChannel - 1);
        await adapter.setGuitarDevice(deviceName, channel0);
      } catch {
      }
    }
  }
  const guitar = new GuitarInputStore();
  function mapEngineState(raw, isRunning) {
    return {
      key: raw.key,
      mode: raw.mode,
      modeNumber: raw.mode_number,
      scaleMode: raw.scale_mode,
      octaveMode: raw.octave_mode,
      voiceLeadingEnabled: raw.voice_leading_enabled,
      voiceLeadingStyle: raw.voice_leading_style,
      interchangeEnabled: raw.interchange_enabled,
      interchangeRange: raw.borrowing_range,
      voicePosition: raw.voice_position,
      voiceCount: raw.voice_count,
      isRunning
    };
  }
  function mapNoteState(raw) {
    return {
      inputNotes: raw.input_notes,
      harmonyNotes: raw.harmony_notes,
      borrowedNotes: raw.borrowed_notes,
      chordName: raw.chord_name,
      lastBorrowedFrom: raw.last_borrowed_from
    };
  }
  class TauriAdapter {
    _isRunning = false;
    _guitarSignalUnsub = null;
    async init() {
      await this.getEngineState();
    }
    async getEngineState() {
      try {
        const raw = await invoke("get_engine_state");
        return mapEngineState(raw, this._isRunning);
      } catch (e) {
        throw new Error(`Failed to get engine state: ${e}`);
      }
    }
    async setKey(key) {
      try {
        await invoke("set_key", {
          key
        });
      } catch (e) {
        throw new Error(`Failed to set key: ${e}`);
      }
    }
    async setMode(mode) {
      try {
        await invoke("set_mode", {
          mode
        });
      } catch (e) {
        throw new Error(`Failed to set mode: ${e}`);
      }
    }
    async setScaleMode(mode) {
      try {
        await invoke("set_scale_mode", {
          mode
        });
      } catch (e) {
        throw new Error(`Failed to set scale mode: ${e}`);
      }
    }
    async setOctaveMode(mode) {
      try {
        await invoke("set_octave_mode", {
          mode
        });
      } catch (e) {
        throw new Error(`Failed to set octave mode: ${e}`);
      }
    }
    async setVoiceLeading(enabled, style) {
      try {
        await invoke("set_voice_leading", {
          enabled,
          style
        });
      } catch (e) {
        throw new Error(`Failed to set voice leading: ${e}`);
      }
    }
    async setInterchange(enabled, range) {
      try {
        await invoke("set_interchange", {
          enabled,
          range
        });
      } catch (e) {
        throw new Error(`Failed to set interchange: ${e}`);
      }
    }
    async setVoicePosition(position) {
      try {
        await invoke("set_voice_position", {
          position
        });
      } catch (e) {
        throw new Error(`Failed to set voice position: ${e}`);
      }
    }
    async setVoiceCount(count) {
      try {
        await invoke("set_voice_count", {
          count
        });
      } catch (e) {
        throw new Error(`Failed to set voice count: ${e}`);
      }
    }
    async getHumanizeState() {
      try {
        const raw = await invoke("get_humanize_state");
        return {
          enabled: raw.enabled,
          jitterEnabled: raw.jitter_enabled,
          jitterMinMs: raw.jitter_min_ms,
          jitterMaxMs: raw.jitter_max_ms,
          velocityEnabled: raw.velocity_enabled,
          velocityVariation: raw.velocity_variation,
          durationEnabled: raw.duration_enabled,
          durationVariationMs: raw.duration_variation_ms,
          swingEnabled: raw.swing_enabled,
          swingAmount: raw.swing_amount,
          bpm: raw.bpm,
          metronomeEnabled: raw.metronome_enabled
        };
      } catch (e) {
        throw new Error(`Failed to get humanize state: ${e}`);
      }
    }
    async setHumanizeConfig(config) {
      try {
        const snakeConfig = {};
        if (config.enabled !== void 0) snakeConfig.enabled = config.enabled;
        if (config.jitterEnabled !== void 0) snakeConfig.jitter_enabled = config.jitterEnabled;
        if (config.jitterMinMs !== void 0) snakeConfig.jitter_min_ms = config.jitterMinMs;
        if (config.jitterMaxMs !== void 0) snakeConfig.jitter_max_ms = config.jitterMaxMs;
        if (config.velocityEnabled !== void 0) snakeConfig.velocity_enabled = config.velocityEnabled;
        if (config.velocityVariation !== void 0) snakeConfig.velocity_variation = config.velocityVariation;
        if (config.durationEnabled !== void 0) snakeConfig.duration_enabled = config.durationEnabled;
        if (config.durationVariationMs !== void 0) snakeConfig.duration_variation_ms = config.durationVariationMs;
        if (config.swingEnabled !== void 0) snakeConfig.swing_enabled = config.swingEnabled;
        if (config.swingAmount !== void 0) snakeConfig.swing_amount = config.swingAmount;
        if (config.bpm !== void 0) snakeConfig.bpm = config.bpm;
        if (config.metronomeEnabled !== void 0) snakeConfig.metronome_enabled = config.metronomeEnabled;
        await invoke("set_humanize_config", {
          config: snakeConfig
        });
      } catch (e) {
        throw new Error(`Failed to set humanize config: ${e}`);
      }
    }
    async listMidiInputs() {
      try {
        const raw = await invoke("list_midi_inputs");
        return raw.map((d) => ({
          index: d.index,
          name: d.name
        }));
      } catch (e) {
        throw new Error(`Failed to list MIDI inputs: ${e}`);
      }
    }
    async listMidiOutputs() {
      try {
        const raw = await invoke("list_midi_outputs");
        return raw.map((d) => ({
          index: d.index,
          name: d.name
        }));
      } catch (e) {
        throw new Error(`Failed to list MIDI outputs: ${e}`);
      }
    }
    async refreshMidiDevices() {
      try {
        await invoke("refresh_midi_devices");
      } catch (e) {
        throw new Error(`Failed to refresh MIDI devices: ${e}`);
      }
    }
    async startRouting(inputIdx, outputIndices) {
      try {
        await invoke("start_routing", {
          inputIdx,
          outputIndices
        });
        this._isRunning = true;
        const GUITAR_AUDIO_SENTINEL2 = 999997;
        if (inputIdx === GUITAR_AUDIO_SENTINEL2) {
          guitar.detecting = true;
          let lastNoteUpdate = 0;
          const NOTE_UPDATE_INTERVAL = 100;
          this._guitarSignalUnsub = await listen("guitar-signal", (event) => {
            const p = event.payload;
            const rms = p.rms;
            const clarity = p.clarity;
            const noteName = p.note_name;
            guitar.pushSignalFrame(rms, clarity);
            const now = performance.now();
            if (now - lastNoteUpdate > NOTE_UPDATE_INTERVAL) {
              lastNoteUpdate = now;
              if (noteName) {
                guitar.currentNote = noteName;
                guitar.confidence = Math.round(clarity * 100);
                guitar.velocity = Math.round(rms * 800);
              } else {
                guitar.currentNote = "";
                guitar.confidence = 0;
              }
            }
          });
        }
      } catch (e) {
        throw new Error(`Failed to start routing: ${e}`);
      }
    }
    async stopRouting() {
      try {
        await invoke("stop_routing");
        this._isRunning = false;
        if (this._guitarSignalUnsub) {
          this._guitarSignalUnsub();
          this._guitarSignalUnsub = null;
        }
        guitar.detecting = false;
        guitar.currentNote = "";
        guitar.confidence = 0;
        guitar.velocity = 0;
      } catch (e) {
        throw new Error(`Failed to stop routing: ${e}`);
      }
    }
    async injectNoteOn(note, velocity) {
      try {
        return await invoke("inject_note_on", {
          note,
          velocity: velocity ?? 100
        });
      } catch {
        return [
          note
        ];
      }
    }
    async injectNoteOff(note) {
      try {
        return await invoke("inject_note_off", {
          note
        });
      } catch {
        return [
          note
        ];
      }
    }
    async getNoteState() {
      try {
        const raw = await invoke("get_note_state");
        return mapNoteState(raw);
      } catch (e) {
        throw new Error(`Failed to get note state: ${e}`);
      }
    }
    onNoteUpdate(callback) {
      let unlisten;
      let cancelled = false;
      listen("note-update", (event) => {
        if (!cancelled) {
          callback(mapNoteState(event.payload));
        }
      }).then((fn) => {
        if (cancelled) {
          fn();
        } else {
          unlisten = fn;
        }
      });
      return () => {
        cancelled = true;
        unlisten?.();
      };
    }
    async listPresets() {
      try {
        const raw = await invoke("list_presets");
        return raw.map((p) => ({
          index: p.index,
          name: p.name,
          persona: p.persona,
          genre: p.genre,
          isBuiltin: p.is_builtin
        }));
      } catch (e) {
        throw new Error(`Failed to list presets: ${e}`);
      }
    }
    async loadPreset(name) {
      try {
        await invoke("load_preset", {
          name
        });
      } catch (e) {
        throw new Error(`Failed to load preset: ${e}`);
      }
    }
    async savePreset(name) {
      try {
        await invoke("save_preset", {
          name
        });
      } catch (e) {
        throw new Error(`Failed to save preset: ${e}`);
      }
    }
    async deletePreset(name) {
      try {
        await invoke("delete_preset", {
          name
        });
      } catch (e) {
        throw new Error(`Failed to delete preset: ${e}`);
      }
    }
    async listAudioDevices() {
      try {
        return await invoke("list_audio_devices");
      } catch (e) {
        throw new Error(`Failed to list audio devices: ${e}`);
      }
    }
    async setGuitarDevice(deviceName, channel) {
      try {
        await invoke("set_guitar_device", {
          deviceName,
          channel
        });
      } catch (e) {
        throw new Error(`Failed to set guitar device: ${e}`);
      }
    }
    async setGuitarConfig(config) {
      try {
        await invoke("set_guitar_config", {
          latencyMs: config.latencyMs,
          gain: config.gain,
          stringConfidence: config.stringConfidence,
          bends: config.bends,
          legato: config.legato,
          slides: config.slides,
          vibrato: config.vibrato
        });
      } catch (e) {
        throw new Error(`Failed to set guitar config: ${e}`);
      }
    }
    _detuneCents = 0;
    setDetune(cents) {
      this._detuneCents = cents;
    }
    getDetune() {
      return this._detuneCents;
    }
  }
  let wasmModule = null;
  let engine = null;
  const GUITAR_AUDIO_SENTINEL = 999997;
  class WasmAdapter {
    initialized = false;
    _isRunning = false;
    noteUpdateCallback = null;
    pollingHandle = null;
    midiAccess = null;
    activeInput = null;
    activeOutputs = [];
    _detuneCents = 0;
    pitchBendRangeSemitones = 2;
    guitarCapture = null;
    _guitarDeviceId = "";
    _guitarChannel = 0;
    async init() {
      if (this.initialized) return;
      try {
        wasmModule = await import("./contrapunk_wasm.js");
        if (wasmModule.default && typeof wasmModule.default === "function") {
          await wasmModule.default();
        }
        engine = new wasmModule.Engine();
        this.initialized = true;
      } catch (e) {
        throw new Error(`Failed to initialize WASM: ${e}`);
      }
    }
    ensureInit() {
      if (!this.initialized || !engine) {
        throw new Error("WASM adapter not initialized. Call init() first.");
      }
    }
    async getEngineState() {
      this.ensureInit();
      try {
        const raw = engine.get_state();
        return {
          key: raw.key ?? "C",
          mode: raw.mode ?? "PassThrough",
          modeNumber: raw.mode_number ?? 1,
          scaleMode: raw.scale_mode ?? "Ionian",
          octaveMode: raw.octave_mode ?? "None",
          voiceLeadingEnabled: raw.voice_leading_enabled ?? false,
          voiceLeadingStyle: raw.voice_leading_style ?? "Free",
          interchangeEnabled: raw.interchange_enabled ?? false,
          interchangeRange: raw.borrowing_range ?? 3,
          voicePosition: raw.voice_position ?? 0,
          voiceCount: raw.voice_count ?? 2,
          isRunning: this._isRunning
        };
      } catch (e) {
        throw new Error(`Failed to get engine state: ${e}`);
      }
    }
    async setKey(key) {
      this.ensureInit();
      try {
        engine.set_key(key);
      } catch (e) {
        throw new Error(`Failed to set key: ${e}`);
      }
    }
    async setMode(mode) {
      this.ensureInit();
      try {
        engine.set_mode(mode);
      } catch (e) {
        throw new Error(`Failed to set mode: ${e}`);
      }
    }
    async setScaleMode(mode) {
      this.ensureInit();
      try {
        engine.set_scale_mode(mode);
      } catch (e) {
        throw new Error(`Failed to set scale mode: ${e}`);
      }
    }
    async setOctaveMode(mode) {
      this.ensureInit();
      try {
        engine.set_octave_mode(mode);
      } catch (e) {
        throw new Error(`Failed to set octave mode: ${e}`);
      }
    }
    async setVoiceLeading(enabled, style) {
      this.ensureInit();
      try {
        engine.set_voice_leading(enabled, style);
      } catch (e) {
        throw new Error(`Failed to set voice leading: ${e}`);
      }
    }
    async setInterchange(enabled, range) {
      this.ensureInit();
      try {
        engine.set_interchange(enabled, range);
      } catch (e) {
        throw new Error(`Failed to set interchange: ${e}`);
      }
    }
    async setVoicePosition(position) {
      this.ensureInit();
      try {
        engine.set_voice_position(position);
      } catch (e) {
        throw new Error(`Failed to set voice position: ${e}`);
      }
    }
    async setVoiceCount(count) {
      this.ensureInit();
      try {
        engine.set_voice_count(count);
      } catch (e) {
        throw new Error(`Failed to set voice count: ${e}`);
      }
    }
    async getHumanizeState() {
      return {
        enabled: false,
        jitterEnabled: false,
        jitterMinMs: 1,
        jitterMaxMs: 10,
        velocityEnabled: false,
        velocityVariation: 10,
        durationEnabled: false,
        durationVariationMs: 0,
        swingEnabled: false,
        swingAmount: 0,
        bpm: 120,
        metronomeEnabled: false
      };
    }
    async setHumanizeConfig(_config) {
    }
    async ensureMidiAccess() {
      if (this.midiAccess) return this.midiAccess;
      if (typeof navigator === "undefined" || !("requestMIDIAccess" in navigator)) {
        return null;
      }
      try {
        this.midiAccess = await navigator.requestMIDIAccess();
        return this.midiAccess;
      } catch {
        return null;
      }
    }
    async listMidiInputs() {
      const access = await this.ensureMidiAccess();
      if (!access) return [];
      const devices = [];
      let index = 0;
      access.inputs.forEach((input) => {
        devices.push({
          index: index++,
          name: input.name ?? `Input ${index}`
        });
      });
      return devices;
    }
    async listMidiOutputs() {
      const access = await this.ensureMidiAccess();
      if (!access) return [];
      const devices = [];
      let index = 0;
      access.outputs.forEach((output) => {
        devices.push({
          index: index++,
          name: output.name ?? `Output ${index}`
        });
      });
      return devices;
    }
    async refreshMidiDevices() {
      this.midiAccess = null;
    }
    async startRouting(inputIdx, outputIndices) {
      this.ensureInit();
      if (inputIdx === GUITAR_AUDIO_SENTINEL) {
        await this.startGuitarCapture(outputIndices);
        return;
      }
      const access = await this.ensureMidiAccess();
      if (!access) {
        this._isRunning = true;
        return;
      }
      const inputs = Array.from(access.inputs.values());
      if (inputIdx >= 0 && inputIdx < inputs.length) {
        this.activeInput = inputs[inputIdx];
      }
      const outputs = Array.from(access.outputs.values());
      this.activeOutputs = outputIndices.filter((i) => i >= 0 && i < outputs.length).map((i) => outputs[i]);
      if (this.activeInput) {
        const self = this;
        const outs = this.activeOutputs;
        this.activeInput.onmidimessage = (event) => {
          if (!event.data || event.data.length < 2) return;
          const status = event.data[0] & 240;
          const note = event.data[1];
          const velocity = event.data.length > 2 ? event.data[2] : 0;
          let resultNotes = [];
          if (status === 144 && velocity > 0) {
            try {
              resultNotes = engine.note_on(note);
            } catch {
              resultNotes = [
                note
              ];
            }
            const sorted = self.sortVoices(resultNotes);
            for (let i = 0; i < sorted.length; i++) {
              if (outs.length > 0) {
                outs[i % outs.length].send([
                  144,
                  sorted[i],
                  velocity
                ]);
              }
            }
          } else if (status === 128 || status === 144 && velocity === 0) {
            try {
              resultNotes = engine.note_off(note);
            } catch {
              resultNotes = [
                note
              ];
            }
            const sorted = self.sortVoices(resultNotes);
            for (let i = 0; i < sorted.length; i++) {
              if (outs.length > 0) {
                outs[i % outs.length].send([
                  128,
                  sorted[i],
                  0
                ]);
              }
            }
          } else {
            for (const output of outs) {
              output.send(Array.from(event.data));
            }
          }
        };
      }
      this._isRunning = true;
      if (this._detuneCents !== 0) {
        this.sendPitchBend();
      }
    }
    async startGuitarCapture(outputIndices) {
      const access = await this.ensureMidiAccess();
      if (access) {
        const outputs = Array.from(access.outputs.values());
        this.activeOutputs = outputIndices.filter((i) => i >= 0 && i < outputs.length).map((i) => outputs[i]);
      }
      this.guitarCapture = new GuitarAudioCapture();
      const self = this;
      guitar.detecting = true;
      this.guitarCapture.noiseGateThreshold = guitar.noiseGateThreshold;
      this.guitarCapture.noiseGateEnabled = guitar.noiseGateEnabled;
      this.guitarCapture.clarityGateEnabled = false;
      const deviceId = guitar.selectedDeviceId || this._guitarDeviceId;
      const channelIndex = Math.max(0, guitar.selectedChannel - 1);
      console.log(`[wasm] startGuitarCapture: device='${deviceId}' channel=${channelIndex} (store.selectedChannel=${guitar.selectedChannel})`);
      await this.guitarCapture.start(deviceId, channelIndex, {
        onNoteOn(note, velocity) {
          self.injectNoteOn(note, velocity).catch(() => {
          });
        },
        onNoteOff(note) {
          self.injectNoteOff(note).catch(() => {
          });
        },
        onDetection(info) {
          if (self.guitarCapture) {
            self.guitarCapture.noiseGateThreshold = guitar.noiseGateThreshold;
            self.guitarCapture.noiseGateEnabled = guitar.noiseGateEnabled;
            guitar.activeChannel = self.guitarCapture.actualChannel + 1;
          }
          guitar.pushSignalFrame(info.rms, info.clarity);
          if (info.frequency !== null) {
            guitar.currentNote = info.noteName;
            guitar.confidence = Math.round(info.clarity * 100);
            guitar.velocity = Math.round(info.rms * 800);
          } else {
            guitar.currentNote = "";
            guitar.confidence = 0;
          }
        }
      });
      guitar.activeChannel = this.guitarCapture.actualChannel + 1;
      this._isRunning = true;
    }
    async stopRouting() {
      if (this.guitarCapture) {
        await this.guitarCapture.stop();
        this.guitarCapture = null;
        guitar.detecting = false;
        guitar.currentNote = "";
        guitar.confidence = 0;
        guitar.velocity = 0;
      }
      for (const output of this.activeOutputs) {
        try {
          output.send([
            176,
            123,
            0
          ]);
        } catch {
        }
      }
      if (engine) {
        try {
          engine.clear_notes();
        } catch {
        }
      }
      if (this.activeInput) {
        this.activeInput.onmidimessage = null;
        this.activeInput = null;
      }
      this.activeOutputs = [];
      this._isRunning = false;
      this.stopNotePolling();
    }
    async injectNoteOn(note, velocity) {
      this.ensureInit();
      try {
        const result = engine.note_on(note);
        const sorted = this.sortVoices(result ?? [
          note
        ]);
        const vel = velocity ?? 100;
        for (let i = 0; i < sorted.length; i++) {
          if (this.activeOutputs.length > 0) {
            this.activeOutputs[i % this.activeOutputs.length].send([
              144,
              sorted[i],
              vel
            ]);
          }
        }
        return sorted;
      } catch {
        return [
          note
        ];
      }
    }
    async injectNoteOff(note) {
      this.ensureInit();
      try {
        const result = engine.note_off(note);
        const sorted = this.sortVoices(result ?? [
          note
        ]);
        for (let i = 0; i < sorted.length; i++) {
          if (this.activeOutputs.length > 0) {
            this.activeOutputs[i % this.activeOutputs.length].send([
              128,
              sorted[i],
              0
            ]);
          }
        }
        return sorted;
      } catch {
        return [
          note
        ];
      }
    }
    async getNoteState() {
      this.ensureInit();
      try {
        const raw = engine.get_note_state();
        return {
          inputNotes: raw?.input_notes ?? [],
          harmonyNotes: raw?.harmony_notes ?? [],
          borrowedNotes: raw?.borrowed_notes ?? [],
          chordName: raw?.chord_name ?? "",
          lastBorrowedFrom: raw?.last_borrowed_from ?? ""
        };
      } catch {
        return {
          inputNotes: [],
          harmonyNotes: [],
          borrowedNotes: [],
          chordName: "",
          lastBorrowedFrom: ""
        };
      }
    }
    onNoteUpdate(callback) {
      this.noteUpdateCallback = callback;
      this.startNotePolling();
      return () => {
        this.noteUpdateCallback = null;
        this.stopNotePolling();
      };
    }
    startNotePolling() {
      if (this.pollingHandle !== null) return;
      const poll = () => {
        if (!this.noteUpdateCallback || !this._isRunning) {
          this.pollingHandle = null;
          return;
        }
        this.getNoteState().then((state) => {
          this.noteUpdateCallback?.(state);
        }).catch(() => {
        });
        this.pollingHandle = requestAnimationFrame(poll);
      };
      this.pollingHandle = requestAnimationFrame(poll);
    }
    sortVoices(notes) {
      return [
        ...notes
      ].sort((a, b) => a - b);
    }
    centsToPitchBend(cents) {
      const maxCents = this.pitchBendRangeSemitones * 100;
      const normalized = Math.max(-1, Math.min(1, cents / maxCents));
      return Math.round(8192 + normalized * 8191);
    }
    sendPitchBend() {
      const bend = this.centsToPitchBend(this._detuneCents);
      const lsb = bend & 127;
      const msb = bend >> 7 & 127;
      for (const output of this.activeOutputs) {
        output.send([
          224,
          lsb,
          msb
        ]);
      }
    }
    async listAudioDevices() {
      if (typeof navigator === "undefined" || !navigator.mediaDevices) {
        return [];
      }
      try {
        const stream = await navigator.mediaDevices.getUserMedia({
          audio: true
        });
        stream.getTracks().forEach((t) => t.stop());
        const allDevices = await navigator.mediaDevices.enumerateDevices();
        return allDevices.filter((d) => d.kind === "audioinput").map((d) => d.label || `Audio Input ${d.deviceId.slice(0, 8)}`);
      } catch {
        return [];
      }
    }
    async setGuitarDevice(deviceName, channel) {
      this._guitarDeviceId = deviceName;
      this._guitarChannel = Math.max(0, channel);
    }
    async setGuitarConfig(_config) {
    }
    setDetune(cents) {
      this._detuneCents = cents;
      if (this._isRunning) {
        this.sendPitchBend();
      }
    }
    getDetune() {
      return this._detuneCents;
    }
    stopNotePolling() {
      if (this.pollingHandle !== null) {
        cancelAnimationFrame(this.pollingHandle);
        this.pollingHandle = null;
      }
    }
    async listPresets() {
      this.ensureInit();
      try {
        const raw = engine.list_presets();
        return (raw ?? []).map((p, i) => ({
          index: i,
          name: p.name ?? "",
          persona: p.persona ?? "",
          genre: p.genre ?? "",
          isBuiltin: p.is_builtin ?? true
        }));
      } catch {
        return [];
      }
    }
    async loadPreset(name) {
      this.ensureInit();
      try {
        engine.load_preset(name);
      } catch (e) {
        throw new Error(`Failed to load preset: ${e}`);
      }
    }
    async savePreset(name) {
      this.ensureInit();
      try {
        engine.save_preset(name);
      } catch (e) {
        throw new Error(`Failed to save preset: ${e}`);
      }
    }
    async deletePreset(name) {
      this.ensureInit();
      try {
        engine.delete_preset(name);
      } catch (e) {
        throw new Error(`Failed to delete preset: ${e}`);
      }
    }
  }
  function isTauri() {
    return typeof window !== "undefined" && ("__TAURI_INTERNALS__" in window || "__TAURI__" in window);
  }
  const platformName = isTauri() ? "tauri" : "browser";
  adapter = isTauri() ? new TauriAdapter() : new WasmAdapter();
  class UiStore {
    reducedMotion = false;
    animationsEnabled = true;
    platform = platformName;
    initialized = false;
    error = null;
    sidebarCollapsed = false;
    activePanel = "play";
    toggleAnimations() {
      this.animationsEnabled = !this.animationsEnabled;
      this.reducedMotion = !this.animationsEnabled;
      this.applyMotionPreference();
    }
    applyMotionPreference() {
      if (typeof document === "undefined") return;
      if (this.reducedMotion) {
        document.body.classList.add("reduced-motion");
      } else {
        document.body.classList.remove("reduced-motion");
      }
    }
    detectSystemMotionPreference() {
      if (typeof window === "undefined") return;
      const mediaQuery = window.matchMedia("(prefers-reduced-motion: reduce)");
      if (mediaQuery.matches) {
        this.reducedMotion = true;
        this.animationsEnabled = false;
        this.applyMotionPreference();
      }
      mediaQuery.addEventListener("change", (e) => {
        this.reducedMotion = e.matches;
        this.animationsEnabled = !e.matches;
        this.applyMotionPreference();
      });
    }
    markInitialized() {
      this.initialized = true;
      this.error = null;
    }
    setError(message) {
      this.error = message;
    }
    clearError() {
      this.error = null;
    }
    toggleSidebar() {
      this.sidebarCollapsed = !this.sidebarCollapsed;
    }
    setActivePanel(panel) {
      this.activePanel = panel;
    }
  }
  ui = new UiStore();
})();
export {
  __tla,
  adapter as a,
  ui as u
};
