export type { Tier, Network, YieldLadderOptions, Position } from './types';
import type { Tier, YieldLadderOptions, Position, Network } from './types';
import { LockNotExpiredError, BelowMinDepositError } from './errors';

const USDC_DECIMALS = 7;
const STROOPS_PER_UNIT = 10 ** USDC_DECIMALS;

const TIER_MIN_DEPOSIT: Record<Tier, bigint> = {
  Flex: BigInt(10 * STROOPS_PER_UNIT),
  L3: BigInt(50 * STROOPS_PER_UNIT),
  L6: BigInt(100 * STROOPS_PER_UNIT),
  L12: BigInt(250 * STROOPS_PER_UNIT),
};

const TIERS: readonly Tier[] = ['Flex', 'L3', 'L6', 'L12'];

const RPC_URLS: Record<Network, string> = {
  mainnet: 'https://rpc-mainnet.stellar.org',
  testnet: 'https://soroban-testnet.stellar.org',
};

export class YieldLadder {
  private readonly options: YieldLadderOptions;
  private readonly rpcUrl: string;

  constructor(options: YieldLadderOptions) {
    this.options = options;
    this.rpcUrl = RPC_URLS[options.network];
  }

  async deposit(params: { tier: Tier; amount: string }): Promise<void> {
    const stroops = this.toStroops(params.amount);
    if (stroops < TIER_MIN_DEPOSIT[params.tier]) {
      throw new BelowMinDepositError();
    }
    await this.simulateThenSubmit('deposit', params.tier, stroops.toString());
  }

  async withdraw(params: { tier: Tier }): Promise<void> {
    try {
      await this.simulateThenSubmit('withdraw', params.tier);
    } catch (err: unknown) {
      if (this.isLockError(err)) throw new LockNotExpiredError();
      throw err;
    }
  }

  async earlyExit(params: { tier: Tier }): Promise<void> {
    await this.simulate('early_exit', params.tier);
    await this.simulateThenSubmit('early_exit', params.tier);
  }

  async position(address: string): Promise<Position> {
    const positions = await Promise.all(
      TIERS.map(tier => this.queryTierPosition(address, tier)),
    );
    return positions.find(p => p.principal !== '0') ?? positions[0];
  }

  private toStroops(amount: string): bigint {
    const [whole = '0', frac = ''] = amount.split('.');
    const fracPadded = frac.padEnd(USDC_DECIMALS, '0').slice(0, USDC_DECIMALS);
    return (
      BigInt(whole) * BigInt(STROOPS_PER_UNIT) +
      BigInt(fracPadded.length > 0 ? fracPadded : '0')
    );
  }

  private isLockError(err: unknown): boolean {
    return err instanceof Error && err.message.toLowerCase().includes('lock');
  }

  private async simulate(_method: string, ..._args: unknown[]): Promise<string> {
    // Build and simulate transaction via SorobanRpc.Server at this.rpcUrl
    void this.rpcUrl;
    return '0';
  }

  private async simulateThenSubmit(
    _method: string,
    ..._args: unknown[]
  ): Promise<void> {
    // Simulate -> sign with this.options.signer -> submit via SorobanRpc
    void this.options;
  }

  private async queryTierPosition(_address: string, tier: Tier): Promise<Position> {
    return { tier, principal: '0', shares: '0', accruedYield: '0', lockUntil: null };
  }
}