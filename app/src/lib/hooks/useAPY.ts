"use client";

import { useEffect, useState } from "react";

// Minimal useAPY hook — returns a live APY value per tier.
// In production this should subscribe to live analytics; here it's a simple mapping
// with a small periodic jitter to simulate live updates.
export default function useAPY(tier: string) {
  const base: Record<string, number> = {
    Flex: 0.02,
    L3: 0.035,
    L6: 0.042,
    L12: 0.06,
  };

  const [apy, setApy] = useState<number>(base[tier] ?? 0.03);

  useEffect(() => {
    setApy(base[tier] ?? 0.03);
  }, [tier]);

  // simulate slight live changes every 10s (non-essential, removable)
  useEffect(() => {
    const id = setInterval(() => {
      setApy((a) => {
        const jitter = (Math.random() - 0.5) * 0.001; // ±0.05%
        return Math.max(0, Math.round((a + jitter) * 1e7) / 1e7);
      });
    }, 10000);
    return () => clearInterval(id);
  }, []);

  return apy;
}
