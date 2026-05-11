<script lang="ts">
	import { voiceLibrary, type VoicePreset } from '$lib/stores/voiceLibrary.svelte';
	import {
		ALL_MODES,
		OCTAVE_MODES,
		SCALE_FAMILIES,
		VOICE_LEADING_STYLES
	} from '$lib/stores/engine.svelte';
	import PixelSelect from './PixelSelect.svelte';

	let selectedId = $state<string | null>(voiceLibrary.all[0]?.id ?? null);
	let creating = $state(false);
	let newName = $state('');

	let selected = $derived(
		selectedId ? voiceLibrary.byId(selectedId) ?? null : null
	);

	// Cloneable "scratch" preset shown in the editor — never mutates a
	// built-in. For user presets the patch flows back through update().
	let editorState = $state<Omit<VoicePreset, 'id' | 'builtIn'>>(emptyEditor());

	function emptyEditor(): Omit<VoicePreset, 'id' | 'builtIn'> {
		return {
			name: 'New Voice',
			harmony_mode: 'StrictCounterpoint',
			scale_mode: 'Ionian',
			voice_leading_enabled: true,
			voice_leading_style: 'Palestrina',
			octave_mode: 'Spread',
			counterpoint_species: 'Species1',
			counterpoint_strictness: 'Strict',
			voice_count: 4,
			voice_position: 3
		};
	}

	// Sync editorState whenever selection changes.
	$effect(() => {
		if (selected) {
			editorState = {
				name: selected.name,
				harmony_mode: selected.harmony_mode,
				scale_mode: selected.scale_mode,
				voice_leading_enabled: selected.voice_leading_enabled,
				voice_leading_style: selected.voice_leading_style,
				octave_mode: selected.octave_mode,
				counterpoint_species: selected.counterpoint_species,
				counterpoint_strictness: selected.counterpoint_strictness,
				voice_count: selected.voice_count,
				voice_position: selected.voice_position
			};
		}
	});

	function applyEditorToSelected() {
		if (!selected || selected.builtIn) return;
		voiceLibrary.update(selected.id, editorState);
	}

	function createPreset() {
		const name = newName.trim();
		if (!name) return;
		const id = voiceLibrary.create({ ...emptyEditor(), name });
		selectedId = id;
		creating = false;
		newName = '';
	}

	function deleteSelected() {
		if (!selected || selected.builtIn) return;
		const id = selected.id;
		const all = voiceLibrary.all;
		const idx = all.findIndex((p) => p.id === id);
		voiceLibrary.delete(id);
		// Move selection to a neighbouring preset.
		const remaining = voiceLibrary.all;
		selectedId = remaining[Math.min(idx, remaining.length - 1)]?.id ?? null;
	}

	function duplicateSelected() {
		if (!selected) return;
		const copy = {
			name: `${selected.name} (copy)`,
			harmony_mode: selected.harmony_mode,
			scale_mode: selected.scale_mode,
			voice_leading_enabled: selected.voice_leading_enabled,
			voice_leading_style: selected.voice_leading_style,
			octave_mode: selected.octave_mode,
			counterpoint_species: selected.counterpoint_species,
			counterpoint_strictness: selected.counterpoint_strictness,
			voice_count: selected.voice_count,
			voice_position: selected.voice_position
		};
		const id = voiceLibrary.create(copy);
		selectedId = id;
	}

	const HARMONY_MODE_OPTIONS = ALL_MODES.map((m) => ({
		value: m.name,
		label: m.label
	}));

	// Composite mode dropdown: Counterpoint expands into the 4 Fux
	// species in the same list so picking "Counterpoint · Species N"
	// sets both harmony_mode + counterpoint_species at once. No
	// separate Strictness control — Strict is always applied.
	const COMPOSITE_MODE_OPTIONS = ALL_MODES.flatMap((m) =>
		m.name === 'StrictCounterpoint'
			? [
					{ value: 'StrictCounterpoint:Species1', label: `${m.label} (1:1)` },
					{ value: 'StrictCounterpoint:Species2', label: `${m.label} (2:1 rules)` },
					{ value: 'StrictCounterpoint:Species3', label: `${m.label} (4:1 rules)` },
					{ value: 'StrictCounterpoint:Species4', label: `${m.label} (syncopated)` }
				]
			: [{ value: m.name, label: m.label }]
	);

	function encodeMode(harmony_mode: string, species: string): string {
		if (harmony_mode === 'StrictCounterpoint') {
			return `StrictCounterpoint:${species || 'Species1'}`;
		}
		return harmony_mode;
	}
	function decodeMode(v: string): { harmony_mode: string; counterpoint_species: string | null } {
		if (v.startsWith('StrictCounterpoint:')) {
			return {
				harmony_mode: 'StrictCounterpoint',
				counterpoint_species: v.slice('StrictCounterpoint:'.length)
			};
		}
		return { harmony_mode: v, counterpoint_species: null };
	}
	const SCALE_MODE_OPTIONS = SCALE_FAMILIES.flatMap((f) =>
		f.modes.map((m) => ({ value: m.name, label: `${f.label} · ${m.label}` }))
	);
	const VL_STYLE_OPTIONS = VOICE_LEADING_STYLES.map((s) => ({
		value: s.name,
		label: s.label
	}));
	const OCTAVE_MODE_OPTIONS = OCTAVE_MODES.map((m) => ({
		value: m.name,
		label: m.label
	}));
