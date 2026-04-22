'use client';

import { MotionResponse } from '@/lib/unibit';

/**
 * Cornell-box ribbon — each fragment a small rectangle in a horizontal
 * strip. Inspired by Gibson's Boxmaker assemblages from Count Zero.
 */
export function ReceiptRibbon({
  history,
}: {
  history: MotionResponse[];
}) {
  return (
    <div
      style={styles.ribbon}
      data-testid="receipt-ribbon"
      data-length={history.length}
    >
      {history.slice(-12).map((r, i) => (
        <div
          key={i}
          data-testid="receipt-fragment"
          data-admitted={r.denyTotal === 0n ? 'true' : 'false'}
          style={{
            ...styles.box,
            background:
              r.denyTotal === 0n
                ? 'rgba(51, 204, 77, 0.4)'
                : 'rgba(230, 51, 51, 0.4)',
            borderColor: r.denyTotal === 0n ? '#33cc4d' : '#e63333',
          }}
          title={`fragment: ${r.fragment.toString(16).padStart(16, '0')}`}
        >
          <div style={styles.fragmentText}>
            {r.fragment.toString(16).slice(0, 4)}
          </div>
        </div>
      ))}
    </div>
  );
}

const styles: Record<string, React.CSSProperties> = {
  ribbon: {
    position: 'absolute',
    bottom: 24,
    left: 24,
    right: 24,
    display: 'flex',
    gap: 4,
    padding: 8,
    background: 'rgba(20, 20, 30, 0.5)',
    borderRadius: 8,
    border: '1px solid #222',
    fontFamily: 'ui-monospace, monospace',
  },
  box: {
    width: 48,
    height: 48,
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    border: '1px solid',
    borderRadius: 4,
  },
  fragmentText: { fontSize: 10, color: '#ddd' },
};
