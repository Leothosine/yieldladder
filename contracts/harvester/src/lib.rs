#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, token, Address, Env};

const BOUNTY_BPS: i128 = 10;
const BPS_DENOMINATOR: i128 = 10_000;

#[contracttype]
pub enum DataKey {
    Strategy,
    Usdc,
    LastHarvestLedger,
    CooldownLedgers,
}

/// Permissionless yield harvester that claims accumulated yield and compounds
/// it into [`StrategyVault`].
///
/// Any address may call `harvest()` after the cooldown elapses and earns a
/// 10 bps bounty on the total harvested amount.
#[contract]
pub struct Harvester;

#[contractimpl]
impl Harvester {
    /// One-time initialisation.  Stores the StrategyVault address, the USDC
    /// token contract address, and the minimum ledger gap between harvests.
    pub fn initialize(env: Env, strategy: Address, usdc: Address, cooldown_ledgers: u32) {
        if env.storage().instance().has(&DataKey::Strategy) {
            panic!("already initialized");
        }
        env.storage().instance().set(&DataKey::Strategy, &strategy);
        env.storage().instance().set(&DataKey::Usdc, &usdc);
        env.storage()
            .instance()
            .set(&DataKey::CooldownLedgers, &cooldown_ledgers);
        env.storage()
            .instance()
            .set(&DataKey::LastHarvestLedger, &0u32);
        env.storage().instance().extend_ttl(17_280, 17_280);
    }

    /// Permissionless harvest entry-point.
    ///
    /// 1. Enforces cooldown via ledger sequence (audit fix L-02 — not timestamp).
    /// 2. Reads the contract's USDC balance as the harvestable amount.
    /// 3. Pays `caller` a 10 bps bounty.
    /// 4. Forwards the remainder to `StrategyVault` via `deposit_capital`.
    /// 5. Records the current ledger as the last harvest ledger.
    ///
    /// Returns the total harvested amount (0 if nothing to harvest).
    pub fn harvest(env: Env, caller: Address) -> i128 {
        caller.require_auth();

        let last: u32 = env
            .storage()
            .instance()
            .get(&DataKey::LastHarvestLedger)
            .unwrap_or(0);
        let cooldown: u32 = env
            .storage()
            .instance()
            .get(&DataKey::CooldownLedgers)
            .unwrap_or_else(|| panic!("not initialized"));

        // Audit fix L-02: compare ledger sequences, not timestamps.
        let current = env.ledger().sequence();
        if current < last.saturating_add(cooldown) {
            panic!("cooldown not elapsed");
        }

        let usdc_id: Address = env
            .storage()
            .instance()
            .get(&DataKey::Usdc)
            .unwrap_or_else(|| panic!("not initialized"));
        let contract_id = env.current_contract_address();
        let usdc = token::Client::new(&env, &usdc_id);
        let harvested = usdc.balance(&contract_id);

        // Zero-yield harvest: update ledger timestamp and return without transfers.
        if harvested == 0 {
            env.storage()
                .instance()
                .set(&DataKey::LastHarvestLedger, &current);
            env.storage().instance().extend_ttl(17_280, 17_280);
            return 0;
        }

        // 10 bps bounty — checked mul prevents overflow on large harvest amounts.
        let bounty = harvested
            .checked_mul(BOUNTY_BPS)
            .expect("bounty mul overflow")
            / BPS_DENOMINATOR;
        let remainder = harvested - bounty;

        let strategy: Address = env
            .storage()
            .instance()
            .get(&DataKey::Strategy)
            .unwrap_or_else(|| panic!("not initialized"));

        if bounty > 0 {
            usdc.transfer(&contract_id, &caller, &bounty);
        }
        if remainder > 0 {
            usdc.transfer(&contract_id, &strategy, &remainder);
        }

        // Atomic update — no re-entrancy window between transfers and ledger write.
        env.storage()
            .instance()
            .set(&DataKey::LastHarvestLedger, &current);
        env.storage().instance().extend_ttl(17_280, 17_280);

        harvested
    }

    /// Returns the ledger sequence of the most recent successful harvest.
    pub fn last_harvest(env: Env) -> u32 {
        env.storage()
            .instance()
            .get(&DataKey::LastHarvestLedger)
            .unwrap_or(0)
    }

    /// Returns the earliest ledger at which the next harvest may be called.
    pub fn next_harvest_ledger(env: Env) -> u32 {
        let last: u32 = env
            .storage()
            .instance()
            .get(&DataKey::LastHarvestLedger)
            .unwrap_or(0);
        let cooldown: u32 = env
            .storage()
            .instance()
            .get(&DataKey::CooldownLedgers)
            .unwrap_or(0);
        last.saturating_add(cooldown)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::Env;

    /// Pure arithmetic helper — mirrors the on-chain formula so tests remain
    /// independent of Soroban's token mock.
    fn split(harvested: i128) -> (i128, i128) {
        let bounty = harvested
            .checked_mul(BOUNTY_BPS)
            .expect("overflow in test helper")
            / BPS_DENOMINATOR;
        (bounty, harvested - bounty)
    }

    // ── Invariant / property tests (fuzz-style) ──────────────────────────────

    #[test]
    fn invariant_bounty_plus_remainder_equals_harvested() {
        let cases: &[i128] = &[
            1,
            9_999,
            10_000,
            100_001,
            1_000_000,
            1_000_000_000,
            1_000_000_000_000,
            i128::MAX / BOUNTY_BPS, // largest safe value — no overflow
        ];
        for &h in cases {
            let (b, r) = split(h);
            assert_eq!(
                b + r,
                h,
                "invariant violated for harvested={h}: bounty={b} remainder={r}"
            );
        }
    }

    #[test]
    fn bounty_is_exactly_ten_bps() {
        // 10 bps of 1_000_000 stroops (0.1 XLM) == 100 stroops
        let (b, _) = split(1_000_000);
        assert_eq!(b, 100);
    }

    #[test]
    fn zero_harvest_yields_zero_bounty_and_remainder() {
        let (b, r) = split(0);
        assert_eq!(b, 0);
        assert_eq!(r, 0);
    }

    #[test]
    fn small_amounts_round_bounty_toward_zero() {
        // 1 stroop: 1 * 10 / 10_000 == 0 — no fractional bounty paid
        let (b, r) = split(1);
        assert_eq!(b, 0);
        assert_eq!(r, 1);

        // 9_999 stroops: 9_999 * 10 / 10_000 == 9
        let (b2, r2) = split(9_999);
        assert_eq!(b2, 9);
        assert_eq!(b2 + r2, 9_999);
    }

    // ── Cooldown boundary check ───────────────────────────────────────────────

    #[test]
    fn cooldown_ledger_arithmetic_does_not_overflow_at_u32_max() {
        // last = u32::MAX, cooldown = 1 => saturating_add returns u32::MAX
        // current < u32::MAX is always false when current == u32::MAX
        let last = u32::MAX;
        let cooldown: u32 = 1;
        let threshold = last.saturating_add(cooldown);
        assert_eq!(threshold, u32::MAX);
    }

    // ── Instantiation ────────────────────────────────────────────────────────

    #[test]
    fn contract_instantiates() {
        let env = Env::default();
        let _id = env.register_contract(None, Harvester);
    }
}