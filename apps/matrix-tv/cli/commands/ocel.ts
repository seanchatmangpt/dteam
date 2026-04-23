import { defineCommand } from 'citty';
import { consola } from 'consola';
import { colors } from 'consola/utils';
import { readFile, writeFile } from 'node:fs/promises';
import { OCEL_PATH } from '../checks/files.js';

const reset = defineCommand({
  meta: {
    name: 'reset',
    description: 'truncate ocel-log.jsonl (start a fresh observed session)',
  },
  async run() {
    try {
      await writeFile(OCEL_PATH, '', 'utf8');
      consola.success(`truncated ${colors.cyan(OCEL_PATH)}`);
    } catch (e) {
      consola.error(`failed to truncate: ${e instanceof Error ? e.message : e}`);
      process.exit(1);
    }
  },
});

const stats = defineCommand({
  meta: {
    name: 'stats',
    description: 'aggregate ocel-log.jsonl — turns / rooms / verdicts / players',
  },
  async run() {
    let lines: string[] = [];
    try {
      const txt = await readFile(OCEL_PATH, 'utf8');
      lines = txt.split('\n').map((s) => s.trim()).filter((s) => s.length > 0);
    } catch {
      consola.warn('no ocel-log.jsonl yet — play /sprawl to populate it');
      return;
    }

    const players = new Map<string, number>();
    const rooms = new Map<string, number>();
    const verbs = new Map<string, number>();
    const verdicts = new Map<string, number>();

    for (const line of lines) {
      try {
        const rec = JSON.parse(line);
        for (const rel of rec.event.relationships as {
          objectId: string;
          qualifier: string;
        }[]) {
          if (rel.qualifier === 'actor')
            players.set(rel.objectId, (players.get(rel.objectId) ?? 0) + 1);
          if (rel.qualifier === 'in_room')
            rooms.set(rel.objectId, (rooms.get(rel.objectId) ?? 0) + 1);
        }
        verbs.set(rec.event.activity, (verbs.get(rec.event.activity) ?? 0) + 1);
        const v = String(rec.event.attributes?.verdict ?? 'unknown');
        verdicts.set(v, (verdicts.get(v) ?? 0) + 1);
      } catch {
        /* skip malformed */
      }
    }

    console.log('');
    console.log(`  ${colors.bold('OCEL log')}  ${colors.dim(OCEL_PATH)}`);
    console.log(`  ${colors.dim(`${lines.length} events`)}`);
    console.log('');
    section('players', players);
    section('rooms', rooms);
    section('verbs', verbs);
    section('verdicts', verdicts);
  },
});

function section(title: string, m: Map<string, number>) {
  console.log(`  ${colors.bold(title.toUpperCase())}`);
  if (m.size === 0) {
    console.log(`    ${colors.dim('(none)')}`);
    return;
  }
  const sorted = [...m.entries()].sort((a, b) => b[1] - a[1]);
  for (const [k, v] of sorted) {
    console.log(`    ${colors.cyan(k.padEnd(32))}  ${colors.dim(`×${v}`)}`);
  }
  console.log('');
}

export default defineCommand({
  meta: {
    name: 'ocel',
    description: 'OCEL log utilities',
  },
  subCommands: {
    reset,
    stats,
  },
});
