#![cfg(test)]
extern crate std;

use super::*;
use soroban_sdk::{testutils::{Address as _, Ledger}, Address, Env};

fn setup_with_cap(cap: i128) -> (Env, VaultL3Client<'static>, Address, Address, Address, Address, Address) {
    let env = Env::default();
    let contract_id = env.register_contract(None, VaultL3);
    let client = VaultL3Client::new(&env, &contract_id);

    let admin      = Address::generate(&env);
    let governance = Address::generate(&env);
    let guardian   = Address::generate(&env);
    let strategy   = Address::generate(&env);
    let usdc       = Address::generate(&env);

    env.mock_all_auths();
    client.initialize(&admin, &governance, &guardian, &strategy, &usdc, &cap);

    (env, client, admin, governance, guardian, strategy, usdc)
}

fn setup() -> (Env, VaultL3Client<'static>, Address, Address, Address, Address, Address) {
    setup_with_cap(100_000_000_000)
}

#[test]
fn test_deposit_and_shares() {
    let (env, client, _admin, _gov, _guardian, _strategy, _usdc) = setup();
    let user = Address::generate(&env);
    env.mock_all_auths();

    let amount = 500_000_000i128;
    client.deposit(&user, &amount);

    assert_eq!(client.shares(&user), 525_000_000);
    assert_eq!(client.balance(&user), amount);
    assert_eq!(client.total_balance(), amount);
    assert_eq!(client.total_shares(), 525_000_000);

    let expected_lock = env.ledger().sequence() + 777_600;
    assert_eq!(client.lock_until(&user), expected_lock);
}

#[test]
#[should_panic(expected = "BelowMinDeposit")]
fn test_deposit_below_min() {
    let (env, client, _admin, _gov, _guardian, _strategy, _usdc) = setup();
    let user = Address::generate(&env);
    env.mock_all_auths();
    client.deposit(&user, &499_999_999i128);
}

#[test]
#[should_panic(expected = "DepositCapExceeded")]
fn test_deposit_above_cap_rejected() {
    let (env, client, _admin, _gov, _guardian, _strategy, _usdc) = setup_with_cap(10_000_000_000);
    let user = Address::generate(&env);
    env.mock_all_auths();

    client.deposit(&user, &10_000_000_000_i128);

    let user2 = Address::generate(&env);
    client.deposit(&user2, &500_000_000_i128);
}

#[test]
fn test_deposit_exactly_at_cap_succeeds() {
    let cap: i128 = 500_000_000;
    let (env, client, _admin, _gov, _guardian, _strategy, _usdc) = setup_with_cap(cap);
    let user = Address::generate(&env);
    env.mock_all_auths();
    client.deposit(&user, &cap);
    assert_eq!(client.total_balance(), cap);
    assert_eq!(client.remaining_capacity(), 0);
}

#[test]
fn test_remaining_capacity_decreases_on_deposit() {
    let cap: i128 = 2_000_000_000;
    let (env, client, _admin, _gov, _guardian, _strategy, _usdc) = setup_with_cap(cap);
    let user = Address::generate(&env);
    env.mock_all_auths();

    client.deposit(&user, &500_000_000_i128);
    assert_eq!(client.remaining_capacity(), cap - 500_000_000);
}

#[test]
fn test_set_max_tvl_by_governance() {
    let (env, client, _admin, _gov, _guardian, _strategy, _usdc) = setup();
    env.mock_all_auths();
    let new_cap: i128 = 500_000_000_000;
    client.set_max_tvl(&new_cap);
    assert_eq!(client.max_tvl(), new_cap);
}

#[test]
#[should_panic]
fn test_set_max_tvl_by_non_governance_rejected() {
    let (_env, client, _admin, _gov, _guardian, _strategy, _usdc) = setup();
    let new_cap: i128 = 500_000_000_000;
    client.set_max_tvl(&new_cap);
}

#[test]
fn test_lower_cap_does_not_affect_existing_depositors() {
    let (env, client, _admin, _gov, _guardian, _strategy, _usdc) = setup();
    let user = Address::generate(&env);
    env.mock_all_auths();

    client.deposit(&user, &500_000_000_i128);
    client.set_max_tvl(&100_000_000_i128);

    assert_eq!(client.balance(&user), 500_000_000);
    assert_eq!(client.remaining_capacity(), 0);
}

