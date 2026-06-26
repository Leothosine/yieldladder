import { describe, it, expect, vi, beforeEach } from 'vitest';
import { YieldLadder } from '../index';
import { BelowMinDepositError, LockNotExpiredError } from '../errors';

const opts = { network: 'testnet' as const, signer: null };

describe('YieldLadder', () => {
  let sdk: YieldLadder;

  beforeEach(() => {
    sdk = new YieldLadder(opts);
    vi.restoreAllMocks();
  });

  describe('deposit', () => {
    it('throws BelowMinDepositError when amount is below Flex minimum (10 USDC)', async () => {
      await expect(sdk.deposit({ tier: 'Flex', amount: '5' })).rejects.toBeInstanceOf(BelowMinDepositError);
    });

    it('throws BelowMinDepositError when amount is below L3 minimum (50 USDC)', async () => {
      await expect(sdk.deposit({ tier: 'L3', amount: '49.9999999' })).rejects.toBeInstanceOf(BelowMinDepositError);
    });

    it('throws BelowMinDepositError when amount is below L6 minimum (100 USDC)', async () => {
      await expect(sdk.deposit({ tier: 'L6', amount: '99.9999999' })).rejects.toBeInstanceOf(BelowMinDepositError);
    });

    it('throws BelowMinDepositError when amount is below L12 minimum (250 USDC)', async () => {
      await expect(sdk.deposit({ tier: 'L12', amount: '249' })).rejects.toBeInstanceOf(BelowMinDepositError);
    });

    it('resolves when amount meets the Flex minimum exactly', async () => {
      vi.spyOn(sdk as any, 'simulateThenSubmit').mockResolvedValue(undefined);
      await expect(sdk.deposit({ tier: 'Flex', amount: '10' })).resolves.toBeUndefined();
    });

    it('resolves when amount exceeds the L12 minimum', async () => {
      vi.spyOn(sdk as any, 'simulateThenSubmit').mockResolvedValue(undefined);
      await expect(sdk.deposit({ tier: 'L12', amount: '500.50' })).resolves.toBeUndefined();
    });
  });

  describe('withdraw', () => {
    it('throws LockNotExpiredError when contract returns a lock error', async () => {
      vi.spyOn(sdk as any, 'simulateThenSubmit').mockRejectedValue(new Error('lock period active'));
      await expect(sdk.withdraw({ tier: 'L3' })).rejects.toBeInstanceOf(LockNotExpiredError);
    });

    it('re-throws non-lock errors unchanged', async () => {
      vi.spyOn(sdk as any, 'simulateThenSubmit').mockRejectedValue(new Error('network timeout'));
      await expect(sdk.withdraw({ tier: 'L3' })).rejects.toThrow('network timeout');
    });

    it('resolves on successful withdrawal', async () => {
      vi.spyOn(sdk as any, 'simulateThenSubmit').mockResolvedValue(undefined);
      await expect(sdk.withdraw({ tier: 'Flex' })).resolves.toBeUndefined();
    });
  });

  describe('earlyExit', () => {
    it('calls simulate before simulateThenSubmit', async () => {
      const order: string[] = [];
      vi.spyOn(sdk as any, 'simulate').mockImplementation(async () => { order.push('simulate'); return '100'; });
      vi.spyOn(sdk as any, 'simulateThenSubmit').mockImplementation(async () => { order.push('submit'); });
      await sdk.earlyExit({ tier: 'L6' });
      expect(order).toEqual(['simulate', 'submit']);
    });

    it('resolves without error on success', async () => {
      vi.spyOn(sdk as any, 'simulate').mockResolvedValue('100');
      vi.spyOn(sdk as any, 'simulateThenSubmit').mockResolvedValue(undefined);
      await expect(sdk.earlyExit({ tier: 'L12' })).resolves.toBeUndefined();
    });
  });

  describe('position', () => {
    it('returns a Position object with all required fields', async () => {
      const result = await sdk.position('GTEST...');
      expect(result).toMatchObject({
        tier: expect.stringMatching(/^(Flex|L3|L6|L12)$/),
        principal: expect.any(String),
        shares: expect.any(String),
        accruedYield: expect.any(String),
      });
    });

    it('returns the first tier with a non-zero principal', async () => {
      vi.spyOn(sdk as any, 'queryTierPosition')
        .mockResolvedValueOnce({ tier: 'Flex', principal: '0', shares: '0', accruedYield: '0', lockUntil: null })
        .mockResolvedValueOnce({ tier: 'L3', principal: '500', shares: '500', accruedYield: '12', lockUntil: 9999 })
        .mockResolvedValue({ tier: 'L6', principal: '0', shares: '0', accruedYield: '0', lockUntil: null });
      const result = await sdk.position('GTEST...');
      expect(result.tier).toBe('L3');
      expect(result.principal).toBe('500');
    });

    it('falls back to Flex when no active position exists', async () => {
      vi.spyOn(sdk as any, 'queryTierPosition').mockResolvedValue({
        tier: 'Flex', principal: '0', shares: '0', accruedYield: '0', lockUntil: null,
      });
      const result = await sdk.position('GNONE...');
      expect(result.tier).toBe('Flex');
    });
  });
});