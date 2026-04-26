<script lang="ts">
	import {
		engine,
		ALL_KEYS,
		KEY_DISPLAY,
		ALL_MODES,
		SCALE_FAMILIES,
		OCTAVE_MODES,
		VOICE_LEADING_STYLES,
		COUNTERPOINT_SPECIES,
		COUNTERPOINT_STRICTNESS,
		type HarmonyModeName,
		type ScaleModeName,
		type OctaveModeName,
		type VoiceLeadingStyleName,
		type CounterpointSpeciesName,
		type CounterpointStrictnessName,
		type KeyName
	} from '$lib/stores/engine.svelte';
	import PixelSelect from './PixelSelect.svelte';

	// --- Dropdown option lists ---
	let keyOptions = ALL_KEYS.map((k) => ({ value: k, label: KEY_DISPLAY[k] }));
	let modeOptions = ALL_MODES.map((m) => ({ value: m.name, label: m.label }));
	let octaveOptions = OCTAVE_MODES.map((om) => ({ value: om.name, label: om.label }));
	let vlStyleOptions = VOICE_LEADING_STYLES.map((s) => ({ value: s.name, label: s.label }));
	let familyOptions = SCALE_FAMILIES.map((g) => ({ value: g.family, label: g.label }));
	let speciesOptions = COUNTERPOINT_SPECIES.map((sp) => ({ value: sp.name, label: sp.label }));
	let strictnessOptions = COUNTERPOINT_STRICTNESS.map((s) => ({ value: s.name, label: s.label }));

	// --- Current scale family derived from the selected scale mode ---
	let currentFamily = $derived(
		SCALE_FAMILIES.find((g) => g.modes.some((m) => m.name === engine.scaleMode))?.family ??
			SCALE_FAMILIES[0].family
	);

	// --- Scale-mode options for the currently-selected family ---
	let scaleInFamilyOptions = $derived(
		(SCALE_FAMILIES.find((g) => g.family === currentFamily)?.modes ?? []).map((m) => ({
			value: m.name,
			label: m.label
		}))
	);

	// --- Handlers ---
	function onFamilyChange(family: string) {
		const group = SCALE_FAMILIES.find((g) => g.family === family);
		if (!group || group.modes.length === 0) return;
		// If the current scale mode isn't in the new family, switch to its first mode.
		if (!group.modes.some((m) => m.name === engine.scaleMode)) {
			engine.setScaleMode(group.modes[0].name);
		}
	}

	function onScaleModeChange(name: string) {
		engine.setScaleMode(name as ScaleModeName);
	}

	function onModeChange(name: string) {
		engine.setMode(name as HarmonyModeName);
	}

	function onOctaveChange(name: string) {
		engine.setOctaveMode(name as OctaveModeName);
	}

	function onVlStyleChange(name: string) {
		engine.setVoiceLeading(true, name as VoiceLeadingStyleName);
	}

	function onSpeciesChange(name: string) {
		engine.setCounterpointSpecies(name as CounterpointSpeciesName);
	}

	function onStrictnessChange(name: string) {
		engine.setCounterpointStrictness(name as CounterpointStrictnessName);
	}

	function onKeyChange(value: string) {
		engine.setAutoKey(false);
		engine.setKey(value as KeyName);
	}

	function onVoiceCountChange(value: string) {
		engine.setVoiceCount(parseInt(value, 10));
	}
</script>

<!-- Key + Mode + Octave — three pickers in one strip. The Auto-key
     toggle is inline under Key so the chrome cost is one card header
     instead of three. -->
<div class="card">
	<div class="row-3col">
		<div class="cell">
			<div class="cell-label-row">
				<span class="cell-label font-ui">Key</span>
				<button
					class="pixel-btn auto-key-btn"
					class:toggle-on={engine.autoKey}
					onclick={() => engine.setAutoKey(!engine.autoKey)}
					title="Auto-detect key from played notes"
				>
					{engine.autoKey ? 'AUTO' : 'MAN'}
				</button>
			</div>
			<PixelSelect options={keyOptions} value={engine.key} placeholder="Key" small={true} onchange={onKeyChange} />
		</div>
		<div class="cell">
			<span class="cell-label font-ui">Mode</span>
			<PixelSelect
				options={modeOptions}
				value={engine.mode}
				placeholder="Mode"
				small={true}
				onchange={onModeChange}
			/>
		</div>
		<div class="cell">
			<span class="cell-label font-ui">Octave</span>
			<PixelSelect
				options={octaveOptions}
				value={engine.octaveMode}
				placeholder="Octave"
				small={true}
				onchange={onOctaveChange}
			/>
		</div>
	</div>
