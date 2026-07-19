#![cfg(test)]
extern crate std;

use super::*;
use soroban_sdk::{testutils::{Address as _, Ledger}, Address, Env};

fn setup() -> (Env, VaultL3Client<'static>, Address, Address, Address, Address) {
    let env = Env::default();
    let contract_id = env.register_contract(None, VaultL3);
    let client = VaultL3Client::new(&env, &contract_id);

    let admin    = Address::generate(&env);
    let guardian = Address::generate(&env);
    let strategy = Address::generate(&env);
    let usdc     = Address::generate(&env);

    env.mock_all_auths();
    client.initialize(&admin, &guardian, &strategy, &usdc);

    (env, client, admin, guardian, strategy, usdc)
}

#[test]
fn test_deposit_and_shares() {
    let (env, client, _admin, _guardian, _strategy, _usdc) = setup();
    let user = Address::generate(&env);
    env.mock_all_auths();

    let amount = 500_000_000i128;
    client.deposit(&user, &amount);

    assert_eq!(client.shares(&user), 525_000_000);
    assert_eq!(client.balance(&user), amount);
    assert_eq!(client.total_shares(), 525_000_000);

    let expected_lock = env.ledger().sequence() + 777_600;
    assert_eq!(client.lock_until(&user), expected_lock);
}

#[test]
#[should_panic(expected = "BelowMinDeposit")]
fn test_deposit_below_min() {
    let (env, client, _admin, _guardian, _strategy, _usdc) = setup();
    let user = Address::generate(&env);
    env.mock_all_auths();
    client.deposit(&user, &499_999_999i128);
}

#[test]
#[should_panic(expected = "LockNotExpired")]
fn test_withdraw_early_fails() {
    let (env, client, _admin, _guardian, _strategy, _usdc) = setup();
    let user = Address::generate(&env);
    env.mock_all_auths();

    client.deposit(&user, &500_000_000i128);
    // Advance only 1 ledger — lock is 777_600 ledgers
    env.ledger().with_mut(|l| l.sequence_number += 1);
    client.withdraw(&user);
}

#[test]
fn test_early_exit_charges_fee_normally() {
    let (env, client, _admin, _guardian, _strategy, _usdc) = setup();
    let user = Address::generate(&env);
    env.mock_all_auths();

    let amount = 1_000_000_000i128; // 100 USDC
    client.deposit(&user, &amount);

    // Emergency unlock is off — 0.5% fee applies
    let returned = client.early_exit(&user);
    let expected_fee = (amount * 50_000) / 10_000_000;
    assert_eq!(returned, amount - expected_fee);
}

// ── Emergency Unlock tests ────────────────────────────────────────────────────

#[test]
fn test_emergency_unlock_defaults_false() {
    let (_env, client, _admin, _guardian, _strategy, _usdc) = setup();
    assert!(!client.emergency_unlock());
}

#[test]
fn test_guardian_can_activate_emergency_unlock() {
    let (_env, client, _admin, _guardian, _strategy, _usdc) = setup();
    client.set_emergency_unlock(&true);
    assert!(client.emergency_unlock());
}

#[test]
fn test_guardian_can_deactivate_emergency_unlock() {
    let (_env, client, _admin, _guardian, _strategy, _usdc) = setup();
    client.set_emergency_unlock(&true);
    client.set_emergency_unlock(&false);
    assert!(!client.emergency_unlock());
}

#[test]
fn test_early_exit_during_emergency_unlock_no_fee() {
    let (env, client, _admin, _guardian, _strategy, _usdc) = setup();
    let user = Address::generate(&env);
    env.mock_all_auths();

    let amount = 1_000_000_000i128;
    client.deposit(&user, &amount);

    // Activate emergency unlock BEFORE the lock expires
    client.set_emergency_unlock(&true);

    // Should return full principal — no fee, no lock check
    let returned = client.early_exit(&user);
    assert_eq!(returned, amount);
}

