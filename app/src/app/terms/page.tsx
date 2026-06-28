import type { Metadata } from 'next';

export const metadata: Metadata = {
  title: 'Terms of Service — YieldLadder',
  description: 'Terms of Service for YieldLadder time-locked USDC vaults on Stellar Soroban.',
};

const prose: React.CSSProperties = { fontSize: '0.9375rem', color: '#94a3b8', lineHeight: 1.7, marginBottom: '1rem' };
const heading2: React.CSSProperties = { fontSize: '1.125rem', fontWeight: 600, color: '#e2e8f0', marginTop: '2.25rem', marginBottom: '0.75rem', letterSpacing: '-0.01em' };
const link: React.CSSProperties = { color: '#7dd3fc', textDecoration: 'none' };

export default function TermsPage() {
  return (
    <div style={{ minHeight: '100vh', background: '#060810', color: '#f1f5f9', fontFamily: "-apple-system, BlinkMacSystemFont, 'Segoe UI', system-ui, sans-serif" }}>
      <nav style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', padding: '1.25rem 2rem', borderBottom: '1px solid rgba(255,255,255,0.07)' }}>
        <a href="/" style={{ fontSize: '1.125rem', fontWeight: 600, letterSpacing: '-0.02em', color: '#f1f5f9', textDecoration: 'none' }}>YieldLadder</a>
        <a href="/#vaults" style={{ fontSize: '0.875rem', color: '#94a3b8', textDecoration: 'none' }}>Explore Vaults</a>
      </nav>

      <main style={{ maxWidth: 720, margin: '0 auto', padding: '4rem 2rem 3rem' }}>
        <h1 style={{ fontSize: 'clamp(2rem,5vw,2.5rem)', fontWeight: 700, letterSpacing: '-0.03em', color: '#f1f5f9', marginBottom: '0.5rem' }}>
          Terms of Service
        </h1>
        <p style={{ fontSize: '0.8125rem', color: '#64748b', marginBottom: '2rem' }}>Last updated: February 10, 2026</p>
        <p style={{ fontSize: '1.0625rem', color: '#cbd5e1', lineHeight: 1.65, marginBottom: '2.5rem', paddingBottom: '2rem', borderBottom: '1px solid rgba(255,255,255,0.07)' }}>
          Please read these Terms of Service carefully before interacting with the YieldLadder protocol. By connecting your wallet or depositing funds, you agree to be bound by these terms.
        </p>

        <h2 style={heading2}>1. Nature of the protocol</h2>
        <p style={prose}>YieldLadder is a non-custodial, permissionless set of smart contracts deployed on the Stellar Soroban network. The protocol routes USDC deposits into curated Stellar AMM liquidity pools and automatically compounds trading-fee yield back into depositor positions. There is no company, foundation, or legal entity that holds, controls, or manages your funds at any time.</p>
        <p style={prose}>All rules governing deposits, withdrawals, lock durations, share-weight multipliers, and early-exit fee redistribution are enforced exclusively by immutable on-chain logic. No administrator, multisig, or upgrade mechanism can modify contract rules or move depositor funds.</p>

        <h2 style={heading2}>2. User responsibilities</h2>
        <p style={prose}>You are solely responsible for the security of your private keys and wallet. YieldLadder has no ability to recover lost keys, reverse transactions, or intervene in any on-chain action. You must ensure that the wallet address you use is under your exclusive control.</p>
        <p style={prose}>You represent that you have sufficient technical understanding of blockchain technology, smart contracts, and decentralised finance to evaluate the risks of interacting with the protocol. You are not relying on YieldLadder or any affiliated party for investment advice.</p>
        <p style={prose}>You agree not to use the protocol in violation of applicable laws or regulations in your jurisdiction, including but not limited to sanctions requirements.</p>

        <h2 style={heading2}>3. No warranty and no guarantee of yield</h2>
        <p style={prose}>The protocol is provided &ldquo;as is&rdquo; and &ldquo;as available&rdquo; without any warranty of any kind, express or implied. Yield is derived entirely from Stellar AMM trading fees and is not guaranteed. Past performance of any vault tier is not indicative of future results. APY figures displayed in the interface are estimates based on recent historical fee data and will fluctuate with pool activity.</p>

        <h2 style={heading2}>4. Risk acknowledgement</h2>
        <p style={prose}>By depositing funds you explicitly acknowledge the following categories of risk:</p>
        <ul style={{ margin: '0 0 1rem 1.5rem', display: 'flex', flexDirection: 'column', gap: '0.5rem' }}>
          {[
            ['Smart contract risk.', 'Bugs or vulnerabilities in the Soroban contracts could result in partial or total loss of deposited funds, even if an audit has been conducted.'],
            ['Impermanent loss.', 'Participation in AMM liquidity pools exposes depositors to impermanent loss relative to simply holding USDC.'],
            ['Stablecoin depeg risk.', 'USDC is issued by a centralised issuer and may temporarily or permanently lose its peg to the US dollar.'],
            ['Stellar network risk.', 'The protocol depends on the availability and correct operation of the Stellar network and Soroban virtual machine.'],
            ['Regulatory risk.', 'Regulatory actions in your jurisdiction could affect your ability to interact with the protocol or access funds.'],
          ].map(([bold, rest]) => (
            <li key={bold} style={{ fontSize: '0.9375rem', color: '#94a3b8', lineHeight: 1.65 }}>
              <strong style={{ color: '#cbd5e1', fontWeight: 600 }}>{bold}</strong> {rest}
            </li>
          ))}
        </ul>
        <p style={prose}>A full discussion of risks is available in the{' '}
          <a href="https://github.com/LadderMine/yieldladder#risk-model" style={link} target="_blank" rel="noopener noreferrer">risk model section of the README</a>.
          You are encouraged to read it before depositing.</p>

        <h2 style={heading2}>5. Limitation of liability</h2>
        <p style={prose}>To the maximum extent permitted by applicable law, no contributor to YieldLadder shall be liable for any indirect, incidental, special, consequential, or punitive damages, including loss of funds, arising from your use of or inability to use the protocol.</p>

        <h2 style={heading2}>6. Governing law</h2>
        <p style={prose}>The governing law and jurisdiction for any disputes relating to these Terms will be determined and published by the protocol&apos;s contributors prior to mainnet launch. This section will be updated when that determination is finalised.</p>

        <h2 style={heading2}>7. Changes to these Terms</h2>
        <p style={prose}>These Terms may be updated from time to time. Material changes will be announced in the <a href="/blog" style={link}>Protocol Updates</a> feed. Continued use of the protocol after changes are posted constitutes acceptance of the revised Terms.</p>

        <h2 style={heading2}>8. Contact</h2>
        <p style={prose}>For security disclosures, contact <a href="mailto:security@yieldladder.dev" style={link}>security@yieldladder.dev</a>. For general questions, open a discussion on <a href="https://github.com/LadderMine/yieldladder/discussions" style={link} target="_blank" rel="noopener noreferrer">GitHub</a>.</p>
      </main>

      <footer style={{ maxWidth: 720, margin: '0 auto', padding: '1.5rem 2rem 3rem', display: 'flex', alignItems: 'center', justifyContent: 'space-between', flexWrap: 'wrap', gap: '1rem', borderTop: '1px solid rgba(255,255,255,0.07)' }}>
        <a href="/" style={{ fontSize: '0.875rem', color: '#64748b', textDecoration: 'none' }}>← Back to YieldLadder</a>
        <div style={{ display: 'flex', gap: '1.5rem' }}>
          <a href="/privacy" style={{ fontSize: '0.875rem', color: '#64748b', textDecoration: 'none' }}>Privacy Policy</a>
          <a href="/blog" style={{ fontSize: '0.875rem', color: '#64748b', textDecoration: 'none' }}>Updates</a>
        </div>
      </footer>
    </div>
  );
}