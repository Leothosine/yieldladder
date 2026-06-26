import { useState, useEffect } from 'react';
import { callRpc } from '../services/rpc';

export interface PoolAllocation {
  poolId: string;
  /** Share of total TVL allocated to this pool, 0–100 */
  percentage: number;
  /** Raw allocation amount in USDC stroops */
  amount: string;
}

export interface Allocations {
  pools: PoolAllocation[];
  lastUpdatedLedger: number;
}

const STRATEGY_VAULT = process.env.NEXT_PUBLIC_STRATEGY_VAULT;

export function useAllocations() {
  const [allocations, setAllocations] = useState<Allocations | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;

    async function load() {
      if (!STRATEGY_VAULT) {
        if (!cancelled) {
          setAllocations({ pools: [], lastUpdatedLedger: 0 });
          setLoading(false);
        }
        return;
      }

      try {
        const ledger = await callRpc<{ sequence: number }>('getLatestLedger');
        if (!cancelled) {
          // Will be filled with XDR-decoded allocation data from StrategyVault
          // once the TypeScript SDK (GF-08) is merged.
          setAllocations({
            pools: [],
            lastUpdatedLedger: ledger.sequence,
          });
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

  return { allocations, loading, error };
}