// ── relock tests ────────────────────────────────────────────────────────────

#[test]
fn test_relock_at_maturity_sets_new_lock_until() {
    let (env, client, _admin, _guardian, _strategy, _usdc) = setup();
    let user = Address::generate(&env);

    env.mock_all_auths();

    let amount = 500_000_000i128;
    client.deposit(&user, &amount);

    // Fast-forward exactly to maturity
    let deposit_seq = env.ledger().sequence();
    env.ledger().with_mut(|l| l.sequence_number = deposit_seq + 777_600);

    let new_lock = client.relock(&user);
    let expected = env.ledger().sequence() + 777_600;
    assert_eq!(new_lock, expected);
    assert_eq!(client.lock_until(&user), expected);
}

#[test]
fn test_relock_does_not_change_balance_or_shares() {
    let (env, client, _admin, _guardian, _strategy, _usdc) = setup();
    let user = Address::generate(&env);

    env.mock_all_auths();

    let amount = 500_000_000i128;
    client.deposit(&user, &amount);

    let shares_before = client.shares(&user);
    let balance_before = client.balance(&user);
    let total_before = client.total_shares();

    let deposit_seq = env.ledger().sequence();
    env.ledger().with_mut(|l| l.sequence_number = deposit_seq + 777_600);

    client.relock(&user);

    assert_eq!(client.shares(&user), shares_before);
    assert_eq!(client.balance(&user), balance_before);
    assert_eq!(client.total_shares(), total_before);
}

#[test]
#[should_panic(expected = "NotYetMatured")]
fn test_relock_before_maturity_is_rejected() {
    let (env, client, _admin, _guardian, _strategy, _usdc) = setup();
    let user = Address::generate(&env);

    env.mock_all_auths();

    let amount = 500_000_000i128;
    client.deposit(&user, &amount);

    // Try to relock while still locked (one ledger before maturity)
    let deposit_seq = env.ledger().sequence();
    env.ledger().with_mut(|l| l.sequence_number = deposit_seq + 777_599);

    client.relock(&user);
}

// ── Emergency withdraw / deactivation tests ──────────────────────────────────

#[test]
fn test_withdraw_during_emergency_unlock_skips_lock_check() {
    let (env, client, _admin, _guardian, _strategy, _usdc) = setup();
    let user = Address::generate(&env);
    env.mock_all_auths();

    let amount = 500_000_000i128;
    client.deposit(&user, &amount);

    // Lock has not expired yet
    env.ledger().with_mut(|l| l.sequence_number += 100);

    client.set_emergency_unlock(&true);

    // Should succeed without LockNotExpired panic
    let returned = client.withdraw(&user);
    assert_eq!(returned, amount);
}

#[test]
fn test_fee_and_lock_enforced_after_emergency_deactivation() {
    let (env, client, _admin, _guardian, _strategy, _usdc) = setup();
    let user = Address::generate(&env);
    env.mock_all_auths();

    let amount = 1_000_000_000i128;
    client.deposit(&user, &amount);

    // Activate then deactivate — normal rules should resume
    client.set_emergency_unlock(&true);
    client.set_emergency_unlock(&false);

    let returned = client.early_exit(&user);
    let expected_fee = (amount * 50_000) / 10_000_000;
    assert_eq!(returned, amount - expected_fee);
}

#[test]
fn test_emergency_unlock_independent_of_balance_accounting() {
    let (env, client, _admin, _guardian, _strategy, _usdc) = setup();
    let user = Address::generate(&env);
    env.mock_all_auths();

    let amount = 500_000_000i128;
    client.deposit(&user, &amount);

    let shares_before = client.shares(&user);
    let total_before  = client.total_shares();

    client.set_emergency_unlock(&true);
    client.early_exit(&user);

    // Shares burned correctly
    assert_eq!(client.shares(&user), 0);
    assert_eq!(client.total_shares(), total_before - shares_before);
}