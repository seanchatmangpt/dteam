import Link from 'next/link';

export default function TutorialPage() {
  return (
    <main data-testid="help-tutorial">
      <div style={{ opacity: 0.4, letterSpacing: 3, fontSize: 10 }}>TUTORIAL</div>
      <h1 style={{ fontSize: 28, margin: '4px 0 8px 0' }}>
        Your first sixty seconds
      </h1>
      <p style={{ opacity: 0.7, fontSize: 13, marginBottom: 32 }}>
        By the end of this page you will know what every element on screen
        means, what the buttons do, and how to open the two other modes
        (Sprawl MUD and OCEL observer).
      </p>

      <Step
        n={1}
        title="What you&apos;re looking at — an episode scene"
      >
        <p>
          If you clicked a run from the front page, you&apos;re on{' '}
          <code style={codeStyle}>/episode/[slug]</code>. The big blue-white
          sphere is a <strong>64×64 point-cloud representation of a TruthBlock</strong>:
          4,096 surface points, one per cell of a 64³ globe. The globe is the
          world state.
        </p>
        <p style={{ marginTop: 12 }}>
          The <strong>coloured great-circle rings</strong> cutting through the
          sphere are the eight <strong>FieldLanes</strong> — the admission
          gates. Four required-bit lanes (prereq, capability, causality,
          conformance) and four forbidden-bit lanes (law, scenario, risk,
          attention). A lit ring = that lane passed; a dim ring = that lane
          fired a denial.
        </p>
        <p style={{ marginTop: 12, opacity: 0.6, fontSize: 12 }}>
          The header text (e.g. <em>&quot;mlock succeeds; position validated&quot;</em>)
          is a scripted scene annotation for that instant of the motion.
        </p>
      </Step>

      <Step
        n={2}
        title={<>Read the <span style={{ color: '#33cc4d' }}>LAWFUL</span> badge</>}
      >
        <p>
          Top right. It has two states:
        </p>
        <ul style={listStyle}>
          <li>
            <Pill bg="#1a3a1f" fg="#33cc4d">LAWFUL</Pill> — the current motion
            was admitted; both the fast (FNV-1a) and causal (BLAKE3) receipt
            chains match the claimed receipt. This is the deterministic
            consensus verdict.
          </li>
          <li>
            <Pill bg="#3a1a1a" fg="#e63333">UNLAWFUL</Pill> — one or both
            chains diverged. The motion is denied; no state changes.
          </li>
        </ul>
        <p style={{ marginTop: 12 }}>
          The <code style={codeStyle}>fragment: 0x…</code> below the badge is
          the low-64 bits of the causal receipt after this motion. It is the
          &quot;block hash&quot; in the blockchain-MUD framing.
        </p>
      </Step>

      <Step n={3} title="Click RUN — admit another motion">
        <p>
          RUN replays the same scene with the instruction id incremented by
          one. Watch: the fragment value changes, the globe pulses briefly,
          and the Cornell-box ribbon at the bottom appends a tile. The ribbon
          is the receipt-chain history, left-to-right = oldest-to-newest.
        </p>
      </Step>

      <Step n={4} title="Click TAMPER — force a denial">
        <p>
          TAMPER sets a forbidden bit that the law lane rejects. The badge
          flips to <Pill bg="#3a1a1a" fg="#e63333">UNLAWFUL</Pill>, a
          denial-flare burst appears at the offending cell, and the law ring
          goes dim red. The fragment still advances — the chain records the
          denial — but the world state is unchanged.
        </p>
        <p style={{ marginTop: 12, opacity: 0.75, fontSize: 12 }}>
          That&apos;s the point: a denial is <em>evidence</em>, not a crash. The
          ribbon keeps growing.
        </p>
      </Step>

      <Step
        n={5}
        title={<>Now open <Link href="/sprawl" style={linkStyle}>/sprawl</Link></>}
      >
        <p>
          This episode view shows one run at a time. The{' '}
          <Link href="/sprawl" style={linkStyle}>Sprawl MUD</Link> is the
          nine-room quest arc: Case → Molly → Wintermute → 3Jane → Angie →
          Armitage → Corto → Neuromancer → Loa. It auto-plays through
          eighteen turns (one probe + one admit per room) and seals the
          chain at the end. Every Neuromancer character from arenas 50–60
          appears once.
        </p>
      </Step>

      <Step
        n={6}
        title={<>Open <Link href="/ocel" style={linkStyle}>/ocel</Link> in a second tab</>}
      >
        <p>
          This is the observer surface. It tails an OCEL 2.0 event log via
          Server-Sent Events. Each time you advance a turn on <code style={codeStyle}>/sprawl</code>,
          the observer tab lands a new row with <code style={codeStyle}>player</code>,
          <code style={codeStyle}> chain</code>, <code style={codeStyle}>room</code>,
          <code style={codeStyle}> cell</code> relationships.
        </p>
        <p style={{ marginTop: 12 }}>
          Two tabs, one screen = watch yourself play.
        </p>
      </Step>

      <div
        style={{
          marginTop: 40,
          padding: 20,
          background: '#0a1a0f',
          border: '1px solid #1a3a1f',
          borderRadius: 8,
        }}
      >
        <div style={{ color: '#33cc4d', fontSize: 12, letterSpacing: 2 }}>
          ✓ YOU KNOW ENOUGH TO PLAY
        </div>
        <p style={{ marginTop: 8, fontSize: 13, opacity: 0.9 }}>
          Next:{' '}
          <Link href="/help/how-to" style={linkStyle}>how-to</Link> for task
          recipes (regenerate the replay, stream OTel, run the doctor) or{' '}
          <Link href="/help/explanation" style={linkStyle}>explanation</Link>{' '}
          if you want to know <em>why</em> a blockchain MUD is the right
          shape for this.
        </p>
      </div>
    </main>
  );
}

function Step({
  n,
  title,
  children,
}: {
  n: number;
  title: React.ReactNode;
  children: React.ReactNode;
}) {
  return (
    <section
      style={{
        display: 'grid',
        gridTemplateColumns: '36px 1fr',
        gap: 16,
        marginBottom: 28,
      }}
    >
      <div
        style={{
          width: 28,
          height: 28,
          borderRadius: 14,
          background: '#1a2030',
          color: '#4db2ff',
          textAlign: 'center',
          lineHeight: '28px',
          fontSize: 13,
          fontWeight: 700,
        }}
      >
        {n}
      </div>
      <div>
        <h2 style={{ fontSize: 16, margin: 0 }}>{title}</h2>
        <div
          style={{
            marginTop: 8,
            fontSize: 13,
            lineHeight: 1.7,
            opacity: 0.9,
          }}
        >
          {children}
        </div>
      </div>
    </section>
  );
}

function Pill({
  bg,
  fg,
  children,
}: {
  bg: string;
  fg: string;
  children: React.ReactNode;
}) {
  return (
    <span
      style={{
        display: 'inline-block',
        background: bg,
        color: fg,
        padding: '2px 8px',
        borderRadius: 4,
        fontSize: 11,
        letterSpacing: 1,
      }}
    >
      {children}
    </span>
  );
}

const codeStyle: React.CSSProperties = {
  background: '#1a1d28',
  padding: '1px 6px',
  borderRadius: 3,
  fontSize: 11,
};

const listStyle: React.CSSProperties = {
  marginTop: 8,
  paddingLeft: 20,
  display: 'flex',
  flexDirection: 'column',
  gap: 8,
};

const linkStyle: React.CSSProperties = { color: '#4db2ff' };
