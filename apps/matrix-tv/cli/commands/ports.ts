import { defineCommand } from 'citty';
import { colors } from 'consola/utils';
import { listListeners } from '../checks/ports.js';

const WATCHED = [
  { port: 31337, role: 'matrix-tv dev server' },
  { port: 3000, role: 'Grafana (typical squatter)' },
  { port: 4318, role: 'OTLP collector (HTTP)' },
  { port: 4317, role: 'OTLP collector (gRPC)' },
  { port: 8088, role: 'unibit-sprawl WebSocket' },
  { port: 3412, role: 'Playwright test server' },
];

export default defineCommand({
  meta: {
    name: 'ports',
    description: 'inspect the ports matrix-tv cares about',
  },
  async run() {
    console.log('');
    console.log(
      `  ${colors.bold('PORT')}   ${colors.bold('ROLE'.padEnd(34))}  ${colors.bold('LISTENER')}`
    );
    console.log(`  ${'─'.repeat(72)}`);
    for (const { port, role } of WATCHED) {
      const listeners = await listListeners(port);
      if (listeners.length === 0) {
        console.log(
          `  ${String(port).padEnd(6)} ${role.padEnd(34)}  ${colors.dim('— free —')}`
        );
      } else {
        const first = listeners[0];
        const summary = `${first.command}(${first.pid}) ${colors.dim(first.address)}`;
        console.log(
          `  ${String(port).padEnd(6)} ${role.padEnd(34)}  ${colors.cyan(summary)}`
        );
        for (const extra of listeners.slice(1)) {
          console.log(
            `  ${''.padEnd(6)} ${''.padEnd(34)}  ${colors.cyan(`${extra.command}(${extra.pid}) ${colors.dim(extra.address)}`)}`
          );
        }
      }
    }
    console.log('');
  },
});
