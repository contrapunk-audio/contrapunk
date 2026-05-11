<script lang="ts">
	import { adapter } from '$lib/adapter';
	import type { Preset } from '$lib/adapter';
	import { engine } from '$lib/stores/engine.svelte';

	// -- State --
	let presets = $state<Preset[]>([]);
	let activePreset = $state<string | null>(null);
	let isLoading = $state(false);
	let error = $state<string | null>(null);

	// -- Save form state --
	let showSaveForm = $state(false);
	let saveName = $state('');
	let isSaving = $state(false);

	// -- Delete confirmation -- (name of the preset whose × was first clicked)
	let confirmDelete = $state<string | null>(null);

	/**
	 * Load the list of presets from the adapter.
	 */
	async function loadPresets() {
		isLoading = true;
		error = null;
		try {
			presets = await adapter.listPresets();
		} catch (e) {
			error = `Failed to load presets: ${e}`;
		} finally {
			isLoading = false;
		}
	}

	/**
	 * Load a preset by name and sync engine state from backend.
	 */
	async function handleLoadPreset(name: string) {
		try {
			await adapter.loadPreset(name);
			await engine.syncFromBackend();
			activePreset = name;
			// A successful load clears any pending delete-confirm on a
			// different pill so a second click on a × elsewhere still
			// requires its own first-click confirm step.
			confirmDelete = null;
		} catch (e) {
			error = `Failed to load preset "${name}": ${e}`;
		}
	}

	/**
	 * Save current configuration as a new custom preset.
	 */
	async function handleSave() {
		if (!saveName.trim()) return;
		isSaving = true;
		error = null;
		try {
			await adapter.savePreset(saveName.trim());
			showSaveForm = false;
			const newName = saveName.trim();
			saveName = '';
			await loadPresets();
			activePreset = newName;
		} catch (e) {
			error = `Failed to save preset: ${e}`;
		} finally {
			isSaving = false;
		}
	}

	/**
	 * Delete a custom preset.
	 */
	async function handleDelete(name: string) {
		try {
			await adapter.deletePreset(name);
			confirmDelete = null;
			if (activePreset === name) {
				activePreset = null;
			}
			await loadPresets();
		} catch (e) {
			error = `Failed to delete "${name}": ${e}`;
		}
	}

	function cancelSave() {
		showSaveForm = false;
		saveName = '';
	}

	function handleSaveKeydown(event: KeyboardEvent) {
		if (event.key === 'Enter') {
			handleSave();
		} else if (event.key === 'Escape') {
			cancelSave();
		}
	}

	// Load presets on mount
	$effect(() => {
		loadPresets();
	});
</script>

