import { useState, useEffect } from 'react';
import { callRpc } from '../services/rpc';
import type { Tier } from './usePosition';

export interface TierTVL {
  tier: Tier;
  /** TVL in USDC, normalised for the vault's share multiplier */
  tvlUsdc: number;
  totalShares: string;
}

export interface TVLData {
  perTier: TierTVL[];
  totalUsdc: number;
}

const TIER_MULTIPLIERS: Record<Tier, number> = {
  Flex: 1.0,
  L3: 1.05,
  L6: 1.15,
  L12: 1.35,
};

const VAULT_CONTRACTS: [Tier, string | undefined][] = [
  ['Flex', process.env.NEXT_PUBLIC_FLEX_VAULT],
  ['L3', process.env.NEXT_PUBLIC_L3_VAULT],
  ['L6', process.env.NEXT_PUBLIC_L6_VAULT],
  ['L12', process.env.NEXT_PUBLIC_L12_VAULT],
];

async function fetchTierShares(contractId: string): Promise<string> {
  // Reads the total_shares storage slot via getLedgerEntries.
  // Returns the raw XDR value; decoded to a BigInt by the caller.
  const res = await callRpc<{ entries: { xdr: string }[] }>(
    'getLedgerEntries',
    [{ keys: [`${contractId}:total_shares`] }],
  );
  return res.entries?.[0]?.xdr ?? '0';
}

export function useTVL() {
  const [tvl, setTvl] = useState<TVLData | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;

    async function load() {
      try {
        const results = await Promise.allSettled(
          VAULT_CONTRACTS.map(async ([tier, contractId]) => {
            const shares = contractId
              ? await fetchTierShares(contractId)
              : '0';
            // Stroops → USDC: divide by 1e7, then reverse share inflation
            const sharesNum = parseInt(shares, 10) || 0;
            const tvlUsdc = sharesNum / 1e7 / TIER_MULTIPLIERS[tier];
            return { tier, tvlUsdc, totalShares: shares } satisfies TierTVL;
          }),
        );

        const perTier: TierTVL[] = results.map((r, i) =>
          r.status === 'fulfilled'
            ? r.value
            : { tier: VAULT_CONTRACTS[i][0], tvlUsdc: 0, totalShares: '0' },
        );

        const totalUsdc = perTier.reduce((s, t) => s + t.tvlUsdc, 0);

        if (!cancelled) {
          setTvl({ perTier, totalUsdc });
          setLoading(false);
        }
      } catch (e) {
        if (!cancelled) {
          setError(e instanceof Error ? e.message : String(e));
          setLoading(false);
        }
      }
    }

    load();
    const interval = setInterval(load, 30_000);
    return () => {
      cancelled = true;
      clearInterval(interval);
    };
  }, []);

  return { tvl, loading, error };
}
