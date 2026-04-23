import { exec } from 'node:child_process';
import { promisify } from 'node:util';
import { access, constants } from 'node:fs/promises';
import { join } from 'node:path';
import { check, type Check } from './index.js';

const run = promisify(exec);

/** Resolve the unibit repo. Tries $UNIBIT_HOME, then ~/unibit. */
export function unibitPath(): string {
  if (process.env.UNIBIT_HOME) return process.env.UNIBIT_HOME;
  return join(process.env.HOME ?? '', 'unibit');
}

export const rustChecks: Check[] = [
  check('cargo-on-path', 'cargo available', async () => {
    try {
      const { stdout } = await run('cargo --version', { timeout: 5000 });
      return { status: 'ok', detail: stdout.trim() };
    } catch (e) {
      return {
        status: 'miss',
        detail: `cargo not on PATH: ${e instanceof Error ? e.message : e}`,
        hint: 'install Rust via rustup.rs',
      };
    }
  }),

  check('unibit-repo', 'unibit repo reachable', async () => {
    const path = unibitPath();
    try {
      await access(join(path, 'Cargo.toml'), constants.R_OK);
      await access(join(path, 'crates', 'unibit-sprawl', 'Cargo.toml'));
      return { status: 'ok', detail: `found at ${path}` };
    } catch {
      return {
        status: 'warn',
        detail: `no unibit repo at ${path}`,
        hint: `set $UNIBIT_HOME or clone unibit to ~/unibit (needed for \`npm run replay\`)`,
      };
    }
  }),
];
