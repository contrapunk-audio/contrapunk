import assert from 'node:assert/strict';
import test from 'node:test';
import {
	BUILT_IN_ARRANGEMENT_PRESETS,
	MENSURATION_WEB_PRESET,
	MODAL_LINEWORK_PRESET
} from './catalog.ts';
import {
	arrangementConfigCapabilities,
	validateArrangementConfig,
	validateArrangementPreset
} from './presets.ts';

test('catalog exposes 50 unique immutable built-ins and only approved baselines', () => {
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
		['02-modal-linework', '04-mensuration-web']
	);
});

test('Mensuration Web uses causal single-line proportional followers', () => {
	assert.equal(MENSURATION_WEB_PRESET.play.transportRequired, true);
	assert.deepEqual(MENSURATION_WEB_PRESET.requirements, ['free_imitation']);
	assert.equal(MENSURATION_WEB_PRESET.config.harmony.mode, 'PassThrough');
	assert.equal(MENSURATION_WEB_PRESET.config.harmony.voiceCount, 1);
	assert.equal(MENSURATION_WEB_PRESET.config.companion.canon.form, 'free_imitation');
	assert.deepEqual(
		MENSURATION_WEB_PRESET.config.companion.canon.voices.map((voice) => voice.timeRatio),
		[1, 1.5, 2]
	);
	assert.ok(
		MENSURATION_WEB_PRESET.config.companion.canon.voices.every(
			(voice) => voice.harmonyMode === 'PassThrough' && voice.voiceCount === 1
		)
	);
	assert.deepEqual(validateArrangementPreset(MENSURATION_WEB_PRESET), []);

	const impossibleLiveDiminution = {
		...MENSURATION_WEB_PRESET,
		config: {
			...MENSURATION_WEB_PRESET.config,
			companion: {
				...MENSURATION_WEB_PRESET.config.companion,
				canon: {
					...MENSURATION_WEB_PRESET.config.companion.canon,
					voices: MENSURATION_WEB_PRESET.config.companion.canon.voices.map((voice, index) =>
						index === 0 ? { ...voice, timeRatio: 0.5 } : voice
					)
				}
			}
		}
	};
	assert.deepEqual(validateArrangementPreset(impossibleLiveDiminution), [
		'Live proportional ratios below 1 require phrase capture'
	]);
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
