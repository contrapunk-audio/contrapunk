<script lang="ts">
	import {
		engine,
		ALL_KEYS,
		KEY_DISPLAY,
		ALL_MODES,
		SCALE_FAMILIES,
		OCTAVE_MODES,
		VOICE_LEADING_STYLES,
		type KeyName,
		type HarmonyModeName,
		type ScaleModeName,
		type OctaveModeName,
		type VoiceLeadingStyleName,
	} from '$lib/stores/engine.svelte';

	// Track which scale family section is expanded
	let expandedFamily = $state<string | null>('Diatonic');

	function toggleFamily(family: string) {
		expandedFamily = expandedFamily === family ? null : family;
	}

	// Voice position labels
	function voiceLabel(index: number, count: number): string {
		const names = ['Soprano', 'Alto', 'Tenor', 'Bass'];
		if (count <= 4) {
			return `${names[index] || 'Voice'} (${index + 1})`;
		}
		return `Voice (${index + 1})`;
	}
</script>

<!-- Key Selector -->
<div class="card">
	<div class="card-header font-pixel">
		Key
		<button
			class="pixel-btn auto-key-btn"
			class:toggle-on={engine.autoKey}
			onclick={() => engine.setAutoKey(!engine.autoKey)}
		>
			{engine.autoKey ? 'AUTO' : 'MANUAL'}
		</button>
	</div>
	<div class="key-grid">
		{#each ALL_KEYS as key}
			<button
				class="pixel-btn"
				class:active={engine.key === key}
				onclick={() => { engine.setAutoKey(false); engine.setKey(key); }}
			>
				{KEY_DISPLAY[key]}
			</button>
		{/each}
	</div>
</div>

<!-- Harmony Mode Selector -->
<div class="card">
	<div class="card-header font-pixel">Mode</div>
	<div class="mode-grid">
		{#each ALL_MODES as m}
			<button
				class="pixel-btn"
				class:active={engine.mode === m.name}
				onclick={() => engine.setMode(m.name)}
				title={m.label}
			>
				{m.shortLabel}
			</button>
		{/each}
	</div>
</div>

<!-- Scale Mode Selector (grouped by family) -->
<div class="card">
	<div class="card-header font-pixel">Scale</div>
	{#each SCALE_FAMILIES as group}
		<button
			class="family-header font-pixel"
			class:expanded={expandedFamily === group.family}
			onclick={() => toggleFamily(group.family)}
		>
			<span class="family-arrow">{expandedFamily === group.family ? 'v' : '>'}</span>
			{group.label}
		</button>
		{#if expandedFamily === group.family}
			<div class="scale-grid">
				{#each group.modes as mode}
					<button
						class="pixel-btn scale-btn"
						class:scale-active={engine.scaleMode === mode.name}
						onclick={() => engine.setScaleMode(mode.name)}
						title={mode.label}
					>
						{mode.label}
					</button>
				{/each}
			</div>
		{/if}
	{/each}
</div>

<!-- Octave Mode Selector -->
<div class="card">
	<div class="card-header font-pixel">Octave</div>
	<div class="octave-grid">
		{#each OCTAVE_MODES as om}
			<button
				class="pixel-btn"
				class:active={engine.octaveMode === om.name}
				onclick={() => engine.setOctaveMode(om.name)}
			>
				{om.label}
			</button>
		{/each}
	</div>
</div>

<!-- Voice Leading -->
<div class="card">
	<div class="card-header font-pixel">Voice Leading</div>
	<div class="toggle-row">
		<button
			class="pixel-btn toggle-btn"
			class:toggle-on={engine.voiceLeadingEnabled}
			onclick={() => engine.setVoiceLeading(!engine.voiceLeadingEnabled)}
		>
			{engine.voiceLeadingEnabled ? 'ON' : 'OFF'}
		</button>
	</div>
	{#if engine.voiceLeadingEnabled}
		<div class="style-grid">
			{#each VOICE_LEADING_STYLES as style}
				<button
					class="pixel-btn"
					class:active={engine.voiceLeadingStyle === style.name}
					onclick={() => engine.setVoiceLeading(true, style.name)}
				>
					{style.label}
				</button>
			{/each}
		</div>
	{/if}
</div>

<!-- Modal Interchange -->
<div class="card">
	<div class="card-header font-pixel">Interchange</div>
	<div class="toggle-row">
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
			<span class="range-label font-pixel">Range: {engine.interchangeRange}</span>
			<input
				type="range"
				min="1"
				max="5"
				step="1"
				value={engine.interchangeRange}
				oninput={(e) => engine.setInterchange(true, parseInt((e.target as HTMLInputElement).value, 10))}
				class="pixel-range"
			/>
		</div>
	{/if}
</div>

<!-- Voice Count & Position -->
<div class="card">
	<div class="card-header font-pixel">Voices</div>
	<div class="count-grid">
		{#each [1, 2, 3, 4] as count}
			<button
				class="pixel-btn"
				class:active={engine.voiceCount === count}
				onclick={() => engine.setVoiceCount(count)}
			>
				{count}
			</button>
		{/each}
	</div>
	{#if engine.voiceCount > 1}
		<div class="card-subheader font-pixel">You play</div>
		<div class="voice-grid">
			{#each Array.from({ length: engine.voiceCount }, (_, i) => i) as idx}
				<button
					class="pixel-btn"
					class:active={engine.voicePosition === idx}
					onclick={() => engine.setVoicePosition(idx)}
				>
					{voiceLabel(idx, engine.voiceCount)}
				</button>
			{/each}
		</div>
	{/if}
</div>

<!-- Detune -->
<div class="card">
	<div class="card-header font-pixel">Detune</div>
	<div class="range-row">
		<span class="range-label font-pixel">{engine.detuneCents > 0 ? '+' : ''}{engine.detuneCents}¢</span>
		<input
			type="range"
			min="-100"
			max="100"
			step="1"
			value={engine.detuneCents}
			oninput={(e) => engine.setDetune(parseInt((e.target as HTMLInputElement).value, 10))}
			class="pixel-range"
		/>
	</div>
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
		-webkit-font-smoothing: none;
		text-rendering: optimizeSpeed;
	}

	.card-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
	}

	.auto-key-btn {
		font-size: 6px !important;
		padding: 2px 6px !important;
		margin: 0;
	}

	.auto-key-btn.toggle-on {
		background: var(--color-accent-teal);
		border-color: var(--color-accent-cyan);
		box-shadow: var(--glow-teal);
		color: #ffffff;
	}

	.key-grid {
		display: grid;
		grid-template-columns: repeat(4, 1fr);
		gap: 2px;
	}

	.mode-grid {
		display: grid;
		grid-template-columns: repeat(4, 1fr);
		gap: 2px;
	}

	.scale-grid {
		display: grid;
		grid-template-columns: 1fr;
		gap: 2px;
		padding: 2px 0;
	}

	.octave-grid {
		display: grid;
		grid-template-columns: repeat(4, 1fr);
		gap: 2px;
	}

	.style-grid {
		display: grid;
		grid-template-columns: repeat(4, 1fr);
		gap: 2px;
		margin-top: 4px;
	}

	.count-grid {
		display: grid;
		grid-template-columns: repeat(4, 1fr);
		gap: 2px;
		margin-bottom: 4px;
	}

	.card-subheader {
		color: var(--color-text-secondary);
		font-size: 7px;
		margin: 4px 0 2px;
		-webkit-font-smoothing: none;
		text-rendering: optimizeSpeed;
	}

	.voice-grid {
		display: grid;
		grid-template-columns: 1fr;
		gap: 2px;
	}

	.family-header {
		display: flex;
		align-items: center;
		gap: 4px;
		width: 100%;
		background: var(--color-bg-panel);
		border: 1px solid var(--color-border);
		color: var(--color-accent-amber);
		font-size: 7px;
		padding: 3px 6px;
		cursor: pointer;
		border-radius: 0;
		margin-bottom: 1px;
		-webkit-font-smoothing: none;
		text-rendering: optimizeSpeed;
		text-align: left;
	}

	.family-header:hover {
		border-color: var(--color-accent-cyan-dim);
	}

	.family-header.expanded {
		border-color: var(--color-accent-cyan);
	}

	.family-arrow {
		font-size: 6px;
		width: 8px;
		text-align: center;
	}

	.scale-btn {
		font-size: 6px !important;
		padding: 3px 6px !important;
		text-align: left;
	}

	.scale-btn.scale-active {
		background: var(--color-accent-cyan-dim);
		border-color: var(--color-accent-cyan);
		box-shadow: var(--glow-cyan);
		color: #ffffff;
	}

	.toggle-row {
		display: flex;
		align-items: center;
		gap: 6px;
	}

	.toggle-btn {
		min-width: 40px;
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
		font-size: 7px;
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

	.detune-presets {
		display: grid;
		grid-template-columns: repeat(4, 1fr);
		gap: 2px;
		margin-top: 4px;
	}

	.pixel-range::-moz-range-thumb {
		width: 10px;
		height: 14px;
		background: var(--color-accent-amber);
		border: 1px solid var(--color-accent-gold);
		border-radius: 0;
		cursor: pointer;
	}
</style>
