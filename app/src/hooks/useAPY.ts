import { useState, useEffect } from 'react';
import { callRpc } from '../services/rpc';
import type { Tier } from './usePosition';

export interface APYData {
  tier: Tier;
  /** Trailing 7-day APY as a percentage, e.g. 14.5 means 14.5% */
  apy7d: number;
  /** Trailing 30-day APY as a percentage */
  apy30d: number;
}

const HARVESTER_CONTRACT = process.env.NEXT_PUBLIC_HARVESTER_CONTRACT;

interface HarvestEvent {
  yieldAmount: number;
  tvlAtHarvest: number;
  ledgerTimestamp: number;
}

async function fetchHarvestEvents(limit: number): Promise<HarvestEvent[]> {
  if (!HARVESTER_CONTRACT) return [];

  const res = await callRpc<{ events: { value: string; ledger: number }[] }>(
    'getEvents',
    [
      {
        startLedger: 0,
        filters: [
          {
            type: 'contract',
            contractIds: [HARVESTER_CONTRACT],
            topics: [['harvest']],
          },
        ],
        pagination: { limit },
      },
    ],
  );

  return (res.events ?? []).map((e) => {
    // Decode XDR event value once SDK is available; use 0 as placeholder
    void e.value;
    return {
      yieldAmount: 0,
      tvlAtHarvest: 1,
      ledgerTimestamp: e.ledger * 5, // ~5 s per ledger
    };
  });
}

function annualise(yieldAmount: number, tvl: number, days: number): number {
  if (tvl === 0 || days === 0) return 0;
  return (yieldAmount / tvl) * (365 / days) * 100;
}

export function useAPY(tier: Tier) {
  const [apy, setApy] = useState<APYData | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;

    async function load() {
      try {
        const events = await fetchHarvestEvents(60);

        // Sum yield over the last 7 and 30 days
        const nowSec = Math.floor(Date.now() / 1000);
        const events7d = events.filter(
          (e) => nowSec - e.ledgerTimestamp <= 7 * 86_400,
        );
        const events30d = events.filter(
          (e) => nowSec - e.ledgerTimestamp <= 30 * 86_400,
        );

        const sum = (evts: HarvestEvent[]) =>
          evts.reduce((s, e) => s + e.yieldAmount, 0);
        const avgTvl = (evts: HarvestEvent[]) =>
          evts.length
            ? evts.reduce((s, e) => s + e.tvlAtHarvest, 0) / evts.length
            : 1;

        if (!cancelled) {
          setApy({
            tier,
            apy7d: annualise(sum(events7d), avgTvl(events7d), 7),
            apy30d: annualise(sum(events30d), avgTvl(events30d), 30),
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
    const interval = setInterval(load, 60_000);
    return () => {
      cancelled = true;
      clearInterval(interval);
    };
  }, [tier]);

  return { apy, loading, error };
}
