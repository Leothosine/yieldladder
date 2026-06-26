'use client';

import { useTVL } from '../hooks/useTVL';
import { useAPY } from '../hooks/useAPY';

function fmt(n: number, decimals = 2): string {
  if (n >= 1_000_000) return `$${(n / 1_000_000).toFixed(decimals)}M`;
  if (n >= 1_000) return `$${(n / 1_000).toFixed(decimals)}K`;
  return `$${n.toFixed(decimals)}`;
}

function StatItem({
  label,
  value,
  loading,
}: {
  label: string;
  value: string;
  loading: boolean;
}) {
  return (
    <div>
      <p style={{ margin: 0, fontSize: '0.85rem', opacity: 0.7 }}>{label}</p>
      <p style={{ margin: 0, fontWeight: 700, fontSize: '1.2rem' }}>
        {loading ? '…' : value}
      </p>
    </div>
  );
}

export default function StatsBar() {
  const { tvl, loading: tvlLoading } = useTVL();
  const { apy: flexApy, loading: apyLoading } = useAPY('Flex');

  const totalTvl = tvl?.totalUsdc ?? 0;
  const vaultCount = tvl?.perTier.filter((t) => t.tvlUsdc > 0).length ?? 0;
  const bestApy = flexApy?.apy7d ?? 0;

  return (
    <div
      style={{
        display: 'flex',
        gap: '2rem',
        flexWrap: 'wrap',
        padding: '1rem',
        background: 'rgba(255,255,255,0.05)',
        borderRadius: '0.5rem',
        marginTop: '1rem',
      }}
    >
      <StatItem
        label="Total Value Locked"
        value={fmt(totalTvl)}
        loading={tvlLoading}
      />
      <StatItem
        label="Active Vaults"
        value={String(vaultCount)}
        loading={tvlLoading}
      />
      <StatItem
        label="Best 7d APY"
        value={`${bestApy.toFixed(2)}%`}
        loading={apyLoading}
      />
    </div>
  );
}
