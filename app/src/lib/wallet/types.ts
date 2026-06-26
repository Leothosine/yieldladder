export type WalletProvider = 'freighter' | 'lobstr' | 'xbull';
export type StellarNetwork = 'mainnet' | 'testnet' | 'futurenet';

export interface WalletAccount {
  publicKey: string;
  network: StellarNetwork;
}

export interface WalletSession {
  account: WalletAccount;
  provider: WalletProvider;
  connectedAt: number;
  expiresAt?: number;
}

export interface SignTransactionOptions {
  network?: StellarNetwork;
  accountToSign?: string;
}

export interface WalletAdapter {
  connect(): Promise<WalletAccount>;
  disconnect(): Promise<void>;
  isConnected(): Promise<boolean>;
  signTransaction(xdr: string, opts?: SignTransactionOptions): Promise<string>;
}