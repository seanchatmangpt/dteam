import Link from 'next/link';
import { notFound } from 'next/navigation';
import { runBySlug } from '@/lib/runs';
import { EpisodeScene } from './scene';

export default async function EpisodePage(props: {
  params: Promise<{ slug: string }>;
}) {
  const { slug } = await props.params;
  const run = runBySlug(slug);
  if (!run) notFound();

  return (
    <main style={{ padding: 0, position: 'relative', minHeight: '100vh' }}>
      <header
        style={{
          position: 'absolute',
          top: 24,
          left: 24,
          zIndex: 10,
          padding: 16,
          background: 'rgba(12, 15, 24, 0.8)',
          border: '1px solid #222',
          borderRadius: 8,
          maxWidth: 420,
        }}
      >
        <Link
          href="/"
          style={{ color: '#9ab', textDecoration: 'none', fontSize: 12 }}
        >
          ← back
        </Link>
        <div
          style={{
            fontSize: 10,
            opacity: 0.5,
            marginTop: 4,
            letterSpacing: 1,
          }}
        >
          {run.source.toUpperCase()} · {run.id}
        </div>
        <h1 style={{ fontSize: 22, margin: '4px 0 8px 0' }}>{run.title}</h1>
        <div style={{ fontSize: 12, opacity: 0.6 }}>
          Arena: <code>{run.arena}</code>
        </div>
        <div style={{ fontSize: 12, marginTop: 8, opacity: 0.8 }}>
          {run.expected.description}
        </div>
      </header>

      <EpisodeScene run={run} />
    </main>
  );
}
