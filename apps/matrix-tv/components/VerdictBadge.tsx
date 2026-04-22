'use client';

import { MotionResponse } from '@/lib/unibit';

export function VerdictBadge({ response }: { response: MotionResponse | null }) {
  if (!response) {
    return (
      <div style={styles.idle}>
        <div style={styles.label}>IDLE</div>
      </div>
    );
  }
  const admitted = response.denyTotal === 0n;
  return (
    <div style={admitted ? styles.admit : styles.deny}>
      <div style={styles.label}>{admitted ? 'LAWFUL' : 'UNLAWFUL'}</div>
      <div style={styles.small}>
        fragment: {response.fragment.toString(16).padStart(16, '0')}
      </div>
    </div>
  );
}

const styles: Record<string, React.CSSProperties> = {
  idle: {
    position: 'absolute',
    top: 24,
    right: 24,
    background: 'rgba(20, 20, 30, 0.6)',
    color: '#ccc',
    padding: '8px 16px',
    borderRadius: 8,
    fontFamily: 'ui-monospace, monospace',
    border: '1px solid #333',
  },
  admit: {
    position: 'absolute',
    top: 24,
    right: 24,
    background: 'rgba(20, 60, 30, 0.8)',
    color: '#9fffb3',
    padding: '8px 16px',
    borderRadius: 8,
    fontFamily: 'ui-monospace, monospace',
    border: '1px solid #33cc4d',
    boxShadow: '0 0 12px rgba(51, 204, 77, 0.4)',
  },
  deny: {
    position: 'absolute',
    top: 24,
    right: 24,
    background: 'rgba(80, 20, 20, 0.8)',
    color: '#ff9f9f',
    padding: '8px 16px',
    borderRadius: 8,
    fontFamily: 'ui-monospace, monospace',
    border: '1px solid #e63333',
    boxShadow: '0 0 12px rgba(230, 51, 51, 0.4)',
  },
  label: { fontSize: 16, fontWeight: 700, letterSpacing: 2 },
  small: { fontSize: 10, marginTop: 4, opacity: 0.7 },
};
