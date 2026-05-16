/**
 * Guitar Input Store — Reactive Guitar Audio Input State (Svelte 5 Runes)
 *
 * Tracks guitar input configuration (latency, gain, string confidence,
 * technique toggles) and live detection state. Calls the platform adapter
 * to sync device selection and DSP config to the backend.
 */

import { adapter } from '$lib/adapter';
import { GuitarAudioCapture } from '$lib/audio/guitarCapture';
import { detectPitch, frequencyToMidi, midiToNoteName } from '$lib/audio/pitchDetector';

const STORAGE_KEY = 'contrapunk-guitar';

function loadSaved(): Record<string, any> {
	try {
		const raw = localStorage.getItem(STORAGE_KEY);
		return raw ? JSON.parse(raw) : {};
	} catch { return {}; }
}

class GuitarInputStore {
	// -- Config (mirrors GuitarInputConfig in Rust) --
	latencyMs = $state(21);
	gain = $state(1.0);
	stringConfidence = $state(0.4);
	bendsEnabled = $state(true);
	legatoEnabled = $state(true);
	slidesEnabled = $state(true);
	vibratoEnabled = $state(false);

	// -- Audio device selection --
	audioDevices: MediaDeviceInfo[] = $state([]);
	selectedDeviceId = $state<string>('');
	selectedChannel = $state(1); // 1-indexed for display
	maxChannels = $state(2); // depends on selected device

	constructor() {
		const saved = loadSaved();
		if (saved.latencyMs !== undefined) this.latencyMs = saved.latencyMs;
		if (saved.gain !== undefined) this.gain = saved.gain;
		if (saved.stringConfidence !== undefined) this.stringConfidence = saved.stringConfidence;
		if (saved.bendsEnabled !== undefined) this.bendsEnabled = saved.bendsEnabled;
		if (saved.legatoEnabled !== undefined) this.legatoEnabled = saved.legatoEnabled;
		if (saved.slidesEnabled !== undefined) this.slidesEnabled = saved.slidesEnabled;
		if (saved.vibratoEnabled !== undefined) this.vibratoEnabled = saved.vibratoEnabled;
		if (saved.selectedDeviceId) this.selectedDeviceId = saved.selectedDeviceId;
		if (saved.selectedChannel) this.selectedChannel = saved.selectedChannel;
		if (saved.calibrated) this.calibrated = saved.calibrated;
		if (saved.noiseGateEnabled !== undefined) this.noiseGateEnabled = saved.noiseGateEnabled;
		if (saved.noiseGateThreshold !== undefined) this.noiseGateThreshold = saved.noiseGateThreshold;
		if (saved.freqGateEnabled !== undefined) this.freqGateEnabled = saved.freqGateEnabled;
		if (saved.freqGateRange !== undefined) this.freqGateRange = saved.freqGateRange;
	}

	private persist() {
		try {
			localStorage.setItem(STORAGE_KEY, JSON.stringify({
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
				freqGateRange: this.freqGateRange,
			}));
		} catch {}
	}
	/** Manual override for channel count (null = auto-detect from device) */
	manualMaxChannels = $state<number | null>(null);
	audioDeviceError = $state<string>('');

	// -- Live detection state (updated from backend when wired) --
	detecting = $state(false);
	/** The actual channel being used by the audio capture (1-indexed, for display). */
	activeChannel = $state(0);
	currentNote = $state('');
	currentString = $state('');
	currentFret = $state(0);
	confidence = $state(0);
	velocity = $state(0);
	/** Current signal RMS level (0.0-1.0, updated every frame). Non-reactive — only read by canvas. */
	signalLevel = 0;
	/** Current pitch clarity (0.0-1.0, updated every frame). Non-reactive — only read by canvas. */
	signalClarity = 0;

	// -- Gate controls --
	/** Noise gate: reject signals below this RMS threshold. */
	noiseGateEnabled = $state(true);
	noiseGateThreshold = $state(0.01);
	/** Frequency gate: reject detections outside expected guitar range (cents from expected). */
	freqGateEnabled = $state(true);
	freqGateRange = $state(1200); // cents — 1 octave

