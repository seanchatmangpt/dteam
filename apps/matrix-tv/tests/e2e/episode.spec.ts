import { expect, test } from '@playwright/test';

test.describe('Episode scene — admission verdicts', () => {
  test('N1 (valid state) starts as LAWFUL', async ({ page }) => {
    await page.goto('/episode/n1');
    const badge = page.getByTestId('verdict-badge');
    await expect(badge).toBeVisible();
    await expect(badge).toHaveAttribute('data-verdict', 'lawful');
    await expect(badge).toContainText('LAWFUL');
  });

  test('N3 (law-forbidden bit set) starts as UNLAWFUL', async ({ page }) => {
    await page.goto('/episode/n3');
    const badge = page.getByTestId('verdict-badge');
    await expect(badge).toBeVisible();
    await expect(badge).toHaveAttribute('data-verdict', 'unlawful');
    await expect(badge).toContainText('UNLAWFUL');
  });

  test('N7 (all forbidden) starts as UNLAWFUL', async ({ page }) => {
    await page.goto('/episode/n7');
    const badge = page.getByTestId('verdict-badge');
    await expect(badge).toHaveAttribute('data-verdict', 'unlawful');
  });

  test('RUN button re-runs the motion and appends to the ribbon', async ({
    page,
  }) => {
    await page.goto('/episode/n1');
    const scene = page.getByTestId('episode-scene');
    const initial = await scene.getAttribute('data-history-length');
    expect(Number(initial)).toBe(1);

    await page.getByTestId('run-button').click();
    await expect(scene).toHaveAttribute('data-history-length', '2');

    await page.getByTestId('run-button').click();
    await expect(scene).toHaveAttribute('data-history-length', '3');
  });

  test('TAMPER button flips the verdict to UNLAWFUL', async ({ page }) => {
    await page.goto('/episode/n1');
    const badge = page.getByTestId('verdict-badge');
    await expect(badge).toHaveAttribute('data-verdict', 'lawful');

    await page.getByTestId('tamper-button').click();
    await expect(badge).toHaveAttribute('data-verdict', 'unlawful');
  });

  test('Receipt ribbon tracks history and marks denied fragments', async ({
    page,
  }) => {
    await page.goto('/episode/n1');
    const ribbon = page.getByTestId('receipt-ribbon');
    await expect(ribbon).toHaveAttribute('data-length', '1');
    // first fragment is the admitted initial run
    await expect(
      ribbon.getByTestId('receipt-fragment').first()
    ).toHaveAttribute('data-admitted', 'true');

    await page.getByTestId('tamper-button').click();
    await expect(ribbon).toHaveAttribute('data-length', '2');
    // last fragment is the tampered (denied) one
    await expect(
      ribbon.getByTestId('receipt-fragment').last()
    ).toHaveAttribute('data-admitted', 'false');
  });

  test('fragment text is deterministic across reloads', async ({ page }) => {
    await page.goto('/episode/n1');
    const fragA = await page
      .getByTestId('verdict-fragment')
      .textContent();

    await page.reload();
    const fragB = await page
      .getByTestId('verdict-fragment')
      .textContent();

    expect(fragA).toBe(fragB);
    expect(fragA).toMatch(/fragment: [0-9a-f]{16}/);
  });
});
