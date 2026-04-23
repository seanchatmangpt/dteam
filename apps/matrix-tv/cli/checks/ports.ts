import { exec } from 'node:child_process';
import { promisify } from 'node:util';
import { check, type Check } from './index.js';

const run = promisify(exec);

/** One row of `lsof -iTCP:<port> -sTCP:LISTEN -n -P`. */
export interface PortRow {
  command: string;
  pid: string;
  user: string;
  address: string;
}

/** Return the processes listening on `port`. Empty array if none. */
export async function listListeners(port: number): Promise<PortRow[]> {
  try {
    const { stdout } = await run(
      `lsof -iTCP:${port} -sTCP:LISTEN -n -P 2>/dev/null`,
      { timeout: 3000 }
    );
    const lines = stdout.trim().split('\n').slice(1); // drop header
    return lines
      .filter((l) => l.trim().length > 0)
      .map((l) => {
        // COMMAND PID USER FD TYPE DEVICE SIZE/OFF NODE NAME
        const parts = l.split(/\s+/);
        return {
          command: parts[0] ?? '',
          pid: parts[1] ?? '',
          user: parts[2] ?? '',
          address: parts[8] ?? parts[parts.length - 1] ?? '',
        };
      });
  } catch {
    // lsof missing or no listeners — both mapped to "none".
    return [];
  }
}

export const portChecks: Check[] = [
  check('next-port', 'dev server on :31337', async () => {
    const listeners = await listListeners(31337);
    if (listeners.length === 0) {
      return {
        status: 'miss',
        detail: 'nothing listening on :31337',
        hint: 'run `npm run dev`',
      };
    }
    const ours = listeners.find((l) =>
      ['node', 'next', 'Next', 'node.js'].some((p) => l.command.includes(p))
    );
    if (ours) {
      return {
        status: 'ok',
        detail: `pid ${ours.pid} (${ours.command}) listening on ${ours.address}`,
      };
    }
    return {
      status: 'warn',
      detail: `:31337 bound by ${listeners.map((l) => `${l.command}(${l.pid})`).join(', ')}`,
      hint: 'port squatted by another process — stop it or change --port in package.json',
    };
  }),

  check('grafana-squatter', 'Grafana off :3000', async () => {
    const listeners = await listListeners(3000);
    if (listeners.length === 0) {
      return { status: 'ok', detail: ':3000 free' };
    }
    const isGrafana = listeners.some((l) => /grafana/i.test(l.command));
    if (isGrafana) {
      return {
        status: 'warn',
        detail: `Grafana on :3000 (pid ${listeners[0].pid})`,
        hint: 'Grafana squatting :3000 is why we pinned 31337 — no action needed',
      };
    }
    return {
      status: 'warn',
      detail: `:3000 bound by ${listeners.map((l) => l.command).join(', ')}`,
      hint: 'not our port — ignore unless something else expects 3000',
    };
  }),
];
