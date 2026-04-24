<script lang="ts">
	import { chainStore } from '$lib/stores/chain.svelte';
	import type { ClapPluginDescriptor } from '$lib/adapter/types';

	let { open = $bindable(false) }: { open?: boolean } = $props();

	let search = $state('');
	let error = $state('');

	$effect(() => {
		if (open && chainStore.clapPlugins.length === 0 && !chainStore.loadingPlugins) {
			chainStore.scanPlugins();
		}
	});

	const filtered = $derived(
		search.trim()
			? chainStore.clapPlugins.filter((p) =>
				p.name.toLowerCase().includes(search.toLowerCase())
			)
			: chainStore.clapPlugins
	);

	async function addPlugin(p: ClapPluginDescriptor) {
		error = '';
		try {
			await chainStore.addClapPlugin(p.path);
			open = false;
		} catch (e) {
			error = String(e);
		}
	}
</script>

{#if open}
	<div
		class="backdrop"
		role="button"
		tabindex="0"
		onclick={() => (open = false)}
		onkeydown={(e) => {
			if (e.key === 'Escape') open = false;
		}}
	></div>
	<div class="modal" role="dialog" aria-label="CLAP plugin picker">
		<div class="modal-header font-pixel">
			<span>Add CLAP plugin</span>
			<button class="pixel-btn font-pixel" onclick={() => (open = false)}>X</button>
		</div>
		<div class="modal-body">
			<input
				class="search font-pixel"
				type="text"
				placeholder="Search…"
				bind:value={search}
				autofocus
			/>
			{#if chainStore.loadingPlugins}
				<div class="status font-pixel">Scanning…</div>
			{:else if chainStore.clapPlugins.length === 0}
				<div class="status font-pixel">
					No .clap plugins found on disk.<br />
					Install a CLAP plugin, then press Rescan.
				</div>
			{:else}
				<div class="plugin-list">
					{#each filtered as p (p.path)}
						<button class="plugin-row font-pixel" onclick={() => addPlugin(p)}>
							<div class="plugin-name">{p.name}</div>
							<div class="plugin-meta">
								{p.vendor || '—'}
								{#if p.version}
									· {p.version}
								{/if}
							</div>
							<div class="plugin-path">{p.path}</div>
						</button>
					{/each}
				</div>
			{/if}
			{#if error}
				<div class="error font-pixel">{error}</div>
			{/if}
		</div>
		<div class="modal-footer font-pixel">
			<button
				class="pixel-btn"
				disabled={chainStore.loadingPlugins}
				onclick={() => chainStore.scanPlugins()}
			>
				Rescan
			</button>
			<span class="hint">
				{chainStore.clapPlugins.length} plugin{chainStore.clapPlugins.length === 1 ? '' : 's'} found
			</span>
		</div>
	</div>
{/if}

<style>
	.backdrop {
		position: fixed;
		inset: 0;
		background: rgba(0, 0, 0, 0.55);
		z-index: 100;
		cursor: pointer;
		border: 0;
		padding: 0;
	}
	.modal {
		position: fixed;
		top: 50%;
		left: 50%;
		transform: translate(-50%, -50%);
		width: min(640px, 92vw);
		max-height: 80vh;
		display: flex;
		flex-direction: column;
		background: linear-gradient(180deg, #12101f, #0a0918);
		border: 1px solid var(--color-accent-magenta-dim);
		box-shadow: 0 0 30px rgba(255, 51, 136, 0.2);
		z-index: 101;
	}
	.modal-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: 8px 12px;
		border-bottom: 1px solid var(--color-border);
		color: var(--color-accent-magenta);
		font-size: var(--font-size-sm);
		letter-spacing: 2px;
		text-transform: uppercase;
	}
	.modal-body {
		padding: 10px 12px;
		flex: 1;
		display: flex;
		flex-direction: column;
		gap: 8px;
		overflow: hidden;
	}
	.modal-footer {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: 8px 12px;
		border-top: 1px solid var(--color-border);
		color: var(--color-text-dim);
		font-size: var(--font-size-xs);
	}
	.search {
		padding: 6px 8px;
		background: rgba(15, 14, 26, 0.6);
		border: 1px solid var(--color-border);
		color: var(--color-text-primary);
		font-size: var(--font-size-xs);
	}
	.status {
		padding: 24px 0;
		color: var(--color-text-dim);
		font-size: var(--font-size-xs);
		text-align: center;
		line-height: 1.6;
	}
	.plugin-list {
		overflow-y: auto;
		display: flex;
		flex-direction: column;
		gap: 4px;
		flex: 1;
	}
	.plugin-row {
		background: rgba(15, 14, 26, 0.6);
		border: 1px solid var(--color-border);
		padding: 6px 10px;
		color: var(--color-text-primary);
		text-align: left;
		cursor: pointer;
		display: flex;
		flex-direction: column;
		gap: 2px;
	}
	.plugin-row:hover {
		border-color: var(--color-accent-magenta-dim);
		background: rgba(40, 20, 40, 0.6);
	}
	.plugin-name {
		color: var(--color-accent-magenta);
		font-size: var(--font-size-xs);
		letter-spacing: 1px;
	}
	.plugin-meta {
		color: var(--color-text-secondary);
		font-size: var(--font-size-xs);
	}
	.plugin-path {
		color: var(--color-text-dim);
		font-size: var(--font-size-xs);
		font-family: var(--font-code);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.error {
		padding: 6px 8px;
		background: rgba(200, 40, 40, 0.12);
		border: 1px solid rgba(200, 40, 40, 0.4);
		color: rgb(255, 120, 120);
		font-size: var(--font-size-xs);
	}
	.hint {
		font-size: var(--font-size-xs);
	}
</style>
