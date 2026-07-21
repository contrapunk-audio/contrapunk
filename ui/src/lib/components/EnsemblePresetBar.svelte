<script lang="ts">
	import { onMount } from 'svelte';
	import PixelSelect from './PixelSelect.svelte';
	import {
		arrangementConfigCapabilities,
		missingArrangementCapabilities,
		validateArrangementPreset,
		type ArrangementPresetV2
	} from '$lib/arrangement/presets';
	import { arrangement } from '$lib/stores/arrangement.svelte';
	import { arrangementPresets } from '$lib/stores/arrangementPresets.svelte';

	let { compact = false }: { compact?: boolean } = $props();
	let applying = $state(false);
	let saveOpen = $state(false);
	let saveName = $state('');
	let saveResult = $state('');
	let savePrompt = $state('');
	let saveReferences = $state('');
	let error = $state('');

	let presets = $derived(arrangementPresets.allPresets);
	let selected = $derived(presets.find((preset) => preset.id === arrangementPresets.selectedId));
	let missing = $derived(
		selected
			? missingArrangementCapabilities(selected, arrangement.availableCapabilities)
			: []
	);
	let researchLocked = $derived(
		selected?.builtIn === true && selected.researchStatus !== 'approved'
	);
	let presetErrors = $derived(selected ? validateArrangementPreset(selected) : []);
	let canApply = $derived(
		!!selected && !researchLocked && missing.length === 0 && presetErrors.length === 0 && !applying
	);
	let options = $derived(
		presets.map((preset) => ({
			value: preset.id,
			label: `${preset.builtIn ? (preset.researchStatus === 'approved' ? '' : '◇ ') : '★ '}${preset.name}`
		}))
	);
	let availability = $derived(
		researchLocked
			? 'Research pending — Apply stays locked.'
			: missing.length
				? `Unavailable here — missing ${missing.join(', ')}.`
				: presetErrors.length
					? `Invalid preset — ${presetErrors.join('; ')}.`
					: 'Available on this surface.'
	);

	onMount(() => {
		void arrangement.syncFromBackend();
	});

	async function applySelected() {
		if (!selected || !canApply) return;
		applying = true;
		error = '';
		try {
			await arrangement.apply(selected.config);
			arrangementPresets.appliedId = selected.id;
		} catch (cause) {
			error = `Apply failed: ${cause}`;
		} finally {
			applying = false;
		}
	}

	function openSaveAs() {
		const source = selected && (!selected.builtIn || selected.researchStatus === 'approved')
			? selected
			: undefined;
		saveName = source ? `${source.name} Copy` : 'Custom Arrangement';
		saveResult = source?.result ?? 'A custom arrangement captured from the current controls.';
		savePrompt = source?.play.prompt ?? 'Play naturally and leave space to hear the arrangement.';
		saveReferences = source?.references.map((reference) => reference.name).join(', ') ?? '';
		saveOpen = true;
	}

	function saveCurrent() {
		const name = saveName.trim();
		const result = saveResult.trim();
		const prompt = savePrompt.trim();
		if (!name || !result || !prompt) return;

		const source = selected && (!selected.builtIn || selected.researchStatus === 'approved')
			? selected
			: undefined;
		const config = arrangement.snapshot();
		const id = arrangementPresets.create({
			name,
			family: 'custom',
			tags: source ? [...source.tags, 'custom'] : ['custom'],
			result,
			approximation: source?.approximation,
			play: {
				...(source?.play ?? defaultPlayGuide()),
				prompt
			},
			references: saveReferences
				.split(',')
				.map((name) => name.trim())
				.filter(Boolean)
				.map((name) => ({ name, context: 'User-authored reference.' })),
			researchStatus: 'not_required',
			requirements: arrangementConfigCapabilities(config),
			suggestedSoundPresetId: source?.suggestedSoundPresetId,
			config
		});
		arrangementPresets.selectedId = id;
		arrangementPresets.appliedId = id;
		saveOpen = false;
		error = '';
	}

	function defaultPlayGuide(): ArrangementPresetV2['play'] {
		return {
			prompt: '',
			input: 'single_notes',
			articulation: 'As configured.',
			density: 'As configured.',
			space: 'Leave room for generated parts.',
			transportRequired: false
		};
	}
</script>

<section
	class="preset-bar"
	class:compact
	aria-label="Arrangement preset"
	title={compact && selected ? `${availability}\nResult: ${selected.result}\nPlay it like: ${selected.play.prompt}` : undefined}
