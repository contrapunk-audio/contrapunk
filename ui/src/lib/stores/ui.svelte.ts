/**
 * UI Store -- Reactive UI Preferences (Svelte 5 Runes)
 *
 * Manages UI preferences like animations, reduced motion, theme,
 * and platform detection. Persists preferences where possible.
 */

import { platformName } from '$lib/adapter';

// === Persistence ===

const SCALE_KEY = 'contrapunk-ui-scale';
const FONT_KEY = 'contrapunk-ui-font';

const MIN_SCALE = 0.75;
const MAX_SCALE = 2.0;

export type FontMode =
	| 'press-start'
	| 'vt323'
	| 'silkscreen'
	| 'pixelify'
	| 'dotgothic'
	| 'jersey'
	| 'tiny5'
	| 'workbench'
	| 'jetbrains'
	| 'fira'
	| 'plex'
	| 'clean';

export const FONT_OPTIONS: { value: FontMode; label: string; kind: 'pixel' | 'mono' | 'clean' }[] = [
	// Pixel / retro
	{ value: 'press-start', label: 'Press Start', kind: 'pixel' },
	{ value: 'vt323', label: 'VT323', kind: 'pixel' },
	{ value: 'silkscreen', label: 'Silkscreen', kind: 'pixel' },
	{ value: 'pixelify', label: 'Pixelify', kind: 'pixel' },
	{ value: 'dotgothic', label: 'DotGothic16', kind: 'pixel' },
	{ value: 'jersey', label: 'Jersey 10', kind: 'pixel' },
	{ value: 'tiny5', label: 'Tiny5', kind: 'pixel' },
	{ value: 'workbench', label: 'Workbench', kind: 'pixel' },
	// Readable mono
	{ value: 'jetbrains', label: 'JetBrains Mono', kind: 'mono' },
	{ value: 'fira', label: 'Fira Code', kind: 'mono' },
	{ value: 'plex', label: 'IBM Plex Mono', kind: 'mono' },
	// System sans
	{ value: 'clean', label: 'Clean', kind: 'clean' }
];

const FONT_CLASSES = FONT_OPTIONS.map((o) => `font-${o.value}`);

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
	/** Which font family the whole UI uses. See FONT_OPTIONS. */
	fontMode = $state<FontMode>('press-start');

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

	/**
	 * Mark the app as initialized (call after adapter.init() succeeds).
	 */
	markInitialized() {
		this.initialized = true;
		this.error = null;
	}

	/**
	 * Set an error state for display.
	 */
	setError(message: string) {
		this.error = message;
	}

	/**
	 * Clear the error state.
	 */
	clearError() {
		this.error = null;
	}

	/**
	 * Toggle sidebar collapsed state.
	 */
	toggleSidebar() {
		this.sidebarCollapsed = !this.sidebarCollapsed;
	}

	/**
	 * Set the active panel (e.g., 'play', 'craft', 'settings').
	 */
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

	// === Font mode ===

	setFontMode(mode: FontMode) {
		this.fontMode = mode;
		this.applyFontMode();
		try {
			localStorage.setItem(FONT_KEY, mode);
		} catch {
			/* localStorage unavailable */
		}
	}

	/** Cycle through the available font modes (used by the compact
	 *  keyboard-style toggle button). */
	cycleFontMode() {
		const idx = FONT_OPTIONS.findIndex((o) => o.value === this.fontMode);
		const next = FONT_OPTIONS[(idx + 1) % FONT_OPTIONS.length].value;
		this.setFontMode(next);
	}

	applyFontMode() {
		if (typeof document === 'undefined') return;
		const body = document.body;
		FONT_CLASSES.forEach((c) => body.classList.remove(c));
		body.classList.add(`font-${this.fontMode}`);
	}

	/** Restore persisted density + font preferences from localStorage. */
	restoreAppearance() {
		try {
			const savedScale = localStorage.getItem(SCALE_KEY);
			if (savedScale) {
				const n = parseFloat(savedScale);
				if (Number.isFinite(n)) {
					this.uiScale = Math.max(MIN_SCALE, Math.min(MAX_SCALE, n));
				}
			}
			const savedFont = localStorage.getItem(FONT_KEY);
			const valid = FONT_OPTIONS.map((o) => o.value);
			if (savedFont && (valid as string[]).includes(savedFont)) {
				this.fontMode = savedFont as FontMode;
			} else if (savedFont === 'pixel') {
				// Legacy value from the two-mode era.
				this.fontMode = 'press-start';
			}
		} catch {
			/* localStorage unavailable */
		}
		this.applyUiScale();
		this.applyFontMode();
	}
}

export const ui = new UiStore();
