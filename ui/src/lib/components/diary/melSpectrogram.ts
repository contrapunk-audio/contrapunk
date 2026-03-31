/**
 * Browser-compatible mel-spectrogram extraction.
 *
 * Produces output matching Python's librosa.feature.melspectrogram with:
 *   n_mels=64, n_fft=1024, hop_length=256, sr=48000
 *
 * Pipeline: Hann window -> STFT (radix-2 FFT) -> mel filterbank -> power_to_db
 *
 * The model expects input shape (1, 1, 64, 94) — a log-mel spectrogram
 * with 64 mel bins and 94 time frames.
 */

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

export type MelSpectrogramParams = {
	nMels: number;    // number of mel frequency bins (default: 64)
	nFft: number;     // FFT window size (default: 1024)
	hopLength: number; // hop between frames (default: 256)
};

const DEFAULT_PARAMS: MelSpectrogramParams = {
	nMels: 64,
	nFft: 1024,
	hopLength: 256,
};

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/**
 * Extract a log-mel spectrogram from raw audio samples.
 *
 * @param audioData   Mono audio samples (Float32Array), values in [-1, 1]
 * @param sampleRate  Sample rate of the audio (e.g. 48000)
 * @param params      Spectrogram parameters (nMels, nFft, hopLength)
 * @returns           2D array [nMels][nFrames] of dB values
 */
export function extractMelSpectrogram(
	audioData: Float32Array,
	sampleRate: number,
	params: Partial<MelSpectrogramParams> = {},
): number[][] {
	const p = { ...DEFAULT_PARAMS, ...params };

	// 1. Compute STFT magnitude-squared (power spectrogram)
	const powerSpec = stft(audioData, p.nFft, p.hopLength);

	// 2. Build mel filterbank
	const melFilters = melFilterbank(sampleRate, p.nFft, p.nMels);

	// 3. Apply mel filterbank: melSpec = melFilters @ powerSpec
	const nFrames = powerSpec[0].length;
	const melSpec: number[][] = new Array(p.nMels);
	for (let m = 0; m < p.nMels; m++) {
		melSpec[m] = new Array(nFrames);
		for (let t = 0; t < nFrames; t++) {
			let sum = 0;
			for (let k = 0; k < melFilters[m].length; k++) {
				sum += melFilters[m][k] * powerSpec[k][t];
			}
			melSpec[m][t] = sum;
		}
	}

	// 4. Convert to dB scale (power_to_db)
	powerToDb(melSpec);

	return melSpec;
}

/**
 * Prepare a mel spectrogram as a flat Float32Array suitable for ONNX model input.
 * Shape: [1, 1, nMels, nFrames] — batch=1, channels=1.
 *
 * If the spectrogram has more than targetFrames columns, it is truncated.
 * If fewer, it is zero-padded on the right.
 *
 * @param melSpec       2D array from extractMelSpectrogram
 * @param targetFrames  Expected number of time frames (default: 94)
 * @returns             Float32Array of length nMels * targetFrames
 */
export function melSpectrogramToModelInput(
	melSpec: number[][],
	targetFrames = 94,
): Float32Array {
	const nMels = melSpec.length;
	const output = new Float32Array(nMels * targetFrames);

	for (let m = 0; m < nMels; m++) {
		const row = melSpec[m];
		const len = Math.min(row.length, targetFrames);
		for (let t = 0; t < len; t++) {
			output[m * targetFrames + t] = row[t];
		}
		// Remaining entries are already 0 from Float32Array initialization
	}

	return output;
}

// ---------------------------------------------------------------------------
// STFT — Short-Time Fourier Transform
// ---------------------------------------------------------------------------

/**
 * Compute the power spectrogram via STFT.
 *
 * @returns 2D array [(nFft/2+1)][nFrames] of power values (magnitude squared)
 */
function stft(
	audioData: Float32Array,
	nFft: number,
	hopLength: number,
): number[][] {
	const window = hannWindow(nFft);
	const nBins = Math.floor(nFft / 2) + 1;
	const nFrames = Math.max(0, 1 + Math.floor((audioData.length - nFft) / hopLength));

	// Pre-allocate output
	const powerSpec: number[][] = new Array(nBins);
	for (let k = 0; k < nBins; k++) {
		powerSpec[k] = new Array(nFrames);
	}

	// Scratch buffers for FFT (reused across frames)
	const real = new Float64Array(nFft);
	const imag = new Float64Array(nFft);

	for (let t = 0; t < nFrames; t++) {
		const offset = t * hopLength;

		// Apply window and copy to FFT buffer
		for (let i = 0; i < nFft; i++) {
			real[i] = (audioData[offset + i] || 0) * window[i];
			imag[i] = 0;
		}

		// In-place FFT
		fft(real, imag);

		// Extract power for positive frequencies
		for (let k = 0; k < nBins; k++) {
			powerSpec[k][t] = real[k] * real[k] + imag[k] * imag[k];
		}
	}

	return powerSpec;
}

