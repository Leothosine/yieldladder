const VAULT_TIERS = [
  {
    id: 'flex',
    label: 'Flex',
    lockDuration: 'None',
    multiplier: '1.00x',
    earlyExitFee: '0%',
    minDeposit: '1 USDC',
  },
  {
    id: 'l3',
    label: 'L3',
    lockDuration: '3 months',
    multiplier: '1.05x',
    earlyExitFee: '0.50%',
    minDeposit: '50 USDC',
  },
  {
    id: 'l6',
    label: 'L6',
    lockDuration: '6 months',
    multiplier: '1.15x',
    earlyExitFee: '1.25%',
    minDeposit: '100 USDC',
  },
  {
    id: 'l12',
    label: 'L12',
    lockDuration: '12 months',
    multiplier: '1.40x',
    earlyExitFee: '3.00%',
    minDeposit: '250 USDC',
  },
] as const;

import Link from 'next/link';

function VaultCard(props: (typeof VAULT_TIERS)[number]) {
  return (
    <div style={styles.card}>
      <div style={styles.cardHeader}>
        <span style={styles.tierLabel}>{props.label}</span>
        <span style={styles.multiplier}>{props.multiplier}</span>
      </div>
      <dl style={styles.dl}>
        <div style={styles.row}>
          <dt style={styles.dt}>Lock duration</dt>
          <dd style={styles.dd}>{props.lockDuration}</dd>
        </div>
        <div style={styles.row}>
          <dt style={styles.dt}>Early-exit fee</dt>
          <dd style={styles.dd}>{props.earlyExitFee}</dd>
        </div>
        <div style={styles.row}>
          <dt style={styles.dt}>Min deposit</dt>
          <dd style={styles.dd}>{props.minDeposit}</dd>
        </div>
      </dl>
      <Link href={`/deposit?tier=${props.id}`} style={styles.button}>
        Deposit
      </Link>
    </div>
  );
}

export default function DashboardPage() {
  return (
    <main style={styles.main}>
      <h1 style={styles.heading}>Dashboard</h1>
      <p style={styles.subheading}>Choose a vault tier to deposit USDC.</p>
      <div style={styles.grid}>
        {VAULT_TIERS.map((tier) => (
          <VaultCard key={tier.id} {...tier} />
        ))}
      </div>
    </main>
  );
}

const styles = {
  main: {
    maxWidth: 960,
    margin: '0 auto',
    padding: '2rem 1.5rem',
    fontFamily: 'sans-serif',
  },
  heading: {
    fontSize: '1.75rem',
    fontWeight: 700,
    marginBottom: '0.25rem',
  },
  subheading: {
    color: '#666',
    marginBottom: '2rem',
  },
  grid: {
    display: 'grid',
    gridTemplateColumns: 'repeat(auto-fill, minmax(200px, 1fr))',
    gap: '1rem',
  },
  card: {
    border: '1px solid #e2e8f0',
    borderRadius: 8,
    padding: '1.25rem',
    display: 'flex',
    flexDirection: 'column' as const,
    gap: '1rem',
    backgroundColor: '#fff',
  },
  cardHeader: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
  },
  tierLabel: {
    fontWeight: 700,
    fontSize: '1.1rem',
  },
  multiplier: {
    fontSize: '0.85rem',
    backgroundColor: '#f0fdf4',
    color: '#16a34a',
    padding: '2px 8px',
    borderRadius: 12,
    fontWeight: 600,
  },
  dl: {
    margin: 0,
    display: 'flex',
    flexDirection: 'column' as const,
    gap: '0.5rem',
  },
  row: {
    display: 'flex',
    justifyContent: 'space-between',
  },
  dt: {
    color: '#666',
    fontSize: '0.85rem',
  },
  dd: {
    margin: 0,
    fontSize: '0.85rem',
    fontWeight: 500,
  },
  button: {
    marginTop: 'auto',
    padding: '0.5rem',
    borderRadius: 6,
    border: 'none',
    backgroundColor: '#1d4ed8',
    color: '#fff',
    fontWeight: 600,
    cursor: 'pointer',
    fontSize: '0.9rem',
    textDecoration: 'none',
    textAlign: 'center' as const,
    display: 'block',
  },
} satisfies Record<string, React.CSSProperties | Record<string, React.CSSProperties>>;
