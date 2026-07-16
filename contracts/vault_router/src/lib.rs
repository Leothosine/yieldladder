#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, panic_with_error, token, Address, Env, IntoVal, Symbol,
    Val, Vec,
};

mod error;
pub use error::VaultError;

const MIN_FLEX: i128 = 10_000_000;
const MIN_L3: i128 = 500_000_000;
const MIN_L6: i128 = 1_000_000_000;
const MIN_L12: i128 = 2_500_000_000;

#[contracttype]
#[derive(Clone, PartialEq, Debug)]
pub enum Tier {
    Flex,
    L3,
    L6,
    L12,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct Position {
    pub principal: i128,
    pub shares: i128,
    pub lock_until: u32,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct VaultCapacity {
    pub max_tvl: i128,
    pub remaining: i128,
}

#[contracttype]
enum DataKey {
    Admin,
    Governance,
    VaultFlex,
    VaultL3,
    VaultL6,
    VaultL12,
    UsdcToken,
}

#[contract]
pub struct VaultRouter;

#[contractimpl]
impl VaultRouter {
    pub fn initialize(
        env: Env,
        admin: Address,
        governance: Address,
        vault_flex: Address,
        vault_l3: Address,
        vault_l6: Address,
        vault_l12: Address,
        usdc_token: Address,
    ) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("already initialized");
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Governance, &governance);
        env.storage().instance().set(&DataKey::VaultFlex, &vault_flex);
        env.storage().instance().set(&DataKey::VaultL3, &vault_l3);
        env.storage().instance().set(&DataKey::VaultL6, &vault_l6);
        env.storage().instance().set(&DataKey::VaultL12, &vault_l12);
        env.storage().instance().set(&DataKey::UsdcToken, &usdc_token);
    }

    /// Route a deposit to the appropriate tier vault.
    /// The tier vault enforces both the minimum deposit and the TVL cap.
    pub fn deposit(env: Env, user: Address, tier: Tier, amount: i128) {
        user.require_auth();

        let min_amt = min_deposit(&tier);
        if amount < min_amt {
            panic_with_error!(&env, VaultError::BelowMinDeposit);
        }

        let vault = vault_addr(&env, &tier);
        let usdc: Address = env.storage().instance().get(&DataKey::UsdcToken).unwrap();

        token::Client::new(&env, &usdc).transfer(&user, &vault, &amount);

        let args: Vec<Val> = (user, amount).into_val(&env);
        env.invoke_contract::<()>(&vault, &Symbol::new(&env, "deposit"), args);
    }

    pub fn withdraw(env: Env, user: Address, tier: Tier) {
        user.require_auth();

        let vault = vault_addr(&env, &tier);
        let usdc: Address = env.storage().instance().get(&DataKey::UsdcToken).unwrap();

        let args: Vec<Val> = (user.clone(),).into_val(&env);
        let payout: i128 = env.invoke_contract(&vault, &Symbol::new(&env, "withdraw"), args);

        if payout > 0 {
            token::Client::new(&env, &usdc).transfer(&vault, &user, &payout);
        }
    }

    pub fn early_exit(env: Env, user: Address, tier: Tier) {
        user.require_auth();

        let vault = vault_addr(&env, &tier);
        let usdc: Address = env.storage().instance().get(&DataKey::UsdcToken).unwrap();

        let args: Vec<Val> = (user.clone(),).into_val(&env);
        let net: i128 = env.invoke_contract(&vault, &Symbol::new(&env, "early_exit"), args);

        if net > 0 {
            token::Client::new(&env, &usdc).transfer(&vault, &user, &net);
        }
    }

    /// Update the TVL cap of a given tier vault.
    /// Only callable by the registered Governance address.
    pub fn set_max_tvl(env: Env, tier: Tier, new_cap: i128) {
        let governance: Address = env.storage().instance().get(&DataKey::Governance).unwrap();
        governance.require_auth();

        let vault = vault_addr(&env, &tier);
        let args: Vec<Val> = (new_cap,).into_val(&env);
        env.invoke_contract::<()>(&vault, &Symbol::new(&env, "set_max_tvl"), args);
    }

    /// Read the current TVL cap and remaining capacity for a tier vault.
    pub fn vault_capacity(env: Env, tier: Tier) -> VaultCapacity {
        let vault = vault_addr(&env, &tier);

        let max_tvl: i128 = env.invoke_contract(
            &vault,
            &Symbol::new(&env, "max_tvl"),
            Vec::new(&env),
        );
        let remaining: i128 = env.invoke_contract(
            &vault,
            &Symbol::new(&env, "remaining_capacity"),
            Vec::new(&env),
        );

        VaultCapacity { max_tvl, remaining }
    }

    pub fn position(env: Env, user: Address, tier: Tier) -> Position {
        let vault = vault_addr(&env, &tier);

        let principal: i128 = env.invoke_contract(
            &vault,
            &Symbol::new(&env, "balance"),
            (user.clone(),).into_val(&env),
        );
        let shares: i128 = env.invoke_contract(
            &vault,
            &Symbol::new(&env, "shares"),
            (user.clone(),).into_val(&env),
        );
        let lock_until: u32 = match tier {
            Tier::Flex => 0,
            _ => env.invoke_contract(
                &vault,
                &Symbol::new(&env, "lock_until"),
                (user,).into_val(&env),
            ),
        };

        Position { principal, shares, lock_until }
    }

