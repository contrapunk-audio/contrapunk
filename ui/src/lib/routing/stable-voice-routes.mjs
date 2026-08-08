/**
 * Return every route the current arrangement can address, including inactive ones.
 *
 * @param {{
 *   voiceCount: number,
 *   voicePosition: number,
 *   harmonyEnabled: boolean,
 *   companionEnabled: boolean,
 *   canonVoiceCount: number,
 *   canonEnabled: boolean,
 *   counterpointEnabled: boolean,
 *   phraseAware: boolean,
 *   patternLowEnabled: boolean,
 *   patternCounterEnabled: boolean
 * }} config
 */
export function stableVoiceRoutes(config) {
	const companionActive = config.companionEnabled;
	return [
		{ section: 'input', route: 'input', active: true },
		...Array.from({ length: config.voiceCount }, (_, slot) => ({
			section: 'harmony',
			route: `harmony:${slot}`,
			active: config.harmonyEnabled && slot !== config.voicePosition
		})),
		...Array.from({ length: config.canonVoiceCount }, (_, voice) => ({
			section: 'canon',
			route: `canon:${voice}`,
			active: companionActive && config.canonEnabled
		})),
		{
			section: 'counterpoint',
			route: 'counterpoint:0',
			active: companionActive && config.counterpointEnabled
		},
		{
			section: 'counterpoint',
			route: 'counterpoint:1',
			active: companionActive && config.counterpointEnabled && config.phraseAware
		},
		{
			section: 'patterns',
			route: 'pattern_low',
			active: companionActive && config.patternLowEnabled
		},
		{
			section: 'patterns',
			route: 'pattern_counter',
			active: companionActive && config.patternCounterEnabled
		}
	];
}
