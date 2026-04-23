/**
 * GET /api/ocel/stream — Server-Sent Events stream of the OCEL log.
 *
 * On connect, the server sends every existing event in the log as an
 * individual SSE message (each message is one JSONL line — the
 * `{event, objects}` pair). After the catch-up, it polls the file every
 * 400ms for appended bytes and emits them as they arrive. An observer
 * tab subscribes to this endpoint and sees turns land in near-real-time.
 */

import { promises as fs } from 'node:fs';
import { resolve } from 'node:path';

export const runtime = 'nodejs';
export const dynamic = 'force-dynamic';

const LOG_PATH = resolve(process.cwd(), 'ocel-log.jsonl');
const POLL_MS = 400;

export function GET(req: Request) {
  const encoder = new TextEncoder();

  const stream = new ReadableStream({
    async start(controller) {
      let offset = 0;
      let closed = false;

      const ping = () => {
        if (closed) return;
        try {
          controller.enqueue(encoder.encode(`: keepalive\n\n`));
        } catch {
          closed = true;
        }
      };

      const send = (payload: string) => {
        if (closed) return;
        try {
          controller.enqueue(encoder.encode(`data: ${payload}\n\n`));
        } catch {
          closed = true;
        }
      };

      const poll = async () => {
        if (closed) return;
        try {
          const stat = await fs.stat(LOG_PATH).catch(() => null);
          if (stat && stat.size > offset) {
            const fh = await fs.open(LOG_PATH, 'r');
            try {
              const len = stat.size - offset;
              const buf = Buffer.alloc(len);
              await fh.read(buf, 0, len, offset);
              offset = stat.size;
              const chunk = buf.toString('utf8');
              for (const line of chunk.split('\n')) {
                const t = line.trim();
                if (t.length > 0) send(t);
              }
            } finally {
              await fh.close();
            }
          }
        } catch {
          /* file not yet created — keep polling */
        }
      };

      // catch-up pass
      await poll();
      // steady state
      const iv = setInterval(poll, POLL_MS);
      const ka = setInterval(ping, 15_000);

      const close = () => {
        closed = true;
        clearInterval(iv);
        clearInterval(ka);
        try {
          controller.close();
        } catch {
          /* already closed */
        }
      };

      req.signal.addEventListener('abort', close, { once: true });
    },
  });

  return new Response(stream, {
    headers: {
      'content-type': 'text/event-stream; charset=utf-8',
      'cache-control': 'no-cache, no-transform',
      'x-accel-buffering': 'no',
      connection: 'keep-alive',
    },
  });
}
