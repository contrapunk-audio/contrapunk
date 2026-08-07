import assert from 'node:assert/strict';
import test from 'node:test';
import { appliedMixGain } from './synth-mix.mjs';

test('mute and solo determine each role effective gain', () => {
	const levels = [0.25, 0.5, 0.75, 1];
	assert.equal(appliedMixGain(levels, [false, false, false, false], null, 1), 0.5);
	assert.equal(appliedMixGain(levels, [false, true, false, false], null, 1), 0);
	assert.equal(appliedMixGain(levels, [false, false, false, false], 2, 1), 0);
	assert.equal(appliedMixGain(levels, [false, false, false, false], 2, 2), 0.75);
});
