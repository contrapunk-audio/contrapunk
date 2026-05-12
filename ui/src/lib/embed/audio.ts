/**
 * Web Audio synth for browser-path consumers (contrapunk web build
 * + website embed widgets).
 *
 * Triangle oscillator per voice, simple ADSR envelope, single shared
 * delay + reverb chain. State-library-agnostic — internal mutable
 * params plus imperative setters; callers can wrap the setters to
 * update their own UI stores.
 *
 * Until the Rust synth + FX modules port to WASM, this is the only
 * way the browser produces sound. Lazy AudioContext init on first
 * noteOn (the user-gesture autoplay-policy contract is satisfied
 * because that first call comes from a click / keypress).
 *
 * Canonical here in contrapunk/ui/src/lib/embed/. Mirror-copied to
 * website/src/lib/contrapunk/embed/. Per #66.
 */

export type Waveform = 'triangle' | 'sine' | 'square' | 'sawtooth';

interface Voice {
	osc: OscillatorNode;
	env: GainNode;
}

let ctx: AudioContext | null = null;
let masterGain: GainNode | null = null;
let dryBus: GainNode | null = null;
/** Shared lowpass filter — every voice routes its env through this
 *  so the synth's cutoff/resonance knobs in the UI control timbre. */
let filterNode: BiquadFilterNode | null = null;
let delayNode: DelayNode | null = null;
let delayFeedback: GainNode | null = null;
let delayWet: GainNode | null = null;
let reverbNode: ConvolverNode | null = null;
let reverbWet: GainNode | null = null;

const voices = new Map<number, Voice>();

// Tunable params. Defaults match the contrapunk app's synth defaults
// closely enough that the browser path doesn't surprise users
// switching between native and web.
let currentWaveform: Waveform = 'triangle';
let currentMasterGain = 0.7;
let currentDelayMix = 0.22;
let currentDelayFeedback = 0.32;
let currentDelayTime = 0.32;
let currentReverbMix = 0.22;
let currentReverbRoomSize = 0.7;
let currentReverbDamping = 0.5;
let currentCutoffHz = 6000;
let currentResonance = 0.2;

// ADSR — mutable so the Harmony tab's envelope knobs work. Defaults
// kept short/snappy so unconfigured presets sound responsive.
let attackS = 0.005;
let decayS = 0.05;
let sustainLevel = 0.6;
let releaseS = 0.25;

const VOICE_GAIN = 0.18;
const SMOOTH_S = 0.02;

function midiToFreq(midi: number): number {
	return 440 * Math.pow(2, (midi - 69) / 12);
}

/**
 * UI "resonance" knob is 0..1; the Rust biquad maps the same range to
 * a perceptually-smooth Q sweep that tops out around 12 (audibly
 * squelchy but not self-oscillating). Linear mapping is good enough
 * for parity until users notice a difference between native and web.
 */
function mapResonanceToQ(resonance: number): number {
	const r = Math.max(0, Math.min(1, resonance));
	return 0.5 + r * 11.5;
}

/**
 * Synthesize a stereo impulse response for the convolver. roomSize
 * (0..1) sets the duration (~0.3s..3.2s); damping (0..1) controls
 * decay rate — higher = brighter / longer tail. The shape is white
 * noise with an exponential envelope so it's CPU-cheap to rebuild
 * when the UI sliders move.
 */
function buildReverbIR(
	audioCtx: AudioContext,
	roomSize = currentReverbRoomSize,
	damping = currentReverbDamping
): AudioBuffer {
	const sampleRate = audioCtx.sampleRate;
	// Map roomSize 0..1 to 0.3..3.2 s. Damping inverts: higher damping
	// = shorter tail (more absorption), so exponent grows.
	const durationS = 0.3 + 2.9 * Math.max(0, Math.min(1, roomSize));
	const decay = 1.0 + 4.5 * Math.max(0, Math.min(1, damping));
	const length = Math.floor(sampleRate * durationS);
	const buffer = audioCtx.createBuffer(2, length, sampleRate);
	for (let ch = 0; ch < 2; ch++) {
		const data = buffer.getChannelData(ch);
		for (let i = 0; i < length; i++) {
			const t = i / length;
			data[i] = (Math.random() * 2 - 1) * Math.pow(1 - t, decay);
		}
	}
	return buffer;
}

function ensureAudio(): AudioContext | null {
	if (ctx) return ctx;
	try {
		const Ctor =
			window.AudioContext ??
			(window as unknown as { webkitAudioContext?: typeof AudioContext }).webkitAudioContext;
		if (!Ctor) return null;
		ctx = new Ctor();
	} catch {
		return null;
	}

	masterGain = ctx.createGain();
	masterGain.gain.value = currentMasterGain;
	masterGain.connect(ctx.destination);

	// Shared lowpass filter — Q maps loosely to Tauri's "resonance"
	// parameter on the Rust biquad. Bypassed-feeling at cutoff
	// >= 18kHz, audibly squelchy below ~1.5kHz.
	filterNode = ctx.createBiquadFilter();
	filterNode.type = 'lowpass';
	filterNode.frequency.value = currentCutoffHz;
	filterNode.Q.value = mapResonanceToQ(currentResonance);
	filterNode.connect(masterGain);

	dryBus = ctx.createGain();
	dryBus.gain.value = 1.0;
	dryBus.connect(filterNode);

	delayNode = ctx.createDelay(2.0);
	delayNode.delayTime.value = currentDelayTime;
	delayFeedback = ctx.createGain();
	delayFeedback.gain.value = currentDelayFeedback;
	delayWet = ctx.createGain();
	delayWet.gain.value = currentDelayMix;
	delayNode.connect(delayFeedback);
	delayFeedback.connect(delayNode);
	delayNode.connect(delayWet);
	delayWet.connect(masterGain);

	reverbNode = ctx.createConvolver();
	reverbNode.buffer = buildReverbIR(ctx);
	reverbWet = ctx.createGain();
	reverbWet.gain.value = currentReverbMix;
	reverbNode.connect(reverbWet);
	reverbWet.connect(masterGain);

	return ctx;
}

