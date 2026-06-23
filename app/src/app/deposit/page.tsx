"use client";

import React, { useState } from "react";
import ApySimulator from "@/components/ApySimulator";
import CompoundChart from "@/components/CompoundChart";

const TIERS = ["Flex", "L3", "L6", "L12"] as const;

export default function DepositPage() {
  const [tier, setTier] = useState<string>("L6");
  const [amount, setAmount] = useState<number>(500);
  const [step, setStep] = useState<number>(1);

  return (
    <div style={{ padding: 20, fontFamily: "Inter, system-ui" }}>
      <h2>Deposit</h2>
      {step === 1 && (
        <div>
          <label>
            Deposit:{" "}
            <input
              type="number"
              value={amount}
              onChange={(e) => setAmount(Number(e.target.value))}
              style={{ width: 120 }}
            />{" "}
            USDC
          </label>
          <div style={{ marginTop: 8 }}>
            <label>Tier: </label>
            <select value={tier} onChange={(e) => setTier(e.target.value)}>
              {TIERS.map((t) => (
                <option key={t} value={t}>
                  {t}
                </option>
              ))}
            </select>
          </div>

          <ApySimulator tier={tier} amount={amount} />

          <div style={{ marginTop: 12 }}>
            <button onClick={() => setStep(2)}>Next: Amount & Projector</button>
          </div>
        </div>
      )}

      {step === 2 && (
        <div>
          <h3>Projected Growth</h3>
          <div style={{ marginBottom: 8 }}>
            <label>
              Amount:{" "}
              <input
                type="number"
                value={amount}
                onChange={(e) => setAmount(Number(e.target.value))}
                style={{ width: 120 }}
              />{" "}
              USDC
            </label>
          </div>
          <CompoundChart
            tier={tier}
            amount={amount}
            months={
              tier === "L12" ? 12 : tier === "L6" ? 6 : tier === "L3" ? 3 : 1
            }
          />
          <div style={{ marginTop: 12 }}>
            <button onClick={() => setStep(1)}>Back</button>
            <button style={{ marginLeft: 8 }}>Proceed to confirm</button>
          </div>
        </div>
      )}
    </div>
  );
}
