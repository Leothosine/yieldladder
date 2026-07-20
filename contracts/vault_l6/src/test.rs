#![cfg(test)]
extern crate std;

use super::*;
use soroban_sdk::{testutils::{Address as _, Ledger}, Address, Env};

fn setup_with_cap(cap: i128) -> (Env, VaultL6Client<'static>, Address, Address, Address, Address) {
    let env = Env::default();
    let contract_id = env.register_contract(None, VaultL6);
    let client = VaultL6Client::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let governance = Address::generate(&env);
    let strategy = Address::generate(&env);
    let usdc = Address::generate(&env);
    env.mock_all_auths();
    client.initialize(&admin, &governance, &strategy, &usdc, &cap);
    (env, client, admin, governance, strategy, usdc)
}

fn setup() -> (Env, VaultL6Client<'static>, Address, Address, Address, Address) {
    setup_with_cap(100_000_000_000_000) // 10,000,000 USDC
}

#[test]
fn test_deposit_and_shares() {
    let (env, client, _admin, _gov, _strategy, _usdc) = setup();
    let user = Address::generate(&env);
    env.mock_all_auths();

    let amount = 1_000_000_000;
    client.deposit(&user, &amount);

    assert_eq!(client.shares(&user), 1_150_000_000);
    assert_eq!(client.balance(&user), amount);
    assert_eq!(client.total_balance(), amount);

    let expected_lock = env.ledger().sequence() + 1_555_200;
    assert_eq!(client.lock_until(&user), expected_lock);
}

#[test]
#[should_panic(expected = "BelowMinDeposit")]
fn test_deposit_below_min() {
    let (env, client, _admin, _gov, _strategy, _usdc) = setup();
    let user = Address::generate(&env);
    env.mock_all_auths();
    client.deposit(&user, &999_999_999_i128);
}

#[test]
#[should_panic(expected = "DepositCapExceeded")]
fn test_deposit_above_cap_rejected() {
    let cap: i128 = 1_000_000_000; // exactly 100 USDC
    let (env, client, _admin, _gov, _strategy, _usdc) = setup_with_cap(cap);
    let user = Address::generate(&env);
    env.mock_all_auths();
    client.deposit(&user, &cap);
    let user2 = Address::generate(&env);
    client.deposit(&user2, &1_000_000_000_i128);
}

#[test]
fn test_deposit_exactly_at_cap_succeeds() {
    let cap: i128 = 1_000_000_000;
    let (env, client, _admin, _gov, _strategy, _usdc) = setup_with_cap(cap);
    let user = Address::generate(&env);
    env.mock_all_auths();
    client.deposit(&user, &cap);
    assert_eq!(client.remaining_capacity(), 0);
}

#[test]
fn test_set_max_tvl_by_governance() {
    let (env, client, _admin, _gov, _strategy, _usdc) = setup();
    env.mock_all_auths();
    client.set_max_tvl(&500_000_000_000_i128);
    assert_eq!(client.max_tvl(), 500_000_000_000);
}

#[test]
#[should_panic]
fn test_set_max_tvl_non_governance_rejected() {
    let (env, client, _admin, _gov, _strategy, _usdc) = setup();
    client.set_max_tvl(&500_000_000_000_i128);
}

#[test]
fn test_lower_cap_does_not_evict_existing_depositor() {
    let (env, client, _admin, _gov, _strategy, _usdc) = setup();
    let user = Address::generate(&env);
    env.mock_all_auths();
    client.deposit(&user, &1_000_000_000_i128);
    client.set_max_tvl(&100_000_000_i128);
    assert_eq!(client.balance(&user), 1_000_000_000);
    assert_eq!(client.remaining_capacity(), 0);
}

#[test]
#[should_panic(expected = "LockNotExpired")]
fn test_withdraw_early_fails() {
    let (env, client, _admin, _gov, _strategy, _usdc) = setup();
    let user = Address::generate(&env);
    env.mock_all_auths();
    client.deposit(&user, &1_000_000_000_i128);
    client.withdraw(&user);
}

#[test]
fn test_withdraw_at_maturity() {
    let (env, client, _admin, _gov, _strategy, _usdc) = setup();
    let user = Address::generate(&env);
    env.mock_all_auths();

    let amount = 1_000_000_000;
    client.deposit(&user, &amount);

    let seq = env.ledger().sequence();
    env.ledger().set_sequence(seq + 1_555_200);

    let returned = client.withdraw(&user);
    assert_eq!(returned, amount);
    assert_eq!(client.shares(&user), 0);
    assert_eq!(client.balance(&user), 0);
}

// ── relock tests ────────────────────────────────────────────────────────────

#[test]
fn test_relock_at_maturity_sets_new_lock_until() {
    let (env, client, _admin, _gov, _strategy, _usdc) = setup();
    let user = Address::generate(&env);
    env.mock_all_auths();

    client.deposit(&user, &1_000_000_000_i128);

    let deposit_seq = env.ledger().sequence();
    env.ledger().set_sequence(deposit_seq + 1_555_200);

    let new_lock = client.relock(&user);
    let expected = env.ledger().sequence() + 1_555_200;
    assert_eq!(new_lock, expected);
    assert_eq!(client.lock_until(&user), expected);
}

#[test]
fn test_relock_does_not_change_balance_or_shares() {
    let (env, client, _admin, _gov, _strategy, _usdc) = setup();
    let user = Address::generate(&env);
    env.mock_all_auths();

    client.deposit(&user, &1_000_000_000_i128);

    let shares_before = client.shares(&user);
    let balance_before = client.balance(&user);
    let total_before = client.total_shares();

    let deposit_seq = env.ledger().sequence();
    env.ledger().set_sequence(deposit_seq + 1_555_200);

    client.relock(&user);

    assert_eq!(client.shares(&user), shares_before);
    assert_eq!(client.balance(&user), balance_before);
    assert_eq!(client.total_shares(), total_before);
}

#[test]
#[should_panic(expected = "NotYetMatured")]
fn test_relock_before_maturity_is_rejected() {
    let (env, client, _admin, _gov, _strategy, _usdc) = setup();
    let user = Address::generate(&env);
    env.mock_all_auths();

    client.deposit(&user, &1_000_000_000_i128);

    let deposit_seq = env.ledger().sequence();
    env.ledger().set_sequence(deposit_seq + 1_555_199);

    client.relock(&user);
}
