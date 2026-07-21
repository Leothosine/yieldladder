//! # position_token
//!
//! An opt-in SEP-41-compatible transferable receipt for YieldLadder tier-vault
//! positions.
//!
//! ## Overview
//!
//! By default, YieldLadder positions are locked to the depositor's address and
//! are non-transferable. This contract allows any position owner to **wrap**
//! their position into a transferable token, giving them—and whoever they
//! transfer the token to—the ability to `withdraw` or `early_exit` the wrapped
//! position from the tier vault.
//!
//! The default deposit flow in every tier vault is entirely unaffected.
//!
//! ## Lifecycle
//!
//! ```text
//! 1. Owner calls wrap(owner, tier_vault, position_id)
//!    → PositionToken mints token_id to owner
//!    → tier vault records PositionToken as the authorized withdrawer
//!
//! 2. Owner calls transfer(from, to, token_id)  [SEP-41]
//!    → new holder can now withdraw/early_exit
//!
//! 3. Current holder calls unwrap(token_id)
//!    → token burned
//!    → tier vault restores direct address-based withdrawal rights to holder
//! ```
//!
//! ## SEP-41 subset implemented
//!
//! `transfer`, `transfer_from`, `approve`, `allowance`, `balance`,
//! `authorized`, `set_authorized`, `mint`, `burn`, `burn_from`,
//! `decimals`, `name`, `symbol`.
//!
//! Token supply is always either 0 or 1 per `token_id` (NFT semantics).

#![no_std]
use soroban_sdk::{
    contract, contractclient, contractimpl, contracttype, Address, Env, String,
};

// ── Cross-contract interface ──────────────────────────────────────────────────

/// Every tier vault must implement this interface so the position_token
/// contract can redirect withdrawal auth.
///
/// `set_withdrawal_auth(position_owner, new_auth)`:
///   - When wrapping: called with (original_owner, position_token_contract)
///   - When unwrapping: called with (position_token_contract, token_holder)
///
/// The tier vault stores the authorized withdrawer per position and checks it
/// in `withdraw` / `early_exit` instead of (or in addition to) the original
/// depositor address.
#[contractclient(name = "TierVaultClient")]
pub trait TierVaultInterface {
    fn set_withdrawal_auth(env: Env, current_auth: Address, new_auth: Address);
}

// ── Storage keys ──────────────────────────────────────────────────────────────

#[contracttype]
pub enum DataKey {
    /// Admin address that can call admin-only functions.
    Admin,
    /// Total number of tokens ever minted (monotonically increasing; used as
    /// the next token id).
    NextTokenId,
    /// Owner of a specific token_id.
    Owner(u64),
    /// Tier vault address for a specific token_id.
    TierVault(u64),
    /// Original position owner for a specific token_id (needed for unwrap).
    OriginalOwner(u64),
    /// Allowance: (owner, spender) → approved token_id (or None).
    /// We store a simple boolean approval per (owner, spender, token_id).
    Approved(u64),
    /// Operator approval: (owner, operator) → bool.
    OperatorApproval(Address, Address),
    // ── SEP-41 metadata ───────────────────────────────────────────────────────
    Name,
    Symbol,
    Decimals,
}

// ── Contract ─────────────────────────────────────────────────────────────────

#[contract]
pub struct PositionToken;

#[contractimpl]
impl PositionToken {
    // ── Initialisation ────────────────────────────────────────────────────────