	// -- Signal history for graphs (rolling buffers) --
	/** Last N amplitude values for the graph. */
	amplitudeHistory: number[] = [];
	/** Last N clarity values for the graph. */
	clarityHistory: number[] = [];
	private static readonly HISTORY_SIZE = 128;

	/** Push a new frame of signal data (called from adapter). */
	pushSignalFrame(rms: number, clarity: number) {
		this.signalLevel = Math.min(1, rms * 5);
		this.signalClarity = clarity;
		this.amplitudeHistory.push(Math.min(1, rms * 5));
		this.clarityHistory.push(clarity);
		if (this.amplitudeHistory.length > GuitarInputStore.HISTORY_SIZE) {
			this.amplitudeHistory.shift();
		}
		if (this.clarityHistory.length > GuitarInputStore.HISTORY_SIZE) {
			this.clarityHistory.shift();
		}
	}

	// -- Calibration state --
	calibrated = $state(false);
	calibrating = $state(false);

	// -- Tuner state --
	tunerActive = $state(false);
	tunerStringIndex = $state(0);
	tunerDetectedNote = $state('');
	tunerDetectedFreq = $state(0);
	tunerCents = $state(0);
	tunerClarity = $state(0);
	tunerStatus = $state<'waiting' | 'sharp' | 'flat' | 'in-tune' | 'holding' | 'done'>('waiting');
	tunerHoldProgress = $state(0); // 0..1
	tunerPhase = $state<'noise-floor' | 'tuning' | 'complete'>('noise-floor');
	tunerNoiseProgress = $state(0); // 0..1

	/** Toggle a technique on/off by name. */
	toggleTechnique(technique: 'bends' | 'legato' | 'slides' | 'vibrato') {
		switch (technique) {
			case 'bends':
				this.bendsEnabled = !this.bendsEnabled;
				break;
			case 'legato':
				this.legatoEnabled = !this.legatoEnabled;
				break;
			case 'slides':
				this.slidesEnabled = !this.slidesEnabled;
				break;
			case 'vibrato':
				this.vibratoEnabled = !this.vibratoEnabled;
				break;
		}
		this.persist();
	}

	/** Noise floor RMS measured during calibration. */
	noiseFloorRms = $state(0);
	/** Status message shown after calibration. */
	calibrationStatus = $state('');

	/** Open strings for standard tuning. */
	static readonly OPEN_STRINGS = [
		{ name: 'Low E', note: 'E2', midi: 40, freq: 82.41 },
		{ name: 'A', note: 'A2', midi: 45, freq: 110.00 },
		{ name: 'D', note: 'D3', midi: 50, freq: 146.83 },
		{ name: 'G', note: 'G3', midi: 55, freq: 196.00 },
		{ name: 'B', note: 'B3', midi: 59, freq: 246.94 },
		{ name: 'High E', note: 'E4', midi: 64, freq: 329.63 },
	];

	private tunerCapture: GuitarAudioCapture | null = null;
	private tunerAnimFrame: number | null = null;
	private inTuneSince: number | null = null;

	/** In-tune threshold in cents. */
	private static readonly IN_TUNE_CENTS = 8;
	/** How long to hold in-tune before auto-advancing (ms). */
	private static readonly IN_TUNE_HOLD_MS = 1500;

