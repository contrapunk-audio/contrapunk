import { expect, test } from '@playwright/test';

test.describe('Elixir synthesizer', () => {
	test('uses instrument language and edits each role sound', async ({ page }) => {
		await page.goto('/');
		await page.getByRole('button', { name: 'Synth', exact: true }).click();
		await expect(page.getByRole('heading', { name: 'Input oscillator' })).toBeVisible();
		await expect(page.getByText('6 HARMONICS × 16 VOICES')).toBeVisible();
		await expect(page.locator('.synth-panel')).not.toContainText(/CHAPTER\s+[12]/i);
		await expect(page.getByLabel('Factory patch')).toBeVisible();

		await page.getByRole('tab', { name: 'Harmony' }).click();
		await expect(page.getByRole('heading', { name: 'Harmony oscillator' })).toBeVisible();
		await page.getByLabel('Wave recipe').selectOption('odd');
		await expect(page.getByLabel('Harmonic 2 amplitude')).toHaveValue('0');
		await expect(page.getByLabel('Harmonic 3 amplitude')).toHaveValue('0.45');
		await expect(page.getByRole('img', { name: 'Oscillator input waveforms' })).toBeVisible();
		await expect(page.getByRole('img', { name: 'Final oscillator output waveform' })).toBeVisible();

		await page.getByLabel('Factory patch').selectOption('phase-cancellation');
		await expect(page.getByRole('button', { name: 'Add', exact: true })).toHaveAttribute('aria-pressed', 'true');
		await expect(page.getByText('Difference 0.0 Hz')).toBeVisible();
		await expect(page.getByText('Phase cancellation.')).toBeVisible();
	});

	test('shows the exact settled result and reacts during a drag', async ({ page }) => {
		await page.goto('/');
		await page.getByRole('button', { name: 'Synth', exact: true }).click();
		await page.getByRole('button', { name: 'Add', exact: true }).click();

		const inputPath = page.locator('.source-a');
		const outputPath = page.locator('.final-output');
		const outputScope = page.getByRole('img', { name: 'Final oscillator output waveform' });
		const sourcePeakBefore = await inputPath.getAttribute('data-peak');
		const outputBefore = await outputPath.getAttribute('d');
		await expect(outputScope).toHaveAttribute('data-formula', 'A + B');
		await expect(outputScope).toHaveAttribute('data-peak', '2.000');

		await page.getByLabel('Operator B phase').evaluate((node: HTMLInputElement) => {
			node.value = '0.5';
			node.dispatchEvent(new Event('input', { bubbles: true }));
		});

		await expect(outputScope).toHaveAttribute('data-peak', '0.000');
		await expect.poll(() => outputPath.getAttribute('d')).not.toBe(outputBefore);
		expect(await inputPath.getAttribute('data-peak')).toBe(sourcePeakBefore);
	});

	test('opens a factory-patch deep link without replacing the saved sound', async ({ page }) => {
		await page.goto('/');
		await page.getByRole('button', { name: 'Synth', exact: true }).click();
		await page.getByLabel('Wave recipe').selectOption('dark');

		await page.goto('/?elixir-example=ring-difference');
		await expect(page.getByRole('heading', { name: 'Input oscillator' })).toBeVisible();
		await expect(page.getByRole('button', { name: 'Ring', exact: true })).toHaveAttribute('aria-pressed', 'true');
		await expect(page.getByText('Ring difference.')).toBeVisible();
		await expect(page.getByText('B 110.0 Hz')).toBeVisible();

		await page.goto('/');
		await page.getByRole('button', { name: 'Synth', exact: true }).click();
		await expect(page.getByLabel('Wave recipe')).toHaveValue('dark');
	});

	test('keeps pitch and amplitude controls separate and named', async ({ page }) => {
		await page.goto('/');
		await page.getByRole('button', { name: 'Synth', exact: true }).click();
		await expect(page.getByRole('heading', { name: 'Articulation' })).toBeVisible();
		await expect(page.getByRole('heading', { name: 'Vibrato' })).toBeVisible();
		await expect(page.getByRole('heading', { name: 'Continuous pitch' })).toBeVisible();
		await expect(page.getByText('Sustain level', { exact: true })).toBeVisible();
		await expect(page.getByText('Mod-wheel add', { exact: true })).toBeVisible();
		await expect(page.getByRole('img', { name: 'Current attack, decay, sustain level, and release trajectory' })).toBeVisible();
		await expect(page.getByRole('img', { name: 'Current one-second vibrato pitch trajectory' })).toBeVisible();
		await expect(page.getByRole('heading', { name: 'Slide' })).toBeVisible();
	});
});
