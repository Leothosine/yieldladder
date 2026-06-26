#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, panic_with_error, token, Address, Env, IntoVal, Symbol,
    Val, Vec,
};

mod error;
pub use error::VaultError;

// ---------------------------------------------------------------------------
// Constants — minimum deposit per tier (USDC, 7 decimal places)
// ---------------------------------------------------------------------------

const MIN_FLEX: i128 = 10_000_000; // 1 USDC
const MIN_L3: i128 = 500_000_000; // 50 USDC
const MIN_L6: i128 = 1_000_000_000; // 100 USDC
const MIN_L12: i128 = 2_500_000_000; // 250 USDC

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Storage keys
// ---------------------------------------------------------------------------

#[contracttype]
enum DataKey {
    Admin,
    VaultFlex,
    VaultL3,
    VaultL6,
    VaultL12,
    UsdcToken,
}

// ---------------------------------------------------------------------------
// Contract
// ---------------------------------------------------------------------------

#[contract]
pub struct VaultRouter;

#[contractimpl]
impl VaultRouter {
    /// One-time setup. Registers the admin and all tier vault addresses.
    pub fn initialize(
        env: Env,
        admin: Address,
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
        env.storage().instance().set(&DataKey::VaultFlex, &vault_flex);
        env.storage().instance().set(&DataKey::VaultL3, &vault_l3);
        env.storage().instance().set(&DataKey::VaultL6, &vault_l6);
        env.storage().instance().set(&DataKey::VaultL12, &vault_l12);
        env.storage().instance().set(&DataKey::UsdcToken, &usdc_token);
    }

    /// Route a deposit to the appropriate tier vault.
    ///
    /// Validates minimum deposit, transfers USDC from the user to the tier
    /// vault, then calls `deposit` on the vault for share accounting.
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

    /// Withdraw from the chosen tier vault after the lock period has elapsed.
    ///
    /// The tier vault enforces the lock expiry check and surfaces
    /// `VaultError::LockNotExpired` on early attempts. On success, payout
    /// USDC is transferred from the vault to the user.
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

    /// Exit a locked tier early, accepting the exit fee deducted by the vault.
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

    /// Return the caller's position in the given tier vault (read-only).
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
        // VaultFlex has no lock period; locked tiers expose `lock_until`.
        let lock_until: u32 = match tier {
            Tier::Flex => 0,
            _ => env.invoke_contract(
                &vault,
                &Symbol::new(&env, "lock_until"),
                (user,).into_val(&env),
            ),
        };

