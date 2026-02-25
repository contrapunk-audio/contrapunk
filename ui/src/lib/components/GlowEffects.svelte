<script lang="ts">
	/**
	 * GlowEffects — Neon glow wrapper component
	 *
	 * Wraps any content with an HLD-style neon glow. Uses CSS box-shadow
	 * from token variables plus the glow-pulse animation from pixel.css.
	 * When ui.animationsEnabled is false, the glow remains visible (static)
	 * but does not pulse (the .reduced-motion class in tokens.css disables animation).
	 *
	 * Usage:
	 *   <GlowEffects color="magenta">
	 *     <button>Click me</button>
	 *   </GlowEffects>
	 *
	 * Or use CSS classes directly: .glow-magenta, .glow-cyan, .glow-teal
	 */

	import type { Snippet } from 'svelte';

	interface Props {
		color?: 'magenta' | 'cyan' | 'teal';
		pulse?: boolean;
		children: Snippet;
	}

	let { color = 'magenta', pulse = true, children }: Props = $props();
</script>

<div
	class="glow-wrapper glow-{color}"
	class:neon-glow={pulse}
>
	{@render children()}
</div>

<style>
	.glow-wrapper {
		display: contents;
	}

	/* When used as a wrapper, apply glow via box-shadow to first child.
	   For CSS-class usage, these same classes can be applied directly. */
	:global(.glow-magenta) {
		box-shadow: var(--glow-magenta);
	}

	:global(.glow-cyan) {
		box-shadow: var(--glow-cyan);
	}

	:global(.glow-teal) {
		box-shadow: var(--glow-teal);
	}
</style>
