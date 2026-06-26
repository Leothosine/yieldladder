'use client';

import { useState } from 'react';
import type { Position } from './mockData';
import { TIER_META } from './mockData';
import YieldChart from './YieldChart';
import type { HarvestEvent } from './mockData';

type TxState = 'idle' | 'pending' | 'confirmed' | 'failed';

interface PositionCardProps {
  position: Position;
  harvestHistory: HarvestEvent[];
}

function formatCountdown(lockUntil: number): string {
  const diff = lockUntil - Math.floor(Date.now() / 1000);
  if (diff <= 0) return 'Unlocked';
  const days = Math.floor(diff / 86400);
  const date = new Date(lockUntil * 1000).toLocaleDateString('en-US', {
    year: 'numeric', month: 'short', day: 'numeric',
  });
  return `Expires in ${days} day${days !== 1 ? 's' : ''} (${date})`;
}

export default function PositionCard({ position, harvestHistory }: PositionCardProps) {
  const { tier, principal, shares, accruedYield, lockUntil } = position;
  const meta = TIER_META[tier];
  const now = Math.floor(Date.now() / 1000);
  const isUnlocked = lockUntil === null || now >= lockUntil;
  const feeAmt = (parseFloat(principal) * meta.earlyExitFee).toFixed(2);
  const netAmt = (parseFloat(principal) - parseFloat(feeAmt) + parseFloat(accruedYield)).toFixed(2);

  const [withdrawState, setWithdrawState] = useState<TxState>('idle');
  const [exitState, setExitState] = useState<TxState>('idle');
  const [showExitModal, setShowExitModal] = useState(false);

  function handleWithdraw() {
    setWithdrawState('pending');
    // sdk.withdraw({ tier }) would go here
    setTimeout(() => setWithdrawState('confirmed'), 1500);
  }

  function handleEarlyExit() {
    setExitState('pending');
    setShowExitModal(false);
    // sdk.earlyExit({ tier }) would go here
    setTimeout(() => setExitState('confirmed'), 1500);
  }

  const lockLabel = lockUntil === null ? 'Unlocked (Flex)' : formatCountdown(lockUntil);

  return (
    <>
      <div style={s.card}>
        <div style={s.cardTop}>
          <div>
            <span style={s.tierBadge}>{tier}</span>
            <span style={s.mult}>{meta.multiplier}</span>
          </div>
          <span style={{ fontSize: '0.78rem', color: isUnlocked ? '#22c55e' : '#94a3b8' }}>
            {lockLabel}
          </span>
        </div>

        <dl style={s.dl}>
          <Row label="Principal" value={`${principal} USDC`} />
          <Row label="Shares" value={shares} />
          <Row label="Accrued yield" value={`${accruedYield} USDC`} highlight />
        </dl>

        <div style={{ marginTop: '0.75rem' }}>
          <p style={s.chartLabel}>Cumulative yield</p>
          <YieldChart data={harvestHistory} tier={tier} />
        </div>

        <div style={s.actions}>
          <button
            style={{ ...s.btn, ...(isUnlocked ? s.btnPrimary : s.btnDisabled) }}
            disabled={!isUnlocked || withdrawState === 'pending'}
            onClick={handleWithdraw}
            type="button"
          >
            {withdrawState === 'pending' ? 'Withdrawing…' : withdrawState === 'confirmed' ? '✓ Withdrawn' : 'Withdraw'}
          </button>

          {tier !== 'Flex' && (
            <button
              style={{ ...s.btn, ...s.btnDanger }}
              disabled={exitState === 'pending'}
              onClick={() => setShowExitModal(true)}
              type="button"
            >
              {exitState === 'pending' ? 'Processing…' : exitState === 'confirmed' ? '✓ Exited' : 'Early Exit'}
            </button>
          )}
        </div>

        {withdrawState === 'failed' && <p style={s.errText}>Withdraw failed. Try again.</p>}
        {exitState === 'failed' && <p style={s.errText}>Early exit failed. Try again.</p>}
      </div>

      {showExitModal && (
        <div style={s.overlay} role="dialog" aria-modal="true" aria-label="Early exit confirmation">
          <div style={s.modal}>
            <h3 style={s.modalTitle}>Confirm Early Exit — {tier}</h3>
            <dl style={s.dl}>
              <Row label="Principal" value={`${principal} USDC`} />
              <Row label={`Exit fee (${(meta.earlyExitFee * 100).toFixed(2)}%)`} value={`−${feeAmt} USDC`} />
              <Row label="Accrued yield" value={`+${accruedYield} USDC`} />
              <Row label="You receive" value={`${netAmt} USDC`} highlight />
            </dl>
            <p style={s.modalNote}>
              The exit fee is redistributed to remaining depositors in this tier, not the protocol.
            </p>
            <div style={s.modalActions}>
              <button style={{ ...s.btn, ...s.btnDanger }} onClick={handleEarlyExit} type="button">
                Confirm Exit
              </button>
              <button style={{ ...s.btn, ...s.btnSecondary }} onClick={() => setShowExitModal(false)} type="button">
                Cancel
              </button>
            </div>
          </div>
        </div>
      )}
    </>
  );
}

