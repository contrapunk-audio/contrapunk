<script lang="ts">
	import { ui } from '$lib/stores/ui.svelte';

	/**
	 * Preview value used while the user drags the scale slider. Committed
	 * to the actual UI scale only on `change` (pointer release) so the
	 * page doesn't re-zoom frame-by-frame during the drag.
	 */
	let previewScale = $state(ui.uiScale);
	let previewFontScale = $state(ui.fontScale);

	// Keep the previews in sync when the store is updated from elsewhere
	// (e.g. localStorage restore on init).
	$effect(() => {
		previewScale = ui.uiScale;
	});
	$effect(() => {
		previewFontScale = ui.fontScale;
	});

	let previewPercent = $derived(Math.round(previewScale * 100));
	let currentPercent = $derived(Math.round(ui.uiScale * 100));
	let previewFontPercent = $derived(Math.round(previewFontScale * 100));
	let currentFontPercent = $derived(Math.round(ui.fontScale * 100));

	function onScaleInput(e: Event) {
		const v = Number((e.target as HTMLInputElement).value);
		if (Number.isFinite(v)) previewScale = v;
	}
	function onScaleCommit() {
		ui.setUiScale(previewScale);
	}

	function onFontScaleInput(e: Event) {
		const v = Number((e.target as HTMLInputElement).value);
		if (Number.isFinite(v)) {
			previewFontScale = v;
			// Live-preview the typography without committing to localStorage.
			if (typeof document !== 'undefined') {
				document.documentElement.style.setProperty('--font-scale', String(v));
			}
		}
	}
	function onFontScaleCommit() {
		ui.setFontScale(previewFontScale);
	}

	function toggleFx() {
		ui.toggleAnimations();
		try {
			localStorage.setItem('contrapunk-fx', ui.animationsEnabled ? 'on' : 'off');
		} catch {
			/* localStorage unavailable */
		}
	}

	function toggleNoteLabels() {
		ui.setShowNoteLabels(!ui.showNoteLabels);
	}

	function close() {
		ui.closeSettings();
	}

	function onBackdrop(e: MouseEvent) {
		if (e.target === e.currentTarget) close();
	}

	function onKey(e: KeyboardEvent) {
		if (e.key === 'Escape') close();
	}
</script>

<svelte:window onkeydown={onKey} />

{#if ui.settingsOpen}
	<!-- svelte-ignore a11y_click_events_have_key_events -->
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div class="backdrop" onclick={onBackdrop}>
		<div class="modal pixel-card" role="dialog" aria-modal="true" aria-label="Settings">
			<div class="modal-header">
				<span class="title font-ui">Settings</span>
				<button class="close-btn pixel-btn font-ui" onclick={close} title="Close (Esc)">
					×
				</button>
			</div>

			<div class="modal-body">
				<!-- Appearance section -->
				<section class="section">
					<h3 class="section-title font-ui">Appearance</h3>

					<div class="row">
						<label class="row-label font-ui" for="settings-ui-scale">UI scale</label>
						<div class="row-control">
							<input
								id="settings-ui-scale"
								class="scale-slider"
								type="range"
								min="0.75"
								max="2"
								step="0.05"
								value={previewScale}
								oninput={onScaleInput}
								onchange={onScaleCommit}
							/>
							<span class="value font-code">
								{previewPercent}%{#if previewPercent !== currentPercent}
									<span class="pending">*</span>
								{/if}
							</span>
						</div>
					</div>

					<div class="row">
						<label class="row-label font-ui" for="settings-font-scale">Font size</label>
						<div class="row-control">
							<input
								id="settings-font-scale"
								class="scale-slider"
								type="range"
								min="0.75"
								max="1.5"
								step="0.05"
								value={previewFontScale}
								oninput={onFontScaleInput}
								onchange={onFontScaleCommit}
							/>
							<span class="value font-code">
								{previewFontPercent}%{#if previewFontPercent !== currentFontPercent}
									<span class="pending">*</span>
								{/if}
							</span>
						</div>
					</div>

					<div class="row">
						<span class="row-label font-ui">Note labels</span>
						<div class="row-control">
							<button
								class="pixel-btn font-ui"
								class:active={ui.showNoteLabels}
								onclick={toggleNoteLabels}
								type="button"
							>
								{ui.showNoteLabels ? 'On' : 'Off'}
							</button>
							<span class="hint font-ui">
								{ui.showNoteLabels ? 'C4, D#5 on active keys' : 'Keys + frets only'}
							</span>
						</div>
					</div>
				</section>

				<!-- Effects section -->
				<section class="section">
					<h3 class="section-title font-ui">Effects</h3>

					<div class="row">
						<span class="row-label font-ui">Visual FX</span>
						<div class="row-control">
							<button
								class="pixel-btn font-ui"
								class:active={ui.animationsEnabled}
								onclick={toggleFx}
							>
								{ui.animationsEnabled ? 'On' : 'Off'}
							</button>
							<span class="hint font-ui">
								{ui.animationsEnabled ? 'Glows + particles' : 'Reduced motion'}
							</span>
						</div>
					</div>
				</section>
			</div>
		</div>
	</div>
{/if}

<style>
	.backdrop {
		position: fixed;
		inset: 0;
		background: rgba(10, 10, 26, 0.72);
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 1000;
	}

	.modal {
		min-width: 360px;
		max-width: 520px;
		width: 50vw;
		background: var(--color-bg-panel);
		border: 2px solid var(--color-border);
		box-shadow: 0 0 20px rgba(255, 51, 136, 0.2);
	}

	.modal-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 8px 12px;
		border-bottom: 1px solid var(--color-border);
	}

	.title {
		font-size: var(--font-size-sm);
		color: var(--color-accent-magenta);
		letter-spacing: 1px;
	}

	.close-btn {
		padding: 2px 8px;
		font-size: var(--font-size-sm);
		min-width: 28px;
	}

	.modal-body {
		padding: 12px;
		display: flex;
		flex-direction: column;
		gap: 14px;
	}

	.section {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}

	.section-title {
		font-size: var(--font-size-xs);
		color: var(--color-accent-cyan);
		letter-spacing: 1px;
		text-transform: uppercase;
		padding-bottom: 4px;
		border-bottom: 1px solid var(--color-border);
		margin: 0;
	}

	.row {
		display: grid;
		grid-template-columns: 90px 1fr;
		align-items: center;
		gap: 10px;
	}

	.row-label {
		font-size: var(--font-size-xs);
		color: var(--color-text-secondary);
	}

	.row-control {
		display: flex;
		align-items: center;
		gap: 6px;
	}

	.scale-slider {
		-webkit-appearance: none;
		appearance: none;
		flex: 1;
		height: 4px;
		background: var(--color-widget-bg);
		border: 1px solid var(--color-border);
		outline: none;
	}
	.scale-slider::-webkit-slider-thumb {
		-webkit-appearance: none;
		appearance: none;
		width: 10px;
		height: 14px;
		background: var(--color-accent-cyan);
		border: none;
		cursor: pointer;
	}
	.scale-slider::-moz-range-thumb {
		width: 10px;
		height: 14px;
		background: var(--color-accent-cyan);
		border: none;
		cursor: pointer;
	}

	.value {
		font-size: var(--font-size-xs);
		color: var(--color-accent-cyan);
		min-width: 42px;
		text-align: right;
	}

	.pending {
		color: var(--color-accent-magenta);
		margin-left: 2px;
	}

	.hint {
		font-size: var(--font-size-xs);
		color: var(--color-text-dim);
	}
</style>
