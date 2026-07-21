#![no_std]
use soroban_sdk::{contract, contracterror, contractimpl, contracttype, panic_with_error, token, Address, Env};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum VaultError {
    BelowMinDeposit      = 2,
    LockNotExpired       = 3,
    NotYetMatured        = 4,
    DepositCapExceeded   = 5,
    Unauthorized         = 6,
    AmountExceedsBalance = 7,
}

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    LockUntil(Address),
    Balance(Address),
    Shares(Address),
    Checkpoint(Address),
    TotalShares,
    TotalBalance,
    Admin,
    Governance,
    Strategy,
    Usdc,
    MaxTvl,
}

const FP_MULTIPLIER: i128 = 1_000_000_0;

pub fn mul_fp(a: i128, b_fp: i128) -> i128 {
    (a * b_fp) / FP_MULTIPLIER
}

// 12-month lock duration in ledgers (~5 s/ledger)
const LOCK_DURATION: u32 = 3_110_400;
const DEFAULT_MAX_TVL: i128 = 1_000_000_0_000_000;

#[contract]
pub struct VaultL12;

#[contractimpl]
impl VaultL12 {
    pub fn initialize(
        env: Env,
        admin: Address,
        governance: Address,
        strategy: Address,
        usdc: Address,
        max_tvl: i128,
    ) {
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Governance, &governance);
        env.storage().instance().set(&DataKey::Strategy, &strategy);
        env.storage().instance().set(&DataKey::Usdc, &usdc);
        env.storage().instance().set(&DataKey::TotalShares, &0i128);
        env.storage().instance().set(&DataKey::TotalBalance, &0i128);
        let cap = if max_tvl > 0 { max_tvl } else { DEFAULT_MAX_TVL };
        env.storage().instance().set(&DataKey::MaxTvl, &cap);
    }

    pub fn deposit(env: Env, user: Address, amount: i128) {
        user.require_auth();

        if amount < 2_500_000_000 {
            panic_with_error!(&env, VaultError::BelowMinDeposit);
        }

        let total_balance: i128 = env.storage().instance().get(&DataKey::TotalBalance).unwrap_or(0);
        let max_tvl: i128 = env.storage().instance().get(&DataKey::MaxTvl).unwrap_or(DEFAULT_MAX_TVL);
        if total_balance + amount > max_tvl {
            panic_with_error!(&env, VaultError::DepositCapExceeded);
        }

        let multiplier_fp = 13_000_000;
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
        env.storage().instance().set(&DataKey::TotalBalance, &(total_balance + amount));

        let lock_until = env.ledger().sequence() + LOCK_DURATION;
        env.storage().persistent().set(&DataKey::LockUntil(user.clone()), &lock_until);
        let checkpoint = env.ledger().sequence() + 1;
        env.storage().persistent().set(&DataKey::Checkpoint(user.clone()), &checkpoint);
    }

    /// Withdraw `amount` from a matured position.
    pub fn withdraw(env: Env, user: Address, amount: i128) -> i128 {
        user.require_auth();

        let lock_until: u32 = env.storage().persistent().get(&DataKey::LockUntil(user.clone())).unwrap_or(0);
        if env.ledger().sequence() < lock_until {
            panic_with_error!(&env, VaultError::LockNotExpired);
        }

        let balance: i128 = env.storage().persistent().get(&DataKey::Balance(user.clone())).unwrap_or(0);
        let user_shares: i128 = env.storage().persistent().get(&DataKey::Shares(user.clone())).unwrap_or(0);
        let total_shares: i128 = env.storage().instance().get(&DataKey::TotalShares).unwrap_or(0);

        if amount > balance {
            panic_with_error!(&env, VaultError::AmountExceedsBalance);
        }

        if amount >= balance {
            let total_balance: i128 = env.storage().instance().get(&DataKey::TotalBalance).unwrap_or(0);
            env.storage().instance().set(&DataKey::TotalShares, &(total_shares - user_shares));
            env.storage().instance().set(&DataKey::TotalBalance, &(total_balance - balance).max(0));
            env.storage().persistent().remove(&DataKey::Balance(user.clone()));
            env.storage().persistent().remove(&DataKey::Shares(user.clone()));
            env.storage().persistent().remove(&DataKey::LockUntil(user.clone()));
            env.storage().persistent().remove(&DataKey::Checkpoint(user.clone()));
            return balance;
        }

        let shares_to_burn = (user_shares * amount) / balance;
        let total_balance: i128 = env.storage().instance().get(&DataKey::TotalBalance).unwrap_or(0);
        env.storage().persistent().set(&DataKey::Balance(user.clone()), &(balance - amount));
        env.storage().persistent().set(&DataKey::Shares(user.clone()), &(user_shares - shares_to_burn));
        env.storage().instance().set(&DataKey::TotalShares, &(total_shares - shares_to_burn));
        env.storage().instance().set(&DataKey::TotalBalance, &(total_balance - amount));

        amount
    }

    /// Early exit `amount` before maturity.
    pub fn early_exit(env: Env, user: Address, amount: i128) -> i128 {
        user.require_auth();

        let balance: i128 = env.storage().persistent().get(&DataKey::Balance(user.clone())).unwrap_or(0);
        let user_shares: i128 = env.storage().persistent().get(&DataKey::Shares(user.clone())).unwrap_or(0);
        let total_shares: i128 = env.storage().instance().get(&DataKey::TotalShares).unwrap_or(0);

        if amount > balance {
            panic_with_error!(&env, VaultError::AmountExceedsBalance);
        }

        // Exit fee: 2.50% on withdrawn amount only
        let exit_fee_fp = 250_000;
        let fee = mul_fp(amount, exit_fee_fp);
        let net_amount = amount - fee;

        if amount >= balance {
            let total_balance: i128 = env.storage().instance().get(&DataKey::TotalBalance).unwrap_or(0);
            env.storage().instance().set(&DataKey::TotalShares, &(total_shares - user_shares));
            env.storage().instance().set(&DataKey::TotalBalance, &(total_balance - balance).max(0));
            env.storage().persistent().remove(&DataKey::Balance(user.clone()));
            env.storage().persistent().remove(&DataKey::Shares(user.clone()));
            env.storage().persistent().remove(&DataKey::LockUntil(user.clone()));
            env.storage().persistent().remove(&DataKey::Checkpoint(user.clone()));
            return net_amount;
        }

        let shares_to_burn = (user_shares * amount) / balance;
        let total_balance: i128 = env.storage().instance().get(&DataKey::TotalBalance).unwrap_or(0);
        env.storage().persistent().set(&DataKey::Balance(user.clone()), &(balance - amount));
        env.storage().persistent().set(&DataKey::Shares(user.clone()), &(user_shares - shares_to_burn));
        env.storage().instance().set(&DataKey::TotalShares, &(total_shares - shares_to_burn));
        env.storage().instance().set(&DataKey::TotalBalance, &(total_balance - amount));

        net_amount
    }

    pub fn set_max_tvl(env: Env, new_cap: i128) {
        let governance: Address = env.storage().instance().get(&DataKey::Governance).unwrap();
        governance.require_auth();
        env.storage().instance().set(&DataKey::MaxTvl, &new_cap);
    }

    pub fn max_tvl(env: Env) -> i128 {
        env.storage().instance().get(&DataKey::MaxTvl).unwrap_or(DEFAULT_MAX_TVL)
    }

    pub fn remaining_capacity(env: Env) -> i128 {
        let max_tvl: i128 = env.storage().instance().get(&DataKey::MaxTvl).unwrap_or(DEFAULT_MAX_TVL);
        let total_balance: i128 = env.storage().instance().get(&DataKey::TotalBalance).unwrap_or(0);
        (max_tvl - total_balance).max(0)
    }

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

    pub fn total_balance(env: Env) -> i128 {
        env.storage().instance().get(&DataKey::TotalBalance).unwrap_or(0)
    }
}

mod test;
