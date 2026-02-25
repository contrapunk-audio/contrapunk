<script lang="ts">
	type Option = { value: string; label: string };

	let {
		options,
		value = '',
		placeholder = 'Select...',
		small = false,
		onchange
	}: {
		options: Option[];
		value?: string;
		placeholder?: string;
		small?: boolean;
		onchange?: (value: string) => void;
	} = $props();

	let open = $state(false);
	let containerEl: HTMLDivElement | undefined = $state();

	let selectedLabel = $derived(
		options.find((o) => o.value === value)?.label ?? placeholder
	);

	function toggle() {
		open = !open;
	}

	function select(val: string) {
		open = false;
		onchange?.(val);
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') open = false;
	}

	function handleBlur(e: FocusEvent) {
		// Close if focus leaves the container entirely
		if (containerEl && !containerEl.contains(e.relatedTarget as Node)) {
			open = false;
		}
	}
</script>

<div
	class="pixel-select-wrap"
	class:small
	bind:this={containerEl}
	onkeydown={handleKeydown}
	onfocusout={handleBlur}
	role="listbox"
	tabindex="-1"
>
	<button
		class="pixel-select-trigger font-pixel"
		class:open
		class:placeholder={value === ''}
		onclick={toggle}
		type="button"
	>
		<span class="trigger-label">{selectedLabel}</span>
		<span class="trigger-arrow">{open ? '\u25B4' : '\u25BE'}</span>
	</button>

	{#if open}
		<div class="pixel-select-dropdown">
			<button
				class="pixel-select-option font-pixel"
				class:active={value === ''}
				onclick={() => select('')}
				type="button"
			>
				{placeholder}
			</button>
			{#each options as opt}
				<button
					class="pixel-select-option font-pixel"
					class:active={opt.value === value}
					onclick={() => select(opt.value)}
					type="button"
				>
					{opt.label}
				</button>
			{/each}
		</div>
	{/if}
</div>

<style>
	.pixel-select-wrap {
		position: relative;
		flex: 1;
	}

	.pixel-select-trigger {
		width: 100%;
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 4px;
		background: var(--color-widget-bg);
		border: 1px solid var(--color-border);
		color: var(--color-text-primary);
		font-size: 7px;
		padding: 3px 4px;
		border-radius: 0;
		cursor: pointer;
		-webkit-font-smoothing: none;
		text-rendering: optimizeSpeed;
		text-align: left;
	}

	.small .pixel-select-trigger {
		font-size: 6px;
		padding: 2px 3px;
	}

	.pixel-select-trigger.open,
	.pixel-select-trigger:focus {
		border-color: var(--color-accent-cyan);
		box-shadow: var(--glow-cyan);
		outline: none;
	}

	.pixel-select-trigger.placeholder .trigger-label {
		color: var(--color-text-dim);
	}

	.trigger-label {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		min-width: 0;
	}

	.trigger-arrow {
		color: var(--color-accent-cyan);
		font-size: 6px;
		flex-shrink: 0;
	}

	.pixel-select-dropdown {
		position: absolute;
		top: 100%;
		left: 0;
		right: 0;
		z-index: 100;
		background: var(--color-bg-panel);
		border: 1px solid var(--color-accent-cyan);
		border-top: none;
		max-height: 160px;
		overflow-y: auto;
		box-shadow: var(--glow-cyan);
	}

	.pixel-select-option {
		display: block;
		width: 100%;
		background: none;
		border: none;
		color: var(--color-text-primary);
		font-size: 7px;
		padding: 3px 4px;
		text-align: left;
		cursor: pointer;
		-webkit-font-smoothing: none;
		text-rendering: optimizeSpeed;
		border-radius: 0;
	}

	.small .pixel-select-option {
		font-size: 6px;
		padding: 2px 3px;
	}

	.pixel-select-option:hover {
		background: var(--color-accent-cyan);
		color: var(--color-bg-deep);
	}

	.pixel-select-option.active {
		color: var(--color-accent-magenta);
	}

	.pixel-select-option.active:hover {
		color: var(--color-bg-deep);
	}

	/* Scrollbar styling for the dropdown */
	.pixel-select-dropdown::-webkit-scrollbar {
		width: 4px;
	}

	.pixel-select-dropdown::-webkit-scrollbar-track {
		background: var(--color-bg-panel);
	}

	.pixel-select-dropdown::-webkit-scrollbar-thumb {
		background: var(--color-accent-cyan);
	}
</style>
