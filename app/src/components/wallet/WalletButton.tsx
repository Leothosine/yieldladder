'use client';

import { useState, useEffect } from 'react';
import type { CSSProperties } from 'react';
import type { WalletProvider, WalletSession, StellarNetwork } from '@/lib/wallet/types';
import {
  loadSession,
  clearSession,
  createSession,
  saveSession,
} from '@/lib/wallet/session';

interface FreighterAPI {
  requestAccess(): Promise<{ address: string }>;
  getNetwork(): Promise<{ network: string; networkPassphrase: string }>;
}

const PROVIDERS: { id: WalletProvider; label: string }[] = [
  { id: 'freighter', label: 'Freighter' },
  { id: 'lobstr', label: 'LOBSTR' },
  { id: 'xbull', label: 'xBull' },
];

export function WalletButton() {
  const [session, setSession] = useState<WalletSession | null>(null);
  const [isConnecting, setIsConnecting] = useState(false);
  const [showPicker, setShowPicker] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const stored = loadSession();
    if (stored) setSession(stored);
  }, []);

  async function connectFreighter(): Promise<void> {
    const w = window as typeof window & { freighter?: FreighterAPI };
    if (!w.freighter) {
      throw new Error('Freighter extension not detected. Install it at freighter.app.');
    }
    const { address } = await w.freighter.requestAccess();
    const { network } = await w.freighter.getNetwork();
    const net: StellarNetwork =
      network.toUpperCase() === 'TESTNET'
        ? 'testnet'
        : network.toUpperCase() === 'FUTURENET'
        ? 'futurenet'
        : 'mainnet';
    const s = createSession(address, 'freighter', net);
    saveSession(s);
    setSession(s);
  }

  async function connect(provider: WalletProvider): Promise<void> {
    setIsConnecting(true);
    setError(null);
    setShowPicker(false);
    try {
      if (provider === 'freighter') {
        await connectFreighter();
      } else {
        throw new Error(`${provider} wallet support is coming soon.`);
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Connection failed');
    } finally {
      setIsConnecting(false);
    }
  }

  function disconnect(): void {
    clearSession();
    setSession(null);
    setError(null);
  }

  const short = session
    ? `${session.account.publicKey.slice(0, 4)}…${session.account.publicKey.slice(-4)}`
    : null;

  return (
    <div style={styles.wrapper}>
      {error && <p style={styles.error}>{error}</p>}

      {session ? (
        <div style={styles.connected}>
          <span style={styles.address}>{short}</span>
          <button style={styles.secondaryBtn} onClick={disconnect} type="button">
            Disconnect
          </button>
        </div>
      ) : (
        <>
          {showPicker && (
            <div style={styles.picker}>
              {PROVIDERS.map(({ id, label }) => (
                <button
                  key={id}
                  style={styles.providerBtn}
                  onClick={() => connect(id)}
                  type="button"
                  disabled={isConnecting}
                >
                  {label}
                </button>
              ))}
            </div>
          )}
          <button
            style={styles.primaryBtn}
            onClick={() => setShowPicker((v) => !v)}
            type="button"
            disabled={isConnecting}
          >
            {isConnecting ? 'Connecting…' : 'Connect Wallet'}
          </button>
        </>
      )}
    </div>
  );
}

const styles: Record<string, CSSProperties> = {
  wrapper: {
    position: 'relative',
    display: 'inline-flex',
    flexDirection: 'column',
    alignItems: 'flex-end',
    gap: 4,
  },
  connected: { display: 'flex', alignItems: 'center', gap: 8 },
  address: { fontSize: '0.85rem', fontWeight: 600, color: '#1d4ed8' },
  error: { color: '#dc2626', fontSize: '0.8rem', margin: 0, maxWidth: 220 },
  picker: {
    position: 'absolute',
    top: '110%',
    right: 0,
    backgroundColor: '#fff',
    border: '1px solid #e2e8f0',
    borderRadius: 8,
    padding: '0.5rem',
    display: 'flex',
    flexDirection: 'column',
    gap: 4,
    zIndex: 10,
    minWidth: 160,
    boxShadow: '0 4px 12px rgba(0,0,0,0.1)',
  },
  primaryBtn: {
    padding: '0.5rem 1rem',
    borderRadius: 6,
    border: 'none',
    backgroundColor: '#1d4ed8',
    color: '#fff',
    fontWeight: 600,
    cursor: 'pointer',
    fontSize: '0.9rem',
  },
  secondaryBtn: {
    padding: '0.4rem 0.8rem',
    borderRadius: 6,
    border: '1px solid #e2e8f0',
    backgroundColor: '#fff',
    color: '#374151',
    fontWeight: 500,
    cursor: 'pointer',
    fontSize: '0.85rem',
  },
  providerBtn: {
    padding: '0.5rem 1rem',
    borderRadius: 6,
    border: '1px solid #e2e8f0',
    backgroundColor: '#f9fafb',
    color: '#111827',
    cursor: 'pointer',
    textAlign: 'left',
    fontWeight: 500,
    width: '100%',
  },
};