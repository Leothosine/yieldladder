export type Tier = 'Flex' | 'L3' | 'L6' | 'L12';

export interface Position {
  tier: Tier;
  principal: string;
  shares: string;
  accruedYield: string;
  lockUntil: number | null; // unix timestamp, null = Flex
  depositedAt: number; // unix timestamp
}

export interface HarvestEvent {
  date: string; // ISO date
  cumulativeYield: string;
}

export interface Transaction {
  date: string;
  type: 'deposit' | 'withdrawal' | 'harvest';
  tier: Tier;
  amount: string;
  txHash: string;
}

const now = Math.floor(Date.now() / 1000);

export const MOCK_POSITIONS: Position[] = [
  {
    tier: 'Flex',
    principal: '200.00',
    shares: '200',
    accruedYield: '3.12',
    lockUntil: null,
    depositedAt: now - 86400 * 30,
  },
  {
    tier: 'L6',
    principal: '500.00',
    shares: '575',
    accruedYield: '12.34',
    lockUntil: now + 86400 * 47,
    depositedAt: now - 86400 * 90,
  },
  {
    tier: 'L12',
    principal: '250.00',
    shares: '350',
    accruedYield: '8.75',
    lockUntil: now + 86400 * 275,
    depositedAt: now - 86400 * 60,
  },
];

export const MOCK_HARVEST_HISTORY: Record<Tier, HarvestEvent[]> = {
  Flex: [
    { date: '2026-01-01', cumulativeYield: '0.50' },
    { date: '2026-02-01', cumulativeYield: '1.10' },
    { date: '2026-03-01', cumulativeYield: '1.80' },
    { date: '2026-04-01', cumulativeYield: '2.40' },
    { date: '2026-05-01', cumulativeYield: '3.12' },
  ],
  L3: [],
  L6: [
    { date: '2025-10-01', cumulativeYield: '1.20' },
    { date: '2025-11-01', cumulativeYield: '3.50' },
    { date: '2025-12-01', cumulativeYield: '6.00' },
    { date: '2026-01-01', cumulativeYield: '8.90' },
    { date: '2026-02-01', cumulativeYield: '12.34' },
  ],
  L12: [
    { date: '2026-01-01', cumulativeYield: '2.10' },
    { date: '2026-02-01', cumulativeYield: '4.80' },
    { date: '2026-03-01', cumulativeYield: '8.75' },
  ],
};

export const MOCK_TRANSACTIONS: Transaction[] = [
  { date: '2025-09-24', type: 'deposit', tier: 'L6', amount: '500.00', txHash: 'abc123' },
  { date: '2025-11-23', type: 'deposit', tier: 'L12', amount: '250.00', txHash: 'def456' },
  { date: '2025-10-01', type: 'harvest', tier: 'L6', amount: '1.20', txHash: 'ghi789' },
  { date: '2025-11-01', type: 'harvest', tier: 'L6', amount: '2.30', txHash: 'jkl012' },
  { date: '2025-12-01', type: 'harvest', tier: 'L6', amount: '2.50', txHash: 'mno345' },
  { date: '2026-01-01', type: 'harvest', tier: 'L12', amount: '2.10', txHash: 'pqr678' },
  { date: '2026-01-01', type: 'harvest', tier: 'L6', amount: '2.90', txHash: 'stu901' },
  { date: '2026-02-01', type: 'harvest', tier: 'L6', amount: '3.44', txHash: 'vwx234' },
  { date: '2026-02-01', type: 'harvest', tier: 'L12', amount: '2.70', txHash: 'yza567' },
  { date: '2026-03-01', type: 'harvest', tier: 'L12', amount: '3.95', txHash: 'bcd890' },
  { date: '2026-03-01', type: 'deposit', tier: 'Flex', amount: '200.00', txHash: 'efg123' },
  { date: '2026-04-01', type: 'harvest', tier: 'Flex', amount: '0.60', txHash: 'hij456' },
  { date: '2026-05-01', type: 'harvest', tier: 'Flex', amount: '0.72', txHash: 'klm789' },
];

export const TIER_META: Record<Tier, { multiplier: string; earlyExitFee: number; label: string }> = {
  Flex: { multiplier: '1.00x', earlyExitFee: 0, label: 'Flex' },
  L3: { multiplier: '1.05x', earlyExitFee: 0.005, label: 'L3' },
  L6: { multiplier: '1.15x', earlyExitFee: 0.0125, label: 'L6' },
  L12: { multiplier: '1.40x', earlyExitFee: 0.03, label: 'L12' },
};
