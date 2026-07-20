/**
 * Shared "composite Mode dropdown" helper.
 *
 * The Mode picker in the Harmony tab, the per-voice Mode picker in
 * the Companion's canon voice cards, and the Mode picker in the
 * Voices tab editor all expose the same thing: the harmony-engine
 * modes, with Counterpoint expanded into its four Fux species. Picking
 * a Counterpoint variant sets BOTH the harmony_mode and the species
 * in one click; everything else just sets the mode. Strictness is
 * always 'Strict' — no Relaxed control surfaced.
 *
 * Encoding: plain modes use their canonical name (`PassThrough`,
 * `BachChorale`, …). Counterpoint variants encode as
 * `StrictCounterpoint:SpeciesN`. This file is the single source of
 * truth for that wire format — callers should NEVER duplicate the
 * split/join logic.
 */
import { adapter } from '$lib/adapter';
import {
	ALL_MODES,
	type CounterpointSpeciesName,
	type HarmonyModeName
} from '$lib/stores/engine.svelte';
import { formatMusicalString } from '$lib/embed/music-utils';

export interface CompositeModeOption {
	value: string;
	label: string;
}

/** Build the dropdown options. Counterpoint expands to 4 species
 *  entries; every other mode keeps its single entry. `labelFormatter`
 *  defaults to `formatMusicalString` (used by the Harmony tab); pass
 *  identity to keep the raw mode label. */
export function compositeModeOptions(
	labelFormatter: (s: string) => string = formatMusicalString
): CompositeModeOption[] {
	return ALL_MODES
		.filter((mode) => adapter.capabilities.intervalMaps || mode.name !== 'ExplicitIntervals')
		.flatMap((m) => {
		const display = labelFormatter(m.label);
		if (m.name === 'StrictCounterpoint') {
			return [
				{ value: 'StrictCounterpoint:Species1', label: `${display} (1:1)` },
				{ value: 'StrictCounterpoint:Species2', label: `${display} (2:1 rules)` },
				{ value: 'StrictCounterpoint:Species3', label: `${display} (4:1 rules)` },
				{ value: 'StrictCounterpoint:Species4', label: `${display} (syncopated rules)` }
			];
		}
			return [{ value: m.name, label: display }];
		});
}

/** Encode (harmony_mode, counterpoint_species) → composite dropdown
 *  value. Counterpoint without a species defaults to Species1. Empty
 *  mode returns `''` (inherit / unset). */
export function encodeMode(
	mode: HarmonyModeName | string | null | undefined,
	species: CounterpointSpeciesName | string | null | undefined
): string {
	if (!mode) return '';
	if (mode === 'StrictCounterpoint') {
		return `StrictCounterpoint:${species ?? 'Species1'}`;
	}
	return mode;
}

/** Decode a composite dropdown value back into the pair. Counterpoint
 *  values produce `{mode: 'StrictCounterpoint', species: 'SpeciesN'}`;
 *  plain modes produce `{mode, species: null}`; empty string
 *  produces `{mode: null, species: null}`. */
export function decodeMode(v: string): {
	mode: HarmonyModeName | null;
	species: CounterpointSpeciesName | null;
} {
	if (!v) return { mode: null, species: null };
	if (v.startsWith('StrictCounterpoint:')) {
		return {
			mode: 'StrictCounterpoint' as HarmonyModeName,
			species: v.slice('StrictCounterpoint:'.length) as CounterpointSpeciesName
		};
	}
	return { mode: v as HarmonyModeName, species: null };
}
