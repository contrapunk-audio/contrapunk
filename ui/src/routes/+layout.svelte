<script lang="ts">
	import '../app.css';

	let { children } = $props();
</script>

<div class="app-shell" style="height: 100vh; width: 100vw;">
	{@render children()}
</div>
