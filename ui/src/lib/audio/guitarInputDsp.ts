/**
 * Guitar Input DSP Pipeline — TypeScript port of Rust `src/audio/guitar_input.rs`
 *
 * Complete DSP pipeline: onset detection, note state machine, pitch detection
 * with multi-frame voting, octave error correction, string identification via
 * inharmonicity, bend/legato/slide/vibrato detection, and MPE MIDI output.
 */

import { detectPitch, frequencyToMidi } from './pitchDetector';

// ── Constants ─────────────────────────────────────────────────────

/** Standard tuning open-string MIDI notes: E2, A2, D3, G3, B3, E4 */
const STRING_BASE_PITCH: readonly number[] = [40, 45, 50, 55, 59, 64];

/** String display names */
const STRING_NAMES: readonly string[] = ['Low E', 'A', 'D', 'G', 'B', 'High E'];

// ── Types ─────────────────────────────────────────────────────────

/** Note state machine states */
export enum NoteState {
	Idle = 'Idle',
	Attack = 'Attack',
	Sustain = 'Sustain',
	Decay = 'Decay'
}

/** MIDI events produced by the pipeline */
export type MidiEvent =
	| { type: 'note_on'; channel: number; note: number; velocity: number }
	| { type: 'note_off'; channel: number; note: number; velocity: number }
	| { type: 'pitch_bend'; channel: number; cents: number }
	| { type: 'midi_pitch_bend'; channel: number; value: number }
	| { type: 'cc'; channel: number; controller: number; value: number }
	| { type: 'channel_pressure'; channel: number; pressure: number }
	| { type: 'vibrato_status'; active: boolean; rateHz: number; depthCents: number };

/** Per-string inharmonicity profile measured during calibration */
export interface StringProfile {
	name: string;
	openMidi: number;
	openFrequency: number;
	inharmonicityB: number;
	inharmonicityBFret5?: number;
	inharmonicityBFret12?: number;
	harmonicRatios: number[];
}

/** Full guitar calibration data */
export interface GuitarCalibration {
	noiseFloor: number;
	signalPeak: number;
	stringProfiles: StringProfile[];
}

/** A fully resolved note detection */
export interface DetectedNote {
	midiNote: number;
	frequency: number;
	stringIdx: number | null;
	fret: number | null;
	stringConfidence: number;
	velocity: number;
	bendCents: number;
}

/** User-adjustable parameters for the guitar input pipeline */
export interface GuitarInputConfig {
	bufferSize: number;
	sampleRate: number;
	onsetThreshold: number;
	stringConfidenceMin: number;
	bendsEnabled: boolean;
	legatoEnabled: boolean;
	slidesEnabled: boolean;
	vibratoDetection: boolean;
	vibratoPassthrough: boolean;
	filterEnabled: boolean;
	minClarity: number;
	cooldownSamples: number;
	nHarmonics: number;
	inputGain: number;
	fluxThreshold: number;
	perStringChannels: boolean;
	pitchBendRange: number;
	pressureEnabled: boolean;
	pressureHold: number;
	brightnessEnabled: boolean;
}

/** Default configuration */
export function defaultGuitarInputConfig(): GuitarInputConfig {
	return {
		bufferSize: 1024,
		sampleRate: 48000,
		onsetThreshold: 0.02,
		stringConfidenceMin: 0.5,
		bendsEnabled: true,
		legatoEnabled: true,
		slidesEnabled: true,
		vibratoDetection: true,
		vibratoPassthrough: true,
		filterEnabled: false,
		minClarity: 0.40,
		cooldownSamples: 4800, // 100ms @ 48kHz
		nHarmonics: 6,
		inputGain: 1.0,
		fluxThreshold: 0.5,
		perStringChannels: true,
		pitchBendRange: 2,
		pressureEnabled: true,
		pressureHold: 0.3,
		brightnessEnabled: true
	};
}

// ── Utility Functions ─────────────────────────────────────────────

/** Compute RMS of an audio buffer */
export function computeRms(samples: Float32Array | number[]): number {
	const len = samples.length;
	if (len === 0) return 0;
	let sumSq = 0;
	for (let i = 0; i < len; i++) {
		const s = samples[i];
		sumSq += s * s;
	}
	return Math.sqrt(sumSq / len);
}

/** Convert frequency to MIDI note and cents offset */
export function freqToMidi(freq: number): [number, number] {
	if (freq <= 0) return [0, 0];
	const midiFloat = 69 + 12 * Math.log2(freq / 440);
	const midiNote = Math.round(midiFloat);
	const cents = Math.round((midiFloat - midiNote) * 100);
	return [Math.max(0, Math.min(127, midiNote)), Math.max(-50, Math.min(50, cents))];
}

/** Convert MIDI note to frequency */
export function midiToFreq(midi: number): number {
	return 440 * Math.pow(2, (midi - 69) / 12);
}

/** Single-frequency magnitude estimation via the Goertzel algorithm */
export function goertzel(samples: Float32Array | number[], sampleRate: number, targetFreq: number): number {
	const n = samples.length;
	if (n === 0) return 0;
	const k = Math.round(0.5 + (n * targetFreq) / sampleRate);
	const w = (2 * Math.PI * k) / n;
	const coeff = 2 * Math.cos(w);
	let s1 = 0;
	let s2 = 0;
	for (let i = 0; i < n; i++) {
		const s0 = coeff * s1 - s2 + samples[i];
		s2 = s1;
		s1 = s0;
	}
	return Math.sqrt(s1 * s1 + s2 * s2 - coeff * s1 * s2);
}

