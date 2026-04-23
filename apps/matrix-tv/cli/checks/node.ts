import { access, constants, readFile, stat } from 'node:fs/promises';
import { join } from 'node:path';
import { check, type Check } from './index.js';

const root = process.cwd();

export const nodeChecks: Check[] = [
  check('node-version', 'Node.js version', async () => {
    const v = process.version; // e.g. "v22.10.0"
    const major = Number(v.replace(/^v/, '').split('.')[0]);
    if (major >= 20) {
      return { status: 'ok', detail: `${v} (≥ 20)` };
    }
    return {
      status: 'miss',
      detail: `${v} (need ≥ 20)`,
      hint: 'upgrade Node — try `nvm install 22 && nvm use 22`',
    };
  }),

  check('pkg-installed', 'node_modules installed', async () => {
    try {
      await access(join(root, 'node_modules'), constants.F_OK);
      const pkg = JSON.parse(
        await readFile(join(root, 'package.json'), 'utf8')
      );
      // Sample three pinned deps to confirm a real install.
      const must = ['next', 'citty', '@opentelemetry/api'];
      for (const m of must) {
        await access(join(root, 'node_modules', m, 'package.json'));
      }
      return {
        status: 'ok',
        detail: `next@${pkg.dependencies?.next} citty@${pkg.dependencies?.citty}`,
      };
    } catch (e) {
      return {
        status: 'miss',
        detail: `node_modules missing or incomplete: ${e instanceof Error ? e.message : e}`,
        hint: 'run `npm install` in apps/matrix-tv/',
      };
    }
  }),

  check('workspace-root', 'single workspace lockfile', async () => {
    const here = join(root, 'package-lock.json');
    const home = process.env.HOME ?? '';
    const suspects = [
      join(home, 'pnpm-lock.yaml'),
      join(home, 'package-lock.json'),
    ];
    try {
      await stat(here);
    } catch {
      return {
        status: 'miss',
        detail: 'no package-lock.json in apps/matrix-tv/',
        hint: 'run `npm install`',
      };
    }
    const stray: string[] = [];
    for (const s of suspects) {
      try {
        await stat(s);
        stray.push(s);
      } catch {
        /* not there — good */
      }
    }
    if (stray.length === 0) {
      return { status: 'ok', detail: 'only ./package-lock.json present' };
    }
    return {
      status: 'warn',
      detail: `stray lockfile(s): ${stray.join(', ')}`,
      hint: 'causes Turbopack root-inference warnings; remove or set `turbopack.root` in next.config',
    };
  }),
];
