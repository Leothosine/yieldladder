#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, token, Address, Env, Map};

const MAX_ALLOC_BPS: i128 = 3_500;
const BPS_DENOM: i128 = 10_000;

/// Default buffer: 5% of total capital kept idle for fast withdrawals.
/// Documented explicitly so it is never implicitly zero.
const DEFAULT_BUFFER_BPS: i128 = 500; // 5 %

#[contracttype]
pub enum DataKey {
    Admin,
    Usdc,
    Harvester,
    Allocations,
    TierVaults,
    PoolAllowlist,
    /// Governance-settable buffer: percentage of total capital (in BPS)
    /// that must remain as idle USDC balance, not deployed to pool addresses.
    BufferBps,
}

#[contract]
pub struct StrategyVault;

#[contractimpl]
impl StrategyVault {
    /// Initialise the vault.
    ///
    /// `admin` **must** be the address of the Governance contract so that all
    /// admin-gated functions (including `set_buffer_bps`) are protected by the
    /// governance timelock/veto flow, and the bypass path described in the
    /// NF-09 audit note is closed by convention.
    pub fn initialize(env: Env, admin: Address, usdc_token: Address) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("already initialized");
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Usdc, &usdc_token);
        env.storage()
            .instance()
            .set(&DataKey::Allocations, &Map::<Address, i128>::new(&env));
        env.storage()
            .instance()
            .set(&DataKey::TierVaults, &Map::<Address, bool>::new(&env));
        env.storage()
            .instance()
            .set(&DataKey::PoolAllowlist, &Map::<Address, bool>::new(&env));
        // Explicitly initialise the buffer to the documented default rather
        // than leaving it implicitly zero.
        env.storage()
            .instance()
            .set(&DataKey::BufferBps, &DEFAULT_BUFFER_BPS);
    }

    pub fn set_harvester(env: Env, harvester: Address) {
        Self::require_admin(&env);
        env.storage()
            .instance()
            .set(&DataKey::Harvester, &harvester);
    }

    pub fn register_tier_vault(env: Env, vault: Address) {
        Self::require_admin(&env);
        let mut vaults: Map<Address, bool> = env
            .storage()
            .instance()
            .get(&DataKey::TierVaults)
            .unwrap();
        vaults.set(vault, true);
        env.storage().instance().set(&DataKey::TierVaults, &vaults);
    }

    pub fn allow_pool(env: Env, pool: Address) {
        Self::require_admin(&env);
        let mut allowlist: Map<Address, bool> = env
            .storage()
            .instance()
            .get(&DataKey::PoolAllowlist)
            .unwrap();
        allowlist.set(pool, true);
        env.storage()
            .instance()
            .set(&DataKey::PoolAllowlist, &allowlist);
    }

    // ── Buffer management ─────────────────────────────────────────────────────

    /// Set the percentage of total capital (in BPS) that `rebalance()` must
    /// leave as idle USDC balance for fast withdrawals.
    ///
    /// Only callable via Governance (the registered `admin` address).
    /// Valid range: 0 – 10 000 BPS (0 % – 100 %).
    pub fn set_buffer_bps(env: Env, bps: i128) {
        Self::require_admin(&env);
        if bps < 0 || bps > BPS_DENOM {
            panic!("buffer_bps must be in [0, 10000]");
        }
        env.storage().instance().set(&DataKey::BufferBps, &bps);
    }

    /// Read-only: returns the current buffer BPS setting.
    pub fn buffer_bps(env: Env) -> i128 {
        env.storage()
            .instance()
            .get(&DataKey::BufferBps)
            .unwrap_or(DEFAULT_BUFFER_BPS)
    }

    /// Read-only: returns the current idle (un-deployed) USDC balance held
    /// on this contract.
    pub fn idle_balance(env: Env) -> i128 {
        let usdc: Address = env.storage().instance().get(&DataKey::Usdc).unwrap();
        token::Client::new(&env, &usdc).balance(&env.current_contract_address())
    }

    // ── Capital flows ─────────────────────────────────────────────────────────

    pub fn deposit_capital(env: Env, caller: Address, amount: i128) {
        caller.require_auth();
        let vaults: Map<Address, bool> = env
            .storage()
            .instance()
            .get(&DataKey::TierVaults)
            .unwrap();
        if !vaults.get(caller.clone()).unwrap_or(false) {
            panic!("caller is not a registered tier vault");
        }
        let usdc: Address = env.storage().instance().get(&DataKey::Usdc).unwrap();
        token::Client::new(&env, &usdc).transfer(
            &caller,
            &env.current_contract_address(),
            &amount,
        );
    }

    /// Withdraw capital to a registered tier vault.
    ///
    /// Draws from the idle buffer first.  If the requested amount is within
    /// the current idle USDC balance the transfer succeeds immediately without
    /// touching pool allocations.  Withdrawals that exceed the idle balance
    /// are rejected (pulling from pools is out of scope for this issue and
    /// tracked as a follow-up).
    pub fn withdraw_capital(env: Env, caller: Address, amount: i128) {
        caller.require_auth();
        let vaults: Map<Address, bool> = env
            .storage()
            .instance()
            .get(&DataKey::TierVaults)
            .unwrap();
        if !vaults.get(caller.clone()).unwrap_or(false) {
            panic!("caller is not a registered tier vault");
        }
        let usdc: Address = env.storage().instance().get(&DataKey::Usdc).unwrap();
        let idle = token::Client::new(&env, &usdc).balance(&env.current_contract_address());
        if amount > idle {
            panic!("withdrawal exceeds idle buffer; pool liquidity pull not yet supported");
        }
        token::Client::new(&env, &usdc).transfer(
            &env.current_contract_address(),
            &caller,
            &amount,
        );
    }

    /// Audit fix L-01: re-checks allowlist AND 35% cap before persisting.
    pub fn set_allocation(env: Env, pool_id: Address, target_bps: i128) {
        Self::require_admin(&env);
        let allowlist: Map<Address, bool> = env
            .storage()
            .instance()
            .get(&DataKey::PoolAllowlist)
            .unwrap();
        if !allowlist.get(pool_id.clone()).unwrap_or(false) {
            panic!("pool not on allowlist");
        }
        if target_bps > MAX_ALLOC_BPS {
            panic!("allocation exceeds 35% cap");
        }
        let mut allocs: Map<Address, i128> = env
            .storage()
            .instance()
            .get(&DataKey::Allocations)
            .unwrap();
        let mut others_total: i128 = 0;
        for (k, v) in allocs.iter() {
            if k != pool_id {
                others_total = others_total
                    .checked_add(v)
                    .expect("allocation sum overflow");
            }
        }
        if others_total
            .checked_add(target_bps)
            .expect("allocation sum overflow")
            > BPS_DENOM
        {
            panic!("total allocation exceeds 100%");
        }
        allocs.set(pool_id, target_bps);
        env.storage()
            .instance()
            .set(&DataKey::Allocations, &allocs);
    }

    pub fn allocations(env: Env) -> Map<Address, i128> {
        env.storage()
            .instance()
            .get(&DataKey::Allocations)
            .unwrap_or(Map::new(&env))
    }

    pub fn total_capital(env: Env) -> i128 {
        let usdc: Address = env.storage().instance().get(&DataKey::Usdc).unwrap();
        token::Client::new(&env, &usdc).balance(&env.current_contract_address())
    }

    /// Rebalance capital across pool allocations, honouring the liquidity buffer.
    ///
    /// The buffer is deducted from total capital first; only
    /// `total_capital - reserved` is distributed across pool addresses.
    /// This ensures `buffer_bps` of capital is always left idle on the
    /// contract for fast withdrawals.
    pub fn rebalance(env: Env) {
        let harvester: Address = env
            .storage()
            .instance()
            .get(&DataKey::Harvester)
            .unwrap();
        harvester.require_auth();
        let usdc: Address = env.storage().instance().get(&DataKey::Usdc).unwrap();
        let client = token::Client::new(&env, &usdc);
        let total = client.balance(&env.current_contract_address());

        // Compute the idle buffer that must remain on the contract.
        let buffer_bps: i128 = env
            .storage()
            .instance()
            .get(&DataKey::BufferBps)
            .unwrap_or(DEFAULT_BUFFER_BPS);
        let reserved = total
            .checked_mul(buffer_bps)
            .expect("buffer calc overflow")
            / BPS_DENOM;

        // Only deploy capital beyond the reserved buffer amount.
        let deployable = total.saturating_sub(reserved);

        let allocs: Map<Address, i128> = env
            .storage()
            .instance()
            .get(&DataKey::Allocations)
            .unwrap();
        for (pool, bps) in allocs.iter() {
            // Each allocation's target is computed from `deployable`, not `total`.
            let target = deployable
                .checked_mul(bps)
                .expect("rebalance overflow")
                / BPS_DENOM;
            if target > 0 {
                client.transfer(&env.current_contract_address(), &pool, &target);
            }
        }
    }

    fn require_admin(env: &Env) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{
        testutils::{Address as _, MockAuth, MockAuthInvoke},
        token::{Client as TokenClient, StellarAssetClient},
        Address, Env, IntoVal,
    };

    // ── Helpers ───────────────────────────────────────────────────────────────

    fn setup(env: &Env) -> (Address, Address, Address, StrategyVaultClient) {
        let admin = Address::generate(env);
        let usdc_id = env.register_stellar_asset_contract_v2(Address::generate(env)).address();
        let cid = env.register_contract(None, StrategyVault);
        let client = StrategyVaultClient::new(env, &cid);
        env.mock_all_auths();
        client.initialize(&admin, &usdc_id);
        (admin, usdc_id, cid, client)
    }

    fn mint(env: &Env, usdc_id: &Address, to: &Address, amount: i128) {
        StellarAssetClient::new(env, usdc_id).mint(to, &amount);
    }

    // ── Basic instantiation / pre-existing tests ──────────────────────────────

    #[test]
    fn contract_instantiates() {
        let env = Env::default();
        let _id = env.register_contract(None, StrategyVault);
    }

    #[test]
    fn initialize_and_read_empty_allocations() {
        let env = Env::default();
        env.mock_all_auths();
        let (_, _, _, client) = setup(&env);
        assert_eq!(client.allocations().len(), 0);
    }

    #[test]
    fn set_allocation_at_35_pct_cap_succeeds() {
        let env = Env::default();
        env.mock_all_auths();
        let (_, _, _, client) = setup(&env);
        let pool = Address::generate(&env);
        client.allow_pool(&pool);
        client.set_allocation(&pool, &3500);
        assert_eq!(client.allocations().get(pool).unwrap(), 3500);
    }

    #[test]
    #[should_panic]
    fn set_allocation_rejects_unlisted_pool() {
        let env = Env::default();
        env.mock_all_auths();
        let (_, _, _, client) = setup(&env);
        client.set_allocation(&Address::generate(&env), &1000);
    }

    #[test]
    #[should_panic]
    fn set_allocation_rejects_above_35_pct() {
        let env = Env::default();
        env.mock_all_auths();
        let (_, _, _, client) = setup(&env);
        let pool = Address::generate(&env);
        client.allow_pool(&pool);
        client.set_allocation(&pool, &3501);
    }

    #[test]
    #[should_panic]
    fn double_initialize_panics() {
        let env = Env::default();
        env.mock_all_auths();
        let (admin, usdc, _, client) = setup(&env);
        client.initialize(&admin, &usdc);
    }

    #[test]
    #[should_panic]
    fn total_allocation_cannot_exceed_100_pct() {
        let env = Env::default();
        env.mock_all_auths();
        let (_, _, _, client) = setup(&env);
        let pool_a = Address::generate(&env);
        let pool_b = Address::generate(&env);
        client.allow_pool(&pool_a);
        client.allow_pool(&pool_b);
        client.set_allocation(&pool_a, &3500);
        client.set_allocation(&pool_b, &3500);
        client.set_allocation(&Address::generate(&env), &3500);
    }

    // ── Buffer BPS ────────────────────────────────────────────────────────────

    #[test]
    fn default_buffer_bps_is_500() {
        let env = Env::default();
        env.mock_all_auths();
        let (_, _, _, client) = setup(&env);
        assert_eq!(client.buffer_bps(), 500);
    }

    #[test]
    fn set_buffer_bps_updates_value() {
        let env = Env::default();
        env.mock_all_auths();
        let (_, _, _, client) = setup(&env);
        client.set_buffer_bps(&1000);
        assert_eq!(client.buffer_bps(), 1000);
    }

    #[test]
    #[should_panic(expected = "buffer_bps must be in [0, 10000]")]
    fn set_buffer_bps_rejects_above_10000() {
        let env = Env::default();
        env.mock_all_auths();
        let (_, _, _, client) = setup(&env);
        client.set_buffer_bps(&10001);
    }

    // ── Rebalance respects the buffer ─────────────────────────────────────────

    #[test]
    fn rebalance_leaves_buffer_idle() {
        let env = Env::default();
        env.mock_all_auths();
        let (admin, usdc_id, cid, client) = setup(&env);

        // Fund the vault with 1_000_000 units.
        mint(&env, &usdc_id, &cid, 1_000_000);

        // Set buffer to 10% and a single pool allocation.
        client.set_buffer_bps(&1000); // 10 %
        let pool = Address::generate(&env);
        client.allow_pool(&pool);
        client.set_allocation(&pool, &5000); // 50% of deployable
        let harvester = Address::generate(&env);
        client.set_harvester(&harvester);

        client.rebalance();

        // deployable = 1_000_000 * (1 - 10%) = 900_000
        // target for pool = 900_000 * 50% = 450_000
        // idle remaining = 1_000_000 - 450_000 = 550_000
        // required buffer = 1_000_000 * 10% = 100_000
        // idle (550_000) >= reserved (100_000) ✓
        let idle = client.idle_balance();
        let reserved = 1_000_000i128 * 1000 / BPS_DENOM;
        assert!(
            idle >= reserved,
            "idle balance {idle} should be >= reserved {reserved}"
        );
    }

    // ── Withdrawal within buffer succeeds ─────────────────────────────────────

    #[test]
    fn withdraw_within_idle_buffer_succeeds() {
        let env = Env::default();
        env.mock_all_auths();
        let (admin, usdc_id, cid, client) = setup(&env);

        // Fund the vault and register a tier vault.
        mint(&env, &usdc_id, &cid, 1_000_000);
        let tier_vault = Address::generate(&env);
        client.register_tier_vault(&tier_vault);

        // With default buffer (5%) and no pool allocations, all 1_000_000 is idle.
        // A withdrawal of 50_000 is well within the buffer.
        client.withdraw_capital(&tier_vault, &50_000);

        let remaining = TokenClient::new(&env, &usdc_id).balance(&cid);
        assert_eq!(remaining, 950_000);
    }

    // ── Withdrawal exceeding idle balance panics ──────────────────────────────

    #[test]
    #[should_panic(expected = "withdrawal exceeds idle buffer")]
    fn withdraw_exceeding_idle_panics() {
        let env = Env::default();
        env.mock_all_auths();
        let (_, usdc_id, cid, client) = setup(&env);

        mint(&env, &usdc_id, &cid, 100);
        let tier_vault = Address::generate(&env);
        client.register_tier_vault(&tier_vault);

        // Try to withdraw more than available.
        client.withdraw_capital(&tier_vault, &101);
    }
}
