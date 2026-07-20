/**
 * Type declarations for the contrapunk-wasm package.
 *
 * This module is built by wasm-pack from the wasm/ crate.
 * These declarations allow TypeScript to compile before the
 * WASM package is actually built.
 */
declare module 'contrapunk-wasm' {
	export default function init(): Promise<void>;

	export class Engine {
		constructor();
		free(): void;
		set_key(key: string): void;
		set_mode(mode: string): void;
		set_scale_mode(mode: string): void;
		set_octave_mode(mode: string): void;
		set_voice_leading(enabled: boolean, style: string): void;
		set_interchange(enabled: boolean, range: number): void;
		set_voice_position(position: number): void;
		set_voice_count(count: number): void;
		set_auto_key(enabled: boolean): void;
		set_explicit_interval_map(json: string): void;
		current_key(): string;
		harmonize(note: number): Uint8Array;
		note_on(note: number): Uint8Array;
		note_off(note: number): Uint8Array;
		get_state(): Record<string, unknown>;
		get_note_state(): Record<string, unknown>;
		list_presets(): Array<Record<string, unknown>>;
		load_preset(name: string): void;
		save_preset(name: string): void;
		delete_preset(name: string): void;
	}

	export function midi_to_name(midi: number): string;
}
