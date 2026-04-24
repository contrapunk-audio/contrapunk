<script lang="ts">
	import { detectPitch, frequencyToMidi, midiToNoteName } from '$lib/audio/pitchDetector';

	// ── State ────────────────────────────────────────────────
	let audioDevices: MediaDeviceInfo[] = $state([]);
	let selectedDeviceId = $state('');
	let selectedChannel = $state(0);
	let maxChannels = $state(2);
	let running = $state(false);
	let error = $state('');

	// Pitch detection state
	let detectedNote = $state('—');
	let detectedFreq = $state(0);
	let detectedCents = $state(0);
	let detectedClarity = $state(0);
	let detectedMidi = $state(0);
	let rmsLevel = $state(0);

	// Envelope follower state
	let envAttack = $state(0.9);
	let envRelease = $state(0.02);
	let envelopeValue = 0;
	let peakRms = 0.01;
	let prevRms = 0;

	// Sound output
	let soundEnabled = $state(true);
	let soundVolume = $state(0.15);
	let midiOutputEnabled = $state(false);
	let selectedMidiOutputId = $state('');
	let midiOutputs: { id: string; name: string }[] = $state([]);

	// Recording state
	let recording = $state(false);
	let recordedChunks: Float32Array[] = [];
	let recordingDuration = $state(0);
	let recordingTimer: number | null = null;

	// Canvas refs
	let waveformCanvas: HTMLCanvasElement;
	let envelopeCanvas: HTMLCanvasElement;
	let spectrumCanvas: HTMLCanvasElement;

	// Buffers
	const WAVE_SIZE = 2048;
	const ENV_HISTORY = 256;
	let waveformBuffer = new Float32Array(WAVE_SIZE);
	let envelopeHistory: number[] = [];
	let onsetMarkers: number[] = [];

	// Spectrum / harmonic analysis
	let spectrumMagnitudes = new Float32Array(0);
	let harmonicScore = $state(0);
	let candidateFundamental = $state(0);
	let allCandidates: {
		freq: number;
		score: number;
		note: string;
		markers: { freq: number; mag: number; label: string; present: boolean }[];
		color: string;
	}[] = $state([]);
	let sampleRate = 48000;

	// Colors for different candidates
	const CANDIDATE_COLORS = ['#4ade80', '#60a5fa', '#fbbf24', '#f472b6', '#a78bfa'];

	// Frequency continuity filter
	let freqContinuityEnabled = $state(true);
	let maxJumpSemitones = $state(12); // max allowed jump between frames (1 octave)
	let prevDetectedFreq = 0;
	let spikeCount = $state(0); // count of rejected spikes

	// MIDI log
	let midiLog: { time: number; type: string; note: string; midi: number; vel: number; freq: number; clarity: number; rejected?: boolean }[] = $state([]);
	const MAX_LOG = 50;
	let lastNoteOnMidi = -1;
	let lastNoteOnTime = 0;

	// Audio context and nodes
	let audioCtx: AudioContext | null = null;
	let stream: MediaStream | null = null;
	let processor: ScriptProcessorNode | null = null;
	let oscillator: OscillatorNode | null = null;
	let gainNode: GainNode | null = null;
	let midiAccess: MIDIAccess | null = null;
	let animFrame: number | null = null;

	// ── Audio device enumeration ─────────────────────────────
	async function enumerateDevices() {
		try {
			const s = await navigator.mediaDevices.getUserMedia({ audio: true });
			s.getTracks().forEach(t => t.stop());
			const all = await navigator.mediaDevices.enumerateDevices();
			audioDevices = all.filter(d => d.kind === 'audioinput');
			if (audioDevices.length > 0 && !selectedDeviceId) {
				const audient = audioDevices.find(d => d.label.toLowerCase().includes('audient'));
				selectedDeviceId = audient ? audient.deviceId : audioDevices[0].deviceId;
			}
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to enumerate devices';
		}
	}

	// ── MIDI output enumeration ──────────────────────────────
	async function enumerateMidiOutputs() {
		if (!('requestMIDIAccess' in navigator)) return;
		try {
			midiAccess = await navigator.requestMIDIAccess();
			const outs: { id: string; name: string }[] = [];
			midiAccess.outputs.forEach((output, id) => {
				outs.push({ id, name: output.name ?? `Output ${id}` });
			});
			midiOutputs = outs;
			if (outs.length > 0 && !selectedMidiOutputId) {
				selectedMidiOutputId = outs[0].id;
			}
		} catch {
			// Web MIDI not available
		}
	}

	// ── FFT (radix-2 Cooley-Tukey) ──────────────────────────
	function fft(re: Float32Array, im: Float32Array) {
		const n = re.length;
		// Bit-reversal permutation
		for (let i = 1, j = 0; i < n; i++) {
			let bit = n >> 1;
			for (; j & bit; bit >>= 1) j ^= bit;
			j ^= bit;
			if (i < j) {
				[re[i], re[j]] = [re[j], re[i]];
				[im[i], im[j]] = [im[j], im[i]];
			}
		}
		// Butterfly stages
		for (let len = 2; len <= n; len <<= 1) {
			const halfLen = len >> 1;
			const angle = (-2 * Math.PI) / len;
			const wRe = Math.cos(angle);
			const wIm = Math.sin(angle);
			for (let i = 0; i < n; i += len) {
				let curRe = 1, curIm = 0;
				for (let j = 0; j < halfLen; j++) {
					const a = i + j;
					const b = a + halfLen;
					const tRe = curRe * re[b] - curIm * im[b];
					const tIm = curRe * im[b] + curIm * re[b];
					re[b] = re[a] - tRe;
					im[b] = im[a] - tIm;
					re[a] += tRe;
					im[a] += tIm;
					const nextRe = curRe * wRe - curIm * wIm;
					curIm = curRe * wIm + curIm * wRe;
					curRe = nextRe;
				}
			}
		}
	}

	/** Compute magnitude spectrum from audio samples. */
	function computeSpectrum(samples: Float32Array): Float32Array {
		const n = samples.length;
		const re = new Float32Array(n);
		const im = new Float32Array(n);
		// Apply Hann window
		for (let i = 0; i < n; i++) {
			const w = 0.5 * (1 - Math.cos((2 * Math.PI * i) / (n - 1)));
			re[i] = samples[i] * w;
		}
		fft(re, im);
		// Magnitude (only first half — Nyquist)
		const halfN = n >> 1;
		const mag = new Float32Array(halfN);
		for (let i = 0; i < halfN; i++) {
			mag[i] = Math.sqrt(re[i] * re[i] + im[i] * im[i]) / halfN;
		}
		return mag;
	}

	/** Get magnitude at a specific frequency using interpolated bin lookup. */
	function magAtFreq(mag: Float32Array, freq: number, sr: number): number {
		const binSize = sr / (mag.length * 2);
		const bin = freq / binSize;
		const lo = Math.floor(bin);
		const hi = Math.ceil(bin);
		if (lo < 0 || hi >= mag.length) return 0;
		if (lo === hi) return mag[lo];
		const frac = bin - lo;
		return mag[lo] * (1 - frac) + mag[hi] * frac;
	}

	/**
	 * Score how well a candidate fundamental's harmonics match the spectrum.
	 * Returns 0..1. Checks harmonics 1F through 8F.
	 * A real fundamental will have most of its harmonics present.
	 * A harmonic masquerading as fundamental will have "harmonics" at weird positions.
	 */
	function scoreHarmonics(mag: Float32Array, fundamental: number, sr: number): {
		score: number;
		markers: { freq: number; mag: number; label: string; present: boolean }[];
	} {
		const numHarmonics = 8;
		const markers: { freq: number; mag: number; label: string; present: boolean }[] = [];
		const fundamentalMag = magAtFreq(mag, fundamental, sr);

		// Find the max magnitude across the full spectrum for normalization
		const binSize = sr / (mag.length * 2);
		let maxMag = 0;
		for (let i = 1; i < mag.length; i++) {
			if (mag[i] > maxMag) maxMag = mag[i];
		}
		if (maxMag < 0.0001) return { score: 0, markers };

		let presentCount = 0;
		let weightedScore = 0;
		const weights = [1.0, 0.8, 0.6, 0.5, 0.4, 0.3, 0.25, 0.2]; // fundamental weighted highest

		for (let h = 1; h <= numHarmonics; h++) {
			const hFreq = fundamental * h;
			if (hFreq > sr / 2) break; // Above Nyquist

			const hMag = magAtFreq(mag, hFreq, sr);
			// Use peak-relative magnitude for display, but check against
			// a neighborhood (±1 bin) to avoid missing harmonics between bins
			const binIdx = Math.round(hFreq / binSize);
			let peakNearby = hMag;
			for (let d = -2; d <= 2; d++) {
				const idx = binIdx + d;
				if (idx >= 0 && idx < mag.length && mag[idx] > peakNearby) {
					peakNearby = mag[idx];
				}
			}
			const relMag = peakNearby / maxMag;
			const threshold = 0.02; // harmonic must be at least 2% of peak
			const present = relMag > threshold;

			markers.push({
				freq: hFreq,
				mag: relMag,
				label: h === 1 ? 'F' : `${h}F`,
				present,
			});

			if (present) {
				presentCount++;
				weightedScore += weights[h - 1] * relMag;
			}
		}

		// Normalize score
		const maxPossible = weights.slice(0, markers.length).reduce((a, b) => a + b, 0);
		const score = maxPossible > 0 ? weightedScore / maxPossible : 0;

		return { score, markers };
	}

	/**
	 * Check if the detected freq might be a harmonic of a lower fundamental.
	 * Tests if freq/2, freq/3, freq/4 score better as fundamentals.
	 */
	function checkSubharmonics(mag: Float32Array, detectedFreq: number, sr: number): { freq: number; score: number; note: string }[] {
		const candidates: { freq: number; score: number; note: string }[] = [];
		const detectedScore = scoreHarmonics(mag, detectedFreq, sr).score;
		candidates.push({
			freq: detectedFreq,
			score: Math.round(detectedScore * 100),
			note: midiToNoteName(frequencyToMidi(detectedFreq).note),
		});

		for (let divisor = 2; divisor <= 5; divisor++) {
			const subFreq = detectedFreq / divisor;
			if (subFreq < 60) continue; // Below guitar range
			const subScore = scoreHarmonics(mag, subFreq, sr).score;
			if (subScore > 0.05) {
				candidates.push({
					freq: Math.round(subFreq * 10) / 10,
					score: Math.round(subScore * 100),
					note: midiToNoteName(frequencyToMidi(subFreq).note),
				});
			}
		}

		// Sort by score descending
		candidates.sort((a, b) => b.score - a.score);
		return candidates;
	}

	// ── Start capture ────────────────────────────────────────
	async function start() {
		if (running) return;
		error = '';

		try {
			audioCtx = new AudioContext({ sampleRate: 48000 });
			stream = await navigator.mediaDevices.getUserMedia({
				audio: {
					...(selectedDeviceId ? { deviceId: { exact: selectedDeviceId } } : {}),
					echoCancellation: false,
					noiseSuppression: false,
					autoGainControl: false,
					channelCount: { ideal: 32 },
				}
			});

			// Probe channel count
			const track = stream.getAudioTracks()[0];
			if (track) {
				const settings = track.getSettings();
				maxChannels = settings.channelCount ?? 2;
			}

			const source = audioCtx.createMediaStreamSource(stream);
			const inputChannels = Math.max(selectedChannel + 1, source.channelCount);
			processor = audioCtx.createScriptProcessor(WAVE_SIZE, inputChannels, 1);
			processor.channelCountMode = 'explicit';
			processor.channelInterpretation = 'discrete';

			// Set up sound output (oscillator synth)
			gainNode = audioCtx.createGain();
			gainNode.gain.value = 0;
			gainNode.connect(audioCtx.destination);

			oscillator = audioCtx.createOscillator();
			oscillator.type = 'triangle';
			oscillator.frequency.value = 0;
			oscillator.connect(gainNode);
			oscillator.start();

			processor.onaudioprocess = (event) => {
				const input = event.inputBuffer;
				const ch = Math.min(selectedChannel, input.numberOfChannels - 1);
				const samples = input.getChannelData(ch);

				// Copy waveform for display
				waveformBuffer = new Float32Array(samples);

				// Record raw audio if recording
				if (recording) {
					recordedChunks.push(new Float32Array(samples));
				}

				// Compute RMS
				let sum = 0;
				for (let i = 0; i < samples.length; i++) sum += samples[i] * samples[i];
				const rms = Math.sqrt(sum / samples.length);
				rmsLevel = rms;

				// Envelope follower
				if (rms > peakRms) peakRms = rms;
				else peakRms = peakRms * 0.998 + rms * 0.002;
				const peak = Math.max(peakRms, 0.005);
				const normalizedRms = Math.min(1, rms / peak);

				if (normalizedRms > envelopeValue) {
					envelopeValue += (normalizedRms - envelopeValue) * envAttack;
				} else {
					envelopeValue += (normalizedRms - envelopeValue) * envRelease;
				}
				envelopeValue = Math.max(0, Math.min(1, envelopeValue));

				// Onset detection
				const rmsJump = rms / Math.max(prevRms, 0.001);
				if (rmsJump > 2.5 && rms > peak * 0.1) {
					onsetMarkers.push(envelopeHistory.length);
				}
				prevRms = rms;

				// Push envelope
				envelopeHistory.push(Math.sqrt(envelopeValue));
				if (envelopeHistory.length > ENV_HISTORY) {
					envelopeHistory.shift();
					for (let i = 0; i < onsetMarkers.length; i++) onsetMarkers[i]--;
					while (onsetMarkers.length > 0 && onsetMarkers[0] < 0) onsetMarkers.shift();
				}

				// Compute spectrum for harmonic analysis + visualization
				// @ts-expect-error TS 5.6+ Float32Array<ArrayBufferLike> from Web Audio getChannelData()
				spectrumMagnitudes = computeSpectrum(samples);
				sampleRate = audioCtx!.sampleRate;

				// Pitch detection
				const result = detectPitch(samples, audioCtx!.sampleRate, 0.7);
				const isOnset = rmsJump > 2.5 && rms > peak * 0.1;
				if (result && rms > 0.005) {
					const midi = frequencyToMidi(result.frequency);

					// Harmonic template analysis — score detected + all subharmonic candidates
					candidateFundamental = result.frequency;
					const candidates: typeof allCandidates = [];

					// Detected frequency
					const detResult = scoreHarmonics(spectrumMagnitudes, result.frequency, sampleRate);
					candidates.push({
						freq: result.frequency,
						score: Math.round(detResult.score * 100),
						note: midiToNoteName(midi.note),
						markers: detResult.markers,
						color: CANDIDATE_COLORS[0],
					});

					// Subharmonic candidates (freq/2, freq/3, freq/4, freq/5)
					for (let divisor = 2; divisor <= 5; divisor++) {
						const subFreq = result.frequency / divisor;
						if (subFreq < 60) continue;
						const subResult = scoreHarmonics(spectrumMagnitudes, subFreq, sampleRate);
						if (subResult.score > 0.05) {
							const subMidi = frequencyToMidi(subFreq);
							candidates.push({
								freq: Math.round(subFreq * 10) / 10,
								score: Math.round(subResult.score * 100),
								note: midiToNoteName(subMidi.note),
								markers: subResult.markers,
								color: CANDIDATE_COLORS[Math.min(divisor - 1, CANDIDATE_COLORS.length - 1)],
							});
						}
					}

					// Sort by score — best candidate first
					candidates.sort((a, b) => b.score - a.score);
					allCandidates = candidates;
					harmonicScore = candidates[0]?.score ?? 0;

					// Frequency continuity filter — reject harmonic spikes during sustain
					// Rule: large frequency jumps are only allowed if there's a new pluck (onset)
					// During sustain (no onset), the pitch shouldn't jump more than N semitones
					let rejected = false;
					if (freqContinuityEnabled && prevDetectedFreq > 0 && !isOnset) {
						const semitoneDiff = Math.abs(12 * Math.log2(result.frequency / prevDetectedFreq));
						if (semitoneDiff > maxJumpSemitones) {
							rejected = true;
							spikeCount++;
							logMidi('SPIKE', midi.note, 0, result.frequency, result.clarity, true);
						}
					}

					if (!rejected) {
						prevDetectedFreq = result.frequency;
						detectedFreq = Math.round(result.frequency * 10) / 10;
						detectedClarity = Math.round(result.clarity * 100);
						detectedMidi = midi.note;
						detectedCents = midi.cents;
						detectedNote = midiToNoteName(midi.note);

						// Sound output — set oscillator to detected frequency
						if (soundEnabled && oscillator && gainNode) {
							oscillator.frequency.setTargetAtTime(result.frequency, audioCtx!.currentTime, 0.01);
							gainNode.gain.setTargetAtTime(soundVolume * result.clarity, audioCtx!.currentTime, 0.02);
						}

						// MIDI output
						if (midiOutputEnabled && midiAccess && selectedMidiOutputId) {
							const output = midiAccess.outputs.get(selectedMidiOutputId);
							if (output) {
								if (midi.note !== lastNoteOnMidi) {
									if (lastNoteOnMidi >= 0) {
										output.send([0x80, lastNoteOnMidi, 0]);
									}
									const vel = Math.min(127, Math.round(rms * 800));
									output.send([0x90, midi.note, vel]);
									logMidi('ON', midi.note, vel, result.frequency, result.clarity);
									lastNoteOnMidi = midi.note;
									lastNoteOnTime = performance.now();
								}
							}
						} else if (detectedMidi !== lastNoteOnMidi) {
							const vel = Math.min(127, Math.round(rms * 800));
							logMidi('ON', midi.note, vel, result.frequency, result.clarity);
							lastNoteOnMidi = detectedMidi;
						}
					}
				} else {
					// Silence — ramp down
					if (soundEnabled && gainNode) {
						gainNode.gain.setTargetAtTime(0, audioCtx!.currentTime, 0.05);
					}
					if (lastNoteOnMidi >= 0 && rms < 0.003) {
						if (midiOutputEnabled && midiAccess && selectedMidiOutputId) {
							const output = midiAccess.outputs.get(selectedMidiOutputId);
							if (output) output.send([0x80, lastNoteOnMidi, 0]);
						}
						logMidi('OFF', lastNoteOnMidi, 0, 0, 0);
						lastNoteOnMidi = -1;
					}
					detectedNote = '—';
					detectedFreq = 0;
					detectedClarity = 0;
					detectedCents = 0;
				}
			};

			source.connect(processor);
			processor.connect(audioCtx.destination);
			running = true;

			// Start render loop
			renderLoop();
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to start';
		}
	}

	function logMidi(type: string, midi: number, vel: number, freq: number, clarity: number, rejected = false) {
		const note = midiToNoteName(midi);
		midiLog = [{ time: performance.now(), type, note, midi, vel, freq: Math.round(freq * 10) / 10, clarity: Math.round(clarity * 100), rejected }, ...midiLog.slice(0, MAX_LOG - 1)];
	}

	// ── Stop capture ─────────────────────────────────────────
	function stop() {
		if (processor) { processor.disconnect(); processor = null; }
		if (oscillator) { oscillator.stop(); oscillator = null; }
		if (gainNode) { gainNode.disconnect(); gainNode = null; }
		if (stream) { stream.getTracks().forEach(t => t.stop()); stream = null; }
		if (audioCtx) { audioCtx.close(); audioCtx = null; }
		if (animFrame) { cancelAnimationFrame(animFrame); animFrame = null; }
		running = false;
		envelopeValue = 0;
		peakRms = 0.01;
		prevRms = 0;
		envelopeHistory = [];
		onsetMarkers = [];
	}

	// ── Render loop ──────────────────────────────────────────
	function renderLoop() {
		drawWaveform();
		drawEnvelope();
		drawSpectrum();
		animFrame = requestAnimationFrame(renderLoop);
	}

	function drawWaveform() {
		if (!waveformCanvas) return;
		const ctx = waveformCanvas.getContext('2d');
		if (!ctx) return;
		const W = waveformCanvas.width;
		const H = waveformCanvas.height;

		ctx.fillStyle = '#0a0a0f';
		ctx.fillRect(0, 0, W, H);

		// Center line
		ctx.strokeStyle = '#222';
		ctx.lineWidth = 1;
		ctx.beginPath();
		ctx.moveTo(0, H / 2);
		ctx.lineTo(W, H / 2);
		ctx.stroke();

		// Waveform
		if (waveformBuffer.length > 0) {
			ctx.beginPath();
			ctx.strokeStyle = '#00e5cc';
			ctx.lineWidth = 1;
			const step = waveformBuffer.length / W;
			for (let x = 0; x < W; x++) {
				const i = Math.floor(x * step);
				const val = waveformBuffer[i] ?? 0;
				const y = H / 2 - val * H * 2;
				if (x === 0) ctx.moveTo(x, y);
				else ctx.lineTo(x, y);
			}
			ctx.stroke();
		}
	}

	function drawEnvelope() {
		if (!envelopeCanvas) return;
		const ctx = envelopeCanvas.getContext('2d');
		if (!ctx) return;
		const W = envelopeCanvas.width;
		const H = envelopeCanvas.height;

		ctx.fillStyle = '#0a0a0f';
		ctx.fillRect(0, 0, W, H);

		const data = envelopeHistory;
		if (data.length < 2) return;

		// Onset markers
		for (const idx of onsetMarkers) {
			if (idx >= 0 && idx < data.length) {
				const x = (idx / (data.length - 1)) * W;
				ctx.strokeStyle = '#ff6b3530';
				ctx.lineWidth = 4;
				ctx.beginPath(); ctx.moveTo(x, 0); ctx.lineTo(x, H); ctx.stroke();
				ctx.strokeStyle = '#ff6b35';
				ctx.lineWidth = 1;
				ctx.beginPath(); ctx.moveTo(x, 0); ctx.lineTo(x, H); ctx.stroke();
			}
		}

		// Filled area
		ctx.beginPath();
		ctx.moveTo(0, H);
		for (let i = 0; i < data.length; i++) {
			const x = (i / (data.length - 1)) * W;
			const y = H - data[i] * H;
			ctx.lineTo(x, y);
		}
		ctx.lineTo(W, H);
		ctx.closePath();
		ctx.fillStyle = '#6366f140';
		ctx.fill();

		// Line
		ctx.beginPath();
		for (let i = 0; i < data.length; i++) {
			const x = (i / (data.length - 1)) * W;
			const y = H - data[i] * H;
			if (i === 0) ctx.moveTo(x, y); else ctx.lineTo(x, y);
		}
		ctx.strokeStyle = '#6366f1';
		ctx.lineWidth = 1.5;
		ctx.stroke();
	}

	function drawSpectrum() {
		if (!spectrumCanvas || spectrumMagnitudes.length === 0) return;
		const ctx = spectrumCanvas.getContext('2d');
		if (!ctx) return;
		const W = spectrumCanvas.width;
		const H = spectrumCanvas.height;

		ctx.fillStyle = '#0a0a0f';
		ctx.fillRect(0, 0, W, H);

		const mag = spectrumMagnitudes;
		const binSize = sampleRate / (mag.length * 2);
		// Show 0 - 4000 Hz (covers guitar range + harmonics up to 8F)
		const maxFreqDisplay = 4000;
		const maxBin = Math.min(Math.floor(maxFreqDisplay / binSize), mag.length - 1);

		// Find peak for normalization
		let maxMag = 0;
		for (let i = 1; i < maxBin; i++) {
			if (mag[i] > maxMag) maxMag = mag[i];
		}
		if (maxMag < 0.0001) return;

		// Draw frequency grid lines
		ctx.strokeStyle = '#1a1a2a';
		ctx.lineWidth = 1;
		for (let f = 100; f <= maxFreqDisplay; f += 100) {
			const x = (f / maxFreqDisplay) * W;
			ctx.beginPath();
			ctx.moveTo(x, 0);
			ctx.lineTo(x, H);
			ctx.stroke();
		}
		// Label key frequencies
		ctx.fillStyle = '#333';
		ctx.font = '9px monospace';
		for (const f of [100, 250, 500, 1000, 1500, 2000, 3000, 4000]) {
			const x = (f / maxFreqDisplay) * W;
			ctx.fillText(`${f}`, x - 10, H - 2);
		}

		// Draw spectrum bars
		ctx.fillStyle = '#00e5cc30';
		for (let i = 1; i < maxBin; i++) {
			const x = (i * binSize / maxFreqDisplay) * W;
			const barW = Math.max(1, (binSize / maxFreqDisplay) * W);
			const barH = (mag[i] / maxMag) * H * 0.9;
			ctx.fillRect(x, H - barH, barW, barH);
		}

		// Draw spectrum line
		ctx.beginPath();
		ctx.strokeStyle = '#00e5cc';
		ctx.lineWidth = 1;
		for (let i = 1; i < maxBin; i++) {
			const x = (i * binSize / maxFreqDisplay) * W;
			const y = H - (mag[i] / maxMag) * H * 0.9;
			if (i === 1) ctx.moveTo(x, y); else ctx.lineTo(x, y);
		}
		ctx.stroke();

		// Draw harmonic markers for ALL candidates (each in its own color)
		for (let ci = allCandidates.length - 1; ci >= 0; ci--) {
			// Draw worst candidates first so best (ci=0) renders on top
			const candidate = allCandidates[ci];
			const color = candidate.color;
			const isBest = ci === 0;
			const labelYBase = 14 + ci * 16; // Stack labels vertically per candidate

			for (const marker of candidate.markers) {
				if (marker.freq > maxFreqDisplay) continue;
				const x = (marker.freq / maxFreqDisplay) * W;

				// Vertical line
				const alpha = marker.present ? (isBest ? '80' : '40') : '20';
				ctx.strokeStyle = color + alpha;
				ctx.lineWidth = (marker.label === 'F' && isBest) ? 3 : 1;
				ctx.beginPath();
				ctx.moveTo(x, 0);
				ctx.lineTo(x, H);
				ctx.stroke();

				// Dot at magnitude
				const dotY = H - marker.mag * H * 0.9;
				ctx.fillStyle = marker.present ? color : color + '40';
				ctx.beginPath();
				ctx.arc(x, dotY, marker.label === 'F' ? 4 : 3, 0, Math.PI * 2);
				ctx.fill();
			}

			// Label at the fundamental frequency only (to avoid clutter)
			const fundMarker = candidate.markers[0];
			if (fundMarker && fundMarker.freq <= maxFreqDisplay) {
				const x = (fundMarker.freq / maxFreqDisplay) * W;
				const labelText = `${candidate.note} ${candidate.score}%`;
				ctx.font = isBest ? 'bold 12px monospace' : '10px monospace';
				const textW = ctx.measureText(labelText).width;
				// Background pill
				ctx.fillStyle = '#0a0a0fcc';
				ctx.fillRect(x - 2, labelYBase - 11, textW + 4, 14);
				// Border to show candidate color
				ctx.strokeStyle = color;
				ctx.lineWidth = 1;
				ctx.strokeRect(x - 2, labelYBase - 11, textW + 4, 14);
				// Text
				ctx.fillStyle = color;
				ctx.fillText(labelText, x, labelYBase);
			}
		}
	}

	// ── Recording ────────────────────────────────────────────
	function startRecording() {
		recordedChunks = [];
		recordingDuration = 0;
		recording = true;
		const startTime = Date.now();
		recordingTimer = window.setInterval(() => {
			recordingDuration = (Date.now() - startTime) / 1000;
		}, 100);
	}

	function stopRecording() {
		recording = false;
		if (recordingTimer) { clearInterval(recordingTimer); recordingTimer = null; }
	}

	function downloadWav() {
		if (recordedChunks.length === 0) return;
		const sr = sampleRate;
		// Concatenate all chunks
		const totalSamples = recordedChunks.reduce((sum, c) => sum + c.length, 0);
		const pcm = new Float32Array(totalSamples);
		let offset = 0;
		for (const chunk of recordedChunks) {
			pcm.set(chunk, offset);
			offset += chunk.length;
		}
		// Encode WAV
		const wav = encodeWav(pcm, sr);
		const blob = new Blob([wav], { type: 'audio/wav' });
		const url = URL.createObjectURL(blob);
		const a = document.createElement('a');
		a.href = url;
		const timestamp = new Date().toISOString().replace(/[:.]/g, '-').slice(0, 19);
		a.download = `guitar-recording-${timestamp}.wav`;
		a.click();
		URL.revokeObjectURL(url);
	}

	function encodeWav(samples: Float32Array, sr: number): ArrayBuffer {
		const numChannels = 1;
		const bitsPerSample = 16;
		const bytesPerSample = bitsPerSample / 8;
		const dataSize = samples.length * bytesPerSample;
		const buffer = new ArrayBuffer(44 + dataSize);
		const view = new DataView(buffer);

		// RIFF header
		writeString(view, 0, 'RIFF');
		view.setUint32(4, 36 + dataSize, true);
		writeString(view, 8, 'WAVE');
		// fmt chunk
		writeString(view, 12, 'fmt ');
		view.setUint32(16, 16, true);
		view.setUint16(20, 1, true); // PCM
		view.setUint16(22, numChannels, true);
		view.setUint32(24, sr, true);
		view.setUint32(28, sr * numChannels * bytesPerSample, true);
		view.setUint16(32, numChannels * bytesPerSample, true);
		view.setUint16(34, bitsPerSample, true);
		// data chunk
		writeString(view, 36, 'data');
		view.setUint32(40, dataSize, true);
		// PCM samples (float32 → int16)
		let pos = 44;
		for (let i = 0; i < samples.length; i++) {
			const s = Math.max(-1, Math.min(1, samples[i]));
			view.setInt16(pos, s < 0 ? s * 0x8000 : s * 0x7FFF, true);
			pos += 2;
		}
		return buffer;
	}

	function writeString(view: DataView, offset: number, str: string) {
		for (let i = 0; i < str.length; i++) {
			view.setUint8(offset + i, str.charCodeAt(i));
		}
	}

	// ── Init ─────────────────────────────────────────────────
	$effect(() => {
		enumerateDevices();
		enumerateMidiOutputs();
		return () => stop();
	});