/** Measure magnitudes of the first N harmonics of a fundamental frequency */
export function measureHarmonics(
	audio: Float32Array | number[],
	fundamental: number,
	nHarmonics: number,
	sampleRate: number
): number[] {
	const magnitudes: number[] = [];
	for (let n = 1; n <= nHarmonics; n++) {
		const target = fundamental * n;
		const lo = target * 0.99;
		const hi = target * 1.01;
		const steps = 5;
		let bestMag = 0;
		for (let i = 0; i <= steps; i++) {
			const freq = lo + ((hi - lo) * i) / steps;
			const mag = goertzel(audio, sampleRate, freq);
			if (mag > bestMag) bestMag = mag;
		}
		magnitudes.push(bestMag);
	}
	return magnitudes;
}

/** Compute the inharmonicity B coefficient from audio */
export function computeInharmonicityB(
	audio: Float32Array | number[],
	fundamental: number,
	sampleRate: number
): number {
	let bSum = 0;
	let count = 0;

	for (let n = 2; n <= 6; n++) {
		const expected = fundamental * n;
		const lo = expected * 0.97;
		const hi = expected * 1.03;
		const steps = 20;
		let bestFreq = expected;
		let bestMag = 0;
		for (let i = 0; i <= steps; i++) {
			const freq = lo + ((hi - lo) * i) / steps;
			const mag = goertzel(audio, sampleRate, freq);
			if (mag > bestMag) {
				bestMag = mag;
				bestFreq = freq;
			}
		}

		if (bestMag > 0) {
			const ratio = bestFreq / (n * fundamental);
			const bN = (ratio * ratio - 1) / (n * n);
			if (bN >= 0 && bN < 0.1) {
				bSum += bN;
				count++;
			}
		}
	}

	return count > 0 ? bSum / count : 0;
}

/** Compute spectral centroid from harmonic frequency/magnitude pairs */
export function computeSpectralCentroid(harmonics: [number, number][]): number {
	let totalMag = 0;
	let weightedSum = 0;
	for (const [freq, mag] of harmonics) {
		totalMag += mag;
		weightedSum += freq * mag;
	}
	return totalMag > 0 ? weightedSum / totalMag : 0;
}

/** Map spectral centroid to CC74 brightness value (0-127) */
export function centroidToCc74(centroid: number): number {
	if (centroid <= 0) return 0;
	const logLow = Math.log2(200);
	const logHigh = Math.log2(2000);
	const logCent = Math.log2(centroid);
	const normalized = clamp((logCent - logLow) / (logHigh - logLow), 0, 1);
	return Math.floor(normalized * 127);
}

/** Half-wave rectified spectral flux between two spectra */
export function spectralFlux(prev: number[], curr: number[]): number {
	let sum = 0;
	const len = Math.min(prev.length, curr.length);
	for (let i = 0; i < len; i++) {
		const diff = curr[i] - prev[i];
		if (diff > 0) sum += diff * diff;
	}
	return Math.sqrt(sum);
}

/** Convert cents offset to 14-bit MIDI pitch bend value */
export function centsToMidiPitchBend(cents: number, rangeSemitones: number): number {
	const rangeCents = rangeSemitones * 100;
	const normalized = clamp(cents / rangeCents, -1, 1);
	return Math.round(clamp(normalized * 8191 + 8192, 0, 16383));
}

/** Convert RMS to MIDI velocity (fallback, no calibration) */
export function rmsToVelocity(rms: number): number {
	const normalized = clamp((rms - 0.01) / 0.49, 0, 1);
	const vel = 30 + normalized * 97;
	return clamp(Math.round(vel), 1, 127);
}

/** Convert RMS to MIDI velocity using calibrated range and power curve (gamma 0.5) */
export function rmsToVelocityCalibrated(rms: number, noiseFloor: number, signalPeak: number): number {
	const floor = Math.max(noiseFloor, 0.001);
	const ceiling = Math.max(signalPeak, floor + 0.01);
	const normalized = clamp((rms - floor) / (ceiling - floor), 0, 1);
	const curved = Math.pow(normalized, 0.5);
	const vel = 1 + curved * 126;
	return clamp(Math.round(vel), 1, 127);
}

/** Check if a MIDI note is physically playable on a given string (frets 0-22) */
export function isPlausible(midiNote: number, stringIdx: number): boolean {
	if (stringIdx >= STRING_BASE_PITCH.length) return false;
	const open = STRING_BASE_PITCH[stringIdx];
	return midiNote >= open && midiNote <= open + 22;
}

/** Plausibility score for assigning a MIDI note to a string */
export function plausibilityScore(
	midiNote: number,
	stringIdx: number,
	prevNote: DetectedNote | null
): number {
	if (!isPlausible(midiNote, stringIdx)) return 0;
	const open = STRING_BASE_PITCH[stringIdx];
	const fret = midiNote - open;
	let score = 1.0;
	score -= fret * 0.01;
	if (prevNote && prevNote.fret != null) {
		const fretDistance = Math.abs(fret - prevNote.fret);
		if (fretDistance <= 5) score += 0.2;
	}
	return score;
}

/** Simple heuristic string/fret identification (no calibration needed) */
export function simpleStringFret(midiNote: number): [number, number] {
	for (let idx = STRING_BASE_PITCH.length - 1; idx >= 0; idx--) {
		const open = STRING_BASE_PITCH[idx];
		if (midiNote >= open && midiNote <= open + 22) {
			return [idx, midiNote - open];
		}
	}
	return [0, Math.max(0, midiNote - STRING_BASE_PITCH[0])];
}

