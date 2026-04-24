/**
 * UI Store -- Reactive UI Preferences (Svelte 5 Runes)
 *
 * Manages UI preferences like animations, reduced motion, theme,
 * and platform detection. Persists preferences where possible.
 */

import { platformName } from '$lib/adapter';

// === Persistence ===

const SCALE_KEY = 'contrapunk-ui-scale';
const FONT_SCALE_KEY = 'contrapunk-font-scale';
const NOTE_LABELS_KEY = 'contrapunk-show-note-labels';

const MIN_SCALE = 0.75;
const MAX_SCALE = 2.0;
const MIN_FONT_SCALE = 0.75;
const MAX_FONT_SCALE = 1.5;

// === UI Store (Svelte 5 runes) ===

class UiStore {
	// -- Accessibility --
	reducedMotion = $state(false);
	animationsEnabled = $state(true);

	// -- Platform --
	platform = $state<'tauri' | 'browser' | 'plugin'>(platformName);

	// -- App state --
	initialized = $state(false);
	error = $state<string | null>(null);

	// -- Layout --
	sidebarCollapsed = $state(false);
	activePanel = $state<string>('play');
	/** Main-view tab: 'play' (harmony UI) or 'chain' (audio chain). */
	activeTab = $state<'play' | 'chain'>('play');

	// -- Settings modal --
	settingsOpen = $state(false);

	// -- Density / readability --
	/** Global UI zoom factor. Applied via CSS `zoom` on <html>. */
	uiScale = $state(1.0);
	/** Typography-only scale. Multiplies --font-size-* tokens without
	 *  affecting layout geometry the way uiScale does. */
	fontScale = $state(1.0);
	/** Whether to show "C4", "D#5" etc. labels on active piano keys + fretboard notes. */
	showNoteLabels = $state(true);

	/**
	 * Toggle animations on/off and apply the reduced-motion class
	 * to the document body.
	 */
	toggleAnimations() {
		this.animationsEnabled = !this.animationsEnabled;
		this.reducedMotion = !this.animationsEnabled;
		this.applyMotionPreference();
	}

	/**
	 * Apply the current reduced motion preference to the DOM.
	 * Adds or removes the .reduced-motion class on document.body.
	 */
	applyMotionPreference() {
		if (typeof document === 'undefined') return;

		if (this.reducedMotion) {
			document.body.classList.add('reduced-motion');
		} else {
			document.body.classList.remove('reduced-motion');
		}
	}

	/**
	 * Detect system reduced motion preference (prefers-reduced-motion media query)
	 * and sync our state to it on initialization.
	 */
	detectSystemMotionPreference() {
		if (typeof window === 'undefined') return;

		const mediaQuery = window.matchMedia('(prefers-reduced-motion: reduce)');
		if (mediaQuery.matches) {
			this.reducedMotion = true;
			this.animationsEnabled = false;
			this.applyMotionPreference();
		}

		// Listen for changes
		mediaQuery.addEventListener('change', (e) => {
			this.reducedMotion = e.matches;
			this.animationsEnabled = !e.matches;
			this.applyMotionPreference();
		});
	}

	markInitialized() {
		this.initialized = true;
		this.error = null;
	}

	setError(message: string) {
		this.error = message;
	}

	clearError() {
		this.error = null;
	}

	toggleSidebar() {
		this.sidebarCollapsed = !this.sidebarCollapsed;
	}

	setActivePanel(panel: string) {
		this.activePanel = panel;
	}

	openSettings() {
		this.settingsOpen = true;
	}

	closeSettings() {
		this.settingsOpen = false;
	}

	toggleSettings() {
		this.settingsOpen = !this.settingsOpen;
	}

	// === UI scale ===

	setUiScale(scale: number) {
		const clamped = Math.max(MIN_SCALE, Math.min(MAX_SCALE, scale));
		this.uiScale = clamped;
		this.applyUiScale();
		try {
			localStorage.setItem(SCALE_KEY, String(clamped));
		} catch {
			/* localStorage unavailable */
		}
	}

	applyUiScale() {
		if (typeof document === 'undefined') return;
		document.documentElement.style.setProperty('--ui-scale', String(this.uiScale));
	}

	// === Font scale (typography-only, independent of uiScale) ===

	setFontScale(scale: number) {
		const clamped = Math.max(MIN_FONT_SCALE, Math.min(MAX_FONT_SCALE, scale));
		this.fontScale = clamped;
		this.applyFontScale();
		try {
			localStorage.setItem(FONT_SCALE_KEY, String(clamped));
		} catch {
			/* localStorage unavailable */
		}
	}

	applyFontScale() {
		if (typeof document === 'undefined') return;
		document.documentElement.style.setProperty('--font-scale', String(this.fontScale));
	}

	// === Note labels ===

	setShowNoteLabels(on: boolean) {
		this.showNoteLabels = on;
		try {
			localStorage.setItem(NOTE_LABELS_KEY, on ? 'on' : 'off');
		} catch {
			/* localStorage unavailable */
		}
	}

	/** Restore persisted appearance preferences from localStorage.
	 *  Font family is no longer user-configurable — the app is locked
	 *  to the website's typography pair (Space Grotesk + JetBrains Mono).
	 *  Also cleans up the legacy `contrapunk-ui-font` key + any stale
	 *  `body.font-*` class applied by a previous app version so existing
	 *  installs don't keep rendering Press Start 2P. */
	restoreAppearance() {
		try {
			const savedScale = localStorage.getItem(SCALE_KEY);
			if (savedScale) {
				const n = parseFloat(savedScale);
				if (Number.isFinite(n)) {
					this.uiScale = Math.max(MIN_SCALE, Math.min(MAX_SCALE, n));
				}
			}
			const savedFontScale = localStorage.getItem(FONT_SCALE_KEY);
			if (savedFontScale) {
				const n = parseFloat(savedFontScale);
				if (Number.isFinite(n)) {
					this.fontScale = Math.max(MIN_FONT_SCALE, Math.min(MAX_FONT_SCALE, n));
				}
			}
			const savedLabels = localStorage.getItem(NOTE_LABELS_KEY);
			if (savedLabels === 'off') this.showNoteLabels = false;

			// Legacy cleanup — strip any font-mode body class applied by
			// older builds and drop the stale localStorage key.
			if (typeof document !== 'undefined') {
				const classes = Array.from(document.body.classList);
				for (const c of classes) {
					if (c.startsWith('font-')) document.body.classList.remove(c);
				}
			}
			localStorage.removeItem('contrapunk-ui-font');
		} catch {
			/* localStorage unavailable */
		}
		this.applyUiScale();
		this.applyFontScale();
	}
}

export const ui = new UiStore();
