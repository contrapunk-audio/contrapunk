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

	// -- Delete confirmation --
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
			saveName = '';
			await loadPresets();
			activePreset = saveName.trim();
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
	<div class="section-header font-pixel">PRESETS</div>

	{#if isLoading}
		<div class="loading-text font-pixel">Loading...</div>
	{:else if error}
		<div class="error-text font-pixel">{error}</div>
	{/if}

	<!-- Preset list -->
	<div class="preset-list">
		{#each presets as preset}
			<button
				class="preset-item font-pixel"
				class:preset-active={activePreset === preset.name}
				onclick={() => handleLoadPreset(preset.name)}
				title="{preset.persona} - {preset.genre}"
			>
				<span class="preset-name">{preset.name}</span>
				<span class="preset-meta">
					{#if preset.isBuiltin}
						<span class="builtin-badge">BUILT-IN</span>
					{:else}
						<button
							class="delete-btn font-pixel"
							onclick={(e: MouseEvent) => {
								e.stopPropagation();
								if (confirmDelete === preset.name) {
									handleDelete(preset.name);
								} else {
									confirmDelete = preset.name;
								}
							}}
							title={confirmDelete === preset.name ? 'Click again to confirm' : 'Delete preset'}
						>
							{confirmDelete === preset.name ? '?' : 'X'}
						</button>
					{/if}
				</span>
			</button>
		{/each}
	</div>

	<!-- Save controls -->
	<div class="save-area">
		{#if showSaveForm}
			<div class="save-form">
				<input
					type="text"
					class="save-input font-pixel"
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
		font-size: 6px;
		margin-bottom: 3px;
		-webkit-font-smoothing: none;
		text-rendering: optimizeSpeed;
	}

	.error-text {
		color: #ff4466;
		font-size: 6px;
		margin-bottom: 3px;
		-webkit-font-smoothing: none;
		text-rendering: optimizeSpeed;
	}

	.preset-list {
		display: flex;
		flex-direction: column;
		gap: 1px;
		margin-bottom: 4px;
		max-height: 200px;
		overflow-y: auto;
	}

	.preset-list::-webkit-scrollbar {
		width: 3px;
	}

	.preset-list::-webkit-scrollbar-track {
		background: var(--color-bg-deep);
	}

	.preset-list::-webkit-scrollbar-thumb {
		background: var(--color-widget-inactive);
		border-radius: 0;
	}

	.preset-item {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 3px 4px;
		background: var(--color-bg-panel);
		border: 1px solid var(--color-border);
		border-radius: 0;
		cursor: pointer;
		color: var(--color-text-primary);
		font-size: 7px;
		text-align: left;
		transition: none;
		-webkit-font-smoothing: none;
		text-rendering: optimizeSpeed;
		width: 100%;
	}

	.preset-item:hover {
		border-color: var(--color-accent-cyan-dim);
		background: var(--color-widget-bg);
	}

	.preset-item.preset-active {
		border-color: var(--color-accent-magenta);
		box-shadow: 0 0 6px var(--color-accent-magenta-dim);
		background: var(--color-widget-bg);
	}

	.preset-name {
		flex: 1;
		min-width: 0;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.preset-meta {
		display: flex;
		align-items: center;
		flex-shrink: 0;
		margin-left: 4px;
	}

	.builtin-badge {
		font-size: 5px;
		color: var(--color-text-dim);
		white-space: nowrap;
	}

	.delete-btn {
		background: none;
		border: 1px solid var(--color-border);
		color: var(--color-text-dim);
		font-size: 6px;
		padding: 1px 3px;
		cursor: pointer;
		border-radius: 0;
		-webkit-font-smoothing: none;
		text-rendering: optimizeSpeed;
		transition: none;
	}

	.delete-btn:hover {
		color: #ff4466;
		border-color: #ff4466;
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
		font-size: 7px;
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
		font-size: 6px !important;
		min-width: 20px;
	}

	.save-btn {
		width: 100%;
		font-size: 7px !important;
		padding: 3px 6px !important;
	}
</style>
