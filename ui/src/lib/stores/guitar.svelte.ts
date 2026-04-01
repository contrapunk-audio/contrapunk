/**
 * Guitar Input Store — Reactive Guitar Audio Input State (Svelte 5 Runes)
 *
 * Tracks guitar input configuration (latency, gain, string confidence,
 * technique toggles) and live detection state. UI-only for now; the
 * actual audio processing and WASM bridge will be connected in a later phase.
 */

class GuitarInputStore {
	// -- Config (mirrors GuitarInputConfig in Rust) --
	latencyMs = $state(21);
	gain = $state(1.0);
	stringConfidence = $state(0.85);
	bendsEnabled = $state(true);
	legatoEnabled = $state(true);
	slidesEnabled = $state(true);
	vibratoEnabled = $state(false);

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
}

export const guitar = new GuitarInputStore();
