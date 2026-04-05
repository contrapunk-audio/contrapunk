import { K as a, g as r, M as o, L as R } from "./nVkCXwlp.js";
import { _ as k } from "./CYEXfWnH.js";
let F, g, ae;
let __tla = (async ()=>{
    function W(i, e = !1) {
        return window.__TAURI_INTERNALS__.transformCallback(i, e);
    }
    async function p(i, e = {}, t) {
        return window.__TAURI_INTERNALS__.invoke(i, e, t);
    }
    var x;
    (function(i) {
        i.WINDOW_RESIZED = "tauri://resize", i.WINDOW_MOVED = "tauri://move", i.WINDOW_CLOSE_REQUESTED = "tauri://close-requested", i.WINDOW_DESTROYED = "tauri://destroyed", i.WINDOW_FOCUS = "tauri://focus", i.WINDOW_BLUR = "tauri://blur", i.WINDOW_SCALE_FACTOR_CHANGED = "tauri://scale-change", i.WINDOW_THEME_CHANGED = "tauri://theme-changed", i.WINDOW_CREATED = "tauri://window-created", i.WEBVIEW_CREATED = "tauri://webview-created", i.DRAG_ENTER = "tauri://drag-enter", i.DRAG_OVER = "tauri://drag-over", i.DRAG_DROP = "tauri://drag-drop", i.DRAG_LEAVE = "tauri://drag-leave";
    })(x || (x = {}));
    async function B(i, e) {
        window.__TAURI_EVENT_PLUGIN_INTERNALS__.unregisterListener(i, e), await p("plugin:event|unlisten", {
            event: i,
            eventId: e
        });
    }
    async function G(i, e, t) {
        var n;
        const s = (n = void 0) !== null && n !== void 0 ? n : {
            kind: "Any"
        };
        return p("plugin:event|listen", {
            event: i,
            target: s,
            handler: W(e)
        }).then((v)=>async ()=>B(i, v));
    }
    let P = null;
    async function V() {
        return P || (P = (await k(()=>import("./CMqHBJMX.js"), [], import.meta.url)).WasmGuitarInput), P;
    }
    const O = 1024;
    class q {
        audioContext = null;
        mediaStream = null;
        sourceNode = null;
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
        async start(e, t, n, s = O) {
            this._isRunning && await this.stop(), this.callbacks = n;
            const v = await V(), l = {
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
            this.mediaStream = await navigator.mediaDevices.getUserMedia(l), this.audioContext = new AudioContext, this.sourceNode = this.audioContext.createMediaStreamSource(this.mediaStream);
            const h = this.sourceNode.channelCount, m = this.audioContext.sampleRate;
            console.log(`[guitar] Device: ${h}ch @ ${m}Hz, want ch${t}, buffer=${s}`), this.windowSize = s, this.hopSize = Math.floor(s / 4), this.overlapBuffer = new Float32Array(s), this.overlapWritePos = 0, this.dsp = new v(m, s), this.dsp.set_onset_threshold(.015), this.dsp.set_string_confidence(.4), console.log(`[guitar] Overlap: window=${this.windowSize} hop=${this.hopSize} (75% overlap)`);
            const b = Math.max(t + 1, h);
            this.processorNode = this.audioContext.createScriptProcessor(s, b, 1), this.processorNode.channelCountMode = "explicit", this.processorNode.channelInterpretation = "discrete";
            const c = this;
            let w = !1;
            this._actualChannel = t, this.processorNode.onaudioprocess = (y)=>{
                if (!c._isRunning || !c.callbacks || !c.dsp) return;
                const E = y.inputBuffer, f = E.numberOfChannels, M = t < f ? t : 0;
                w || (console.log(`[guitar] Processing: requested ch=${t}, available=${f}, using ch=${M}`), c._actualChannel = M, w = !0);
                const u = E.getChannelData(M);
                let S = 0;
                for(let d = 0; d < u.length; d++)S += u[d] * u[d];
                const N = Math.sqrt(S / u.length);
                c._frameCount++, c._frameCount % 25 === 0 && console.log(`[guitar] frame=${c._frameCount} rms=${N.toFixed(4)} ch=${M}`);
                const D = c.dsp.process_block(u);
                let _;
                try {
                    _ = JSON.parse(D);
                } catch (d) {
                    console.error("[guitar] Failed to parse WASM events:", d);
                    return;
                }
                if (_.length > 0 && console.log(`[guitar] WASM events (${_.length}):`, JSON.stringify(_)), c.callbacks.onDetection) {
                    const d = _.findLast?.((L)=>L.type === "note_on");
                    d ? c.callbacks.onDetection({
                        frequency: null,
                        clarity: 1,
                        noteName: T(d.note),
                        midi: d.note,
                        cents: 0,
                        rms: N
                    }) : c.callbacks.onDetection({
                        frequency: null,
                        clarity: 0,
                        noteName: "-",
                        midi: 0,
                        cents: 0,
                        rms: N
                    });
                }
                for (const d of _)switch(d.type){
                    case "note_on":
                        console.log(`[midi] NOTE ON: ${T(d.note)} (${d.note}) vel=${d.velocity} ch=${d.channel}`), c.callbacks.onNoteOn(d.note, d.velocity);
                        break;
                    case "note_off":
                        console.log(`[midi] NOTE OFF: ${T(d.note)} (${d.note}) ch=${d.channel}`), c.callbacks.onNoteOff(d.note);
                        break;
                    case "pitch_bend":
                        c.callbacks.onPitchBend?.(d.channel, d.cents);
                        break;
                    case "midi_pitch_bend":
                        c.callbacks.onMidiPitchBend?.(d.channel, d.value);
                        break;
                    case "cc":
                        c.callbacks.onCC?.(d.channel, d.controller, d.value);
                        break;
                    case "channel_pressure":
                        c.callbacks.onChannelPressure?.(d.channel, d.pressure);
                        break;
                    case "vibrato":
                        c.callbacks.onVibratoStatus?.(d.active, d.rate_hz, d.depth_cents);
                        break;
                }
            }, this.sourceNode.connect(this.processorNode), this.processorNode.connect(this.audioContext.destination), this._isRunning = !0;
        }
        async stop() {
            if (this._isRunning = !1, this.processorNode && (this.processorNode.onaudioprocess = null, this.processorNode.disconnect(), this.processorNode = null), this.sourceNode && (this.sourceNode.disconnect(), this.sourceNode = null), this.audioContext) {
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
            this.callbacks = null;
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
            }), v = new AudioContext, l = v.createMediaStreamSource(s), h = Math.max(n + 1, l.channelCount), m = v.createScriptProcessor(O, h, 1);
            m.channelCountMode = "explicit", m.channelInterpretation = "discrete";
            let b = 0, c = 0;
            return new Promise((w)=>{
                m.onaudioprocess = (y)=>{
                    const E = Math.min(n, y.inputBuffer.numberOfChannels - 1), f = y.inputBuffer.getChannelData(E);
                    let M = 0;
                    for(let u = 0; u < f.length; u++)M += f[u] * f[u];
                    b += Math.sqrt(M / f.length), c++;
                }, l.connect(m), m.connect(v.destination), setTimeout(()=>{
                    m.onaudioprocess = null, m.disconnect(), l.disconnect(), v.close().catch(()=>{}), s.getTracks().forEach((y)=>y.stop()), w(c > 0 ? b / c : 0);
                }, t);
            });
        }
    }
    function T(i) {
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
        ], t = (i % 12 + 12) % 12, n = Math.floor(i / 12) - 1;
        return `${e[t]}${n}`;
    }
    const j = [
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
    function z(i, e, t = .7) {
        const n = i.length;
        let s = 0;
        for(let u = 0; u < n; u++)s += i[u] * i[u];
        if (Math.sqrt(s / n) < .01) return null;
        const l = Math.floor(e / 1400), h = Math.ceil(e / 60), m = Math.min(h, n - 1), b = new Float32Array(m + 1);
        for(let u = l; u <= m; u++){
            let S = 0, N = 0;
            const D = n - u;
            for(let _ = 0; _ < D; _++)S += i[_] * i[_ + u], N += i[_] * i[_] + i[_ + u] * i[_ + u];
            b[u] = N > 0 ? 2 * S / N : 0;
        }
        let c = -1, w = -1, y = !1, E = -1, f = -1;
        for(let u = l; u <= m; u++)if (b[u] > 0) y ? b[u] > E && (E = b[u], f = u) : (y = !0, E = b[u], f = u);
        else if (y) {
            if (E >= t) {
                c = f, w = E;
                break;
            }
            y = !1;
        }
        if (y && c < 0 && E >= t && (c = f, w = E), c < 0) return null;
        let M = c;
        if (c > l && c < m) {
            const u = b[c - 1], S = b[c], N = b[c + 1], D = 2 * (2 * S - u - N);
            Math.abs(D) > 1e-10 && (M = c + (u - N) / D);
        }
        return {
            frequency: e / M,
            clarity: w
        };
    }
    function J(i) {
        const e = 69 + 12 * Math.log2(i / 440), t = Math.round(e), n = Math.round((e - t) * 100);
        return {
            note: Math.max(0, Math.min(127, t)),
            cents: n
        };
    }
    function Y(i) {
        const e = (i % 12 + 12) % 12, t = Math.floor(i / 12) - 1;
        return `${j[e]}${t}`;
    }
    const U = "contrapunk-guitar";
    function Z() {
        try {
            const i = localStorage.getItem(U);
            return i ? JSON.parse(i) : {};
        } catch  {
            return {};
        }
    }
    class I {
        #e = a(21);
        get latencyMs() {
            return r(this.#e);
        }
        set latencyMs(e) {
            o(this.#e, e, !0);
        }
        #t = a(1);
        get gain() {
            return r(this.#t);
        }
        set gain(e) {
            o(this.#t, e, !0);
        }
        #n = a(.4);
        get stringConfidence() {
            return r(this.#n);
        }
        set stringConfidence(e) {
            o(this.#n, e, !0);
        }
        #i = a(!0);
        get bendsEnabled() {
            return r(this.#i);
        }
        set bendsEnabled(e) {
            o(this.#i, e, !0);
        }
        #s = a(!0);
        get legatoEnabled() {
            return r(this.#s);
        }
        set legatoEnabled(e) {
            o(this.#s, e, !0);
        }
        #a = a(!0);
        get slidesEnabled() {
            return r(this.#a);
        }
        set slidesEnabled(e) {
            o(this.#a, e, !0);
        }
        #r = a(!1);
        get vibratoEnabled() {
            return r(this.#r);
        }
        set vibratoEnabled(e) {
            o(this.#r, e, !0);
        }
        #o = a(R([]));
        get audioDevices() {
            return r(this.#o);
        }
        set audioDevices(e) {
            o(this.#o, e, !0);
        }
        #c = a("");
        get selectedDeviceId() {
            return r(this.#c);
        }
        set selectedDeviceId(e) {
            o(this.#c, e, !0);
        }
        #l = a(1);
        get selectedChannel() {
            return r(this.#l);
        }
        set selectedChannel(e) {
            o(this.#l, e, !0);
        }
        #u = a(2);
        get maxChannels() {
            return r(this.#u);
        }
        set maxChannels(e) {
            o(this.#u, e, !0);
        }
        constructor(){
            const e = Z();
            e.latencyMs !== void 0 && (this.latencyMs = e.latencyMs), e.gain !== void 0 && (this.gain = e.gain), e.stringConfidence !== void 0 && (this.stringConfidence = e.stringConfidence), e.bendsEnabled !== void 0 && (this.bendsEnabled = e.bendsEnabled), e.legatoEnabled !== void 0 && (this.legatoEnabled = e.legatoEnabled), e.slidesEnabled !== void 0 && (this.slidesEnabled = e.slidesEnabled), e.vibratoEnabled !== void 0 && (this.vibratoEnabled = e.vibratoEnabled), e.selectedDeviceId && (this.selectedDeviceId = e.selectedDeviceId), e.selectedChannel && (this.selectedChannel = e.selectedChannel), e.calibrated && (this.calibrated = e.calibrated), e.noiseGateEnabled !== void 0 && (this.noiseGateEnabled = e.noiseGateEnabled), e.noiseGateThreshold !== void 0 && (this.noiseGateThreshold = e.noiseGateThreshold), e.freqGateEnabled !== void 0 && (this.freqGateEnabled = e.freqGateEnabled), e.freqGateRange !== void 0 && (this.freqGateRange = e.freqGateRange);
        }
        persist() {
            try {
                localStorage.setItem(U, JSON.stringify({
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
        #d = a(null);
        get manualMaxChannels() {
            return r(this.#d);
        }
        set manualMaxChannels(e) {
            o(this.#d, e, !0);
        }
        #h = a("");
        get audioDeviceError() {
            return r(this.#h);
        }
        set audioDeviceError(e) {
            o(this.#h, e, !0);
        }
        #g = a(!1);
        get detecting() {
            return r(this.#g);
        }
        set detecting(e) {
            o(this.#g, e, !0);
        }
        #m = a(0);
        get activeChannel() {
            return r(this.#m);
        }
        set activeChannel(e) {
            o(this.#m, e, !0);
        }
        #f = a("");
        get currentNote() {
            return r(this.#f);
        }
        set currentNote(e) {
            o(this.#f, e, !0);
        }
        #p = a("");
        get currentString() {
            return r(this.#p);
        }
        set currentString(e) {
            o(this.#p, e, !0);
        }
        #v = a(0);
        get currentFret() {
            return r(this.#v);
        }
        set currentFret(e) {
            o(this.#v, e, !0);
        }
        #b = a(0);
        get confidence() {
            return r(this.#b);
        }
        set confidence(e) {
            o(this.#b, e, !0);
        }
        #_ = a(0);
        get velocity() {
            return r(this.#_);
        }
        set velocity(e) {
            o(this.#_, e, !0);
        }
        signalLevel = 0;
        signalClarity = 0;
        #y = a(!0);
        get noiseGateEnabled() {
            return r(this.#y);
        }
        set noiseGateEnabled(e) {
            o(this.#y, e, !0);
        }
        #E = a(.01);
        get noiseGateThreshold() {
            return r(this.#E);
        }
        set noiseGateThreshold(e) {
            o(this.#E, e, !0);
        }
        #C = a(!0);
        get freqGateEnabled() {
            return r(this.#C);
        }
        set freqGateEnabled(e) {
            o(this.#C, e, !0);
        }
        #w = a(1200);
        get freqGateRange() {
            return r(this.#w);
        }
        set freqGateRange(e) {
            o(this.#w, e, !0);
        }
        amplitudeHistory = [];
        clarityHistory = [];
        static HISTORY_SIZE = 128;
        pushSignalFrame(e, t) {
            this.signalLevel = Math.min(1, e * 5), this.signalClarity = t, this.amplitudeHistory.push(Math.min(1, e * 5)), this.clarityHistory.push(t), this.amplitudeHistory.length > I.HISTORY_SIZE && this.amplitudeHistory.shift(), this.clarityHistory.length > I.HISTORY_SIZE && this.clarityHistory.shift();
        }
        #M = a(!1);
        get calibrated() {
            return r(this.#M);
        }
        set calibrated(e) {
            o(this.#M, e, !0);
        }
        #N = a(!1);
        get calibrating() {
            return r(this.#N);
        }
        set calibrating(e) {
            o(this.#N, e, !0);
        }
        #S = a(!1);
        get tunerActive() {
            return r(this.#S);
        }
        set tunerActive(e) {
            o(this.#S, e, !0);
        }
        #D = a(0);
        get tunerStringIndex() {
            return r(this.#D);
        }
        set tunerStringIndex(e) {
            o(this.#D, e, !0);
        }
        #I = a("");
        get tunerDetectedNote() {
            return r(this.#I);
        }
        set tunerDetectedNote(e) {
            o(this.#I, e, !0);
        }
        #A = a(0);
        get tunerDetectedFreq() {
            return r(this.#A);
        }
        set tunerDetectedFreq(e) {
            o(this.#A, e, !0);
        }
        #P = a(0);
        get tunerCents() {
            return r(this.#P);
        }
        set tunerCents(e) {
            o(this.#P, e, !0);
        }
        #T = a(0);
        get tunerClarity() {
            return r(this.#T);
        }
        set tunerClarity(e) {
            o(this.#T, e, !0);
        }
        #F = a("waiting");
        get tunerStatus() {
            return r(this.#F);
        }
        set tunerStatus(e) {
            o(this.#F, e, !0);
        }
        #R = a(0);
        get tunerHoldProgress() {
            return r(this.#R);
        }
        set tunerHoldProgress(e) {
            o(this.#R, e, !0);
        }
        #x = a("noise-floor");
        get tunerPhase() {
            return r(this.#x);
        }
        set tunerPhase(e) {
            o(this.#x, e, !0);
        }
        #G = a(0);
        get tunerNoiseProgress() {
            return r(this.#G);
        }
        set tunerNoiseProgress(e) {
            o(this.#G, e, !0);
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
        #O = a(0);
        get noiseFloorRms() {
            return r(this.#O);
        }
        set noiseFloorRms(e) {
            o(this.#O, e, !0);
        }
        #$ = a("");
        get calibrationStatus() {
            return r(this.#$);
        }
        set calibrationStatus(e) {
            o(this.#$, e, !0);
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
                    const t = new q().measureNoiseFloor(this.selectedDeviceId, 3e3, this.selectedChannel - 1), n = Date.now(), s = setInterval(()=>{
                        const l = Date.now() - n;
                        this.tunerNoiseProgress = Math.min(1, l / 3e3);
                    }, 50), v = await t;
                    clearInterval(s), this.tunerNoiseProgress = 1, this.noiseFloorRms = v, this.calibrationStatus = `Noise floor: ${(v * 1e3).toFixed(1)} mRMS`, this.tunerPhase = "tuning", this.tunerStringIndex = 0, await this.startTunerCapture();
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
            }, n = await navigator.mediaDevices.getUserMedia(t);
            this.tunerStream = n;
            const s = new AudioContext({
                sampleRate: 48e3
            });
            this.tunerContext = s;
            const v = s.createMediaStreamSource(n), l = Math.max(e + 1, v.channelCount), h = s.createScriptProcessor(2048, l, 1);
            h.channelCountMode = "explicit", h.channelInterpretation = "discrete", this.tunerProcessor = h, h.onaudioprocess = (m)=>{
                if (!this.tunerActive || this.tunerPhase !== "tuning") return;
                const b = m.inputBuffer, c = Math.min(e, b.numberOfChannels - 1), w = b.getChannelData(c);
                let y = 0;
                for(let _ = 0; _ < w.length; _++)y += w[_] * w[_];
                const E = Math.sqrt(y / w.length), f = z(w, s.sampleRate), M = I.OPEN_STRINGS[this.tunerStringIndex];
                if (!M) return;
                if (!f || E < this.noiseFloorRms * 2) {
                    this.tunerDetectedNote = "", this.tunerDetectedFreq = 0, this.tunerCents = 0, this.tunerClarity = 0, this.tunerStatus = "waiting", this.tunerHoldProgress = 0, this.inTuneSince = null;
                    return;
                }
                const u = J(f.frequency), S = Y(u.note);
                this.tunerDetectedNote = S, this.tunerDetectedFreq = f.frequency, this.tunerClarity = f.clarity;
                const N = 1200 * Math.log2(f.frequency / M.freq);
                this.tunerCents = Math.round(N);
                const D = Math.abs(this.tunerCents);
                if (D > 2400) {
                    this.tunerStatus = "waiting", this.tunerHoldProgress = 0, this.inTuneSince = null;
                    return;
                }
                if (D <= I.IN_TUNE_CENTS) {
                    this.tunerStatus = "holding", this.inTuneSince === null && (this.inTuneSince = Date.now());
                    const _ = Date.now() - this.inTuneSince;
                    this.tunerHoldProgress = Math.min(1, _ / I.IN_TUNE_HOLD_MS), _ >= I.IN_TUNE_HOLD_MS && (this.tunerStatus = "in-tune", this.advanceTunerString());
                } else this.inTuneSince = null, this.tunerHoldProgress = 0, this.tunerStatus = this.tunerCents > 0 ? "sharp" : "flat";
            }, v.connect(h), h.connect(s.destination);
        }
        tunerStream = null;
        tunerContext = null;
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
            this.tunerProcessor && (this.tunerProcessor.disconnect(), this.tunerProcessor = null), this.tunerStream && (this.tunerStream.getTracks().forEach((e)=>e.stop()), this.tunerStream = null), this.tunerContext && (await this.tunerContext.close(), this.tunerContext = null), this.tunerCapture && (await this.tunerCapture.stop(), this.tunerCapture = null), this.tunerAnimFrame !== null && (cancelAnimationFrame(this.tunerAnimFrame), this.tunerAnimFrame = null);
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
        #k = a(R([]));
        get backendAudioDevices() {
            return r(this.#k);
        }
        set backendAudioDevices(e) {
            o(this.#k, e, !0);
        }
        async loadAudioDevices() {
            try {
                this.backendAudioDevices = await F.listAudioDevices();
            } catch  {
                this.backendAudioDevices = [];
            }
        }
        async syncConfig() {
            try {
                await F.setGuitarConfig({
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
                await F.setGuitarDevice(t, n);
            } catch  {}
        }
    }
    g = new I;
    function K(i, e) {
        return {
            key: i.key,
            mode: i.mode,
            modeNumber: i.mode_number,
            scaleMode: i.scale_mode,
            octaveMode: i.octave_mode,
            voiceLeadingEnabled: i.voice_leading_enabled,
            voiceLeadingStyle: i.voice_leading_style,
            interchangeEnabled: i.interchange_enabled,
            interchangeRange: i.borrowing_range,
            voicePosition: i.voice_position,
            voiceCount: i.voice_count,
            isRunning: e
        };
    }
    function $(i) {
        return {
            inputNotes: i.input_notes,
            harmonyNotes: i.harmony_notes,
            borrowedNotes: i.borrowed_notes,
            chordName: i.chord_name,
            lastBorrowedFrom: i.last_borrowed_from
        };
    }
    class Q {
        _isRunning = !1;
        _guitarSignalUnsub = null;
        async init() {
            await this.getEngineState();
        }
        async getEngineState() {
            try {
                const e = await p("get_engine_state");
                return K(e, this._isRunning);
            } catch (e) {
                throw new Error(`Failed to get engine state: ${e}`);
            }
        }
        async setKey(e) {
            try {
                await p("set_key", {
                    key: e
                });
            } catch (t) {
                throw new Error(`Failed to set key: ${t}`);
            }
        }
        async setMode(e) {
            try {
                await p("set_mode", {
                    mode: e
                });
            } catch (t) {
                throw new Error(`Failed to set mode: ${t}`);
            }
        }
        async setScaleMode(e) {
            try {
                await p("set_scale_mode", {
                    mode: e
                });
            } catch (t) {
                throw new Error(`Failed to set scale mode: ${t}`);
            }
        }
        async setOctaveMode(e) {
            try {
                await p("set_octave_mode", {
                    mode: e
                });
            } catch (t) {
                throw new Error(`Failed to set octave mode: ${t}`);
            }
        }
        async setVoiceLeading(e, t) {
            try {
                await p("set_voice_leading", {
                    enabled: e,
                    style: t
                });
            } catch (n) {
                throw new Error(`Failed to set voice leading: ${n}`);
            }
        }
        async setInterchange(e, t) {
            try {
                await p("set_interchange", {
                    enabled: e,
                    range: t
                });
            } catch (n) {
                throw new Error(`Failed to set interchange: ${n}`);
            }
        }
        async setVoicePosition(e) {
            try {
                await p("set_voice_position", {
                    position: e
                });
            } catch (t) {
                throw new Error(`Failed to set voice position: ${t}`);
            }
        }
        async setVoiceCount(e) {
            try {
                await p("set_voice_count", {
                    count: e
                });
            } catch (t) {
                throw new Error(`Failed to set voice count: ${t}`);
            }
        }
        async getHumanizeState() {
            try {
                const e = await p("get_humanize_state");
                return {
                    enabled: e.enabled,
                    jitterEnabled: e.jitter_enabled,
                    jitterMinMs: e.jitter_min_ms,
                    jitterMaxMs: e.jitter_max_ms,
                    velocityEnabled: e.velocity_enabled,
                    velocityVariation: e.velocity_variation,
                    durationEnabled: e.duration_enabled,
                    durationVariationMs: e.duration_variation_ms,
                    swingEnabled: e.swing_enabled,
                    swingAmount: e.swing_amount,
                    bpm: e.bpm,
                    metronomeEnabled: e.metronome_enabled
                };
            } catch (e) {
                throw new Error(`Failed to get humanize state: ${e}`);
            }
        }
        async setHumanizeConfig(e) {
            try {
                const t = {};
                e.enabled !== void 0 && (t.enabled = e.enabled), e.jitterEnabled !== void 0 && (t.jitter_enabled = e.jitterEnabled), e.jitterMinMs !== void 0 && (t.jitter_min_ms = e.jitterMinMs), e.jitterMaxMs !== void 0 && (t.jitter_max_ms = e.jitterMaxMs), e.velocityEnabled !== void 0 && (t.velocity_enabled = e.velocityEnabled), e.velocityVariation !== void 0 && (t.velocity_variation = e.velocityVariation), e.durationEnabled !== void 0 && (t.duration_enabled = e.durationEnabled), e.durationVariationMs !== void 0 && (t.duration_variation_ms = e.durationVariationMs), e.swingEnabled !== void 0 && (t.swing_enabled = e.swingEnabled), e.swingAmount !== void 0 && (t.swing_amount = e.swingAmount), e.bpm !== void 0 && (t.bpm = e.bpm), e.metronomeEnabled !== void 0 && (t.metronome_enabled = e.metronomeEnabled), await p("set_humanize_config", {
                    config: t
                });
            } catch (t) {
                throw new Error(`Failed to set humanize config: ${t}`);
            }
        }
        async listMidiInputs() {
            try {
                return (await p("list_midi_inputs")).map((t)=>({
                        index: t.index,
                        name: t.name
                    }));
            } catch (e) {
                throw new Error(`Failed to list MIDI inputs: ${e}`);
            }
        }
        async listMidiOutputs() {
            try {
                return (await p("list_midi_outputs")).map((t)=>({
                        index: t.index,
                        name: t.name
                    }));
            } catch (e) {
                throw new Error(`Failed to list MIDI outputs: ${e}`);
            }
        }
        async refreshMidiDevices() {
            try {
                await p("refresh_midi_devices");
            } catch (e) {
                throw new Error(`Failed to refresh MIDI devices: ${e}`);
            }
        }
        async startRouting(e, t) {
            try {
                if (await p("start_routing", {
                    inputIdx: e,
                    outputIndices: t
                }), this._isRunning = !0, e === 999997) {
                    g.detecting = !0;
                    let s = 0;
                    const v = 100;
                    this._guitarSignalUnsub = await G("guitar-signal", (l)=>{
                        const h = l.payload, m = h.rms, b = h.clarity, c = h.note_name;
                        g.pushSignalFrame(m, b);
                        const w = performance.now();
                        w - s > v && (s = w, c ? (g.currentNote = c, g.confidence = Math.round(b * 100), g.velocity = Math.round(m * 800)) : (g.currentNote = "", g.confidence = 0));
                    });
                }
            } catch (n) {
                throw new Error(`Failed to start routing: ${n}`);
            }
        }
        async stopRouting() {
            try {
                await p("stop_routing"), this._isRunning = !1, this._guitarSignalUnsub && (this._guitarSignalUnsub(), this._guitarSignalUnsub = null), g.detecting = !1, g.currentNote = "", g.confidence = 0, g.velocity = 0;
            } catch (e) {
                throw new Error(`Failed to stop routing: ${e}`);
            }
        }
        async injectNoteOn(e, t) {
            try {
                return await p("inject_note_on", {
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
                return await p("inject_note_off", {
                    note: e
                });
            } catch  {
                return [
                    e
                ];
            }
        }
        async getNoteState() {
            try {
                const e = await p("get_note_state");
                return $(e);
            } catch (e) {
                throw new Error(`Failed to get note state: ${e}`);
            }
        }
        onNoteUpdate(e) {
            let t, n = !1;
            return G("note-update", (s)=>{
                n || e($(s.payload));
            }).then((s)=>{
                n ? s() : t = s;
            }), ()=>{
                n = !0, t?.();
            };
        }
        async listPresets() {
            try {
                return (await p("list_presets")).map((t)=>({
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
                await p("load_preset", {
                    name: e
                });
            } catch (t) {
                throw new Error(`Failed to load preset: ${t}`);
            }
        }
        async savePreset(e) {
            try {
                await p("save_preset", {
                    name: e
                });
            } catch (t) {
                throw new Error(`Failed to save preset: ${t}`);
            }
        }
        async deletePreset(e) {
            try {
                await p("delete_preset", {
                    name: e
                });
            } catch (t) {
                throw new Error(`Failed to delete preset: ${t}`);
            }
        }
        async listAudioDevices() {
            try {
                return await p("list_audio_devices");
            } catch (e) {
                throw new Error(`Failed to list audio devices: ${e}`);
            }
        }
        async setGuitarDevice(e, t) {
            try {
                await p("set_guitar_device", {
                    deviceName: e,
                    channel: t
                });
            } catch (n) {
                throw new Error(`Failed to set guitar device: ${n}`);
            }
        }
        async setGuitarConfig(e) {
            try {
                await p("set_guitar_config", {
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
            this._detuneCents = e;
        }
        getDetune() {
            return this._detuneCents;
        }
    }
    let A = null, C = null;
    const X = 999997;
    class ee {
        initialized = !1;
        _isRunning = !1;
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
            if (!this.initialized) try {
                A = await k(()=>import("./CMqHBJMX.js"), [], import.meta.url), A.default && typeof A.default == "function" && await A.default(), C = new A.Engine, this.initialized = !0;
            } catch (e) {
                throw new Error(`Failed to initialize WASM: ${e}`);
            }
        }
        ensureInit() {
            if (!this.initialized || !C) throw new Error("WASM adapter not initialized. Call init() first.");
        }
        async getEngineState() {
            this.ensureInit();
            try {
                const e = C.get_state();
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
                    isRunning: this._isRunning
                };
            } catch (e) {
                throw new Error(`Failed to get engine state: ${e}`);
            }
        }
        async setKey(e) {
            this.ensureInit();
            try {
                C.set_key(e);
            } catch (t) {
                throw new Error(`Failed to set key: ${t}`);
            }
        }
        async setMode(e) {
            this.ensureInit();
            try {
                C.set_mode(e);
            } catch (t) {
                throw new Error(`Failed to set mode: ${t}`);
            }
        }
        async setScaleMode(e) {
            this.ensureInit();
            try {
                C.set_scale_mode(e);
            } catch (t) {
                throw new Error(`Failed to set scale mode: ${t}`);
            }
        }
        async setOctaveMode(e) {
            this.ensureInit();
            try {
                C.set_octave_mode(e);
            } catch (t) {
                throw new Error(`Failed to set octave mode: ${t}`);
            }
        }
        async setVoiceLeading(e, t) {
            this.ensureInit();
            try {
                C.set_voice_leading(e, t);
            } catch (n) {
                throw new Error(`Failed to set voice leading: ${n}`);
            }
        }
        async setInterchange(e, t) {
            this.ensureInit();
            try {
                C.set_interchange(e, t);
            } catch (n) {
                throw new Error(`Failed to set interchange: ${n}`);
            }
        }
        async setVoicePosition(e) {
            this.ensureInit();
            try {
                C.set_voice_position(e);
            } catch (t) {
                throw new Error(`Failed to set voice position: ${t}`);
            }
        }
        async setVoiceCount(e) {
            this.ensureInit();
            try {
                C.set_voice_count(e);
            } catch (t) {
                throw new Error(`Failed to set voice count: ${t}`);
            }
        }
        async getHumanizeState() {
            return {
                enabled: !1,
                jitterEnabled: !1,
                jitterMinMs: 1,
                jitterMaxMs: 10,
                velocityEnabled: !1,
                velocityVariation: 10,
                durationEnabled: !1,
                durationVariationMs: 0,
                swingEnabled: !1,
                swingAmount: 0,
                bpm: 120,
                metronomeEnabled: !1
            };
        }
        async setHumanizeConfig(e) {}
        async ensureMidiAccess() {
            if (this.midiAccess) return this.midiAccess;
            if (typeof navigator > "u" || !("requestMIDIAccess" in navigator)) return null;
            try {
                return this.midiAccess = await navigator.requestMIDIAccess(), this.midiAccess;
            } catch  {
                return null;
            }
        }
        async listMidiInputs() {
            const e = await this.ensureMidiAccess();
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
            const e = await this.ensureMidiAccess();
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
            this.midiAccess = null;
        }
        async startRouting(e, t) {
            if (this.ensureInit(), e === X) {
                await this.startGuitarCapture(t);
                return;
            }
            const n = await this.ensureMidiAccess();
            if (!n) {
                this._isRunning = !0;
                return;
            }
            const s = Array.from(n.inputs.values());
            e >= 0 && e < s.length && (this.activeInput = s[e]);
            const v = Array.from(n.outputs.values());
            if (this.activeOutputs = t.filter((l)=>l >= 0 && l < v.length).map((l)=>v[l]), this.activeInput) {
                const l = this, h = this.activeOutputs;
                this.activeInput.onmidimessage = (m)=>{
                    if (!m.data || m.data.length < 2) return;
                    const b = m.data[0] & 240, c = m.data[1], w = m.data.length > 2 ? m.data[2] : 0;
                    let y = [];
                    if (b === 144 && w > 0) {
                        try {
                            y = C.note_on(c);
                        } catch  {
                            y = [
                                c
                            ];
                        }
                        const E = l.sortVoices(y);
                        for(let f = 0; f < E.length; f++)h.length > 0 && h[f % h.length].send([
                            144,
                            E[f],
                            w
                        ]);
                    } else if (b === 128 || b === 144 && w === 0) {
                        try {
                            y = C.note_off(c);
                        } catch  {
                            y = [
                                c
                            ];
                        }
                        const E = l.sortVoices(y);
                        for(let f = 0; f < E.length; f++)h.length > 0 && h[f % h.length].send([
                            128,
                            E[f],
                            0
                        ]);
                    } else for (const E of h)E.send(Array.from(m.data));
                };
            }
            this._isRunning = !0, this._detuneCents !== 0 && this.sendPitchBend();
        }
        async startGuitarCapture(e) {
            const t = await this.ensureMidiAccess();
            if (t) {
                const l = Array.from(t.outputs.values());
                this.activeOutputs = e.filter((h)=>h >= 0 && h < l.length).map((h)=>l[h]);
            }
            this.guitarCapture = new q;
            const n = this;
            g.detecting = !0, this.guitarCapture.noiseGateThreshold = g.noiseGateThreshold, this.guitarCapture.noiseGateEnabled = g.noiseGateEnabled, this.guitarCapture.clarityGateEnabled = !1;
            const s = g.selectedDeviceId || this._guitarDeviceId, v = Math.max(0, g.selectedChannel - 1);
            console.log(`[wasm] startGuitarCapture: device='${s}' channel=${v} (store.selectedChannel=${g.selectedChannel})`), await this.guitarCapture.start(s, v, {
                onNoteOn (l, h) {
                    n.injectNoteOn(l, h).catch(()=>{});
                },
                onNoteOff (l) {
                    n.injectNoteOff(l).catch(()=>{});
                },
                onDetection (l) {
                    n.guitarCapture && (n.guitarCapture.noiseGateThreshold = g.noiseGateThreshold, n.guitarCapture.noiseGateEnabled = g.noiseGateEnabled, g.activeChannel = n.guitarCapture.actualChannel + 1), g.pushSignalFrame(l.rms, l.clarity), l.frequency !== null ? (g.currentNote = l.noteName, g.confidence = Math.round(l.clarity * 100), g.velocity = Math.round(l.rms * 800)) : (g.currentNote = "", g.confidence = 0);
                }
            }), g.activeChannel = this.guitarCapture.actualChannel + 1, this._isRunning = !0;
        }
        async stopRouting() {
            this.guitarCapture && (await this.guitarCapture.stop(), this.guitarCapture = null, g.detecting = !1, g.currentNote = "", g.confidence = 0, g.velocity = 0);
            for (const e of this.activeOutputs)try {
                e.send([
                    176,
                    123,
                    0
                ]);
            } catch  {}
            if (C) try {
                C.clear_notes();
            } catch  {}
            this.activeInput && (this.activeInput.onmidimessage = null, this.activeInput = null), this.activeOutputs = [], this._isRunning = !1, this.stopNotePolling();
        }
        async injectNoteOn(e, t) {
            this.ensureInit();
            try {
                const n = C.note_on(e), s = this.sortVoices(n ?? [
                    e
                ]), v = t ?? 100;
                for(let l = 0; l < s.length; l++)this.activeOutputs.length > 0 && this.activeOutputs[l % this.activeOutputs.length].send([
                    144,
                    s[l],
                    v
                ]);
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
                const t = C.note_off(e), n = this.sortVoices(t ?? [
                    e
                ]);
                for(let s = 0; s < n.length; s++)this.activeOutputs.length > 0 && this.activeOutputs[s % this.activeOutputs.length].send([
                    128,
                    n[s],
                    0
                ]);
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
                const e = C.get_note_state();
                return {
                    inputNotes: e?.input_notes ?? [],
                    harmonyNotes: e?.harmony_notes ?? [],
                    borrowedNotes: e?.borrowed_notes ?? [],
                    chordName: e?.chord_name ?? "",
                    lastBorrowedFrom: e?.last_borrowed_from ?? ""
                };
            } catch  {
                return {
                    inputNotes: [],
                    harmonyNotes: [],
                    borrowedNotes: [],
                    chordName: "",
                    lastBorrowedFrom: ""
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
        stopNotePolling() {
            this.pollingHandle !== null && (cancelAnimationFrame(this.pollingHandle), this.pollingHandle = null);
        }
        async listPresets() {
            this.ensureInit();
            try {
                return (C.list_presets() ?? []).map((t, n)=>({
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
                C.load_preset(e);
            } catch (t) {
                throw new Error(`Failed to load preset: ${t}`);
            }
        }
        async savePreset(e) {
            this.ensureInit();
            try {
                C.save_preset(e);
            } catch (t) {
                throw new Error(`Failed to save preset: ${t}`);
            }
        }
        async deletePreset(e) {
            this.ensureInit();
            try {
                C.delete_preset(e);
            } catch (t) {
                throw new Error(`Failed to delete preset: ${t}`);
            }
        }
    }
    function H() {
        return typeof window < "u" && ("__TAURI_INTERNALS__" in window || "__TAURI__" in window);
    }
    let te;
    te = H() ? "tauri" : "browser";
    F = H() ? new Q : new ee;
    class ne {
        #e = a(!1);
        get reducedMotion() {
            return r(this.#e);
        }
        set reducedMotion(e) {
            o(this.#e, e, !0);
        }
        #t = a(!0);
        get animationsEnabled() {
            return r(this.#t);
        }
        set animationsEnabled(e) {
            o(this.#t, e, !0);
        }
        #n = a(R(te));
        get platform() {
            return r(this.#n);
        }
        set platform(e) {
            o(this.#n, e, !0);
        }
        #i = a(!1);
        get initialized() {
            return r(this.#i);
        }
        set initialized(e) {
            o(this.#i, e, !0);
        }
        #s = a(null);
        get error() {
            return r(this.#s);
        }
        set error(e) {
            o(this.#s, e, !0);
        }
        #a = a(!1);
        get sidebarCollapsed() {
            return r(this.#a);
        }
        set sidebarCollapsed(e) {
            o(this.#a, e, !0);
        }
        #r = a("play");
        get activePanel() {
            return r(this.#r);
        }
        set activePanel(e) {
            o(this.#r, e, !0);
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
    }
    ae = new ne;
})();
export { F as a, g, ae as u, __tla };
