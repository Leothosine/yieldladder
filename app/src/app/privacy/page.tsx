import type { Metadata } from 'next';

export const metadata: Metadata = {
  title: 'Privacy Policy — YieldLadder',
  description: 'Privacy Policy for YieldLadder — what data is and is not collected when you interact with the protocol.',
};

const prose: React.CSSProperties = { fontSize: '0.9375rem', color: '#94a3b8', lineHeight: 1.7, marginBottom: '1rem' };
const heading2: React.CSSProperties = { fontSize: '1.125rem', fontWeight: 600, color: '#e2e8f0', marginTop: '2.25rem', marginBottom: '0.75rem', letterSpacing: '-0.01em' };
const link: React.CSSProperties = { color: '#7dd3fc', textDecoration: 'none' };

export default function PrivacyPage() {
  return (
    <div style={{ minHeight: '100vh', background: '#060810', color: '#f1f5f9', fontFamily: "-apple-system, BlinkMacSystemFont, 'Segoe UI', system-ui, sans-serif" }}>
      <nav style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', padding: '1.25rem 2rem', borderBottom: '1px solid rgba(255,255,255,0.07)' }}>
        <a href="/" style={{ fontSize: '1.125rem', fontWeight: 600, letterSpacing: '-0.02em', color: '#f1f5f9', textDecoration: 'none' }}>YieldLadder</a>
        <a href="/#vaults" style={{ fontSize: '0.875rem', color: '#94a3b8', textDecoration: 'none' }}>Explore Vaults</a>
      </nav>

      <main style={{ maxWidth: 720, margin: '0 auto', padding: '4rem 2rem 3rem' }}>
        <h1 style={{ fontSize: 'clamp(2rem,5vw,2.5rem)', fontWeight: 700, letterSpacing: '-0.03em', color: '#f1f5f9', marginBottom: '0.5rem' }}>
          Privacy Policy
        </h1>
        <p style={{ fontSize: '0.8125rem', color: '#64748b', marginBottom: '2rem' }}>Last updated: February 10, 2026</p>
        <p style={{ fontSize: '1.0625rem', color: '#cbd5e1', lineHeight: 1.65, marginBottom: '2.5rem', paddingBottom: '2rem', borderBottom: '1px solid rgba(255,255,255,0.07)' }}>
          YieldLadder is a non-custodial protocol with no user accounts, no login, and no KYC. This policy explains exactly what information is and is not collected when you interact with the interface.
        </p>

        <h2 style={heading2}>1. What we collect</h2>
        <p style={prose}><strong style={{ color: '#cbd5e1', fontWeight: 600 }}>Wallet address.</strong> When you connect a wallet, your public Stellar address is read by the interface to display your positions and balances. Your public address is visible on the Stellar blockchain to anyone; it is not a secret. We do not store it in any database under our control.</p>
        <p style={prose}><strong style={{ color: '#cbd5e1', fontWeight: 600 }}>Standard web server logs.</strong> Hosting infrastructure may collect standard HTTP request logs including IP address, browser user-agent string, referring URL, and timestamps. These logs are retained for up to 30 days for security and abuse-prevention purposes and are not linked to wallet addresses or financial activity.</p>

        <h2 style={heading2}>2. What we do not collect</h2>
        <ul style={{ margin: '0 0 1rem 1.5rem', display: 'flex', flexDirection: 'column', gap: '0.5rem' }}>
          {[
            'We do not collect email addresses, names, or any personally identifiable information.',
            'We do not require or perform KYC (Know Your Customer) checks of any kind.',
            'We do not have user accounts or authentication systems.',
            'We do not have access to your private keys or seed phrase under any circumstances.',
            'We do not track individual transactions you send to the protocol beyond what is publicly visible on the Stellar blockchain.',
          ].map((item) => (
            <li key={item} style={{ fontSize: '0.9375rem', color: '#94a3b8', lineHeight: 1.65 }}>{item}</li>
          ))}
        </ul>

        <h2 style={heading2}>3. Analytics</h2>
        <p style={prose}>At present, YieldLadder does not use any third-party analytics scripts (e.g., Google Analytics, Mixpanel). If analytics tooling is added in the future, this policy will be updated and the change will be announced in the <a href="/blog" style={link}>Protocol Updates</a> feed at least 14 days before it takes effect.</p>

        <h2 style={heading2}>4. Cookies</h2>
        <p style={prose}>The interface does not set any cookies beyond what is strictly necessary for the site to function (for example, a session preference if you have chosen a light or dark theme). No tracking or advertising cookies are used.</p>

        <h2 style={heading2}>5. Third-party services</h2>
        <p style={prose}>The interface may load assets from trusted content-delivery networks. These services may collect standard HTTP request metadata (IP address, user-agent) per their own privacy policies. We do not share wallet addresses or financial data with any third party.</p>
        <p style={prose}>On-chain interactions go directly from your wallet to the Stellar network. No transaction data is routed through servers we control.</p>

        <h2 style={heading2}>6. Blockchain data</h2>
        <p style={prose}>All deposits, withdrawals, and yield accrual are permanently recorded on the public Stellar blockchain. This data is immutable and visible to anyone who inspects the ledger. The YieldLadder interface does not add any information to the blockchain beyond what your wallet signs.</p>

        <h2 style={heading2}>7. Children</h2>
        <p style={prose}>The protocol is not directed at persons under the age of 18. We do not knowingly collect information from minors.</p>

        <h2 style={heading2}>8. Changes to this Policy</h2>
        <p style={prose}>Material changes to this Privacy Policy will be announced in the <a href="/blog" style={link}>Protocol Updates</a> feed before they take effect. The date at the top of this page reflects the most recent revision.</p>

        <h2 style={heading2}>9. Contact</h2>
        <p style={prose}>Questions about this Privacy Policy can be directed to <a href="mailto:security@yieldladder.dev" style={link}>security@yieldladder.dev</a>.</p>
      </main>

      <footer style={{ maxWidth: 720, margin: '0 auto', padding: '1.5rem 2rem 3rem', display: 'flex', alignItems: 'center', justifyContent: 'space-between', flexWrap: 'wrap', gap: '1rem', borderTop: '1px solid rgba(255,255,255,0.07)' }}>
        <a href="/" style={{ fontSize: '0.875rem', color: '#64748b', textDecoration: 'none' }}>← Back to YieldLadder</a>
        <div style={{ display: 'flex', gap: '1.5rem' }}>
          <a href="/terms" style={{ fontSize: '0.875rem', color: '#64748b', textDecoration: 'none' }}>Terms of Service</a>
          <a href="/blog" style={{ fontSize: '0.875rem', color: '#64748b', textDecoration: 'none' }}>Updates</a>
        </div>
      </footer>
    </div>
  );
}