	/**
	 * Start the full tuner calibration flow.
	 * Phase 1: Noise floor measurement (3s).
	 * Phase 2: Per-string tuning with live pitch detection.
	 */
	async startCalibration() {
		if (this.calibrating) return;

		// Calibration uses getUserMedia + JS pitch detection in both
		// browser and Tauri webview — the webview supports Web Audio APIs.
		this.calibrating = true;
		this.calibrated = false;
		this.tunerActive = true;
		this.tunerPhase = 'noise-floor';
		this.tunerStringIndex = 0;
		this.tunerNoiseProgress = 0;
		this.tunerStatus = 'waiting';
		this.tunerHoldProgress = 0;
		this.calibrationStatus = 'Measuring noise floor...';

		try {
			// Phase 1: Noise floor
			const capture = new GuitarAudioCapture();
			const noisePromise = capture.measureNoiseFloor(this.selectedDeviceId, 3000, this.selectedChannel - 1);

			// Animate progress bar for noise floor
			const noiseStart = Date.now();
			const noiseTimer = setInterval(() => {
				const elapsed = Date.now() - noiseStart;
				this.tunerNoiseProgress = Math.min(1, elapsed / 3000);
			}, 50);

			const avgRms = await noisePromise;
			clearInterval(noiseTimer);
			this.tunerNoiseProgress = 1;
			this.noiseFloorRms = avgRms;
			this.calibrationStatus = `Noise floor: ${(avgRms * 1000).toFixed(1)} mRMS`;

			// Phase 2: Per-string tuning
			this.tunerPhase = 'tuning';
			this.tunerStringIndex = 0;
			await this.startTunerCapture();
		} catch (err) {
			this.calibrationStatus =
				err instanceof Error ? `Calibration failed: ${err.message}` : 'Calibration failed';
			this.calibrated = false;
			this.tunerActive = false;
			this.calibrating = false;
		}
	}

