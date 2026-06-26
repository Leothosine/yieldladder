import { useState, useEffect } from 'react';
import { callRpc } from '../services/rpc';

export type Tier = 'Flex' | 'L3' | 'L6' | 'L12';

export interface Position {
  tier: Tier;
  /** Raw share balance returned by the vault contract */
  shares: string;
  /** Principal in USDC stroops (shares / multiplier for locked tiers) */
  principal: string;
  /** Accrued yield estimate in USDC stroops */
  accruedYield: string;
  /** Ledger timestamp when the lock expires, null for Flex */
  lockUntil: number | null;
}

const VAULT_CONTRACT: Record<Tier, string | undefined> = {
  Flex: process.env.NEXT_PUBLIC_FLEX_VAULT,
  L3: process.env.NEXT_PUBLIC_L3_VAULT,
  L6: process.env.NEXT_PUBLIC_L6_VAULT,
  L12: process.env.NEXT_PUBLIC_L12_VAULT,
};

export function usePosition(address: string | null, tier: Tier) {
  const [position, setPosition] = useState<Position | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!address) {
      setPosition(null);
      return;
    }

    const contractId = VAULT_CONTRACT[tier];
    if (!contractId) {
      setError(`Contract address for ${tier} vault not configured`);
      return;
    }

    let cancelled = false;
    setLoading(true);
    setError(null);

    async function load() {
      try {
        // simulateTransaction would be used here with the TypeScript SDK.
        // For now, query contract data via getLedgerEntries using the
        // user's position storage key derived from their address.
        const result = await callRpc<{ entries: unknown[] }>(
          'getLedgerEntries',
          [{ keys: [`${contractId}:position:${address}`] }],
        );
        if (!cancelled) {
          // Parse XDR result once SDK is available; return zeros until then.
          void result;
          setPosition({
            tier,
            shares: '0',
            principal: '0',
            accruedYield: '0',
            lockUntil: null,
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
    return () => { cancelled = true; };
  }, [address, tier]);

  return { position, loading, error };
}
