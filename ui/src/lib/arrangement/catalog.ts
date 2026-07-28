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

export const CLOISTER_ORGANUM_PRESET: ArrangementPresetV2 = {
	schemaVersion: 2,
	id: '01-cloister-organum',
	name: 'Cloister Organum',
	family: 'classical',
	tags: ['notre-dame', 'open-intervals', 'two-voice', 'modal'],
	builtIn: true,
	result:
		'One upper voice answers a slow monophonic line with a degree-mapped perfect fourth, fifth, or octave.',
	approximation:
		'A bounded modern open-interval arrangement inspired by selected Notre-Dame organum repertory associated with Léonin and Pérotin. Your line supplies the chant-like contour and phrase shape. It does not generate chant or reproduce organum purum’s florid duplum, discant rhythm, Pérotin’s independent upper voices, medieval tuning, liturgy, notation, or historical performance.',
	play: {
		prompt:
			'Play one slow, chant-like note at a time. Hold arrivals, move mostly by step, return to the tonic, and leave a complete breath between phrases.',
		input: 'single_notes',
		articulation:
			'Rounded clean attacks; hold foundation and arrival notes for 2–6 seconds and make connecting notes shorter.',
		density: 'Two to five monophonic attacks per gesture; no chords, tremolo, or rapid runs.',
		space: 'Release completely between phrases and confirm every generated note has stopped.',
		tempo: '48–66 BPM for pacing only; transport is optional.',
		transportRequired: false
	},
	references: [
		{
			name: 'Léonin-associated Notre-Dame organum duplum',
			context:
				'Bounded reference to the late-twelfth-century chant-supported layer described retrospectively by Anonymous IV.'
		},
		{
			name: 'Pérotin-associated Magnus liber revisions',
			context:
				'Repertorial context only; this preset does not claim the measured independent parts of the tripla or quadrupla.'
		}
	],
	researchStatus: 'approved',
	requirements: ['harmony', 'interval_stacks'],
	config: {
		harmony: {
			scaleMode: 'Dorian',
			mode: 'ExplicitIntervals',
			voiceCount: 2,
			voicePosition: 1,
			voiceLeadingEnabled: false,
			voiceLeadingStyle: 'Free',
			octaveMode: 'None',
			octaveIntensity: 1,
			interchangeEnabled: false,
			interchangeRange: 3,
			counterpointSpecies: 'Species1',
			counterpointStrictness: 'Strict',
			explicitIntervalMap: {
				degreeOffsets: [[12], [7], [7], [5], [7], [5], [5]],
				fallbackOffsets: [7]
			}
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

export const STRETTO_ENGINE_PRESET: ArrangementPresetV2 = {
	schemaVersion: 2,
	id: '07-stretto-engine',
	name: 'Stretto Engine',
	family: 'classical',
	tags: ['baroque', 'stretto', 'strict-canon', 'transport'],
	builtIn: true,
	result:
		'One short subject returns at unison-, dominant-, and octave-related levels with entry gaps contracting from 2 to 1.25 to 0.75 beats.',
	approximation:
		'Fixed Strict Canon inspired by selected Bach stretto procedures. It repeats every note of your short live cell at three preset pitch levels and contracting entry gaps; it does not recognize a subject, generate real versus tonal answers, test invertible counterpoint, compose episodes or cadences, or reconstruct a Bach fugue.',
	play: {
		prompt:
			'Play two to four clear single notes with one memorable rhythm, then stop and listen as unison-, fifth-, and octave-related answers enter progressively closer.',
		input: 'motif',
		articulation:
			'Lightly detached, even attacks; gate each note to about 50–75% of its inter-onset interval.',
		density: 'One monophonic cell only; play less as the answers crowd in.',
		space:
			'Stay silent through the full delayed group, then leave more than two additional transport beats.',
		tempo: '72–100 BPM; 90 BPM is the test default.',
		transportRequired: true
	},
	references: [
		{
			name: 'J. S. Bach',
			context:
				'Bounded reference to varied stretto procedures in WTC I BWV 846/2 and The Art of Fugue Contrapunctus 5.'
		}
	],
	researchStatus: 'approved',
	requirements: ['strict_canon'],
	config: {
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
			enabled: true,
			globalHoldMode: { kind: 'cancel' },
			canon: {
				enabled: true,
				form: 'strict_canon',
				holdMode: { kind: 'forever' },
				voices: [
					singleLineFollower(0, 1, 2),
					singleLineFollower(4, 1, 3.25),
					singleLineFollower(7, 1, 4)
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

export const SUSPENSION_GARLAND_PRESET: ArrangementPresetV2 = {
	schemaVersion: 2,
	id: '08-suspension-garland',
	name: 'Suspension Garland',
	family: 'classical',
	tags: ['renaissance', 'fourth-species', 'prepared-suspension', 'transport'],
	builtIn: true,
	result:
		'A consonant counterline enters between beats; compatible on-beat motion holds it into tension, then resolves it down one diatonic step.',
	approximation:
		"A live, opportunistic fourth-species study inspired by Fux's pedagogy and recurrent suspension practice in selected Palestrina sources. It uses a simple beat/half-beat grid independent of bar meter. Because Contrapunk cannot predict your next note, incompatible motion becomes consonant syncopation or is released before the new attack. It does not reproduce Palestrina's text setting, ficta, mensural rhythm, changing vocal density, suspension statistics, cadence planning, or historical tuning.",
	play: {
		prompt:
			'At 60–72 BPM, play one note on a transport beat, then a note a perfect fourth above on the next beat; listen for the counterline to enter between them, hold without retriggering, and fall by step.',
		input: 'single_notes',
		articulation:
			'Connected single notes with a clean legato handoff at each beat; keyboard pedal up, guitar strings actively muted, no bends or slides.',
		density: 'One attack per beat for two to four notes; no chords, repeated-note overlaps, or fast runs.',
		space: 'After each two-to-four-note chain, release and leave one full bar silent.',
		tempo: '60–72 BPM; 64 BPM is the test default.',
		transportRequired: true
	},
	references: [
		{
			name: 'Johann Joseph Fux',
			context:
				'Bounded reference to the preparation–suspension–resolution grid of fourth-species pedagogy in Gradus ad Parnassum.'
		},
		{
			name: 'Giovanni Pierluigi da Palestrina',
			context:
				'Bounded reference to recurrent prepared suspensions in selected four-voice sacred sources, not continuous fourth species.'
		}
	],
	researchStatus: 'approved',
	requirements: ['species_counterpoint'],
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
			canon: { enabled: false, form: 'free_imitation', holdMode: null, voices: [] },
			counterpoint: {
				enabled: true,
				species: 'Species4',
				transposeDegrees: 2,
				preferAbove: true,
				holdMode: { kind: 'near_future', tail_beats: 2 }
			}
		},
		mix: { input: 1, harmony: 1, canon: 1, counterpoint: 1 }
	}
};

export const PLANED_CATHEDRAL_PRESET: ArrangementPresetV2 = {
	schemaVersion: 2,
	id: '12-planed-cathedral',
	name: 'Planed Cathedral',
	family: 'classical',
	tags: ['early-modern', 'whole-tone', 'exact-planing', 'player-shaped-form'],
	builtIn: true,
	result:
		'Each in-collection note becomes a fixed three-note whole-tone augmented plane; your register, velocity, and silence shape the rise and withdrawal.',
	approximation:
		"A static whole-tone chord plane inspired by one selected coloristic process in Debussy's mature piano writing. It does not reconstruct La cathédrale engloutie, model Debussy's pentatonic, diatonic, or chromatic collection changes, preserve an independent pedal, reproduce piano resonance or orchestration, infer a phrase arc, or represent his whole career. You supply emergence and submergence through register, velocity, duration, density, and silence.",
	play: {
		prompt:
			'Hold one soft in-collection note at a time. Rise and grow until one broad, bright peak, then fall away into lower notes and longer silences; release every note cleanly before the next.',
		input: 'single_notes',
		articulation:
			'Rounded 2–5 second tones with complete releases; one 5–7 second peak, no physical overlap, pedal, bends, slides, or ringing strings.',
		density: 'Three to six planes per 15 seconds; exactly one physical source note at a time.',
		space: 'Leave 0.5–2 seconds between planes and 2–4 seconds before and after the peak.',
		tempo: '48–66 BPM if a reference pulse helps; transport is optional and remains untouched.',
		transportRequired: false
	},
	references: [
		{
			name: 'Claude Debussy',
			context:
				'Bounded reference to selected collection-bound planing, register, dynamics, and silence in mature piano works from approximately 1903–1910.'
		}
	],
	researchStatus: 'approved',
	requirements: ['harmony'],
	config: {
		harmony: {
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

export const COLOR_MODE_WINDOWS_PRESET: ArrangementPresetV2 = {
	schemaVersion: 2,
	id: '14-color-mode-windows',
	name: 'Color-Mode Windows',
	family: 'classical',
	tags: ['early-modern', 'mode-2', 'octatonic', 'player-shaped-form'],
	builtIn: true,
	result:
		'Each in-collection note opens a fixed four-note Mode-2 diminished-seventh window; your repeated cells, register, attack, and silence frame the contrasts.',
	approximation:
		"A static fixed-transposition Mode-2 window inspired by one pitch-and-voicing property of Messiaen's modal-harmonic language as codified in 1944. It does not reproduce his added or resonance chords, pedals, tonal poles, nine-note Mode 3, rhythmic procedures, birdsong, orchestration, theology, or personal chord-color perceptions, and it does not rotate collections automatically. You supply the cells, contrasts, register, dynamics, articulation, duration, and silence.",
	play: {
		prompt:
			'Play a crisp two-to-four-note cell from the shown collection, then leave a long gap. Repeat it once higher or stronger, change only one note, and hear each generated chord as a separate color window.',
		input: 'motif',
		articulation:
			'Clear 0.6–2 second notes with complete releases; one 2–3 second destination, no pedal, overlap, bends, slides, or ringing strings.',
		density: 'Two to four source notes per cell; one physical note at a time.',
		space: 'Leave 0.3–0.8 seconds between notes and 1.5–3 seconds after each cell.',
		tempo: '54–76 BPM if a reference pulse helps; transport is optional and remains untouched.',
		transportRequired: false
	},
	references: [
		{
			name: 'Olivier Messiaen',
			context:
				'Bounded reference to Mode-2 modal-harmonic color in the language codified in 1944 and selected works from approximately 1935–44.'
		}
	],
	researchStatus: 'approved',
	requirements: ['harmony'],
	config: {
		harmony: {
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

export const SIXTH_DIMINISHED_CONVEYOR_PRESET: ArrangementPresetV2 = {
	schemaVersion: 2,
	id: '23-sixth-diminished-conveyor',
	name: 'Sixth-Diminished Conveyor',
	family: 'jazz',
	tags: ['bebop-pedagogy', 'sixth-diminished', 'drop-2', 'player-shaped-form'],
	builtIn: true,
	result:
		'Each eligible note selects either the tonic major-sixth family or its related diminished seventh as one four-voice drop-2 block.',
	approximation:
		"A fixed live drop-2 study of one major sixth-diminished collection from Barry Harris's mature teaching. It does not infer a song, chord progression, bass, harmonic region, borrowed-note movement, related dominant family, chromatic extra-note rules, swing, cadence, phrase form, or Harris's touch and improvisational judgment. Notes outside the displayed collection pass through without guaranteed generated harmony; you supply the movement, target, timing, accents, dynamics, register, resolution, and silence.",
	play: {
		prompt:
			'Play one clean note at a time through the shown eight-note collection. Move through a diminished passing tone into a clear sixth-chord arrival, hold it briefly, release, and leave a full-bar rest.',
		input: 'single_notes',
		articulation:
			'Connected, lightly swung or even eighth notes with distinct attacks and clean releases; pedal up, unused strings muted, and no physical overlap.',
		density: 'Four to eight source attacks per phrase; exactly one physical source note at a time.',
		space: 'Rest one to two beats after a clause and one full bar after each phrase; wait for every generated note to release.',
		tempo: '88–160 BPM; 108 BPM is the test default. Straighten the eighths as tempo rises.',
		transportRequired: false
	},
	references: [
		{
			name: 'Barry Harris',
			context:
				'Bounded reference to the mature sixth-diminished scale-of-chords pedagogy documented from the Harris/Howard Rees workshop corpus through Harris’s final teaching years.'
		}
	],
	researchStatus: 'approved',
	requirements: ['harmony'],
	config: {
		harmony: {
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

export const BEBOP_CHASE_PRESET: ArrangementPresetV2 = {
	schemaVersion: 2,
	id: '25-bebop-chase',
	name: 'Bebop Chase',
	family: 'jazz',
	tags: ['bebop', 'call-and-response', 'free-imitation', 'transport'],
	builtIn: true,
	result:
		'A clean monophonic burst returns four beats later as a complete octave answer; every source attack is retained.',
	approximation:
		'A disclosed turn-taking exercise inspired by Parker and Gillespie small-group exchanges from 1945–46 and their four-bar exchanges on “Leap Frog” (1950). It repeats your complete live burst at one fixed delay and scale-octave displacement. It does not recognize or shorten phrases, generate swing or accents, follow chord changes, trade fours, model either artist, or reproduce their individual vocabulary; you supply the phrase, rhythmic feel, harmonic direction, development, and ending.',
	play: {
		prompt:
			'In 4/4, play four to six clean in-scale eighth notes within three beats, then stop and leave the next four-beat window open for the complete octave answer.',
		input: 'motif',
		articulation:
			'Clean monophonic attacks with distinct releases; pedal up, unused strings muted, and no overlapping physical notes.',
		density: 'Four to six source attacks within at most three transport beats.',
		space:
			'Leave at least four beats after each burst and wait for the answer to release before beginning the next call.',
		tempo: '140–200 BPM in 4/4; 160 BPM is the test default.',
		transportRequired: true
	},
	references: [
		{
			name: 'Charlie Parker',
			context:
				'Bounded reference to turn-taking within Parker and Gillespie’s shared 1945–46 small-group bebop language, not a simulation of Parker’s playing.'
		},
		{
			name: 'Dizzy Gillespie',
			context:
				'Bounded reference to documented Parker/Gillespie exchanges through “Leap Frog” (1950), not a simulation of Gillespie’s playing.'
		}
	],
	researchStatus: 'approved',
	requirements: ['free_imitation'],
	config: {
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
			enabled: true,
			globalHoldMode: { kind: 'cancel' },
			canon: {
				enabled: true,
				form: 'free_imitation',
				holdMode: { kind: 'near_future', tail_beats: 4 },
				voices: [singleLineFollower(7, 1, 4)]
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

export const QUARTAL_COLOSSUS_PRESET: ArrangementPresetV2 = {
	schemaVersion: 2,
	id: '27-quartal-colossus',
	name: 'Quartal Colossus',
	family: 'jazz',
	tags: ['post-bop', 'dorian', 'fourth-derived', 'player-shaped-form'],
	builtIn: true,
	result:
		'Each in-collection note becomes one four-voice Dorian fourth-derived block with the exact source at the bottom; degrees 1, 2, 5, and 6 form exact perfect-fourth stacks.',
	approximation:
		'A fixed harmony study inspired by one open fourth-rich feature documented in selected 1960–67 McCoy Tyner performances. It does not generate Tyner’s phrases, rhythm, touch, pedals, bass movement, dominant substitutions, chromatic resolutions, comping decisions, ensemble interaction, or formal development. Degrees flat-3, 4, and flat-7 contain one augmented fourth; you supply the cell, accents, register, dynamics, repetition, resolution, and silence.',
	play: {
		prompt:
			'Punch a two-to-four-note cell from displayed degrees 1, 2, 5, and 6, repeat its rhythm with one stronger accent or octave shift, then release and leave a full bar of air.',
		input: 'motif',
		articulation:
			'Firm, dry, non-legato single notes with complete releases; keyboard pedal up, guitar strings muted, and no bends, slides, double-stops, or ringing strings.',
		density: 'Two to four source attacks per clause; exactly one physical note and one resulting block at a time.',
		space: 'Leave at least one beat between clauses and one full bar after each phrase.',
		tempo: '88–132 BPM; transport is optional and remains untouched.',
		transportRequired: false
	},
	references: [
		{
			name: 'McCoy Tyner',
			context:
				'Bounded reference to one fourth-rich modal/open-voicing property in selected 1960–65 John Coltrane Quartet performances and Tyner’s independent consolidation through The Real McCoy (1967), not a simulation of the artist or ensemble.'
		}
	],
	researchStatus: 'approved',
	requirements: ['harmony'],
	config: {
		harmony: {
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

export const PIXEL_TRIO_PRESET: ArrangementPresetV2 = {
	schemaVersion: 2,
	id: '36-pixel-trio',
	name: 'Pixel Trio',
	family: 'game',
	tags: ['famicom-nes', 'three-pitched-roles', 'pattern-lane', 'stable-role-groups'],
	builtIn: true,
	result:
		'A live hook leads a phrase-relative bass arpeggio while a compact counterpulse answers your latest note in the spaces you leave.',
	approximation:
		'A modern deterministic MIDI trio informed by three-pitched-role economy in selected 1985–87 Famicom/NES practice. The low pattern takes its pitch center from the phrase opening and moves through a compact four-step arpeggio. The counter pattern retunes from the latest player attack and yields to held notes, so contour changes its answer without adding another voice. It does not emulate NES hardware, reproduce game music, simulate Koji Kondo or Manami Matsumae, or add noise, DMC, percussion, chip timbre, sound effects, motif recognition, or game-state logic.',
	play: {
		prompt:
			'Play a short single-note hook with clear holes. Change direction inside the hook and hear each gap answer follow your latest note; start the next phrase elsewhere to move the bass.',
		input: 'motif',
		articulation: 'Use crisp attacks and short gaps, then hold one arrival to make the counterpulse yield.',
		density: 'Three to six source attacks per phrase; two voices are normal and the full trio is brief.',
		space: 'Leave one short opening inside the hook, then release long enough for the pattern window to close.',
		tempo: '100–150 BPM in 4/4; transport is required for the two stable pattern roles.',
		transportRequired: true
	},
	references: [
		{
			name: 'Selected early Famicom/NES practice',
			context:
				'Bounded reference to pitched-role economy in Koji Kondo’s Super Mario Bros. (1985) and Manami Matsumae’s original Mega Man (1987), not either composer’s complete output or an artist/hardware simulation.'
		}
	],
	researchStatus: 'approved',
	requirements: ['pattern_lane', 'stable_lane_groups', 'role_mix'],
	config: {
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
			enabled: true,
			globalHoldMode: { kind: 'cancel' },
			canon: { enabled: false, form: 'free_imitation', holdMode: null, voices: [] },
			counterpoint: {
				enabled: false,
				species: 'Species1',
				transposeDegrees: 2,
				preferAbove: true,
				holdMode: null
			},
			patterns: {
				lowSupport: {
					enabled: true,
					cycleBeats: 4,
					tailBeats: 4,
					pitchAnchor: 'phrase_start',
					onlyWhenInputIdle: false,
					events: [
						{ beat: 0, degree: 0, octave: -2, durationBeats: 0.5, velocity: 72 },
						{ beat: 1, degree: 4, octave: -2, durationBeats: 0.375, velocity: 64 },
						{ beat: 2, degree: 2, octave: -2, durationBeats: 0.5, velocity: 68 },
						{ beat: 3, degree: 4, octave: -2, durationBeats: 0.375, velocity: 62 }
					]
				},
				counterline: {
					enabled: true,
					cycleBeats: 4,
					tailBeats: 4,
					pitchAnchor: 'latest_input',
					onlyWhenInputIdle: true,
					events: [
						{ beat: 0.5, degree: 4, octave: 0, durationBeats: 1, velocity: 60 },
						{ beat: 1.5, degree: 2, octave: 0, durationBeats: 1, velocity: 56 },
						{ beat: 2.5, degree: 5, octave: 0, durationBeats: 1, velocity: 58 },
						{ beat: 3.5, degree: 4, octave: 0, durationBeats: 1, velocity: 54 }
					]
				}
			}
		},
		mix: { input: 1, harmony: 1, canon: 0.52, counterpoint: 0.42 }
	}
};

export const HOLLOW_CHOIR_PRESET: ArrangementPresetV2 = {
	schemaVersion: 2,
	id: '43-hollow-choir',
	name: 'Hollow Choir',
	family: 'game',
	tags: ['dark-fantasy', 'aeolian', 'satb-shadow', 'player-shaped-form'],
	builtIn: true,
	result:
		'A singable Aeolian line receives a restrained four-part, SATB-style minor harmonic shadow.',
	approximation:
		'A static harmony study informed by sparse, melancholic vocal/orchestral atmosphere in a bounded 2014–18 Hollow Knight project corpus. It does not generate a literal choir, acoustic orchestration, distant layers, independent counterline, adaptive game scenes, ambience, reverb, narrative response, protected cue material, or Christopher Larkin’s identity. Sound choice and performance create the color.',
	play: {
		prompt:
			'Play one soft two-to-five-note minor phrase, hold the destination, release completely, then leave a full bar of silence before one small variation.',
		input: 'single_notes',
		articulation:
			'Soft, rounded, non-overlapping notes with clean releases; keyboard pedal up, guitar strings muted, and no bends, slides, harmonics, wide vibrato, or ringing strings.',
		density: 'Two to five source attacks per phrase; exactly one physical source note at a time.',
		space: 'Hold the destination for three to four beats, then leave at least one full bar silent.',
		tempo: '56–76 BPM; transport is optional and remains untouched.',
		transportRequired: false
	},
	references: [
		{
			name: 'Christopher Larkin',
			context:
				'Bounded reference to selected sparse-to-climactic area and narrative scoring from the 2014–18 Hollow Knight project, not a cue reconstruction, endorsement, or artist simulation.'
		}
	],
	researchStatus: 'approved',
	requirements: ['harmony'],
	config: {
		harmony: {
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

export const CRYSTAL_CHORALE_PRESET: ArrangementPresetV2 = {
	schemaVersion: 2,
	id: '48-crystal-chorale',
	name: 'Crystal Chorale',
	family: 'game',
	tags: ['fantasy-rpg', 'harmonic-minor', 'satb-shadow', 'octave-echo'],
	builtIn: true,
	result:
		'Each clean harmonic-minor melody note receives a four-part SATB-style chord and one complete octave echo two beats later.',
	approximation:
		'A bounded melody-first fantasy-RPG arrangement study informed by selected 1987–94 console practice. Harmonic minor and the per-note octave echo are product choices, not Nobuo Uematsu signatures. It does not compose or recognize themes, replay phrases, reproduce the Prelude or any score, orchestrate a choir, follow game or narrative state, develop form, or simulate Uematsu.',
	play: {
		prompt:
			'Perform an original four-bar single-note phrase, repeat its opening with one small change, hold the cadence for two to four beats, then release and leave a full bar for the echo tail.',
		input: 'motif',
		articulation:
			'Clean, lightly connected non-overlapping notes; keyboard pedal up, guitar strings muted, and no bends, slides, harmonics, ringing strings, or effects before detection.',
		density: 'Three to six source attacks per bar; exactly one physical source note at a time.',
		space: 'Hold the cadence for two to four beats, release, and leave at least one full bar for the delayed tail and silence.',
		tempo: '72–88 BPM in 4/4; 80 BPM is the test default.',
		transportRequired: true
	},
	references: [
		{
			name: 'Nobuo Uematsu',
			context:
				'Bounded reference to melody-first, limited-voice, thematically recurring arrangement practice in selected 1987–94 Final Fantasy console scores, not a franchise-theme reconstruction, endorsement, or artist simulation.'
		}
	],
	researchStatus: 'approved',
	requirements: ['harmony', 'free_imitation'],
	config: {
		harmony: {
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
		},
		companion: {
			enabled: true,
			globalHoldMode: { kind: 'cancel' },
			canon: {
				enabled: true,
				form: 'free_imitation',
				holdMode: { kind: 'forever' },
				voices: [singleLineFollower(7, 1, 2)]
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
	{ number: 1, name: 'Cloister Organum', family: 'classical', result: CLOISTER_ORGANUM_PRESET.result, prompt: CLOISTER_ORGANUM_PRESET.play.prompt, references: ['Léonin', 'Pérotin'], requirements: ['harmony', 'interval_stacks'] },
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
	{ number: 25, name: 'Bebop Chase', family: 'jazz', result: BEBOP_CHASE_PRESET.result, prompt: BEBOP_CHASE_PRESET.play.prompt, references: ['Charlie Parker', 'Dizzy Gillespie'], requirements: ['free_imitation'], input: 'motif', transportRequired: true },
	{ number: 26, name: 'Rootless Glass', family: 'jazz', result: 'Smooth rootless ninth, eleventh, and thirteenth voicings follow the lead.', prompt: 'Play ballad lines, sustained upper tones, and gentle chromatic approaches.', references: ['Bill Evans'], requirements: ['jazz_extensions'] },
	{ number: 27, name: 'Quartal Colossus', family: 'jazz', result: QUARTAL_COLOSSUS_PRESET.result, prompt: QUARTAL_COLOSSUS_PRESET.play.prompt, references: ['McCoy Tyner'], requirements: ['harmony'], input: 'motif' },
	{ number: 28, name: 'Third-Cycle Labyrinth', family: 'jazz', result: 'Tonal centers rotate through major thirds.', prompt: 'Repeat a compact motif while harmony changes beneath it.', references: ['John Coltrane'], requirements: ['harmonic_timeline'], input: 'motif', transportRequired: true },
	{ number: 29, name: 'Shout Counterpoint', family: 'jazz', result: 'Bass ostinato, gospel response, and unruly counterline combine.', prompt: 'Play forceful two-bar blues calls followed by a full response opening.', references: ['Charles Mingus'], requirements: ['pattern_lane', 'stable_lane_groups'], input: 'motif', transportRequired: true },
	{ number: 30, name: 'Free Quartet', family: 'jazz', result: 'Independent contours occupy separate tonal centers.', prompt: 'Play exploratory lines with wide leaps and pauses to answer the ensemble.', references: ['Ornette Coleman'], requirements: ['independent_tonal_centers', 'stable_lane_groups'] },
	{ number: 31, name: 'Side-Slip Matrix', family: 'jazz', result: 'Quartal harmony slips a semitone outside and returns.', prompt: 'Play a simple inside-key modal motif and leave room for outside motion.', references: ['Herbie Hancock'], requirements: ['harmonic_timeline'], input: 'motif', transportRequired: true },
	{ number: 32, name: 'Harmonic Maze', family: 'jazz', result: 'Harmony moves ambiguously among diatonic and symmetric regions.', prompt: 'Play sustained unresolved cells that avoid obvious cadences.', references: ['Wayne Shorter'], requirements: ['harmonic_timeline'], input: 'motif' },
	{ number: 33, name: 'Metric Labyrinth', family: 'jazz', result: 'Percussive counterpoint crosses uneven groupings.', prompt: 'Repeat an exact accented figure in 5, 7, or grouped 4/4.', references: ['Tigran Hamasyan', 'Vijay Iyer'], requirements: ['odd_meter', 'pattern_lane'], input: 'motif', transportRequired: true },
	{ number: 34, name: 'Chamber Sky', family: 'jazz', result: 'Lyrical voices expand into airy ensemble harmony.', prompt: 'Play breath-shaped arcs with wide expressive intervals and long releases.', references: ['Maria Schneider', 'Kenny Wheeler'], requirements: ['stable_lane_groups'] },
	{ number: 35, name: 'Elastic Counter-Groove', family: 'jazz', result: 'Syncopated harmony alternates with contrapuntal answers.', prompt: 'Play concise one- or two-bar riffs with deliberate holes.', references: ['Esperanza Spalding', 'Brad Mehldau', 'Robert Glasper'], requirements: ['pattern_lane'], input: 'motif', transportRequired: true },
	{ number: 36, name: 'Pixel Trio', family: 'game', result: PIXEL_TRIO_PRESET.result, prompt: PIXEL_TRIO_PRESET.play.prompt, references: ['Selected early Famicom/NES practice'], requirements: ['pattern_lane', 'stable_lane_groups', 'role_mix'], input: 'motif', transportRequired: true },
	{ number: 37, name: 'Fractured Crystal', family: 'game', result: 'A tiny motif fractures into limited-voice ostinatos and modal echoes.', prompt: 'Repeat a three-to-five-note motif slowly, alter one note, and leave substantial air.', references: ['Disasterpeace — FEZ'], requirements: ['motif_memory', 'pattern_lane', 'probability_density'], input: 'motif', transportRequired: true },
	{ number: 38, name: 'Summit Pulse', family: 'game', result: 'An intimate line grows into rhythmic upper layers.', prompt: 'Begin softly with hesitant fragments, then increase register, velocity, and repetition.', references: ['Lena Raine'], requirements: ['adaptive_scenes'], input: 'motif' },
	{ number: 39, name: 'Adaptive Pilgrim', family: 'game', result: 'The arrangement follows an emotional intensity arc.', prompt: 'Play isolated sustained notes, then longer and louder phrases, then withdraw.', references: ['Austin Wintory', 'Jessica Curry', 'Gareth Coker'], requirements: ['adaptive_scenes'] },
	{ number: 40, name: 'Ash and Ember', family: 'game', result: 'Rough modal riffs receive acoustic, metallic, and vocal-like answers.', prompt: 'Play gritty short riffs, strong downbeats, slides, and dramatic pauses.', references: ['Darren Korb', 'Ashley Barrett'], requirements: ['pattern_lane'], input: 'motif', transportRequired: true },
	{ number: 41, name: 'Blocks at Dusk', family: 'game', result: 'Sparse figures accumulate soft extended harmony.', prompt: 'Repeat two or three gentle notes with small variations and long decay space.', references: ['C418'], requirements: ['pattern_lane', 'probability_density'], input: 'motif', transportRequired: true },
	{ number: 42, name: 'Clockwork Stars', family: 'game', result: 'Recurring motifs return across registers and counter-rhythms.', prompt: 'Repeat a clean four-to-eight-note motif while preserving its rhythm.', references: ['Ben Prunty'], requirements: ['motif_memory', 'motif_transform'], input: 'motif' },
	{ number: 43, name: 'Hollow Choir', family: 'game', result: HOLLOW_CHOIR_PRESET.result, prompt: HOLLOW_CHOIR_PRESET.play.prompt, references: ['Christopher Larkin'], requirements: ['harmony'] },
	{ number: 44, name: 'Memory Weave', family: 'game', result: 'Two learned themes return transformed and combined.', prompt: 'Teach two distinct motifs, repeating each twice with a pause.', references: ['Toby Fox'], requirements: ['motif_memory', 'motif_transform'], input: 'motif' },
	{ number: 45, name: 'Neon Noir', family: 'game', result: 'Minor jazz-funk harmony gains chromatic passing motion and rhythmic responses.', prompt: 'Play tight syncopated riffs, repeated notes, and short chromatic runs.', references: ['Shoji Meguro'], requirements: ['pattern_lane', 'jazz_extensions'], input: 'motif', transportRequired: true },
	{ number: 46, name: 'Forest Layer Unfold', family: 'game', result: 'Quiet modal material gains naturalistic layers as density rises.', prompt: 'Begin sparse and pentatonic, then add repeats and higher notes gradually.', references: ['David Wise'], requirements: ['adaptive_scenes'] },
	{ number: 47, name: 'Time-Cross Rhythm', family: 'game', result: 'Compound-meter melody meets duple counter-rhythms.', prompt: 'Repeat a six- or twelve-beat cell with strong internal accents.', references: ['Yasunori Mitsuda'], requirements: ['polymeter', 'pattern_lane'], input: 'motif', transportRequired: true },
	{ number: 48, name: 'Crystal Chorale', family: 'game', result: CRYSTAL_CHORALE_PRESET.result, prompt: CRYSTAL_CHORALE_PRESET.play.prompt, references: ['Nobuo Uematsu'], requirements: ['harmony', 'free_imitation'], input: 'motif', transportRequired: true },
	{ number: 49, name: 'Kingdom Counterline', family: 'game', result: 'A lyrical lead gains urgent inner lines and dramatic harmonic turns.', prompt: 'Play a memorable theme with expressive leaps, dynamics, and phrase-end rests.', references: ['Yoko Shimomura'], requirements: ['motif_memory', 'harmonic_timeline'], input: 'motif' },
	{ number: 50, name: 'Rust and Static', family: 'game', result: 'Fragile fragments meet unstable clusters, shadows, and empty space.', prompt: 'Play isolated notes, slides, tritones, muted attacks, and unpredictable pauses.', references: ['Akira Yamaoka'], requirements: ['bounded_clusters', 'probability_density'] }
];

export const BUILT_IN_ARRANGEMENT_PRESETS: readonly ArrangementPresetV2[] = DRAFT_SPECS.map(
	(spec) => {
		if (spec.number === 1) return CLOISTER_ORGANUM_PRESET;
		if (spec.number === 2) return MODAL_LINEWORK_PRESET;
		if (spec.number === 4) return MENSURATION_WEB_PRESET;
		if (spec.number === 7) return STRETTO_ENGINE_PRESET;
		if (spec.number === 8) return SUSPENSION_GARLAND_PRESET;
		if (spec.number === 12) return PLANED_CATHEDRAL_PRESET;
		if (spec.number === 14) return COLOR_MODE_WINDOWS_PRESET;
		if (spec.number === 23) return SIXTH_DIMINISHED_CONVEYOR_PRESET;
		if (spec.number === 25) return BEBOP_CHASE_PRESET;
		if (spec.number === 27) return QUARTAL_COLOSSUS_PRESET;
		if (spec.number === 36) return PIXEL_TRIO_PRESET;
		if (spec.number === 43) return HOLLOW_CHOIR_PRESET;
		if (spec.number === 48) return CRYSTAL_CHORALE_PRESET;
		return draftPreset(spec);
	}
);

export const OPERATIONAL_BUILT_IN_ARRANGEMENT_PRESETS = BUILT_IN_ARRANGEMENT_PRESETS.filter(
	(preset) => preset.researchStatus === 'approved'
);

function singleLineFollower(transposeDegrees: number, timeRatio: number, delayBeats = 0) {
	return {
		delayBeats,
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
