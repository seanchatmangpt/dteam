import { expect, test } from '@playwright/test';
import { exec } from 'node:child_process';
import { promisify } from 'node:util';
import { resolve } from 'node:path';

const run = promisify(exec);

test.describe('matrix-tv-doctor CLI', () => {
  test('doctor --json emits a valid schema', async () => {
    const cwd = resolve(__dirname, '..', '..');
    const { stdout } = await run('npm run doctor:json --silent', {
      cwd,
      timeout: 30_000,
    }).catch((e) => ({ stdout: (e as { stdout?: string }).stdout ?? '' }));

    const trimmed = stdout.trim();
    // The JSON may be preceded by npm noise; find the first `{`.
    const start = trimmed.indexOf('{');
    expect(start).toBeGreaterThanOrEqual(0);

    const report = JSON.parse(trimmed.slice(start)) as {
      checks: { id: string; status: string; detail: string }[];
      summary: { total: number; ok: number; warn: number; miss: number; skip: number };
    };

    expect(Array.isArray(report.checks)).toBe(true);
    expect(report.checks.length).toBeGreaterThanOrEqual(10);
    expect(report.summary.total).toEqual(report.checks.length);

    const ids = new Set(report.checks.map((c) => c.id));
    for (const required of [
      'node-version',
      'pkg-installed',
      'replay-file',
      'ocel-log',
      'dev-server',
      'api-log-route',
      'api-stream-route',
    ]) {
      expect(ids.has(required), `doctor must include check '${required}'`).toBe(
        true
      );
    }

    // Every check records one of the four canonical statuses.
    for (const c of report.checks) {
      expect(['ok', 'warn', 'miss', 'skip']).toContain(c.status);
      expect(typeof c.detail).toBe('string');
    }
  });
});
