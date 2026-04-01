/**
 * Autocorrelation-based Pitch Detector
 *
 * Detects the fundamental frequency of an audio signal using
 * normalized autocorrelation (McLeod Pitch Method simplified).
 * Designed for monophonic guitar input.
 */

/** Result of a single pitch detection frame. */
export interface PitchResult {
	/** Detected fundamental frequency in Hz. */
	frequency: number;
	/** Confidence of the detection, 0..1. */
	clarity: number;
}

/** MIDI note information derived from a frequency. */
export interface MidiNoteInfo {
	/** MIDI note number (0-127). */
	note: number;
	/** Deviation from exact pitch in cents (-50..+50). */
	cents: number;
}

const NOTE_NAMES = ['C', 'C#', 'D', 'Eb', 'E', 'F', 'F#', 'G', 'Ab', 'A', 'Bb', 'B'];

/**
 * Detect the pitch of a mono audio buffer using normalized autocorrelation.
 *
 * Returns null if no clear pitch is found (noise, silence, polyphonic content).
 *
 * @param buffer  - Float32Array of audio samples (mono)
 * @param sampleRate - Sample rate in Hz (e.g. 44100, 48000)
 * @param clarityThreshold - Minimum clarity to accept a detection (default 0.9)
 */
export function detectPitch(
	buffer: Float32Array,
	sampleRate: number,
	clarityThreshold = 0.9
): PitchResult | null {
	const size = buffer.length;

	// Quick RMS check — skip silent buffers
	let rmsSum = 0;
	for (let i = 0; i < size; i++) {
		rmsSum += buffer[i] * buffer[i];
	}
	const rms = Math.sqrt(rmsSum / size);
	if (rms < 0.01) return null;

	// --- Normalized autocorrelation ---
	// Guitar fundamental range: ~82 Hz (E2) to ~1319 Hz (E6)
	// At 44100 Hz: period for 1319 Hz = 33 samples, for 82 Hz = 538 samples
	const minPeriod = Math.floor(sampleRate / 1400); // above highest expected
	const maxPeriod = Math.ceil(sampleRate / 60);     // below lowest expected

	// Compute autocorrelation for each lag
	let bestCorrelation = -1;
	let bestPeriod = -1;

	// Use normalized autocorrelation (NSDF-like)
	for (let lag = minPeriod; lag <= maxPeriod && lag < size; lag++) {
		let correlation = 0;
		let norm1 = 0;
		let norm2 = 0;
		const windowSize = size - lag;

		for (let i = 0; i < windowSize; i++) {
			correlation += buffer[i] * buffer[i + lag];
			norm1 += buffer[i] * buffer[i];
			norm2 += buffer[i + lag] * buffer[i + lag];
		}

		const normFactor = Math.sqrt(norm1 * norm2);
		if (normFactor > 0) {
			correlation /= normFactor;
		} else {
			correlation = 0;
		}

		if (correlation > bestCorrelation) {
			bestCorrelation = correlation;
			bestPeriod = lag;
		}
	}

	if (bestPeriod < 0 || bestCorrelation < clarityThreshold) {
		return null;
	}

	// --- Parabolic interpolation for sub-sample accuracy ---
	let refinedPeriod = bestPeriod;
	if (bestPeriod > minPeriod && bestPeriod < maxPeriod && bestPeriod < size - 1) {
		const corrPrev = computeNormalizedCorrelation(buffer, bestPeriod - 1);
		const corrNext = computeNormalizedCorrelation(buffer, bestPeriod + 1);
		const corrCurr = bestCorrelation;

		const denom = 2 * (2 * corrCurr - corrPrev - corrNext);
		if (Math.abs(denom) > 1e-10) {
			const shift = (corrPrev - corrNext) / denom;
			refinedPeriod = bestPeriod + shift;
		}
	}

	const frequency = sampleRate / refinedPeriod;

	return {
		frequency,
		clarity: bestCorrelation
	};
}

/** Helper: compute normalized correlation at a specific lag. */
function computeNormalizedCorrelation(buffer: Float32Array, lag: number): number {
	const size = buffer.length;
	if (lag >= size || lag < 0) return 0;

	let correlation = 0;
	let norm1 = 0;
	let norm2 = 0;
	const windowSize = size - lag;

	for (let i = 0; i < windowSize; i++) {
		correlation += buffer[i] * buffer[i + lag];
		norm1 += buffer[i] * buffer[i];
		norm2 += buffer[i + lag] * buffer[i + lag];
	}

	const normFactor = Math.sqrt(norm1 * norm2);
	return normFactor > 0 ? correlation / normFactor : 0;
}

/**
 * Convert a frequency in Hz to the nearest MIDI note number and cent offset.
 *
 * Uses A4 = 440 Hz standard tuning. MIDI note 69 = A4.
 */
export function frequencyToMidi(freq: number): MidiNoteInfo {
	// MIDI note = 69 + 12 * log2(freq / 440)
	const midiFloat = 69 + 12 * Math.log2(freq / 440);
	const note = Math.round(midiFloat);
	const cents = Math.round((midiFloat - note) * 100);

	return {
		note: Math.max(0, Math.min(127, note)),
		cents
	};
}

/**
 * Convert a MIDI note number to a human-readable note name with octave.
 *
 * Example: 60 -> "C4", 69 -> "A4", 40 -> "E2"
 */
export function midiToNoteName(midi: number): string {
	const noteIndex = ((midi % 12) + 12) % 12;
	const octave = Math.floor(midi / 12) - 1;
	return `${NOTE_NAMES[noteIndex]}${octave}`;
}