        Position {
            principal,
            shares,
            lock_until,
        }
    }

    pub fn get_admin(env: Env) -> Address {
        env.storage().instance().get(&DataKey::Admin).unwrap()
    }

    pub fn get_vault(env: Env, tier: Tier) -> Address {
        vault_addr(&env, &tier)
    }
}

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod test {
    extern crate std;

    use super::{min_deposit, Tier, VaultRouter, MIN_FLEX, MIN_L12, MIN_L3, MIN_L6};
    use soroban_sdk::{contract, contractimpl, testutils::Address as _, Address, Env};

    // ── Minimal mock tier vault ─────────────────────────────────────────────

    #[contract]
    pub struct MockVault;

    #[contractimpl]
    impl MockVault {
        pub fn deposit(_env: Env, _user: Address, _amount: i128) {}
        pub fn withdraw(_env: Env, _user: Address) -> i128 {
            500_000_000_i128
        }
        pub fn early_exit(_env: Env, _user: Address) -> i128 {
            497_500_000_i128
        }
        pub fn balance(_env: Env, _user: Address) -> i128 {
            500_000_000_i128
        }
        pub fn shares(_env: Env, _user: Address) -> i128 {
            525_000_000_i128
        }
        pub fn lock_until(_env: Env, _user: Address) -> u32 {
            1_000_000_u32
        }
    }

    fn setup() -> (Env, super::VaultRouterClient<'static>, Address) {
        let env = Env::default();
        env.mock_all_auths();

        let vault_id = env.register_contract(None, MockVault);
        let router_id = env.register_contract(None, VaultRouter);
        let client = super::VaultRouterClient::new(&env, &router_id);

        let admin = Address::generate(&env);
        let usdc = Address::generate(&env);
        client.initialize(&admin, &vault_id, &vault_id, &vault_id, &vault_id, &usdc);

        (env, client, vault_id)
    }

    // ── Minimum-deposit guard ───────────────────────────────────────────────

    #[test]
    #[should_panic]
    fn test_flex_below_min_panics() {
        let (env, client, _) = setup();
        let user = Address::generate(&env);
        client.deposit(&user, &Tier::Flex, &(MIN_FLEX - 1));
    }

    #[test]
    #[should_panic]
    fn test_l3_below_min_panics() {
        let (env, client, _) = setup();
        let user = Address::generate(&env);
        client.deposit(&user, &Tier::L3, &(MIN_L3 - 1));
    }

    #[test]
    #[should_panic]
    fn test_l6_below_min_panics() {
        let (env, client, _) = setup();
        let user = Address::generate(&env);
        client.deposit(&user, &Tier::L6, &(MIN_L6 - 1));
    }

    #[test]
    #[should_panic]
    fn test_l12_below_min_panics() {
        let (env, client, _) = setup();
        let user = Address::generate(&env);
        client.deposit(&user, &Tier::L12, &(MIN_L12 - 1));
    }

    // ── Position queries ────────────────────────────────────────────────────

    #[test]
    fn test_position_locked_tier() {
        let (env, client, _) = setup();
        let user = Address::generate(&env);
        let pos = client.position(&user, &Tier::L3);
        assert_eq!(pos.principal, 500_000_000_i128);
        assert_eq!(pos.shares, 525_000_000_i128);
        assert_eq!(pos.lock_until, 1_000_000_u32);
    }

    #[test]
    fn test_position_flex_lock_until_zero() {
        let (env, client, _) = setup();
        let user = Address::generate(&env);
        let pos = client.position(&user, &Tier::Flex);
        assert_eq!(pos.lock_until, 0_u32);
    }

    #[test]
    fn test_position_all_tiers() {
        let (env, client, _) = setup();
        let user = Address::generate(&env);
        for tier in [Tier::Flex, Tier::L3, Tier::L6, Tier::L12] {
            let pos = client.position(&user, &tier);
            assert_eq!(pos.principal, 500_000_000_i128);
        }
    }

    // ── Init guard ──────────────────────────────────────────────────────────

    #[test]
    #[should_panic(expected = "already initialized")]
    fn test_double_initialize_panics() {
        let (env, client, vault_id) = setup();
        let admin = Address::generate(&env);
        let usdc = Address::generate(&env);
        client.initialize(&admin, &vault_id, &vault_id, &vault_id, &vault_id, &usdc);
    }

    // ── Vault getter ────────────────────────────────────────────────────────

    #[test]
    fn test_get_vault_returns_registered_address() {
        let (env, client, vault_id) = setup();
        assert_eq!(client.get_vault(&Tier::Flex), vault_id);
        assert_eq!(client.get_vault(&Tier::L3), vault_id);
        assert_eq!(client.get_vault(&Tier::L6), vault_id);
        assert_eq!(client.get_vault(&Tier::L12), vault_id);
    }

    // ── Routing smoke tests ─────────────────────────────────────────────────

    #[test]
    fn test_deposit_routes_to_vault() {
        let (env, client, _) = setup();
        let user = Address::generate(&env);
        client.deposit(&user, &Tier::Flex, &MIN_FLEX);
    }

    #[test]
    fn test_deposit_l12_exact_min() {
        let (env, client, _) = setup();
        let user = Address::generate(&env);
        client.deposit(&user, &Tier::L12, &MIN_L12);
    }

    #[test]
    fn test_withdraw_routes_to_vault() {
        let (env, client, _) = setup();
        let user = Address::generate(&env);
        client.withdraw(&user, &Tier::L3);
    }

    #[test]
    fn test_early_exit_routes_to_vault() {
        let (env, client, _) = setup();
        let user = Address::generate(&env);
        client.early_exit(&user, &Tier::L6);
    }

    // ── Helper min-deposit validation ───────────────────────────────────────

    #[test]
    fn test_min_deposit_values() {
        assert_eq!(min_deposit(&Tier::Flex), MIN_FLEX);
        assert_eq!(min_deposit(&Tier::L3), MIN_L3);
        assert_eq!(min_deposit(&Tier::L6), MIN_L6);
        assert_eq!(min_deposit(&Tier::L12), MIN_L12);
    }
}
