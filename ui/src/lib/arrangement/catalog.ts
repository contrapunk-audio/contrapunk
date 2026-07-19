import type {
	ArrangementCapability,
	ArrangementConfig,
	ArrangementFamily,
	ArrangementPresetV2,
	PresetPlayGuide
} from './presets';

type DraftSpec = {
	number: number;
	name: string;
	family: ArrangementFamily;
	result: string;
	prompt: string;
	references: string[];
	requirements: ArrangementCapability[];
	input?: PresetPlayGuide['input'];
	transportRequired?: boolean;
};

const SAFE_DRAFT_CONFIG: ArrangementConfig = {
	harmony: {
		scaleMode: 'Ionian',
		mode: 'PassThrough',
		voiceCount: 1,
		voicePosition: 0,
		voiceLeadingEnabled: false,
		voiceLeadingStyle: 'Free',
		octaveMode: 'None',
		octaveIntensity: 1,
		interchangeEnabled: false,
		interchangeRange: 3,
		counterpointSpecies: 'Species1',
		counterpointStrictness: 'Strict'
	},
	companion: {
		enabled: false,
		globalHoldMode: { kind: 'cancel' },
		canon: { enabled: false, form: 'free_imitation', holdMode: null, voices: [] },
		counterpoint: {
			enabled: false,
			species: 'Species1',
			transposeDegrees: 2,
			preferAbove: true,
			holdMode: null
		}
	},
	mix: { input: 1, harmony: 1, canon: 1, counterpoint: 1 }
};

export const MODAL_LINEWORK_PRESET: ArrangementPresetV2 = {
	schemaVersion: 2,
	id: '02-modal-linework',
	name: 'Modal Linework',
	family: 'classical',
	tags: ['late-renaissance', 'modal', 'four-voice', 'counterpoint'],
	builtIn: true,
	result:
		'Four close modal lines follow each note with conservative counterpoint and smooth voice leading.',
	approximation:
		"A bounded note-against-note arrangement inspired by mature four-voice Renaissance vocal counterpoint. It does not reproduce Palestrina's text setting, historical tuning, contextual ficta, independent vocal rhythms, imitative form, or cadence planning; your contour, pacing, and rests supply the phrase shape.",
	play: {
		prompt:
			'Play like one calm singer in a four-part choir: shape a mostly stepwise modal arc, breathe after each short phrase, and leave space to hear the other lines settle.',
		input: 'single_notes',
		articulation:
			'Connected one- and two-beat notes with clean releases; use one longer destination note.',
		density: 'Four to seven attacks per phrase; avoid chords and rapid runs.',
		space: 'Rest one to two beats between phrases and one full bar before the final phrase.',
		tempo: '60–84 BPM; 72 BPM is the test default.',
		transportRequired: false
	},
	references: [
		{
			name: 'Giovanni Pierluigi da Palestrina',
			context:
				'Bounded reference to mature, late-published four-voice sacred polyphony (1584–90).'
		}
	],
	researchStatus: 'approved',
	requirements: ['harmony', 'voice_leading'],
	config: {
		harmony: {
			scaleMode: 'Dorian',
			mode: 'StrictCounterpoint',
			voiceCount: 4,
			voicePosition: 1,
			voiceLeadingEnabled: true,
			voiceLeadingStyle: 'Palestrina',
			octaveMode: 'None',
			octaveIntensity: 1,
			interchangeEnabled: false,
			interchangeRange: 3,
			counterpointSpecies: 'Species1',
			counterpointStrictness: 'Strict'
		},
		companion: {
			enabled: false,
			globalHoldMode: { kind: 'cancel' },
			canon: { enabled: false, form: 'free_imitation', holdMode: null, voices: [] },
			counterpoint: {
				enabled: false,
				species: 'Species1',
				transposeDegrees: 2,
				preferAbove: true,
				holdMode: null
			}
		},
		mix: { input: 1, harmony: 1, canon: 1, counterpoint: 1 }
	}
};

