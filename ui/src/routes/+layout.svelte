<script lang="ts">
	import { onDestroy } from 'svelte';
	import '../app.css';
	import Particles from '$lib/components/Particles.svelte';
	import { ui } from '$lib/stores/ui.svelte';
	import { adapter } from '$lib/adapter';

	let { children } = $props();

	$effect(() => {
		ui.applyMotionPreference();
	});

	$effect(() => {
		ui.detectSystemMotionPreference();
	});

	// Tear down adapter background loops on unmount (prevents RAF leaks
	// across HMR and page navigation in dev).
	onDestroy(() => {
		adapter.destroy?.();
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
		/* Transparent so the Particles canvas (z-index 0) bleeds through
		   wherever panels / columns don't cover the screen. Body's own
		   background paints the base color. */
		background: transparent;
	}
</style>