// ---------------------------------------------------------------------------
// FFT — Cooley-Tukey radix-2 in-place
// ---------------------------------------------------------------------------

/**
 * In-place radix-2 FFT. Input length must be a power of 2.
 * Operates on separate real and imaginary Float64Arrays.
 */
function fft(real: Float64Array, imag: Float64Array): void {
	const n = real.length;
	if (n <= 1) return;

	// Bit-reversal permutation
	let j = 0;
	for (let i = 0; i < n; i++) {
		if (i < j) {
			// Swap
			let tmp = real[i]; real[i] = real[j]; real[j] = tmp;
			tmp = imag[i]; imag[i] = imag[j]; imag[j] = tmp;
		}
		let m = n >> 1;
		while (m >= 1 && j >= m) {
			j -= m;
			m >>= 1;
		}
		j += m;
	}

	// Cooley-Tukey butterfly
	for (let size = 2; size <= n; size *= 2) {
		const halfSize = size >> 1;
		const angleStep = -2 * Math.PI / size;

		for (let i = 0; i < n; i += size) {
			for (let k = 0; k < halfSize; k++) {
				const angle = angleStep * k;
				const twiddleReal = Math.cos(angle);
				const twiddleImag = Math.sin(angle);

				const evenIdx = i + k;
				const oddIdx = i + k + halfSize;

				const tr = real[oddIdx] * twiddleReal - imag[oddIdx] * twiddleImag;
				const ti = real[oddIdx] * twiddleImag + imag[oddIdx] * twiddleReal;

				real[oddIdx] = real[evenIdx] - tr;
				imag[oddIdx] = imag[evenIdx] - ti;
				real[evenIdx] = real[evenIdx] + tr;
				imag[evenIdx] = imag[evenIdx] + ti;
			}
		}
	}
}

// ---------------------------------------------------------------------------
// Hann window
// ---------------------------------------------------------------------------

/**
 * Generate a periodic Hann window of length N.
 * Matches numpy.hanning / scipy.signal.hann (periodic=True) used by librosa.
 */
function hannWindow(n: number): Float64Array {
	const w = new Float64Array(n);
	for (let i = 0; i < n; i++) {
		w[i] = 0.5 * (1 - Math.cos((2 * Math.PI * i) / n));
	}
	return w;
}

// ---------------------------------------------------------------------------
// Mel filterbank
// ---------------------------------------------------------------------------

/**
 * Build a mel filterbank matrix of shape [nMels][nFftBins].
 * Matches librosa.filters.mel with htk=False (Slaney formula).
 */
function melFilterbank(
	sampleRate: number,
	nFft: number,
	nMels: number,
): number[][] {
	const nBins = Math.floor(nFft / 2) + 1;
	const fMin = 0;
	const fMax = sampleRate / 2;

	// Mel scale conversion (Slaney / O'Shaughnessy formula, matching librosa default htk=False)
	const melMin = hzToMel(fMin);
	const melMax = hzToMel(fMax);

	// nMels + 2 equally spaced points in mel space
	const melPoints = new Float64Array(nMels + 2);
	for (let i = 0; i < nMels + 2; i++) {
		melPoints[i] = melMin + (i * (melMax - melMin)) / (nMels + 1);
	}

	// Convert mel points back to Hz
	const hzPoints = new Float64Array(nMels + 2);
	for (let i = 0; i < nMels + 2; i++) {
		hzPoints[i] = melToHz(melPoints[i]);
	}

	// Convert Hz points to FFT bin indices (floating point)
	const binPoints = new Float64Array(nMels + 2);
	for (let i = 0; i < nMels + 2; i++) {
		binPoints[i] = (hzPoints[i] * nFft) / sampleRate;
	}

	// Build triangular filters
	const filters: number[][] = new Array(nMels);
	for (let m = 0; m < nMels; m++) {
		filters[m] = new Array(nBins).fill(0);
		const left = binPoints[m];
		const center = binPoints[m + 1];
		const right = binPoints[m + 2];

		for (let k = 0; k < nBins; k++) {
			if (k >= left && k <= center && center > left) {
				filters[m][k] = (k - left) / (center - left);
			} else if (k > center && k <= right && right > center) {
				filters[m][k] = (right - k) / (right - center);
			}
		}

		// Slaney normalization (area = 1): divide by width of triangle in Hz
		const width = hzPoints[m + 2] - hzPoints[m];
		if (width > 0) {
			const norm = 2.0 / width;
			for (let k = 0; k < nBins; k++) {
				filters[m][k] *= norm;
			}
		}
	}

	return filters;
}

/**
 * Convert frequency in Hz to mel scale.
 * Uses the Slaney/O'Shaughnessy formula (matching librosa htk=False default):
 *   - Linear below 1000 Hz: mel = 3 * f / 200
 *   - Logarithmic above 1000 Hz: mel = 15 + 27 * ln(f / 1000) / ln(6.4)
 */