    pub fn get_admin(env: Env) -> Address {
        env.storage().instance().get(&DataKey::Admin).unwrap()
    }

    pub fn get_vault(env: Env, tier: Tier) -> Address {
        vault_addr(&env, &tier)
    }
}

fn vault_addr(env: &Env, tier: &Tier) -> Address {
    match tier {
        Tier::Flex => env.storage().instance().get(&DataKey::VaultFlex).unwrap(),
        Tier::L3 => env.storage().instance().get(&DataKey::VaultL3).unwrap(),
        Tier::L6 => env.storage().instance().get(&DataKey::VaultL6).unwrap(),
        Tier::L12 => env.storage().instance().get(&DataKey::VaultL12).unwrap(),
    }
}

fn min_deposit(tier: &Tier) -> i128 {
    match tier {
        Tier::Flex => MIN_FLEX,
        Tier::L3 => MIN_L3,
        Tier::L6 => MIN_L6,
        Tier::L12 => MIN_L12,
    }
}

#[cfg(test)]
mod test {
    extern crate std;

    use super::{min_deposit, Tier, VaultRouter, MIN_FLEX, MIN_L12, MIN_L3, MIN_L6};
    use soroban_sdk::{contract, contractimpl, testutils::Address as _, Address, Env};

    #[contract]
    pub struct MockVault;

    #[contractimpl]
    impl MockVault {
        pub fn deposit(_env: Env, _user: Address, _amount: i128) {}
        pub fn withdraw(_env: Env, _user: Address) -> i128 { 500_000_000_i128 }
        pub fn early_exit(_env: Env, _user: Address) -> i128 { 497_500_000_i128 }
        pub fn balance(_env: Env, _user: Address) -> i128 { 500_000_000_i128 }
        pub fn shares(_env: Env, _user: Address) -> i128 { 525_000_000_i128 }
        pub fn lock_until(_env: Env, _user: Address) -> u32 { 1_000_000_u32 }
        pub fn set_max_tvl(_env: Env, _new_cap: i128) {}
        pub fn max_tvl(_env: Env) -> i128 { 10_000_000_000_i128 }
        pub fn remaining_capacity(_env: Env) -> i128 { 9_500_000_000_i128 }
    }

    fn setup() -> (Env, super::VaultRouterClient<'static>, Address, Address) {
        let env = Env::default();
        env.mock_all_auths();
        let vault_id = env.register_contract(None, MockVault);
        let router_id = env.register_contract(None, VaultRouter);
        let client = super::VaultRouterClient::new(&env, &router_id);
        let admin = Address::generate(&env);
        let governance = Address::generate(&env);
        let usdc = Address::generate(&env);
        client.initialize(&admin, &governance, &vault_id, &vault_id, &vault_id, &vault_id, &usdc);
        (env, client, admin, governance)
    }

    #[test]
    #[should_panic]
    fn test_flex_below_min_panics() {
        let (env, client, _, _) = setup();
        let user = Address::generate(&env);
        client.deposit(&user, &Tier::Flex, &(MIN_FLEX - 1));
    }

    #[test]
    #[should_panic]
    fn test_l3_below_min_panics() {
        let (env, client, _, _) = setup();
        let user = Address::generate(&env);
        client.deposit(&user, &Tier::L3, &(MIN_L3 - 1));
    }

    #[test]
    #[should_panic]
    fn test_l6_below_min_panics() {
        let (env, client, _, _) = setup();
        let user = Address::generate(&env);
        client.deposit(&user, &Tier::L6, &(MIN_L6 - 1));
    }

    #[test]
    #[should_panic]
    fn test_l12_below_min_panics() {
        let (env, client, _, _) = setup();
        let user = Address::generate(&env);
        client.deposit(&user, &Tier::L12, &(MIN_L12 - 1));
    }

    #[test]
    fn test_set_max_tvl_via_router() {
        let (env, client, _, _) = setup();
        client.set_max_tvl(&Tier::L3, &5_000_000_000_i128);
    }

    #[test]
    fn test_vault_capacity_query() {
        let (env, client, _, _) = setup();
        let cap = client.vault_capacity(&Tier::L3);
        assert_eq!(cap.max_tvl, 10_000_000_000_i128);
        assert_eq!(cap.remaining, 9_500_000_000_i128);
    }

    #[test]
    fn test_position_locked_tier() {
        let (env, client, _, _) = setup();
        let user = Address::generate(&env);
        let pos = client.position(&user, &Tier::L3);
        assert_eq!(pos.principal, 500_000_000_i128);
        assert_eq!(pos.lock_until, 1_000_000_u32);
    }

    #[test]
    #[should_panic(expected = "already initialized")]
    fn test_double_initialize_panics() {
        let (env, client, _, _) = setup();
        let vault_id = Address::generate(&env);
        let admin = Address::generate(&env);
        let gov = Address::generate(&env);
        let usdc = Address::generate(&env);
        client.initialize(&admin, &gov, &vault_id, &vault_id, &vault_id, &vault_id, &usdc);
    }

    #[test]
    fn test_min_deposit_values() {
        assert_eq!(min_deposit(&Tier::Flex), MIN_FLEX);
        assert_eq!(min_deposit(&Tier::L3), MIN_L3);
        assert_eq!(min_deposit(&Tier::L6), MIN_L6);
        assert_eq!(min_deposit(&Tier::L12), MIN_L12);
    }
}