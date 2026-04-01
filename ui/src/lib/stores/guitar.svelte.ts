/**
 * Guitar Input Store — Reactive Guitar Audio Input State (Svelte 5 Runes)
 *
 * Tracks guitar input configuration (latency, gain, string confidence,
 * technique toggles) and live detection state. Calls the platform adapter
 * to sync device selection and DSP config to the backend.
 */

import { adapter } from '$lib/adapter';

class GuitarInputStore {
	// -- Config (mirrors GuitarInputConfig in Rust) --
	latencyMs = $state(21);
	gain = $state(1.0);
	stringConfidence = $state(0.85);
	bendsEnabled = $state(true);
	legatoEnabled = $state(true);
	slidesEnabled = $state(true);
	vibratoEnabled = $state(false);

	// -- Audio device selection --
	audioDevices: MediaDeviceInfo[] = $state([]);
	selectedDeviceId = $state<string>('');
	selectedChannel = $state(1); // 1-indexed for display
	maxChannels = $state(2); // depends on selected device
	audioDeviceError = $state<string>('');

	// -- Live detection state (updated from backend when wired) --
	detecting = $state(false);
	currentNote = $state('');
	currentString = $state('');
	currentFret = $state(0);
	confidence = $state(0);
	velocity = $state(0);

	// -- Calibration state --
	calibrated = $state(false);
	calibrating = $state(false);

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
	}

	/** Start calibration flow (placeholder — will be wired to backend). */
	startCalibration() {
		console.log('[contrapunk] Guitar calibration requested (not yet wired to backend)');
	}

	/** Enumerate available audio input devices via the Web Audio API. */
	async enumerateAudioDevices() {
		if (typeof navigator === 'undefined' || !navigator.mediaDevices) {
			this.audioDeviceError = 'Audio devices not available';
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
			this.audioDeviceError =
				err instanceof Error ? err.message : 'Failed to enumerate audio devices';
		}
	}

	/** Select an audio input device and probe its channel count. */
	selectDevice(deviceId: string) {
		this.selectedDeviceId = deviceId;
		this.selectedChannel = 1;

		// Try to determine channel count by opening a stream with the device
		this.probeChannelCount(deviceId);
	}

	/** Select an input channel (1-indexed). */
	selectChannel(channel: number) {
		this.selectedChannel = channel;
	}

	/** Probe the max channel count for a given device. */
	private async probeChannelCount(deviceId: string) {
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
			await adapter.setGuitarDevice(
				this.selectedDeviceId,
				this.selectedChannel,
			);
		} catch {
			// Silently ignore — backend may not be ready yet
		}
	}
}

export const guitar = new GuitarInputStore();
