"use client";

import React, { useMemo } from "react";

function fixed7(n: number) {
  return Math.round(n * 1e7) / 1e7;
}

export default function EarlyExitModal({
  open,
  onClose,
  principal,
  accruedYield,
  exitFeeRate,
  remainingDays,
}: {
  open: boolean;
  onClose: () => void;
  principal: number;
  accruedYield: number;
  exitFeeRate: number; // e.g., 0.0125
  remainingDays: number;
}) {
  const calc = useMemo(() => {
    const p = fixed7(principal);
    const y = fixed7(accruedYield);
    const fee = fixed7(p * exitFeeRate);
    const net = fixed7(p + y - fee);
    const ifMature = fixed7(p + y + 0); // assume current accrued yield is total at maturity in this minimal impl
    const opportunity = fixed7(ifMature - net);
    return { p, y, fee, net, ifMature, opportunity };
  }, [principal, accruedYield, exitFeeRate]);

  if (!open) return null;

  return (
    <div
      style={{
        position: "fixed",
        inset: 0,
        background: "rgba(0,0,0,0.4)",
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
      }}
    >
      <div
        style={{ background: "#fff", padding: 20, width: 480, borderRadius: 8 }}
      >
        <h3>Confirm Early Exit</h3>
        <div style={{ borderTop: "1px solid #eee", paddingTop: 12 }}>
          <div>
            Current principal: <strong>{calc.p.toFixed(2)} USDC</strong>
          </div>
          <div>
            Accrued yield: <strong>+{calc.y.toFixed(2)} USDC</strong>
          </div>
          <div>
            Early exit fee ({(exitFeeRate * 100).toFixed(2)}%):{" "}
            <strong>-{calc.fee.toFixed(2)} USDC</strong>
          </div>
          <div
            style={{ borderTop: "1px solid #ddd", marginTop: 8, paddingTop: 8 }}
          >
            You will receive: <strong>{calc.net.toFixed(2)} USDC</strong>
          </div>
          <div style={{ marginTop: 8 }}>
            Remaining lock: <strong>{remainingDays} days</strong>
          </div>
          <div>
            If you wait until maturity, you receive:{" "}
            <strong>~{calc.ifMature.toFixed(2)} USDC</strong>
          </div>
          <div>
            Opportunity cost of exiting now:{" "}
            <strong>~{calc.opportunity.toFixed(2)} USDC</strong>
          </div>
        </div>
        <div
          style={{
            display: "flex",
            justifyContent: "flex-end",
            gap: 8,
            marginTop: 12,
          }}
        >
          <button onClick={onClose}>Cancel</button>
          <button
            style={{
              background: "#c33",
              color: "#fff",
              border: "none",
              padding: "6px 10px",
              borderRadius: 4,
            }}
          >
            Exit Early
          </button>
        </div>
      </div>
    </div>
  );
}
