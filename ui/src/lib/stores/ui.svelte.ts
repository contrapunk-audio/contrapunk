/** Reactive preferences for the canonical Contrapunk workspace. */

const SCALE_KEY = 'contrapunk-ui-scale';
const FONT_SCALE_KEY = 'contrapunk-font-scale';
const NOTE_LABELS_KEY = 'contrapunk-show-note-labels';
const NOTE_LINGERING_KEY = 'contrapunk-note-lingering';
const PIANO_KEY_COUNT_KEY = 'contrapunk-piano-key-count';

const MIN_SCALE = 0.75;
const MAX_SCALE = 2.0;
const MIN_FONT_SCALE = 0.75;
const MAX_FONT_SCALE = 1.5;

export const PIANO_KEY_COUNTS = [25, 32, 37, 49, 61, 76, 88] as const;
export type PianoKeyCount = (typeof PIANO_KEY_COUNTS)[number];

class UiStore {
	reducedMotion = $state(false);
	animationsEnabled = $state(true);
	uiScale = $state(1.0);
	fontScale = $state(1.0);
	showNoteLabels = $state(true);
	pianoKeyCount = $state<PianoKeyCount>(49);
	noteLingering = $state(true);

	toggleAnimations() {
		this.animationsEnabled = !this.animationsEnabled;
		this.reducedMotion = !this.animationsEnabled;
		this.applyMotionPreference();
	}

	applyMotionPreference() {
		if (typeof document === 'undefined') return;
		document.body.classList.toggle('reduced-motion', this.reducedMotion);
	}

	detectSystemMotionPreference() {
		if (typeof window === 'undefined') return;
		const mediaQuery = window.matchMedia('(prefers-reduced-motion: reduce)');
		const apply = (reduced: boolean) => {
			this.reducedMotion = reduced;
			this.animationsEnabled = !reduced;
			this.applyMotionPreference();
		};
		if (mediaQuery.matches) apply(true);
		mediaQuery.addEventListener('change', (event) => apply(event.matches));
	}

	setUiScale(scale: number) {
		this.uiScale = Math.max(MIN_SCALE, Math.min(MAX_SCALE, scale));
		this.applyUiScale();
		this.persist(SCALE_KEY, String(this.uiScale));
	}

	applyUiScale() {
		if (typeof document === 'undefined') return;
		document.documentElement.style.setProperty('--ui-scale', String(this.uiScale));
	}

	setFontScale(scale: number) {
		this.fontScale = Math.max(MIN_FONT_SCALE, Math.min(MAX_FONT_SCALE, scale));
		this.applyFontScale();
		this.persist(FONT_SCALE_KEY, String(this.fontScale));
	}

	applyFontScale() {
		if (typeof document === 'undefined') return;
		document.documentElement.style.setProperty('--font-scale', String(this.fontScale));
	}

	setShowNoteLabels(on: boolean) {
		this.showNoteLabels = on;
		this.persist(NOTE_LABELS_KEY, on ? 'on' : 'off');
	}

	setNoteLingering(on: boolean) {
		this.noteLingering = on;
		this.persist(NOTE_LINGERING_KEY, on ? 'on' : 'off');
	}

	setPianoKeyCount(count: number) {
		if (!PIANO_KEY_COUNTS.includes(count as PianoKeyCount)) return;
		this.pianoKeyCount = count as PianoKeyCount;
		this.persist(PIANO_KEY_COUNT_KEY, String(count));
	}

	restoreAppearance() {
		if (typeof window === 'undefined') return;
		try {
			const scale = Number(localStorage.getItem(SCALE_KEY));
			if (Number.isFinite(scale) && scale > 0) {
				this.uiScale = Math.max(MIN_SCALE, Math.min(MAX_SCALE, scale));
			}
			const fontScale = Number(localStorage.getItem(FONT_SCALE_KEY));
			if (Number.isFinite(fontScale) && fontScale > 0) {
				this.fontScale = Math.max(MIN_FONT_SCALE, Math.min(MAX_FONT_SCALE, fontScale));
			}
			this.showNoteLabels = localStorage.getItem(NOTE_LABELS_KEY) !== 'off';
			this.noteLingering = localStorage.getItem(NOTE_LINGERING_KEY) !== 'off';
			const keyCount = Number(localStorage.getItem(PIANO_KEY_COUNT_KEY));
			if (PIANO_KEY_COUNTS.includes(keyCount as PianoKeyCount)) {
				this.pianoKeyCount = keyCount as PianoKeyCount;
			}

			for (const className of Array.from(document.body.classList)) {
				if (className.startsWith('font-')) document.body.classList.remove(className);
			}
			localStorage.removeItem('contrapunk-ui-font');
		} catch {
			/* localStorage unavailable */
		}
		this.applyUiScale();
		this.applyFontScale();
	}

	private persist(key: string, value: string) {
		try {
			localStorage.setItem(key, value);
		} catch {
			/* localStorage unavailable */
		}
	}
}

export const ui = new UiStore();
