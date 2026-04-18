/**
 * AudioWorklet processor for guitar audio capture.
 *
 * Runs on the audio rendering thread and forwards raw Float32Array
 * samples to the main thread via MessagePort.  The main thread then
 * feeds them into the WASM DSP pipeline (WASM cannot run inside a
 * worklet because it lives on a separate thread without access to the
 * main-thread module instance).
 *
 * The worklet receives a `channelIndex` parameter so it can pick the
 * correct input channel (e.g. channel 3 on a multi-channel interface).
 */

class AudioCaptureProcessor extends AudioWorkletProcessor {
	constructor(options) {
		super();
		// Which input channel to forward (default 0)
		this._channelIndex =
			(options.processorOptions && options.processorOptions.channelIndex) || 0;
	}

	process(inputs /*, outputs, parameters */) {
		const input = inputs[0]; // first input
		if (!input || input.length === 0) return true;

		const ch = Math.min(this._channelIndex, input.length - 1);
		const channelData = input[ch];
		if (channelData && channelData.length > 0) {
			// Copy into a plain Float32Array so it can be transferred
			this.port.postMessage(
				{ samples: channelData.slice(), channel: ch },
			);
		}

		return true; // keep processor alive
	}
}

registerProcessor('audio-capture-processor', AudioCaptureProcessor);
