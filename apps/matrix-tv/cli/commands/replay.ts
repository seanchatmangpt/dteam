import { defineCommand } from 'citty';
import { consola } from 'consola';
import { colors } from 'consola/utils';
import { spawn } from 'node:child_process';
import { readFile } from 'node:fs/promises';
import { join } from 'node:path';
import { REPLAY_PATH } from '../checks/files.js';
import { unibitPath } from '../checks/rust.js';

export default defineCommand({
  meta: {
    name: 'replay',
    description: 'regenerate public/sprawl-replay.ndjson from the unibit crate',
  },
  args: {
    unibit: {
      type: 'string',
      description: 'path to the unibit repo (default $UNIBIT_HOME or ~/unibit)',
    },
  },
  async run({ args }) {
    const repo = args.unibit ? String(args.unibit) : unibitPath();
    consola.info(`regenerating replay from ${colors.cyan(repo)}`);

    await new Promise<void>((resolve, reject) => {
      const child = spawn(
        'cargo',
        [
          'run',
          '-q',
          '-p',
          'unibit-sprawl',
          '--bin',
          'sprawl',
          '--no-default-features',
          '--features',
          'std',
          '--',
          'walk',
        ],
        {
          cwd: repo,
          stdio: ['ignore', 'pipe', 'inherit'],
        }
      );

      const outPath = REPLAY_PATH;
      const chunks: Buffer[] = [];
      child.stdout.on('data', (c) => chunks.push(c));
      child.on('error', reject);
      child.on('exit', async (code) => {
        if (code !== 0) {
          reject(new Error(`cargo exited ${code}`));
          return;
        }
        const out = Buffer.concat(chunks);
        const fsMod = await import('node:fs/promises');
        await fsMod.writeFile(outPath, out);
        resolve();
      });
    });

    const txt = await readFile(REPLAY_PATH, 'utf8');
    const count = txt.split('\n').filter((l) => l.trim().length > 0).length;
    consola.success(`wrote ${colors.cyan(REPLAY_PATH)}: ${count} turns`);
    const _ = join; // keep import used in some platforms
  },
});
