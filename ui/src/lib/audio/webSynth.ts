/**
 * Minimal Web Audio API synthesizer for browser audio output.
 *
 * Plays MIDI note numbers as sine oscillators with a simple gain envelope.
 * Used by the WASM adapter so browser users can hear harmony output without
 * external MIDI hardware.
 */
export class WebSynth {
	private ctx: AudioContext;
	private voices: Map<number, { osc: OscillatorNode; gain: GainNode }>;
	private masterGain: GainNode;

	constructor() {
		this.ctx = new AudioContext();
		this.voices = new Map();
		this.masterGain = this.ctx.createGain();
		this.masterGain.gain.value = 0.3;
		this.masterGain.connect(this.ctx.destination);
	}

	noteOn(note: number, velocity: number): void {
		// Kill existing voice for this note to avoid double-triggers
		this.noteOff(note);

		const freq = 440 * Math.pow(2, (note - 69) / 12);
		const osc = this.ctx.createOscillator();
		osc.type = 'sine';
		osc.frequency.value = freq;

		const noteGain = this.ctx.createGain();
		noteGain.gain.value = (velocity / 127) * 0.5;

		osc.connect(noteGain);
		noteGain.connect(this.masterGain);
		osc.start();
		this.voices.set(note, { osc, gain: noteGain });

		// Auto-release envelope: fade out over ~200ms
		noteGain.gain.setTargetAtTime(0, this.ctx.currentTime + 0.01, 0.05);
	}

	noteOff(note: number): void {
		const voice = this.voices.get(note);
		if (voice) {
			try {
				voice.osc.stop();
			} catch {
				/* already stopped */
			}
			try {
				voice.osc.disconnect();
				voice.gain.disconnect();
			} catch {
				/* already disconnected */
			}
			this.voices.delete(note);
		}
	}

	/** Stop all active voices without closing the AudioContext. */
	silence(): void {
		for (const [note] of this.voices) {
			this.noteOff(note);
		}
	}

	setVolume(v: number): void {
		this.masterGain.gain.value = v;
	}

	resume(): Promise<void> {
		return this.ctx.resume();
	}

	destroy(): void {
		for (const [note] of this.voices) {
			this.noteOff(note);
		}
		this.ctx.close();
	}
}
