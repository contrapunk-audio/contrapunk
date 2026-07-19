import assert from 'node:assert/strict';
import test from 'node:test';
import { migrateLegacyPreset } from './persistence.ts';

test('migrates legacy user preset without carrying key or performance environment', () => {
	const migrated = migrateLegacyPreset({
		id: 'user-1',
		name: 'Old setup',
		builtIn: false,
		key: 'F#',
		scaleMode: 'Dorian',
		mode: 'DiatonicThirds',
		voiceCount: 3,
		voicePosition: 2,
		voiceLeadingEnabled: true,
		voiceLeadingStyle: 'Jazz',
		octaveMode: 'Spread',
		companionEnabled: true,
		canonEnabled: true,
		imitativeForm: 'free_imitation',
		canonVoices: [{ delay_beats: 1, transpose_degrees: 2, time_ratio: 0.5 }],
		counterpoint: { enabled: true, species: 'Species2', transpose_degrees: -2, prefer_above: false }
	});

	assert.ok(migrated);
	assert.equal(migrated.schemaVersion, 2);
	assert.equal(migrated.family, 'custom');
	assert.equal(migrated.config.harmony.scaleMode, 'Dorian');
	assert.equal(migrated.config.companion.canon.voices[0].timeRatio, 0.5);
	assert.equal(migrated.config.companion.counterpoint.species, 'Species2');
	assert.equal(migrated.config.companion.counterpoint.preferAbove, false);
	assert.equal('key' in migrated.config, false);
	assert.deepEqual(migrated.config.mix, { input: 1, harmony: 1, canon: 1, counterpoint: 1 });
	assert.ok(migrated.requirements.includes('free_imitation'));
	assert.ok(migrated.requirements.includes('species_counterpoint'));
});

test('rejects built-ins and malformed legacy values', () => {
	assert.equal(migrateLegacyPreset({ builtIn: true }), null);
	assert.equal(migrateLegacyPreset({ builtIn: false, id: 'bad' }), null);
	assert.equal(migrateLegacyPreset(null), null);
});
