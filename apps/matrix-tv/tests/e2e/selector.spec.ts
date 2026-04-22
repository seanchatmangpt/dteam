import { expect, test } from '@playwright/test';

test.describe('Episode selector', () => {
  test('home page lists all 11 Sprawl runs', async ({ page }) => {
    await page.goto('/');
    await expect(page.getByTestId('episode-selector')).toBeVisible();

    const runIds = ['n1', 'n2', 'n3', 'n6', 'n7', 'cz1', 'cz3', 'cz4', 'mlo2', 'mlo6', 'mlo10'];
    for (const id of runIds) {
      await expect(page.getByTestId(`run-link-${id}`)).toBeVisible();
    }
  });

  test('three source sections render', async ({ page }) => {
    await page.goto('/');
    await expect(page.getByRole('heading', { name: 'Neuromancer' })).toBeVisible();
    await expect(page.getByRole('heading', { name: 'Count Zero' })).toBeVisible();
    await expect(
      page.getByRole('heading', { name: 'Mona Lisa Overdrive' })
    ).toBeVisible();
  });

  test('clicking a run navigates to the episode page', async ({ page }) => {
    await page.goto('/');
    await page.getByTestId('run-link-n1').click();
    await expect(page).toHaveURL(/\/episode\/n1$/);
    await expect(page.getByTestId('episode-scene')).toBeVisible();
  });
});
