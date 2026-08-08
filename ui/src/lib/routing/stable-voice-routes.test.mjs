import assert from 'node:assert/strict';
import test from 'node:test';
import { stableVoiceRoutes } from './stable-voice-routes.mjs';

const base = {
	voiceCount: 3,
	voicePosition: 1,
	harmonyEnabled: false,
	companionEnabled: false,
	canonVoiceCount: 2,
	canonEnabled: false,
	counterpointEnabled: false,
	phraseAware: false,
	patternLowEnabled: false,
	patternCounterEnabled: false
};

test('stable routing keeps every configured route visible while inactive', () => {
	const rows = stableVoiceRoutes(base);
	assert.deepEqual(rows.map(({ route }) => route), [
		'input',
		'harmony:0',
		'harmony:1',
		'harmony:2',
		'canon:0',
		'canon:1',
		'counterpoint:0',
		'counterpoint:1',
		'pattern_low',
		'pattern_counter'
	]);
	assert.deepEqual(rows.filter(({ active }) => active).map(({ route }) => route), ['input']);
});

test('stable routing marks only currently producing parts active', () => {
	const rows = stableVoiceRoutes({
		...base,
		harmonyEnabled: true,
		companionEnabled: true,
		canonEnabled: true,
		counterpointEnabled: true,
		phraseAware: true,
		patternLowEnabled: true,
		patternCounterEnabled: true
	});
	assert.deepEqual(rows.filter(({ active }) => !active).map(({ route }) => route), ['harmony:1']);
});