	/** Start audio capture for the tuner phase using raw JS pitch detection. */
	private async startTunerCapture() {
		this.inTuneSince = null;

		const channelIndex = this.selectedChannel - 1;
		const constraints: MediaStreamConstraints = {
			audio: {
				...(this.selectedDeviceId ? { deviceId: { exact: this.selectedDeviceId } } : {}),
				echoCancellation: false,
				noiseSuppression: false,
				autoGainControl: false,
				channelCount: { ideal: 32 },
			},
		};

		const TUNER_BUFFER_SIZE = 2048;

		const stream = await navigator.mediaDevices.getUserMedia(constraints);
		this.tunerStream = stream;
		const ctx = new AudioContext({ sampleRate: 48000 });
		this.tunerContext = ctx;
		const source = ctx.createMediaStreamSource(stream);
		const inputChannels = Math.max(channelIndex + 1, source.channelCount);

		/** Process a buffer of tuner samples (shared by both paths). */
		const processTunerSamples = (samples: Float32Array) => {
			if (!this.tunerActive || this.tunerPhase !== 'tuning') return;

			// Compute RMS
			let sum = 0;
			for (let i = 0; i < samples.length; i++) sum += samples[i] * samples[i];
			const rms = Math.sqrt(sum / samples.length);

			// Run pitch detection
			const result = detectPitch(samples, ctx.sampleRate);

			const target = GuitarInputStore.OPEN_STRINGS[this.tunerStringIndex];
			if (!target) return;

			if (!result || rms < this.noiseFloorRms * 2) {
				this.tunerDetectedNote = '';
				this.tunerDetectedFreq = 0;
				this.tunerCents = 0;
				this.tunerClarity = 0;
				this.tunerStatus = 'waiting';
				this.tunerHoldProgress = 0;
				this.inTuneSince = null;
				return;
			}

			const midiInfo = frequencyToMidi(result.frequency);
			const noteName = midiToNoteName(midiInfo.note);
			this.tunerDetectedNote = noteName;
			this.tunerDetectedFreq = result.frequency;
			this.tunerClarity = result.clarity;

			// Calculate cents relative to the target frequency
			const centsFromTarget = 1200 * Math.log2(result.frequency / target.freq);
			this.tunerCents = Math.round(centsFromTarget);

			const absCents = Math.abs(this.tunerCents);

			// More than 2 octaves away = noise/garbage, ignore completely
			if (absCents > 2400) {
				this.tunerStatus = 'waiting';
				this.tunerHoldProgress = 0;
				this.inTuneSince = null;
				return;
			}

			if (absCents <= GuitarInputStore.IN_TUNE_CENTS) {
				this.tunerStatus = 'holding';
				if (this.inTuneSince === null) {
					this.inTuneSince = Date.now();
				}
				const held = Date.now() - this.inTuneSince;
				this.tunerHoldProgress = Math.min(1, held / GuitarInputStore.IN_TUNE_HOLD_MS);

				if (held >= GuitarInputStore.IN_TUNE_HOLD_MS) {
					this.tunerStatus = 'in-tune';
					this.advanceTunerString();
				}
			} else {
				this.inTuneSince = null;
				this.tunerHoldProgress = 0;
				this.tunerStatus = this.tunerCents > 0 ? 'sharp' : 'flat';
			}
		};

		if (ctx.audioWorklet) {
			// AudioWorklet path — accumulate 128-sample chunks into TUNER_BUFFER_SIZE
			await ctx.audioWorklet.addModule('/audio-capture-processor.js');
			const worklet = new AudioWorkletNode(ctx, 'audio-capture-processor', {
				numberOfInputs: 1,
				numberOfOutputs: 0,
				channelCount: inputChannels,
				channelCountMode: 'explicit',
				channelInterpretation: 'discrete',
				processorOptions: { channelIndex },
			});
			this.tunerWorklet = worklet;

			const accumBuf = new Float32Array(TUNER_BUFFER_SIZE);
			let accumPos = 0;

			worklet.port.onmessage = (event: MessageEvent) => {
				const chunk: Float32Array = event.data.samples;
				let srcOffset = 0;
				while (srcOffset < chunk.length) {
					const remaining = TUNER_BUFFER_SIZE - accumPos;
					const toCopy = Math.min(remaining, chunk.length - srcOffset);
					accumBuf.set(chunk.subarray(srcOffset, srcOffset + toCopy), accumPos);
					accumPos += toCopy;
					srcOffset += toCopy;

					if (accumPos >= TUNER_BUFFER_SIZE) {
						processTunerSamples(accumBuf);
						accumPos = 0;
					}
				}
			};

			source.connect(worklet);
		} else {
			// ScriptProcessorNode fallback
			const proc = ctx.createScriptProcessor(TUNER_BUFFER_SIZE, inputChannels, 1);
			proc.channelCountMode = 'explicit';
			proc.channelInterpretation = 'discrete';
			this.tunerProcessor = proc;

			proc.onaudioprocess = (event) => {
				const input = event.inputBuffer;
				const ch = Math.min(channelIndex, input.numberOfChannels - 1);
				processTunerSamples(input.getChannelData(ch));
			};

			source.connect(proc);
			proc.connect(ctx.destination);
		}
	}

	// Tuner audio resources
	private tunerStream: MediaStream | null = null;
	private tunerContext: AudioContext | null = null;
	private tunerWorklet: AudioWorkletNode | null = null;
	private tunerProcessor: ScriptProcessorNode | null = null; // fallback

	/** Advance to the next string, or finish tuning. */
	private advanceTunerString() {
		this.inTuneSince = null;
		this.tunerHoldProgress = 0;

		if (this.tunerStringIndex >= 5) {
			// All 6 strings done
			this.tunerPhase = 'complete';
			this.calibrated = true;
			this.calibrating = false;
			this.calibrationStatus = 'Tuning complete! Ready to play.';
			this.stopTunerCapture();

			// Auto-close tuner UI after 2 seconds so main routing can take over
			setTimeout(() => {
				this.tunerActive = false;
				this.tunerPhase = 'noise-floor';
			}, 2000);
			// Auto-close after 2 seconds
			setTimeout(() => {
				if (this.tunerPhase === 'complete') {
					this.tunerActive = false;
				}
			}, 2000);
		} else {
			this.tunerStringIndex++;
			this.tunerStatus = 'waiting';
			this.tunerDetectedNote = '';
			this.tunerCents = 0;
		}
	}

	/** Skip the current tuner string. */
	skipTunerString() {
		if (!this.tunerActive || this.tunerPhase !== 'tuning') return;
		this.advanceTunerString();
	}

