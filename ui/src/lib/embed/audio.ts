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
/** Master-bus dynamics compressor — limits amplitude on dense
 *  multi-voice attacks (companion stacking 4-8 voices). See setup
 *  in ensureAudio() for tuning rationale. */
let masterCompressor: DynamicsCompressorNode | null = null;
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
/** Voice-retrigger behavior toggle. When `true` (default), a NoteOn
 *  arriving on a MIDI pitch that already has a Voice gracefully fades
 *  the old voice out over ~15 ms while the new voice's attack ramp
 *  comes up — no audible click. When `false`, falls back to the
 *  legacy 1 ms hard `osc.stop` (cheaper, but clicks on retrigger).
 *  UI surface exposes this so users can pick fast-but-clicky vs
 *  smooth-but-50ms-attack-overlap. */
let crossfadeRetrigger = true;
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

	// Dry-path dynamics compressor — protects against amplitude
	// pile-up when the companion stacks 4-8 voices on the same beat.
	// Placement: AFTER the lowpass filter, BEFORE the master sum.
	// Critically, the delay/reverb wets bypass this compressor and
	// reach masterGain directly — otherwise reverb tails would
	// trigger the compressor on every dry attack and "duck" the
	// reverb (audible pumping). Tuned for transparency: 4:1 ratio
	// above -12 dB with a 12 dB knee = ~6 dB of gain reduction on a
	// 6-voice attack, almost no audible squashing in normal use.
	// Slow release (250 ms) avoids breathing artifacts on sustained
	// chords.
	masterCompressor = ctx.createDynamicsCompressor();
	masterCompressor.threshold.value = -12;
	masterCompressor.knee.value = 12;
	masterCompressor.ratio.value = 4;
	masterCompressor.attack.value = 0.005;
	masterCompressor.release.value = 0.25;
	masterCompressor.connect(masterGain);

	// Shared lowpass filter — Q maps loosely to Tauri's "resonance"
	// parameter on the Rust biquad. Bypassed-feeling at cutoff
	// >= 18kHz, audibly squelchy below ~1.5kHz.
	filterNode = ctx.createBiquadFilter();
	filterNode.type = 'lowpass';
	filterNode.frequency.value = currentCutoffHz;
	filterNode.Q.value = mapResonanceToQ(currentResonance);
	filterNode.connect(masterCompressor);

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

/** Note-on with optional precise scheduling.
 *
 *  `scheduleAt` (audio-clock seconds, i.e. an `AudioContext.currentTime`
 *  value) lets the caller pre-schedule a note in the future instead of
 *  firing immediately. Used by the companion tick loop to lookahead-
 *  schedule lane emissions: rAF jitter (16-50 ms stalls under load) no
 *  longer translates 1:1 into audio jitter, because the underlying
 *  Web Audio scheduler honors the exact `when` regardless of when
 *  `noteOn` was called.
 *
 *  Player-input callers omit `scheduleAt` → defaults to
 *  `audio.currentTime` (zero added latency, preserves the sub-10 ms
 *  branding). Only companion-emitted notes pay the lookahead cost. */
export function noteOn(midi: number, velocity = 100, scheduleAt?: number) {
	const audio = ensureAudio();
	if (!audio || !dryBus || !delayNode || !reverbNode) return;
	void resumeIfSuspended();
	// NaN/Infinity guard — `Math.max(currentTime, NaN)` returns NaN,
	// which then propagates into setValueAtTime / osc.start and throws
	// in Chrome (silent no-op in some Safari versions). Today the only
	// caller is dispatchCompanionOps which computes a real number, but
	// any future scheduler-sync caller could pass garbage. Treat NaN /
	// Infinity / negative as "fire now".
	const safeScheduleAt =
		typeof scheduleAt === 'number' && Number.isFinite(scheduleAt) ? scheduleAt : audio.currentTime;
	const when = Math.max(audio.currentTime, safeScheduleAt);

	const existing = voices.get(midi);
	if (existing) {
		if (crossfadeRetrigger) {
			// Envelope-aware crossfade. The previous hard `osc.stop(now
			// + 0.001)` was a 1 ms gain step at a non-zero waveform
			// phase — a textbook click. Now: ramp the existing env
			// down to 0 over FADE_OUT_S, then stop the oscillator
			// slightly after the ramp ends. The new voice (created
			// below) starts at `when` with its own attack ramp from
			// 0 — old and new sum during the overlap, with old's
			// downward ramp + new's upward ramp crossing near zero
			// gain, so there's no audible discontinuity at the
			// retrigger.
			const FADE_OUT_S = 0.015;
			try {
				const currentGain = existing.env.gain.value;
				existing.env.gain.cancelScheduledValues(when);
				existing.env.gain.setValueAtTime(currentGain, when);
				existing.env.gain.linearRampToValueAtTime(0, when + FADE_OUT_S);
				existing.osc.stop(when + FADE_OUT_S + 0.005);
			} catch {
				// already stopped
			}
		} else {
			// Legacy hard-stop retrigger. Cheaper, but clicks audibly
			// on retrigger because the osc cuts mid-cycle. Exposed as
			// a toggle for cases where the user prefers the snappier
			// attack at the cost of the click.
			try {
				existing.osc.stop(when + 0.001);
			} catch {
				// already stopped
			}
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

	const peak = VOICE_GAIN * (velocity / 127);
	env.gain.setValueAtTime(0, when);
	env.gain.linearRampToValueAtTime(peak, when + attackS);
	env.gain.linearRampToValueAtTime(peak * sustainLevel, when + attackS + decayS);

	osc.start(when);

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

/** Note-off with optional precise scheduling. See `noteOn` for the
 *  rationale; same `scheduleAt` semantics. */
export function noteOff(midi: number, scheduleAt?: number) {
	if (!ctx) return;
	const v = voices.get(midi);
	if (!v) return;
	// Same NaN/Infinity guard as noteOn — keep both setters symmetric.
	const safeScheduleAt =
		typeof scheduleAt === 'number' && Number.isFinite(scheduleAt) ? scheduleAt : ctx.currentTime;
	const when = Math.max(ctx.currentTime, safeScheduleAt);
	const current = v.env.gain.value;
	v.env.gain.cancelScheduledValues(when);
	v.env.gain.setValueAtTime(current, when);
	v.env.gain.linearRampToValueAtTime(0, when + releaseS);
	try {
		v.osc.stop(when + releaseS + 0.02);
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

// === Audio-clock accessors — expose AudioContext properties to the
//     Companion tick loop so it advances the transport on the same
//     clock the synth uses (eliminates the performance.now() drift
//     and the hardcoded-48k sample-rate bug).
//
// `getAudioContext()` lazy-initializes via `ensureAudio()` — safe
// to call before any noteOn. Returns null only if Web Audio is
// completely unavailable. Read sampleRate / currentTime off the
// returned context.

export function getAudioContext(): AudioContext | null {
	return ensureAudio();
}

// === Imperative setters — update internal state + live audio nodes.

export function setWaveform(w: Waveform) {
	currentWaveform = w;
	for (const v of voices.values()) v.osc.type = w;
}
export function getWaveform(): Waveform {
	return currentWaveform;
}

/** Toggle voice-retrigger crossfade. See `crossfadeRetrigger` const
 *  docblock for the trade-off. UI surface exposes this on the Chain
 *  / synth panel. */
export function setCrossfadeRetrigger(on: boolean) {
	crossfadeRetrigger = !!on;
}
export function getCrossfadeRetrigger(): boolean {
	return crossfadeRetrigger;
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