/** Interpolate the expected inharmonicity B for a fret position */
function interpolateB(profile: StringProfile, fret: number): number {
	const bOpen = profile.inharmonicityB;
	const b5 = profile.inharmonicityBFret5;
	const b12 = profile.inharmonicityBFret12;

	if (b5 != null && b12 != null) {
		if (fret <= 5) {
			const t = fret / 5;
			return bOpen + t * (b5 - bOpen);
		} else if (fret <= 12) {
			const t = (fret - 5) / 7;
			return b5 + t * (b12 - b5);
		} else {
			const slope = (b12 - b5) / 7;
			return b12 + slope * (fret - 12);
		}
	} else if (b5 != null) {
		if (fret <= 5) {
			const t = fret / 5;
			return bOpen + t * (b5 - bOpen);
		} else {
			const slope = (b5 - bOpen) / 5;
			return b5 + slope * (fret - 5);
		}
	} else {
		return bOpen * Math.pow(2, fret / 6);
	}
}

/** Identify string with plausibility filtering and inharmonicity matching */
export function identifyStringWithPlausibility(
	audio: Float32Array | number[],
	fundamental: number,
	midiNote: number,
	profiles: StringProfile[],
	sampleRate: number,
	nHarmonics: number,
	confidenceMin: number,
	prevNote: DetectedNote | null
): [number, number] | null {
	const measuredB = computeInharmonicityB(audio, fundamental, sampleRate);
	const measuredHarmonics = measureHarmonics(audio, fundamental, nHarmonics, sampleRate);
	const h1 = Math.max(measuredHarmonics[0] ?? 1, 1e-10);
	const measuredRatios = measuredHarmonics.slice(1).map((h) => h / h1);

	let bestString: number | null = null;
	let bestConfidence = 0;

	for (let idx = 0; idx < profiles.length; idx++) {
		if (!isPlausible(midiNote, idx)) continue;
		const profile = profiles[idx];
		const fret = midiNote - profile.openMidi;
		const expectedB = interpolateB(profile, fret);

		// B similarity
		let bRatio: number;
		if (measuredB > 0 && expectedB > 0) {
			const logRatio = Math.abs(Math.log(measuredB) - Math.log(expectedB));
			bRatio = Math.exp(-logRatio * 2);
		} else {
			bRatio = 0.3;
		}

		// Harmonic ratio similarity (cosine distance)
		let harmonicSim: number;
		if (profile.harmonicRatios.length > 0 && measuredRatios.length > 0) {
			const n = Math.min(profile.harmonicRatios.length, measuredRatios.length);
			let dot = 0, magA = 0, magB = 0;
			for (let i = 0; i < n; i++) {
				dot += profile.harmonicRatios[i] * measuredRatios[i];
				magA += profile.harmonicRatios[i] * profile.harmonicRatios[i];
				magB += measuredRatios[i] * measuredRatios[i];
			}
			const denom = Math.max(Math.sqrt(magA) * Math.sqrt(magB), 1e-10);
			harmonicSim = clamp(dot / denom, 0, 1);
		} else {
			harmonicSim = 0.5;
		}

		const fretBonus = 1.0 - Math.min(fret * 0.02, 0.3);
		const plaus = plausibilityScore(midiNote, idx, prevNote);
		const confidence = bRatio * 0.4 + harmonicSim * 0.25 + fretBonus * 0.15 + plaus * 0.2;

		if (confidence > bestConfidence && confidence >= confidenceMin) {
			bestConfidence = confidence;
			bestString = idx;
		}
	}

	return bestString != null ? [bestString, bestConfidence] : null;
}

/** Clamp a value to [min, max] */
function clamp(v: number, min: number, max: number): number {
	return v < min ? min : v > max ? max : v;
}

/** Convert latency_ms to buffer_size (nearest power of 2, clamped [256, 4096]) */
export function bufferSizeForLatency(latencyMs: number, sampleRate: number): number {
	const samples = (latencyMs / 1000) * sampleRate;
	let p = 1;
	while (p < samples) p *= 2;
	return Math.max(256, Math.min(4096, p));
}

// ── Main Pipeline Class ───────────────────────────────────────────

export class GuitarInputDsp {
	readonly config: GuitarInputConfig;
	private calibration: GuitarCalibration | null = null;

	// State
	private currentNote: DetectedNote | null = null;
	private prevRms = 0;
	private cooldownRemaining = 0;
	private ringBuffer: Float32Array;
	private ringPos = 0;

	// Note state machine
	private _noteState: NoteState = NoteState.Idle;
	private stateSamples = 0;
	private totalSamples = 0;

	// Note-off gating
	private suppressFrequency: number | null = null;
	private suppressUntil = 0;

	// Multi-frame pitch voting (3-frame ring buffer)
	private pitchHistory: (number | null)[] = [null, null, null];
	private pitchHistoryIdx = 0;

	// Spectral flux onset
	private prevSpectrum: number[] | null = null;

	// Slide detection
	private slideHistory: number[];
	private slideHistoryIdx = 0;
	private slideVelocityDecay = 0.9;

	// Vibrato detection
	private vibratoBuffer: number[] = [];
	private vibratoBufferIdx = 0;
	private vibratoDetected = false;
	private vibratoRateHz = 0;
	private vibratoDepthCents = 0;

