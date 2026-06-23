"use client";

import React, { useMemo } from "react";
import useAPY from "@/lib/hooks/useAPY";

function fixed7(n: number) {
  return Math.round(n * 1e7) / 1e7;
}

export default function ApySimulator({
  tier,
  amount,
}: {
  tier: string;
  amount: number;
}) {
  const apy = useAPY(tier);

  const result = useMemo(() => {
    const principal = fixed7(amount);
    const projectedYield = fixed7(principal * apy * (1 / 52)); // show 7-day approx -> apy/52
    const total = fixed7(principal + projectedYield);

    // simple share-units: principal * multiplier (derive from tier name)
    const multipliers: Record<string, number> = {
      Flex: 1.0,
      L3: 1.05,
      L6: 1.15,
      L12: 1.4,
    };
    const shares = Math.round(principal * (multipliers[tier] ?? 1.0));

    return { apy, projectedYield, total, shares };
  }, [apy, amount, tier]);

  return (
    <div
      style={{
        borderTop: "1px solid #ddd",
        paddingTop: 12,
        marginTop: 12,
        fontFamily: "Inter, system-ui",
      }}
    >
      <div style={{ color: "#666", fontSize: 13 }}>
        At current 7-day APY ({(result.apy * 100).toFixed(2)}%):
      </div>
      <div style={{ marginTop: 8 }}>
        <div>
          Projected yield: &nbsp;
          <strong>~{result.projectedYield.toFixed(2)} USDC</strong>
        </div>
        <div>
          Total at maturity: &nbsp;
          <strong>~{result.total.toFixed(2)} USDC</strong>
        </div>
        <div>
          Share-units minted: &nbsp;<strong>{result.shares} shares</strong>
        </div>
      </div>
    </div>
  );
}
