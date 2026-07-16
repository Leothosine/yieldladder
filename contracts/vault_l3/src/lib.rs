#![no_std]
use soroban_sdk::{contract, contracterror, contractimpl, contracttype, panic_with_error, token, Address, Env};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum VaultError {
    BelowMinDeposit    = 2,
    LockNotExpired     = 3,
    Unauthorized       = 4,
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
    Guardian,
    Strategy,
    Usdc,
    /// Emergency unlock flag — when true, early_exit and withdraw skip
    /// lock and fee enforcement so depositors can exit safely.
    EmergencyUnlock,
}

// GF-01 internal mock: fixed-point math
const FP_MULTIPLIER: i128 = 1_000_000_0;

pub fn mul_fp(a: i128, b_fp: i128) -> i128 {
    (a * b_fp) / FP_MULTIPLIER
}

#[contract]
pub struct VaultL3;

#[contractimpl]
impl VaultL3 {
    // 3 months = ~777,600 ledgers at 5s/ledger
    pub fn initialize(env: Env, admin: Address, guardian: Address, strategy: Address, usdc: Address) {
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Guardian, &guardian);
        env.storage().instance().set(&DataKey::Strategy, &strategy);
        env.storage().instance().set(&DataKey::Usdc, &usdc);
        env.storage().instance().set(&DataKey::TotalShares, &0i128);
        env.storage().instance().set(&DataKey::EmergencyUnlock, &false);
    }

    // ── Emergency Unlock ─────────────────────────────────────────────────────

    /// Toggle the emergency unlock mode. Only the Guardian may call this.
    /// When active: `early_exit` and `withdraw` skip lock checks and fees.
    /// This flag is independent of any pause mechanism (NF-07).
    pub fn set_emergency_unlock(env: Env, active: bool) {
        let guardian: Address = env
            .storage()
            .instance()
            .get(&DataKey::Guardian)
            .expect("not initialized");
        guardian.require_auth();
        env.storage().instance().set(&DataKey::EmergencyUnlock, &active);
    }

    /// Returns the current state of the emergency unlock flag.
    pub fn emergency_unlock(env: Env) -> bool {
        env.storage()
            .instance()
            .get(&DataKey::EmergencyUnlock)
            .unwrap_or(false)
    }

    // ── Core vault operations ─────────────────────────────────────────────────

    pub fn deposit(env: Env, user: Address, amount: i128) {
        user.require_auth();

        if amount < 500_000_000 {
            panic_with_error!(&env, VaultError::BelowMinDeposit);
        }

        let multiplier_fp = 10_500_000;
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

        let lock_until = env.ledger().sequence() + 777_600;
        env.storage().persistent().set(&DataKey::LockUntil(user.clone()), &lock_until);

        let checkpoint = env.ledger().sequence() + 1;
        env.storage().persistent().set(&DataKey::Checkpoint(user.clone()), &checkpoint);
    }

    pub fn withdraw(env: Env, user: Address) -> i128 {
        user.require_auth();

        let emergency: bool = env
            .storage()
            .instance()
            .get(&DataKey::EmergencyUnlock)
            .unwrap_or(false);

        if !emergency {
            let lock_until: u32 = env.storage().persistent().get(&DataKey::LockUntil(user.clone())).unwrap_or(0);
            if env.ledger().sequence() < lock_until {
                panic_with_error!(&env, VaultError::LockNotExpired);
            }
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

        let emergency: bool = env
            .storage()
            .instance()
            .get(&DataKey::EmergencyUnlock)
            .unwrap_or(false);

        // During emergency unlock: no fee, no lock check — return full principal
        let net_amount = if emergency {
            principal
        } else {
            // Exit fee: 0.50% = 50_000 in FP_MULTIPLIER
            let fee = mul_fp(principal, 50_000);
            principal - fee
        };

        let total_shares: i128 = env.storage().instance().get(&DataKey::TotalShares).unwrap_or(0);
        env.storage().instance().set(&DataKey::TotalShares, &(total_shares - current_shares));

        env.storage().persistent().remove(&DataKey::Balance(user.clone()));
        env.storage().persistent().remove(&DataKey::Shares(user.clone()));
        env.storage().persistent().remove(&DataKey::LockUntil(user.clone()));
        env.storage().persistent().remove(&DataKey::Checkpoint(user.clone()));

        net_amount
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