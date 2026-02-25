<script lang="ts">
	import '../app.css';
	import Particles from '$lib/components/Particles.svelte';
	import { ui } from '$lib/stores/ui.svelte';

	let { children } = $props();

	// Apply .reduced-motion class to body based on ui store
	$effect(() => {
		ui.applyMotionPreference();
	});

	// Detect system reduced-motion preference on mount
	$effect(() => {
		ui.detectSystemMotionPreference();
	});
</script>

<!-- Particles: lowest z-index layer, fixed position background -->
<Particles />

<!-- All UI content renders above particles -->
<div class="app-shell">
	{@render children()}
</div>

<style>
	.app-shell {
		position: relative;
		z-index: 1;
		height: 100vh;
		width: 100vw;
		overflow: hidden;
		background: var(--color-bg-deep);
	}
</style>