>
	<div class="toolbar">
		<div class="preset-label font-ui">{compact ? 'PRESET' : 'ARRANGEMENT'}</div>
		<PixelSelect
			options={options}
			value={arrangementPresets.selectedId}
			label="Arrangement preset"
			help="Selects a fully implemented arrangement. Nothing changes until Apply. Your custom presets are also shown."
			onchange={(value) => {
				arrangementPresets.selectedId = value;
				error = '';
			}}
		/>
		<button class="apply font-ui" type="button" disabled={!canApply} onclick={applySelected}>
			{applying ? 'APPLYING…' : arrangementPresets.appliedId === arrangementPresets.selectedId ? 'APPLIED' : 'APPLY'}
		</button>
		<button class="save font-ui" type="button" onclick={openSaveAs}>SAVE AS…</button>
	</div>

	{#if selected}
		<div class="details">
			<div class="detail-copy">
				<div class="eyebrow font-code">
					{selected.family.toUpperCase()} · {selected.builtIn ? 'BUILT-IN' : 'CUSTOM'} · {selected.researchStatus.replaceAll('_', ' ').toUpperCase()}
				</div>
				<p><strong>RESULT</strong> {selected.result}</p>
				<p><strong>PLAY IT LIKE</strong> {selected.play.prompt}</p>
				{#if selected.approximation}<p class="approximation">{selected.approximation}</p>{/if}
			</div>
			<div class="evidence font-code">
				<div class:locked={researchLocked || missing.length > 0 || presetErrors.length > 0} class="availability">{availability}</div>
				<div>{selected.play.input.replaceAll('_', ' ')} · {selected.play.transportRequired ? 'transport required' : 'transport optional'}</div>
				<div>{selected.play.articulation}</div>
				<div>{selected.play.density} {selected.play.space}</div>
				{#if selected.play.tempo}<div>{selected.play.tempo}</div>{/if}
				<div>Reference: {selected.references.map((reference) => reference.name).join(', ')}</div>
			</div>
		</div>
	{/if}

	{#if saveOpen}
		<div class="save-editor">
			<label class="font-code">NAME<input bind:value={saveName} /></label>
			<label class="font-code">RESULT<input bind:value={saveResult} /></label>
			<label class="font-code wide">REFERENCES<input bind:value={saveReferences} placeholder="Optional, comma separated" /></label>
			<label class="font-code wide">PLAY IT LIKE<textarea rows="2" bind:value={savePrompt}></textarea></label>
			<div class="save-actions">
				<button class="font-ui" type="button" onclick={() => (saveOpen = false)}>CANCEL</button>
				<button class="font-ui" type="button" disabled={!saveName.trim() || !saveResult.trim() || !savePrompt.trim()} onclick={saveCurrent}>SAVE COPY</button>
			</div>
		</div>
	{/if}

	{#if error}<div class="error font-code">{error}</div>{/if}
</section>

<style>
	.preset-bar {
		order: 1;
		display: grid;
		gap: 7px;
		padding: 7px 8px;
		border: 1px solid rgba(51, 221, 255, 0.48);
		background: linear-gradient(90deg, rgba(16, 34, 46, 0.96), rgba(18, 16, 32, 0.96));
	}
	.toolbar {
		display: grid;
		grid-template-columns: 104px minmax(0, 1fr) auto auto;
		align-items: center;
		gap: 7px;
	}
	.preset-label { color: var(--color-accent-cyan); font-size: 9px; letter-spacing: 1px; }
	.preset-bar.compact { order: 0; flex: 1; min-width: 0; gap: 0; padding: 0; border: 0; background: transparent; }
	.preset-bar.compact .toolbar { grid-template-columns: 52px minmax(180px, 1fr) auto; }
	.preset-bar.compact .details,
	.preset-bar.compact .save,
	.preset-bar.compact .save-editor,
	.preset-bar.compact .error { display: none; }
	button { min-height: 27px; padding: 0 9px; border-radius: 0; cursor: pointer; }
	.apply { border: 1px solid var(--color-accent-cyan); background: rgba(18, 49, 64, 0.86); color: var(--color-accent-cyan); }
	.save { border: 1px solid var(--color-border); background: var(--color-widget-bg); color: var(--color-text-secondary); }
	button:disabled { opacity: 0.45; cursor: not-allowed; }
	.details {
		display: grid;
		grid-template-columns: minmax(0, 1.35fr) minmax(240px, 0.65fr);
		gap: 12px;
		padding: 7px;
		border-top: 1px solid rgba(255, 255, 255, 0.08);
	}
	.eyebrow { color: var(--color-text-tertiary); font-size: 8px; letter-spacing: 0.08em; }
	.detail-copy p { margin: 4px 0; color: var(--color-text-secondary); font-size: 10px; line-height: 1.35; }
	.detail-copy strong { margin-right: 5px; color: var(--color-text-primary); font-size: 8px; letter-spacing: 0.08em; }
	.approximation { padding-left: 7px; border-left: 2px solid rgba(255, 255, 255, 0.16); color: var(--color-text-tertiary) !important; }
	.evidence { display: grid; align-content: start; gap: 3px; color: var(--color-text-tertiary); font-size: 8px; line-height: 1.3; }
	.availability { color: var(--color-accent-lime); }
	.availability.locked { color: #ffbf69; }
	.save-editor {
		display: grid;
		grid-template-columns: minmax(140px, 0.45fr) minmax(200px, 1fr) auto;
		gap: 7px;
		padding-top: 7px;
		border-top: 1px solid rgba(255, 255, 255, 0.08);
	}
	.save-editor label { display: grid; gap: 3px; color: var(--color-text-tertiary); font-size: 8px; }
	.save-editor .wide { grid-column: 1 / 3; }
	.save-editor input,
	.save-editor textarea {
		min-width: 0;
		padding: 5px 7px;
		border: 1px solid var(--color-border);
		background: var(--color-bg-deep);
		color: var(--color-text-primary);
		font: inherit;
		resize: vertical;
	}
	.save-actions { grid-column: 3; grid-row: 1 / 4; display: grid; align-content: end; gap: 5px; }
	.save-actions button { border: 1px solid var(--color-accent-cyan); background: rgba(18, 49, 64, 0.86); color: var(--color-accent-cyan); }
	.error { color: #ff6b81; font-size: 8px; }
	@media (max-width: 720px) {
		.toolbar { grid-template-columns: 78px minmax(0, 1fr) auto; }
		.save { display: none; }
		.details { grid-template-columns: 1fr; }
	}
</style>
