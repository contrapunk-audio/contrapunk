/**
 * Platform Adapter — Factory and Export
 *
 * Detects the runtime platform (Tauri desktop vs browser) and
 * exports the appropriate adapter implementation as a singleton.
 */

import type { ContrapunkAdapter } from './types';
import { TauriAdapter } from './tauri';
import { WasmAdapter } from './wasm';
import { PluginAdapter } from './plugin';

/**
 * Detect whether we are running inside a nih-plug-webview plugin.
 * The webview injects `window.plugin` with send/listen/resize methods.
 */
function isPlugin(): boolean {
	return typeof window !== 'undefined' && 'plugin' in window && typeof window.plugin?.send === 'function';
}

/**
 * Detect whether we are running inside a Tauri webview.
 *
 * Tauri v2 only injects `__TAURI_INTERNALS__` by default.
 * The `__TAURI__` global requires `withGlobalTauri: true` in
 * tauri.conf.json, which is not set. Check both to be safe.
 */
function isTauri(): boolean {
	return (
		typeof window !== 'undefined' &&
		('__TAURI_INTERNALS__' in window || '__TAURI__' in window)
	);
}

/** The active platform name, for display purposes. */
export const platformName: 'tauri' | 'browser' | 'plugin' = isPlugin()
	? 'plugin'
	: isTauri()
		? 'tauri'
		: 'browser';

/**
 * Singleton adapter instance.
 *
 * Components import this and call methods without knowing the backend:
 * ```ts
 * import { adapter } from '$lib/adapter';
 * await adapter.setKey('C');
 * ```
 */
export const adapter: ContrapunkAdapter = isPlugin()
	? new PluginAdapter()
	: isTauri()
		? new TauriAdapter()
		: new WasmAdapter();

// Re-export types for convenience
export type {
	ContrapunkAdapter,
	EngineState,
	GuitarConfig,
	MidiDevice,
	NoteState,
	Preset,
	SynthState,
	TransportState
} from './types';
