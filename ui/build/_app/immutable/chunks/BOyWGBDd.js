import { a5 as c, i as l, aa as u, a6 as N } from "./CA4AbA-g.js";
import { l as U, i as r } from "./BuXADhKH.js";
import { _ as ae } from "./Ct5FWWRu.js";
import { d as ue, f as he, m as de } from "./CfpmbwKP.js";
let T, $e, v, Le, je, m, Be, B, Ke;
let __tla = (async ()=>{
    let L = null;
    async function ge() {
        return L || (L = (await ae(()=>import("./CnTsoIij.js"), [], import.meta.url)).WasmGuitarInput), L;
    }
    const V = 1024;
    function H(h, e, t, n, s) {
        let a = 0;
        for(let i = 0; i < h.length; i++)a += h[i] * h[i];
        const o = Math.sqrt(a / h.length);
        n.count++, n.count % 25 === 0 && console.log(`[guitar] frame=${n.count} rms=${o.toFixed(4)} ch=${s}`);
        const g = e.process_block(h);
        let p;
        try {
            p = JSON.parse(g);
        } catch (i) {
            console.error("[guitar] Failed to parse WASM events:", i);
            return;
        }
        if (p.length > 0 && console.log(`[guitar] WASM events (${p.length}):`, JSON.stringify(p)), t.onDetection) {
            const i = p.findLast?.((b)=>b.type === "note_on");
            i ? t.onDetection({
                frequency: null,
                clarity: 1,
                noteName: q(i.note),
                midi: i.note,
                cents: 0,
                rms: o
            }) : t.onDetection({
                frequency: null,
                clarity: 0,
                noteName: "-",
                midi: 0,
                cents: 0,
                rms: o
            });
        }
        for (const i of p)switch(i.type){
            case "note_on":
                console.log(`[midi] NOTE ON: ${q(i.note)} (${i.note}) vel=${i.velocity} ch=${i.channel}`), t.onNoteOn(i.note, i.velocity);
                break;
            case "note_off":
                console.log(`[midi] NOTE OFF: ${q(i.note)} (${i.note}) ch=${i.channel}`), t.onNoteOff(i.note);
                break;
            case "pitch_bend":
                t.onPitchBend?.(i.channel, i.cents);
                break;
            case "midi_pitch_bend":
                t.onMidiPitchBend?.(i.channel, i.value);
                break;
            case "cc":
                t.onCC?.(i.channel, i.controller, i.value);
                break;
            case "channel_pressure":
                t.onChannelPressure?.(i.channel, i.pressure);
                break;
            case "vibrato":
                t.onVibratoStatus?.(i.active, i.rate_hz, i.depth_cents);
                break;
        }
    }
    class ie {
        audioContext = null;
        mediaStream = null;
        sourceNode = null;
        workletNode = null;
        processorNode = null;
        callbacks = null;
        dsp = null;
        _isRunning = !1;
        _actualChannel = 0;
        _frameCount = 0;
        overlapBuffer = null;
        overlapWritePos = 0;
        hopSize = 0;
        windowSize = 0;
        accumBuffer = null;
        accumWritePos = 0;
        get isRunning() {
            return this._isRunning;
        }
        get actualChannel() {
            return this._actualChannel;
        }
        noiseGateThreshold = .01;
        noiseGateEnabled = !0;
        clarityGateEnabled = !1;
        clarityThreshold = .7;
        async start(e, t, n, s = V) {
            this._isRunning && await this.stop(), this.callbacks = n;
            const a = await ge(), o = {
                audio: {
                    deviceId: e ? {
                        exact: e
                    } : void 0,
                    echoCancellation: !1,
                    noiseSuppression: !1,
                    autoGainControl: !1,
                    channelCount: {
                        ideal: 32
                    }
                }
            };
            this.mediaStream = await navigator.mediaDevices.getUserMedia(o), this.audioContext = new AudioContext, this.sourceNode = this.audioContext.createMediaStreamSource(this.mediaStream);
            const g = this.sourceNode.channelCount, p = this.audioContext.sampleRate;
            console.log(`[guitar] Device: ${g}ch @ ${p}Hz, want ch${t}, buffer=${s}`), this.windowSize = s, this.hopSize = Math.floor(s / 4), this.overlapBuffer = new Float32Array(s), this.overlapWritePos = 0, this.dsp = new a(p, s), this.dsp.set_onset_threshold(.015), this.dsp.set_string_confidence(.4), console.log(`[guitar] Overlap: window=${this.windowSize} hop=${this.hopSize} (75% overlap)`);
            const i = Math.max(t + 1, g);
            this.audioContext.audioWorklet ? await this._startWithWorklet(t, i, s) : (console.warn("[guitar] AudioWorklet not supported — falling back to ScriptProcessorNode"), this._startWithScriptProcessor(t, i, s)), this._isRunning = !0;
        }
        async _startWithWorklet(e, t, n) {
            const s = this.audioContext;
            await s.audioWorklet.addModule("/audio-capture-processor.js"), this.workletNode = new AudioWorkletNode(s, "audio-capture-processor", {
                numberOfInputs: 1,
                numberOfOutputs: 0,
                channelCount: t,
                channelCountMode: "explicit",
                channelInterpretation: "discrete",
                processorOptions: {
                    channelIndex: e
                }
            }), this.accumBuffer = new Float32Array(n), this.accumWritePos = 0;
            const a = this;
            let o = !1;
            this._actualChannel = e;
            const g = {
                count: 0
            };
            this.workletNode.port.onmessage = (p)=>{
                if (!a._isRunning || !a.callbacks || !a.dsp || !a.accumBuffer) return;
                const i = p.data.samples, b = p.data.channel;
                o || (console.log(`[guitar] Worklet processing: requested ch=${e}, using ch=${b}`), a._actualChannel = b, o = !0);
                let d = 0;
                for(; d < i.length;){
                    const f = n - a.accumWritePos, y = Math.min(f, i.length - d);
                    a.accumBuffer.set(i.subarray(d, d + y), a.accumWritePos), a.accumWritePos += y, d += y, a.accumWritePos >= n && (H(a.accumBuffer, a.dsp, a.callbacks, g, b), a._frameCount = g.count, a.accumWritePos = 0);
                }
            }, this.sourceNode.connect(this.workletNode);
        }
        _startWithScriptProcessor(e, t, n) {
            const s = this.audioContext;
            this.processorNode = s.createScriptProcessor(n, t, 1), this.processorNode.channelCountMode = "explicit", this.processorNode.channelInterpretation = "discrete";
            const a = this;
            let o = !1;
            this._actualChannel = e;
            const g = {
                count: 0
            };
            this.processorNode.onaudioprocess = (p)=>{
                if (!a._isRunning || !a.callbacks || !a.dsp) return;
                const i = p.inputBuffer, b = i.numberOfChannels, d = e < b ? e : 0;
                o || (console.log(`[guitar] Processing: requested ch=${e}, available=${b}, using ch=${d}`), a._actualChannel = d, o = !0);
                const f = i.getChannelData(d);
                H(f, a.dsp, a.callbacks, g, d), a._frameCount = g.count;
            }, this.sourceNode.connect(this.processorNode), this.processorNode.connect(s.destination);
        }
        async stop() {
            if (this._isRunning = !1, this.workletNode && (this.workletNode.port.onmessage = null, this.workletNode.disconnect(), this.workletNode = null), this.processorNode && (this.processorNode.onaudioprocess = null, this.processorNode.disconnect(), this.processorNode = null), this.sourceNode && (this.sourceNode.disconnect(), this.sourceNode = null), this.audioContext) {
                try {
                    await this.audioContext.close();
                } catch  {}
                this.audioContext = null;
            }
            if (this.mediaStream && (this.mediaStream.getTracks().forEach((e)=>e.stop()), this.mediaStream = null), this.dsp) {
                try {
                    this.dsp.free();
                } catch  {}
                this.dsp = null;
            }
            this.callbacks = null, this.accumBuffer = null, this.accumWritePos = 0;
        }
        setConfig(e) {
            this.dsp && (e.bends !== void 0 && this.dsp.set_bends_enabled(e.bends), e.legato !== void 0 && this.dsp.set_legato_enabled(e.legato), e.slides !== void 0 && this.dsp.set_slides_enabled(e.slides), e.vibrato !== void 0 && this.dsp.set_vibrato_enabled(e.vibrato), e.gain !== void 0 && this.dsp.set_input_gain(e.gain), e.onsetThreshold !== void 0 && this.dsp.set_onset_threshold(e.onsetThreshold), e.stringConfidence !== void 0 && this.dsp.set_string_confidence(e.stringConfidence));
        }
        async measureNoiseFloor(e, t = 3e3, n = 0) {
            const s = await navigator.mediaDevices.getUserMedia({
                audio: {
                    deviceId: e ? {
                        exact: e
                    } : void 0,
                    echoCancellation: !1,
                    noiseSuppression: !1,
                    autoGainControl: !1,
                    channelCount: {
                        ideal: 32
                    }
                }
            }), a = new AudioContext, o = a.createMediaStreamSource(s), g = Math.max(n + 1, o.channelCount);
            let p = 0, i = 0;
            const b = (d)=>{
                let f = 0;
                for(let y = 0; y < d.length; y++)f += d[y] * d[y];
                p += Math.sqrt(f / d.length), i++;
            };
            if (a.audioWorklet) {
                await a.audioWorklet.addModule("/audio-capture-processor.js");
                const d = new AudioWorkletNode(a, "audio-capture-processor", {
                    numberOfInputs: 1,
                    numberOfOutputs: 0,
                    channelCount: g,
                    channelCountMode: "explicit",
                    channelInterpretation: "discrete",
                    processorOptions: {
                        channelIndex: n
                    }
                });
                return d.port.onmessage = (f)=>{
                    b(f.data.samples);
                }, o.connect(d), new Promise((f)=>{
                    setTimeout(()=>{
                        d.port.onmessage = null, d.disconnect(), o.disconnect(), a.close().catch(()=>{}), s.getTracks().forEach((y)=>y.stop()), f(i > 0 ? p / i : 0);
                    }, t);
                });
            } else {
                const d = a.createScriptProcessor(V, g, 1);
                return d.channelCountMode = "explicit", d.channelInterpretation = "discrete", new Promise((f)=>{
                    d.onaudioprocess = (y)=>{
                        const w = Math.min(n, y.inputBuffer.numberOfChannels - 1);
                        b(y.inputBuffer.getChannelData(w));
                    }, o.connect(d), d.connect(a.destination), setTimeout(()=>{
                        d.onaudioprocess = null, d.disconnect(), o.disconnect(), a.close().catch(()=>{}), s.getTracks().forEach((y)=>y.stop()), f(i > 0 ? p / i : 0);
                    }, t);
                });
            }
        }
    }
    function q(h) {
        const e = [
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
        ], t = (h % 12 + 12) % 12, n = Math.floor(h / 12) - 1;
        return `${e[t]}${n}`;
    }
    const re = "contrapunk-guitar";
    function pe() {
        try {
            const h = localStorage.getItem(re);
            return h ? JSON.parse(h) : {};
        } catch  {
            return {};
        }
    }
    class E {
        #e = c(21);
        get latencyMs() {
            return l(this.#e);
        }
        set latencyMs(e) {
            u(this.#e, e, !0);
        }
        #t = c(1);
        get gain() {
            return l(this.#t);
        }
        set gain(e) {
            u(this.#t, e, !0);
        }
        #n = c(.4);
        get stringConfidence() {
            return l(this.#n);
        }
        set stringConfidence(e) {
            u(this.#n, e, !0);
        }
        #s = c(!0);
        get bendsEnabled() {
            return l(this.#s);
        }
        set bendsEnabled(e) {
            u(this.#s, e, !0);
        }
        #a = c(!0);
        get legatoEnabled() {
            return l(this.#a);
        }
        set legatoEnabled(e) {
            u(this.#a, e, !0);
        }
        #i = c(!0);
        get slidesEnabled() {
            return l(this.#i);
        }
        set slidesEnabled(e) {
            u(this.#i, e, !0);
        }
        #r = c(!1);
        get vibratoEnabled() {
            return l(this.#r);
        }
        set vibratoEnabled(e) {
            u(this.#r, e, !0);
        }
        #o = c(N([]));
        get audioDevices() {
            return l(this.#o);
        }
        set audioDevices(e) {
            u(this.#o, e, !0);
        }
        #c = c("");
        get selectedDeviceId() {
            return l(this.#c);
        }
        set selectedDeviceId(e) {
            u(this.#c, e, !0);
        }
        #l = c(1);
        get selectedChannel() {
            return l(this.#l);
        }
        set selectedChannel(e) {
            u(this.#l, e, !0);
        }
        #u = c(2);
        get maxChannels() {
            return l(this.#u);
        }
        set maxChannels(e) {
            u(this.#u, e, !0);
        }
        constructor(){
            const e = pe();
            e.latencyMs !== void 0 && (this.latencyMs = e.latencyMs), e.gain !== void 0 && (this.gain = e.gain), e.stringConfidence !== void 0 && (this.stringConfidence = e.stringConfidence), e.bendsEnabled !== void 0 && (this.bendsEnabled = e.bendsEnabled), e.legatoEnabled !== void 0 && (this.legatoEnabled = e.legatoEnabled), e.slidesEnabled !== void 0 && (this.slidesEnabled = e.slidesEnabled), e.vibratoEnabled !== void 0 && (this.vibratoEnabled = e.vibratoEnabled), e.selectedDeviceId && (this.selectedDeviceId = e.selectedDeviceId), e.selectedChannel && (this.selectedChannel = e.selectedChannel), e.calibrated && (this.calibrated = e.calibrated), e.noiseGateEnabled !== void 0 && (this.noiseGateEnabled = e.noiseGateEnabled), e.noiseGateThreshold !== void 0 && (this.noiseGateThreshold = e.noiseGateThreshold), e.freqGateEnabled !== void 0 && (this.freqGateEnabled = e.freqGateEnabled), e.freqGateRange !== void 0 && (this.freqGateRange = e.freqGateRange);
        }
        persist() {
            try {
                localStorage.setItem(re, JSON.stringify({
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
            } catch  {}
        }
        #h = c(null);
        get manualMaxChannels() {
            return l(this.#h);
        }
        set manualMaxChannels(e) {
            u(this.#h, e, !0);
        }
        #d = c("");
        get audioDeviceError() {
            return l(this.#d);
        }
        set audioDeviceError(e) {
            u(this.#d, e, !0);
        }
        #g = c(!1);
        get detecting() {
            return l(this.#g);
        }
        set detecting(e) {
            u(this.#g, e, !0);
        }
        #p = c(0);
        get activeChannel() {
            return l(this.#p);
        }
        set activeChannel(e) {
            u(this.#p, e, !0);
        }
        #m = c("");
        get currentNote() {
            return l(this.#m);
        }
        set currentNote(e) {
            u(this.#m, e, !0);
        }
        #y = c("");
        get currentString() {
            return l(this.#y);
        }
        set currentString(e) {
            u(this.#y, e, !0);
        }
        #f = c(0);
        get currentFret() {
            return l(this.#f);
        }
        set currentFret(e) {
            u(this.#f, e, !0);
        }
        #_ = c(0);
        get confidence() {
            return l(this.#_);
        }
        set confidence(e) {
            u(this.#_, e, !0);
        }
        #b = c(0);
        get velocity() {
            return l(this.#b);
        }
        set velocity(e) {
            u(this.#b, e, !0);
        }
        signalLevel = 0;
        signalClarity = 0;
        #v = c(!0);
        get noiseGateEnabled() {
            return l(this.#v);
        }
        set noiseGateEnabled(e) {
            u(this.#v, e, !0);
        }
        #w = c(.01);
        get noiseGateThreshold() {
            return l(this.#w);
        }
        set noiseGateThreshold(e) {
            u(this.#w, e, !0);
        }
        #C = c(!0);
        get freqGateEnabled() {
            return l(this.#C);
        }
        set freqGateEnabled(e) {
            u(this.#C, e, !0);
        }
        #S = c(1200);
        get freqGateRange() {
            return l(this.#S);
        }
        set freqGateRange(e) {
            u(this.#S, e, !0);
        }
        amplitudeHistory = [];
        clarityHistory = [];
        static HISTORY_SIZE = 128;
        pushSignalFrame(e, t) {
            this.signalLevel = Math.min(1, e * 5), this.signalClarity = t, this.amplitudeHistory.push(Math.min(1, e * 5)), this.clarityHistory.push(t), this.amplitudeHistory.length > E.HISTORY_SIZE && this.amplitudeHistory.shift(), this.clarityHistory.length > E.HISTORY_SIZE && this.clarityHistory.shift();
        }
        #E = c(!1);
        get calibrated() {
            return l(this.#E);
        }
        set calibrated(e) {
            u(this.#E, e, !0);
        }
        #M = c(!1);
        get calibrating() {
            return l(this.#M);
        }
        set calibrating(e) {
            u(this.#M, e, !0);
        }
        #P = c(!1);
        get tunerActive() {
            return l(this.#P);
        }
        set tunerActive(e) {
            u(this.#P, e, !0);
        }
        #I = c(0);
        get tunerStringIndex() {
            return l(this.#I);
        }
        set tunerStringIndex(e) {
            u(this.#I, e, !0);
        }
        #N = c("");
        get tunerDetectedNote() {
            return l(this.#N);
        }
        set tunerDetectedNote(e) {
            u(this.#N, e, !0);
        }
        #k = c(0);
        get tunerDetectedFreq() {
            return l(this.#k);
        }
        set tunerDetectedFreq(e) {
            u(this.#k, e, !0);
        }
        #D = c(0);
        get tunerCents() {
            return l(this.#D);
        }
        set tunerCents(e) {
            u(this.#D, e, !0);
        }
        #T = c(0);
        get tunerClarity() {
            return l(this.#T);
        }
        set tunerClarity(e) {
            u(this.#T, e, !0);
        }
        #A = c("waiting");
        get tunerStatus() {
            return l(this.#A);
        }
        set tunerStatus(e) {
            u(this.#A, e, !0);
        }
        #F = c(0);
        get tunerHoldProgress() {
            return l(this.#F);
        }
        set tunerHoldProgress(e) {
            u(this.#F, e, !0);
        }
        #R = c("noise-floor");
        get tunerPhase() {
            return l(this.#R);
        }
        set tunerPhase(e) {
            u(this.#R, e, !0);
        }
        #x = c(0);
        get tunerNoiseProgress() {
            return l(this.#x);
        }
        set tunerNoiseProgress(e) {
            u(this.#x, e, !0);
        }
        toggleTechnique(e) {
            switch(e){
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
        #O = c(0);
        get noiseFloorRms() {
            return l(this.#O);
        }
        set noiseFloorRms(e) {
            u(this.#O, e, !0);
        }
        #G = c("");
        get calibrationStatus() {
            return l(this.#G);
        }
        set calibrationStatus(e) {
            u(this.#G, e, !0);
        }
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
            if (!this.calibrating) {
                this.calibrating = !0, this.calibrated = !1, this.tunerActive = !0, this.tunerPhase = "noise-floor", this.tunerStringIndex = 0, this.tunerNoiseProgress = 0, this.tunerStatus = "waiting", this.tunerHoldProgress = 0, this.calibrationStatus = "Measuring noise floor...";
                try {
                    const t = new ie().measureNoiseFloor(this.selectedDeviceId, 3e3, this.selectedChannel - 1), n = Date.now(), s = setInterval(()=>{
                        const o = Date.now() - n;
                        this.tunerNoiseProgress = Math.min(1, o / 3e3);
                    }, 50), a = await t;
                    clearInterval(s), this.tunerNoiseProgress = 1, this.noiseFloorRms = a, this.calibrationStatus = `Noise floor: ${(a * 1e3).toFixed(1)} mRMS`, this.tunerPhase = "tuning", this.tunerStringIndex = 0, await this.startTunerCapture();
                } catch (e) {
                    this.calibrationStatus = e instanceof Error ? `Calibration failed: ${e.message}` : "Calibration failed", this.calibrated = !1, this.tunerActive = !1, this.calibrating = !1;
                }
            }
        }
        async startTunerCapture() {
            this.inTuneSince = null;
            const e = this.selectedChannel - 1, t = {
                audio: {
                    ...this.selectedDeviceId ? {
                        deviceId: {
                            exact: this.selectedDeviceId
                        }
                    } : {},
                    echoCancellation: !1,
                    noiseSuppression: !1,
                    autoGainControl: !1,
                    channelCount: {
                        ideal: 32
                    }
                }
            }, n = 2048, s = await navigator.mediaDevices.getUserMedia(t);
            this.tunerStream = s;
            const a = new AudioContext({
                sampleRate: 48e3
            });
            this.tunerContext = a;
            const o = a.createMediaStreamSource(s), g = Math.max(e + 1, o.channelCount), p = (i)=>{
                if (!this.tunerActive || this.tunerPhase !== "tuning") return;
                let b = 0;
                for(let M = 0; M < i.length; M++)b += i[M] * i[M];
                const d = Math.sqrt(b / i.length), f = ue(i, a.sampleRate), y = E.OPEN_STRINGS[this.tunerStringIndex];
                if (!y) return;
                if (!f || d < this.noiseFloorRms * 2) {
                    this.tunerDetectedNote = "", this.tunerDetectedFreq = 0, this.tunerCents = 0, this.tunerClarity = 0, this.tunerStatus = "waiting", this.tunerHoldProgress = 0, this.inTuneSince = null;
                    return;
                }
                const w = he(f.frequency), $ = de(w.note);
                this.tunerDetectedNote = $, this.tunerDetectedFreq = f.frequency, this.tunerClarity = f.clarity;
                const k = 1200 * Math.log2(f.frequency / y.freq);
                this.tunerCents = Math.round(k);
                const W = Math.abs(this.tunerCents);
                if (W > 2400) {
                    this.tunerStatus = "waiting", this.tunerHoldProgress = 0, this.inTuneSince = null;
                    return;
                }
                if (W <= E.IN_TUNE_CENTS) {
                    this.tunerStatus = "holding", this.inTuneSince === null && (this.inTuneSince = Date.now());
                    const M = Date.now() - this.inTuneSince;
                    this.tunerHoldProgress = Math.min(1, M / E.IN_TUNE_HOLD_MS), M >= E.IN_TUNE_HOLD_MS && (this.tunerStatus = "in-tune", this.advanceTunerString());
                } else this.inTuneSince = null, this.tunerHoldProgress = 0, this.tunerStatus = this.tunerCents > 0 ? "sharp" : "flat";
            };
            if (a.audioWorklet) {
                await a.audioWorklet.addModule("/audio-capture-processor.js");
                const i = new AudioWorkletNode(a, "audio-capture-processor", {
                    numberOfInputs: 1,
                    numberOfOutputs: 0,
                    channelCount: g,
                    channelCountMode: "explicit",
                    channelInterpretation: "discrete",
                    processorOptions: {
                        channelIndex: e
                    }
                });
                this.tunerWorklet = i;
                const b = new Float32Array(n);
                let d = 0;
                i.port.onmessage = (f)=>{
                    const y = f.data.samples;
                    let w = 0;
                    for(; w < y.length;){
                        const $ = n - d, k = Math.min($, y.length - w);
                        b.set(y.subarray(w, w + k), d), d += k, w += k, d >= n && (p(b), d = 0);
                    }
                }, o.connect(i);
            } else {
                const i = a.createScriptProcessor(n, g, 1);
                i.channelCountMode = "explicit", i.channelInterpretation = "discrete", this.tunerProcessor = i, i.onaudioprocess = (b)=>{
                    const d = b.inputBuffer, f = Math.min(e, d.numberOfChannels - 1);
                    p(d.getChannelData(f));
                }, o.connect(i), i.connect(a.destination);
            }
        }
        tunerStream = null;
        tunerContext = null;
        tunerWorklet = null;
        tunerProcessor = null;
        advanceTunerString() {
            this.inTuneSince = null, this.tunerHoldProgress = 0, this.tunerStringIndex >= 5 ? (this.tunerPhase = "complete", this.calibrated = !0, this.calibrating = !1, this.calibrationStatus = "Tuning complete! Ready to play.", this.stopTunerCapture(), setTimeout(()=>{
                this.tunerActive = !1, this.tunerPhase = "noise-floor";
            }, 2e3), setTimeout(()=>{
                this.tunerPhase === "complete" && (this.tunerActive = !1);
            }, 2e3)) : (this.tunerStringIndex++, this.tunerStatus = "waiting", this.tunerDetectedNote = "", this.tunerCents = 0);
        }
        skipTunerString() {
            !this.tunerActive || this.tunerPhase !== "tuning" || this.advanceTunerString();
        }
        cancelTuner() {
            this.stopTunerCapture(), this.tunerActive = !1, this.tunerPhase = "noise-floor", this.calibrating = !1, this.calibrationStatus = this.calibrated ? "Calibration preserved" : "";
        }
        async stopTunerCapture() {
            this.tunerWorklet && (this.tunerWorklet.port.onmessage = null, this.tunerWorklet.disconnect(), this.tunerWorklet = null), this.tunerProcessor && (this.tunerProcessor.onaudioprocess = null, this.tunerProcessor.disconnect(), this.tunerProcessor = null), this.tunerStream && (this.tunerStream.getTracks().forEach((e)=>e.stop()), this.tunerStream = null), this.tunerContext && (await this.tunerContext.close(), this.tunerContext = null), this.tunerCapture && (await this.tunerCapture.stop(), this.tunerCapture = null), this.tunerAnimFrame !== null && (cancelAnimationFrame(this.tunerAnimFrame), this.tunerAnimFrame = null);
        }
        setLatency(e) {
            this.latencyMs = Math.max(1, Math.min(50, e)), this.syncConfig(), this.persist();
        }
        setGain(e) {
            this.gain = Math.max(.1, Math.min(2, Math.round(e * 20) / 20)), this.syncConfig(), this.persist();
        }
        setStringConfidence(e) {
            this.stringConfidence = Math.max(.1, Math.min(1, Math.round(e * 20) / 20)), this.syncConfig(), this.persist();
        }
        async enumerateAudioDevices() {
            if (typeof navigator > "u" || !navigator.mediaDevices) {
                this.audioDeviceError = "Audio devices not available";
                return;
            }
            try {
                (await navigator.mediaDevices.getUserMedia({
                    audio: !0
                })).getTracks().forEach((n)=>n.stop());
                const t = await navigator.mediaDevices.enumerateDevices();
                if (this.audioDevices = t.filter((n)=>n.kind === "audioinput"), this.audioDeviceError = "", this.audioDevices.length > 0 && !this.selectedDeviceId) {
                    const n = this.audioDevices.find((s)=>s.label.toLowerCase().includes("audient"));
                    this.selectDevice(n ? n.deviceId : this.audioDevices[0].deviceId);
                }
            } catch (e) {
                this.audioDeviceError = e instanceof Error ? e.message : "Failed to enumerate audio devices";
            }
        }
        selectDevice(e) {
            this.selectedDeviceId = e, this.selectedChannel = 1, this.persist(), this.probeChannelCount(e);
        }
        selectChannel(e) {
            this.selectedChannel = e, this.persist();
        }
        setManualMaxChannels(e) {
            this.manualMaxChannels = e, e != null && e > 0 ? this.maxChannels = e : this.selectedDeviceId && this.probeChannelCount(this.selectedDeviceId);
        }
        async probeChannelCount(e) {
            if (this.manualMaxChannels != null && this.manualMaxChannels > 0) {
                this.maxChannels = this.manualMaxChannels;
                return;
            }
            if (!(typeof navigator > "u" || !navigator.mediaDevices)) try {
                const t = await navigator.mediaDevices.getUserMedia({
                    audio: {
                        deviceId: {
                            exact: e
                        },
                        channelCount: {
                            ideal: 32
                        }
                    }
                }), n = t.getAudioTracks()[0];
                if (n) {
                    const s = n.getSettings();
                    this.maxChannels = s.channelCount ?? 2;
                }
                t.getTracks().forEach((s)=>s.stop());
            } catch  {
                this.maxChannels = 2;
            }
        }
        #B = c(N([]));
        get backendAudioDevices() {
            return l(this.#B);
        }
        set backendAudioDevices(e) {
            u(this.#B, e, !0);
        }
        async loadAudioDevices() {
            try {
                this.backendAudioDevices = await v.listAudioDevices();
            } catch  {
                this.backendAudioDevices = [];
            }
        }
        async syncConfig() {
            try {
                await v.setGuitarConfig({
                    latencyMs: this.latencyMs,
                    gain: this.gain,
                    stringConfidence: this.stringConfidence,
                    bends: this.bendsEnabled,
                    legato: this.legatoEnabled,
                    slides: this.slidesEnabled,
                    vibrato: this.vibratoEnabled
                });
            } catch  {}
        }
        async syncDevice() {
            try {
                const t = this.audioDevices.find((s)=>s.deviceId === this.selectedDeviceId)?.label || this.selectedDeviceId, n = Math.max(0, this.selectedChannel - 1);
                await v.setGuitarDevice(t, n);
            } catch  {}
        }
    }
    m = new E;
    class me {
        #e = c(!1);
        get running() {
            return l(this.#e);
        }
        set running(e) {
            u(this.#e, e, !0);
        }
        #t = c(120);
        get bpm() {
            return l(this.#t);
        }
        set bpm(e) {
            u(this.#t, e, !0);
        }
        #n = c(4);
        get beatsPerBar() {
            return l(this.#n);
        }
        set beatsPerBar(e) {
            u(this.#n, e, !0);
        }
        #s = c(4);
        get beatUnit() {
            return l(this.#s);
        }
        set beatUnit(e) {
            u(this.#s, e, !0);
        }
        #a = c(0);
        get beatInBar() {
            return l(this.#a);
        }
        set beatInBar(e) {
            u(this.#a, e, !0);
        }
        #i = c(0);
        get totalBeat() {
            return l(this.#i);
        }
        set totalBeat(e) {
            u(this.#i, e, !0);
        }
        #r = c(0);
        get bar() {
            return l(this.#r);
        }
        set bar(e) {
            u(this.#r, e, !0);
        }
        #o = c(48e3);
        get sampleRate() {
            return l(this.#o);
        }
        set sampleRate(e) {
            u(this.#o, e, !0);
        }
        #c = c(0);
        get pulse() {
            return l(this.#c);
        }
        set pulse(e) {
            u(this.#c, e, !0);
        }
        #l = c(0);
        get lastCrossedAt() {
            return l(this.#l);
        }
        set lastCrossedAt(e) {
            u(this.#l, e, !0);
        }
        #u = c(!1);
        get metronomeEnabled() {
            return l(this.#u);
        }
        set metronomeEnabled(e) {
            u(this.#u, e, !0);
        }
        async syncFromBackend() {
            try {
                const e = await v.getTransportState();
                this.running = e.running, this.bpm = e.bpm, this.beatsPerBar = e.beatsPerBar, this.beatUnit = e.beatUnit, this.sampleRate = e.sampleRate, this.bar = e.bar, this.metronomeEnabled = e.metronomeEnabled;
            } catch  {}
        }
        async play() {
            await v.transportPlay(), this.running = !0;
        }
        async stop() {
            await v.transportStop(), this.running = !1;
        }
        async reset() {
            await v.transportReset(), this.totalBeat = 0, this.bar = 0, this.beatInBar = 0;
        }
        async setBpm(e) {
            const t = Math.max(20, Math.min(400, e));
            this.bpm = t, await v.setBpm(t);
        }
        async setTimeSignature(e, t) {
            this.beatsPerBar = e, this.beatUnit = t, await v.setTimeSignature(e, t);
        }
        async setMetronomeEnabled(e) {
            this.metronomeEnabled = e, await v.setMetronomeEnabled(e);
        }
        async toggleMetronome() {
            await this.setMetronomeEnabled(!this.metronomeEnabled);
        }
        applyBeatUpdate(e) {
            this.totalBeat = e.totalBeat, this.beatInBar = e.beatInBar, this.bar = e.bar, this.bpm = e.bpm, this.running = e.running, this.pulse = this.pulse + 1, this.lastCrossedAt = performance.now();
        }
    }
    B = new me;
    function ye(h, e) {
        return {
            key: oe(h.key),
            mode: h.mode,
            modeNumber: h.mode_number,
            scaleMode: h.scale_mode,
            octaveMode: h.octave_mode,
            voiceLeadingEnabled: h.voice_leading_enabled,
            voiceLeadingStyle: h.voice_leading_style,
            interchangeEnabled: h.interchange_enabled,
            interchangeRange: h.borrowing_range,
            voicePosition: h.voice_position,
            voiceCount: h.voice_count,
            autoKey: h.auto_key,
            isRunning: e,
            counterpointSpecies: h.counterpoint_species ?? "Species1",
            counterpointStrictness: h.counterpoint_strictness ?? "Strict"
        };
    }
    const fe = {
        Db: "C#",
        Eb: "D#",
        Gb: "F#",
        Ab: "G#",
        Bb: "A#"
    };
    function oe(h) {
        return fe[h] ?? h;
    }
    function _e(h) {
        return {
            inputNotes: h.input_notes,
            harmonyNotes: h.harmony_notes,
            borrowedNotes: h.borrowed_notes,
            chordName: h.chord_name,
            lastBorrowedFrom: h.last_borrowed_from,
            currentKey: oe(h.current_key ?? "C")
        };
    }
    class be {
        _isRunning = !1;
        _guitarSignalUnsub = null;
        _beatUpdateUnsub = null;
        async init() {
            await this.getEngineState(), this._beatUpdateUnsub = await U("beat-update", (e)=>{
                const t = e.payload;
                B.applyBeatUpdate({
                    totalBeat: t.total_beat,
                    beatInBar: t.beat_in_bar,
                    bar: t.bar,
                    bpm: t.bpm,
                    running: t.running
                });
            });
        }
        async getEngineState() {
            try {
                const e = await r("get_engine_state");
                return ye(e, this._isRunning);
            } catch (e) {
                throw new Error(`Failed to get engine state: ${e}`);
            }
        }
        async setKey(e) {
            try {
                await r("set_key", {
                    key: e
                });
            } catch (t) {
                throw new Error(`Failed to set key: ${t}`);
            }
        }
        async setMode(e) {
            try {
                await r("set_mode", {
                    mode: e
                });
            } catch (t) {
                throw new Error(`Failed to set mode: ${t}`);
            }
        }
        async setScaleMode(e) {
            try {
                await r("set_scale_mode", {
                    mode: e
                });
            } catch (t) {
                throw new Error(`Failed to set scale mode: ${t}`);
            }
        }
        async setOctaveMode(e) {
            try {
                await r("set_octave_mode", {
                    mode: e
                });
            } catch (t) {
                throw new Error(`Failed to set octave mode: ${t}`);
            }
        }
        async setOctaveIntensity(e) {
            try {
                await r("set_octave_intensity", {
                    amount: e
                });
            } catch (t) {
                throw new Error(`Failed to set octave intensity: ${t}`);
            }
        }
        async setVoiceLeading(e, t) {
            try {
                await r("set_voice_leading", {
                    enabled: e,
                    style: t
                });
            } catch (n) {
                throw new Error(`Failed to set voice leading: ${n}`);
            }
        }
        async setInterchange(e, t) {
            try {
                await r("set_interchange", {
                    enabled: e,
                    range: t
                });
            } catch (n) {
                throw new Error(`Failed to set interchange: ${n}`);
            }
        }
        async setVoicePosition(e) {
            try {
                await r("set_voice_position", {
                    position: e
                });
            } catch (t) {
                throw new Error(`Failed to set voice position: ${t}`);
            }
        }
        async setVoiceCount(e) {
            try {
                await r("set_voice_count", {
                    count: e
                });
            } catch (t) {
                throw new Error(`Failed to set voice count: ${t}`);
            }
        }
        async setAutoKey(e) {
            try {
                await r("set_auto_key", {
                    enabled: e
                });
            } catch (t) {
                throw new Error(`Failed to set auto key: ${t}`);
            }
        }
        async setCounterpointSpecies(e) {
            try {
                await r("set_counterpoint_species", {
                    species: e
                });
            } catch (t) {
                throw new Error(`Failed to set counterpoint species: ${t}`);
            }
        }
        async setCounterpointStrictness(e) {
            try {
                await r("set_counterpoint_strictness", {
                    strictness: e
                });
            } catch (t) {
                throw new Error(`Failed to set counterpoint strictness: ${t}`);
            }
        }
        async setPatternEnabled(e) {
            try {
                await r("set_pattern_enabled", {
                    enabled: e
                });
            } catch (t) {
                throw new Error(`Failed to set pattern enabled: ${t}`);
            }
        }
        async setPatternConfig(e) {
            try {
                await r("set_pattern_config", {
                    cells: e.cells,
                    subdivision: e.subdivision,
                    length: e.length,
                    beatsPerBar: e.beatsPerBar,
                    inputMode: e.inputMode
                });
            } catch (t) {
                throw new Error(`Failed to set pattern config: ${t}`);
            }
        }
        midiPermissionState = "granted";
        async requestMidiPermission() {
            return "granted";
        }
        async listMidiInputs() {
            try {
                return (await r("list_midi_inputs")).map((t)=>({
                        index: t.index,
                        name: t.name
                    }));
            } catch (e) {
                throw new Error(`Failed to list MIDI inputs: ${e}`);
            }
        }
        async listMidiOutputs() {
            try {
                return (await r("list_midi_outputs")).map((t)=>({
                        index: t.index,
                        name: t.name
                    }));
            } catch (e) {
                throw new Error(`Failed to list MIDI outputs: ${e}`);
            }
        }
        async refreshMidiDevices() {
            try {
                await r("refresh_midi_devices");
            } catch (e) {
                throw new Error(`Failed to refresh MIDI devices: ${e}`);
            }
        }
        async startRouting(e, t) {
            try {
                if (await r("start_routing", {
                    inputIdx: e,
                    outputIndices: t
                }), this._isRunning = !0, e === 999997) {
                    m.detecting = !0;
                    let s = 0;
                    const a = 100;
                    this._guitarSignalUnsub = await U("guitar-signal", (o)=>{
                        const g = o.payload, p = g.rms, i = g.clarity, b = g.note_name;
                        m.pushSignalFrame(p, i);
                        const d = performance.now();
                        d - s > a && (s = d, b ? (m.currentNote = b, m.confidence = Math.round(i * 100), m.velocity = Math.round(p * 800)) : (m.currentNote = "", m.confidence = 0));
                    });
                }
            } catch (n) {
                throw new Error(`Failed to start routing: ${n}`);
            }
        }
        async stopRouting() {
            try {
                await r("stop_routing"), this._isRunning = !1, this._guitarSignalUnsub && (this._guitarSignalUnsub(), this._guitarSignalUnsub = null), m.detecting = !1, m.currentNote = "", m.confidence = 0, m.velocity = 0;
            } catch (e) {
                throw new Error(`Failed to stop routing: ${e}`);
            }
        }
        async setVoiceOutput(e, t) {
            try {
                await r("set_voice_output", {
                    voiceIdx: e,
                    target: t
                });
            } catch (n) {
                throw new Error(`Failed to set voice output: ${n}`);
            }
        }
        async getVoiceOutputs() {
            try {
                return await r("get_voice_outputs");
            } catch (e) {
                throw new Error(`Failed to get voice outputs: ${e}`);
            }
        }
        async injectNoteOn(e, t) {
            try {
                return await r("inject_note_on", {
                    note: e,
                    velocity: t ?? 100
                });
            } catch  {
                return [
                    e
                ];
            }
        }
        async injectNoteOff(e) {
            try {
                return await r("inject_note_off", {
                    note: e
                });
            } catch  {
                return [
                    e
                ];
            }
        }
        onNoteUpdate(e) {
            let t, n = !1;
            return U("note-update", (s)=>{
                n || e(_e(s.payload));
            }).then((s)=>{
                n ? s() : t = s;
            }), ()=>{
                n = !0, t?.();
            };
        }
        async listPresets() {
            try {
                return (await r("list_presets")).map((t)=>({
                        index: t.index,
                        name: t.name,
                        persona: t.persona,
                        genre: t.genre,
                        isBuiltin: t.is_builtin
                    }));
            } catch (e) {
                throw new Error(`Failed to list presets: ${e}`);
            }
        }
        async loadPreset(e) {
            try {
                await r("load_preset", {
                    name: e
                });
            } catch (t) {
                throw new Error(`Failed to load preset: ${t}`);
            }
        }
        async savePreset(e) {
            try {
                await r("save_preset", {
                    name: e
                });
            } catch (t) {
                throw new Error(`Failed to save preset: ${t}`);
            }
        }
        async deletePreset(e) {
            try {
                await r("delete_preset", {
                    name: e
                });
            } catch (t) {
                throw new Error(`Failed to delete preset: ${t}`);
            }
        }
        async listAudioDevices() {
            try {
                return await r("list_audio_devices");
            } catch (e) {
                throw new Error(`Failed to list audio devices: ${e}`);
            }
        }
        async setGuitarDevice(e, t) {
            try {
                await r("set_guitar_device", {
                    deviceName: e,
                    channel: t
                });
            } catch (n) {
                throw new Error(`Failed to set guitar device: ${n}`);
            }
        }
        async setGuitarConfig(e) {
            try {
                await r("set_guitar_config", {
                    latencyMs: e.latencyMs,
                    gain: e.gain,
                    stringConfidence: e.stringConfidence,
                    bends: e.bends,
                    legato: e.legato,
                    slides: e.slides,
                    vibrato: e.vibrato
                });
            } catch (t) {
                throw new Error(`Failed to set guitar config: ${t}`);
            }
        }
        _detuneCents = 0;
        setDetune(e) {
            this._detuneCents = e, r("set_detune", {
                cents: Math.round(e)
            }).catch(()=>{});
        }
        getDetune() {
            return this._detuneCents;
        }
        async getTransportState() {
            const e = await r("get_transport_state");
            return {
                running: e.running,
                bpm: e.bpm,
                beatsPerBar: e.beats_per_bar,
                beatUnit: e.beat_unit,
                sampleRate: e.sample_rate,
                samplePos: e.sample_pos,
                beatPosition: e.beat_position,
                bar: e.bar,
                metronomeEnabled: e.metronome_enabled
            };
        }
        async transportPlay() {
            await r("transport_play");
        }
        async transportStop() {
            await r("transport_stop");
        }
        async transportReset() {
            await r("transport_reset");
        }
        async setBpm(e) {
            await r("set_bpm", {
                bpm: e
            });
        }
        async setTimeSignature(e, t) {
            await r("set_time_signature", {
                beatsPerBar: e,
                beatUnit: t
            });
        }
        async setMetronomeEnabled(e) {
            await r("set_metronome_enabled", {
                enabled: e
            });
        }
        async getSynthState() {
            const e = await r("get_synth_state");
            return {
                enabled: e.enabled,
                waveform: e.waveform,
                attackMs: e.attack_ms,
                decayMs: e.decay_ms,
                sustain: e.sustain,
                releaseMs: e.release_ms,
                cutoffHz: e.cutoff_hz,
                resonance: e.resonance,
                masterGain: e.master_gain
            };
        }
        async setSynthEnabled(e) {
            await r("set_synth_enabled", {
                enabled: e
            });
        }
        async setSynthWaveform(e) {
            await r("set_synth_waveform", {
                value: e
            });
        }
        async setSynthAttackMs(e) {
            await r("set_synth_attack_ms", {
                ms: e
            });
        }
        async setSynthDecayMs(e) {
            await r("set_synth_decay_ms", {
                ms: e
            });
        }
        async setSynthSustain(e) {
            await r("set_synth_sustain", {
                level: e
            });
        }
        async setSynthReleaseMs(e) {
            await r("set_synth_release_ms", {
                ms: e
            });
        }
        async setSynthCutoffHz(e) {
            await r("set_synth_cutoff_hz", {
                hz: e
            });
        }
        async setSynthResonance(e) {
            await r("set_synth_resonance", {
                value: e
            });
        }
        async setSynthMasterGain(e) {
            await r("set_synth_master_gain", {
                value: e
            });
        }
        async getReverbState() {
            const e = await r("get_reverb_state");
            return {
                enabled: e.enabled,
                mix: e.mix,
                roomSize: e.room_size,
                damping: e.damping
            };
        }
        async setReverbEnabled(e) {
            await r("set_reverb_enabled", {
                enabled: e
            });
        }
        async setReverbMix(e) {
            await r("set_reverb_mix", {
                value: e
            });
        }
        async setReverbRoomSize(e) {
            await r("set_reverb_room_size", {
                value: e
            });
        }
        async setReverbDamping(e) {
            await r("set_reverb_damping", {
                value: e
            });
        }
        async getDelayState() {
            const e = await r("get_delay_state");
            return {
                enabled: e.enabled,
                mix: e.mix,
                timeMs: e.time_ms,
                feedback: e.feedback
            };
        }
        async setDelayEnabled(e) {
            await r("set_delay_enabled", {
                enabled: e
            });
        }
        async setDelayMix(e) {
            await r("set_delay_mix", {
                value: e
            });
        }
        async setDelayTimeMs(e) {
            await r("set_delay_time_ms", {
                ms: e
            });
        }
        async setDelayFeedback(e) {
            await r("set_delay_feedback", {
                value: e
            });
        }
        async listChainBlocks() {
            return (await r("list_chain_blocks")).map((t)=>({
                    typeId: t.type_id,
                    name: t.name
                }));
        }
        async removeChainBlock(e) {
            await r("remove_chain_block", {
                index: e
            });
        }
        async clearChain() {
            await r("clear_chain");
        }
        async listClapPlugins() {
            return (await r("list_clap_plugins")).map((t)=>({
                    id: t.id,
                    name: t.name,
                    vendor: t.vendor ?? "",
                    version: t.version ?? "",
                    path: t.path
                }));
        }
        async addClapPluginToChain(e) {
            const t = await r("add_clap_plugin_to_chain", {
                path: e
            });
            return {
                pluginId: t.plugin_id,
                name: t.name,
                path: t.path,
                hasGui: t.has_gui
            };
        }
        async openPluginGui(e) {
            await r("open_plugin_gui", {
                pluginId: e
            });
        }
        async getPluginGuiSize(e) {
            return await r("get_plugin_gui_size", {
                pluginId: e
            });
        }
        async openPluginGuiEmbedded(e, t, n, s, a) {
            await r("open_plugin_gui_embedded", {
                pluginId: e,
                x: t,
                y: n,
                width: s,
                height: a
            });
        }
        async setPluginGuiFrame(e, t, n, s, a) {
            await r("set_plugin_gui_frame", {
                pluginId: e,
                x: t,
                y: n,
                width: s,
                height: a
            });
        }
        async closePluginGui(e) {
            await r("close_plugin_gui", {
                pluginId: e
            });
        }
        async removePlugin(e) {
            await r("remove_plugin", {
                pluginId: e
            });
        }
    }
    T = 8;
    let C = null, I = null, A = null, P = null, x = null, O = null, F = null, G = null;
    const R = new Map;
    let ve = "triangle", we = .7, Ce = .22, Se = .32, Ee = .32, Me = .22;
    const z = .005, Pe = .05, Ie = .6, K = .25, Ne = .18;
    function ke(h) {
        return 440 * Math.pow(2, (h - 69) / 12);
    }
    function De(h, e = 1.6, t = 2.5) {
        const n = h.sampleRate, s = Math.floor(n * e), a = h.createBuffer(2, s, n);
        for(let o = 0; o < 2; o++){
            const g = a.getChannelData(o);
            for(let p = 0; p < s; p++){
                const i = p / s;
                g[p] = (Math.random() * 2 - 1) * Math.pow(1 - i, t);
            }
        }
        return a;
    }
    function Te() {
        if (C) return C;
        try {
            const h = window.AudioContext ?? window.webkitAudioContext;
            if (!h) return null;
            C = new h;
        } catch  {
            return null;
        }
        return I = C.createGain(), I.gain.value = we, I.connect(C.destination), A = C.createGain(), A.gain.value = 1, A.connect(I), P = C.createDelay(2), P.delayTime.value = Ee, x = C.createGain(), x.gain.value = Se, O = C.createGain(), O.gain.value = Ce, P.connect(x), x.connect(P), P.connect(O), O.connect(I), F = C.createConvolver(), F.buffer = De(C), G = C.createGain(), G.gain.value = Me, F.connect(G), G.connect(I), C;
    }
    async function Ae() {
        if (C && C.state === "suspended") try {
            await C.resume();
        } catch  {}
    }
    function Fe(h, e = 100) {
        const t = Te();
        if (!t || !A || !P || !F) return;
        Ae();
        const n = R.get(h);
        if (n) {
            try {
                n.osc.stop(t.currentTime + .001);
            } catch  {}
            R.delete(h);
        }
        const s = t.createOscillator();
        s.type = ve, s.frequency.value = ke(h);
        const a = t.createGain();
        a.gain.value = 0, s.connect(a), a.connect(A), a.connect(P), a.connect(F);
        const o = t.currentTime, g = Ne * (e / 127);
        a.gain.setValueAtTime(0, o), a.gain.linearRampToValueAtTime(g, o + z), a.gain.linearRampToValueAtTime(g * Ie, o + z + Pe), s.start(o), s.onended = ()=>{
            try {
                a.disconnect(), s.disconnect();
            } catch  {}
        }, R.set(h, {
            osc: s,
            env: a
        });
    }
    function Re(h) {
        if (!C) return;
        const e = R.get(h);
        if (!e) return;
        const t = C.currentTime, n = e.env.gain.value;
        e.env.gain.cancelScheduledValues(t), e.env.gain.setValueAtTime(n, t), e.env.gain.linearRampToValueAtTime(0, t + K);
        try {
            e.osc.stop(t + K + .02);
        } catch  {}
        R.delete(h);
    }
    let D = null, _ = null;
    const xe = 999997;
    class Oe {
        initialized = !1;
        _isRunning = !1;
        noteUpdateCallback = null;
        pollingHandle = null;
        midiAccess = null;
        _midiPermissionState = "idle";
        activeInput = null;
        activeOutputs = [];
        _detuneCents = 0;
        pitchBendRangeSemitones = 2;
        guitarCapture = null;
        _guitarDeviceId = "";
        _guitarChannel = 0;
        async init() {
            if (!this.initialized) try {
                D = await ae(()=>import("./CnTsoIij.js"), [], import.meta.url), D.default && typeof D.default == "function" && await D.default(), _ = new D.Engine, this.initialized = !0;
            } catch (e) {
                throw new Error(`Failed to initialize WASM: ${e}`);
            }
        }
        destroy() {
            if (this.guitarCapture) {
                try {
                    this.guitarCapture.stop();
                } catch  {}
                this.guitarCapture = null;
            }
        }
        ensureInit() {
            if (!this.initialized || !_) throw new Error("WASM adapter not initialized. Call init() first.");
        }
        async getEngineState() {
            this.ensureInit();
            try {
                const e = _.get_state();
                return {
                    key: e.key ?? "C",
                    mode: e.mode ?? "PassThrough",
                    modeNumber: e.mode_number ?? 1,
                    scaleMode: e.scale_mode ?? "Ionian",
                    octaveMode: e.octave_mode ?? "None",
                    voiceLeadingEnabled: e.voice_leading_enabled ?? !1,
                    voiceLeadingStyle: e.voice_leading_style ?? "Free",
                    interchangeEnabled: e.interchange_enabled ?? !1,
                    interchangeRange: e.borrowing_range ?? 3,
                    voicePosition: e.voice_position ?? 0,
                    voiceCount: e.voice_count ?? 2,
                    autoKey: e.auto_key ?? !1,
                    isRunning: this._isRunning,
                    counterpointSpecies: e.counterpoint_species ?? "Species1",
                    counterpointStrictness: e.counterpoint_strictness ?? "Strict"
                };
            } catch (e) {
                throw new Error(`Failed to get engine state: ${e}`);
            }
        }
        async setKey(e) {
            this.ensureInit();
            try {
                _.set_key(e);
            } catch (t) {
                throw new Error(`Failed to set key: ${t}`);
            }
        }
        async setMode(e) {
            this.ensureInit();
            try {
                _.set_mode(e);
            } catch (t) {
                throw new Error(`Failed to set mode: ${t}`);
            }
        }
        async setScaleMode(e) {
            this.ensureInit();
            try {
                _.set_scale_mode(e);
            } catch (t) {
                throw new Error(`Failed to set scale mode: ${t}`);
            }
        }
        async setOctaveMode(e) {
            this.ensureInit();
            try {
                _.set_octave_mode(e);
            } catch (t) {
                throw new Error(`Failed to set octave mode: ${t}`);
            }
        }
        async setOctaveIntensity(e) {
            this.ensureInit();
            try {
                _.set_octave_intensity(e);
            } catch (t) {
                throw new Error(`Failed to set octave intensity: ${t}`);
            }
        }
        async setVoiceLeading(e, t) {
            this.ensureInit();
            try {
                _.set_voice_leading(e, t);
            } catch (n) {
                throw new Error(`Failed to set voice leading: ${n}`);
            }
        }
        async setInterchange(e, t) {
            this.ensureInit();
            try {
                _.set_interchange(e, t);
            } catch (n) {
                throw new Error(`Failed to set interchange: ${n}`);
            }
        }
        async setVoicePosition(e) {
            this.ensureInit();
            try {
                _.set_voice_position(e);
            } catch (t) {
                throw new Error(`Failed to set voice position: ${t}`);
            }
        }
        async setVoiceCount(e) {
            this.ensureInit();
            try {
                _.set_voice_count(e);
            } catch (t) {
                throw new Error(`Failed to set voice count: ${t}`);
            }
        }
        async setAutoKey(e) {
            this.ensureInit();
            try {
                _.set_auto_key(e);
            } catch (t) {
                throw new Error(`Failed to set auto key: ${t}`);
            }
        }
        async setCounterpointSpecies(e) {
            this.ensureInit();
            try {
                _.set_counterpoint_species(e);
            } catch (t) {
                throw new Error(`Failed to set counterpoint species: ${t}`);
            }
        }
        async setCounterpointStrictness(e) {
            this.ensureInit();
            try {
                _.set_counterpoint_strictness(e);
            } catch (t) {
                throw new Error(`Failed to set counterpoint strictness: ${t}`);
            }
        }
        async setPatternEnabled(e) {}
        async setPatternConfig(e) {}
        get midiPermissionState() {
            return this._midiPermissionState;
        }
        ensureMidiAccess() {
            return this.midiAccess;
        }
        async requestMidiPermission() {
            if (typeof navigator > "u" || !("requestMIDIAccess" in navigator)) return this._midiPermissionState = "unsupported", this._midiPermissionState;
            if (this.midiAccess) return this._midiPermissionState = "granted", this._midiPermissionState;
            try {
                this.midiAccess = await navigator.requestMIDIAccess(), this._midiPermissionState = "granted";
            } catch  {
                this._midiPermissionState = "denied";
            }
            return this._midiPermissionState;
        }
        async listMidiInputs() {
            const e = this.ensureMidiAccess();
            if (!e) return [];
            const t = [];
            let n = 0;
            return e.inputs.forEach((s)=>{
                t.push({
                    index: n++,
                    name: s.name ?? `Input ${n}`
                });
            }), t;
        }
        async listMidiOutputs() {
            const e = this.ensureMidiAccess();
            if (!e) return [];
            const t = [];
            let n = 0;
            return e.outputs.forEach((s)=>{
                t.push({
                    index: n++,
                    name: s.name ?? `Output ${n}`
                });
            }), t;
        }
        async refreshMidiDevices() {
            if (this._midiPermissionState === "granted") try {
                this.midiAccess = await navigator.requestMIDIAccess();
            } catch  {
                this.midiAccess = null, this._midiPermissionState = "denied";
            }
        }
        async startRouting(e, t) {
            if (this.ensureInit(), e === xe) {
                await this.startGuitarCapture(t);
                return;
            }
            const n = this.ensureMidiAccess();
            if (!n) {
                this._isRunning = !0;
                return;
            }
            const s = Array.from(n.inputs.values());
            e >= 0 && e < s.length && (this.activeInput = s[e]);
            const a = Array.from(n.outputs.values());
            if (this.activeOutputs = t.filter((o)=>o >= 0 && o < a.length).map((o)=>a[o]), this.activeInput) {
                const o = this, g = this.activeOutputs;
                this.activeInput.onmidimessage = (p)=>{
                    if (!p.data || p.data.length < 2) return;
                    const i = p.data[0] & 240, b = p.data[1], d = p.data.length > 2 ? p.data[2] : 0;
                    let f = [];
                    if (i === 144 && d > 0) try {
                        f = _.note_on(b);
                        const y = o.sortVoices(f);
                        for(let w = 0; w < y.length; w++)g.length > 0 && g[w % g.length].send([
                            144,
                            y[w],
                            d
                        ]);
                    } catch  {}
                    else if (i === 128 || i === 144 && d === 0) try {
                        f = _.note_off(b);
                        const y = o.sortVoices(f);
                        for(let w = 0; w < y.length; w++)g.length > 0 && g[w % g.length].send([
                            128,
                            y[w],
                            0
                        ]);
                    } catch  {}
                    else for (const y of g)y.send(Array.from(p.data));
                };
            }
            this._isRunning = !0, this._detuneCents !== 0 && this.sendPitchBend();
        }
        async startGuitarCapture(e) {
            const t = this.ensureMidiAccess();
            if (t) {
                const o = Array.from(t.outputs.values());
                this.activeOutputs = e.filter((g)=>g >= 0 && g < o.length).map((g)=>o[g]);
            }
            this.guitarCapture = new ie;
            const n = this;
            m.detecting = !0, this.guitarCapture.noiseGateThreshold = m.noiseGateThreshold, this.guitarCapture.noiseGateEnabled = m.noiseGateEnabled, this.guitarCapture.clarityGateEnabled = !1;
            const s = m.selectedDeviceId || this._guitarDeviceId, a = Math.max(0, m.selectedChannel - 1);
            console.log(`[wasm] startGuitarCapture: device='${s}' channel=${a} (store.selectedChannel=${m.selectedChannel})`), await this.guitarCapture.start(s, a, {
                onNoteOn (o, g) {
                    n.injectNoteOn(o, g).catch(()=>{});
                },
                onNoteOff (o) {
                    n.injectNoteOff(o).catch(()=>{});
                },
                onDetection (o) {
                    n.guitarCapture && (n.guitarCapture.noiseGateThreshold = m.noiseGateThreshold, n.guitarCapture.noiseGateEnabled = m.noiseGateEnabled, m.activeChannel = n.guitarCapture.actualChannel + 1), m.pushSignalFrame(o.rms, o.clarity), o.frequency !== null ? (m.currentNote = o.noteName, m.confidence = Math.round(o.clarity * 100), m.velocity = Math.round(o.rms * 800)) : (m.currentNote = "", m.confidence = 0);
                }
            }), m.activeChannel = this.guitarCapture.actualChannel + 1, this._isRunning = !0;
        }
        async stopRouting() {
            this.guitarCapture && (await this.guitarCapture.stop(), this.guitarCapture = null, m.detecting = !1, m.currentNote = "", m.confidence = 0, m.velocity = 0);
            for (const e of this.activeOutputs)try {
                e.send([
                    176,
                    123,
                    0
                ]);
            } catch  {}
            if (_) try {
                _.clear_notes();
            } catch  {}
            this.activeInput && (this.activeInput.onmidimessage = null, this.activeInput = null), this.activeOutputs = [], this._isRunning = !1, this.stopNotePolling();
        }
        _voiceOutputs = Array.from({
            length: T
        }, ()=>({
                kind: "synth"
            }));
        async setVoiceOutput(e, t) {
            if (e < 0 || e >= T) throw new Error(`voiceIdx ${e} out of range (0..${T - 1})`);
            this._voiceOutputs[e] = t;
        }
        async getVoiceOutputs() {
            return this._voiceOutputs.slice();
        }
        async injectNoteOn(e, t) {
            this.ensureInit();
            try {
                const n = _.note_on(e), s = this.sortVoices(n ?? [
                    e
                ]), a = t ?? 100;
                for(let o = 0; o < s.length; o++)this.activeOutputs.length > 0 && this.activeOutputs[o % this.activeOutputs.length].send([
                    144,
                    s[o],
                    a
                ]), Fe(s[o], a);
                return s;
            } catch  {
                return [
                    e
                ];
            }
        }
        async injectNoteOff(e) {
            this.ensureInit();
            try {
                const t = _.note_off(e), n = this.sortVoices(t ?? [
                    e
                ]);
                for(let s = 0; s < n.length; s++)this.activeOutputs.length > 0 && this.activeOutputs[s % this.activeOutputs.length].send([
                    128,
                    n[s],
                    0
                ]), Re(n[s]);
                return n;
            } catch  {
                return [
                    e
                ];
            }
        }
        async getNoteState() {
            this.ensureInit();
            try {
                const e = _.get_note_state();
                return {
                    inputNotes: e?.input_notes ?? [],
                    harmonyNotes: e?.harmony_notes ?? [],
                    borrowedNotes: e?.borrowed_notes ?? [],
                    chordName: e?.chord_name ?? "",
                    lastBorrowedFrom: e?.last_borrowed_from ?? "",
                    currentKey: _.current_key?.() ?? "C"
                };
            } catch  {
                return {
                    inputNotes: [],
                    harmonyNotes: [],
                    borrowedNotes: [],
                    chordName: "",
                    lastBorrowedFrom: "",
                    currentKey: "C"
                };
            }
        }
        onNoteUpdate(e) {
            return this.noteUpdateCallback = e, this.startNotePolling(), ()=>{
                this.noteUpdateCallback = null, this.stopNotePolling();
            };
        }
        startNotePolling() {
            if (this.pollingHandle !== null) return;
            const e = ()=>{
                if (!this.noteUpdateCallback || !this._isRunning) {
                    this.pollingHandle = null;
                    return;
                }
                this.getNoteState().then((t)=>{
                    this.noteUpdateCallback?.(t);
                }).catch(()=>{}), this.pollingHandle = requestAnimationFrame(e);
            };
            this.pollingHandle = requestAnimationFrame(e);
        }
        sortVoices(e) {
            return [
                ...e
            ].sort((t, n)=>t - n);
        }
        centsToPitchBend(e) {
            const t = this.pitchBendRangeSemitones * 100, n = Math.max(-1, Math.min(1, e / t));
            return Math.round(8192 + n * 8191);
        }
        sendPitchBend() {
            const e = this.centsToPitchBend(this._detuneCents), t = e & 127, n = e >> 7 & 127;
            for (const s of this.activeOutputs)s.send([
                224,
                t,
                n
            ]);
        }
        async listAudioDevices() {
            if (typeof navigator > "u" || !navigator.mediaDevices) return [];
            try {
                return (await navigator.mediaDevices.getUserMedia({
                    audio: !0
                })).getTracks().forEach((n)=>n.stop()), (await navigator.mediaDevices.enumerateDevices()).filter((n)=>n.kind === "audioinput").map((n)=>n.label || `Audio Input ${n.deviceId.slice(0, 8)}`);
            } catch  {
                return [];
            }
        }
        async setGuitarDevice(e, t) {
            this._guitarDeviceId = e, this._guitarChannel = Math.max(0, t);
        }
        async setGuitarConfig(e) {}
        setDetune(e) {
            this._detuneCents = e, this._isRunning && this.sendPitchBend();
        }
        getDetune() {
            return this._detuneCents;
        }
        _transport = {
            running: !1,
            bpm: 120,
            beatsPerBar: 4,
            beatUnit: 4,
            beatInBar: 0,
            totalBeat: 0,
            bar: 0,
            metronomeEnabled: !1
        };
        _audioCtx = null;
        _clockTimer = null;
        ensureAudioContext() {
            if (this._audioCtx) return this._audioCtx;
            try {
                const e = window.AudioContext ?? window.webkitAudioContext;
                return e ? (this._audioCtx = new e, this._audioCtx) : null;
            } catch  {
                return null;
            }
        }
        playClick(e) {
            const t = this.ensureAudioContext();
            if (!t) return;
            const n = e ? 900 : 600, s = .015, a = t.currentTime, o = t.createOscillator(), g = t.createGain();
            o.type = "sine", o.frequency.value = n, g.gain.setValueAtTime(0, a), g.gain.linearRampToValueAtTime(.3, a + .001), g.gain.exponentialRampToValueAtTime(1e-4, a + s), o.connect(g).connect(t.destination), o.start(a), o.stop(a + s + .005);
        }
        tick() {
            const e = this._transport;
            B.applyBeatUpdate({
                totalBeat: e.totalBeat,
                beatInBar: e.beatInBar,
                bar: e.bar,
                bpm: e.bpm,
                running: e.running
            }), e.metronomeEnabled && this.playClick(e.beatInBar === 0), e.totalBeat += 1, e.beatInBar += 1, e.beatInBar >= e.beatsPerBar && (e.beatInBar = 0, e.bar += 1);
        }
        startClock() {
            this.stopClock();
            const e = 6e4 / this._transport.bpm;
            this._clockTimer = setInterval(()=>this.tick(), e), this.tick();
        }
        stopClock() {
            this._clockTimer !== null && (clearInterval(this._clockTimer), this._clockTimer = null);
        }
        async getTransportState() {
            return {
                running: this._transport.running,
                bpm: this._transport.bpm,
                beatsPerBar: this._transport.beatsPerBar,
                beatUnit: this._transport.beatUnit,
                sampleRate: 48e3,
                samplePos: 0,
                beatPosition: 0,
                bar: this._transport.bar,
                metronomeEnabled: this._transport.metronomeEnabled
            };
        }
        async transportPlay() {
            if (this._transport.running) return;
            this.ensureAudioContext();
            const e = this._audioCtx;
            if (e && e.state === "suspended") try {
                await e.resume();
            } catch  {}
            this._transport.running = !0, this._transport.beatInBar = 0, this._transport.totalBeat = 0, this._transport.bar = 0, this.startClock();
        }
        async transportStop() {
            this._transport.running = !1, this.stopClock(), B.applyBeatUpdate({
                totalBeat: this._transport.totalBeat,
                beatInBar: this._transport.beatInBar,
                bar: this._transport.bar,
                bpm: this._transport.bpm,
                running: !1
            });
        }
        async transportReset() {
            this._transport.beatInBar = 0, this._transport.totalBeat = 0, this._transport.bar = 0, B.applyBeatUpdate({
                totalBeat: 0,
                beatInBar: 0,
                bar: 0,
                bpm: this._transport.bpm,
                running: this._transport.running
            });
        }
        async setBpm(e) {
            const t = Math.max(20, Math.min(400, e));
            this._transport.bpm = t, this._transport.running && this.startClock();
        }
        async setTimeSignature(e, t) {
            this._transport.beatsPerBar = Math.max(1, Math.min(16, e)), this._transport.beatUnit = t, this._transport.beatInBar = 0;
        }
        async setMetronomeEnabled(e) {
            this._transport.metronomeEnabled = e, e && this.ensureAudioContext();
        }
        async getSynthState() {
            return {
                enabled: !1,
                waveform: 0,
                attackMs: 5,
                decayMs: 120,
                sustain: .7,
                releaseMs: 250,
                cutoffHz: 6e3,
                resonance: .2,
                masterGain: .25
            };
        }
        async setSynthEnabled(e) {}
        async setSynthWaveform(e) {}
        async setSynthAttackMs(e) {}
        async setSynthDecayMs(e) {}
        async setSynthSustain(e) {}
        async setSynthReleaseMs(e) {}
        async setSynthCutoffHz(e) {}
        async setSynthResonance(e) {}
        async setSynthMasterGain(e) {}
        async getReverbState() {
            return {
                enabled: !1,
                mix: .3,
                roomSize: .7,
                damping: .5
            };
        }
        async setReverbEnabled(e) {}
        async setReverbMix(e) {}
        async setReverbRoomSize(e) {}
        async setReverbDamping(e) {}
        async getDelayState() {
            return {
                enabled: !1,
                mix: .3,
                timeMs: 375,
                feedback: .35
            };
        }
        async setDelayEnabled(e) {}
        async setDelayMix(e) {}
        async setDelayTimeMs(e) {}
        async setDelayFeedback(e) {}
        async listChainBlocks() {
            return [];
        }
        async removeChainBlock(e) {}
        async clearChain() {}
        async listClapPlugins() {
            return [];
        }
        async addClapPluginToChain(e) {
            return {
                pluginId: 0,
                name: "",
                path: "",
                hasGui: !1
            };
        }
        async openPluginGui(e) {}
        async getPluginGuiSize(e) {
            return null;
        }
        async openPluginGuiEmbedded(e, t, n, s, a) {}
        async setPluginGuiFrame(e, t, n, s, a) {}
        async closePluginGui(e) {}
        async removePlugin(e) {}
        stopNotePolling() {
            this.pollingHandle !== null && (cancelAnimationFrame(this.pollingHandle), this.pollingHandle = null);
        }
        async listPresets() {
            this.ensureInit();
            try {
                return (_.list_presets() ?? []).map((t, n)=>({
                        index: n,
                        name: t.name ?? "",
                        persona: t.persona ?? "",
                        genre: t.genre ?? "",
                        isBuiltin: t.is_builtin ?? !0
                    }));
            } catch  {
                return [];
            }
        }
        async loadPreset(e) {
            this.ensureInit();
            try {
                _.load_preset(e);
            } catch (t) {
                throw new Error(`Failed to load preset: ${t}`);
            }
        }
        async savePreset(e) {
            this.ensureInit();
            try {
                _.save_preset(e);
            } catch (t) {
                throw new Error(`Failed to save preset: ${t}`);
            }
        }
        async deletePreset(e) {
            this.ensureInit();
            try {
                _.delete_preset(e);
            } catch (t) {
                throw new Error(`Failed to delete preset: ${t}`);
            }
        }
    }
    let S = {};
    class Ge {
        noteUpdateCallback = null;
        _detuneCents = 0;
        _isReady = !1;
        async init() {
            window.plugin.listen((e)=>{
                try {
                    const t = JSON.parse(e);
                    t.type === "paramsUpdate" ? S = t : t.type === "noteUpdate" && this.noteUpdateCallback && this.noteUpdateCallback({
                        inputNotes: t.inputNotes ?? [],
                        harmonyNotes: t.harmonyNotes ?? [],
                        borrowedNotes: t.borrowedNotes ?? [],
                        chordName: t.chordName ?? "",
                        lastBorrowedFrom: t.lastBorrowedFrom ?? "",
                        currentKey: t.currentKey ?? "C"
                    });
                } catch  {}
            }), window.plugin.send(JSON.stringify({
                type: "ready"
            })), this._isReady = !0;
        }
        send(e, t) {
            window.plugin.send(JSON.stringify({
                type: e,
                value: t
            }));
        }
        async getEngineState() {
            return {
                key: S.key ?? "C",
                mode: S.mode ?? "DiatonicThirds",
                modeNumber: 0,
                scaleMode: "Ionian",
                octaveMode: S.octaveMode ?? "None",
                voiceLeadingEnabled: S.voiceLeading ?? !1,
                voiceLeadingStyle: "Free",
                interchangeEnabled: !1,
                interchangeRange: 3,
                voicePosition: S.voicePosition ?? 0,
                voiceCount: S.voiceCount ?? 2,
                autoKey: S.autoKey ?? !1,
                isRunning: !0,
                counterpointSpecies: S.counterpointSpecies ?? "Species1",
                counterpointStrictness: S.counterpointStrictness ?? "Strict"
            };
        }
        async setKey(e) {
            this.send("setKey", e);
        }
        async setMode(e) {
            this.send("setMode", e);
        }
        async setScaleMode(e) {}
        async setOctaveMode(e) {
            this.send("setOctaveMode", e);
        }
        async setOctaveIntensity(e) {
            this.send("setOctaveIntensity", e);
        }
        async setVoiceLeading(e, t) {
            this.send("setVoiceLeading", e);
        }
        async setInterchange(e, t) {}
        async setVoicePosition(e) {
            this.send("setVoicePosition", e);
        }
        async setVoiceCount(e) {
            this.send("setVoiceCount", e);
        }
        async setAutoKey(e) {
            this.send("setAutoKey", e);
        }
        async setCounterpointSpecies(e) {
            this.send("setCounterpointSpecies", e);
        }
        async setCounterpointStrictness(e) {
            this.send("setCounterpointStrictness", e);
        }
        async setPatternEnabled(e) {}
        async setPatternConfig(e) {}
        midiPermissionState = "granted";
        async requestMidiPermission() {
            return "granted";
        }
        async listMidiInputs() {
            return [];
        }
        async listMidiOutputs() {
            return [];
        }
        async refreshMidiDevices() {}
        async startRouting(e, t) {}
        async stopRouting() {}
        _voiceOutputs = Array.from({
            length: T
        }, ()=>({
                kind: "synth"
            }));
        async setVoiceOutput(e, t) {
            e < 0 || e >= T || (this._voiceOutputs[e] = t);
        }
        async getVoiceOutputs() {
            return this._voiceOutputs.slice();
        }
        onNoteUpdate(e) {
            return this.noteUpdateCallback = e, ()=>{
                this.noteUpdateCallback = null;
            };
        }
        async injectNoteOn(e, t) {
            return [
                e
            ];
        }
        async injectNoteOff(e) {
            return [
                e
            ];
        }
        async listPresets() {
            return [];
        }
        async loadPreset(e) {}
        async savePreset(e) {}
        async deletePreset(e) {}
        async listAudioDevices() {
            return [];
        }
        async setGuitarDevice(e, t) {}
        async setGuitarConfig(e) {}
        setDetune(e) {
            this._detuneCents = e;
        }
        getDetune() {
            return this._detuneCents;
        }
        async getTransportState() {
            return {
                running: !1,
                bpm: 120,
                beatsPerBar: 4,
                beatUnit: 4,
                sampleRate: 48e3,
                samplePos: 0,
                beatPosition: 0,
                bar: 0,
                metronomeEnabled: !1
            };
        }
        async transportPlay() {}
        async transportStop() {}
        async transportReset() {}
        async setBpm(e) {}
        async setTimeSignature(e, t) {}
        async setMetronomeEnabled(e) {}
        async getSynthState() {
            return {
                enabled: !1,
                waveform: 0,
                attackMs: 5,
                decayMs: 120,
                sustain: .7,
                releaseMs: 250,
                cutoffHz: 6e3,
                resonance: .2,
                masterGain: .25
            };
        }
        async setSynthEnabled(e) {}
        async setSynthWaveform(e) {}
        async setSynthAttackMs(e) {}
        async setSynthDecayMs(e) {}
        async setSynthSustain(e) {}
        async setSynthReleaseMs(e) {}
        async setSynthCutoffHz(e) {}
        async setSynthResonance(e) {}
        async setSynthMasterGain(e) {}
        async getReverbState() {
            return {
                enabled: !1,
                mix: .3,
                roomSize: .7,
                damping: .5
            };
        }
        async setReverbEnabled(e) {}
        async setReverbMix(e) {}
        async setReverbRoomSize(e) {}
        async setReverbDamping(e) {}
        async getDelayState() {
            return {
                enabled: !1,
                mix: .3,
                timeMs: 375,
                feedback: .35
            };
        }
        async setDelayEnabled(e) {}
        async setDelayMix(e) {}
        async setDelayTimeMs(e) {}
        async setDelayFeedback(e) {}
        async listChainBlocks() {
            return [];
        }
        async removeChainBlock(e) {}
        async clearChain() {}
        async listClapPlugins() {
            return [];
        }
        async addClapPluginToChain(e) {
            return {
                pluginId: 0,
                name: "",
                path: "",
                hasGui: !1
            };
        }
        async openPluginGui(e) {}
        async getPluginGuiSize(e) {
            return null;
        }
        async openPluginGuiEmbedded(e, t, n, s, a) {}
        async setPluginGuiFrame(e, t, n, s, a) {}
        async closePluginGui(e) {}
        async removePlugin(e) {}
    }
    function ce() {
        return typeof window < "u" && "plugin" in window && typeof window.plugin?.send == "function";
    }
    function le() {
        return typeof window < "u" && ("__TAURI_INTERNALS__" in window || "__TAURI__" in window);
    }
    let j, J, Y, Z, X, Q, ee, te, ne, se;
    Be = ce() ? "plugin" : le() ? "tauri" : "browser";
    v = ce() ? new Ge : le() ? new be : new Oe;
    j = "contrapunk-ui-scale";
    J = "contrapunk-font-scale";
    Y = "contrapunk-show-note-labels";
    Z = "contrapunk-panels";
    X = "contrapunk-view-mode";
    Q = .75;
    ee = 2;
    te = .75;
    ne = 1.5;
    $e = [
        {
            id: "midi",
            label: "MIDI"
        },
        {
            id: "controls",
            label: "Controls"
        },
        {
            id: "activeNotes",
            label: "Notes"
        },
        {
            id: "history",
            label: "History"
        },
        {
            id: "fretboard",
            label: "Fret"
        },
        {
            id: "piano",
            label: "Piano"
        },
        {
            id: "pattern",
            label: "Pattern"
        }
    ];
    se = {
        midi: !0,
        controls: !0,
        activeNotes: !0,
        history: !0,
        fretboard: !0,
        piano: !0,
        pattern: !1
    };
    class Ue {
        #e = c(!1);
        get reducedMotion() {
            return l(this.#e);
        }
        set reducedMotion(e) {
            u(this.#e, e, !0);
        }
        #t = c(!0);
        get animationsEnabled() {
            return l(this.#t);
        }
        set animationsEnabled(e) {
            u(this.#t, e, !0);
        }
        #n = c(N(Be));
        get platform() {
            return l(this.#n);
        }
        set platform(e) {
            u(this.#n, e, !0);
        }
        #s = c(!1);
        get initialized() {
            return l(this.#s);
        }
        set initialized(e) {
            u(this.#s, e, !0);
        }
        #a = c(null);
        get error() {
            return l(this.#a);
        }
        set error(e) {
            u(this.#a, e, !0);
        }
        #i = c(!1);
        get sidebarCollapsed() {
            return l(this.#i);
        }
        set sidebarCollapsed(e) {
            u(this.#i, e, !0);
        }
        #r = c("play");
        get activePanel() {
            return l(this.#r);
        }
        set activePanel(e) {
            u(this.#r, e, !0);
        }
        #o = c("play");
        get activeTab() {
            return l(this.#o);
        }
        set activeTab(e) {
            u(this.#o, e, !0);
        }
        #c = c(!1);
        get settingsOpen() {
            return l(this.#c);
        }
        set settingsOpen(e) {
            u(this.#c, e, !0);
        }
        #l = c(1);
        get uiScale() {
            return l(this.#l);
        }
        set uiScale(e) {
            u(this.#l, e, !0);
        }
        #u = c(1);
        get fontScale() {
            return l(this.#u);
        }
        set fontScale(e) {
            u(this.#u, e, !0);
        }
        #h = c(!0);
        get showNoteLabels() {
            return l(this.#h);
        }
        set showNoteLabels(e) {
            u(this.#h, e, !0);
        }
        #d = c(N({
            ...se
        }));
        get panels() {
            return l(this.#d);
        }
        set panels(e) {
            u(this.#d, e, !0);
        }
        #g = c("advanced");
        get viewMode() {
            return l(this.#g);
        }
        set viewMode(e) {
            u(this.#g, e, !0);
        }
        toggleAnimations() {
            this.animationsEnabled = !this.animationsEnabled, this.reducedMotion = !this.animationsEnabled, this.applyMotionPreference();
        }
        applyMotionPreference() {
            typeof document > "u" || (this.reducedMotion ? document.body.classList.add("reduced-motion") : document.body.classList.remove("reduced-motion"));
        }
        detectSystemMotionPreference() {
            if (typeof window > "u") return;
            const e = window.matchMedia("(prefers-reduced-motion: reduce)");
            e.matches && (this.reducedMotion = !0, this.animationsEnabled = !1, this.applyMotionPreference()), e.addEventListener("change", (t)=>{
                this.reducedMotion = t.matches, this.animationsEnabled = !t.matches, this.applyMotionPreference();
            });
        }
        markInitialized() {
            this.initialized = !0, this.error = null;
        }
        setError(e) {
            this.error = e;
        }
        clearError() {
            this.error = null;
        }
        toggleSidebar() {
            this.sidebarCollapsed = !this.sidebarCollapsed;
        }
        setActivePanel(e) {
            this.activePanel = e;
        }
        openSettings() {
            this.settingsOpen = !0;
        }
        closeSettings() {
            this.settingsOpen = !1;
        }
        toggleSettings() {
            this.settingsOpen = !this.settingsOpen;
        }
        setUiScale(e) {
            const t = Math.max(Q, Math.min(ee, e));
            this.uiScale = t, this.applyUiScale();
            try {
                localStorage.setItem(j, String(t));
            } catch  {}
        }
        applyUiScale() {
            typeof document > "u" || document.documentElement.style.setProperty("--ui-scale", String(this.uiScale));
        }
        setFontScale(e) {
            const t = Math.max(te, Math.min(ne, e));
            this.fontScale = t, this.applyFontScale();
            try {
                localStorage.setItem(J, String(t));
            } catch  {}
        }
        applyFontScale() {
            typeof document > "u" || document.documentElement.style.setProperty("--font-scale", String(this.fontScale));
        }
        setShowNoteLabels(e) {
            this.showNoteLabels = e;
            try {
                localStorage.setItem(Y, e ? "on" : "off");
            } catch  {}
        }
        togglePanel(e) {
            this.panels = {
                ...this.panels,
                [e]: !this.panels[e]
            }, this.persistPanels();
        }
        setPanels(e) {
            this.panels = {
                ...this.panels,
                ...e
            }, this.persistPanels();
        }
        persistPanels() {
            try {
                localStorage.setItem(Z, JSON.stringify(this.panels));
            } catch  {}
        }
        setViewMode(e) {
            this.viewMode = e;
            try {
                localStorage.setItem(X, e);
            } catch  {}
        }
        toggleViewMode() {
            this.setViewMode(this.viewMode === "performance" ? "advanced" : "performance");
        }
        restoreViewMode() {
            if (!(typeof window > "u")) try {
                const e = localStorage.getItem(X);
                (e === "performance" || e === "advanced") && (this.viewMode = e);
            } catch  {}
        }
        restorePanels() {
            if (!(typeof window > "u")) try {
                const e = localStorage.getItem(Z);
                if (!e) return;
                const t = JSON.parse(e);
                if (!t || typeof t != "object") return;
                const n = {
                    ...se
                };
                for (const { id: s } of $e)typeof t[s] == "boolean" && (n[s] = t[s]);
                this.panels = n;
            } catch  {}
        }
        restoreAppearance() {
            try {
                const e = localStorage.getItem(j);
                if (e) {
                    const s = parseFloat(e);
                    Number.isFinite(s) && (this.uiScale = Math.max(Q, Math.min(ee, s)));
                }
                const t = localStorage.getItem(J);
                if (t) {
                    const s = parseFloat(t);
                    Number.isFinite(s) && (this.fontScale = Math.max(te, Math.min(ne, s)));
                }
                if (localStorage.getItem(Y) === "off" && (this.showNoteLabels = !1), typeof document < "u") {
                    const s = Array.from(document.body.classList);
                    for (const a of s)a.startsWith("font-") && document.body.classList.remove(a);
                }
                localStorage.removeItem("contrapunk-ui-font");
            } catch  {}
            this.applyUiScale(), this.applyFontScale();
        }
    }
    Ke = new Ue;
    Le = function(h) {
        if (!h.startsWith("clap:")) return null;
        const e = parseInt(h.slice(5), 10);
        return Number.isFinite(e) ? e : null;
    };
    class qe {
        #e = c(N([]));
        get blocks() {
            return l(this.#e);
        }
        set blocks(e) {
            u(this.#e, e, !0);
        }
        #t = c(N([]));
        get clapPlugins() {
            return l(this.#t);
        }
        set clapPlugins(e) {
            u(this.#t, e, !0);
        }
        #n = c(!1);
        get loadingPlugins() {
            return l(this.#n);
        }
        set loadingPlugins(e) {
            u(this.#n, e, !0);
        }
        async refresh() {
            try {
                const e = await v.listChainBlocks();
                console.log("[chainStore] refresh got", e.length, "blocks", e), this.blocks = e, e.length === 0 && setTimeout(async ()=>{
                    try {
                        const t = await v.listChainBlocks();
                        t.length > 0 && (console.log("[chainStore] retry got", t.length, "blocks"), this.blocks = t);
                    } catch  {}
                }, 500);
            } catch (e) {
                console.warn("[chainStore] refresh failed:", e), this.blocks = [];
            }
        }
        async removeAt(e) {
            const t = this.blocks[e], n = t ? Le(t.typeId) : null;
            try {
                if (await v.removeChainBlock(e), n !== null) try {
                    await v.removePlugin(n);
                } catch  {}
            } finally{
                await this.refresh();
            }
        }
        async openPluginGui(e) {
            await v.openPluginGui(e);
        }
        async openPluginGuiEmbedded(e, t, n, s, a) {
            await v.openPluginGuiEmbedded(e, t, n, s, a);
        }
        async setPluginGuiFrame(e, t, n, s, a) {
            await v.setPluginGuiFrame(e, t, n, s, a);
        }
        async closePluginGui(e) {
            await v.closePluginGui(e);
        }
        async scanPlugins() {
            this.loadingPlugins = !0;
            try {
                this.clapPlugins = await v.listClapPlugins();
            } catch  {
                this.clapPlugins = [];
            } finally{
                this.loadingPlugins = !1;
            }
        }
        async addClapPlugin(e) {
            await v.addClapPluginToChain(e), await this.refresh();
        }
    }
    je = new qe;
})();
export { T as M, $e as P, v as a, Le as b, je as c, m as g, Be as p, B as t, Ke as u, __tla };