</script>

<svelte:head>
	<title>Guitar/MIDI Debug — Contrapunk</title>
</svelte:head>

<div class="debug-page">
	<header>
		<h1>Guitar / MIDI Debug</h1>
		<span class="status" class:running>{running ? 'RUNNING' : 'STOPPED'}</span>
	</header>

	{#if error}
		<div class="error">{error}</div>
	{/if}

	<!-- ── Controls ──────────────────────────────────── -->
	<section class="controls">
		<div class="control-group">
			<label>Audio Device</label>
			<select bind:value={selectedDeviceId} disabled={running}>
				{#each audioDevices as dev}
					<option value={dev.deviceId}>{dev.label || dev.deviceId.slice(0, 16)}</option>
				{/each}
			</select>
		</div>
		<div class="control-group">
			<label>Channel</label>
			<select bind:value={selectedChannel} disabled={running}>
				{#each Array.from({ length: maxChannels }, (_, i) => i) as ch}
					<option value={ch}>Ch {ch + 1}</option>
				{/each}
			</select>
		</div>
		<button class="start-btn" onclick={() => running ? stop() : start()}>
			{running ? 'Stop' : 'Start'}
		</button>
		{#if running}
			<button class="rec-btn" class:rec-active={recording} onclick={() => recording ? stopRecording() : startRecording()}>
				{recording ? `⏺ ${recordingDuration.toFixed(1)}s` : '⏺ Rec'}
			</button>
		{/if}
		{#if !recording && recordedChunks.length > 0}
			<button class="download-btn" onclick={downloadWav}>
				↓ Download WAV ({recordingDuration.toFixed(1)}s)
			</button>
		{/if}
	</section>

	<!-- ── Detection display ─────────────────────────── -->
	<section class="detection">
		<div class="big-note">{detectedNote}</div>
		<div class="stats">
			<div><span class="label">Freq</span> <span class="value">{detectedFreq} Hz</span></div>
			<div><span class="label">MIDI</span> <span class="value">{detectedMidi}</span></div>
			<div><span class="label">Cents</span> <span class="value" class:sharp={detectedCents > 5} class:flat={detectedCents < -5}>{detectedCents > 0 ? '+' : ''}{detectedCents}</span></div>
			<div><span class="label">Clarity</span> <span class="value">{detectedClarity}%</span></div>
			<div><span class="label">RMS</span> <span class="value">{(rmsLevel * 1000).toFixed(1)}</span></div>
		</div>
	</section>

	<!-- ── Waveform ──────────────────────────────────── -->
	<section class="graph-section">
		<h2>Raw Waveform</h2>
		<canvas bind:this={waveformCanvas} width={600} height={120}></canvas>
	</section>

	<!-- ── Envelope ──────────────────────────────────── -->
	<section class="graph-section">
		<h2>Envelope <span class="dim">(onset markers in orange)</span></h2>
		<canvas bind:this={envelopeCanvas} width={600} height={100}></canvas>
		<div class="envelope-controls">
			<label>Attack <input type="range" min="0.1" max="1" step="0.05" bind:value={envAttack} /> {envAttack.toFixed(2)}</label>
			<label>Release <input type="range" min="0.005" max="0.2" step="0.005" bind:value={envRelease} /> {envRelease.toFixed(3)}</label>
		</div>
	</section>

	<!-- ── Spectrum + Harmonics ───────────────────────── -->
	<section class="graph-section">
		<h2>Spectrum + Harmonic Template <span class="dim">(score: {harmonicScore}%)</span></h2>
		<canvas bind:this={spectrumCanvas} width={600} height={160}></canvas>
		<div class="harmonic-detail">
			{#each allCandidates as candidate, i}
				<div class="candidate-row" class:best={i === 0}>
					<span class="candidate-badge" style="background: {candidate.color}20; color: {candidate.color}; border-color: {candidate.color}40">
						{candidate.note} {candidate.freq}Hz — {candidate.score}%
					</span>
					<div class="harmonic-markers-row">
						{#each candidate.markers as m}
							<span class="harmonic-tag" style="background: {m.present ? candidate.color + '20' : '#2a0f0f'}; color: {m.present ? candidate.color : '#f87171'}">
								{m.label} {Math.round(m.freq)}Hz
							</span>
						{/each}
					</div>
				</div>
			{/each}
		</div>
	</section>

	<!-- ── Continuity filter ─────────────────────────── -->
	<section class="graph-section">
		<h2>Spike Filter <span class="dim">({spikeCount} rejected)</span></h2>
		<div class="envelope-controls">
			<label class="toggle">
				<input type="checkbox" bind:checked={freqContinuityEnabled} />
				Frequency continuity
			</label>
			<label>Max jump <input type="range" min="2" max="24" step="1" bind:value={maxJumpSemitones} /> {maxJumpSemitones} semitones</label>
		</div>
	</section>

	<!-- ── Sound output ──────────────────────────────── -->
	<section class="output-section">
		<h2>Sound Output</h2>
		<div class="output-row">
			<label class="toggle">
				<input type="checkbox" bind:checked={soundEnabled} />
				Oscillator Synth (hear what detector sees)
			</label>
			<label>Vol <input type="range" min="0" max="0.5" step="0.01" bind:value={soundVolume} disabled={!soundEnabled} /></label>
		</div>
		<div class="output-row">
			<label class="toggle">
				<input type="checkbox" bind:checked={midiOutputEnabled} />
				MIDI Output
			</label>
			{#if midiOutputs.length > 0}
				<select bind:value={selectedMidiOutputId} disabled={!midiOutputEnabled}>
					{#each midiOutputs as out}
						<option value={out.id}>{out.name}</option>
					{/each}
				</select>
			{:else}
				<span class="dim">No MIDI outputs found</span>
			{/if}
		</div>
	</section>

	<!-- ── MIDI Log ──────────────────────────────────── -->
	<section class="midi-log-section">
		<h2>MIDI Log <button class="clear-btn" onclick={() => { midiLog = []; }}>Clear</button></h2>
		<div class="midi-log">
			{#each midiLog as entry}
				<div class="log-entry" class:note-on={entry.type === 'ON'} class:note-off={entry.type === 'OFF'} class:spike={entry.type === 'SPIKE'}>
					<span class="log-type">{entry.type}</span>
					<span class="log-note">{entry.note}</span>
					<span class="log-midi">M{entry.midi}</span>
					<span class="log-vel">v{entry.vel}</span>
					<span class="log-freq">{entry.freq}Hz</span>
					<span class="log-clarity">{entry.clarity}%</span>
				</div>
			{/each}
			{#if midiLog.length === 0}
				<div class="log-empty">Play something...</div>
			{/if}
		</div>
	</section>
</div>

<style>
	.debug-page {
		font-family: var(--font-code);
		background: #0a0a0f;
		color: #e0e0e0;
		min-height: 100vh;
		padding: 20px;
		max-width: 680px;
		margin: 0 auto;
	}

	header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		margin-bottom: 16px;
	}

	h1 { font-size: 18px; font-weight: 600; color: #fff; margin: 0; }
	h2 { font-size: 13px; font-weight: 600; color: #888; margin: 0 0 8px 0; text-transform: uppercase; letter-spacing: 0.5px; }

	.status {
		font-size: 11px;
		padding: 3px 8px;
		border-radius: 3px;
		background: #1a1a2e;
		color: #666;
	}
	.status.running { background: #0f2a1f; color: #4ade80; }

	.error { background: #2a0f0f; color: #f87171; padding: 8px; border-radius: 4px; margin-bottom: 12px; font-size: 12px; }

	.controls {
		display: flex;
		gap: 12px;
		align-items: end;
		margin-bottom: 16px;
	}

	.control-group {
		display: flex;
		flex-direction: column;
		gap: 4px;
	}

	.control-group label { font-size: 10px; color: #666; text-transform: uppercase; }
	.control-group select {
		background: #1a1a2e;
		color: #e0e0e0;
		border: 1px solid #333;
		padding: 6px 8px;
		font-size: 12px;
		border-radius: 3px;
		font-family: inherit;
	}

	.start-btn {
		padding: 6px 20px;
		background: #6366f1;
		color: #fff;
		border: none;
		border-radius: 3px;
		font-size: 13px;
		font-weight: 600;
		cursor: pointer;
		font-family: inherit;
	}
	.start-btn:hover { background: #4f46e5; }

	.rec-btn {
		padding: 6px 14px;
		background: #1a1a2e;
		color: #f87171;
		border: 1px solid #f8717140;
		border-radius: 3px;
		font-size: 13px;
		font-weight: 600;
		cursor: pointer;
		font-family: inherit;
	}
	.rec-btn.rec-active { background: #f87171; color: #fff; border-color: #f87171; }

	.download-btn {
		padding: 6px 14px;
		background: #1a2a1f;
		color: #4ade80;
		border: 1px solid #4ade8040;
		border-radius: 3px;
		font-size: 13px;
		font-weight: 600;
		cursor: pointer;
		font-family: inherit;
	}
	.download-btn:hover { background: #0f3a1f; }

	.detection {
		display: flex;
		align-items: center;
		gap: 24px;
		padding: 16px;
		background: #111118;
		border: 1px solid #222;
		border-radius: 6px;
		margin-bottom: 16px;
	}

	.big-note {
		font-size: 48px;
		font-weight: 700;
		color: #fff;
		min-width: 120px;
		text-align: center;
	}

	.stats {
		display: grid;
		grid-template-columns: repeat(5, 1fr);
		gap: 8px;
		flex: 1;
	}

	.stats .label { font-size: 9px; color: #555; text-transform: uppercase; display: block; }
	.stats .value { font-size: 16px; font-weight: 600; color: #00e5cc; }
	.stats .value.sharp { color: #f87171; }
	.stats .value.flat { color: #60a5fa; }

	.graph-section {
		margin-bottom: 16px;
	}

	canvas {
		width: 100%;
		height: auto;
		border: 1px solid #222;
		border-radius: 4px;
		image-rendering: auto;
	}

	.envelope-controls {
		display: flex;
		gap: 16px;
		margin-top: 8px;
	}

	.envelope-controls label {
		font-size: 11px;
		color: #888;
		display: flex;
		align-items: center;
		gap: 6px;
	}

	.envelope-controls input[type="range"] {
		width: 100px;
		accent-color: #6366f1;
	}

	.output-section, .midi-log-section {
		margin-bottom: 16px;
	}

	.output-row {
		display: flex;
		align-items: center;
		gap: 12px;
		margin-bottom: 8px;
	}

	.toggle {
		font-size: 12px;
		display: flex;
		align-items: center;
		gap: 6px;
		cursor: pointer;
	}

	.output-row select {
		background: #1a1a2e;
		color: #e0e0e0;
		border: 1px solid #333;
		padding: 4px 6px;
		font-size: 11px;
		border-radius: 3px;
		font-family: inherit;
	}

	.output-row input[type="range"] {
		width: 80px;
		accent-color: #6366f1;
	}

	.dim { color: #555; font-size: 11px; }

	.clear-btn {
		font-size: 10px;
		padding: 2px 8px;
		background: #1a1a2e;
		border: 1px solid #333;
		color: #888;
		border-radius: 3px;
		cursor: pointer;
		margin-left: 8px;
		font-family: inherit;
	}

	.midi-log {
		background: #111118;
		border: 1px solid #222;
		border-radius: 4px;
		max-height: 200px;
		overflow-y: auto;
		font-size: 11px;
	}

	.log-entry {
		display: flex;
		gap: 8px;
		padding: 4px 8px;
		border-bottom: 1px solid #1a1a1a;
	}

	.log-entry.note-on { color: #4ade80; }
	.log-entry.note-off { color: #666; }
	.log-entry.spike { color: #f87171; text-decoration: line-through; opacity: 0.6; }

	.log-type { width: 28px; font-weight: 600; }
	.log-note { width: 36px; color: #fff; font-weight: 600; }
	.log-midi { width: 32px; color: #888; }
	.log-vel { width: 28px; color: #888; }
	.log-freq { width: 60px; color: #00e5cc; }
	.log-clarity { color: #6366f1; }

	.log-empty { padding: 16px; text-align: center; color: #444; }

	.harmonic-detail { margin-top: 8px; }

	.harmonic-bar {
		height: 6px;
		background: #1a1a2e;
		border-radius: 3px;
		overflow: hidden;
		margin-bottom: 6px;
	}

	.harmonic-fill {
		height: 100%;
		border-radius: 3px;
		transition: width 0.1s, background 0.1s;
	}

	.harmonic-markers-row {
		display: flex;
		flex-wrap: wrap;
		gap: 4px;
		margin-bottom: 6px;
	}

	.harmonic-tag {
		font-size: 10px;
		padding: 2px 6px;
		border-radius: 3px;
		font-family: inherit;
	}

	.harmonic-tag.present { background: #0f2a1f; color: #4ade80; }
	.harmonic-tag.absent { background: #2a0f0f; color: #f87171; opacity: 0.6; }

	.candidate-row {
		margin-bottom: 6px;
		padding: 4px 6px;
		border-radius: 4px;
		background: #111118;
		opacity: 0.6;
	}

	.candidate-row.best {
		opacity: 1;
		border: 1px solid #333;
	}

	.candidate-badge {
		font-size: 12px;
		font-weight: 600;
		padding: 2px 8px;
		border-radius: 3px;
		border: 1px solid;
		display: inline-block;
		margin-bottom: 4px;
	}
</style>
