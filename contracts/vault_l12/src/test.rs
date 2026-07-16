#![cfg(test)]
extern crate std;

use super::*;
use soroban_sdk::{testutils::{Address as _, Ledger}, Address, Env};

fn setup_with_cap(cap: i128) -> (Env, VaultL12Client<'static>, Address, Address) {
    let env = Env::default();
    let contract_id = env.register_contract(None, VaultL12);
    let client = VaultL12Client::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let governance = Address::generate(&env);
    let strategy = Address::generate(&env);
    let usdc = Address::generate(&env);
    env.mock_all_auths();
    client.initialize(&admin, &governance, &strategy, &usdc, &cap);
    (env, client, admin, governance)
}

fn setup() -> (Env, VaultL12Client<'static>, Address, Address) {
    setup_with_cap(100_000_000_000_000)
}

#[test]
fn test_deposit_and_shares() {
    let (env, client, _, _) = setup();
    let user = Address::generate(&env);
    env.mock_all_auths();
    client.deposit(&user, &2_500_000_000_i128);
    assert_eq!(client.shares(&user), 3_250_000_000);
    assert_eq!(client.total_balance(), 2_500_000_000);
}

#[test]
#[should_panic(expected = "BelowMinDeposit")]
fn test_deposit_below_min() {
    let (env, client, _, _) = setup();
    let user = Address::generate(&env);
    env.mock_all_auths();
    client.deposit(&user, &2_499_999_999_i128);
}

#[test]
#[should_panic(expected = "DepositCapExceeded")]
fn test_deposit_above_cap_rejected() {
    let cap: i128 = 2_500_000_000;
    let (env, client, _, _) = setup_with_cap(cap);
    let user = Address::generate(&env);
    env.mock_all_auths();
    client.deposit(&user, &cap);
    let user2 = Address::generate(&env);
    client.deposit(&user2, &2_500_000_000_i128);
}

#[test]
fn test_deposit_exactly_at_cap_succeeds() {
    let cap: i128 = 2_500_000_000;
    let (env, client, _, _) = setup_with_cap(cap);
    let user = Address::generate(&env);
    env.mock_all_auths();
    client.deposit(&user, &cap);
    assert_eq!(client.remaining_capacity(), 0);
}

#[test]
fn test_set_max_tvl_by_governance() {
    let (env, client, _, _) = setup();
    env.mock_all_auths();
    client.set_max_tvl(&5_000_000_000_i128);
    assert_eq!(client.max_tvl(), 5_000_000_000);
}

#[test]
#[should_panic]
fn test_set_max_tvl_non_governance_rejected() {
    let (env, client, _, _) = setup();
    client.set_max_tvl(&5_000_000_000_i128);
}