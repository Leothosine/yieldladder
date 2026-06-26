'use client';

import { useState, useEffect, Suspense } from 'react';
import { useSearchParams } from 'next/navigation';
import Link from 'next/link';

const TIERS = [
  { id: 'flex', label: 'Flex', lock: 'None', lockMonths: 0, multiplier: 1.0, multiplierLabel: '1.00x', exitFee: '0%', minDeposit: 1, apy: 4.2 },
  { id: 'l3',   label: 'L3',   lock: '3 months', lockMonths: 3, multiplier: 1.05, multiplierLabel: '1.05x', exitFee: '0.50%', minDeposit: 50, apy: 5.1 },
  { id: 'l6',   label: 'L6',   lock: '6 months', lockMonths: 6, multiplier: 1.15, multiplierLabel: '1.15x', exitFee: '1.25%', minDeposit: 100, apy: 6.8 },
  { id: 'l12',  label: 'L12',  lock: '12 months', lockMonths: 12, multiplier: 1.4, multiplierLabel: '1.40x', exitFee: '3.00%', minDeposit: 250, apy: 9.3 },
] as const;

type TierId = typeof TIERS[number]['id'];
type Step = 1 | 2 | 3 | 4;
type TxStatus = 'pending' | 'confirmed' | 'failed';

function lockExpiry(lockMonths: number): string {
  if (lockMonths === 0) return 'No lock';
  const d = new Date();
  d.setMonth(d.getMonth() + lockMonths);
  return d.toLocaleDateString('en-US', { year: 'numeric', month: 'short', day: 'numeric' });
}

