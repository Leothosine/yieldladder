"use client";

import React, { useMemo } from "react";
import {
  LineChart,
  Line,
  XAxis,
  YAxis,
  Tooltip,
  Legend,
  ResponsiveContainer,
} from "recharts";

function fixed7(n: number) {
  return Math.round(n * 1e7) / 1e7;
}

export default function CompoundChart({
  tier,
  amount,
  months,
}: {
  tier: string;
  amount: number;
  months: number;
}) {
  // simple multiplier by tier
  const multipliers: Record<string, number> = {
    Flex: 1.0,
    L3: 1.05,
    L6: 1.15,
    L12: 1.4,
  };

  const apyMap: Record<string, number> = {
    Flex: 0.02,
    L3: 0.035,
    L6: 0.042,
    L12: 0.06,
  };

  const data = useMemo(() => {
    const apy = apyMap[tier] ?? 0.03;
    const monthlyRate = apy / 12;
    const rows = [] as any[];
    let simple = fixed7(amount);
    let compound = fixed7(amount);
    for (let m = 0; m <= months; m++) {
      if (m > 0) {
        simple = fixed7(amount + amount * monthlyRate * m);
        compound = fixed7(compound * (1 + monthlyRate));
      }
      rows.push({ month: m, simple: simple, compound: compound });
    }
    return rows;
  }, [tier, amount, months]);

  return (
    <div style={{ width: "100%", height: 260 }}>
      <ResponsiveContainer>
        <LineChart data={data}>
          <XAxis dataKey="month" />
          <YAxis />
          <Tooltip formatter={(v: any) => `${Number(v).toFixed(2)} USDC`} />
          <Legend />
          <Line
            type="monotone"
            dataKey="simple"
            stroke="#8884d8"
            name="Simple"
            dot={false}
          />
          <Line
            type="monotone"
            dataKey="compound"
            stroke="#82ca9d"
            name="Compound"
            dot={false}
          />
        </LineChart>
      </ResponsiveContainer>
    </div>
  );
}