</script>

<div class="voices-root">
	<header class="header">
		<div class="header-left">
			<h2 class="title font-ui">Voices</h2>
			<span class="subtitle font-code">
				named harmony presets · pick from the Companion's Relative + Mode picker
			</span>
		</div>
	</header>

	<div class="body">
		<aside class="rail">
			<div class="section-header font-ui">
				LIBRARY
				<button
					class="pixel-btn"
					onclick={() => {
						creating = !creating;
						newName = '';
					}}
					title="Create a new user preset"
				>
					+ New
				</button>
			</div>

			{#if creating}
				<div class="save-form font-code">
					<input
						type="text"
						bind:value={newName}
						placeholder="Preset name…"
						class="pixel-input"
						onkeydown={(e) => {
							if (e.key === 'Enter') createPreset();
							if (e.key === 'Escape') creating = false;
						}}
					/>
					<button class="pixel-btn" onclick={createPreset} disabled={!newName.trim()}>
						Create
					</button>
				</div>
			{/if}

			<div class="rail-section-label font-code">BUILT-IN</div>
			{#each voiceLibrary.builtIn as p (p.id)}
				<button
					class="preset-row"
					class:active={selectedId === p.id}
					onclick={() => (selectedId = p.id)}
					title={`${p.name} · ${p.harmony_mode} · ${p.voice_count} voices @ pos ${p.voice_position}`}
				>
					<div class="preset-name font-ui">{p.name}</div>
					<div class="preset-desc font-code">
						{p.harmony_mode} · {p.voice_count}v
					</div>
				</button>
			{/each}

			{#if voiceLibrary.userPresets.length > 0}
				<div class="rail-section-label font-code">USER</div>
				{#each voiceLibrary.userPresets as p (p.id)}
					<button
						class="preset-row user"
						class:active={selectedId === p.id}
						onclick={() => (selectedId = p.id)}
						title={`${p.name} · ${p.harmony_mode} · ${p.voice_count} voices @ pos ${p.voice_position}`}
					>
						<div class="preset-name font-ui">{p.name}</div>
						<div class="preset-desc font-code">
							{p.harmony_mode} · {p.voice_count}v
						</div>
					</button>
				{/each}
			{/if}
		</aside>

		<section class="editor">
			{#if selected}
				<div class="editor-header">
					<div class="editor-title font-ui">
						{selected.name}
						{#if selected.builtIn}<span class="builtin-badge font-code">BUILT-IN</span>{/if}
					</div>
					<div class="editor-actions">
						<button
							class="pixel-btn"
							onclick={duplicateSelected}
							title="Create a user preset starting from this one's config"
						>
							Duplicate
						</button>
						{#if !selected.builtIn}
							<button
								class="pixel-btn"
								onclick={deleteSelected}
								title="Delete this user preset (built-ins cannot be deleted)"
							>
								Delete
							</button>
						{/if}
					</div>
				</div>

				<div class="editor-grid">
					<label class="row">
						<span class="row-label font-ui">Name</span>
						<input
							type="text"
							class="pixel-input"
							bind:value={editorState.name}
							disabled={selected.builtIn}
							onblur={applyEditorToSelected}
						/>
					</label>

					<label class="row">
						<span class="row-label font-ui">Harmony Mode</span>
						<PixelSelect
							options={COMPOSITE_MODE_OPTIONS}
							value={encodeMode(editorState.harmony_mode, editorState.counterpoint_species)}
							small={true}
							onchange={(v) => {
								const dec = decodeMode(v);
								editorState.harmony_mode =
									dec.harmony_mode as typeof editorState.harmony_mode;
								if (dec.counterpoint_species) {
									editorState.counterpoint_species =
										dec.counterpoint_species as typeof editorState.counterpoint_species;
								}
								applyEditorToSelected();
							}}
						/>
					</label>

					<label class="row">
						<span class="row-label font-ui">Scale Mode</span>
						<PixelSelect
							options={SCALE_MODE_OPTIONS}
							value={editorState.scale_mode}
							small={true}
							onchange={(v) => {
								editorState.scale_mode = v as typeof editorState.scale_mode;
								applyEditorToSelected();
							}}
						/>
					</label>

					<label class="row">
						<span class="row-label font-ui">Voice-Leading</span>
						<div class="vl-pair">
							<input
								type="checkbox"
								bind:checked={editorState.voice_leading_enabled}
								disabled={selected.builtIn}
								onchange={applyEditorToSelected}
							/>
							<PixelSelect
								options={VL_STYLE_OPTIONS}
								value={editorState.voice_leading_style}
								small={true}
								onchange={(v) => {
									editorState.voice_leading_style = v as typeof editorState.voice_leading_style;
									applyEditorToSelected();
								}}
							/>
						</div>
					</label>

					<label class="row">
						<span class="row-label font-ui">Octave Mode</span>
						<PixelSelect
							options={OCTAVE_MODE_OPTIONS}
							value={editorState.octave_mode}
							small={true}
							onchange={(v) => {
								editorState.octave_mode = v as typeof editorState.octave_mode;
								applyEditorToSelected();
							}}
						/>
					</label>

					<label class="row">
						<span class="row-label font-ui">Voice Count</span>
						<input
							type="number"
							min="1"
							max="4"
							class="pixel-input num"
							bind:value={editorState.voice_count}
							disabled={selected.builtIn}
							onchange={applyEditorToSelected}
						/>
					</label>

					<label class="row">
						<span class="row-label font-ui">Voice Position</span>
						<input
							type="number"
							min="0"
							max={Math.max(0, editorState.voice_count - 1)}
							class="pixel-input num"
							bind:value={editorState.voice_position}
							disabled={selected.builtIn}
							onchange={applyEditorToSelected}
						/>
					</label>
				</div>

				<div class="footer-hint font-code">
					{#if selected.builtIn}
						Built-in presets are read-only — duplicate one to edit.
					{:else}
						Changes save as you edit. Pick this preset from the Companion's voice cards to apply.
					{/if}
				</div>
			{:else}
				<div class="empty font-code">No presets yet. Click + New to create one.</div>
			{/if}
		</section>
	</div>
</div>

<style>
	.voices-root {
		display: flex;
		flex-direction: column;
		height: 100%;
		overflow: hidden;
	}

	.header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: 8px 12px;
		border-bottom: 1px solid var(--color-border);
		background: var(--color-widget-bg);
	}

	.header-left {
		display: flex;
		align-items: baseline;
		gap: 8px;
	}

	.title {
		margin: 0;
		font-size: var(--font-size-lg);
		color: var(--color-accent-cyan, #33ddff);
	}

	.subtitle {
		font-size: var(--font-size-xs);
		opacity: 0.6;
	}

	.body {
		display: grid;
		grid-template-columns: 240px 1fr;
		flex: 1;
		min-height: 0;
		overflow: hidden;
	}

	.rail {
		display: flex;
		flex-direction: column;
		gap: 2px;
		padding: 8px;
		border-right: 1px solid var(--color-border);
		overflow-y: auto;
		background: rgba(15, 14, 26, 0.5);
	}

	.section-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		font-size: var(--font-size-xs);
		color: var(--color-text-secondary);
		text-transform: uppercase;
		letter-spacing: 0.1em;
		margin-bottom: 6px;
	}

	.rail-section-label {
		font-size: 0.65em;
		color: var(--color-text-secondary);
		letter-spacing: 0.2em;
		opacity: 0.5;
		margin: 8px 0 2px;
	}

	.preset-row {
		display: flex;
		flex-direction: column;
		align-items: flex-start;
		gap: 1px;
		padding: 6px 8px;
		background: var(--color-widget-bg);
		border: 1px solid var(--color-border);
		text-align: left;
		cursor: pointer;
		color: inherit;
	}

	.preset-row:hover {
		border-color: var(--color-accent-cyan, #33ddff);
	}

	.preset-row.active {
		border-color: var(--color-accent-magenta, #ff33aa);
		background: rgba(255, 51, 170, 0.1);
	}

	.preset-name {
		font-size: var(--font-size-sm);
		color: var(--color-accent-cyan, #33ddff);
	}

	.preset-row.user .preset-name {
		color: #ffcc44;
	}

	.preset-desc {
		font-size: var(--font-size-xs);
		opacity: 0.6;
	}

	.save-form {
		display: grid;
		grid-template-columns: 1fr auto;
		gap: 4px;
		padding: 4px 0 6px;
		border-bottom: 1px dashed rgba(255, 255, 255, 0.1);
		margin-bottom: 4px;
	}

	.editor {
		display: flex;
		flex-direction: column;
		gap: 8px;
		padding: 12px;
		overflow-y: auto;
	}

	.editor-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding-bottom: 6px;
		border-bottom: 1px solid var(--color-border);
	}

	.editor-title {
		font-size: var(--font-size-lg);
		color: var(--color-accent-cyan, #33ddff);
		display: flex;
		align-items: baseline;
		gap: 8px;
	}

	.builtin-badge {
		font-size: 0.55em;
		color: var(--color-text-secondary);
		letter-spacing: 0.1em;
		opacity: 0.7;
	}

	.editor-actions {
		display: flex;
		gap: 6px;
	}

	.editor-grid {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 6px 16px;
		padding-top: 8px;
	}

	.row {
		display: grid;
		grid-template-columns: 7em 1fr;
		align-items: center;
		gap: 6px;
	}

	.row-label {
		color: var(--color-text-secondary);
		font-size: var(--font-size-xs);
	}

	.pixel-input {
		background: var(--color-widget-bg);
		border: 1px solid var(--color-border);
		color: var(--color-text);
		padding: 4px 6px;
		font-size: var(--font-size-xs);
		font-family: inherit;
	}
	.pixel-input:focus {
		outline: none;
		border-color: var(--color-accent-cyan, #33ddff);
	}
	.pixel-input.num {
		width: 4em;
	}

	.vl-pair {
		display: grid;
		grid-template-columns: auto 1fr;
		gap: 6px;
		align-items: center;
	}

	.footer-hint {
		font-size: var(--font-size-xs);
		opacity: 0.55;
		padding: 6px 0;
		line-height: 1.4;
	}

	.empty {
		opacity: 0.5;
		padding: 12px;
	}
</style>
