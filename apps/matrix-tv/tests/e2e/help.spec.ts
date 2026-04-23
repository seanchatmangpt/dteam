import { expect, test } from '@playwright/test';

test.describe('Diataxis help pages', () => {
  test('/help lands and links to all four quadrants', async ({ page }) => {
    await page.goto('/help');
    await expect(page.getByTestId('help-home')).toBeVisible();
    for (const q of ['tutorial', 'how-to', 'reference', 'explanation']) {
      await expect(page.getByTestId(`help-card-${q}`)).toBeVisible();
    }
  });

  test.describe.parallel('each quadrant renders its signature section', () => {
    for (const { path, testid, heading } of [
      { path: '/help/tutorial', testid: 'help-tutorial', heading: /Your first sixty seconds/i },
      { path: '/help/how-to', testid: 'help-how-to', heading: /Task recipes/i },
      { path: '/help/reference', testid: 'help-reference', heading: /Lookup tables/i },
      { path: '/help/explanation', testid: 'help-explanation', heading: /Why a blockchain MUD/i },
    ]) {
      test(path, async ({ page }) => {
        await page.goto(path);
        await expect(page.getByTestId(testid)).toBeVisible();
        await expect(page.locator('h1').first()).toHaveText(heading);
      });
    }
  });

  test('home page surfaces the tutorial link', async ({ page }) => {
    await page.goto('/');
    const help = page.getByTestId('help-link');
    await expect(help).toBeVisible();
    await help.click();
    await expect(page).toHaveURL(/\/help\/tutorial$/);
    await expect(page.getByTestId('help-tutorial')).toBeVisible();
  });

  test('how-to recipes are addressable by anchor', async ({ page }) => {
    await page.goto('/help/how-to');
    for (const id of [
      'how-play-sprawl',
      'how-watch',
      'how-regen',
      'how-otel',
      'how-doctor',
    ]) {
      await expect(page.getByTestId(`recipe-${id}`)).toBeVisible();
    }
  });
});
