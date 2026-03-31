<script lang="ts">
	type Stat = { value: string; label: string; color?: string };
	let { stats = [] }: { stats: Stat[] } = $props();
</script>

<div class="stat-bar">
	{#each stats as stat}
		<div class="stat">
			<div class="stat-value" style:color={stat.color || 'var(--color-accent-cyan)'}>
				{stat.value}
			</div>
			<div class="stat-label">{stat.label}</div>
		</div>
	{/each}
</div>

<style>
	.stat-bar {
		display: flex;
		border-top: 1px solid var(--color-border);
		border-bottom: 1px solid var(--color-border);
	}
	.stat {
		flex: 1;
		padding: 16px;
		text-align: center;
		border-right: 1px solid var(--color-border);
	}
	.stat:last-child {
		border-right: none;
	}
	.stat-value {
		font-family: var(--font-pixel);
		font-size: 22px;
		font-weight: 700;
	}
	.stat-label {
		color: var(--color-text-dim);
		font-size: 11px;
		margin-top: 4px;
	}
</style>