export const MENSURATION_WEB_PRESET: ArrangementPresetV2 = {
	schemaVersion: 2,
	id: '04-mensuration-web',
	name: 'Mensuration Web',
	family: 'classical',
	tags: ['renaissance', 'proportional-canon', 'free-imitation', 'transport'],
	builtIn: true,
	result:
		'One short motif unfolds as four single-note lines at locked 1:1, 3:2, and 2:1 timing relationships.',
	approximation:
		"A scalar proportional Free Imitation effect inspired by the coordinated canonic procedure of Ockeghem's Missa prolationum. It does not reconstruct the Mass's two written lines, mensural notation, binary/ternary note-value rules, ficta, canonic interval plan, cadences, tactus, or historical performance.",
	play: {
		prompt:
			'Play one exact three-to-six-note line, clean and even, then leave a wide silence and listen as the same shape opens at locked proportional rates.',
		input: 'motif',
		articulation:
			'Clean non-legato or soft tenuto with deliberate releases; no pedal, slides, trills, or overlapping strings.',
		density: 'One note per beat at first; three to six source attacks total.',
		space: 'Stay silent until the 2:1 tail ends, then wait at least two more transport beats.',
		tempo: '66–84 BPM; 72 BPM is the test default.',
		transportRequired: true
	},
	references: [
		{
			name: 'Johannes Ockeghem',
			context:
				'Bounded procedural reference to the paired mensuration canons of Missa prolationum.'
		}
	],
	researchStatus: 'approved',
	requirements: ['free_imitation'],
	config: {
		harmony: {
			scaleMode: 'Dorian',
			mode: 'PassThrough',
			voiceCount: 1,
			voicePosition: 0,
			voiceLeadingEnabled: false,
			voiceLeadingStyle: 'Free',
			octaveMode: 'None',
			octaveIntensity: 1,
			interchangeEnabled: false,
			interchangeRange: 3,
			counterpointSpecies: 'Species1',
			counterpointStrictness: 'Strict'
		},
		companion: {
			enabled: true,
			globalHoldMode: { kind: 'cancel' },
			canon: {
				enabled: true,
				form: 'free_imitation',
				holdMode: { kind: 'forever' },
				voices: [
					singleLineFollower(7, 1),
					singleLineFollower(4, 1.5),
					singleLineFollower(-4, 2)
				]
			},
			counterpoint: {
				enabled: false,
				species: 'Species1',
				transposeDegrees: 2,
				preferAbove: true,
				holdMode: null
			}
		},
		mix: { input: 1, harmony: 1, canon: 1, counterpoint: 1 }
	}
};

