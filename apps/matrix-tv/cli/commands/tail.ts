import { defineCommand } from 'citty';
import { colors } from 'consola/utils';
import { open } from 'node:fs/promises';
import { stat, watch } from 'node:fs';
import { OCEL_PATH } from '../checks/files.js';

const VERDICT_COLOR: Record<string, (s: string) => string> = {
  Lawful: colors.green,
  FastOnly: colors.yellow,
  CausalOnly: colors.yellow,
  Unlawful: colors.red,
};

const ROOM_PAD = 14;

export default defineCommand({
  meta: {
    name: 'tail',
    description: 'follow ocel-log.jsonl — one compact line per OCEL event',
  },
  args: {
    head: {
      type: 'boolean',
      default: false,
      description: 'print existing history before tailing',
    },
  },
  async run({ args }) {
    console.log(`${colors.dim('tailing')} ${OCEL_PATH}${colors.dim(' — Ctrl-C to stop')}`);
    console.log('');

    const printLine = (line: string) => {
      const s = line.trim();
      if (!s) return;
      try {
        const rec = JSON.parse(s) as {
          event: {
            attributes: Record<string, unknown>;
            activity: string;
            relationships: { objectId: string; qualifier: string }[];
          };
        };
        const tick = String(rec.event.attributes.tick ?? '?').padStart(4);
        const verb = String(rec.event.activity).padEnd(7);
        const verdict = String(rec.event.attributes.verdict ?? '-');
        const scope = String(rec.event.attributes.scope ?? '?').padStart(4);
        const room =
          rec.event.relationships.find((r) => r.qualifier === 'in_room')
            ?.objectId.replace(/^room:/, '') ?? '-';
        const paint = VERDICT_COLOR[verdict] ?? ((x: string) => x);
        console.log(
          `  ${colors.dim(tick)}  ${colors.bold(room.padEnd(ROOM_PAD))}  ${verb}  ${paint(verdict.padEnd(11))}  scope=${scope}`
        );
      } catch {
        console.log(`  ${colors.red('? malformed line')} ${s.slice(0, 100)}`);
      }
    };

    // Optional head pass.
    let offset = 0;
    try {
      const st = await new Promise<import('node:fs').Stats>((res, rej) =>
        stat(OCEL_PATH, (e, s) => (e ? rej(e) : res(s)))
      );
      if (args.head) {
        const fh = await open(OCEL_PATH, 'r');
        try {
          const buf = Buffer.alloc(st.size);
          await fh.read(buf, 0, st.size, 0);
          for (const line of buf.toString('utf8').split('\n')) printLine(line);
        } finally {
          await fh.close();
        }
      }
      offset = st.size;
    } catch {
      // file not yet present — first write will trigger watch
    }

    // Steady state: poll every 400ms for appended bytes.
    let running = true;
    process.on('SIGINT', () => {
      running = false;
      console.log('');
      console.log(colors.dim('tail stopped'));
      process.exit(0);
    });

    while (running) {
      await new Promise((r) => setTimeout(r, 400));
      try {
        const st = await new Promise<import('node:fs').Stats>((res, rej) =>
          stat(OCEL_PATH, (e, s) => (e ? rej(e) : res(s)))
        );
        if (st.size > offset) {
          const fh = await open(OCEL_PATH, 'r');
          try {
            const len = st.size - offset;
            const buf = Buffer.alloc(len);
            await fh.read(buf, 0, len, offset);
            offset = st.size;
            for (const line of buf.toString('utf8').split('\n')) printLine(line);
          } finally {
            await fh.close();
          }
        }
      } catch {
        // file still missing — keep waiting
      }
    }
    const _ = watch; // keep imported
  },
});
