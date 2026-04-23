import Link from 'next/link';

export default function HowToPage() {
  return (
    <main data-testid="help-how-to">
      <div style={{ opacity: 0.4, letterSpacing: 3, fontSize: 10 }}>HOW-TO</div>
      <h1 style={{ fontSize: 28, margin: '4px 0 24px 0' }}>Task recipes</h1>
      <p style={{ opacity: 0.7, fontSize: 13, marginBottom: 24 }}>
        You already know <em>what</em> things are
        (see <Link href="/help/tutorial" style={link}>tutorial</Link>) or <em>why</em>{' '}
        (see <Link href="/help/explanation" style={link}>explanation</Link>).
        This page answers <em>how do I</em>. Every recipe is self-contained.
      </p>

      <Recipe id="how-play-sprawl" title="Play the full Sprawl MUD arc">
        <ol style={ol}>
          <li>
            Open <Link href="/sprawl" style={link}>/sprawl</Link>.
          </li>
          <li>
            The Case → Loa quest auto-plays at ~1 Hz. Eighteen turns total
            (one probe + one admit per room).
          </li>
          <li>
            To pause or scrub: use the controls in the top-left HUD. The
            chain verifies as Lawful once all 18 turns land.
          </li>
          <li>
            At the end you&apos;ll see <em>✓ chain sealed — 18 turns, all Lawful</em>.
          </li>
        </ol>
      </Recipe>

      <Recipe id="how-watch" title="Watch someone else (or yourself) play">
        <ol style={ol}>
          <li>Keep <Link href="/sprawl" style={link}>/sprawl</Link> open in one tab.</li>
          <li>
            Open <Link href="/ocel" style={link}>/ocel</Link> in another tab
            (same browser, same machine).
          </li>
          <li>
            The observer tails <code style={code}>ocel-log.jsonl</code> via
            SSE. Every turn emitted on /sprawl lands as an OCEL 2.0 event
            row — tick, room, verdict, relationships.
          </li>
          <li>
            The stats panel (top) aggregates players / rooms / verdicts as
            rows arrive.
          </li>
        </ol>
      </Recipe>

      <Recipe id="how-tail-terminal" title="Tail the OCEL log in a terminal">
        <pre style={pre}>
{`cd ~/dteam/apps/matrix-tv
npm run tail`}
        </pre>
        <p style={p}>
          Prints one coloured line per turn: <code style={code}>tick  room  verb  verdict  scope</code>.
          Green = Lawful, red = Unlawful. Use <code style={code}>npm run ocel:reset</code> to clear
          the log before a fresh play-through, and <code style={code}>npm run ocel:stats</code> for
          an aggregate.
        </p>
      </Recipe>

      <Recipe id="how-regen" title="Regenerate the replay file">
        <pre style={pre}>
{`# from anywhere
cd ~/unibit && make sprawl-replay

# or from matrix-tv
cd ~/dteam/apps/matrix-tv && npm run replay`}
        </pre>
        <p style={p}>
          Both forms shell to{' '}
          <code style={code}>cargo run -p unibit-sprawl --bin sprawl -- walk</code>{' '}
          and write the 18-line ndjson to{' '}
          <code style={code}>apps/matrix-tv/public/sprawl-replay.ndjson</code>.
          The /sprawl page fetches it on load.
        </p>
      </Recipe>

      <Recipe id="how-otel" title="Stream OTel traces to a collector">
        <p style={p}>
          The <code style={code}>/sprawl</code> page opens a client-side
          OpenTelemetry tracer on every turn. By default it POSTs to{' '}
          <code style={code}>http://localhost:4318/v1/traces</code>. If no
          collector is listening the exporter fails silently.
        </p>
        <ol style={ol}>
          <li>
            Start an OTLP/HTTP collector on port 4318. Quickest option is
            Jaeger: <code style={code}>docker run -p 16686:16686 -p
            4318:4318 jaegertracing/all-in-one:latest</code>. Then open{' '}
            <code style={code}>http://localhost:16686</code> for the Jaeger UI.
          </li>
          <li>
            Override the exporter endpoint with{' '}
            <code style={code}>NEXT_PUBLIC_OTLP_TRACES_URL</code> before{' '}
            <code style={code}>npm run dev</code> if your collector lives
            elsewhere.
          </li>
          <li>
            Confirm it&apos;s reachable: <code style={code}>npm run doctor</code>{' '}
            shows <em>OTLP collector on :4318</em> → ok.
          </li>
        </ol>
      </Recipe>

      <Recipe id="how-doctor" title="Run the doctor">
        <pre style={pre}>
{`cd ~/dteam/apps/matrix-tv
npm run doctor         # coloured summary
npm run doctor:json    # machine-readable
npm run ports          # just the port table`}
        </pre>
        <p style={p}>
          Thirteen health checks: Node version, installed deps, workspace
          lockfile hygiene, the replay file&apos;s shape, OCEL log writability,
          port :31337, Grafana on :3000, GET /sprawl, POST /api/ocel/log
          round-trip, SSE handshake, OTLP collector, cargo on PATH, unibit
          repo discoverable. Exits 1 on any MISS.
        </p>
      </Recipe>

      <Recipe id="how-open-browsers" title="Open both tabs in one command">
        <pre style={pre}>
{`cd ~/dteam/apps/matrix-tv
npm run play`}
        </pre>
        <p style={p}>
          Opens <code style={code}>/sprawl</code> and{' '}
          <code style={code}>/ocel</code> in the default browser via{' '}
          <code style={code}>open(1)</code>. Add{' '}
          <code style={code}>-- --no-observer</code> to open just the
          player.
        </p>
      </Recipe>
    </main>
  );
}

function Recipe({
  id,
  title,
  children,
}: {
  id: string;
  title: string;
  children: React.ReactNode;
}) {
  return (
    <section
      id={id}
      data-testid={`recipe-${id}`}
      style={{
        marginBottom: 28,
        paddingBottom: 20,
        borderBottom: '1px solid #1a1d28',
      }}
    >
      <h2 style={{ fontSize: 17, margin: '0 0 12px 0' }}>{title}</h2>
      <div style={{ fontSize: 13, lineHeight: 1.7, opacity: 0.9 }}>{children}</div>
    </section>
  );
}

const ol: React.CSSProperties = {
  paddingLeft: 24,
  display: 'flex',
  flexDirection: 'column',
  gap: 6,
  margin: 0,
};
const p: React.CSSProperties = { margin: '10px 0 0 0' };
const pre: React.CSSProperties = {
  marginTop: 10,
  padding: 12,
  background: '#0a0c14',
  border: '1px solid #1a1d28',
  borderRadius: 6,
  fontSize: 12,
  overflow: 'auto',
};
const code: React.CSSProperties = {
  background: '#1a1d28',
  padding: '1px 6px',
  borderRadius: 3,
  fontSize: 11,
};
const link: React.CSSProperties = { color: '#4db2ff' };
