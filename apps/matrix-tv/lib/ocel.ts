/**
 * OCEL 2.0 emitter for the Sprawl MUD.
 *
 * Converts a `SprawlEvent` (one turn of the blockchain-MUD) into an OCEL
 * 2.0 event + object-update record and POSTs it to `/api/ocel/log`. The
 * server appends it to a JSONL file; `/ocel` tails the file via SSE so
 * an observer can watch a session live.
 *
 * Object types used (OCEL 2.0 section 3):
 *   - player       — the session author (one per browser tab)
 *   - chain        — the DualReceipt hash chain this session advances
 *   - room         — one of the nine Neuromancer-anchored MUD rooms
 *   - cell         — the GlobeCell word the turn touched (scope)
 *
 * Activity:
 *   - one per SprawlEvent verb (look, probe, admit, move, use, seal)
 */

import type { SprawlEvent } from '@/lib/sprawl';
import { initOtel } from '@/lib/otel';

/** OCEL 2.0 object types. */
export const OCEL_OBJECT_TYPES = ['player', 'chain', 'room', 'cell'] as const;
export type OcelObjectType = (typeof OCEL_OBJECT_TYPES)[number];

/** One OCEL 2.0 event in log-line form. */
export interface OcelEvent {
  /** Unique event id; using `player:tick` guarantees uniqueness per session. */
  id: string;
  /** Activity name — equals the SprawlEvent verb. */
  activity: string;
  /** ISO-8601 timestamp. */
  timestamp: string;
  /** Event-level attributes (flat primitives only, per OCEL 2.0). */
  attributes: Record<string, string | number | boolean>;
  /** Object relationships: (object_id, qualifier). */
  relationships: { objectId: string; qualifier: string }[];
}

/** One OCEL 2.0 object reference (for live observer aggregations). */
export interface OcelObject {
  id: string;
  type: OcelObjectType;
  attributes: Record<string, string | number | boolean>;
}

/** Build an OCEL event + object-update batch from a SprawlEvent. */
export function toOcel(
  ev: SprawlEvent,
  playerId: string
): { event: OcelEvent; objects: OcelObject[] } {
  const chainId = `chain:${playerId}`;
  const roomId = `room:${ev.room}`;
  const cellId = `cell:${ev.scope}`;
  const receiptHex = ev.receipt_causal_prefix
    .map((b) => b.toString(16).padStart(2, '0'))
    .join('');

  const event: OcelEvent = {
    id: `${playerId}:${ev.tick}`,
    activity: ev.verb,
    timestamp: new Date().toISOString(),
    attributes: {
      tick: ev.tick,
      verdict: ev.verdict,
      scope: ev.scope,
      delta_popcount: ev.delta_popcount,
      receipt_fast: `0x${ev.receipt_fast.toString(16)}`,
      receipt_causal_prefix: receiptHex,
      before_cell: `0x${ev.before_cell.toString(16)}`,
      after_cell: `0x${ev.after_cell.toString(16)}`,
    },
    relationships: [
      { objectId: playerId, qualifier: 'actor' },
      { objectId: chainId, qualifier: 'advances' },
      { objectId: roomId, qualifier: 'in_room' },
      { objectId: cellId, qualifier: 'touches' },
    ],
  };

  const objects: OcelObject[] = [
    { id: playerId, type: 'player', attributes: { kind: 'browser' } },
    { id: chainId, type: 'chain', attributes: { genesis: 'blake3_of_32_zero_octets' } },
    { id: roomId, type: 'room', attributes: { name: ev.room } },
    { id: cellId, type: 'cell', attributes: { word_index: ev.scope } },
  ];

  return { event, objects };
}

/**
 * Emit one turn: open an OTel span, post the OCEL event, return.
 *
 * Both the span and the POST are fire-and-forget; we never block the
 * renderer on observability.
 */
export async function emitTurn(
  ev: SprawlEvent,
  playerId: string
): Promise<void> {
  const tracer = initOtel();
  const span = tracer.startSpan(`sprawl.${ev.verb}`, {
    attributes: {
      'sprawl.tick': ev.tick,
      'sprawl.room': ev.room,
      'sprawl.verb': ev.verb,
      'sprawl.verdict': ev.verdict,
      'sprawl.scope': ev.scope,
      'sprawl.delta_popcount': ev.delta_popcount,
      'sprawl.player_id': playerId,
    },
  });

  const { event, objects } = toOcel(ev, playerId);

  try {
    const res = await fetch('/api/ocel/log', {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ event, objects }),
    });
    if (!res.ok) {
      span.recordException(new Error(`ocel/log returned ${res.status}`));
    }
  } catch (e) {
    span.recordException(e as Error);
  } finally {
    if (ev.verdict !== 'Lawful') {
      span.setStatus({ code: 2, message: ev.verdict });
    }
    span.end();
  }
}

/** Stable per-tab player id, persisted in sessionStorage. */
export function getPlayerId(): string {
  if (typeof window === 'undefined') return 'ssr-player';
  const key = 'matrix-tv.player_id';
  let id = window.sessionStorage.getItem(key);
  if (!id) {
    id = `player:${Math.random().toString(36).slice(2, 10)}`;
    window.sessionStorage.setItem(key, id);
  }
  return id;
}