#[test]
#[should_panic(expected = "LockNotExpired")]
fn test_withdraw_early_fails() {
    let (env, client, _admin, _gov, _guardian, _strategy, _usdc) = setup();
    let user = Address::generate(&env);
    env.mock_all_auths();

    client.deposit(&user, &500_000_000i128);
    env.ledger().with_mut(|l| l.sequence_number += 1);
    client.withdraw(&user, &500_000_000_i128);
}

#[test]
fn test_withdraw_at_maturity() {
    let (env, client, _admin, _gov, _guardian, _strategy, _usdc) = setup();
    let user = Address::generate(&env);
    env.mock_all_auths();

    client.deposit(&user, &500_000_000i128);
    let seq = env.ledger().sequence();
    env.ledger().set_sequence(seq + 777_600);

    let returned = client.withdraw(&user, &500_000_000_i128);
    assert_eq!(returned, 500_000_000);
    assert_eq!(client.total_balance(), 0);
    assert_eq!(client.shares(&user), 0);
    assert_eq!(client.balance(&user), 0);
}

#[test]
fn test_partial_withdraw_reduces_position() {
    let (env, client, _admin, _gov, _guardian, _strategy, _usdc) = setup();
    let user = Address::generate(&env);
    env.mock_all_auths();

    client.deposit(&user, &1_000_000_000_i128);
    env.ledger().set_sequence(env.ledger().sequence() + 777_600);

    let returned = client.withdraw(&user, &500_000_000_i128);
    assert_eq!(returned, 500_000_000);
    assert_eq!(client.balance(&user), 500_000_000);
    assert_eq!(client.total_balance(), 500_000_000);
    assert!(client.lock_until(&user) > 0);
}

#[test]
fn test_full_amount_partial_withdraw_equals_full_withdraw() {
    let (env, client, _admin, _gov, _guardian, _strategy, _usdc) = setup();
    let user = Address::generate(&env);
    env.mock_all_auths();

    client.deposit(&user, &500_000_000_i128);
    env.ledger().set_sequence(env.ledger().sequence() + 777_600);

    let returned = client.withdraw(&user, &500_000_000_i128);
    assert_eq!(returned, 500_000_000);
    assert_eq!(client.balance(&user), 0);
    assert_eq!(client.shares(&user), 0);
    assert_eq!(client.lock_until(&user), 0);
}

#[test]
#[should_panic(expected = "AmountExceedsBalance")]
fn test_over_withdrawal_rejected() {
    let (env, client, _admin, _gov, _guardian, _strategy, _usdc) = setup();
    let user = Address::generate(&env);
    env.mock_all_auths();

    client.deposit(&user, &500_000_000_i128);
    env.ledger().set_sequence(env.ledger().sequence() + 777_600);
    client.withdraw(&user, &500_000_001_i128);
}

#[test]
fn test_early_exit_charges_fee_normally() {
    let (env, client, _admin, _gov, _guardian, _strategy, _usdc) = setup();
    let user = Address::generate(&env);
    env.mock_all_auths();

    let amount = 1_000_000_000i128;
    client.deposit(&user, &amount);

    let returned = client.early_exit(&user, &amount);
    let expected_fee = (amount * 50_000) / 10_000_000;
    assert_eq!(returned, amount - expected_fee);
}

#[test]
fn test_partial_early_exit_fee_on_withdrawn_amount_only() {
    let (env, client, _admin, _gov, _guardian, _strategy, _usdc) = setup();
    let user = Address::generate(&env);
    env.mock_all_auths();

    client.deposit(&user, &1_000_000_000_i128);
    let net = client.early_exit(&user, &500_000_000_i128);
    assert_eq!(net, 497_500_000);
    assert_eq!(client.balance(&user), 500_000_000);
}

#[test]
#[should_panic(expected = "AmountExceedsBalance")]
fn test_over_early_exit_rejected() {
    let (env, client, _admin, _gov, _guardian, _strategy, _usdc) = setup();
    let user = Address::generate(&env);
    env.mock_all_auths();

    client.deposit(&user, &500_000_000_i128);
    client.early_exit(&user, &500_000_001_i128);
}

// ── Emergency Unlock tests ────────────────────────────────────────────────────

#[test]
fn test_emergency_unlock_defaults_false() {
    let (_env, client, _admin, _gov, _guardian, _strategy, _usdc) = setup();
    assert!(!client.emergency_unlock());
}

