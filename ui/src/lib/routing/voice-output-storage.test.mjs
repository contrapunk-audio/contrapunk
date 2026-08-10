import test from 'node:test';
import assert from 'node:assert/strict';
import { buildStoredVoiceRoutes } from './voice-output-storage.mjs';

test('editing one route preserves unavailable device-name assignments', () => {
	const stored = buildStoredVoiceRoutes(
		{ input: { kind: 'off' } },
		[{ index: 4, name: 'Available Bus' }],
		{ 'canon:3': { kind: 'midi_port', deviceName: 'Disconnected Synth' } }
	);

	assert.deepEqual(stored, {
		input: { kind: 'off' },
		'canon:3': { kind: 'midi_port', deviceName: 'Disconnected Synth' }
	});
});

test('a newly available assignment replaces its stale saved name', () => {
	const stored = buildStoredVoiceRoutes(
		{ 'canon:3': { kind: 'midi_port', port: 4 } },
		[{ index: 4, name: 'Reconnected Synth' }],
		{ 'canon:3': { kind: 'midi_port', deviceName: 'Old Name' } }
	);

	assert.deepEqual(stored['canon:3'], {
		kind: 'midi_port',
		deviceName: 'Reconnected Synth'
	});
});
