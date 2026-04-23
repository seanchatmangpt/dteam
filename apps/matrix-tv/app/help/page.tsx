import Link from 'next/link';

const QUADRANTS = [
  {
    href: '/help/tutorial',
    title: 'Tutorial',
    mode: 'learning',
    blurb:
      'Your first sixty seconds. What the globe is, what RUN and TAMPER do, how to open the MUD.',
    cta: 'start here →',
  },
  {
    href: '/help/how-to',
    title: 'How-to',
    mode: 'goal-oriented',
    blurb:
      'Play the Case→Loa arc. Watch someone else play. Regenerate the replay. Stream OTel to a collector.',
    cta: 'task recipes →',
  },
  {
    href: '/help/reference',
    title: 'Reference',
    mode: 'information',
    blurb:
      'Lane colours. Verdict meanings. Every room → character → arena. SprawlEvent and OCEL schemas.',
    cta: 'look it up →',
  },
  {
    href: '/help/explanation',
    title: 'Explanation',
    mode: 'understanding',
    blurb:
      'Why a blockchain MUD. What "Lawful" actually proves. The dual-receipt chain. Why the globe is 64³.',
    cta: 'the why →',
  },
];

export default function HelpHome() {
  return (
    <main data-testid="help-home">
      <h1 style={{ fontSize: 32, margin: '0 0 8px 0', letterSpacing: 1 }}>
        matrix-tv · help
      </h1>
      <p style={{ opacity: 0.7, fontSize: 14, margin: '0 0 32px 0' }}>
        Four doors, organised by what you need right now. If you don&apos;t
        know which door, take <Link href="/help/tutorial" style={{ color: '#4db2ff' }}>tutorial</Link>.
      </p>

      <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 16 }}>
        {QUADRANTS.map((q) => (
          <Link
            key={q.href}
            href={q.href}
            data-testid={`help-card-${q.title.toLowerCase()}`}
            style={{
              display: 'block',
              padding: 20,
              background: '#0a0c14',
              border: '1px solid #1a1d28',
              borderRadius: 10,
              color: 'inherit',
              textDecoration: 'none',
              transition: 'border-color 0.2s',
            }}
          >
            <div
              style={{
                fontSize: 10,
                opacity: 0.4,
                letterSpacing: 3,
                marginBottom: 8,
              }}
            >
              {q.mode.toUpperCase()}
            </div>
            <div style={{ fontSize: 20, marginBottom: 10, fontWeight: 700 }}>
              {q.title}
            </div>
            <p style={{ fontSize: 12, opacity: 0.75, lineHeight: 1.6, margin: 0 }}>
              {q.blurb}
            </p>
            <div style={{ marginTop: 14, color: '#4db2ff', fontSize: 12 }}>
              {q.cta}
            </div>
          </Link>
        ))}
      </div>

      <p
        style={{
          marginTop: 40,
          opacity: 0.5,
          fontSize: 11,
          borderTop: '1px solid #1a1d28',
          paddingTop: 16,
        }}
      >
        This help system follows the{' '}
        <a
          href="https://diataxis.fr/"
          style={{ color: '#9ab' }}
          target="_blank"
          rel="noopener noreferrer"
        >
          Diataxis
        </a>{' '}
        framework: tutorial / how-to / reference / explanation, each a
        different mode of documentation for a different moment of need.
      </p>
    </main>
  );
}
