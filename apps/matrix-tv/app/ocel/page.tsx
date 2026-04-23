'use client';

import Link from 'next/link';
import { useEffect, useMemo, useRef, useState } from 'react';
import type { OcelEvent, OcelObject } from '@/lib/ocel';

/**
 * `/ocel` — live observer for the Sprawl MUD.
 *
 * Opens an SSE connection to `/api/ocel/stream` and renders every
 * `{event, objects}` record the server appends to the log. Watch
 * someone else play — or replay a past session — without touching
 * the /sprawl page itself.
 */
export default function OcelObserverPage() {
  const [events, setEvents] = useState<
    { event: OcelEvent; objects: OcelObject[] }[]
  >([]);
  const [connected, setConnected] = useState(false);
  const listRef = useRef<HTMLOListElement>(null);

  useEffect(() => {
    const es = new EventSource('/api/ocel/stream');
    es.onopen = () => setConnected(true);
    es.onerror = () => setConnected(false);
    es.onmessage = (msg) => {
      try {
        const rec = JSON.parse(msg.data) as {
          event: OcelEvent;
          objects: OcelObject[];
        };
        setEvents((prev) => [...prev, rec]);
      } catch {
        /* malformed line — skip */
      }
    };
    return () => es.close();
  }, []);

  useEffect(() => {
    // auto-scroll to bottom as new events arrive
    const el = listRef.current;
    if (el) el.scrollTop = el.scrollHeight;
  }, [events]);

  const stats = useMemo(() => {
    const players = new Set<string>();
    const rooms = new Map<string, number>();
    const verdicts = new Map<string, number>();
    for (const { event } of events) {
      for (const rel of event.relationships) {
        if (rel.qualifier === 'actor') players.add(rel.objectId);
      }
      const roomRel = event.relationships.find((r) => r.qualifier === 'in_room');
      if (roomRel) {
        rooms.set(roomRel.objectId, (rooms.get(roomRel.objectId) ?? 0) + 1);
      }
      const v = String(event.attributes.verdict ?? 'unknown');
      verdicts.set(v, (verdicts.get(v) ?? 0) + 1);
    }
    return { players, rooms, verdicts };
  }, [events]);

  return (
    <main
      data-testid="ocel-page"
      data-event-count={events.length}
      style={{
        padding: 24,
        minHeight: '100vh',
        background: '#0c0f18',
        color: '#e8e8f0',
        fontFamily: 'ui-monospace, monospace',
      }}
    >
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'baseline' }}>
        <div>
          <div style={{ display: 'flex', gap: 12 }}>
            <Link href="/" style={{ color: '#9ab', fontSize: 12, textDecoration: 'none' }}>
              ← back
            </Link>
            <Link href="/help/tutorial" style={{ color: '#9ab', fontSize: 12, textDecoration: 'none' }}>
              ? help
            </Link>
          </div>
          <h1 style={{ fontSize: 28, margin: '8px 0' }}>
            OCEL · live observer
          </h1>
          <p style={{ opacity: 0.7, fontSize: 13, marginTop: 0 }}>
            Tailing <code>ocel-log.jsonl</code> via SSE. Open <Link href="/sprawl" style={{ color: '#4db2ff' }}>/sprawl</Link>{' '}
            in another tab and play — every turn lands here as an OCEL 2.0 event.
          </p>
        </div>
        <div
          style={{
            padding: '4px 10px',
            borderRadius: 12,
            background: connected ? '#1a3a1f' : '#3a1a1a',
            color: connected ? '#33cc4d' : '#e63333',
            fontSize: 11,
            letterSpacing: 1,
          }}
          data-testid="sse-status"
        >
          {connected ? '● CONNECTED' : '○ DISCONNECTED'}
        </div>
      </div>

      <section
        style={{
          display: 'grid',
          gridTemplateColumns: '1fr 1fr 1fr',
          gap: 12,
          margin: '20px 0',
        }}
      >
        <Panel title="players">
          {stats.players.size === 0 ? (
            <span style={{ opacity: 0.5 }}>waiting…</span>
          ) : (
            [...stats.players].map((p) => (
              <div key={p} style={{ fontSize: 11 }}>
                {p}
              </div>
            ))
          )}
        </Panel>
        <Panel title="rooms visited">
          {[...stats.rooms.entries()].map(([r, n]) => (
            <div key={r} style={{ fontSize: 11, display: 'flex', justifyContent: 'space-between' }}>
              <span>{r}</span>
              <span style={{ opacity: 0.6 }}>×{n}</span>
            </div>
          ))}
        </Panel>
        <Panel title="verdicts">
          {[...stats.verdicts.entries()].map(([v, n]) => (
            <div key={v} style={{ fontSize: 11, display: 'flex', justifyContent: 'space-between' }}>
              <span style={{ color: v === 'Lawful' ? '#33cc4d' : '#e63333' }}>{v}</span>
              <span style={{ opacity: 0.6 }}>×{n}</span>
            </div>
          ))}
        </Panel>
      </section>

      <ol
        ref={listRef}
        data-testid="ocel-event-list"
        style={{
          listStyle: 'none',
          padding: 0,
          margin: 0,
          border: '1px solid #222',
          borderRadius: 8,
          maxHeight: '60vh',
          overflowY: 'auto',
          background: '#0a0c14',
        }}
      >
        {events.length === 0 && (
          <li style={{ padding: 16, opacity: 0.5, fontSize: 12 }}>
            (no events yet — play /sprawl to populate this stream)
          </li>
        )}
        {events.map(({ event }, i) => (
          <li
            key={`${event.id}:${i}`}
            style={{
              padding: '8px 12px',
              borderBottom: '1px solid #1a1d28',
              fontSize: 11,
              display: 'grid',
              gridTemplateColumns: '80px 90px 80px 1fr',
              gap: 12,
              alignItems: 'center',
            }}
          >
            <span style={{ opacity: 0.5 }}>#{event.attributes.tick}</span>
            <span>{event.activity}</span>
            <span
              style={{
                color:
                  event.attributes.verdict === 'Lawful' ? '#33cc4d' : '#e63333',
              }}
            >
              {String(event.attributes.verdict)}
            </span>
            <span style={{ opacity: 0.7, fontFamily: 'ui-monospace, monospace' }}>
              {event.relationships
                .map((r) => `${r.qualifier}=${r.objectId}`)
                .join('  ')}
            </span>
          </li>
        ))}
      </ol>
    </main>
  );
}

function Panel({
  title,
  children,
}: {
  title: string;
  children: React.ReactNode;
}) {
  return (
    <div
      style={{
        padding: 12,
        background: '#0a0c14',
        border: '1px solid #1a1d28',
        borderRadius: 8,
      }}
    >
      <div
        style={{
          fontSize: 10,
          letterSpacing: 2,
          opacity: 0.5,
          marginBottom: 8,
          textTransform: 'uppercase',
        }}
      >
        {title}
      </div>
      {children}
    </div>
  );
}