	// Pressure / NoteOff velocity
	private lastSustainRms = 0;
	private lastPressure = 0;
	private noteOnsetRms = 0;
	private lastSentPressure = 0;
	private lastBrightness = 0;

	constructor(config?: Partial<GuitarInputConfig>) {
		this.config = { ...defaultGuitarInputConfig(), ...config };
		this.ringBuffer = new Float32Array(this.config.bufferSize);
		this.slideHistory = new Array(8).fill(0);
	}

	get noteState(): NoteState {
		return this._noteState;
	}

	get currentDetectedNote(): DetectedNote | null {
		return this.currentNote;
	}

	/** Set calibration data */
	setCalibration(cal: GuitarCalibration): void {
		this.calibration = cal;
	}

	/** Get current calibration */
	getCalibration(): GuitarCalibration | null {
		return this.calibration;
	}

	// ── MPE pressure envelope ─────────────────────────────────────

	private computePressure(currentRms: number): number {
		const onset = Math.max(this.noteOnsetRms, 0.001);
		const ratio = clamp(currentRms / onset, 0, 1);
		const held = ratio + (1 - ratio) * this.config.pressureHold;
		const heldClamped = clamp(held, 0, 1);
		const curved = Math.pow(heldClamped, 0.7);
		return Math.floor(curved * 127);
	}

	// ── Main processing ───────────────────────────────────────────

	/** Process a block of mono audio samples and return MIDI events */
	processBlock(audio: Float32Array): MidiEvent[] {
		const events: MidiEvent[] = [];
		const gain = this.config.inputGain;

		for (let si = 0; si < audio.length; si++) {
			const gainedSample = audio[si] * gain;
			this.ringBuffer[this.ringPos] = gainedSample;
			this.ringPos = (this.ringPos + 1) % this.config.bufferSize;
			this.totalSamples++;

			if (this.ringPos === 0) {
				const windowEvents = this.analyzeWindow();
				for (const e of windowEvents) events.push(e);
			}
		}

		return events;
	}

