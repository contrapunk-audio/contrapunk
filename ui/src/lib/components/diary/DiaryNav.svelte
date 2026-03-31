<script lang="ts">
	type Crumb = { label: string; href?: string };
	let { crumbs = [] }: { crumbs: Crumb[] } = $props();
</script>

<nav class="diary-nav">
	<a href="/" class="brand">CONTRAPUNK</a>
	{#each crumbs as crumb, i}
		<span class="sep">&gt;</span>
		{#if crumb.href && i < crumbs.length - 1}
			<a href={crumb.href} class="crumb">{crumb.label}</a>
		{:else}
			<span class="crumb active">{crumb.label}</span>
		{/if}
	{/each}
</nav>

<style>
	.diary-nav {
		position: sticky;
		top: 0;
		z-index: 50;
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 8px 16px;
		background: var(--color-bg-panel);
		border-bottom: 1px solid var(--color-border);
		font-size: 11px;
	}
	.brand {
		font-family: var(--font-pixel);
		font-size: var(--font-size-xs);
		color: var(--color-accent-magenta);
		text-decoration: none;
	}
	.brand:hover {
		color: var(--color-accent-pink);
	}
	.sep {
		color: var(--color-text-dim);
	}
	.crumb {
		color: var(--color-text-secondary);
		text-decoration: none;
		font-family: var(--font-reading);
	}
	.crumb:hover {
		color: var(--color-text-primary);
	}
	.crumb.active {
		color: var(--color-accent-cyan);
	}
</style>
