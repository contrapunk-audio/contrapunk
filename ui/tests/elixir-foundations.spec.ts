import { expect, test } from '@playwright/test';

test.describe('Elixir Chapter 1 and 2 foundations', () => {
	test('edits role colour and applies deterministic chapter examples', async ({ page }) => {
		await page.goto('/');
		await page.getByRole('button', { name: 'Synth', exact: true }).click();
		await expect(page.getByRole('heading', { name: 'Harmonic colour' })).toBeVisible();
		await expect(page.getByText('6 HARMONICS × 16 VOICES')).toBeVisible();
		await expect(page.getByRole('link', { name: 'CHAPTER 1 ↗' })).toHaveAttribute('href', 'https://contrapunk.com/learn/wavetable-synthesis/chapter-1/');
		await expect(page.getByRole('link', { name: 'CHAPTER 2 ↗' })).toHaveAttribute('href', 'https://contrapunk.com/learn/wavetable-synthesis/chapter-2/');

		await page.getByRole('tab', { name: 'Harmony' }).click();
		await page.getByLabel('Recipe').selectOption('odd');
		await expect(page.getByLabel('Harmonic 2 amplitude')).toHaveValue('0');
		await expect(page.getByLabel('Harmonic 3 amplitude')).toHaveValue('0.45');
		await expect(page.getByLabel('Current waveform and one-sided spectrum preview')).toBeVisible();

		await page.getByLabel('Chapter example').selectOption('phase-cancellation');
		await expect(page.getByRole('button', { name: 'Add', exact: true })).toHaveAttribute('aria-pressed', 'true');
		await expect(page.getByText('Difference 0.0 Hz')).toBeVisible();
		await expect(page.getByText('Phase cancellation.')).toBeVisible();
	});

	test('opens a recording-ready deep link without replacing the saved sound', async ({ page }) => {
		await page.goto('/');
		await page.getByRole('button', { name: 'Synth', exact: true }).click();
		await page.getByLabel('Recipe').selectOption('dark');

		await page.goto('/?elixir-example=ring-difference');
		await expect(page.getByRole('heading', { name: 'Harmonic colour' })).toBeVisible();
		await expect(page.getByRole('button', { name: 'Ring', exact: true })).toHaveAttribute('aria-pressed', 'true');
		await expect(page.getByText('Ring difference.')).toBeVisible();
		await expect(page.getByText('B 110.0 Hz')).toBeVisible();

		await page.goto('/');
		await page.getByRole('button', { name: 'Synth', exact: true }).click();
		await expect(page.getByLabel('Recipe')).toHaveValue('dark');
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