const DRAFT_SPECS: DraftSpec[] = [
	{ number: 1, name: 'Cloister Organum', family: 'classical', result: 'Open fourths, fifths, and octaves surround a chant line.', prompt: 'Play slow held stepwise notes with clear phrase rests.', references: ['Léonin', 'Pérotin'], requirements: ['interval_stacks'] },
	{ number: 2, name: 'Modal Linework', family: 'classical', result: MODAL_LINEWORK_PRESET.result, prompt: MODAL_LINEWORK_PRESET.play.prompt, references: ['Giovanni Pierluigi da Palestrina'], requirements: ['harmony', 'voice_leading'] },
	{ number: 3, name: 'Venetian Galleries', family: 'classical', result: 'Separated choirs answer and combine.', prompt: 'Play short declarations followed by one or two bars of silence.', references: ['Andrea Gabrieli', 'Giovanni Gabrieli'], requirements: ['stable_lane_groups'] },
	{ number: 4, name: 'Mensuration Web', family: 'classical', result: 'One motif moves at simultaneous proportional speeds.', prompt: 'Play an exact three-to-six-note motif, then leave space.', references: ['Johannes Ockeghem'], requirements: ['free_imitation'], input: 'motif', transportRequired: true },
	{ number: 5, name: 'Crabwise Reflection', family: 'classical', result: 'A completed phrase returns backward.', prompt: 'Play a clean four-to-eight-note phrase, then stop for the reverse answer.', references: ["J. S. Bach's Crab Canon"], requirements: ['phrase_capture', 'phrase_reverse'], input: 'motif', transportRequired: true },
	{ number: 6, name: 'Mirror Canon', family: 'classical', result: 'Intervals invert equally around a tonal axis.', prompt: 'Play recognizable angular intervals without chords.', references: ['J. S. Bach', 'Renaissance inversion canons'], requirements: ['axis_inversion'], input: 'motif' },
	{ number: 7, name: 'Stretto Engine', family: 'classical', result: 'Tonic, fifth, and octave answers enter progressively closer.', prompt: 'Repeat a strong two-to-four-note subject with consistent rhythm.', references: ['J. S. Bach fugues'], requirements: ['strict_canon'], input: 'motif', transportRequired: true },
	{ number: 8, name: 'Suspension Garland', family: 'classical', result: 'Prepared suspensions resolve across strong beats.', prompt: 'Play long legato notes across beats and barlines.', references: ['Johann Joseph Fux', 'Giovanni Pierluigi da Palestrina'], requirements: ['species_counterpoint'], transportRequired: true },
	{ number: 9, name: 'Ground-Bass Theatre', family: 'classical', result: 'A repeating bass supports evolving counterpoint.', prompt: 'Record a simple bass figure, then play a freer upper melody.', references: ['Claudio Monteverdi', 'Henry Purcell'], requirements: ['pattern_lane'], input: 'motif', transportRequired: true },
	{ number: 10, name: 'Alberti Clockwork', family: 'classical', result: 'Low-high-middle-high accompaniment follows the lead.', prompt: 'Play sustained melody or simple harmony changes on downbeats.', references: ['W. A. Mozart', 'Muzio Clementi'], requirements: ['pattern_lane'], transportRequired: true },
	{ number: 11, name: 'Motive Forge', family: 'classical', result: 'A small motif is fragmented, displaced, and expanded.', prompt: 'Repeat one emphatic three-to-five-note idea and leave development space.', references: ['Ludwig van Beethoven'], requirements: ['motif_transform'], input: 'motif' },
	{ number: 12, name: 'Planed Cathedral', family: 'classical', result: 'Whole-tone or pentatonic chord planes move in parallel.', prompt: 'Play a slow spacious line with sustained notes and few tonal cadences.', references: ['Claude Debussy'], requirements: ['harmony'] },
	{ number: 13, name: 'Axis Mirror', family: 'classical', result: 'Symmetrical reflections orbit a recurring center.', prompt: 'Alternate steps and bold leaps around one anchor note.', references: ['Béla Bartók'], requirements: ['axis_inversion'] },
	{ number: 14, name: 'Color-Mode Windows', family: 'classical', result: 'Whole-tone, octatonic, and augmented colors rotate.', prompt: 'Play sparse, strongly articulated modal notes.', references: ['Olivier Messiaen'], requirements: ['harmony'] },
	{ number: 15, name: 'Micropolyphonic Cloud', family: 'classical', result: 'Micro-delayed lines blur into a shifting mass.', prompt: 'Play very few long tones with gentle bends and gradual dynamics.', references: ['György Ligeti'], requirements: ['microtiming', 'per_voice_detune', 'probability_density'], transportRequired: true },
	{ number: 16, name: 'Tintinnabuli Halo', family: 'classical', result: 'A melody is shadowed by nearest tonic-triad tones.', prompt: 'Play a slow plain stepwise line with meaningful silence.', references: ['Arvo Pärt'], requirements: ['tintinnabuli'] },
	{ number: 17, name: 'Phase Lattice', family: 'classical', result: 'Repeated patterns drift out of phase and realign.', prompt: 'Repeat an exact three-to-eight-note cell for several bars.', references: ['Steve Reich'], requirements: ['phrase_capture', 'phase'], input: 'motif', transportRequired: true },
	{ number: 18, name: 'Broken Consort', family: 'classical', result: 'Fragments pass, align, and break apart.', prompt: 'Play irregular two-to-five-note gestures separated by breaths.', references: ['Caroline Shaw'], requirements: ['phrase_capture', 'motif_transform'], input: 'motif' },
	{ number: 19, name: 'Kinetic Blocks', family: 'classical', result: 'Accented rhythmic blocks collide with bright repetitions.', prompt: 'Play staccato repeats, octave jumps, and exact accents.', references: ['Anna Meredith'], requirements: ['pattern_lane', 'polymeter'], input: 'motif', transportRequired: true },
	{ number: 20, name: 'Spectral Bloom', family: 'classical', result: 'Harmony expands from overtone relationships.', prompt: 'Play long expressive notes with velocity, bend, vibrato, and register changes.', references: ['Gérard Grisey', 'Tristan Murail', 'Kaija Saariaho'], requirements: ['spectral_voicing', 'per_voice_detune'] },
	{ number: 21, name: 'Sectional Velvet', family: 'jazz', result: 'Low, middle, and high groups act like contrasting sections.', prompt: 'Play lyrical medium-tempo lines with dynamics and response space.', references: ['Duke Ellington'], requirements: ['stable_lane_groups'] },
	{ number: 22, name: 'Stride Engine', family: 'jazz', result: 'Bass and chord punches alternate.', prompt: 'Play a clear swinging melody or chord tones; leave the bass open.', references: ['James P. Johnson', 'Fats Waller'], requirements: ['pattern_lane'], input: 'chords', transportRequired: true },
	{ number: 23, name: 'Sixth-Diminished Conveyor', family: 'jazz', result: 'Sixth chords alternate with diminished passing harmony.', prompt: 'Play connected bebop eighths, chromatic approaches, and clear resolutions.', references: ['Barry Harris'], requirements: ['harmony'] },
	{ number: 24, name: 'Angular Cells', family: 'jazz', result: 'Tritones, clusters, displaced responses, and negative space answer the lead.', prompt: 'Play short dry motifs with surprising accents and long rests.', references: ['Thelonious Monk'], requirements: ['bounded_clusters', 'microtiming'], input: 'motif' },
	{ number: 25, name: 'Bebop Chase', family: 'jazz', result: 'Fast phrases are pursued by shortened delayed answers.', prompt: 'Play clean four-to-eight-note bursts with gaps.', references: ['Charlie Parker', 'Dizzy Gillespie'], requirements: ['free_imitation'], input: 'motif', transportRequired: true },
	{ number: 26, name: 'Rootless Glass', family: 'jazz', result: 'Smooth rootless ninth, eleventh, and thirteenth voicings follow the lead.', prompt: 'Play ballad lines, sustained upper tones, and gentle chromatic approaches.', references: ['Bill Evans'], requirements: ['jazz_extensions'] },
	{ number: 27, name: 'Quartal Colossus', family: 'jazz', result: 'Forceful fourth stacks occupy Dorian or pentatonic space.', prompt: 'Play assertive modal riffs with rhythmic repetition.', references: ['McCoy Tyner'], requirements: ['harmony'], input: 'motif' },
	{ number: 28, name: 'Third-Cycle Labyrinth', family: 'jazz', result: 'Tonal centers rotate through major thirds.', prompt: 'Repeat a compact motif while harmony changes beneath it.', references: ['John Coltrane'], requirements: ['harmonic_timeline'], input: 'motif', transportRequired: true },
	{ number: 29, name: 'Shout Counterpoint', family: 'jazz', result: 'Bass ostinato, gospel response, and unruly counterline combine.', prompt: 'Play forceful two-bar blues calls followed by a full response opening.', references: ['Charles Mingus'], requirements: ['pattern_lane', 'stable_lane_groups'], input: 'motif', transportRequired: true },
	{ number: 30, name: 'Free Quartet', family: 'jazz', result: 'Independent contours occupy separate tonal centers.', prompt: 'Play exploratory lines with wide leaps and pauses to answer the ensemble.', references: ['Ornette Coleman'], requirements: ['independent_tonal_centers', 'stable_lane_groups'] },
	{ number: 31, name: 'Side-Slip Matrix', family: 'jazz', result: 'Quartal harmony slips a semitone outside and returns.', prompt: 'Play a simple inside-key modal motif and leave room for outside motion.', references: ['Herbie Hancock'], requirements: ['harmonic_timeline'], input: 'motif', transportRequired: true },
	{ number: 32, name: 'Harmonic Maze', family: 'jazz', result: 'Harmony moves ambiguously among diatonic and symmetric regions.', prompt: 'Play sustained unresolved cells that avoid obvious cadences.', references: ['Wayne Shorter'], requirements: ['harmonic_timeline'], input: 'motif' },
	{ number: 33, name: 'Metric Labyrinth', family: 'jazz', result: 'Percussive counterpoint crosses uneven groupings.', prompt: 'Repeat an exact accented figure in 5, 7, or grouped 4/4.', references: ['Tigran Hamasyan', 'Vijay Iyer'], requirements: ['odd_meter', 'pattern_lane'], input: 'motif', transportRequired: true },
	{ number: 34, name: 'Chamber Sky', family: 'jazz', result: 'Lyrical voices expand into airy ensemble harmony.', prompt: 'Play breath-shaped arcs with wide expressive intervals and long releases.', references: ['Maria Schneider', 'Kenny Wheeler'], requirements: ['stable_lane_groups'] },
	{ number: 35, name: 'Elastic Counter-Groove', family: 'jazz', result: 'Syncopated harmony alternates with contrapuntal answers.', prompt: 'Play concise one- or two-bar riffs with deliberate holes.', references: ['Esperanza Spalding', 'Brad Mehldau', 'Robert Glasper'], requirements: ['pattern_lane'], input: 'motif', transportRequired: true },
	{ number: 36, name: 'Pixel Trio', family: 'game', result: 'Melody, bass, and one counterline obey three-channel economy.', prompt: 'Play compact memorable single-note hooks with a clear repeated rhythm.', references: ['Early console writing', 'Koji Kondo', 'Manami Matsumae'], requirements: ['harmony', 'voice_leading'], input: 'motif' },
	{ number: 37, name: 'Fractured Crystal', family: 'game', result: 'A tiny motif fractures into limited-voice ostinatos and modal echoes.', prompt: 'Repeat a three-to-five-note motif slowly, alter one note, and leave substantial air.', references: ['Disasterpeace — FEZ'], requirements: ['motif_memory', 'pattern_lane', 'probability_density'], input: 'motif', transportRequired: true },
	{ number: 38, name: 'Summit Pulse', family: 'game', result: 'An intimate line grows into rhythmic upper layers.', prompt: 'Begin softly with hesitant fragments, then increase register, velocity, and repetition.', references: ['Lena Raine'], requirements: ['adaptive_scenes'], input: 'motif' },
	{ number: 39, name: 'Adaptive Pilgrim', family: 'game', result: 'The arrangement follows an emotional intensity arc.', prompt: 'Play isolated sustained notes, then longer and louder phrases, then withdraw.', references: ['Austin Wintory', 'Jessica Curry', 'Gareth Coker'], requirements: ['adaptive_scenes'] },
	{ number: 40, name: 'Ash and Ember', family: 'game', result: 'Rough modal riffs receive acoustic, metallic, and vocal-like answers.', prompt: 'Play gritty short riffs, strong downbeats, slides, and dramatic pauses.', references: ['Darren Korb', 'Ashley Barrett'], requirements: ['pattern_lane'], input: 'motif', transportRequired: true },
	{ number: 41, name: 'Blocks at Dusk', family: 'game', result: 'Sparse figures accumulate soft extended harmony.', prompt: 'Repeat two or three gentle notes with small variations and long decay space.', references: ['C418'], requirements: ['pattern_lane', 'probability_density'], input: 'motif', transportRequired: true },
	{ number: 42, name: 'Clockwork Stars', family: 'game', result: 'Recurring motifs return across registers and counter-rhythms.', prompt: 'Repeat a clean four-to-eight-note motif while preserving its rhythm.', references: ['Ben Prunty'], requirements: ['motif_memory', 'motif_transform'], input: 'motif' },
	{ number: 43, name: 'Hollow Choir', family: 'game', result: 'A dark minor melody expands into distant chorale and counterline.', prompt: 'Play singable minor phrases with held destinations and long pauses.', references: ['Christopher Larkin'], requirements: ['harmony', 'voice_leading'] },
	{ number: 44, name: 'Memory Weave', family: 'game', result: 'Two learned themes return transformed and combined.', prompt: 'Teach two distinct motifs, repeating each twice with a pause.', references: ['Toby Fox'], requirements: ['motif_memory', 'motif_transform'], input: 'motif' },
	{ number: 45, name: 'Neon Noir', family: 'game', result: 'Minor jazz-funk harmony gains chromatic passing motion and rhythmic responses.', prompt: 'Play tight syncopated riffs, repeated notes, and short chromatic runs.', references: ['Shoji Meguro'], requirements: ['pattern_lane', 'jazz_extensions'], input: 'motif', transportRequired: true },
	{ number: 46, name: 'Forest Layer Unfold', family: 'game', result: 'Quiet modal material gains naturalistic layers as density rises.', prompt: 'Begin sparse and pentatonic, then add repeats and higher notes gradually.', references: ['David Wise'], requirements: ['adaptive_scenes'] },
	{ number: 47, name: 'Time-Cross Rhythm', family: 'game', result: 'Compound-meter melody meets duple counter-rhythms.', prompt: 'Repeat a six- or twelve-beat cell with strong internal accents.', references: ['Yasunori Mitsuda'], requirements: ['polymeter', 'pattern_lane'], input: 'motif', transportRequired: true },
	{ number: 48, name: 'Crystal Chorale', family: 'game', result: 'A memorable melody grows into harmonic-minor chorale and octave answers.', prompt: 'Play clear four- or eight-bar phrases with held cadence notes.', references: ['Nobuo Uematsu'], requirements: ['harmony', 'voice_leading'] },
	{ number: 49, name: 'Kingdom Counterline', family: 'game', result: 'A lyrical lead gains urgent inner lines and dramatic harmonic turns.', prompt: 'Play a memorable theme with expressive leaps, dynamics, and phrase-end rests.', references: ['Yoko Shimomura'], requirements: ['motif_memory', 'harmonic_timeline'], input: 'motif' },
	{ number: 50, name: 'Rust and Static', family: 'game', result: 'Fragile fragments meet unstable clusters, shadows, and empty space.', prompt: 'Play isolated notes, slides, tritones, muted attacks, and unpredictable pauses.', references: ['Akira Yamaoka'], requirements: ['bounded_clusters', 'probability_density'] }
];

