import Link from 'next/link';

export default function ExplanationPage() {
  return (
    <main data-testid="help-explanation">
      <div style={{ opacity: 0.4, letterSpacing: 3, fontSize: 10 }}>EXPLANATION</div>
      <h1 style={{ fontSize: 28, margin: '4px 0 24px 0' }}>
        Why a blockchain MUD
      </h1>
      <p style={{ opacity: 0.7, fontSize: 13, marginBottom: 24 }}>
        The shortest honest answer: we already had every primitive a
        blockchain MUD needs, living as separate concerns inside the{' '}
        <code style={code}>unibit</code> kernel. The MUD just gives them
        one frame.
      </p>

      <H2>The argument in one paragraph</H2>
      <p style={p}>
        A blockchain is a two-party verifiable hash chain. unibit already
        has one: <code style={code}>DualReceipt</code> is an FNV-1a fast
        chain woven with a BLAKE3 causal chain, replayable by an
        independent verifier. A MUD is a set of rooms, a command
        language, and a turn machine. unibit already has those too:{' '}
        <code style={code}>GlobeCell</code> is rooms (64³ = 262,144
        addressable places), <code style={code}>Activity</code>/<code style={code}>MotionPacket</code>{' '}
        is a compiled command, <code style={code}>UMotion&lt;Pending|Hot|Spent&gt;</code>{' '}
        is the typestate-enforced turn lifecycle. Nothing about the MUD
        required new kernel code.
      </p>

      <H2>What &quot;Lawful&quot; actually proves</H2>
      <p style={p}>
        When the verdict badge is green, the claim is precise: the{' '}
        <code style={code}>MotionEvent</code> tape, replayed from genesis
        by an auditor with no privileged state, produces the two receipt
        heads you see on the page. The FNV-1a chain is fast and cheap;
        the BLAKE3 chain is cryptographic. Agreement across both is the
        two-party consensus — neither channel can quietly lie without the
        other catching it. Tampering with either chain flips the verdict
        to{' '}
        <span style={{ color: '#e6a533' }}>FastOnly</span> or{' '}
        <span style={{ color: '#e6a533' }}>CausalOnly</span>, which is
        what the <code style={code}>TAMPER</code> button demonstrates.
      </p>

      <H2>What the 64³ globe is for</H2>
      <p style={p}>
        The <code style={code}>TruthBlock</code> is 4,096 <code style={code}>u64</code>{' '}
        words = 262,144 bits. <code style={code}>GlobeCell(domain, cell,
        place)</code> addresses those bits as a 64×64×64 lattice. The
        point-cloud sphere on screen is the <em>surface</em> of that
        lattice (4,096 points, 64×64 surface of a 64³ cube, projected
        onto a sphere) so you can see cells light up without rendering a
        quarter-million points. Moving through rooms in the MUD is
        moving through anchor cells on that lattice.
      </p>

      <H2>Why the eight lanes</H2>
      <p style={p}>
        Admission is branchless: each lane computes a denial mask in
        parallel, the masks OR together, zero means admitted, anything
        else means denied. Four lanes (prereq, capability, causality,
        conformance) fire when a required bit is <em>clear</em>. The
        other four (law, scenario, risk, attention) fire when a forbidden
        bit is <em>set</em>. Every denial preserves its lane of origin,
        so a denied motion is still useful evidence — you can see{' '}
        <em>which</em> gate killed the motion, not just that it died.
      </p>

      <H2>Why the two tabs</H2>
      <p style={p}>
        <Link href="/sprawl" style={link}>/sprawl</Link> is first-person:
        you advance the turn. <Link href="/ocel" style={link}>/ocel</Link>{' '}
        is third-person: it receives the turn as an OCEL 2.0 event and
        renders the object graph (player, chain, room, cell). Splitting
        the two tabs makes the <em>process-mining</em> framing real: the
        event log is not a debug surface, it&apos;s the primary artifact.
        The <code style={code}>/sprawl</code> tab is one realisation of
        the process; <code style={code}>/ocel</code> tab is how the
        process is inspected.
      </p>

      <H2>Why Neuromancer</H2>
      <p style={p}>
        The nine rooms in the Sprawl MUD quest arc are the nine
        Neuromancer-themed e2e tests in{' '}
        <code style={code}>unibit-e2e</code> (arenas 50–60). Each
        character pins a specific kinetic mechanism: Case = T0 scalar
        gate, Molly = 128-bit HDC distance, Wintermute = 256-bit consensus,
        3Jane = outcome ranking, Angie = refinement stack, Armitage =
        dual-chain verdict, Corto = negative-knowledge registry,
        Neuromancer = proof-template catalog, Loa = the six-stage capstone
        that composes all of it. The fiction is a mnemonic for the
        mechanism — nothing more, but nothing less.
      </p>

      <H2>What we&apos;re not claiming</H2>
      <p style={p}>
        This is not a cryptocurrency — no balances, no transfers, no
        economic value. It&apos;s not a BFT consensus network — the chain
        has a single author, the verifier is an auditor, not a validator.
        It&apos;s not a multiplayer game — the live-mode WebSocket is
        single-world multi-observer, not concurrent editors. Those are
        deliberate scope cuts; each can be added above the current
        primitives without changing anything below{' '}
        <code style={code}>unibit-sprawl</code>.
      </p>

      <H2>Further reading</H2>
      <ul style={list}>
        <li>
          <code style={code}>docs/hdit/sprawl_mud.md</code> — architecture +
          wire protocol + SP-1…SP-7 invariants
        </li>
        <li>
          <code style={code}>docs/hdit/semantic_to_kinetic_chain.md</code> —
          how <code style={code}>Activity</code> lowers to kinetic gates
        </li>
        <li>
          <code style={code}>docs/hdit/tier_contract.md</code> — T0/T1/T2/T3
          tier budgets
        </li>
        <li>
          <code style={code}>crates/unibit-sprawl/tests/sprawl_suite.rs</code>{' '}
          — twelve integration tests pinning the primitives
        </li>
      </ul>
    </main>
  );
}

function H2({ children }: { children: React.ReactNode }) {
  return (
    <h2 style={{ fontSize: 16, margin: '28px 0 8px 0', letterSpacing: 1 }}>
      {children}
    </h2>
  );
}

const p: React.CSSProperties = {
  fontSize: 13,
  opacity: 0.9,
  lineHeight: 1.75,
  margin: '0 0 12px 0',
};
const code: React.CSSProperties = {
  background: '#1a1d28',
  padding: '1px 6px',
  borderRadius: 3,
  fontSize: 11,
};
const link: React.CSSProperties = { color: '#4db2ff' };
const list: React.CSSProperties = {
  paddingLeft: 22,
  fontSize: 13,
  lineHeight: 1.9,
};
