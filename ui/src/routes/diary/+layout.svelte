<script lang="ts">
	let { children } = $props();
</script>

<!-- Override parent app-shell: diary pages need to scroll and use standard fonts -->
<div class="diary-shell">
	{@render children()}
</div>

<style>
	.diary-shell {
		position: fixed;
		inset: 0;
		z-index: 100;
		overflow-y: auto;
		background: #030712;
		font-family: ui-sans-serif, system-ui, sans-serif;
		-webkit-font-smoothing: antialiased;
		-moz-osx-font-smoothing: grayscale;
	}
</style>
