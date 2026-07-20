import assert from 'node:assert/strict';
import test from 'node:test';
import {
	BEBOP_CHASE_PRESET,
	BUILT_IN_ARRANGEMENT_PRESETS,
	COLOR_MODE_WINDOWS_PRESET,
	CRYSTAL_CHORALE_PRESET,
	HOLLOW_CHOIR_PRESET,
	MENSURATION_WEB_PRESET,
	MODAL_LINEWORK_PRESET,
	PIXEL_TRIO_PRESET,
	PLANED_CATHEDRAL_PRESET,
	QUARTAL_COLOSSUS_PRESET,
	SIXTH_DIMINISHED_CONVEYOR_PRESET,
	STRETTO_ENGINE_PRESET,
	SUSPENSION_GARLAND_PRESET
} from './catalog.ts';
import {
	arrangementConfigCapabilities,
	missingArrangementCapabilities,
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
		[
			'02-modal-linework',
			'04-mensuration-web',
			'07-stretto-engine',
			'08-suspension-garland',
			'12-planed-cathedral',
			'14-color-mode-windows',
			'23-sixth-diminished-conveyor',
			'25-bebop-chase',
			'27-quartal-colossus',
			'43-hollow-choir',
			'48-crystal-chorale'
		]
	);
});

test('Stretto Engine contracts entry gaps with strict single-line followers', () => {
	const voices = STRETTO_ENGINE_PRESET.config.companion.canon.voices;
	const delays = voices.map((voice) => voice.delayBeats);
	const entries = [0, ...delays];
	const gaps = entries.slice(1).map((entry, index) => entry - entries[index]);

	assert.deepEqual(STRETTO_ENGINE_PRESET.requirements, ['strict_canon']);
	assert.equal(STRETTO_ENGINE_PRESET.play.transportRequired, true);
	assert.equal(STRETTO_ENGINE_PRESET.config.companion.canon.form, 'strict_canon');
	assert.deepEqual(delays, [2, 3.25, 4]);
	assert.deepEqual(gaps, [2, 1.25, 0.75]);
	assert.ok(
		voices.every(
			(voice) =>
				voice.timeRatio === 1 &&
				voice.harmonyMode === 'PassThrough' &&
				voice.voiceCount === 1
		)
	);
	assert.deepEqual(validateArrangementPreset(STRETTO_ENGINE_PRESET), []);

	const chordStack = {
		...STRETTO_ENGINE_PRESET,
		config: {
			...STRETTO_ENGINE_PRESET.config,
			companion: {
				...STRETTO_ENGINE_PRESET.config.companion,
				canon: {
					...STRETTO_ENGINE_PRESET.config.companion.canon,
					voices: voices.map((voice, index) =>
						index === 0 ? { ...voice, voiceCount: 2 } : voice
					)
				}
			}
		}
	};
	assert.ok(
		validateArrangementPreset(chordStack).includes(
			'Strict Canon followers must be single PassThrough lines at timeRatio 1'
		)
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

test('Suspension Garland uses one bounded transport-scheduled Species IV line', () => {
	assert.equal(SUSPENSION_GARLAND_PRESET.play.transportRequired, true);
	assert.deepEqual(SUSPENSION_GARLAND_PRESET.requirements, ['species_counterpoint']);
	assert.equal(SUSPENSION_GARLAND_PRESET.config.harmony.mode, 'PassThrough');
	assert.equal(SUSPENSION_GARLAND_PRESET.config.harmony.voiceCount, 1);
	assert.equal(SUSPENSION_GARLAND_PRESET.config.companion.enabled, true);
	assert.equal(SUSPENSION_GARLAND_PRESET.config.companion.canon.enabled, false);
	assert.deepEqual(SUSPENSION_GARLAND_PRESET.config.companion.counterpoint, {
		enabled: true,
		species: 'Species4',
		transposeDegrees: 2,
		preferAbove: true,
		holdMode: { kind: 'near_future', tail_beats: 2 }
	});
	assert.match(SUSPENSION_GARLAND_PRESET.approximation, /cannot predict your next note/);
	assert.deepEqual(validateArrangementPreset(SUSPENSION_GARLAND_PRESET), []);
	assert.deepEqual(arrangementConfigCapabilities(SUSPENSION_GARLAND_PRESET.config), [
		'harmony',
		'species_counterpoint'
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
		assert.equal(preserved in SUSPENSION_GARLAND_PRESET.config, false);
	}
});

test('Planed Cathedral fixes one exact whole-tone three-note plane', () => {
	assert.deepEqual(PLANED_CATHEDRAL_PRESET.requirements, ['harmony']);
	assert.equal(PLANED_CATHEDRAL_PRESET.play.transportRequired, false);
	assert.deepEqual(PLANED_CATHEDRAL_PRESET.config.harmony, {
		scaleMode: 'WholeTone',
		mode: 'DiatonicThirds',
		voiceCount: 3,
		voicePosition: 2,
		voiceLeadingEnabled: false,
		voiceLeadingStyle: 'Free',
		octaveMode: 'None',
		octaveIntensity: 1,
		interchangeEnabled: false,
		interchangeRange: 3,
		counterpointSpecies: 'Species1',
		counterpointStrictness: 'Strict'
	});
	assert.equal(PLANED_CATHEDRAL_PRESET.config.companion.enabled, false);
	assert.match(PLANED_CATHEDRAL_PRESET.approximation, /does not reconstruct/);
	assert.deepEqual(validateArrangementPreset(PLANED_CATHEDRAL_PRESET), []);
	assert.deepEqual(arrangementConfigCapabilities(PLANED_CATHEDRAL_PRESET.config), ['harmony']);

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
		assert.equal(preserved in PLANED_CATHEDRAL_PRESET.config, false);
	}
});

test('Color-Mode Windows fixes one exact Mode-2 diminished-seventh plane', () => {
	assert.deepEqual(COLOR_MODE_WINDOWS_PRESET.requirements, ['harmony']);
	assert.equal(COLOR_MODE_WINDOWS_PRESET.play.transportRequired, false);
	assert.deepEqual(COLOR_MODE_WINDOWS_PRESET.config.harmony, {
		scaleMode: 'DiminishedHalfWhole',
		mode: 'DiatonicThirds',
		voiceCount: 4,
		voicePosition: 3,
		voiceLeadingEnabled: false,
		voiceLeadingStyle: 'Free',
		octaveMode: 'None',
		octaveIntensity: 1,
		interchangeEnabled: false,
		interchangeRange: 3,
		counterpointSpecies: 'Species1',
		counterpointStrictness: 'Strict'
	});
	assert.equal(COLOR_MODE_WINDOWS_PRESET.config.companion.enabled, false);
	assert.match(COLOR_MODE_WINDOWS_PRESET.approximation, /does not rotate/);
	assert.match(COLOR_MODE_WINDOWS_PRESET.approximation, /nine-note Mode 3/);
	assert.deepEqual(validateArrangementPreset(COLOR_MODE_WINDOWS_PRESET), []);
	assert.deepEqual(arrangementConfigCapabilities(COLOR_MODE_WINDOWS_PRESET.config), ['harmony']);

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
		assert.equal(preserved in COLOR_MODE_WINDOWS_PRESET.config, false);
	}
});

test('Sixth-Diminished Conveyor is one bounded four-voice scale-of-chords study', () => {
	assert.deepEqual(SIXTH_DIMINISHED_CONVEYOR_PRESET.requirements, ['harmony']);
	assert.equal(SIXTH_DIMINISHED_CONVEYOR_PRESET.play.transportRequired, false);
	assert.deepEqual(SIXTH_DIMINISHED_CONVEYOR_PRESET.config.harmony, {
		scaleMode: 'BHMajor6thDim',
		mode: 'BarryHarris',
		voiceCount: 4,
		voicePosition: 0,
		voiceLeadingEnabled: false,
		voiceLeadingStyle: 'Free',
		octaveMode: 'None',
		octaveIntensity: 1,
		interchangeEnabled: false,
		interchangeRange: 3,
		counterpointSpecies: 'Species1',
		counterpointStrictness: 'Strict'
	});
	assert.equal(SIXTH_DIMINISHED_CONVEYOR_PRESET.config.companion.enabled, false);
	assert.match(SIXTH_DIMINISHED_CONVEYOR_PRESET.approximation, /does not infer a song/);
	assert.match(SIXTH_DIMINISHED_CONVEYOR_PRESET.approximation, /borrowed-note movement/);
	assert.match(SIXTH_DIMINISHED_CONVEYOR_PRESET.approximation, /chromatic extra-note rules/);
	assert.deepEqual(validateArrangementPreset(SIXTH_DIMINISHED_CONVEYOR_PRESET), []);
	assert.deepEqual(arrangementConfigCapabilities(SIXTH_DIMINISHED_CONVEYOR_PRESET.config), [
		'harmony'
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
		assert.equal(preserved in SIXTH_DIMINISHED_CONVEYOR_PRESET.config, false);
	}
});

test('Bebop Chase is one bounded complete delayed octave answer', () => {
	assert.deepEqual(BEBOP_CHASE_PRESET.requirements, ['free_imitation']);
	assert.equal(BEBOP_CHASE_PRESET.play.transportRequired, true);
	assert.equal(BEBOP_CHASE_PRESET.config.harmony.mode, 'PassThrough');
	assert.equal(BEBOP_CHASE_PRESET.config.harmony.voiceCount, 1);
	assert.equal(BEBOP_CHASE_PRESET.config.companion.enabled, true);
	assert.deepEqual(BEBOP_CHASE_PRESET.config.companion.canon, {
		enabled: true,
		form: 'free_imitation',
		holdMode: { kind: 'near_future', tail_beats: 4 },
		voices: [
			{
				delayBeats: 4,
				transposeDegrees: 7,
				timeRatio: 1,
				harmonyMode: 'PassThrough',
				referenceVoice: null,
				voiceCount: 1,
				voicePosition: 0,
				voiceLeadingEnabled: false,
				voiceLeadingStyle: 'Free',
				octaveMode: 'None',
				counterpointSpecies: 'Species1',
				counterpointStrictness: 'Strict',
				holdMode: null
			}
		]
	});
	assert.match(BEBOP_CHASE_PRESET.result, /every source attack is retained/i);
	assert.match(BEBOP_CHASE_PRESET.approximation, /does not recognize or shorten phrases/i);
	assert.match(BEBOP_CHASE_PRESET.approximation, /does not.*trade fours/i);
	assert.deepEqual(validateArrangementPreset(BEBOP_CHASE_PRESET), []);
	assert.deepEqual(arrangementConfigCapabilities(BEBOP_CHASE_PRESET.config), [
		'harmony',
		'free_imitation'
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
		assert.equal(preserved in BEBOP_CHASE_PRESET.config, false);
	}
});

test('Quartal Colossus is one bounded Dorian fourth-derived block', () => {
	assert.deepEqual(QUARTAL_COLOSSUS_PRESET.requirements, ['harmony']);
	assert.equal(QUARTAL_COLOSSUS_PRESET.play.transportRequired, false);
	assert.deepEqual(QUARTAL_COLOSSUS_PRESET.config.harmony, {
		scaleMode: 'Dorian',
		mode: 'DiatonicFourths',
		voiceCount: 4,
		voicePosition: 3,
		voiceLeadingEnabled: false,
		voiceLeadingStyle: 'Free',
		octaveMode: 'None',
		octaveIntensity: 1,
		interchangeEnabled: false,
		interchangeRange: 3,
		counterpointSpecies: 'Species1',
		counterpointStrictness: 'Strict'
	});
	assert.equal(QUARTAL_COLOSSUS_PRESET.config.companion.enabled, false);
	assert.match(QUARTAL_COLOSSUS_PRESET.result, /degrees 1, 2, 5, and 6/i);
	assert.match(QUARTAL_COLOSSUS_PRESET.approximation, /contain one augmented fourth/i);
	assert.match(QUARTAL_COLOSSUS_PRESET.approximation, /does not generate/i);
	assert.deepEqual(validateArrangementPreset(QUARTAL_COLOSSUS_PRESET), []);
	assert.deepEqual(arrangementConfigCapabilities(QUARTAL_COLOSSUS_PRESET.config), ['harmony']);

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
		assert.equal(preserved in QUARTAL_COLOSSUS_PRESET.config, false);
	}
});

test('Hollow Choir is one bounded Aeolian SATB-style harmonic shadow', () => {
	assert.deepEqual(HOLLOW_CHOIR_PRESET.requirements, ['harmony']);
	assert.equal(HOLLOW_CHOIR_PRESET.play.transportRequired, false);
	assert.deepEqual(HOLLOW_CHOIR_PRESET.config.harmony, {
		scaleMode: 'Aeolian',
		mode: 'BachChorale',
		voiceCount: 4,
		voicePosition: 0,
		voiceLeadingEnabled: false,
		voiceLeadingStyle: 'Free',
		octaveMode: 'None',
		octaveIntensity: 1,
		interchangeEnabled: false,
		interchangeRange: 3,
		counterpointSpecies: 'Species1',
		counterpointStrictness: 'Strict'
	});
	assert.equal(HOLLOW_CHOIR_PRESET.config.companion.enabled, false);
	assert.match(HOLLOW_CHOIR_PRESET.result, /SATB-style minor harmonic shadow/i);
	assert.match(HOLLOW_CHOIR_PRESET.approximation, /does not generate a literal choir/i);
	assert.match(HOLLOW_CHOIR_PRESET.approximation, /independent counterline/i);
	assert.match(HOLLOW_CHOIR_PRESET.approximation, /adaptive game scenes/i);
	assert.deepEqual(validateArrangementPreset(HOLLOW_CHOIR_PRESET), []);
	assert.deepEqual(arrangementConfigCapabilities(HOLLOW_CHOIR_PRESET.config), ['harmony']);

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
		assert.equal(preserved in HOLLOW_CHOIR_PRESET.config, false);
	}
});

test('Crystal Chorale combines a four-part harmonic-minor shadow with one delayed octave', () => {
	assert.deepEqual(CRYSTAL_CHORALE_PRESET.requirements, ['harmony', 'free_imitation']);
	assert.equal(CRYSTAL_CHORALE_PRESET.play.transportRequired, true);
	assert.deepEqual(CRYSTAL_CHORALE_PRESET.config.harmony, {
		scaleMode: 'HarmonicMinor',
		mode: 'BachChorale',
		voiceCount: 4,
		voicePosition: 0,
		voiceLeadingEnabled: false,
		voiceLeadingStyle: 'Free',
		octaveMode: 'None',
		octaveIntensity: 1,
		interchangeEnabled: false,
		interchangeRange: 3,
		counterpointSpecies: 'Species1',
		counterpointStrictness: 'Strict'
	});
	assert.deepEqual(CRYSTAL_CHORALE_PRESET.config.companion.canon, {
		enabled: true,
		form: 'free_imitation',
		holdMode: { kind: 'forever' },
		voices: [
			{
				delayBeats: 2,
				transposeDegrees: 7,
				timeRatio: 1,
				harmonyMode: 'PassThrough',
				referenceVoice: null,
				voiceCount: 1,
				voicePosition: 0,
				voiceLeadingEnabled: false,
				voiceLeadingStyle: 'Free',
				octaveMode: 'None',
				counterpointSpecies: 'Species1',
				counterpointStrictness: 'Strict',
				holdMode: null
			}
		]
	});
	assert.match(CRYSTAL_CHORALE_PRESET.result, /one complete octave echo two beats later/i);
	assert.match(CRYSTAL_CHORALE_PRESET.approximation, /per-note octave echo/i);
	assert.match(CRYSTAL_CHORALE_PRESET.approximation, /does not compose or recognize themes/i);
	assert.deepEqual(validateArrangementPreset(CRYSTAL_CHORALE_PRESET), []);
	assert.deepEqual(arrangementConfigCapabilities(CRYSTAL_CHORALE_PRESET.config), [
		'harmony',
		'free_imitation'
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
		assert.equal(preserved in CRYSTAL_CHORALE_PRESET.config, false);
	}
});

test('Pixel Trio remains visible and honestly capability-locked', () => {
	assert.equal(PIXEL_TRIO_PRESET.researchStatus, 'blocked');
	assert.equal(PIXEL_TRIO_PRESET.play.transportRequired, true);
	assert.deepEqual(PIXEL_TRIO_PRESET.requirements, ['pattern_lane', 'stable_lane_groups']);
	assert.deepEqual(
		missingArrangementCapabilities(
			PIXEL_TRIO_PRESET,
			new Set(['harmony', 'voice_leading', 'strict_canon', 'free_imitation'])
		),
		['pattern_lane', 'stable_lane_groups']
	);
	assert.match(PIXEL_TRIO_PRESET.result, /independent low support/i);
	assert.match(PIXEL_TRIO_PRESET.approximation, /share every source onset and release/i);
	assert.match(PIXEL_TRIO_PRESET.approximation, /cannot supply independent/i);
	assert.match(PIXEL_TRIO_PRESET.approximation, /will not emulate NES hardware/i);
	assert.deepEqual(validateArrangementPreset(PIXEL_TRIO_PRESET), []);

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
		assert.equal(preserved in PIXEL_TRIO_PRESET.config, false);
	}
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
