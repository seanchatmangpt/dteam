import { defineCommand } from 'citty';
import { consola } from 'consola';
import { colors } from 'consola/utils';
import openUrl from 'open';

export default defineCommand({
  meta: {
    name: 'play',
    description: 'open /sprawl and /ocel in the default browser',
  },
  args: {
    base: {
      type: 'string',
      default: 'http://localhost:31337',
      description: 'dev-server base URL',
    },
    observer: {
      type: 'boolean',
      default: true,
      description: 'also open /ocel (disable with --no-observer)',
    },
  },
  async run({ args }) {
    const base = String(args.base);
    const player = `${base}/sprawl`;
    const observer = `${base}/ocel`;

    consola.info(`player   → ${colors.cyan(player)}`);
    await openUrl(player);

    if (args.observer) {
      consola.info(`observer → ${colors.cyan(observer)}`);
      await openUrl(observer);
    }
  },
});
