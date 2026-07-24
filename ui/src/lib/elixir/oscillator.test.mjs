import assert from 'node:assert/strict';
import test from 'node:test';
import {
	DEFAULT_OSCILLATOR_STATE,
	PHASE_DISTORTIONS,
	SPECTRAL_MORPHS,
	UNISON_STYLES,
	createOscillatorState
} from './oscillator.ts';

test('uses the Elixir oscillator enum sets and resets to independent engine defaults', () => {
	assert.equal(SPECTRAL_MORPHS.length, 12);
	assert.equal(PHASE_DISTORTIONS.length, 12);
	assert.equal(UNISON_STYLES.length, 11);

	const edited = createOscillatorState();
	edited.spectralMorph = 'skew';
	edited.phaseDistortion = 'sync';
	edited.unisonVoices = 16;

	assert.deepEqual(createOscillatorState(), DEFAULT_OSCILLATOR_STATE);
	assert.equal(DEFAULT_OSCILLATOR_STATE.spectralMorph, 'passthrough');
	assert.equal(DEFAULT_OSCILLATOR_STATE.phaseDistortion, 'off');
	assert.equal(DEFAULT_OSCILLATOR_STATE.unisonStyle, 'centered');
	assert.equal(DEFAULT_OSCILLATOR_STATE.unisonVoices, 1);
	assert.equal(DEFAULT_OSCILLATOR_STATE.unisonDetuneCents, 8);
});