function DepositFlow() {
  const params = useSearchParams();
  const paramTier = params.get('tier') as TierId | null;

  const [step, setStep] = useState<Step>(1);
  const [tierId, setTierId] = useState<TierId>(paramTier ?? 'flex');
  const [amount, setAmount] = useState('');
  const [txStatus, setTxStatus] = useState<TxStatus | null>(null);
  const [txError, setTxError] = useState('');

  useEffect(() => {
    if (paramTier) setTierId(paramTier);
  }, [paramTier]);

  const tier = TIERS.find((t) => t.id === tierId)!;
  const amountNum = parseFloat(amount) || 0;
  const belowMin = amountNum < tier.minDeposit;
  const projectedYield = tier.lockMonths === 0
    ? (amountNum * tier.apy / 100).toFixed(2)
    : (amountNum * tier.apy / 100 * (tier.lockMonths / 12)).toFixed(2);
  const shares = (amountNum * tier.multiplier).toFixed(0);
  const estimatedFee = '0.00001 XLM';

  function handleConfirm() {
    setStep(4);
    setTxStatus('pending');
    // sdk.deposit({ tier: tier.label, amount }) would go here
    setTimeout(() => setTxStatus('confirmed'), 2000);
  }

  function handleRetry() {
    setTxStatus(null);
    setTxError('');
    setStep(3);
  }

  return (
    <main style={s.main}>
      <nav style={s.nav}>
        <Link href="/" style={s.logo}>YieldLadder</Link>
        <Link href="/dashboard" style={s.navLink}>Dashboard</Link>
      </nav>

      <h1 style={s.heading}>Deposit USDC</h1>
      <Stepper current={step} />

      {/* Step 1 — Tier selection */}
      {step === 1 && (
        <div>
          <p style={s.sub}>Select a vault tier.</p>
          <div style={s.tierGrid}>
            {TIERS.map((t) => (
              <button
                key={t.id}
                type="button"
                onClick={() => setTierId(t.id)}
                style={{ ...s.tierCard, ...(tierId === t.id ? s.tierCardSelected : {}) }}
              >
                <div style={s.tierTop}>
                  <span style={s.tierName}>{t.label}</span>
                  <span style={s.tierMult}>{t.multiplierLabel}</span>
                </div>
                <div style={s.tierApy}>{t.apy}% APY</div>
                <dl style={s.dl}>
                  <Row label="Lock" value={t.lock} />
                  <Row label="Exit fee" value={t.exitFee} />
                  <Row label="Min deposit" value={`${t.minDeposit} USDC`} />
                </dl>
              </button>
            ))}
          </div>
          <div style={s.btnRow}>
            <button style={s.btnPrimary} type="button" onClick={() => setStep(2)}>Continue</button>
          </div>
        </div>
      )}

      {/* Step 2 — Amount input */}
      {step === 2 && (
        <div style={s.card}>
          <p style={s.sub}>Enter deposit amount for <strong>{tier.label}</strong>.</p>
          <label style={s.label} htmlFor="amount">Amount (USDC)</label>
          <input
            id="amount"
            type="number"
            min={tier.minDeposit}
            step="0.01"
            value={amount}
            onChange={(e) => setAmount(e.target.value)}
            style={s.input}
            placeholder={`Min ${tier.minDeposit} USDC`}
          />
          {amount && belowMin && (
            <p style={s.errText}>Minimum deposit is {tier.minDeposit} USDC.</p>
          )}
          {amountNum > 0 && !belowMin && (
            <dl style={{ ...s.dl, marginTop: '1rem' }}>
              <Row label="Shares to mint" value={shares} />
              <Row label={`Projected yield (${tier.lockMonths || 'flex'} mo)`} value={`~${projectedYield} USDC`} />
            </dl>
          )}
          <div style={s.btnRow}>
            <button style={s.btnSecondary} type="button" onClick={() => setStep(1)}>Back</button>
            <button
              style={belowMin || !amountNum ? s.btnDisabled : s.btnPrimary}
              type="button"
              disabled={belowMin || !amountNum}
              onClick={() => setStep(3)}
            >
              Continue
            </button>
          </div>
        </div>
      )}

      {/* Step 3 — Confirmation modal */}
      {step === 3 && (
        <div style={s.card}>
          <h2 style={s.cardTitle}>Confirm Deposit</h2>
          <dl style={s.dl}>
            <Row label="Vault tier" value={tier.label} />
            <Row label="Lock duration" value={tier.lock} />
            <Row label="Amount" value={`${amount} USDC`} />
            <Row label="Shares minted" value={shares} />
            <Row label="Lock expiry" value={lockExpiry(tier.lockMonths)} />
            <Row label="Projected yield" value={`~${projectedYield} USDC`} />
            <Row label="Estimated fee" value={estimatedFee} />
          </dl>
          <div style={s.btnRow}>
            <button style={s.btnSecondary} type="button" onClick={() => setStep(2)}>Back</button>
            <button style={s.btnPrimary} type="button" onClick={handleConfirm}>
              Confirm &amp; Deposit
            </button>
          </div>
        </div>
      )}

      {/* Step 4 — Transaction states */}
      {step === 4 && (
        <div style={{ ...s.card, alignItems: 'center', textAlign: 'center' }}>
          {txStatus === 'pending' && (
            <>
              <div style={s.spinner} aria-label="Loading" />
              <p style={s.sub}>Submitting transaction…</p>
            </>
          )}
          {txStatus === 'confirmed' && (
            <>
              <div style={s.successIcon}>✓</div>
              <h2 style={s.cardTitle}>Deposit Confirmed</h2>
              <dl style={{ ...s.dl, textAlign: 'left', width: '100%' }}>
                <Row label="Tier" value={tier.label} />
                <Row label="Amount" value={`${amount} USDC`} />
                <Row label="Shares" value={shares} />
              </dl>
              <Link href="/portfolio" style={{ ...s.btnPrimary, marginTop: '1rem', textDecoration: 'none' }}>
                View Portfolio
              </Link>
            </>
          )}
          {txStatus === 'failed' && (
            <>
              <div style={s.failIcon}>✗</div>
              <h2 style={s.cardTitle}>Transaction Failed</h2>
              <p style={s.errText}>{txError || 'An error occurred. Please try again.'}</p>
              <button style={s.btnPrimary} type="button" onClick={handleRetry}>
                Try Again
              </button>
            </>
          )}
        </div>
      )}
    </main>
  );
}

function Stepper({ current }: { current: Step }) {
  const labels = ['Select Tier', 'Amount', 'Confirm', 'Done'];
  return (
    <div style={s.stepper} aria-label="Progress">
      {labels.map((label, i) => {
        const n = (i + 1) as Step;
        const active = n === current;
        const done = n < current;
        return (
          <div key={label} style={s.stepItem}>
            <div style={{ ...s.stepDot, ...(done ? s.stepDone : active ? s.stepActive : {}) }}>
              {done ? '✓' : n}
            </div>
            <span style={{ ...s.stepLabel, ...(active ? { color: '#f1f5f9' } : {}) }}>{label}</span>
          </div>
        );
      })}
    </div>
  );
}

function Row({ label, value }: { label: string; value: string }) {
  return (
    <div style={{ display: 'flex', justifyContent: 'space-between', padding: '3px 0' }}>
      <dt style={{ color: '#94a3b8', fontSize: '0.83rem' }}>{label}</dt>
      <dd style={{ margin: 0, fontSize: '0.83rem', fontWeight: 500 }}>{value}</dd>
    </div>
  );
}

