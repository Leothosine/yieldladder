"use client";

import React, { useState } from "react";
import EarlyExitModal from "@/components/EarlyExitModal";

export default function PortfolioPage() {
  const [modalOpen, setModalOpen] = useState(false);

  // minimal mock position
  const principal = 500;
  const accruedYield = 12.34;
  const exitFeeRate = 0.0125; // 1.25%
  const remainingDays = 47;

  return (
    <div style={{ padding: 20, fontFamily: "Inter, system-ui" }}>
      <h2>Portfolio</h2>
      <div style={{ border: "1px solid #eee", padding: 12, width: 520 }}>
        <div>Position: 500 USDC in L6</div>
        <div>Accrued yield: 12.34 USDC</div>
        <div style={{ marginTop: 8 }}>
          <button onClick={() => setModalOpen(true)}>Exit Early</button>
        </div>
      </div>

      <EarlyExitModal
        open={modalOpen}
        onClose={() => setModalOpen(false)}
        principal={principal}
        accruedYield={accruedYield}
        exitFeeRate={exitFeeRate}
        remainingDays={remainingDays}
      />
    </div>
  );
}