	/** Analyze the current ring buffer contents */
	private analyzeWindow(): MidiEvent[] {
		const events: MidiEvent[] = [];
		const buf = this.linearizeRingBuffer();

		// 1. Compute RMS
		const rms = computeRms(buf);

		// 2. Spectral flux onset detection
		const flux = this.computeSpectralFlux(buf);

		// 3. Onset detection: RMS spike OR spectral flux
		const rmsOnset = this.detectOnset(rms);
		const fluxOnset = flux > this.config.fluxThreshold && this.cooldownRemaining === 0;
		const onset = rmsOnset || fluxOnset;

		// 4. Pitch detection (uses existing autocorrelation-based detector)
		const pitchResult = detectPitch(buf, this.config.sampleRate, this.config.minClarity);

		// Tick down cooldown
		if (this.cooldownRemaining > 0) {
			this.cooldownRemaining = Math.max(0, this.cooldownRemaining - this.config.bufferSize);
		}

		// Advance state timer
		this.stateSamples += this.config.bufferSize;

		// Thresholds
		const sustainThreshold = this.config.onsetThreshold * 0.3;
		const noiseFloor = this.calibration?.noiseFloor ?? 0.001;

		if (pitchResult) {
			const rawFreq = pitchResult.frequency;

			// Octave error correction
			const freq = this.correctOctave(rawFreq);
			const [midiNote, cents] = freqToMidi(freq);

			// Note-off gating: suppress previous note's frequency
			if (this.isSuppressed(freq)) {
				this.prevRms = rms;
				return events;
			}

			// Note state machine transitions
			switch (this._noteState) {
				case NoteState.Idle: {
					if (onset) {
						this._noteState = NoteState.Attack;
						this.stateSamples = 0;
						this.pushPitch(midiNote);
					}
					break;
				}

				case NoteState.Attack: {
					this.pushPitch(midiNote);
					const attackTimeoutSamples = Math.floor(this.config.sampleRate * 0.05);

					if (this.isPitchStable()) {
						const stableMidi = this.pitchHistory[0] ?? midiNote;

						// End previous note
						if (this.currentNote) {
							this.suppressFrequency = this.currentNote.frequency;
							this.suppressUntil =
								this.totalSamples + Math.floor(this.config.sampleRate * 0.1);
							const prevChannel = this.config.perStringChannels
								? (this.currentNote.stringIdx != null ? this.currentNote.stringIdx + 1 : 0)
								: 0;
							events.push({
								type: 'note_off',
								channel: prevChannel,
								note: this.currentNote.midiNote,
								velocity: this.lastSentPressure
							});
						}

						// Velocity from attack peak (~5ms)
						const attackSamples = Math.min(
							Math.floor(this.config.sampleRate * 0.005),
							buf.length
						);
						const attackRms = computeRms(buf.subarray(0, attackSamples));
						const velocity = this.calibration
							? rmsToVelocityCalibrated(attackRms, this.calibration.noiseFloor, this.calibration.signalPeak)
							: rmsToVelocity(attackRms);

						// String identification
						const [stringIdx, fret, stringConf] = this.identifyStringAndFret(
							buf, freq, stableMidi
						);
						const channel = this.config.perStringChannels
							? (stringIdx != null ? stringIdx + 1 : 0)
							: 0;

						const detected: DetectedNote = {
							midiNote: stableMidi,
							frequency: freq,
							stringIdx,
							fret,
							stringConfidence: stringConf,
							velocity,
							bendCents: cents
						};

						// MPE message ordering: PitchBend -> CC74 -> Pressure -> NoteOn
						const initialBend = cents;
						if (Math.abs(initialBend) > 0) {
							events.push({ type: 'pitch_bend', channel, cents: initialBend });
							events.push({
								type: 'midi_pitch_bend',
								channel,
								value: centsToMidiPitchBend(initialBend, this.config.pitchBendRange)
							});
						}

						if (this.config.brightnessEnabled) {
							const cc74 = this.measureBrightness(buf, freq);
							events.push({ type: 'cc', channel, controller: 74, value: cc74 });
							this.lastBrightness = cc74;
						}

						this.noteOnsetRms = attackRms;
						if (this.config.pressureEnabled) {
							const initialPressure = this.computePressure(attackRms);
							events.push({ type: 'channel_pressure', channel, pressure: initialPressure });
							this.lastSentPressure = initialPressure;
							this.lastPressure = initialPressure;
						}

						events.push({
							type: 'note_on',
							channel,
							note: stableMidi,
							velocity
						});

						this.currentNote = detected;
						this.cooldownRemaining = this.config.cooldownSamples;
						this._noteState = NoteState.Sustain;
						this.stateSamples = 0;
						this.lastSustainRms = rms;
						this.clearSlideHistory();
						this.clearVibratoState();
					} else if (this.stateSamples > attackTimeoutSamples) {
						// Attack -> Idle: false trigger
						this._noteState = NoteState.Idle;
						this.stateSamples = 0;
						this.clearPitchHistory();
						this.clearSlideHistory();
						this.clearVibratoState();
					}
					break;
				}

				case NoteState.Sustain: {
					if (onset) {
						// 1. Re-pluck -> Attack
						this._noteState = NoteState.Attack;
						this.stateSamples = 0;
						this.clearPitchHistory();
						this.pushPitch(midiNote);
						this.clearSlideHistory();
						this.clearVibratoState();
					} else if (rms < sustainThreshold) {
						// 2. Sustain -> Decay
						this._noteState = NoteState.Decay;
						this.stateSamples = 0;
						if (this.vibratoDetected) {
							events.push({
								type: 'vibrato_status',
								active: false,
								rateHz: 0,
								depthCents: 0
							});
							this.clearVibratoState();
						}
					} else if (this.currentNote) {
						const noteMidi = this.currentNote.midiNote;
						const noteFreq = this.currentNote.frequency;
						const noteVel = this.currentNote.velocity;
						const noteBend = this.currentNote.bendCents;
						const noteStringIdx = this.currentNote.stringIdx;
						const channel = this.config.perStringChannels
							? (noteStringIdx != null ? noteStringIdx + 1 : 0)
							: 0;

						this.lastSustainRms = rms;

						const semitoneDiff = Math.abs(midiNote - noteMidi);
						const baseFreq = midiToFreq(noteMidi);
						const centsFromBase =
							baseFreq > 0 && freq > 0
								? Math.round(1200 * Math.log2(freq / baseFreq))
								: 0;

						this.pushSlideCents(centsFromBase);
						this.pushVibratoCents(centsFromBase);

						// Detect rapid pitch change
						const prevCents = this.slideHistoryPrevCents();
						const centsPerFrame = Math.abs(centsFromBase - prevCents);
						const isRapidJump = centsPerFrame > 50;

						if (semitoneDiff >= 1 && isRapidJump && this.config.legatoEnabled) {
							// 3. LEGATO
							this.suppressFrequency = noteFreq;
							this.suppressUntil =
								this.totalSamples + Math.floor(this.config.sampleRate * 0.1);

							events.push({
								type: 'note_off',
								channel,
								note: noteMidi,
								velocity: this.lastSentPressure
							});

							const legatoVel = Math.min(Math.round(noteVel * 0.8), 127);
							const [sIdx, fretVal, sConf] = this.identifyStringAndFret(
								buf, freq, midiNote
							);
							const newChannel = this.config.perStringChannels
								? (sIdx != null ? sIdx + 1 : 0)
								: 0;

							const newNote: DetectedNote = {
								midiNote,
								frequency: freq,
								stringIdx: sIdx,
								fret: fretVal,
								stringConfidence: sConf,
								velocity: legatoVel,
								bendCents: 0
							};

							// MPE ordering for legato
							const legatoCents = cents;
							if (Math.abs(legatoCents) > 0) {
								events.push({ type: 'pitch_bend', channel: newChannel, cents: legatoCents });
								events.push({
									type: 'midi_pitch_bend',
									channel: newChannel,
									value: centsToMidiPitchBend(legatoCents, this.config.pitchBendRange)
								});
							}
							if (this.config.brightnessEnabled) {
								const cc74 = this.measureBrightness(buf, freq);
								events.push({ type: 'cc', channel: newChannel, controller: 74, value: cc74 });
								this.lastBrightness = cc74;
							}
							this.noteOnsetRms = rms;
							if (this.config.pressureEnabled) {
								const p = this.computePressure(rms);
								events.push({ type: 'channel_pressure', channel: newChannel, pressure: p });
								this.lastSentPressure = p;
								this.lastPressure = p;
							}

							events.push({
								type: 'note_on',
								channel: newChannel,
								note: midiNote,
								velocity: legatoVel
							});

							this.currentNote = newNote;
							this.clearPitchHistory();
							this.clearSlideHistory();
							this.clearVibratoState();
						} else if (semitoneDiff >= 1 && isRapidJump && !this.config.legatoEnabled) {
							// Rapid jump without legato -> missed onset
							this._noteState = NoteState.Attack;
							this.stateSamples = 0;
							this.clearPitchHistory();
							this.pushPitch(midiNote);
							this.clearSlideHistory();
							this.clearVibratoState();
						} else if (
							Math.abs(centsFromBase) >= 100 &&
							this.config.slidesEnabled &&
							this.isSlideMovement()
						) {
							// 4. SLIDE
							const [newMidi, newCentsVal] = freqToMidi(freq);
							if (newMidi !== noteMidi) {
								events.push({
									type: 'note_off',
									channel,
									note: noteMidi,
									velocity: this.lastSentPressure
								});

								const slideVel = Math.max(
									1,
									Math.min(127, Math.round(noteVel * this.slideVelocityDecay))
								);
								const [sIdx, fretVal, sConf] = this.identifyStringAndFret(
									buf, freq, newMidi
								);
								const newChannel = this.config.perStringChannels
									? (sIdx != null ? sIdx + 1 : 0)
									: 0;

								const newNote: DetectedNote = {
									midiNote: newMidi,
									frequency: freq,
									stringIdx: sIdx,
									fret: fretVal,
									stringConfidence: sConf,
									velocity: slideVel,
									bendCents: 0
								};

								// MPE ordering for slide
								const slideBend = newCentsVal;
								if (Math.abs(slideBend) > 0) {
									events.push({ type: 'pitch_bend', channel: newChannel, cents: slideBend });
									events.push({
										type: 'midi_pitch_bend',
										channel: newChannel,
										value: centsToMidiPitchBend(slideBend, this.config.pitchBendRange)
									});
								}
								if (this.config.brightnessEnabled) {
									const cc74 = this.measureBrightness(buf, freq);
									events.push({ type: 'cc', channel: newChannel, controller: 74, value: cc74 });
									this.lastBrightness = cc74;
								}
								this.noteOnsetRms = rms;
								if (this.config.pressureEnabled) {
									const p = this.computePressure(rms);
									events.push({ type: 'channel_pressure', channel: newChannel, pressure: p });
									this.lastSentPressure = p;
									this.lastPressure = p;
								}

								events.push({
									type: 'note_on',
									channel: newChannel,
									note: newMidi,
									velocity: slideVel
								});

								this.currentNote = newNote;
								this.clearSlideHistory();
							}
						} else {
							// 5. VIBRATO detection
							const vibratoResult = this.config.vibratoDetection
								? this.detectVibrato()
								: null;

							if (this.config.vibratoDetection) {
								if (vibratoResult) {
									const [rateHz, amplitude] = vibratoResult;
									if (!this.vibratoDetected) {
										this.vibratoDetected = true;
										this.vibratoRateHz = rateHz;
										this.vibratoDepthCents = amplitude;
										events.push({
											type: 'vibrato_status',
											active: true,
											rateHz,
											depthCents: amplitude
										});
									} else if (
										Math.abs(rateHz - this.vibratoRateHz) > 0.5 ||
										Math.abs(amplitude - this.vibratoDepthCents) > 5
									) {
										this.vibratoRateHz = rateHz;
										this.vibratoDepthCents = amplitude;
										events.push({
											type: 'vibrato_status',
											active: true,
											rateHz,
											depthCents: amplitude
										});
									}

									if (!this.config.vibratoPassthrough) {
										// Suppress pitch bend (smooth out vibrato)
										if (noteBend !== 0) {
											events.push({ type: 'pitch_bend', channel, cents: 0 });
											events.push({
												type: 'midi_pitch_bend',
												channel,
												value: centsToMidiPitchBend(0, this.config.pitchBendRange)
											});
											if (this.currentNote) this.currentNote.bendCents = 0;
										}
									} else if (this.config.bendsEnabled) {
										// Vibrato passthrough: let bend events flow
										this.emitBendEvents(events, channel, centsFromBase, noteBend, freq);
									}
								} else {
									// No vibrato detected
									if (this.vibratoDetected) {
										events.push({
											type: 'vibrato_status',
											active: false,
											rateHz: 0,
											depthCents: 0
										});
										this.vibratoDetected = false;
										this.vibratoRateHz = 0;
										this.vibratoDepthCents = 0;
									}
									// 6. BEND
									if (this.config.bendsEnabled) {
										this.emitBendEvents(events, channel, centsFromBase, noteBend, freq);
									}
								}
							} else {
								// Vibrato detection disabled; just do bends
								if (this.config.bendsEnabled) {
									this.emitBendEvents(events, channel, centsFromBase, noteBend, freq);
								}
							}

							// Shaped pressure envelope during Sustain
							if (this.config.pressureEnabled) {
								const pressure = this.computePressure(rms);
								if (pressure !== this.lastPressure) {
									events.push({ type: 'channel_pressure', channel, pressure });
									this.lastPressure = pressure;
									this.lastSentPressure = pressure;
								}
							}

							// CC74 brightness updates during Sustain
							if (this.config.brightnessEnabled) {
								const cc74 = this.measureBrightness(buf, freq);
								if (Math.abs(cc74 - this.lastBrightness) > 2) {
									events.push({ type: 'cc', channel, controller: 74, value: cc74 });
									this.lastBrightness = cc74;
								}
							}
						}
					}
					break;
				}

				case NoteState.Decay: {
					const decayTimeoutSamples = Math.floor(this.config.sampleRate * 0.5);

					if (onset) {
						this._noteState = NoteState.Attack;
						this.stateSamples = 0;
						this.clearPitchHistory();
						this.pushPitch(midiNote);
					} else if (rms < noiseFloor * 2 || this.stateSamples > decayTimeoutSamples) {
						if (this.currentNote) {
							const decayChannel = this.config.perStringChannels
								? (this.currentNote.stringIdx != null
									? this.currentNote.stringIdx + 1
									: 0)
								: 0;
							events.push({
								type: 'note_off',
								channel: decayChannel,
								note: this.currentNote.midiNote,
								velocity: this.lastSentPressure
							});
							this.currentNote = null;
						}
						this._noteState = NoteState.Idle;
						this.stateSamples = 0;
						this.clearPitchHistory();
					}
					break;
				}
			}
		} else {
			// No pitch detected
			switch (this._noteState) {
				case NoteState.Attack:
					this._noteState = NoteState.Idle;
					this.stateSamples = 0;
					this.clearPitchHistory();
					break;
				case NoteState.Sustain:
				case NoteState.Decay:
					if (rms < noiseFloor * 2) {
						if (this.currentNote) {
							const ch = this.config.perStringChannels
								? (this.currentNote.stringIdx != null
									? this.currentNote.stringIdx + 1
									: 0)
								: 0;
							events.push({
								type: 'note_off',
								channel: ch,
								note: this.currentNote.midiNote,
								velocity: this.lastSentPressure
							});
							this.currentNote = null;
						}
						this._noteState = NoteState.Idle;
						this.stateSamples = 0;
						this.clearPitchHistory();
					}
					break;
				case NoteState.Idle:
					break;
			}
		}

		this.prevRms = rms;
		return events;
	}

