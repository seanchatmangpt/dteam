import type { Metadata } from 'next';
import type { ReactNode } from 'react';

export const metadata: Metadata = {
  title: 'Matrix · Sprawl Trilogy',
  description:
    'Three.js visualisation of unibit arenas — Sprawl-trilogy runs rendered from real branchless admission.',
};

export default function RootLayout({ children }: { children: ReactNode }) {
  return (
    <html lang="en">
      <body
        style={{
          margin: 0,
          background: '#05070d',
          color: '#e8e8f0',
          fontFamily: 'ui-sans-serif, system-ui, -apple-system, sans-serif',
          minHeight: '100vh',
        }}
      >
        {children}
      </body>
    </html>
  );
}
