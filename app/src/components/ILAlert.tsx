interface PoolIL {
  poolName: string;
  ilPercent: number;
}

const IL_THRESHOLD = 2;

interface Props {
  pools: PoolIL[];
}

/** Amber alert card shown when any pool exceeds the 2% IL threshold. */
export function ILAlert({ pools }: Props) {
  const breaching = pools.filter((p) => p.ilPercent > IL_THRESHOLD);
  if (breaching.length === 0) return null;

  return (
    <div
      role="alert"
      style={{
        background: '#fefce8',
        border: '1px solid #fbbf24',
        borderRadius: 8,
        padding: '0.75rem 1rem',
        display: 'flex',
        flexDirection: 'column',
        gap: '0.25rem',
        fontSize: '0.9rem',
        color: '#92400e',
        marginBottom: '1rem',
      }}
    >
      {breaching.map((p) => (
        <div key={p.poolName} style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
          <span aria-hidden="true">⚠️</span>
          <span>
            {p.poolName} pool IL at <strong>{p.ilPercent.toFixed(1)}%</strong> — approaching
            rebalance threshold.
          </span>
        </div>
      ))}
    </div>
  );
}