	// ── Onset detection ───────────────────────────────────────────

	private detectOnset(rms: number): boolean {
		if (this.cooldownRemaining > 0) return false;
		return rms > this.config.onsetThreshold && rms > this.prevRms * 2;
	}

	// ── Octave error correction ───────────────────────────────────

	private correctOctave(detectedFreq: number): number {
		if (this.currentNote) {
			const ratio = detectedFreq / this.currentNote.frequency;
			if (Math.abs(ratio - 2) < 0.05) return detectedFreq / 2;
			if (Math.abs(ratio - 0.5) < 0.05) return detectedFreq * 2;
		}
		return detectedFreq;
	}

	// ── Note-off gating ───────────────────────────────────────────

	private isSuppressed(freq: number): boolean {
		if (this.totalSamples >= this.suppressUntil) return false;
		if (this.suppressFrequency == null) return false;
		const ratio = freq / this.suppressFrequency;
		const semitoneRatio = Math.pow(2, 1 / 12);
		return ratio >= 1 / semitoneRatio && ratio <= semitoneRatio;
	}

	// ── Multi-frame pitch voting ──────────────────────────────────

	private pushPitch(midiNote: number): void {
		this.pitchHistory[this.pitchHistoryIdx] = midiNote;
		this.pitchHistoryIdx = (this.pitchHistoryIdx + 1) % 3;
	}