export default function DepositPage() {
  return (
    <Suspense>
      <DepositFlow />
    </Suspense>
  );
}

const s = {
  main: { maxWidth: 680, margin: '0 auto', padding: '1.5rem', fontFamily: 'sans-serif', color: '#f1f5f9' },
  nav: { display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '2rem' },
  logo: { fontWeight: 700, fontSize: '1.1rem', color: '#f1f5f9', textDecoration: 'none' },
  navLink: { color: '#94a3b8', textDecoration: 'none', fontSize: '0.9rem' },
  heading: { fontSize: '1.75rem', fontWeight: 700, marginBottom: '1.5rem' },
  sub: { color: '#94a3b8', marginBottom: '1rem', fontSize: '0.9rem' },
  stepper: { display: 'flex', gap: '1.5rem', marginBottom: '2rem', flexWrap: 'wrap' as const },
  stepItem: { display: 'flex', alignItems: 'center', gap: '0.4rem' },
  stepDot: { width: 26, height: 26, borderRadius: '50%', background: '#1e293b', display: 'flex', alignItems: 'center', justifyContent: 'center', fontSize: '0.75rem', fontWeight: 700, color: '#64748b', flexShrink: 0 },
  stepActive: { background: '#1d4ed8', color: '#fff' },
  stepDone: { background: '#15803d', color: '#fff' },
  stepLabel: { fontSize: '0.8rem', color: '#64748b' },
  tierGrid: { display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(140px, 1fr))', gap: '0.75rem', marginBottom: '1.5rem' },
  tierCard: { background: '#0f172a', border: '1px solid #1e293b', borderRadius: 8, padding: '1rem', cursor: 'pointer', textAlign: 'left' as const, color: '#f1f5f9', display: 'flex', flexDirection: 'column' as const, gap: '0.5rem' },
  tierCardSelected: { border: '2px solid #1d4ed8', background: '#0f1f4a' },
  tierTop: { display: 'flex', justifyContent: 'space-between', alignItems: 'center' },
  tierName: { fontWeight: 700, fontSize: '1rem' },
  tierMult: { fontSize: '0.72rem', background: '#14532d', color: '#4ade80', padding: '2px 6px', borderRadius: 10, fontWeight: 600 },
  tierApy: { fontSize: '1.1rem', fontWeight: 700, color: '#4ade80' },
  card: { background: '#0f172a', border: '1px solid #1e293b', borderRadius: 10, padding: '1.5rem', display: 'flex', flexDirection: 'column' as const, gap: '0.75rem' },
  cardTitle: { fontWeight: 700, fontSize: '1.1rem', marginBottom: '0.25rem' },
  dl: { margin: 0, display: 'flex', flexDirection: 'column' as const },
  label: { fontSize: '0.85rem', color: '#94a3b8', marginBottom: '0.25rem' },
  input: { padding: '0.6rem 0.75rem', borderRadius: 6, border: '1px solid #334155', background: '#020617', color: '#f1f5f9', fontSize: '1rem', outline: 'none', width: '100%' },
  btnRow: { display: 'flex', gap: '0.75rem', marginTop: '0.5rem', flexWrap: 'wrap' as const },
  btnPrimary: { padding: '0.55rem 1.25rem', borderRadius: 6, border: 'none', background: '#1d4ed8', color: '#fff', fontWeight: 600, fontSize: '0.9rem', cursor: 'pointer', display: 'inline-block' },
  btnSecondary: { padding: '0.55rem 1.25rem', borderRadius: 6, border: 'none', background: '#1e293b', color: '#cbd5e1', fontWeight: 600, fontSize: '0.9rem', cursor: 'pointer' },
  btnDisabled: { padding: '0.55rem 1.25rem', borderRadius: 6, border: 'none', background: '#1e293b', color: '#475569', fontWeight: 600, fontSize: '0.9rem', cursor: 'not-allowed' as const },
  errText: { color: '#f87171', fontSize: '0.82rem' },
  spinner: { width: 36, height: 36, border: '3px solid #1e293b', borderTop: '3px solid #3b82f6', borderRadius: '50%', animation: 'spin 0.8s linear infinite', margin: '1rem auto' },
  successIcon: { fontSize: '2.5rem', color: '#4ade80' },
  failIcon: { fontSize: '2.5rem', color: '#f87171' },
} satisfies Record<string, React.CSSProperties | Record<string, React.CSSProperties>>;
