<script lang="ts">
	type Option = { value: string; label: string };

	let {
		options,
		value = '',
		placeholder = 'Select...',
		label,
		help,
		allowEmpty = false,
		small = false,
		disabled = false,
		onchange
	}: {
		options: Option[];
		value?: string;
		placeholder?: string;
		label?: string;
		/** Plain-language description shown on hover. */
		help?: string;
		allowEmpty?: boolean;
		small?: boolean;
		disabled?: boolean;
		onchange?: (value: string) => void;
	} = $props();

	let open = $state(false);
	let containerEl: HTMLDivElement | undefined = $state();

	let selectedLabel = $derived(
		options.find((o) => o.value === value)?.label ?? placeholder
	);

	function toggle() {
		if (disabled) return;
		open = !open;
	}

	function select(val: string) {
		open = false;
		onchange?.(val);
	}

	function handleKeydown(e: KeyboardEvent) {
		if (disabled) return;
		if (e.key === 'Escape') {
			open = false;
			return;
		}
		if (e.key !== 'ArrowDown' && e.key !== 'ArrowUp' && e.key !== 'Home' && e.key !== 'End') return;
		e.preventDefault();
		const values = allowEmpty
			? ['', ...options.map((option) => option.value)]
			: options.map((option) => option.value);
		if (values.length === 0) return;
		if (!open) {
			open = true;
			return;
		}
		const current = Math.max(0, values.indexOf(value));
		const next = e.key === 'Home'
			? 0
			: e.key === 'End'
				? values.length - 1
				: e.key === 'ArrowDown'
					? Math.min(values.length - 1, current + 1)
					: Math.max(0, current - 1);
		onchange?.(values[next]);
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
	class:disabled
	bind:this={containerEl}
	onfocusout={handleBlur}
>
	<button
		class="pixel-select-trigger font-ui"
		class:open
		class:placeholder={value === ''}
		disabled={disabled}
		onclick={toggle}
		onkeydown={handleKeydown}
		type="button"
		aria-haspopup="listbox"
		aria-expanded={open}
		aria-label={label ? `${label}: ${selectedLabel}` : selectedLabel}
		title={help}
	>
		<span class="trigger-label">{selectedLabel}</span>
		<span class="trigger-arrow">{open ? '\u25B4' : '\u25BE'}</span>
	</button>

	{#if open}
		<div class="pixel-select-dropdown" role="listbox" aria-label={label ?? placeholder}>
			{#if allowEmpty}
				<button
					class="pixel-select-option font-ui"
					class:active={value === ''}
					onclick={() => select('')}
					type="button"
					role="option"
					aria-selected={value === ''}
				>
					{placeholder}
				</button>
			{/if}
			{#each options as opt}
				<button
					class="pixel-select-option font-ui"
					class:active={opt.value === value}
					onclick={() => select(opt.value)}
					type="button"
					role="option"
					aria-selected={opt.value === value}
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
		font-size: var(--font-size-xs);
		padding: 3px 4px;
		border-radius: 0;
		cursor: pointer;
		-webkit-font-smoothing: none;
		text-rendering: optimizeSpeed;
		text-align: left;
	}

	.small .pixel-select-trigger {
		font-size: var(--font-size-xs);
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

	.pixel-select-wrap.disabled { opacity: 0.52; }
	.pixel-select-trigger:disabled { cursor: not-allowed; }

	.trigger-label {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		min-width: 0;
	}

	.trigger-arrow {
		color: var(--color-accent-cyan);
		font-size: var(--font-size-xs);
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
		font-size: var(--font-size-xs);
		padding: 3px 4px;
		text-align: left;
		cursor: pointer;
		-webkit-font-smoothing: none;
		text-rendering: optimizeSpeed;
		border-radius: 0;
	}

	.small .pixel-select-option {
		font-size: var(--font-size-xs);
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
