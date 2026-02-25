/**
 * UI Store -- Reactive UI Preferences (Svelte 5 Runes)
 *
 * Manages UI preferences like animations, reduced motion, theme,
 * and platform detection. Persists preferences where possible.
 */

import { platformName } from '$lib/adapter';

// === UI Store (Svelte 5 runes) ===

class UiStore {
	// -- Accessibility --
	reducedMotion = $state(false);
	animationsEnabled = $state(true);

	// -- Platform --
	platform = $state<'tauri' | 'browser'>(platformName);

	// -- App state --
	initialized = $state(false);
	error = $state<string | null>(null);

	// -- Layout --
	sidebarCollapsed = $state(false);
	activePanel = $state<string>('play');

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
}

export const ui = new UiStore();