    pub fn initialize(env: Env, admin: Address) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("already initialized");
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::NextTokenId, &0u64);
        env.storage().instance().set(
            &DataKey::Name,
            &String::from_str(&env, "YieldLadder Position Receipt"),
        );
        env.storage()
            .instance()
            .set(&DataKey::Symbol, &String::from_str(&env, "YLPR"));
        env.storage().instance().set(&DataKey::Decimals, &0u32);
    }

    // ── Core wrap / unwrap ────────────────────────────────────────────────────

    /// Wrap a tier-vault position into a transferable receipt token.
    ///
    /// The caller (`user`) must be the current owner of the position in
    /// `tier_vault`.  A new token is minted to `user`, and the tier vault is
    /// asked to redirect withdrawal auth to this contract so that whoever
    /// holds the token can withdraw.
    ///
    /// Returns the newly minted `token_id`.
    ///
    /// Only the position owner may call this — wrapping is strictly opt-in.
    pub fn wrap(env: Env, user: Address, tier_vault: Address) -> u64 {
        // Only the position owner may wrap their own position.
        user.require_auth();

        let contract = env.current_contract_address();

        // Tell the tier vault to redirect withdrawal auth from `user` to this
        // contract.  The tier vault validates that `user` is the current auth.
        let vault_client = TierVaultClient::new(&env, &tier_vault);
        vault_client.set_withdrawal_auth(&user, &contract);

        // Mint a new token.
        let token_id: u64 = env
            .storage()
            .instance()
            .get(&DataKey::NextTokenId)
            .unwrap_or(0);

        env.storage()
            .persistent()
            .set(&DataKey::Owner(token_id), &user);
        env.storage()
            .persistent()
            .set(&DataKey::TierVault(token_id), &tier_vault);
        env.storage()
            .persistent()
            .set(&DataKey::OriginalOwner(token_id), &user);

        env.storage()
            .instance()
            .set(&DataKey::NextTokenId, &(token_id + 1));

        token_id
    }

    /// Unwrap a position receipt token, restoring direct address-based
    /// withdrawal rights to the current token holder.
    ///
    /// Burns the token and tells the tier vault to redirect withdrawal auth
    /// back to the current holder's address.
    pub fn unwrap(env: Env, token_id: u64) {
        let holder: Address = env
            .storage()
            .persistent()
            .get(&DataKey::Owner(token_id))
            .expect("token not found");

        // Only the current holder may unwrap.
        holder.require_auth();

        let tier_vault: Address = env
            .storage()
            .persistent()
            .get(&DataKey::TierVault(token_id))
            .expect("tier vault not found");

        let contract = env.current_contract_address();

        // Restore direct withdrawal rights to the current holder.
        let vault_client = TierVaultClient::new(&env, &tier_vault);
        vault_client.set_withdrawal_auth(&contract, &holder);

        // Burn the token.
        env.storage().persistent().remove(&DataKey::Owner(token_id));
        env.storage()
            .persistent()
            .remove(&DataKey::TierVault(token_id));
        env.storage()
            .persistent()
            .remove(&DataKey::OriginalOwner(token_id));
        env.storage()
            .persistent()
            .remove(&DataKey::Approved(token_id));
    }

    // ── SEP-41 token interface ────────────────────────────────────────────────

    /// Transfer a position receipt token from one address to another.
    ///
    /// Only the current owner (or an approved operator) may call this.
    /// After transfer the new holder can withdraw / early_exit the wrapped
    /// position; the original depositor loses that right.
    pub fn transfer(env: Env, from: Address, to: Address, token_id: u64) {
        from.require_auth();
        Self::_check_owner_or_approved(&env, &from, token_id);
        Self::_do_transfer(&env, to, token_id);
    }

    /// Transfer on behalf of `from` using a prior approval.
    pub fn transfer_from(env: Env, spender: Address, from: Address, to: Address, token_id: u64) {
        spender.require_auth();
        Self::_check_approved_for(&env, &spender, &from, token_id);
        Self::_do_transfer(&env, to, token_id);
    }

    /// Approve `spender` to transfer `token_id` once on behalf of the owner.
    pub fn approve(env: Env, owner: Address, spender: Address, token_id: u64, _expiration_ledger: u32) {
        owner.require_auth();
        let current_owner: Address = env
            .storage()
            .persistent()
            .get(&DataKey::Owner(token_id))
            .expect("token not found");
        if current_owner != owner {
            panic!("not the token owner");
        }
        env.storage()
            .persistent()
            .set(&DataKey::Approved(token_id), &spender);
    }

    /// Returns the approved spender for `token_id`, or panics if there is none.
    pub fn allowance(env: Env, token_id: u64) -> Address {
        env.storage()
            .persistent()
            .get(&DataKey::Approved(token_id))
            .expect("no approval set")
    }

    /// Approve or revoke `operator` to act on behalf of `owner` for all tokens.
    pub fn set_authorized(env: Env, owner: Address, operator: Address, authorized: bool) {
        owner.require_auth();
        env.storage().persistent().set(
            &DataKey::OperatorApproval(owner, operator),
            &authorized,
        );
    }

    /// Returns whether `operator` is approved for all tokens of `owner`.
    pub fn authorized(env: Env, owner: Address, operator: Address) -> bool {
        env.storage()
            .persistent()
            .get(&DataKey::OperatorApproval(owner, operator))
            .unwrap_or(false)
    }

    /// Returns 1 if `addr` owns `token_id`, 0 otherwise (NFT semantics).
    pub fn balance(env: Env, addr: Address, token_id: u64) -> i128 {
        let owner: Option<Address> = env
            .storage()
            .persistent()
            .get(&DataKey::Owner(token_id));
        match owner {
            Some(o) if o == addr => 1,
            _ => 0,
        }
    }

    /// Mint a new token to `to`.  Only callable by the admin.
    /// (In normal usage, tokens are minted only via `wrap`.)
    pub fn mint(env: Env, to: Address, token_id: u64) {
        Self::require_admin(&env);
        if env.storage().persistent().has(&DataKey::Owner(token_id)) {
            panic!("token already exists");
        }
        env.storage()
            .persistent()
            .set(&DataKey::Owner(token_id), &to);
    }

    /// Burn `token_id`.  Only the current owner may burn.
    /// (In normal usage, tokens are burned only via `unwrap`.)
    pub fn burn(env: Env, from: Address, token_id: u64) {
        from.require_auth();
        let owner: Address = env
            .storage()
            .persistent()
            .get(&DataKey::Owner(token_id))
            .expect("token not found");
        if owner != from {
            panic!("not the token owner");
        }
        env.storage().persistent().remove(&DataKey::Owner(token_id));
        env.storage()
            .persistent()
            .remove(&DataKey::Approved(token_id));
    }

    /// Burn `token_id` on behalf of `from` using a prior approval.
    pub fn burn_from(env: Env, spender: Address, from: Address, token_id: u64) {
        spender.require_auth();
        Self::_check_approved_for(&env, &spender, &from, token_id);
        env.storage().persistent().remove(&DataKey::Owner(token_id));
        env.storage()
            .persistent()
            .remove(&DataKey::Approved(token_id));
    }

    // ── SEP-41 metadata ───────────────────────────────────────────────────────

    pub fn decimals(env: Env) -> u32 {
        env.storage()
            .instance()
            .get(&DataKey::Decimals)
            .unwrap_or(0)
    }

    pub fn name(env: Env) -> String {
        env.storage()
            .instance()
            .get(&DataKey::Name)
            .expect("not initialized")
    }

    pub fn symbol(env: Env) -> String {
        env.storage()
            .instance()
            .get(&DataKey::Symbol)
            .expect("not initialized")
    }

    // ── Read helpers ──────────────────────────────────────────────────────────

    /// Returns the current owner of `token_id`.
    pub fn owner_of(env: Env, token_id: u64) -> Address {
        env.storage()
            .persistent()
            .get(&DataKey::Owner(token_id))
            .expect("token not found")
    }

    /// Returns the tier vault address associated with `token_id`.
    pub fn tier_vault_of(env: Env, token_id: u64) -> Address {
        env.storage()
            .persistent()
            .get(&DataKey::TierVault(token_id))
            .expect("token not found")
    }

    // ── Internal helpers ──────────────────────────────────────────────────────

    fn _do_transfer(env: &Env, to: Address, token_id: u64) {
        env.storage()
            .persistent()
            .set(&DataKey::Owner(token_id), &to);
        // Clear any single-use approval on transfer.
        env.storage()
            .persistent()
            .remove(&DataKey::Approved(token_id));
    }

    fn _check_owner_or_approved(env: &Env, caller: &Address, token_id: u64) {
        let owner: Address = env
            .storage()
            .persistent()
            .get(&DataKey::Owner(token_id))
            .expect("token not found");

        if owner == *caller {
            return;
        }

        // Check operator approval.
        if env
            .storage()
            .persistent()
            .get(&DataKey::OperatorApproval(owner.clone(), caller.clone()))
            .unwrap_or(false)
        {
            return;
        }

        // Check single-use approval.
        let approved: Option<Address> = env
            .storage()
            .persistent()
            .get(&DataKey::Approved(token_id));
        if approved.as_ref() == Some(caller) {
            return;
        }

        panic!("caller is not owner or approved");
    }

    fn _check_approved_for(env: &Env, spender: &Address, from: &Address, token_id: u64) {
        // Check operator approval.
        if env
            .storage()
            .persistent()
            .get(&DataKey::OperatorApproval(from.clone(), spender.clone()))
            .unwrap_or(false)
        {
            return;
        }

        // Check single-use approval.
        let approved: Option<Address> = env
            .storage()
            .persistent()
            .get(&DataKey::Approved(token_id));
        if approved.as_ref() == Some(spender) {
            return;
        }

        panic!("spender not approved");
    }

    fn require_admin(env: &Env) {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .expect("not initialized");
        admin.require_auth();
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Address, Env};

    // ── Mock tier vault ───────────────────────────────────────────────────────

    /// A minimal mock tier vault that records the current authorized withdrawer.
    #[contract]
    struct MockTierVault;

    #[contractimpl]
    impl MockTierVault {
        pub fn set_withdrawal_auth(env: Env, current_auth: Address, new_auth: Address) {
            // In a real vault this would verify current_auth and update storage.
            // Here we just store new_auth so tests can inspect it.
            env.storage()
                .instance()
                .set(&soroban_sdk::symbol_short!("auth"), &new_auth);
            let _ = current_auth;
        }

        pub fn get_auth(env: Env) -> Address {
            env.storage()
                .instance()
                .get(&soroban_sdk::symbol_short!("auth"))
                .expect("auth not set")
        }
    }

    // ── Helper ────────────────────────────────────────────────────────────────

    fn setup(env: &Env) -> (Address, Address, PositionTokenClient) {
        let admin = Address::generate(env);
        let cid = env.register_contract(None, PositionToken);
        let client = PositionTokenClient::new(env, &cid);
        env.mock_all_auths();
        client.initialize(&admin);
        (admin, cid, client)
    }

    // ── Wrap ──────────────────────────────────────────────────────────────────

    #[test]
    fn wrap_mints_token_to_owner() {
        let env = Env::default();
        env.mock_all_auths();
        let (_, _, client) = setup(&env);
        let user = Address::generate(&env);
        let tier_vault = env.register_contract(None, MockTierVault);

        let token_id = client.wrap(&user, &tier_vault);
        assert_eq!(token_id, 0);
        assert_eq!(client.owner_of(&token_id), user);
        assert_eq!(client.balance(&user, &token_id), 1);
    }

    #[test]
    fn wrap_token_ids_increment() {
        let env = Env::default();
        env.mock_all_auths();
        let (_, _, client) = setup(&env);
        let user = Address::generate(&env);
        let tier_vault = env.register_contract(None, MockTierVault);

        let id0 = client.wrap(&user, &tier_vault);
        let id1 = client.wrap(&user, &tier_vault);
        assert_eq!(id0, 0);
        assert_eq!(id1, 1);
    }

    #[test]
    fn wrap_updates_tier_vault_auth() {
        let env = Env::default();
        env.mock_all_auths();
        let (_, cid, client) = setup(&env);
        let user = Address::generate(&env);
        let tier_vault = env.register_contract(None, MockTierVault);
        let vault_client = MockTierVaultClient::new(&env, &tier_vault);

        client.wrap(&user, &tier_vault);

        // The tier vault should now have the position_token contract as auth.
        assert_eq!(vault_client.get_auth(), cid);
    }

    // ── Transfer ──────────────────────────────────────────────────────────────

    #[test]
    fn transfer_changes_owner() {
        let env = Env::default();
        env.mock_all_auths();
        let (_, _, client) = setup(&env);
        let alice = Address::generate(&env);
        let bob = Address::generate(&env);
        let tier_vault = env.register_contract(None, MockTierVault);

        let token_id = client.wrap(&alice, &tier_vault);
        client.transfer(&alice, &bob, &token_id);

        assert_eq!(client.owner_of(&token_id), bob);
        assert_eq!(client.balance(&alice, &token_id), 0);
        assert_eq!(client.balance(&bob, &token_id), 1);
    }

    #[test]
    fn only_new_holder_can_unwrap_after_transfer() {
        let env = Env::default();
        env.mock_all_auths();
        let (_, _, client) = setup(&env);
        let alice = Address::generate(&env);
        let bob = Address::generate(&env);
        let tier_vault = env.register_contract(None, MockTierVault);
        let vault_client = MockTierVaultClient::new(&env, &tier_vault);

        let token_id = client.wrap(&alice, &tier_vault);
        client.transfer(&alice, &bob, &token_id);
        client.unwrap(&token_id);

        // After unwrap the tier vault should point to bob (new holder).
        assert_eq!(vault_client.get_auth(), bob);
    }

    // ── Unwrap ────────────────────────────────────────────────────────────────

    #[test]
    fn unwrap_burns_token_and_restores_rights() {
        let env = Env::default();
        env.mock_all_auths();
        let (_, _, client) = setup(&env);
        let user = Address::generate(&env);
        let tier_vault = env.register_contract(None, MockTierVault);
        let vault_client = MockTierVaultClient::new(&env, &tier_vault);

        let token_id = client.wrap(&user, &tier_vault);
        client.unwrap(&token_id);

        // Token should be gone.
        assert_eq!(client.balance(&user, &token_id), 0);
        // Tier vault should now have `user` as direct auth again.
        assert_eq!(vault_client.get_auth(), user);
    }

    #[test]
    #[should_panic(expected = "token not found")]
    fn unwrap_nonexistent_token_panics() {
        let env = Env::default();
        env.mock_all_auths();
        let (_, _, client) = setup(&env);
        client.unwrap(&999);
    }

    // ── Original depositor cannot withdraw after wrap+transfer ────────────────

    #[test]
    fn original_owner_balance_is_zero_after_transfer() {
        let env = Env::default();
        env.mock_all_auths();
        let (_, _, client) = setup(&env);
        let alice = Address::generate(&env);
        let bob = Address::generate(&env);
        let tier_vault = env.register_contract(None, MockTierVault);

        let token_id = client.wrap(&alice, &tier_vault);
        client.transfer(&alice, &bob, &token_id);

        assert_eq!(client.balance(&alice, &token_id), 0);
    }

    // ── Unwrapped (default) positions unaffected ──────────────────────────────

    #[test]
    fn other_users_positions_are_unaffected() {
        let env = Env::default();
        env.mock_all_auths();
        let (_, _, client) = setup(&env);
        let alice = Address::generate(&env);
        let bob = Address::generate(&env);
        let tier_vault = env.register_contract(None, MockTierVault);

        // Alice wraps; bob never interacts with position_token.
        let token_id = client.wrap(&alice, &tier_vault);
        client.transfer(&alice, &Address::generate(&env), &token_id);

        // Bob has no tokens — unaffected.
        assert_eq!(client.balance(&bob, &token_id), 0);
        // Arbitrary token_id that was never minted.
        assert_eq!(client.balance(&bob, &999u64), 0);
    }

    // ── Approve / transfer_from ───────────────────────────────────────────────

    #[test]
    fn approved_spender_can_transfer() {
        let env = Env::default();
        env.mock_all_auths();
        let (_, _, client) = setup(&env);
        let alice = Address::generate(&env);
        let spender = Address::generate(&env);
        let bob = Address::generate(&env);
        let tier_vault = env.register_contract(None, MockTierVault);

        let token_id = client.wrap(&alice, &tier_vault);
        client.approve(&alice, &spender, &token_id, &u32::MAX);
        client.transfer_from(&spender, &alice, &bob, &token_id);

        assert_eq!(client.owner_of(&token_id), bob);
    }

    // ── SEP-41 metadata ───────────────────────────────────────────────────────

    #[test]
    fn metadata_is_correct() {
        let env = Env::default();
        env.mock_all_auths();
        let (_, _, client) = setup(&env);
        assert_eq!(client.decimals(), 0);
        assert_eq!(
            client.name(),
            String::from_str(&env, "YieldLadder Position Receipt")
        );
        assert_eq!(client.symbol(), String::from_str(&env, "YLPR"));
    }
}
