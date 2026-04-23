/**
 * POST /api/ocel/log — append one OCEL 2.0 event + object-update to the
 * session log. The file is a JSON Lines document: one
 * `{ "event": OcelEvent, "objects": OcelObject[] }` object per line.
 *
 * DELETE /api/ocel/log — truncate the log (used when a new session starts).
 *
 * File location: `./ocel-log.jsonl` in the Next.js working directory.
 * This is intentionally outside `public/` so it's not served statically;
 * only the `/api/ocel/stream` SSE route reads it.
 */

import { promises as fs } from 'node:fs';
import { NextResponse } from 'next/server';
import { resolve } from 'node:path';

export const runtime = 'nodejs';
export const dynamic = 'force-dynamic';

const LOG_PATH = resolve(process.cwd(), 'ocel-log.jsonl');

export async function POST(req: Request) {
  const body = await req.json();
  if (!body || typeof body !== 'object' || !body.event) {
    return NextResponse.json(
      { error: 'expected { event, objects }' },
      { status: 400 }
    );
  }
  // Newline-delimited JSON; one OCEL event per line.
  await fs.appendFile(LOG_PATH, JSON.stringify(body) + '\n', 'utf8');
  return NextResponse.json({ ok: true, path: LOG_PATH });
}

export async function DELETE() {
  try {
    await fs.writeFile(LOG_PATH, '', 'utf8');
  } catch (e) {
    return NextResponse.json({ error: String(e) }, { status: 500 });
  }
  return NextResponse.json({ ok: true, truncated: LOG_PATH });
}

export async function GET() {
  try {
    const txt = await fs.readFile(LOG_PATH, 'utf8');
    const lines = txt
      .split('\n')
      .map((s) => s.trim())
      .filter((s) => s.length > 0);
    return NextResponse.json({ count: lines.length, path: LOG_PATH });
  } catch {
    return NextResponse.json({ count: 0, path: LOG_PATH });
  }
}
