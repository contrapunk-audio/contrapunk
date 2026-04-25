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

const ATTACK_S = 0.005;
const DECAY_S = 0.05;
const SUSTAIN = 0.6;
const RELEASE_S = 0.25;
const VOICE_GAIN = 0.18;
const SMOOTH_S = 0.02;

function midiToFreq(midi: number): number {
	return 440 * Math.pow(2, (midi - 69) / 12);
}

function buildReverbIR(audioCtx: AudioContext, durationS = 1.6, decay = 2.5): AudioBuffer {
	const sampleRate = audioCtx.sampleRate;
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

	dryBus = ctx.createGain();
	dryBus.gain.value = 1.0;
	dryBus.connect(masterGain);

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
	env.gain.linearRampToValueAtTime(peak, now + ATTACK_S);
	env.gain.linearRampToValueAtTime(peak * SUSTAIN, now + ATTACK_S + DECAY_S);

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
	v.env.gain.linearRampToValueAtTime(0, now + RELEASE_S);
	try {
		v.osc.stop(now + RELEASE_S + 0.02);
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