<div class="preset-section pixel-card">
	<div class="section-header font-ui">PRESETS</div>

	{#if isLoading}
		<div class="loading-text font-ui">Loading...</div>
	{:else if error}
		<div class="error-text font-ui">{error}</div>
	{/if}

	<!-- Horizontal pill row. One pill per preset; active pill gets a
	     neon-magenta glow. Built-in pills have no delete button; user
	     pills get an inline × that becomes a "?" confirm on first
	     click. Pill click loads; × click is stopPropagation'd so it
	     never doubles as a load. -->
	<div class="preset-row" role="list">
		{#each presets as preset}
			{@const isActive = activePreset === preset.name}
			{@const isConfirming = confirmDelete === preset.name}
			<div
				class="preset-pill font-ui"
				class:active={isActive}
				class:builtin={preset.isBuiltin}
				class:confirming={isConfirming}
				role="listitem"
			>
				<button
					type="button"
					class="pill-load font-ui"
					onclick={() => handleLoadPreset(preset.name)}
					title="{preset.persona} - {preset.genre}"
				>
					<span class="pill-name">{preset.name}</span>
				</button>
				{#if !preset.isBuiltin}
					<button
						type="button"
						class="pill-delete font-ui"
						class:confirming={isConfirming}
						onclick={(e: MouseEvent) => {
							e.stopPropagation();
							if (isConfirming) {
								handleDelete(preset.name);
							} else {
								confirmDelete = preset.name;
							}
						}}
						title={isConfirming ? 'Click again to confirm' : 'Delete preset'}
						aria-label={isConfirming ? `Confirm delete ${preset.name}` : `Delete ${preset.name}`}
					>
						{isConfirming ? '?' : '×'}
					</button>
				{/if}
			</div>
		{/each}
	</div>

	<!-- Save controls -->
	<div class="save-area">
		{#if showSaveForm}
			<div class="save-form">
				<input
					type="text"
					class="save-input font-ui"
					bind:value={saveName}
					placeholder="Preset name..."
					onkeydown={handleSaveKeydown}
					disabled={isSaving}
				/>
				<button
					class="pixel-btn save-confirm-btn"
					onclick={handleSave}
					disabled={isSaving || !saveName.trim()}
				>
					{isSaving ? '...' : 'OK'}
				</button>
				<button
					class="pixel-btn save-cancel-btn"
					onclick={cancelSave}
				>
					X
				</button>
			</div>
		{:else}
			<button
				class="pixel-btn save-btn"
				onclick={() => { showSaveForm = true; }}
			>
				Save Preset
			</button>
		{/if}
	</div>
</div>

<style>
	.preset-section {
		padding: 6px;
		margin-bottom: 4px;
	}

	.section-header {
		color: var(--color-accent-gold);
		font-size: var(--font-size-xs);
		margin-bottom: 4px;
		-webkit-font-smoothing: none;
		text-rendering: optimizeSpeed;
	}

	.loading-text {
		color: var(--color-text-dim);
		font-size: var(--font-size-xs);
		margin-bottom: 3px;
		-webkit-font-smoothing: none;
		text-rendering: optimizeSpeed;
	}

	.error-text {
		color: #ff4466;
		font-size: var(--font-size-xs);
		margin-bottom: 3px;
		-webkit-font-smoothing: none;
		text-rendering: optimizeSpeed;
	}

	/* Horizontal pill row. Wraps when there are more presets than fit
	   on one line; horizontal scrolling would clip preset names and
	   hide the active pill on narrow viewports. */
	.preset-row {
		display: flex;
		flex-direction: row;
		flex-wrap: wrap;
		gap: 4px;
		margin-bottom: 6px;
	}

	.preset-pill {
		display: inline-flex;
		align-items: stretch;
		background: var(--color-bg-panel);
		border: 1px solid var(--color-border);
		border-radius: 0;
		color: var(--color-text-primary);
		font-size: var(--font-size-xs);
		overflow: hidden;
		max-width: 100%;
		transition: none;
		-webkit-font-smoothing: none;
		text-rendering: optimizeSpeed;
	}

	.preset-pill:hover {
		border-color: var(--color-accent-cyan-dim);
	}

	/* Active pill — magenta neon glow. */
	.preset-pill.active {
		border-color: var(--color-accent-magenta);
		box-shadow: 0 0 6px var(--color-accent-magenta-dim), 0 0 12px var(--color-accent-magenta-dim);
		background: var(--color-widget-bg);
	}

	.preset-pill.active .pill-name {
		color: var(--color-accent-cyan);
		text-shadow: 0 0 4px var(--color-accent-cyan);
	}

	/* Built-in pills get a faint cyan tint on the left so users can
	   spot which presets they can't delete. The cyan accent matches
	   the secondary-accent color in the theme. */
	.preset-pill.builtin {
		border-left: 2px solid var(--color-accent-cyan-dim);
	}

	.pill-load {
		background: none;
		border: none;
		color: inherit;
		font: inherit;
		padding: 3px 6px;
		cursor: pointer;
		text-align: left;
		min-width: 0;
		max-width: 160px;
		-webkit-font-smoothing: none;
		text-rendering: optimizeSpeed;
	}

	.pill-name {
		display: inline-block;
		max-width: 100%;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.pill-delete {
		background: none;
		border: none;
		border-left: 1px solid var(--color-border);
		color: var(--color-text-dim);
		font-size: var(--font-size-xs);
		padding: 0 5px;
		cursor: pointer;
		-webkit-font-smoothing: none;
		text-rendering: optimizeSpeed;
		transition: none;
	}

	.pill-delete:hover {
		color: #ff4466;
		background: rgba(255, 68, 102, 0.08);
	}

	/* First click — switch to "?" with magenta glow as the confirm prompt. */
	.pill-delete.confirming {
		color: var(--color-accent-magenta);
		background: rgba(255, 51, 136, 0.12);
		text-shadow: 0 0 4px var(--color-accent-magenta);
	}

	.save-area {
		margin-top: 4px;
	}

	.save-form {
		display: flex;
		gap: 2px;
		align-items: center;
	}

	.save-input {
		flex: 1;
		background: var(--color-widget-bg);
		border: 1px solid var(--color-border);
		color: var(--color-text-primary);
		font-size: var(--font-size-xs);
		padding: 3px 4px;
		border-radius: 0;
		outline: none;
		-webkit-font-smoothing: none;
		text-rendering: optimizeSpeed;
	}

	.save-input:focus {
		border-color: var(--color-accent-cyan);
	}

	.save-input::placeholder {
		color: var(--color-text-dim);
	}

	.save-confirm-btn,
	.save-cancel-btn {
		padding: 3px 6px !important;
		font-size: var(--font-size-xs) !important;
		min-width: 20px;
	}

	.save-btn {
		width: 100%;
		font-size: var(--font-size-xs) !important;
		padding: 3px 6px !important;
	}
</style>