</div>

<!-- Counterpoint Species + Strictness (only when mode = StrictCounterpoint) -->
{#if engine.mode === 'StrictCounterpoint'}
	<div class="row-2col">
		<div class="card">
			<div class="card-header font-ui">Species</div>
			<PixelSelect
				options={speciesOptions}
				value={engine.counterpointSpecies}
				placeholder="Species"
				onchange={onSpeciesChange}
			/>
		</div>
		<div class="card">
			<div class="card-header font-ui">Strictness</div>
			<PixelSelect
				options={strictnessOptions}
				value={engine.counterpointStrictness}
				placeholder="Strictness"
				onchange={onStrictnessChange}
			/>
		</div>
	</div>
{/if}

<!-- Scale Family + Scale Mode in family (side by side) -->
<div class="row-2col">
	<div class="card">
		<div class="card-header font-ui">Family</div>
		<PixelSelect
			options={familyOptions}
			value={currentFamily}
			placeholder="Family"
			onchange={onFamilyChange}
		/>
	</div>
	<div class="card">
		<div class="card-header font-ui">Scale</div>
		<PixelSelect
			options={scaleInFamilyOptions}
			value={engine.scaleMode}
			placeholder="Scale"
			onchange={onScaleModeChange}
		/>
	</div>
</div>

<!-- Voice Leading + Interchange (side by side) -->
<div class="row-2col">
	<div class="card">
		<div class="card-header font-ui">
			<span>Voice Leading</span>
			<button
				class="pixel-btn toggle-btn"
				class:toggle-on={engine.voiceLeadingEnabled}
				onclick={() => engine.setVoiceLeading(!engine.voiceLeadingEnabled)}
			>
				{engine.voiceLeadingEnabled ? 'ON' : 'OFF'}
			</button>
		</div>
		{#if engine.voiceLeadingEnabled}
			<PixelSelect
				options={vlStyleOptions}
				value={engine.voiceLeadingStyle}
				placeholder="Style"
				onchange={onVlStyleChange}
			/>
		{/if}
	</div>
	<div class="card">
		<div class="card-header font-ui">
			<span>Interchange</span>
			<button
				class="pixel-btn toggle-btn"
				class:toggle-on={engine.interchangeEnabled}
				onclick={() => engine.setInterchange(!engine.interchangeEnabled)}
			>
				{engine.interchangeEnabled ? 'ON' : 'OFF'}
			</button>
		</div>
		{#if engine.interchangeEnabled}
			<div class="range-row">
				<span class="range-label font-ui">Rng</span>
				<input
					type="range"
					min="1"
					max="5"
					step="1"
					value={engine.interchangeRange}
					oninput={(e) => engine.setInterchange(true, parseInt((e.target as HTMLInputElement).value, 10))}
					class="pixel-range"
				/>
				<span class="range-label font-code">{engine.interchangeRange}</span>
			</div>
		{/if}
	</div>
</div>

<!-- Detune — compact strip: label + slider + readout + presets in one
     row, no card-header chrome. -->
<div class="card detune-strip">
	<span class="cell-label font-ui">Detune</span>
	<input
		type="range"
		min="-100"
		max="100"
		step="1"
		value={engine.detuneCents}
		oninput={(e) => engine.setDetune(parseInt((e.target as HTMLInputElement).value, 10))}
		class="pixel-range"
	/>
	<span class="range-label font-code detune-readout">{engine.detuneCents > 0 ? '+' : ''}{engine.detuneCents}¢</span>
	<div class="detune-presets">
		<button class="pixel-btn" class:active={engine.detuneCents === 0} onclick={() => engine.setDetune(0)}>0</button>
		<button class="pixel-btn" class:active={engine.detuneCents === 33} onclick={() => engine.setDetune(33)}>+33</button>
		<button class="pixel-btn" class:active={engine.detuneCents === -33} onclick={() => engine.setDetune(-33)}>-33</button>
		<button class="pixel-btn" class:active={engine.detuneCents === 50} onclick={() => engine.setDetune(50)}>+50</button>
	</div>
</div>

<style>
	.card {
		background: var(--color-widget-bg);
		border: 1px solid var(--color-border);
		padding: 6px;
		margin-bottom: 4px;
		border-radius: 0;
	}

	.card-header {
		color: var(--color-accent-gold);
		font-size: var(--font-size-xs);
		margin-bottom: 4px;
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 4px;
		-webkit-font-smoothing: none;
		text-rendering: optimizeSpeed;
	}

	.row-2col {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 4px;
		align-items: stretch;
	}

	.row-2col > .card {
		margin-bottom: 4px;
	}

	/* Compact 3-column strip used by the Key/Mode/Octave card. Each
	   cell carries its own tiny label so the parent card doesn't need
	   a header. */
	.row-3col {
		display: grid;
		grid-template-columns: 1fr 1fr 1fr;
		gap: 6px;
		align-items: end;
	}

	.cell {
		display: flex;
		flex-direction: column;
		gap: 2px;
		min-width: 0;
	}

	.cell-label {
		color: var(--color-accent-gold);
		font-size: var(--font-size-xs);
		-webkit-font-smoothing: none;
		text-rendering: optimizeSpeed;
	}

	.cell-label-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 4px;
		min-height: 14px;
	}

	/* Detune is one horizontal strip — no header, no preset row below.
	   Slider takes the available space; readout + presets sit inline. */
	.detune-strip {
		display: flex;
		align-items: center;
		gap: 6px;
	}
	.detune-strip .pixel-range {
		flex: 1;
		min-width: 60px;
	}
	.detune-readout {
		min-width: 36px;
		text-align: right;
	}

	.auto-key-btn {
		font-size: var(--font-size-xs) !important;
		padding: 2px 6px !important;
		margin: 0;
	}

	.auto-key-btn.toggle-on {
		background: var(--color-accent-teal);
		border-color: var(--color-accent-cyan);
		box-shadow: var(--glow-teal);
		color: #ffffff;
	}

	.toggle-btn {
		min-width: 40px;
		padding: 2px 6px !important;
		font-size: var(--font-size-xs) !important;
	}

	.toggle-btn.toggle-on {
		background: var(--color-accent-teal);
		border-color: var(--color-accent-cyan);
		box-shadow: var(--glow-teal);
		color: #ffffff;
	}

	.range-row {
		display: flex;
		align-items: center;
		gap: 6px;
		margin-top: 4px;
	}

	.range-label {
		color: var(--color-text-secondary);
		font-size: var(--font-size-xs);
		white-space: nowrap;
		-webkit-font-smoothing: none;
		text-rendering: optimizeSpeed;
	}

	.pixel-range {
		flex: 1;
		height: 4px;
		-webkit-appearance: none;
		appearance: none;
		background: var(--color-widget-inactive);
		border: 1px solid var(--color-border);
		border-radius: 0;
		outline: none;
	}

	.pixel-range::-webkit-slider-thumb {
		-webkit-appearance: none;
		appearance: none;
		width: 10px;
		height: 14px;
		background: var(--color-accent-amber);
		border: 1px solid var(--color-accent-gold);
		border-radius: 0;
		cursor: pointer;
	}

	.pixel-range::-moz-range-thumb {
		width: 10px;
		height: 14px;
		background: var(--color-accent-amber);
		border: 1px solid var(--color-accent-gold);
		border-radius: 0;
		cursor: pointer;
	}

	.detune-presets {
		display: grid;
		grid-template-columns: repeat(4, 1fr);
		gap: 2px;
		margin-top: 4px;
	}

	/* Inside the inline detune strip the presets sit alongside the
	   slider, not below it — drop the top margin and let them size
	   to content. */
	.detune-strip .detune-presets {
		grid-template-columns: repeat(4, auto);
		margin-top: 0;
	}
</style>
