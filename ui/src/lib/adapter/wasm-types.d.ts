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

		// Humanizer + Beat Clock + Metronome (added alongside the
		// WASM Humanizer port; see wasm/src/lib.rs).
		tick(now_ms: number): Record<string, unknown>;
		set_humanize_enabled(enabled: boolean): void;
		set_jitter_enabled(enabled: boolean): void;
		set_timing_jitter(max_ms: number): void;
		set_velocity_enabled(enabled: boolean): void;
		set_velocity_jitter(variation: number): void;
		set_duration_enabled(enabled: boolean): void;
		set_duration_variation(ms: number): void;
		set_swing_enabled(enabled: boolean): void;
		set_swing(amount: number): void;
		set_bpm(bpm: number): void;
		set_time_signature(beats_per_bar: number, beat_unit: number): void;
		set_metronome_enabled(enabled: boolean): void;
		beat_position(): number;
		bpm(): number;
		is_metronome_enabled(): boolean;
		is_humanize_enabled(): boolean;
		get_humanize_config(): Record<string, unknown>;
		set_humanize_config(config: Record<string, unknown>): void;
		humanized_note_on(note: number, velocity: number): Record<string, unknown>;
		humanized_note_off(note: number): Record<string, unknown>;
	}

	export function midi_to_name(midi: number): string;
}