async function resumeIfSuspended() {
	if (!ctx) return;
	if (ctx.state === 'suspended') {
		try {
			await ctx.resume();
		} catch {
			// suspended → resume fails outside a user gesture; the next
			// click will give us another chance
		}
	}
}

export function noteOn(midi: number, velocity = 100) {
	const audio = ensureAudio();
	if (!audio || !dryBus || !delayNode || !reverbNode) return;
	void resumeIfSuspended();

	const existing = voices.get(midi);
	if (existing) {
		try {
			existing.osc.stop(audio.currentTime + 0.001);
		} catch {
			// already stopped
		}
		voices.delete(midi);
	}

	const osc = audio.createOscillator();
	osc.type = currentWaveform;
	osc.frequency.value = midiToFreq(midi);

	const env = audio.createGain();
	env.gain.value = 0;

	osc.connect(env);
	env.connect(dryBus);
	env.connect(delayNode);
	env.connect(reverbNode);

	const now = audio.currentTime;
	const peak = VOICE_GAIN * (velocity / 127);
	env.gain.setValueAtTime(0, now);
	env.gain.linearRampToValueAtTime(peak, now + attackS);
	env.gain.linearRampToValueAtTime(peak * sustainLevel, now + attackS + decayS);

	osc.start(now);

	osc.onended = () => {
		try {
			env.disconnect();
			osc.disconnect();
		} catch {
			// already disconnected
		}
	};

	voices.set(midi, { osc, env });
}

export function noteOff(midi: number) {
	if (!ctx) return;
	const v = voices.get(midi);
	if (!v) return;
	const now = ctx.currentTime;
	const current = v.env.gain.value;
	v.env.gain.cancelScheduledValues(now);
	v.env.gain.setValueAtTime(current, now);
	v.env.gain.linearRampToValueAtTime(0, now + releaseS);
	try {
		v.osc.stop(now + releaseS + 0.02);
	} catch {
		// already stopped
	}
	voices.delete(midi);
}

export function allNotesOff() {
	if (!ctx) return;
	for (const midi of Array.from(voices.keys())) {
		noteOff(midi);
	}
}

// === Imperative setters — update internal state + live audio nodes.

export function setWaveform(w: Waveform) {
	currentWaveform = w;
	for (const v of voices.values()) v.osc.type = w;
}
export function getWaveform(): Waveform {
	return currentWaveform;
}

export function setMasterGain(g: number) {
	currentMasterGain = Math.max(0, Math.min(1, g));
	if (ctx && masterGain) {
		masterGain.gain.setTargetAtTime(currentMasterGain, ctx.currentTime, SMOOTH_S);
	}
}
export function setDelayMix(mix: number) {
	currentDelayMix = Math.max(0, Math.min(1, mix));
	if (ctx && delayWet) {
		delayWet.gain.setTargetAtTime(currentDelayMix, ctx.currentTime, SMOOTH_S);
	}
}
export function setDelayFeedback(fb: number) {
	currentDelayFeedback = Math.max(0, Math.min(0.95, fb));
	if (ctx && delayFeedback) {
		delayFeedback.gain.setTargetAtTime(currentDelayFeedback, ctx.currentTime, SMOOTH_S);
	}
}
export function setDelayTime(t: number) {
	currentDelayTime = Math.max(0.01, Math.min(2, t));
	if (ctx && delayNode) {
		delayNode.delayTime.setTargetAtTime(currentDelayTime, ctx.currentTime, SMOOTH_S);
	}
}
export function setReverbMix(mix: number) {
	currentReverbMix = Math.max(0, Math.min(1, mix));
	if (ctx && reverbWet) {
		reverbWet.gain.setTargetAtTime(currentReverbMix, ctx.currentTime, SMOOTH_S);
	}
}

// === Filter (lowpass after the dry bus) ===
export function setCutoffHz(hz: number) {
	currentCutoffHz = Math.max(20, Math.min(20_000, hz));
	if (ctx && filterNode) {
		filterNode.frequency.setTargetAtTime(currentCutoffHz, ctx.currentTime, SMOOTH_S);
	}
}
export function setResonance(value: number) {
	currentResonance = Math.max(0, Math.min(1, value));
	if (ctx && filterNode) {
		filterNode.Q.setTargetAtTime(mapResonanceToQ(currentResonance), ctx.currentTime, SMOOTH_S);
	}
}

// === ADSR (applied to subsequent noteOn / noteOff calls) ===
export function setAttackMs(ms: number) {
	attackS = Math.max(0, ms) / 1000;
}
export function setDecayMs(ms: number) {
	decayS = Math.max(0, ms) / 1000;
}
export function setSustain(level: number) {
	sustainLevel = Math.max(0, Math.min(1, level));
}
export function setReleaseMs(ms: number) {
	releaseS = Math.max(0, ms) / 1000;
}

// === Reverb shape (rebuilds the impulse-response buffer) ===
export function setReverbRoomSize(size: number) {
	currentReverbRoomSize = Math.max(0, Math.min(1, size));
	rebuildReverbIR();
}
export function setReverbDamping(d: number) {
	currentReverbDamping = Math.max(0, Math.min(1, d));
	rebuildReverbIR();
}

function rebuildReverbIR() {
	if (!ctx || !reverbNode) return;
	reverbNode.buffer = buildReverbIR(ctx, currentReverbRoomSize, currentReverbDamping);
}