export const BUILT_IN_ARRANGEMENT_PRESETS: readonly ArrangementPresetV2[] = DRAFT_SPECS.map(
	(spec) => {
		if (spec.number === 2) return MODAL_LINEWORK_PRESET;
		if (spec.number === 4) return MENSURATION_WEB_PRESET;
		return draftPreset(spec);
	}
);

function singleLineFollower(transposeDegrees: number, timeRatio: number) {
	return {
		delayBeats: 0,
		transposeDegrees,
		timeRatio,
		harmonyMode: 'PassThrough' as const,
		referenceVoice: null,
		voiceCount: 1,
		voicePosition: 0,
		voiceLeadingEnabled: false,
		voiceLeadingStyle: 'Free' as const,
		octaveMode: 'None' as const,
		counterpointSpecies: 'Species1' as const,
		counterpointStrictness: 'Strict' as const,
		holdMode: null
	};
}

function draftPreset(spec: DraftSpec): ArrangementPresetV2 {
	return {
		schemaVersion: 2,
		id: `${String(spec.number).padStart(2, '0')}-${slug(spec.name)}`,
		name: spec.name,
		family: spec.family,
		tags: [spec.family, 'research-pending'],
		builtIn: true,
		result: spec.result,
		play: {
			prompt: spec.prompt,
			input: spec.input ?? 'single_notes',
			articulation: 'Research pending.',
			density: 'Research pending.',
			space: 'Research pending.',
			transportRequired: spec.transportRequired ?? false
		},
		references: spec.references.map((name) => ({
			name,
			context: 'Catalog reference only; independent research is pending.'
		})),
		researchStatus: 'pending',
		requirements: spec.requirements,
		config: SAFE_DRAFT_CONFIG
	};
}

function slug(name: string): string {
	return name.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/(^-|-$)/g, '');
}
