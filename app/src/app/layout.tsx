import type { Metadata } from 'next';
import { Toaster } from 'sonner';
import './globals.css';

export const metadata: Metadata = {
  title: 'YieldLadder — Time-Locked USDC Vaults on Stellar',
  description:
    'Deposit USDC into time-locked vaults on Soroban. Auto-routed to curated Stellar AMM pools. Non-custodial, immutable, 100% on-chain yield.',
  openGraph: {
    title: 'YieldLadder — Time-Locked USDC Vaults on Stellar',
    description:
      'Deposit USDC into time-locked vaults on Soroban. Auto-routed to curated Stellar AMM pools. Non-custodial, immutable, 100% on-chain yield.',
    url: 'https://yieldladder.dev',
    siteName: 'YieldLadder',
    images: [
      {
        url: '/og-image.png',
        width: 1200,
        height: 630,
        alt: 'YieldLadder — Time-Locked USDC Vaults on Stellar',
      },
    ],
    type: 'website',
  },
  twitter: {
    card: 'summary_large_image',
    title: 'YieldLadder — Time-Locked USDC Vaults on Stellar',
    description:
      'Deposit USDC into time-locked vaults on Soroban. Auto-routed to curated Stellar AMM pools. Non-custodial, immutable, 100% on-chain yield.',
    images: ['/og-image.png'],
  },
  metadataBase: new URL('https://yieldladder.dev'),
};

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en">
      <body>
        {children}
        <Toaster position="bottom-right" richColors />
      </body>
    </html>
  );
}
