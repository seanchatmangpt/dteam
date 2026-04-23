import Link from 'next/link';
import type { ReactNode } from 'react';

const NAV = [
  { href: '/help', label: 'overview' },
  { href: '/help/tutorial', label: 'tutorial', hint: 'start here' },
  { href: '/help/how-to', label: 'how-to', hint: 'task recipes' },
  { href: '/help/reference', label: 'reference', hint: 'lookup' },
  { href: '/help/explanation', label: 'explanation', hint: 'the why' },
];

export default function HelpLayout({ children }: { children: ReactNode }) {
  return (
    <div
      style={{
        minHeight: '100vh',
        background: '#0c0f18',
        color: '#e8e8f0',
        fontFamily: 'ui-monospace, monospace',
      }}
    >
      <nav
        style={{
          borderBottom: '1px solid #222',
          padding: '16px 32px',
          display: 'flex',
          gap: 24,
          alignItems: 'baseline',
          flexWrap: 'wrap',
        }}
      >
        <Link
          href="/"
          style={{
            color: '#9ab',
            textDecoration: 'none',
            fontSize: 12,
            letterSpacing: 2,
          }}
        >
          ← MATRIX-TV
        </Link>
        <div style={{ display: 'flex', gap: 20, flexWrap: 'wrap' }}>
          {NAV.map((n) => (
            <Link
              key={n.href}
              href={n.href}
              style={{
                color: '#e8e8f0',
                textDecoration: 'none',
                fontSize: 13,
                letterSpacing: 1,
              }}
            >
              {n.label}
              {n.hint && (
                <span style={{ opacity: 0.4, marginLeft: 6, fontSize: 10 }}>
                  {n.hint}
                </span>
              )}
            </Link>
          ))}
        </div>
      </nav>
      <div style={{ padding: '32px 48px', maxWidth: 820 }}>{children}</div>
    </div>
  );
}
