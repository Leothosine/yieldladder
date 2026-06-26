'use client';

import Link from 'next/link';
import PositionCard from './PositionCard';
import {
  MOCK_POSITIONS,
  MOCK_HARVEST_HISTORY,
  MOCK_TRANSACTIONS,
} from './mockData';

// In production these would come from sdk.position(walletAddress)
// and the data service layer (GF-12). Using mock data until
// GF-09 (wallet integration) and GF-12 (data service) are merged.
const WALLET_CONNECTED = true;
const positions = MOCK_POSITIONS;

function downloadCsv() {
  const header = 'date,type,tier,amount,txHash';
  const rows = MOCK_TRANSACTIONS.map(
    (t) => `${t.date},${t.type},${t.tier},${t.amount},${t.txHash}`
  );
  const csv = [header, ...rows].join('\n');
  const blob = new Blob([csv], { type: 'text/csv' });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = 'yieldladder-transactions.csv';
  a.click();
  URL.revokeObjectURL(url);
}

export default function PortfolioPage() {
  if (!WALLET_CONNECTED) {
    return (
      <main style={s.main}>
        <div style={s.connectPrompt}>
          <p style={{ color: '#94a3b8', marginBottom: '1rem' }}>
            Connect your wallet to view your portfolio.
          </p>
          <Link href="/" style={{ ...s.btn, ...s.btnPrimary }}>
            Connect Wallet
          </Link>
        </div>
      </main>
    );
  }

  if (positions.length === 0) {
    return (
      <main style={s.main}>
        <h1 style={s.heading}>Portfolio</h1>
        <div style={s.emptyState}>
          <p style={s.emptyText}>You have no active positions yet.</p>
          <Link href="/dashboard" style={{ ...s.btn, ...s.btnPrimary }}>
            Deposit Now
          </Link>
        </div>
      </main>
    );
  }

  const totalDeposited = positions
    .reduce((sum, p) => sum + parseFloat(p.principal), 0)
    .toFixed(2);
  const totalYield = positions
    .reduce((sum, p) => sum + parseFloat(p.accruedYield), 0)
    .toFixed(2);
  const blendedApy = (
    positions.reduce((sum, p) => {
      const apr =
        (parseFloat(p.accruedYield) / parseFloat(p.principal)) *
        (365 / Math.max(1, Math.floor((Date.now() / 1000 - p.depositedAt) / 86400))) *
        100;
      return sum + apr;
    }, 0) / positions.length
  ).toFixed(1);

  return (
    <main style={s.main}>
      <nav style={s.nav}>
        <Link href="/" style={s.navLogo}>YieldLadder</Link>
        <div style={s.navLinks}>
          <Link href="/dashboard" style={s.navLink}>Dashboard</Link>
          <Link href="/portfolio" style={{ ...s.navLink, color: '#f1f5f9', fontWeight: 600 }}>
            Portfolio
          </Link>
        </div>
      </nav>

      <h1 style={s.heading}>Portfolio</h1>

      {/* Summary card */}
      <div style={s.summaryCard}>
        <SummaryItem label="Total Deposited" value={`$${totalDeposited}`} />
        <div style={s.divider} />
        <SummaryItem label="Total Yield Earned" value={`$${totalYield}`} accent />
        <div style={s.divider} />
        <SummaryItem label="Blended APY" value={`${blendedApy}%`} />
        <div style={s.divider} />
        <SummaryItem label="Active Positions" value={String(positions.length)} />
      </div>

      {/* Position cards */}
      <div style={s.grid}>
        {positions.map((pos) => (
          <PositionCard
            key={pos.tier}
            position={pos}
            harvestHistory={MOCK_HARVEST_HISTORY[pos.tier]}
          />
        ))}
      </div>

      {/* CSV export */}
      <div style={{ marginTop: '2rem' }}>
        <button style={{ ...s.btn, ...s.btnSecondary }} type="button" onClick={downloadCsv}>
          ⬇ Download CSV
        </button>
        <p style={s.csvNote}>Exports all deposits, withdrawals, and harvests for tax reporting.</p>
      </div>
    </main>
  );
}

function SummaryItem({
  label,
  value,
  accent,
}: {
  label: string;
  value: string;
  accent?: boolean;
}) {
  return (
    <div style={s.summaryItem}>
      <span style={s.summaryLabel}>{label}</span>
      <span style={{ ...s.summaryValue, ...(accent ? { color: '#4ade80' } : {}) }}>
        {value}
      </span>
    </div>
  );
}

const s = {
  main: {
    maxWidth: 960,
    margin: '0 auto',
    padding: '1.5rem',
    fontFamily: 'sans-serif',
    color: '#f1f5f9',
  },
  nav: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: '2rem',
  },
  navLogo: { fontWeight: 700, fontSize: '1.1rem', color: '#f1f5f9', textDecoration: 'none' },
  navLinks: { display: 'flex', gap: '1.25rem' },
  navLink: { color: '#94a3b8', textDecoration: 'none', fontSize: '0.9rem' },
  heading: { fontSize: '1.75rem', fontWeight: 700, marginBottom: '1.5rem' },
  summaryCard: {
    display: 'flex',
    flexWrap: 'wrap' as const,
    gap: '0',
    background: '#0f172a',
    border: '1px solid #1e293b',
    borderRadius: 10,
    padding: '1.25rem 1.5rem',
    marginBottom: '2rem',
  },
  summaryItem: {
    display: 'flex',
    flexDirection: 'column' as const,
    gap: '0.2rem',
    flex: '1 1 140px',
    padding: '0.25rem 0.75rem',
  },
  summaryLabel: { fontSize: '0.78rem', color: '#64748b', textTransform: 'uppercase' as const, letterSpacing: '0.05em' },
  summaryValue: { fontSize: '1.3rem', fontWeight: 700 },
  divider: { width: '1px', background: '#1e293b', margin: '0 0.25rem', alignSelf: 'stretch' },
  grid: {
    display: 'grid',
    gridTemplateColumns: 'repeat(auto-fill, minmax(280px, 1fr))',
    gap: '1rem',
  },
  btn: {
    display: 'inline-block',
    padding: '0.5rem 1.25rem',
    borderRadius: 6,
    border: 'none',
    fontWeight: 600,
    fontSize: '0.88rem',
    cursor: 'pointer',
    textDecoration: 'none',
  },
  btnPrimary: { background: '#1d4ed8', color: '#fff' },
  btnSecondary: { background: '#1e293b', color: '#cbd5e1' },
  csvNote: { fontSize: '0.75rem', color: '#64748b', marginTop: '0.4rem' },
  connectPrompt: {
    textAlign: 'center' as const,
    padding: '4rem 1rem',
  },
  emptyState: {
    textAlign: 'center' as const,
    padding: '4rem 1rem',
    background: '#0f172a',
    borderRadius: 10,
    border: '1px solid #1e293b',
  },
  emptyText: { color: '#94a3b8', marginBottom: '1.25rem', fontSize: '1rem' },
} satisfies Record<string, React.CSSProperties | Record<string, React.CSSProperties>>;