function Row({ label, value, highlight }: { label: string; value: string; highlight?: boolean }) {
  return (
    <div style={{ display: 'flex', justifyContent: 'space-between', padding: '2px 0' }}>
      <dt style={{ color: '#94a3b8', fontSize: '0.82rem' }}>{label}</dt>
      <dd style={{ margin: 0, fontSize: '0.82rem', fontWeight: highlight ? 700 : 500, color: highlight ? '#22c55e' : '#f1f5f9' }}>
        {value}
      </dd>
    </div>
  );
}

const s = {
  card: {
    background: '#0f172a',
    border: '1px solid #1e293b',
    borderRadius: 10,
    padding: '1.25rem',
    display: 'flex',
    flexDirection: 'column' as const,
    gap: '0.75rem',
  },
  cardTop: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'flex-start',
    flexWrap: 'wrap' as const,
    gap: '0.25rem',
  },
  tierBadge: {
    fontWeight: 700,
    fontSize: '1rem',
    marginRight: '0.5rem',
  },
  mult: {
    fontSize: '0.78rem',
    background: '#14532d',
    color: '#4ade80',
    padding: '2px 8px',
    borderRadius: 12,
    fontWeight: 600,
  },
  dl: { margin: 0, display: 'flex', flexDirection: 'column' as const, gap: '2px' },
  chartLabel: { fontSize: '0.75rem', color: '#64748b', marginBottom: '0.25rem' },
  actions: { display: 'flex', gap: '0.5rem', flexWrap: 'wrap' as const },
  btn: {
    padding: '0.45rem 1rem',
    borderRadius: 6,
    border: 'none',
    fontWeight: 600,
    fontSize: '0.82rem',
    cursor: 'pointer',
  },
  btnPrimary: { background: '#1d4ed8', color: '#fff' },
  btnDisabled: { background: '#1e293b', color: '#64748b', cursor: 'not-allowed' as const },
  btnDanger: { background: '#7f1d1d', color: '#fca5a5', cursor: 'pointer' as const },
  btnSecondary: { background: '#1e293b', color: '#cbd5e1', cursor: 'pointer' as const },
  errText: { fontSize: '0.78rem', color: '#f87171' },
  overlay: {
    position: 'fixed' as const,
    inset: 0,
    background: 'rgba(0,0,0,0.6)',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    zIndex: 50,
  },
  modal: {
    background: '#0f172a',
    border: '1px solid #1e293b',
    borderRadius: 12,
    padding: '1.5rem',
    width: 'min(400px, 92vw)',
    display: 'flex',
    flexDirection: 'column' as const,
    gap: '1rem',
  },
  modalTitle: { fontWeight: 700, fontSize: '1rem' },
  modalNote: { fontSize: '0.78rem', color: '#94a3b8' },
  modalActions: { display: 'flex', gap: '0.5rem' },
} satisfies Record<string, React.CSSProperties | Record<string, React.CSSProperties>>;
