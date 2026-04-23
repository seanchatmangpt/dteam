import { defineCommand } from 'citty';
import { consola } from 'consola';
import { colors } from 'consola/utils';
import {
  fileChecks,
  httpChecks,
  nodeChecks,
  portChecks,
  rustChecks,
  type Check,
  type CheckResult,
} from '../checks/index.js';

const ALL_CHECKS: Check[] = [
  ...nodeChecks,
  ...fileChecks,
  ...portChecks,
  ...httpChecks,
  ...rustChecks,
];

const GLYPH: Record<CheckResult['status'], string> = {
  ok: colors.green('●'),
  warn: colors.yellow('◐'),
  miss: colors.red('○'),
  skip: colors.gray('·'),
};

const LABEL_WIDTH = 32;

export default defineCommand({
  meta: {
    name: 'doctor',
    description: 'run every matrix-tv DX health check',
  },
  args: {
    json: {
      type: 'boolean',
      default: false,
      description: 'emit machine-readable JSON (one object to stdout)',
    },
    base: {
      type: 'string',
      description: 'override dev-server base URL (default http://localhost:31337)',
    },
  },
  async run({ args }) {
    if (args.base) process.env.MATRIX_TV_BASE = String(args.base);

    const results = await Promise.all(ALL_CHECKS.map((c) => c()));
    const summary = {
      total: results.length,
      ok: results.filter((r) => r.status === 'ok').length,
      warn: results.filter((r) => r.status === 'warn').length,
      miss: results.filter((r) => r.status === 'miss').length,
      skip: results.filter((r) => r.status === 'skip').length,
    };

    if (args.json) {
      process.stdout.write(
        JSON.stringify({ checks: results, summary }, null, 2) + '\n'
      );
      process.exit(summary.miss > 0 ? 1 : 0);
    }

    consola.box({
      title: 'matrix-tv doctor',
      message: `  ${DEV_LINE}  \n  ${colors.dim(`base: ${process.env.MATRIX_TV_BASE ?? 'http://localhost:31337'}`)}`,
      style: { borderColor: 'cyan' },
    });

    for (const r of results) {
      const label = r.label.padEnd(LABEL_WIDTH);
      const line = `  ${GLYPH[r.status]} ${colors.bold(label)}  ${colors.dim(r.detail)}`;
      console.log(line);
      if (r.status !== 'ok' && r.status !== 'skip' && r.hint) {
        console.log(`    ${colors.dim('→')} ${colors.yellow(r.hint)}`);
      }
    }

    console.log('');
    const parts = [
      `${colors.green(String(summary.ok))} ok`,
      summary.warn > 0 ? `${colors.yellow(String(summary.warn))} warn` : null,
      summary.miss > 0 ? `${colors.red(String(summary.miss))} miss` : null,
      summary.skip > 0 ? `${colors.gray(String(summary.skip))} skip` : null,
    ].filter(Boolean);
    console.log(`  ${parts.join('   ')}   ${colors.dim(`of ${summary.total} checks`)}`);
    console.log('');

    if (summary.miss > 0) {
      consola.error(`${summary.miss} check(s) in MISS state — see hints above`);
      process.exit(1);
    }
    if (summary.warn > 0) {
      consola.warn('some checks are warning-only — demo should still work');
    } else {
      consola.success('all checks passed');
    }
  },
});

const DEV_LINE = `open ${colors.cyan('http://localhost:31337/sprawl')}  ·  watch at ${colors.cyan('http://localhost:31337/ocel')}`;
