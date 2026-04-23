import { LANE_HUE, LANES, LANE_IS_REQUIRED } from '@/lib/unibit';

const ROOMS = [
  { name: 'case', arena: 50, character: 'Case', mechanism: 'T0 scalar prereq / forbidden gate' },
  { name: 'molly', arena: 51, character: 'Molly', mechanism: 'T1/128-bit HDC distance threshold' },
  { name: 'wintermute', arena: 52, character: 'Wintermute', mechanism: 'T1/256-bit multi-word consensus' },
  { name: 'three_jane', arena: 55, character: '3Jane', mechanism: 'outcome ranking' },
  { name: 'angie', arena: 56, character: 'Angie', mechanism: 'refinement stack cascade' },
  { name: 'armitage', arena: 57, character: 'Armitage', mechanism: 'dual-chain verdict' },
  { name: 'corto', arena: 58, character: 'Corto', mechanism: 'negative-knowledge registry' },
  { name: 'neuromancer', arena: 59, character: 'Neuromancer', mechanism: 'proof-template catalog' },
  { name: 'loa', arena: 60, character: 'Loa', mechanism: 'six-stage capstone' },
];

const VERDICTS = [
  { v: 'Lawful', colour: '#33cc4d', meaning: 'both FNV-1a and BLAKE3 chains match the claimed receipt. Admitted.' },
  { v: 'FastOnly', colour: '#e6a533', meaning: 'FNV-1a ok but BLAKE3 diverged. Suspicious — treat as denied.' },
  { v: 'CausalOnly', colour: '#e6a533', meaning: 'BLAKE3 ok but FNV-1a diverged. Similarly denied.' },
  { v: 'Unlawful', colour: '#e63333', meaning: 'both chains diverged. Denied. No state change.' },
];

export default function ReferencePage() {
  return (
    <main data-testid="help-reference">
      <div style={{ opacity: 0.4, letterSpacing: 3, fontSize: 10 }}>REFERENCE</div>
      <h1 style={{ fontSize: 28, margin: '4px 0 24px 0' }}>Lookup tables</h1>
      <p style={{ opacity: 0.7, fontSize: 13, marginBottom: 24 }}>
        Raw facts, no narrative. If you&apos;re trying to understand what a
        colour / verdict / field means mid-session, you&apos;re on the right
        page.
      </p>

      <H2>FieldLane palette</H2>
      <p style={p}>
        The eight great-circle rings on the globe. Four require a bit to
        be set, four forbid a bit.
      </p>
      <Table
        head={['ring', 'lane', 'polarity', 'denies when']}
        rows={LANES.map((l) => [
          <Swatch key={l} colour={LANE_HUE[l]} />,
          <code key="n" style={code}>{l}</code>,
          LANE_IS_REQUIRED[l] ? 'required' : 'forbidden',
          LANE_IS_REQUIRED[l]
            ? 'the required bit is clear'
            : 'the forbidden bit is set',
        ])}
      />

      <H2>Verdicts</H2>
      <Table
        head={['verdict', 'meaning']}
        rows={VERDICTS.map((v) => [
          <span key="v" style={{ color: v.colour, fontWeight: 700 }}>{v.v}</span>,
          v.meaning,
        ])}
      />

      <H2>Room → character → kinetic mechanism</H2>
      <Table
        head={['room', 'arena', 'character', 'mechanism']}
        rows={ROOMS.map((r) => [
          <code key="n" style={code}>{r.name}</code>,
          <code key="a" style={code}>{r.arena}</code>,
          r.character,
          <span key="m" style={{ fontSize: 12, opacity: 0.85 }}>{r.mechanism}</span>,
        ])}
      />

      <H2><code style={code}>SprawlEvent</code> wire format</H2>
      <pre style={pre}>{`{
  tick: number,                    // monotonic turn counter
  room: string,                    // "case" | "molly" | ... | "loa"
  verb: string,                    // "look" | "probe" | "admit" | "move" | "use" | "seal"
  verdict: "Lawful" | "FastOnly" | "CausalOnly" | "Unlawful",
  scope: number,                   // TruthBlock word index, 0..=4095
  before_cell: number,             // u64 before the turn (raw bits)
  after_cell: number,              // u64 after the turn; equals before on denial
  lane_denies: number[8],          // per-lane denial accumulator
  delta_popcount: number,          // bits changed, 0 on denial
  receipt_fast: number,            // FNV-1a receipt tail
  receipt_causal_prefix: number[16]// first 16 octets of the BLAKE3 causal receipt
}`}</pre>

      <H2><code style={code}>OCEL 2.0</code> envelope (one per turn)</H2>
      <pre style={pre}>{`{
  event: {
    id: "<player>:<tick>",         // unique per session per turn
    activity: "<verb>",            // same as SprawlEvent.verb
    timestamp: "<ISO-8601>",
    attributes: { tick, verdict, scope, delta_popcount,
                  receipt_fast, receipt_causal_prefix,
                  before_cell, after_cell },
    relationships: [
      { objectId: "<playerId>",    qualifier: "actor" },
      { objectId: "chain:<pid>",   qualifier: "advances" },
      { objectId: "room:<name>",   qualifier: "in_room" },
      { objectId: "cell:<scope>",  qualifier: "touches" },
    ],
  },
  objects: [ { id, type, attributes }, ... ]
    // object types: player | chain | room | cell
}`}</pre>

      <H2>Ports</H2>
      <Table
        head={['port', 'role']}
        rows={[
          ['31337', 'matrix-tv dev server (our Next.js app)'],
          ['3000', 'Grafana (typical squatter — we avoid this port)'],
          ['4318', 'OTLP/HTTP collector (optional, for OTel traces)'],
          ['4317', 'OTLP/gRPC collector (same role, different transport)'],
          ['8088', 'unibit-sprawl WebSocket (live mode, not wired yet)'],
          ['3412', 'Playwright test server (separate from dev)'],
        ]}
      />

      <H2>Files</H2>
      <Table
        head={['path', 'what']}
        rows={[
          [<code key="1" style={code}>public/sprawl-replay.ndjson</code>, '18-line ndjson, Case→Loa turn stream'],
          [<code key="2" style={code}>ocel-log.jsonl</code>, 'session OCEL log (gitignored)'],
          [<code key="3" style={code}>lib/sprawl.ts</code>, 'SprawlEvent type + replay loader'],
          [<code key="4" style={code}>lib/ocel.ts</code>, 'OCEL 2.0 emitter; calls emitTurn per turn'],
          [<code key="5" style={code}>lib/otel.ts</code>, 'OpenTelemetry bootstrap'],
          [<code key="6" style={code}>cli/</code>, 'matrix-tv-doctor citty CLI'],
        ]}
      />
    </main>
  );
}

