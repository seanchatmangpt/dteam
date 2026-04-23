import { expect, test } from '@playwright/test';

test.describe('OCEL live observer', () => {
  test.beforeEach(async ({ request }) => {
    // Truncate the session log so each test starts fresh.
    await request.delete('/api/ocel/log');
  });

  test('/ocel page loads with SSE connected status', async ({ page }) => {
    await page.goto('/ocel');
    await expect(page.getByTestId('ocel-page')).toBeVisible();
    await expect(page.getByTestId('sse-status')).toContainText('CONNECTED');
  });

  test('playing /sprawl populates /ocel within a few seconds', async ({
    browser,
  }) => {
    const ctx = await browser.newContext();
    const observer = await ctx.newPage();
    await observer.goto('/ocel');
    await expect(observer.getByTestId('ocel-page')).toBeVisible();

    const player = await ctx.newPage();
    await player.goto('/sprawl');
    await expect(player.getByTestId('sprawl-page')).toBeVisible();

    // Let the autoplay emit a handful of turns (~1Hz, 900ms interval).
    await player.waitForTimeout(4000);

    // The observer should have received at least 3 OCEL events.
    const count = await observer
      .getByTestId('ocel-page')
      .getAttribute('data-event-count');
    expect(Number(count)).toBeGreaterThanOrEqual(3);

    await ctx.close();
  });

  test('OCEL event log endpoint round-trips a JSONL entry', async ({
    request,
  }) => {
    const payload = {
      event: {
        id: 'player:test:0',
        activity: 'probe',
        timestamp: new Date().toISOString(),
        attributes: { tick: 0, verdict: 'Lawful', scope: 66 },
        relationships: [
          { objectId: 'player:test', qualifier: 'actor' },
          { objectId: 'room:case', qualifier: 'in_room' },
        ],
      },
      objects: [
        { id: 'player:test', type: 'player', attributes: { kind: 'test' } },
      ],
    };
    const post = await request.post('/api/ocel/log', { data: payload });
    expect(post.ok()).toBe(true);

    const get = await request.get('/api/ocel/log');
    const j = await get.json();
    expect(j.count).toBeGreaterThanOrEqual(1);
  });
});
