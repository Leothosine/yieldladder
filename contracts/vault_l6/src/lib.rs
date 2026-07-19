#![no_std]
use soroban_sdk::{contract, contracterror, contractimpl, contracttype, panic_with_error, token, Address, Env};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum VaultError {
    BelowMinDeposit = 2,
    LockNotExpired = 3,
    NotYetMatured = 4,
}

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    LockUntil(Address),
    Balance(Address),
    Shares(Address),
    Checkpoint(Address),
    TotalShares,
    Admin,
    Strategy,
    Usdc,
}

const FP_MULTIPLIER: i128 = 1_000_000_0;

pub fn mul_fp(a: i128, b_fp: i128) -> i128 {
    (a * b_fp) / FP_MULTIPLIER
}

// 6-month lock duration in ledgers (~5 s/ledger)
const LOCK_DURATION: u32 = 1_555_200;

#[contract]
pub struct VaultL6;

#[contractimpl]
impl VaultL6 {
    pub fn initialize(env: Env, admin: Address, strategy: Address, usdc: Address) {
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Strategy, &strategy);
        env.storage().instance().set(&DataKey::Usdc, &usdc);
        env.storage().instance().set(&DataKey::TotalShares, &0i128);
    }

    pub fn deposit(env: Env, user: Address, amount: i128) {
        user.require_auth();

        if amount < 1_000_000_000 {
            panic_with_error!(&env, VaultError::BelowMinDeposit);
        }

        let multiplier_fp = 11_500_000;
        let new_shares = mul_fp(amount, multiplier_fp);

        let usdc_addr: Address = env.storage().instance().get(&DataKey::Usdc).unwrap();
        let strategy: Address = env.storage().instance().get(&DataKey::Strategy).unwrap();

        let token_client = token::Client::new(&env, &usdc_addr);
        token_client.transfer(&user, &strategy, &amount);

        let current_balance: i128 = env.storage().persistent().get(&DataKey::Balance(user.clone())).unwrap_or(0);
        let current_shares: i128 = env.storage().persistent().get(&DataKey::Shares(user.clone())).unwrap_or(0);

        env.storage().persistent().set(&DataKey::Balance(user.clone()), &(current_balance + amount));
        env.storage().persistent().set(&DataKey::Shares(user.clone()), &(current_shares + new_shares));

        let total_shares: i128 = env.storage().instance().get(&DataKey::TotalShares).unwrap_or(0);
        env.storage().instance().set(&DataKey::TotalShares, &(total_shares + new_shares));

        let lock_until = env.ledger().sequence() + LOCK_DURATION;
        env.storage().persistent().set(&DataKey::LockUntil(user.clone()), &lock_until);

        let checkpoint = env.ledger().sequence() + 1;
        env.storage().persistent().set(&DataKey::Checkpoint(user.clone()), &checkpoint);
    }

    pub fn withdraw(env: Env, user: Address) -> i128 {
        user.require_auth();

        let lock_until: u32 = env.storage().persistent().get(&DataKey::LockUntil(user.clone())).unwrap_or(0);
        if env.ledger().sequence() < lock_until {
            panic_with_error!(&env, VaultError::LockNotExpired);
        }

        let current_shares: i128 = env.storage().persistent().get(&DataKey::Shares(user.clone())).unwrap_or(0);
        let principal: i128 = env.storage().persistent().get(&DataKey::Balance(user.clone())).unwrap_or(0);

        if current_shares == 0 {
            return 0;
        }

        let total_shares: i128 = env.storage().instance().get(&DataKey::TotalShares).unwrap_or(0);
        env.storage().instance().set(&DataKey::TotalShares, &(total_shares - current_shares));

        env.storage().persistent().remove(&DataKey::Balance(user.clone()));
        env.storage().persistent().remove(&DataKey::Shares(user.clone()));
        env.storage().persistent().remove(&DataKey::LockUntil(user.clone()));
        env.storage().persistent().remove(&DataKey::Checkpoint(user.clone()));

        principal
    }

    pub fn early_exit(env: Env, user: Address) -> i128 {
        user.require_auth();

        let current_shares: i128 = env.storage().persistent().get(&DataKey::Shares(user.clone())).unwrap_or(0);
        let principal: i128 = env.storage().persistent().get(&DataKey::Balance(user.clone())).unwrap_or(0);

        if current_shares == 0 {
            return 0;
        }

        let exit_fee_fp = 125_000;
        let fee = mul_fp(principal, exit_fee_fp);
        let net_amount = principal - fee;

        let total_shares: i128 = env.storage().instance().get(&DataKey::TotalShares).unwrap_or(0);
        env.storage().instance().set(&DataKey::TotalShares, &(total_shares - current_shares));

        env.storage().persistent().remove(&DataKey::Balance(user.clone()));
        env.storage().persistent().remove(&DataKey::Shares(user.clone()));
        env.storage().persistent().remove(&DataKey::LockUntil(user.clone()));
        env.storage().persistent().remove(&DataKey::Checkpoint(user.clone()));

        net_amount
    }

    /// Renew the lock on a matured position without touching Balance or Shares.
    ///
    /// Callable only when `current_ledger >= lock_until` (position has matured).
    /// Resets `lock_until = current_ledger + LOCK_DURATION` in place.
    /// No USDC transfer occurs — this is a pure lock-extension.
    ///
    /// Returns the new `lock_until` ledger sequence number.
    pub fn relock(env: Env, user: Address) -> u32 {
        user.require_auth();

        let lock_until: u32 = env
            .storage()
            .persistent()
            .get(&DataKey::LockUntil(user.clone()))
            .unwrap_or(0);

        if env.ledger().sequence() < lock_until {
            panic_with_error!(&env, VaultError::NotYetMatured);
        }

        let new_lock_until = env.ledger().sequence() + LOCK_DURATION;
        env.storage()
            .persistent()
            .set(&DataKey::LockUntil(user.clone()), &new_lock_until);

        new_lock_until
    }

    pub fn lock_until(env: Env, user: Address) -> u32 {
        env.storage().persistent().get(&DataKey::LockUntil(user)).unwrap_or(0)
    }

    pub fn balance(env: Env, user: Address) -> i128 {
        env.storage().persistent().get(&DataKey::Balance(user)).unwrap_or(0)
    }

    pub fn shares(env: Env, user: Address) -> i128 {
        env.storage().persistent().get(&DataKey::Shares(user)).unwrap_or(0)
    }

    pub fn total_shares(env: Env) -> i128 {
        env.storage().instance().get(&DataKey::TotalShares).unwrap_or(0)
    }
}

mod test;