	private isPitchStable(): boolean {
		const first = this.pitchHistory[0];
		return first != null && this.pitchHistory.every((p) => p === first);
	}

	private clearPitchHistory(): void {
		this.pitchHistory = [null, null, null];
		this.pitchHistoryIdx = 0;
	}

	// ── Slide detection ───────────────────────────────────────────

	private pushSlideCents(cents: number): void {
		this.slideHistory[this.slideHistoryIdx] = cents;
		this.slideHistoryIdx = (this.slideHistoryIdx + 1) % this.slideHistory.length;
	}

	private slideHistoryPrevCents(): number {
		const len = this.slideHistory.length;
		if (len === 0) return 0;
		const prevIdx = this.slideHistoryIdx === 0 ? len - 1 : this.slideHistoryIdx - 1;
		return this.slideHistory[prevIdx];
	}

	private isSlideMovement(): boolean {
		const len = this.slideHistory.length;
		if (len < 3) return false;

		const ordered: number[] = [];
		for (let i = 0; i < len; i++) {
			ordered.push(this.slideHistory[(this.slideHistoryIdx + i) % len]);
		}

		const checkLen = Math.min(5, len);
		const recent = ordered.slice(len - checkLen);

		for (let i = 0; i < recent.length - 1; i++) {
			if (Math.abs(recent[i + 1] - recent[i]) > 50) return false;
		}

		let increasing = 0;
		let decreasing = 0;
		for (let i = 0; i < recent.length - 1; i++) {
			const diff = recent[i + 1] - recent[i];
			if (diff > 2) increasing++;
			else if (diff < -2) decreasing++;
		}

		return (
			(increasing >= checkLen - 2 && decreasing === 0) ||
			(decreasing >= checkLen - 2 && increasing === 0)
		);
	}

	private clearSlideHistory(): void {
		this.slideHistory.fill(0);
		this.slideHistoryIdx = 0;
	}

	// ── Vibrato detection ─────────────────────────────────────────

	private pushVibratoCents(cents: number): void {
		const cap = 24;
		if (this.vibratoBuffer.length < cap) {
			this.vibratoBuffer.push(cents);
		} else {
			this.vibratoBuffer[this.vibratoBufferIdx % cap] = cents;
		}
		this.vibratoBufferIdx++;
	}

