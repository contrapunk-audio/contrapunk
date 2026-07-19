import assert from 'node:assert/strict';
import test from 'node:test';
import { BUILT_IN_ARRANGEMENT_PRESETS, MODAL_LINEWORK_PRESET } from './catalog.ts';
import { arrangementConfigCapabilities, validateArrangementConfig } from './presets.ts';

test('catalog exposes 50 unique immutable built-in records with one approved baseline', () => {
	assert.equal(BUILT_IN_ARRANGEMENT_PRESETS.length, 50);
	assert.equal(new Set(BUILT_IN_ARRANGEMENT_PRESETS.map((preset) => preset.id)).size, 50);
	assert.equal(new Set(BUILT_IN_ARRANGEMENT_PRESETS.map((preset) => preset.name)).size, 50);

	for (const preset of BUILT_IN_ARRANGEMENT_PRESETS) {
		assert.equal(preset.builtIn, true);
		assert.ok(preset.result.length > 0);
		assert.ok(preset.play.prompt.length > 0);
		assert.ok(preset.references.length > 0);
		assert.deepEqual(validateArrangementConfig(preset.config), []);
	}

	assert.deepEqual(
		BUILT_IN_ARRANGEMENT_PRESETS.filter((preset) => preset.researchStatus === 'approved').map(
			(preset) => preset.id
		),
		['02-modal-linework']
	);
});

test('Modal Linework matches its bounded research synthesis', () => {
	assert.equal(MODAL_LINEWORK_PRESET.config.harmony.scaleMode, 'Dorian');
	assert.equal(MODAL_LINEWORK_PRESET.config.harmony.mode, 'StrictCounterpoint');
	assert.equal(MODAL_LINEWORK_PRESET.config.harmony.voiceCount, 4);
	assert.equal(MODAL_LINEWORK_PRESET.config.harmony.voicePosition, 1);
	assert.equal(MODAL_LINEWORK_PRESET.config.harmony.voiceLeadingStyle, 'Palestrina');
	assert.equal(MODAL_LINEWORK_PRESET.config.harmony.octaveMode, 'None');
	assert.equal(MODAL_LINEWORK_PRESET.config.harmony.interchangeEnabled, false);
	assert.equal(MODAL_LINEWORK_PRESET.config.companion.enabled, false);
	assert.equal(MODAL_LINEWORK_PRESET.play.transportRequired, false);
	assert.deepEqual(MODAL_LINEWORK_PRESET.requirements, ['harmony', 'voice_leading']);
	assert.deepEqual(arrangementConfigCapabilities(MODAL_LINEWORK_PRESET.config), [
		'harmony',
		'voice_leading'
	]);

	for (const preserved of [
		'key',
		'tonic',
		'bpm',
		'timeSignature',
		'device',
		'routing',
		'sound',
		'masterLevel',
		'mute',
		'solo',
		'plugins',
		'transport'
	]) {
		assert.equal(preserved in MODAL_LINEWORK_PRESET.config, false);
	}
});
