import type { WalletSession, WalletProvider, StellarNetwork } from './types';

const SESSION_KEY = 'yl:wallet:session';
const SESSION_TTL_MS = 24 * 60 * 60 * 1000;

export function saveSession(session: WalletSession): void {
  if (typeof window === 'undefined') return;
  try {
    sessionStorage.setItem(SESSION_KEY, JSON.stringify(session));
  } catch {
    // sessionStorage unavailable
  }
}

export function loadSession(): WalletSession | null {
  if (typeof window === 'undefined') return null;
  try {
    const raw = sessionStorage.getItem(SESSION_KEY);
    if (!raw) return null;
    const session = JSON.parse(raw) as WalletSession;
    if (session.expiresAt !== undefined && Date.now() > session.expiresAt) {
      clearSession();
      return null;
    }
    return session;
  } catch {
    return null;
  }
}

export function clearSession(): void {
  if (typeof window === 'undefined') return;
  try {
    sessionStorage.removeItem(SESSION_KEY);
  } catch {
    // sessionStorage unavailable
  }
}

export function createSession(
  publicKey: string,
  provider: WalletProvider,
  network: StellarNetwork = 'mainnet',
): WalletSession {
  return {
    account: { publicKey, network },
    provider,
    connectedAt: Date.now(),
    expiresAt: Date.now() + SESSION_TTL_MS,
  };
}