import { access, constants, readFile, stat } from 'node:fs/promises';
import { join } from 'node:path';
import { check, type Check } from './index.js';

const root = process.cwd();
const REPLAY_PATH = join(root, 'public', 'sprawl-replay.ndjson');
const OCEL_PATH = join(root, 'ocel-log.jsonl');

export const fileChecks: Check[] = [
  check('replay-file', 'sprawl-replay.ndjson present', async () => {
    try {
      const txt = await readFile(REPLAY_PATH, 'utf8');
      const lines = txt
        .split('\n')
        .map((s) => s.trim())
        .filter((s) => s.length > 0);
      if (lines.length < 18) {
        return {
          status: 'miss',
          detail: `${lines.length} lines in replay (expected ≥ 18 for Case→Loa)`,
          hint: 'regenerate with `npm run replay` or `make sprawl-replay` in the unibit repo',
        };
      }
      // Validate shape: every line must parse as JSON with a `tick` field.
      for (const line of lines) {
        try {
          const ev = JSON.parse(line);
          if (typeof ev?.tick !== 'number') {
            return {
              status: 'miss',
              detail: `line without a numeric tick: ${line.slice(0, 60)}…`,
              hint: '`npm run replay` to regenerate',
            };
          }
        } catch (e) {
          return {
            status: 'miss',
            detail: `invalid JSON: ${e instanceof Error ? e.message : e}`,
            hint: '`npm run replay` to regenerate',
          };
        }
      }
      return { status: 'ok', detail: `${lines.length} turns, all parseable` };
    } catch {
      return {
        status: 'miss',
        detail: `${REPLAY_PATH} missing`,
        hint: '`npm run replay` to generate it',
      };
    }
  }),

  check('ocel-log', 'OCEL session log writable', async () => {
    try {
      await stat(OCEL_PATH);
      await access(OCEL_PATH, constants.W_OK);
      const txt = await readFile(OCEL_PATH, 'utf8');
      const lines = txt
        .split('\n')
        .map((s) => s.trim())
        .filter((s) => s.length > 0);
      if (lines.length === 0) {
        return {
          status: 'ok',
          detail: 'empty (fresh session)',
        };
      }
      try {
        const last = JSON.parse(lines[lines.length - 1]);
        const ts = last?.event?.timestamp ?? 'unknown';
        return {
          status: 'ok',
          detail: `${lines.length} events, last at ${ts}`,
        };
      } catch {
        return {
          status: 'warn',
          detail: `${lines.length} lines but last line is not JSON`,
          hint: '`npm run ocel:reset` to truncate',
        };
      }
    } catch {
      // File doesn't exist yet — that's fine, first POST will create it.
      try {
        await access(root, constants.W_OK);
        return { status: 'ok', detail: 'not yet created (will appear on first turn)' };
      } catch {
        return {
          status: 'miss',
          detail: 'matrix-tv cwd not writable',
          hint: 'fix fs permissions on apps/matrix-tv/',
        };
      }
    }
  }),
];

export { REPLAY_PATH, OCEL_PATH };