	/** Cancel the tuner and return to normal view. */
	cancelTuner() {
		this.stopTunerCapture();
		this.tunerActive = false;
		this.tunerPhase = 'noise-floor';
		this.calibrating = false;
		this.calibrationStatus = this.calibrated ? 'Calibration preserved' : '';
	}

	/** Stop the tuner audio capture. */
	private async stopTunerCapture() {
		if (this.tunerWorklet) {
			this.tunerWorklet.port.onmessage = null;
			this.tunerWorklet.disconnect();
			this.tunerWorklet = null;
		}
		if (this.tunerProcessor) {
			this.tunerProcessor.onaudioprocess = null;
			this.tunerProcessor.disconnect();
			this.tunerProcessor = null;
		}
		if (this.tunerStream) {
			this.tunerStream.getTracks().forEach(t => t.stop());
			this.tunerStream = null;
		}
		if (this.tunerContext) {
			await this.tunerContext.close();
			this.tunerContext = null;
		}
		if (this.tunerCapture) {
			await this.tunerCapture.stop();
			this.tunerCapture = null;
		}
		if (this.tunerAnimFrame !== null) {
			cancelAnimationFrame(this.tunerAnimFrame);
			this.tunerAnimFrame = null;
		}
	}

	/** Set latency with bounds checking and sync. */
	setLatency(value: number) {
		this.latencyMs = Math.max(1, Math.min(50, value));
		this.syncConfig(); this.persist();
	}

	/** Set gain with bounds checking and sync. */
	setGain(value: number) {
		this.gain = Math.max(0.1, Math.min(2.0, Math.round(value * 20) / 20));
		this.syncConfig(); this.persist();
	}

	/** Set string confidence with bounds checking and sync. */
	setStringConfidence(value: number) {
		this.stringConfidence = Math.max(0.1, Math.min(1.0, Math.round(value * 20) / 20));
		this.syncConfig(); this.persist();
	}

	/** Enumerate available audio input devices via the Web Audio API. */
	async enumerateAudioDevices() {
		if (typeof navigator === 'undefined' || !navigator.mediaDevices) {
			this.audioDeviceError = 'Audio devices not available on this surface';
			return;
		}

		try {
			// Request permission first so device labels are populated
			const stream = await navigator.mediaDevices.getUserMedia({ audio: true });
			// Stop the stream immediately — we only needed it for permission
			stream.getTracks().forEach((t) => t.stop());

			const allDevices = await navigator.mediaDevices.enumerateDevices();
			this.audioDevices = allDevices.filter((d) => d.kind === 'audioinput');
			this.audioDeviceError = '';

			// Auto-select: prefer Audient iD14, otherwise first device
			if (this.audioDevices.length > 0 && !this.selectedDeviceId) {
				const audient = this.audioDevices.find((d) =>
					d.label.toLowerCase().includes('audient')
				);
				this.selectDevice(audient ? audient.deviceId : this.audioDevices[0].deviceId);
			}
		} catch (err) {
			// Translate the raw browser/system error into something a
			// musician can act on. Branch on the standard DOMException
			// names from the MediaDevices spec; fall back to the raw
			// message only when we can't classify it. Messages stay
			// OS-agnostic so Linux/Windows users don't read macOS
			// instructions.
			const raw = err instanceof Error ? err : null;
			const name = raw?.name ?? '';
			const msg = raw?.message ?? String(err);
			if (
				name === 'NotAllowedError' ||
				name === 'SecurityError' ||
				/denied permission|not allowed|permission/i.test(msg)
			) {
				this.audioDeviceError =
					"Microphone access denied. Open your system's microphone privacy settings, allow Contrapunk, then re-open the app.";
			} else if (
				name === 'NotFoundError' ||
				/no.*device|not found|no audio/i.test(msg)
			) {
				this.audioDeviceError =
					'No audio input device detected. Plug in your interface, then click R to refresh.';
			} else if (
				name === 'NotReadableError' ||
				name === 'AbortError' ||
				/in use|read.*err|busy/i.test(msg)
			) {
				this.audioDeviceError =
					'Audio device is in use or unreadable. Close the other app using it, then click R to refresh.';
			} else if (
				name === 'OverconstrainedError' ||
				/constrain/i.test(msg)
			) {
				this.audioDeviceError =
					"This device doesn't support the requested audio format. Try a different input device.";
			} else if (name === 'TypeError') {
				this.audioDeviceError =
					'Audio request was rejected as malformed. This is a Contrapunk bug — please report it with your OS + browser version.';
			} else {
				this.audioDeviceError = `Couldn't access the audio device (${msg}).`;
			}
		}
	}

