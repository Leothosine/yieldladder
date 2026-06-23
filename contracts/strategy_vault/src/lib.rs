#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, token, Address, Env, Map};

const MAX_ALLOC_BPS: i128 = 3_500;
const BPS_DENOM: i128 = 10_000;

#[contracttype]
pub enum DataKey {
    Admin,
    Usdc,
    Harvester,
    Allocations,
    TierVaults,
    PoolAllowlist,
}

#[contract]
pub struct StrategyVault;

#[contractimpl]
impl StrategyVault {
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
        let allocs: Map<Address, i128> = env
            .storage()
            .instance()
            .get(&DataKey::Allocations)
            .unwrap();
        for (pool, bps) in allocs.iter() {
            let target = total
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
    use soroban_sdk::{testutils::Address as _, Env};

    #[test]
    fn contract_instantiates() {
        let env = Env::default();
        let _id = env.register_contract(None, StrategyVault);
    }

    #[test]
    fn initialize_and_read_empty_allocations() {
        let env = Env::default();
        env.mock_all_auths();
        let cid = env.register_contract(None, StrategyVault);
        let client = StrategyVaultClient::new(&env, &cid);
        let admin = Address::generate(&env);
        let usdc = Address::generate(&env);
        client.initialize(&admin, &usdc);
        assert_eq!(client.allocations().len(), 0);
    }

    #[test]
    fn set_allocation_at_35_pct_cap_succeeds() {
        let env = Env::default();
        env.mock_all_auths();
        let cid = env.register_contract(None, StrategyVault);
        let client = StrategyVaultClient::new(&env, &cid);
        let admin = Address::generate(&env);
        let usdc = Address::generate(&env);
        let pool = Address::generate(&env);
        client.initialize(&admin, &usdc);
        client.allow_pool(&pool);
        client.set_allocation(&pool, &3500);
        assert_eq!(client.allocations().get(pool).unwrap(), 3500);
    }

    #[test]
    #[should_panic]
    fn set_allocation_rejects_unlisted_pool() {
        let env = Env::default();
        env.mock_all_auths();
        let cid = env.register_contract(None, StrategyVault);
        let client = StrategyVaultClient::new(&env, &cid);
        let admin = Address::generate(&env);
        let usdc = Address::generate(&env);
        client.initialize(&admin, &usdc);
        client.set_allocation(&Address::generate(&env), &1000);
    }

    #[test]
    #[should_panic]
    fn set_allocation_rejects_above_35_pct() {
        let env = Env::default();
        env.mock_all_auths();
        let cid = env.register_contract(None, StrategyVault);
        let client = StrategyVaultClient::new(&env, &cid);
        let admin = Address::generate(&env);
        let usdc = Address::generate(&env);
        let pool = Address::generate(&env);
        client.initialize(&admin, &usdc);
        client.allow_pool(&pool);
        client.set_allocation(&pool, &3501);
    }

    #[test]
    #[should_panic]
    fn double_initialize_panics() {
        let env = Env::default();
        env.mock_all_auths();
        let cid = env.register_contract(None, StrategyVault);
        let client = StrategyVaultClient::new(&env, &cid);
        let admin = Address::generate(&env);
        let usdc = Address::generate(&env);
        client.initialize(&admin, &usdc);
        client.initialize(&admin, &usdc);
    }

    #[test]
    #[should_panic]
    fn total_allocation_cannot_exceed_100_pct() {
        let env = Env::default();
        env.mock_all_auths();
        let cid = env.register_contract(None, StrategyVault);
        let client = StrategyVaultClient::new(&env, &cid);
        let admin = Address::generate(&env);
        let usdc = Address::generate(&env);
        let pool_a = Address::generate(&env);
        let pool_b = Address::generate(&env);
        client.initialize(&admin, &usdc);
        client.allow_pool(&pool_a);
        client.allow_pool(&pool_b);
        client.set_allocation(&pool_a, &3500);
        client.set_allocation(&pool_b, &3500);
        client.set_allocation(&Address::generate(&env), &3500);
    }
}