	private detectVibrato(): [number, number] | null {
		const buf = this.vibratoBuffer;
		if (buf.length < 10) return null;

		const len = buf.length;
		const cap = Math.min(24, len);
		let ordered: number[];
		if (len < 24) {
			ordered = [...buf];
		} else {
			const start = this.vibratoBufferIdx % cap;
			ordered = [];
			for (let i = 0; i < cap; i++) {
				ordered.push(buf[(start + i) % cap]);
			}
		}

		const mean = ordered.reduce((a, b) => a + b, 0) / ordered.length;

		let crossings = 0;
		for (let i = 0; i < ordered.length - 1; i++) {
			if (Math.sign(ordered[i] - mean) !== Math.sign(ordered[i + 1] - mean)) {
				crossings++;
			}
		}

		const frameDuration = this.config.bufferSize / this.config.sampleRate;
		const durationSecs = ordered.length * frameDuration;
		if (durationSecs < 0.05) return null;

		const rateHz = crossings / (2 * durationSecs);
		if (rateHz < 3 || rateHz > 8) return null;

		let amplitude = 0;
		for (const c of ordered) {
			amplitude = Math.max(amplitude, Math.abs(c - mean));
		}
		if (amplitude < 10 || amplitude > 60) return null;

		return [rateHz, amplitude];
	}

	private clearVibratoState(): void {
		this.vibratoBuffer = [];
		this.vibratoBufferIdx = 0;
		this.vibratoDetected = false;
		this.vibratoRateHz = 0;
		this.vibratoDepthCents = 0;
	}

	// ── Spectral flux onset detection ─────────────────────────────

	private computeSpectralFlux(audio: Float32Array): number {
		const freqs: number[] = [];
		for (let i = 0; i < 24; i++) {
			freqs.push(80 * Math.pow(2, i / 6));
		}

		const currSpectrum = freqs.map((f) => goertzel(audio, this.config.sampleRate, f));

		const flux = this.prevSpectrum ? spectralFlux(this.prevSpectrum, currSpectrum) : 0;
		this.prevSpectrum = currSpectrum;
		return flux;
	}

	// ── String identification ─────────────────────────────────────

	private identifyStringAndFret(
		audio: Float32Array,
		fundamental: number,
		midiNote: number
	): [number | null, number | null, number] {
		if (this.calibration && this.calibration.stringProfiles.length === 6) {
			const result = identifyStringWithPlausibility(
				audio,
				fundamental,
				midiNote,
				this.calibration.stringProfiles,
				this.config.sampleRate,
				this.config.nHarmonics,
				this.config.stringConfidenceMin,
				this.currentNote
			);
			if (result) {
				const [stringIdx, confidence] = result;
				const profile = this.calibration.stringProfiles[stringIdx];
				const fret =
					midiNote >= profile.openMidi ? midiNote - profile.openMidi : null;
				return [stringIdx, fret, confidence];
			}
		}
		// Fallback: simple heuristic
		const [s, f] = simpleStringFret(midiNote);
		return [s, f, 0.3];
	}

	// ── Brightness measurement helper ─────────────────────────────

	private measureBrightness(audio: Float32Array, freq: number): number {
		const harmonicsData = measureHarmonics(
			audio, freq, this.config.nHarmonics, this.config.sampleRate
		);
		const harmonicPairs: [number, number][] = harmonicsData.map((mag, i) => [
			freq * (i + 1),
			mag
		]);
		const centroid = computeSpectralCentroid(harmonicPairs);
		return centroidToCc74(centroid);
	}

	// ── Bend emission helper ──────────────────────────────────────

	private emitBendEvents(
		events: MidiEvent[],
		channel: number,
		centsFromBase: number,
		noteBend: number,
		freq: number
	): void {
		if (Math.abs(centsFromBase) > 5 && Math.abs(centsFromBase) < 200) {
			if (Math.abs(centsFromBase - noteBend) > 3) {
				events.push({ type: 'pitch_bend', channel, cents: centsFromBase });
				events.push({
					type: 'midi_pitch_bend',
					channel,
					value: centsToMidiPitchBend(centsFromBase, this.config.pitchBendRange)
				});
				if (this.currentNote) {
					this.currentNote.bendCents = centsFromBase;
					this.currentNote.frequency = freq;
				}
			}
		} else if (Math.abs(centsFromBase) <= 5 && noteBend !== 0) {
			events.push({ type: 'pitch_bend', channel, cents: 0 });
			events.push({
				type: 'midi_pitch_bend',
				channel,
				value: centsToMidiPitchBend(0, this.config.pitchBendRange)
			});
			if (this.currentNote) {
				this.currentNote.bendCents = 0;
			}
		}
	}

	// ── Ring buffer ───────────────────────────────────────────────

	private linearizeRingBuffer(): Float32Array {
		const buf = new Float32Array(this.config.bufferSize);
		for (let i = 0; i < this.config.bufferSize; i++) {
			buf[i] = this.ringBuffer[(this.ringPos + i) % this.config.bufferSize];
		}
		return buf;
	}

	// ── Calibration helpers ───────────────────────────────────────

	/** Measure the noise floor from a silence capture */
	static measureNoiseFloor(audio: Float32Array): number {
		return computeRms(audio);
	}

	/** Compute optimal input gain from calibration peak measurement */
	static computeInputGain(peakPluckLevel: number): number {
		if (peakPluckLevel <= 0) return 1;
		return clamp(0.8 / peakPluckLevel, 0.1, 2.0);
	}
}
