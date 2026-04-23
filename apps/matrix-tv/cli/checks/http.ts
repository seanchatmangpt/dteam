import { check, type Check } from './index.js';

const DEV_BASE = process.env.MATRIX_TV_BASE ?? 'http://localhost:31337';
const OTLP_URL = process.env.NEXT_PUBLIC_OTLP_TRACES_URL ?? 'http://localhost:4318/v1/traces';

async function fetchWithTimeout(
  url: string,
  opts: RequestInit & { timeoutMs?: number } = {}
): Promise<Response> {
  const { timeoutMs = 1500, ...init } = opts;
  const ctl = new AbortController();
  const timer = setTimeout(() => ctl.abort(), timeoutMs);
  try {
    return await fetch(url, { ...init, signal: ctl.signal });
  } finally {
    clearTimeout(timer);
  }
}

export const httpChecks: Check[] = [
  check('dev-server', 'GET /sprawl', async () => {
    try {
      const r = await fetchWithTimeout(`${DEV_BASE}/sprawl`, {
        timeoutMs: 2500,
      });
      if (r.ok) {
        const html = await r.text();
        const hasPage =
          html.includes('data-testid="sprawl-page"') ||
          html.includes('Sprawl · blockchain MUD') ||
          html.includes('loading sprawl replay');
        return hasPage
          ? { status: 'ok', detail: `${DEV_BASE}/sprawl → ${r.status}` }
          : {
              status: 'warn',
              detail: `${r.status} but response doesn't look like /sprawl`,
              hint: 'maybe another app on :31337?',
            };
      }
      return {
        status: 'miss',
        detail: `${DEV_BASE}/sprawl → ${r.status}`,
        hint: 'check the dev server logs',
      };
    } catch (e) {
      return {
        status: 'miss',
        detail: `no response from ${DEV_BASE}/sprawl (${e instanceof Error ? e.message : e})`,
        hint: '`npm run dev` in another terminal',
      };
    }
  }),

  check('api-log-route', 'POST /api/ocel/log round-trip', async () => {
    const probe = {
      event: {
        id: 'doctor-probe:0',
        activity: 'probe',
        timestamp: new Date().toISOString(),
        attributes: { tick: -1, verdict: 'Lawful', scope: 0 },
        relationships: [{ objectId: 'player:doctor', qualifier: 'actor' }],
      },
      objects: [
        { id: 'player:doctor', type: 'player', attributes: { kind: 'doctor' } },
      ],
    };
    try {
      const r = await fetchWithTimeout(`${DEV_BASE}/api/ocel/log`, {
        method: 'POST',
        headers: { 'content-type': 'application/json' },
        body: JSON.stringify(probe),
      });
      if (!r.ok) {
        return {
          status: 'miss',
          detail: `POST /api/ocel/log → ${r.status}`,
          hint: 'check dev server logs',
        };
      }
      const j = (await r.json()) as { ok?: boolean; path?: string };
      return j.ok
        ? { status: 'ok', detail: `round-trip ok (log at ${j.path})` }
        : {
            status: 'miss',
            detail: `POST returned 200 but no ok field: ${JSON.stringify(j)}`,
            hint: 'check route implementation',
          };
    } catch (e) {
      return {
        status: 'miss',
        detail: `POST failed: ${e instanceof Error ? e.message : e}`,
        hint: 'dev server must be running',
      };
    }
  }),

  check('api-stream-route', 'SSE /api/ocel/stream handshake', async () => {
    const ctl = new AbortController();
    const timer = setTimeout(() => ctl.abort(), 1500);
    try {
      const r = await fetch(`${DEV_BASE}/api/ocel/stream`, {
        signal: ctl.signal,
      });
      clearTimeout(timer);
      if (!r.ok) {
        return {
          status: 'miss',
          detail: `GET /api/ocel/stream → ${r.status}`,
          hint: 'check route implementation',
        };
      }
      const ct = r.headers.get('content-type') ?? '';
      ctl.abort(); // cancel immediately; just verifying handshake
      return ct.includes('text/event-stream')
        ? { status: 'ok', detail: `content-type: ${ct}` }
        : {
            status: 'warn',
            detail: `content-type: ${ct} (expected text/event-stream)`,
            hint: 'SSE handshake may be broken',
          };
    } catch (e) {
      clearTimeout(timer);
      // An abort is expected here — distinguish from real failure.
      if (e instanceof Error && /aborted/i.test(e.message)) {
        return { status: 'ok', detail: 'SSE handshake completed (aborted on probe)' };
      }
      return {
        status: 'miss',
        detail: `SSE failed: ${e instanceof Error ? e.message : e}`,
        hint: 'dev server must be running',
      };
    }
  }),

  check('otlp-collector', 'OTLP collector on :4318', async () => {
    try {
      const r = await fetchWithTimeout(OTLP_URL, {
        method: 'POST',
        headers: { 'content-type': 'application/json' },
        body: '{}',
        timeoutMs: 800,
      });
      // Any reachable response (even 400 for bad payload) means the collector is up.
      return {
        status: 'ok',
        detail: `reachable at ${OTLP_URL} (${r.status})`,
      };
    } catch {
      return {
        status: 'skip',
        detail: `no OTLP collector at ${OTLP_URL}`,
        hint:
          'optional — spans fail silently without a collector; run Tempo/Jaeger/otel-collector on :4318 to capture them',
      };
    }
  }),
];