#[test]
fn test_guardian_can_activate_emergency_unlock() {
    let (_env, client, _admin, _gov, _guardian, _strategy, _usdc) = setup();
    client.set_emergency_unlock(&true);
    assert!(client.emergency_unlock());
}

#[test]
fn test_guardian_can_deactivate_emergency_unlock() {
    let (_env, client, _admin, _gov, _guardian, _strategy, _usdc) = setup();
    client.set_emergency_unlock(&true);
    client.set_emergency_unlock(&false);
    assert!(!client.emergency_unlock());
}

#[test]
fn test_early_exit_during_emergency_unlock_no_fee() {
    let (env, client, _admin, _gov, _guardian, _strategy, _usdc) = setup();
    let user = Address::generate(&env);
    env.mock_all_auths();

    let amount = 1_000_000_000i128;
    client.deposit(&user, &amount);

    client.set_emergency_unlock(&true);

    let returned = client.early_exit(&user, &amount);
    assert_eq!(returned, amount);
}

// ── relock tests ────────────────────────────────────────────────────────────

#[test]
fn test_relock_at_maturity_sets_new_lock_until() {
    let (env, client, _admin, _gov, _guardian, _strategy, _usdc) = setup();
    let user = Address::generate(&env);
    env.mock_all_auths();

    let amount = 500_000_000i128;
    client.deposit(&user, &amount);

    let deposit_seq = env.ledger().sequence();
    env.ledger().with_mut(|l| l.sequence_number = deposit_seq + 777_600);

    let new_lock = client.relock(&user);
    let expected = env.ledger().sequence() + 777_600;
    assert_eq!(new_lock, expected);
    assert_eq!(client.lock_until(&user), expected);
}

#[test]
fn test_relock_does_not_change_balance_or_shares() {
    let (env, client, _admin, _gov, _guardian, _strategy, _usdc) = setup();
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
    let (env, client, _admin, _gov, _guardian, _strategy, _usdc) = setup();
    let user = Address::generate(&env);
    env.mock_all_auths();

    let amount = 500_000_000i128;
    client.deposit(&user, &amount);

    let deposit_seq = env.ledger().sequence();
    env.ledger().with_mut(|l| l.sequence_number = deposit_seq + 777_599);

    client.relock(&user);
}

// ── Emergency withdraw / deactivation tests ──────────────────────────────────

#[test]
fn test_withdraw_during_emergency_unlock_skips_lock_check() {
    let (env, client, _admin, _gov, _guardian, _strategy, _usdc) = setup();
    let user = Address::generate(&env);
    env.mock_all_auths();

    let amount = 500_000_000i128;
    client.deposit(&user, &amount);

    env.ledger().with_mut(|l| l.sequence_number += 100);
    client.set_emergency_unlock(&true);

    let returned = client.withdraw(&user, &amount);
    assert_eq!(returned, amount);
}

#[test]
fn test_fee_and_lock_enforced_after_emergency_deactivation() {
    let (env, client, _admin, _gov, _guardian, _strategy, _usdc) = setup();
    let user = Address::generate(&env);
    env.mock_all_auths();

    let amount = 1_000_000_000i128;
    client.deposit(&user, &amount);

    client.set_emergency_unlock(&true);
    client.set_emergency_unlock(&false);

    let returned = client.early_exit(&user, &amount);
    let expected_fee = (amount * 50_000) / 10_000_000;
    assert_eq!(returned, amount - expected_fee);
}

#[test]
fn test_emergency_unlock_independent_of_balance_accounting() {
    let (env, client, _admin, _gov, _guardian, _strategy, _usdc) = setup();
    let user = Address::generate(&env);
    env.mock_all_auths();

    let amount = 500_000_000i128;
    client.deposit(&user, &amount);

    let shares_before = client.shares(&user);
    let total_before  = client.total_shares();

    client.set_emergency_unlock(&true);
    client.early_exit(&user, &amount);

    assert_eq!(client.shares(&user), 0);
    assert_eq!(client.total_shares(), total_before - shares_before);
}

#[test]
#[should_panic(expected = "LockNotExpired")]
fn test_withdraw_before_lock_fails() {
    let (env, client, _admin, _gov, _guardian, _strategy, _usdc) = setup();
    let user = Address::generate(&env);
    env.mock_all_auths();

    client.deposit(&user, &500_000_000_i128);
    client.withdraw(&user, &100_000_000_i128);
}
