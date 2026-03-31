<script lang="ts">
	let { children } = $props();
</script>

<div class="diary-shell">
	{@render children()}
</div>

<style>
	.diary-shell {
		position: fixed;
		inset: 0;
		z-index: 100;
		overflow-y: auto;
		background: var(--color-bg-deep);
		font-family: var(--font-reading);
		color: var(--color-text-primary);
		-webkit-font-smoothing: antialiased;
		-moz-osx-font-smoothing: grayscale;
	}
</style>