function H2({ children }: { children: React.ReactNode }) {
  return <h2 style={{ fontSize: 16, margin: '28px 0 8px 0', letterSpacing: 1 }}>{children}</h2>;
}

function Swatch({ colour }: { colour: string }) {
  return (
    <span
      style={{
        display: 'inline-block',
        width: 24,
        height: 10,
        borderRadius: 2,
        background: colour,
      }}
    />
  );
}

function Table({
  head,
  rows,
}: {
  head: string[];
  rows: React.ReactNode[][];
}) {
  return (
    <div
      style={{
        border: '1px solid #1a1d28',
        borderRadius: 8,
        overflow: 'hidden',
        marginTop: 8,
        fontSize: 12,
      }}
    >
      <div
        style={{
          display: 'grid',
          gridTemplateColumns: `repeat(${head.length}, 1fr)`,
          background: '#0a0c14',
          padding: '8px 12px',
          opacity: 0.5,
          letterSpacing: 2,
          fontSize: 10,
        }}
      >
        {head.map((h) => (
          <div key={h}>{h.toUpperCase()}</div>
        ))}
      </div>
      {rows.map((row, i) => (
        <div
          key={i}
          style={{
            display: 'grid',
            gridTemplateColumns: `repeat(${head.length}, 1fr)`,
            padding: '10px 12px',
            borderTop: '1px solid #1a1d28',
            alignItems: 'center',
          }}
        >
          {row.map((cell, j) => (
            <div key={j}>{cell}</div>
          ))}
        </div>
      ))}
    </div>
  );
}

const p: React.CSSProperties = { fontSize: 13, opacity: 0.85, margin: '0 0 12px 0' };
const pre: React.CSSProperties = {
  marginTop: 8,
  padding: 14,
  background: '#0a0c14',
  border: '1px solid #1a1d28',
  borderRadius: 6,
  fontSize: 11,
  overflow: 'auto',
  lineHeight: 1.5,
};
const code: React.CSSProperties = {
  background: '#1a1d28',
  padding: '1px 6px',
  borderRadius: 3,
  fontSize: 11,
};