	/** Select an audio input device and probe its channel count. */
	selectDevice(deviceId: string) {
		this.selectedDeviceId = deviceId;
		this.selectedChannel = 1;
		this.persist();

		// Try to determine channel count by opening a stream with the device
		this.probeChannelCount(deviceId);
	}

	/** Select an input channel (1-indexed). */
	selectChannel(channel: number) {
		this.selectedChannel = channel;
		this.persist();
	}

	/** Set a manual channel count override, or null to auto-detect. */
	setManualMaxChannels(count: number | null) {
		this.manualMaxChannels = count;
		if (count != null && count > 0) {
			this.maxChannels = count;
		} else if (this.selectedDeviceId) {
			// Re-probe to restore auto-detected value
			this.probeChannelCount(this.selectedDeviceId);
		}
	}

	/** Probe the max channel count for a given device. */
	private async probeChannelCount(deviceId: string) {
		// Skip probing if manual override is set
		if (this.manualMaxChannels != null && this.manualMaxChannels > 0) {
			this.maxChannels = this.manualMaxChannels;
			return;
		}

		if (typeof navigator === 'undefined' || !navigator.mediaDevices) return;

		try {
			const stream = await navigator.mediaDevices.getUserMedia({
				audio: {
					deviceId: { exact: deviceId },
					// Request all available channels
					channelCount: { ideal: 32 }
				}
			});
			const track = stream.getAudioTracks()[0];
			if (track) {
				const settings = track.getSettings();
				this.maxChannels = settings.channelCount ?? 2;
			}
			stream.getTracks().forEach((t) => t.stop());
		} catch {
			// Fallback: assume stereo
			this.maxChannels = 2;
		}
	}

	// -- Backend audio devices (from cpal via adapter) --
	backendAudioDevices: string[] = $state([]);

	/** Load audio input devices from the backend (cpal). */
	async loadAudioDevices() {
		try {
			this.backendAudioDevices = await adapter.listAudioDevices();
		} catch {
			// Backend may not support audio device listing (e.g. WASM)
			this.backendAudioDevices = [];
		}
	}

	/** Sync the current guitar DSP config to the backend. */
	async syncConfig() {
		try {
			await adapter.setGuitarConfig({
				latencyMs: this.latencyMs,
				gain: this.gain,
				stringConfidence: this.stringConfidence,
				bends: this.bendsEnabled,
				legato: this.legatoEnabled,
				slides: this.slidesEnabled,
				vibrato: this.vibratoEnabled,
			});
		} catch {
			// Silently ignore — backend may not be ready yet
		}
	}

	/** Sync the selected device and channel to the backend. */
	async syncDevice() {
		try {
			// For Tauri: send the device label (cpal name) instead of browser deviceId hash,
			// and convert 1-indexed UI channel to 0-indexed for cpal.
			const device = this.audioDevices.find(d => d.deviceId === this.selectedDeviceId);
			const deviceName = device?.label || this.selectedDeviceId;
			const channel0 = Math.max(0, this.selectedChannel - 1);
			await adapter.setGuitarDevice(deviceName, channel0);
		} catch {
			// Silently ignore — backend may not be ready yet
		}
	}
}

export const guitar = new GuitarInputStore();
