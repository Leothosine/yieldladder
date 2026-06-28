import type { Metadata } from 'next';

export const metadata: Metadata = {
  title: 'Updates — YieldLadder',
  description: 'Protocol announcements, governance changes, new pool allocations, and milestones for YieldLadder on Stellar Soroban.',
};

const POSTS = [
  {
    slug: 'internal-audit-complete',
    title: 'Internal audit complete — zero critical findings',
    date: '2026-01-15',
    category: 'Security',
    summary:
      'Our internal security review of the Soroban vault contracts concluded with zero critical or high-severity findings. Two low-severity observations around event emission ordering were resolved before testnet deployment. The full audit report is available in the repository.',
    link: 'https://github.com/LadderMine/yieldladder#audit',
  },
  {
    slug: 'governance-contract-deployed',
    title: 'Governance contract deployed to Stellar testnet',
    date: '2026-01-28',
    category: 'Governance',
    summary:
      'The guardian multisig and 72-hour timelock contracts are live on Stellar testnet. Allocation proposals can now be submitted on-chain. The timelock enforces a mandatory waiting period before any pool weight change takes effect, giving depositors time to exit if they disagree with a proposed rebalance.',
    link: 'https://github.com/LadderMine/yieldladder/discussions',
  },
  {
    slug: 'testnet-launch',
    title: 'YieldLadder launches on Stellar testnet',
    date: '2026-02-10',
    category: 'Protocol Update',
    summary:
      'All four vault tiers — Flex, L3, L6, and L12 — are now live on Stellar testnet. Deposits are open for testing. The harvest bot is running every 24 hours, compounding AMM trading fees back into each vault position. We encourage early testers to try the early-exit flow and verify that fees redistribute correctly to remaining depositors.',
    link: 'https://github.com/LadderMine/yieldladder',
  },
] as const;

const TAG_STYLES: Record<string, { background: string; color: string }> = {
  Security:         { background: 'rgba(234,88,12,0.15)',  color: '#fb923c' },
  Governance:       { background: 'rgba(99,102,241,0.15)', color: '#a5b4fc' },
  'Protocol Update':{ background: 'rgba(20,184,166,0.15)', color: '#5eead4' },
  'New Pool':       { background: 'rgba(34,197,94,0.15)',  color: '#86efac' },
};

function formatDate(iso: string) {
  return new Date(iso).toLocaleDateString('en-US', {
    year: 'numeric', month: 'long', day: 'numeric',
  });
}

export default function BlogPage() {
  const sorted = [...POSTS].reverse();

  return (
    <div style={{ minHeight: '100vh', background: '#060810', color: '#f1f5f9', fontFamily: "-apple-system, BlinkMacSystemFont, 'Segoe UI', system-ui, sans-serif" }}>
      {/* Nav */}
      <nav style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', padding: '1.25rem 2rem', borderBottom: '1px solid rgba(255,255,255,0.07)' }}>
        <a href="/" style={{ fontSize: '1.125rem', fontWeight: 600, letterSpacing: '-0.02em', color: '#f1f5f9', textDecoration: 'none' }}>YieldLadder</a>
        <a href="/#vaults" style={{ fontSize: '0.875rem', color: '#94a3b8', textDecoration: 'none' }}>Explore Vaults</a>
      </nav>

      {/* Header */}
      <header style={{ maxWidth: 720, margin: '0 auto', padding: '4rem 2rem 2.5rem' }}>
        <h1 style={{ fontSize: 'clamp(2rem,5vw,2.75rem)', fontWeight: 700, letterSpacing: '-0.03em', color: '#f1f5f9', marginBottom: '0.75rem' }}>
          Protocol Updates
        </h1>
        <p style={{ fontSize: '1.0625rem', color: '#94a3b8', lineHeight: 1.6 }}>
          Governance changes, new pool allocations, security notices, and protocol milestones.
        </p>
      </header>

      {/* Feed */}
      <main style={{ maxWidth: 720, margin: '0 auto', padding: '0 2rem 4rem' }}>
        {sorted.map((post) => {
          const tagStyle = TAG_STYLES[post.category] ?? { background: 'rgba(255,255,255,0.08)', color: '#94a3b8' };
          return (
            <article
              key={post.slug}
              style={{ padding: '2rem 0', borderBottom: '1px solid rgba(255,255,255,0.07)' }}
            >
              <div style={{ display: 'flex', alignItems: 'center', gap: '0.75rem', marginBottom: '0.875rem' }}>
                <span style={{
                  display: 'inline-block',
                  fontSize: '0.6875rem',
                  fontWeight: 600,
                  letterSpacing: '0.06em',
                  textTransform: 'uppercase',
                  padding: '0.2rem 0.6rem',
                  borderRadius: 4,
                  ...tagStyle,
                }}>
                  {post.category}
                </span>
                <time style={{ fontSize: '0.8125rem', color: '#64748b' }} dateTime={post.date}>
                  {formatDate(post.date)}
                </time>
              </div>
              <h2 style={{ fontSize: '1.3125rem', fontWeight: 600, letterSpacing: '-0.02em', color: '#f1f5f9', marginBottom: '0.625rem', lineHeight: 1.35 }}>
                {post.title}
              </h2>
              <p style={{ fontSize: '0.9375rem', color: '#94a3b8', lineHeight: 1.65, marginBottom: '1rem' }}>
                {post.summary}
              </p>
              {post.link && (
                <a
                  href={post.link}
                  style={{ fontSize: '0.875rem', color: '#7dd3fc', textDecoration: 'none', fontWeight: 500 }}
                  target="_blank"
                  rel="noopener noreferrer"
                >
                  Read more ↗
                </a>
              )}
            </article>
          );
        })}
      </main>

      {/* Footer */}
      <footer style={{ maxWidth: 720, margin: '0 auto', padding: '0 2rem 3rem' }}>
        <a href="/" style={{ fontSize: '0.875rem', color: '#64748b', textDecoration: 'none' }}>← Back to YieldLadder</a>
      </footer>
    </div>
  );
}