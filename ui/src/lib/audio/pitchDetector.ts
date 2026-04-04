/**
 * Pitch Detector — Normalized Square Difference Function (McLeod method)
 *
 * Detects the fundamental frequency of a monophonic audio signal.
 * Uses the NSDF with first-peak-above-threshold selection to avoid
 * octave errors common with global-max autocorrelation approaches.
 *
 * Reference: McLeod & Wyvill, "A Smarter Way to Find Pitch" (2005)
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
 * Detect the pitch of a mono audio buffer using the NSDF (McLeod method).
 *
 * The key difference from naive autocorrelation: instead of picking the
 * global maximum correlation, we find zero-crossings of the NSDF and
 * pick the FIRST peak that exceeds `clarityThreshold`. This avoids
 * locking onto harmonics (octave-down errors).
 *
 * @param buffer  - Float32Array of audio samples (mono)
 * @param sampleRate - Sample rate in Hz
 * @param clarityThreshold - Minimum NSDF peak to accept (default 0.7)
 */
export function detectPitch(
	buffer: Float32Array,
	sampleRate: number,
	clarityThreshold = 0.7
): PitchResult | null {
	const size = buffer.length;

	// Quick RMS check — skip silent buffers
	let rmsSum = 0;
	for (let i = 0; i < size; i++) {
		rmsSum += buffer[i] * buffer[i];
	}
	const rms = Math.sqrt(rmsSum / size);
	if (rms < 0.01) return null;

	// Guitar fundamental range: ~82 Hz (E2) to ~1319 Hz (E6)
	const minPeriod = Math.floor(sampleRate / 1400);
	const maxPeriod = Math.ceil(sampleRate / 60);
	const searchMax = Math.min(maxPeriod, size - 1);

	// Compute the Normalized Square Difference Function (NSDF)
	// NSDF(tau) = 2 * r(tau) / (m(0) + m(tau))
	// where r(tau) is autocorrelation and m(tau) = sum of squared samples
	const nsdf = new Float32Array(searchMax + 1);

	for (let tau = minPeriod; tau <= searchMax; tau++) {
		let acf = 0;  // autocorrelation
		let m = 0;    // energy normalization term
		const windowSize = size - tau;

		for (let i = 0; i < windowSize; i++) {
			acf += buffer[i] * buffer[i + tau];
			m += buffer[i] * buffer[i] + buffer[i + tau] * buffer[i + tau];
		}

		nsdf[tau] = m > 0 ? (2 * acf) / m : 0;
	}

	// Find peaks by looking for positive zero crossings, then the max in each positive lobe.
	// Pick the FIRST peak above the threshold — this is the fundamental, not a harmonic.
	let bestPeriod = -1;
	let bestClarity = -1;

	let inPositiveLobe = false;
	let lobePeakVal = -1;
	let lobePeakIdx = -1;

	for (let tau = minPeriod; tau <= searchMax; tau++) {
		if (nsdf[tau] > 0) {
			if (!inPositiveLobe) {
				// Entering a new positive lobe
				inPositiveLobe = true;
				lobePeakVal = nsdf[tau];
				lobePeakIdx = tau;
			} else if (nsdf[tau] > lobePeakVal) {
				lobePeakVal = nsdf[tau];
				lobePeakIdx = tau;
			}
		} else if (inPositiveLobe) {
			// Leaving a positive lobe — check if its peak exceeds threshold
			if (lobePeakVal >= clarityThreshold) {
				bestPeriod = lobePeakIdx;
				bestClarity = lobePeakVal;
				break; // First peak above threshold = fundamental
			}
			inPositiveLobe = false;
		}
	}

	// Check the last lobe if we ended in one
	if (inPositiveLobe && bestPeriod < 0 && lobePeakVal >= clarityThreshold) {
		bestPeriod = lobePeakIdx;
		bestClarity = lobePeakVal;
	}

	if (bestPeriod < 0) return null;

	// Parabolic interpolation for sub-sample accuracy
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

/**
 * Convert a frequency in Hz to the nearest MIDI note number and cent offset.
 * Uses A4 = 440 Hz standard tuning. MIDI note 69 = A4.
 */
export function frequencyToMidi(freq: number): MidiNoteInfo {
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
 * Example: 60 -> "C4", 69 -> "A4", 40 -> "E2"
 */
export function midiToNoteName(midi: number): string {
	const noteIndex = ((midi % 12) + 12) % 12;
	const octave = Math.floor(midi / 12) - 1;
	return `${NOTE_NAMES[noteIndex]}${octave}`;
}