function hzToMel(hz: number): number {
	const f_sp = 200.0 / 3;  // 66.6667 Hz per mel below 1 kHz
	let mel = hz / f_sp;

	const minLogHz = 1000.0;
	const minLogMel = minLogHz / f_sp;  // 15.0
	const logstep = Math.log(6.4) / 27.0;

	if (hz >= minLogHz) {
		mel = minLogMel + Math.log(hz / minLogHz) / logstep;
	}

	return mel;
}

/**
 * Convert mel scale value back to Hz.
 * Inverse of hzToMel (Slaney formula).
 */
function melToHz(mel: number): number {
	const f_sp = 200.0 / 3;
	let hz = mel * f_sp;

	const minLogHz = 1000.0;
	const minLogMel = minLogHz / f_sp;  // 15.0
	const logstep = Math.log(6.4) / 27.0;

	if (mel >= minLogMel) {
		hz = minLogHz * Math.exp((mel - minLogMel) * logstep);
	}

	return hz;
}

// ---------------------------------------------------------------------------
// Power to dB conversion
// ---------------------------------------------------------------------------

/**
 * Convert a power spectrogram to dB scale, in place.
 * Matches librosa.power_to_db with ref=1.0, amin=1e-10, top_db=80.
 */
function powerToDb(spec: number[][]): void {
	const amin = 1e-10;
	const topDb = 80;
	const refDb = 0; // 10 * log10(1.0) = 0

	let maxDb = -Infinity;

	for (let m = 0; m < spec.length; m++) {
		for (let t = 0; t < spec[m].length; t++) {
			const val = Math.max(spec[m][t], amin);
			const db = 10 * Math.log10(val) - refDb;
			spec[m][t] = db;
			if (db > maxDb) maxDb = db;
		}
	}

	// Clamp to top_db below maximum
	const threshold = maxDb - topDb;
	for (let m = 0; m < spec.length; m++) {
		for (let t = 0; t < spec[m].length; t++) {
			if (spec[m][t] < threshold) {
				spec[m][t] = threshold;
			}
		}
	}
}

// ---------------------------------------------------------------------------
// Utility: resample audio to target sample rate (simple linear interpolation)
// ---------------------------------------------------------------------------

/**
 * Resample audio to a target sample rate using linear interpolation.
 * For production use, a higher-quality resampler (e.g. polyphase) would be
 * better, but linear is adequate for mel-spectrogram extraction where the
 * filterbank already applies frequency smoothing.
 */
export function resampleLinear(
	audioData: Float32Array,
	fromRate: number,
	toRate: number,
): Float32Array {
	if (fromRate === toRate) return audioData;

	const ratio = fromRate / toRate;
	const outLength = Math.round(audioData.length / ratio);
	const output = new Float32Array(outLength);

	for (let i = 0; i < outLength; i++) {
		const srcIdx = i * ratio;
		const lo = Math.floor(srcIdx);
		const hi = Math.min(lo + 1, audioData.length - 1);
		const frac = srcIdx - lo;
		output[i] = audioData[lo] * (1 - frac) + audioData[hi] * frac;
	}

	return output;
}

// ---------------------------------------------------------------------------
// Guitar position label mapping
// ---------------------------------------------------------------------------

/** Standard guitar string names, low to high */
const STRING_NAMES = ['E2', 'A2', 'D3', 'G3', 'B3', 'E4'];

/**
 * Convert a class index (0-137) to a human-readable guitar position label.
 * Classes are ordered as: string 0 fret 0, string 0 fret 1, ..., string 5 fret 22.
 * 6 strings x 23 frets = 138 classes.
 */
export function classIndexToLabel(classIndex: number): {
	string: number;
	fret: number;
	stringName: string;
	label: string;
} {
	const string = Math.floor(classIndex / 23);
	const fret = classIndex % 23;
	const stringName = STRING_NAMES[string] || `S${string}`;
	const fretLabel = fret === 0 ? 'open' : `fret ${fret}`;
	return {
		string,
		fret,
		stringName,
		label: `${stringName} ${fretLabel}`,
	};
}

/**
 * Get the top-k predictions from model output logits.
 * Returns sorted array of { classIndex, probability, label }.
 */
export function topKPredictions(
	logits: Float32Array | number[],
	k = 5,
): { classIndex: number; probability: number; label: string }[] {
	// Softmax
	let maxLogit = -Infinity;
	for (let i = 0; i < logits.length; i++) {
		if (logits[i] > maxLogit) maxLogit = logits[i];
	}

	const expValues = new Float64Array(logits.length);
	let sumExp = 0;
	for (let i = 0; i < logits.length; i++) {
		expValues[i] = Math.exp(logits[i] - maxLogit);
		sumExp += expValues[i];
	}

	// Build indexed probabilities and sort
	const indexed = Array.from({ length: logits.length }, (_, i) => ({
		classIndex: i,
		probability: expValues[i] / sumExp,
	}));

	indexed.sort((a, b) => b.probability - a.probability);

	return indexed.slice(0, k).map((item) => ({
		...item,
		label: classIndexToLabel(item.classIndex).label,
	}));
}
