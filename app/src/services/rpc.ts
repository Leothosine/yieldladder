const RPC_URL =
  process.env.NEXT_PUBLIC_SOROBAN_RPC_URL ??
  'https://soroban-testnet.stellar.org';

let _seq = 0;

export async function callRpc<T = unknown>(
  method: string,
  params: unknown[] = [],
): Promise<T> {
  const res = await fetch(RPC_URL, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ jsonrpc: '2.0', id: ++_seq, method, params }),
    cache: 'no-store',
  });

  if (!res.ok) throw new Error(`Soroban RPC HTTP ${res.status}`);

  const json = (await res.json()) as {
    result?: T;
    error?: { message: string };
  };

  if (json.error) throw new Error(json.error.message);
  if (json.result === undefined) throw new Error('Empty Soroban RPC result');
  return json.result;
}

export interface LedgerInfo {
  sequence: number;
  closedAt: string;
  protocolVersion: number;
}

export function getLatestLedger(): Promise<LedgerInfo> {
  return callRpc<LedgerInfo>('getLatestLedger');
}
