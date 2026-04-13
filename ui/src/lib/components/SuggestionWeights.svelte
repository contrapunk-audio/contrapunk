<script lang="ts">
	import {
		suggestions,
		WEIGHT_TERMS,
		SUGGESTION_PRESETS,
		type SuggestionWeightTerm
	} from '$lib/stores/suggestion.svelte';

	let expanded = $state(false);

	const harmonicTerms = WEIGHT_TERMS.filter((t) => t.group === 'harmonic');
	const melodicTerms = WEIGHT_TERMS.filter((t) => t.group === 'melodic');
	const contextualTerms = WEIGHT_TERMS.filter((t) => t.group === 'contextual');

	function handleWeightChange(term: SuggestionWeightTerm, e: Event) {
		const target = e.target as HTMLInputElement;
		suggestions.setWeight(term, parseFloat(target.value));
	}
</script>

<div class="suggestion-panel">
	<div class="panel-header">
		<button class="toggle-btn font-pixel" onclick={() => suggestions.toggle()}>
			<span class="toggle-indicator" class:active={suggestions.enabled}></span>
			Suggestions
		</button>
		{#if suggestions.enabled}
			<button
				class="expand-btn font-pixel"
				onclick={() => (expanded = !expanded)}
				title={expanded ? 'Collapse weights' : 'Expand weights'}
			>
				{expanded ? '[-]' : '[+]'}
			</button>
		{/if}
	</div>

	{#if suggestions.enabled && expanded}
		<div class="weights-container">
			<!-- Presets -->
			<div class="preset-row">
				{#each SUGGESTION_PRESETS as preset}
					<button
						class="preset-btn font-pixel"
						onclick={() => suggestions.applyPreset(preset)}
						title={preset.name}
					>
						{preset.name}
					</button>
				{/each}
			</div>

			<!-- Harmonic group -->
			<div class="weight-group">
				<div class="group-label font-pixel">Harmonic</div>
				{#each harmonicTerms as { term, label }}
					<div class="weight-row">
						<label class="weight-label font-pixel" for="w-{term}">{label}</label>
						<input
							type="range"
							id="w-{term}"
							class="weight-slider"
							min="0"
							max="10"
							step="0.5"
							value={suggestions.weights[term]}
							oninput={(e) => handleWeightChange(term, e)}
						/>
						<span class="weight-value font-pixel">{suggestions.weights[term].toFixed(1)}</span>
					</div>
				{/each}
			</div>

			<!-- Melodic group -->
			<div class="weight-group">
				<div class="group-label font-pixel">Melodic</div>
				{#each melodicTerms as { term, label }}
					<div class="weight-row">
						<label class="weight-label font-pixel" for="w-{term}">{label}</label>
						<input
							type="range"
							id="w-{term}"
							class="weight-slider"
							min="0"
							max="10"
							step="0.5"
							value={suggestions.weights[term]}
							oninput={(e) => handleWeightChange(term, e)}
						/>
						<span class="weight-value font-pixel">{suggestions.weights[term].toFixed(1)}</span>
					</div>
				{/each}
			</div>

			<!-- Contextual group -->
			<div class="weight-group">
				<div class="group-label font-pixel">Contextual</div>
				{#each contextualTerms as { term, label }}
					<div class="weight-row">
						<label class="weight-label font-pixel" for="w-{term}">{label}</label>
						<input
							type="range"
							id="w-{term}"
							class="weight-slider"
							min="0"
							max="10"
							step="0.5"
							value={suggestions.weights[term]}
							oninput={(e) => handleWeightChange(term, e)}
						/>
						<span class="weight-value font-pixel">{suggestions.weights[term].toFixed(1)}</span>
					</div>
				{/each}
			</div>

			<!-- Reset button -->
			<button class="reset-btn font-pixel" onclick={() => suggestions.resetToDefaults()}>
				Reset to Defaults
			</button>
		</div>
	{/if}
</div>

<style>
	.suggestion-panel {
		background: var(--color-bg-panel);
		border: 1px solid var(--color-border);
		border-radius: 2px;
		padding: 6px 8px;
	}

	.panel-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 8px;
	}

	.toggle-btn {
		display: flex;
		align-items: center;
		gap: 6px;
		background: none;
		border: none;
		color: var(--color-text-primary);
		font-size: var(--font-size-xs);
		cursor: pointer;
		padding: 2px 4px;
	}

	.toggle-btn:hover {
		color: var(--color-accent-cyan);
	}

	.toggle-indicator {
		display: inline-block;
		width: 8px;
		height: 8px;
		border-radius: 1px;
		background: var(--color-text-dim);
		transition: background 0.15s;
	}

	.toggle-indicator.active {
		background: #00e436;
		box-shadow: 0 0 4px #00e43688;
	}

	.expand-btn {
		background: none;
		border: none;
		color: var(--color-text-secondary);
		font-size: var(--font-size-xs);
		cursor: pointer;
		padding: 2px 4px;
	}

	.expand-btn:hover {
		color: var(--color-accent-cyan);
	}

	.weights-container {
		margin-top: 8px;
		display: flex;
		flex-direction: column;
		gap: 8px;
	}

	.preset-row {
		display: flex;
		gap: 4px;
		flex-wrap: wrap;
	}

	.preset-btn {
		background: var(--color-widget-bg);
		border: 1px solid var(--color-border);
		color: var(--color-text-secondary);
		font-size: 7px;
		padding: 2px 6px;
		cursor: pointer;
		border-radius: 1px;
	}

	.preset-btn:hover {
		border-color: var(--color-accent-cyan);
		color: var(--color-accent-cyan);
	}

	.weight-group {
		display: flex;
		flex-direction: column;
		gap: 3px;
	}

	.group-label {
		color: var(--color-accent-magenta);
		font-size: 7px;
		text-transform: uppercase;
		letter-spacing: 1px;
		padding-bottom: 2px;
		border-bottom: 1px solid var(--color-border);
		margin-bottom: 2px;
	}

	.weight-row {
		display: grid;
		grid-template-columns: 80px 1fr 30px;
		align-items: center;
		gap: 6px;
	}

	.weight-label {
		color: var(--color-text-secondary);
		font-size: 7px;
		white-space: nowrap;
	}

	.weight-slider {
		-webkit-appearance: none;
		appearance: none;
		width: 100%;
		height: 4px;
		background: var(--color-widget-bg);
		border-radius: 0;
		outline: none;
	}

	.weight-slider::-webkit-slider-thumb {
		-webkit-appearance: none;
		appearance: none;
		width: 8px;
		height: 12px;
		background: var(--color-accent-cyan);
		border: none;
		cursor: pointer;
	}

	.weight-slider::-moz-range-thumb {
		width: 8px;
		height: 12px;
		background: var(--color-accent-cyan);
		border: none;
		cursor: pointer;
	}

	.weight-value {
		color: var(--color-text-dim);
		font-size: 7px;
		text-align: right;
	}

	.reset-btn {
		background: var(--color-widget-bg);
		border: 1px solid var(--color-border);
		color: var(--color-text-secondary);
		font-size: 7px;
		padding: 3px 8px;
		cursor: pointer;
		border-radius: 1px;
		align-self: flex-start;
	}

	.reset-btn:hover {
		border-color: var(--color-accent-magenta);
		color: var(--color-accent-magenta);
	}
</style>
