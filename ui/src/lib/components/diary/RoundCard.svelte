<script lang="ts">
	let {
		number,
		title,
		accuracy,
		delta = null,
		description,
		date,
		complete = false,
		href = '',
	}: {
		number: number;
		title: string;
		accuracy: number;
		delta?: number | null;
		description: string;
		date: string;
		complete?: boolean;
		href?: string;
	} = $props();

	function pct(n: number): string {
		return (n * 100).toFixed(1) + '%';
	}
</script>

<a class="round-card" class:complete class:active={!complete && href} {href}>
	<div class="round-number">
		<div class="circle" class:complete>
			<span>{number}</span>
		</div>
		<div class="line"></div>
	</div>
	<div class="round-body">
		<div class="round-header">
			<span class="round-title">{title}</span>
			<span class="round-accuracy">
				{pct(accuracy)}
				{#if delta !== null}
					<span class="delta" class:positive={delta > 0} class:negative={delta < 0}>
						({delta > 0 ? '+' : ''}{pct(delta)})
					</span>
				{/if}
			</span>
		</div>
		<div class="round-desc">{description}</div>
		<div class="round-meta">
			{#if complete}
				<span class="done">Complete</span>
			{:else}
				<span class="in-progress">In progress</span>
			{/if}
			<span class="date">{date}</span>
		</div>
	</div>
</a>

<style>
	.round-card {
		display: flex;
		gap: 16px;
		text-decoration: none;
		margin-bottom: 16px;
	}
	.round-number {
		flex-shrink: 0;
		width: 48px;
		text-align: center;
	}
	.circle {
		width: 40px;
		height: 40px;
		border-radius: 50%;
		background: transparent;
		border: 2px dashed var(--color-text-dim);
		display: flex;
		align-items: center;
		justify-content: center;
		margin: 0 auto;
		font-family: var(--font-code);
		font-size: 16px;
		color: var(--color-text-dim);
	}
	.circle.complete {
		border-style: solid;
		border-color: var(--color-accent-cyan);
		background: rgba(51, 221, 255, 0.1);
		color: var(--color-accent-cyan);
	}
	.line {
		width: 2px;
		height: 24px;
		margin: 4px auto;
		background: var(--color-border);
	}
	.round-body {
		flex: 1;
		background: var(--color-bg-panel);
		border: 1px solid var(--color-border);
		border-radius: 0;
		padding: 16px;
	}
	.complete .round-body {
		border-color: rgba(51, 221, 255, 0.3);
	}
	.round-header {
		display: flex;
		justify-content: space-between;
		align-items: baseline;
	}
	.round-title {
		font-family: var(--font-reading);
		font-size: 15px;
		font-weight: 600;
		color: var(--color-text-primary);
	}
	.round-accuracy {
		font-family: var(--font-code);
		font-size: 16px;
		color: var(--color-accent-cyan);
	}
	.delta {
		font-size: 11px;
		color: var(--color-text-dim);
	}
	.delta.positive { color: var(--color-accent-teal); }
	.delta.negative { color: var(--color-accent-magenta); }
	.round-desc {
		color: var(--color-text-secondary);
		font-size: 13px;
		margin-top: 4px;
	}
	.round-meta {
		font-size: 11px;
		margin-top: 8px;
	}
	.done { color: var(--color-accent-teal); }
	.in-progress { color: var(--color-accent-cyan); }
	.date { color: var(--color-text-dim); margin-left: 8px; }
</style>
