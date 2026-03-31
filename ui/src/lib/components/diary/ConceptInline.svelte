<script lang="ts">
	let { term, children }: { term: string; children: any } = $props();
	let expanded = $state(false);
</script>

<span class="concept-trigger" onclick={() => expanded = !expanded}>
	{term}
</span>
{#if expanded}
	<div class="concept-body">
		{@render children()}
	</div>
{/if}

<style>
	.concept-trigger {
		color: var(--color-accent-cyan);
		border-bottom: 1px dashed var(--color-accent-cyan-dim);
		cursor: pointer;
	}
	.concept-trigger:hover {
		color: var(--color-accent-cyan);
		border-bottom-color: var(--color-accent-cyan);
	}
	.concept-body {
		margin: 8px 0 12px;
		padding: 12px 16px;
		background: var(--color-bg-panel);
		border-left: 2px solid var(--color-accent-cyan-dim);
		font-size: 14px;
		line-height: 1.7;
		color: var(--color-text-secondary);
	}
</style>
