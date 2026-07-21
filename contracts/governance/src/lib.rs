#![no_std]
use soroban_sdk::{
    contract, contractclient, contractimpl, contracttype, Address, Env,
};

const TIMELOCK_LEDGERS: u32 = 51_840; // 72 hours at ~5 s/ledger

// ── Tier parameter tag ────────────────────────────────────────────────────────

/// Identifies which tier-vault parameter an `UpdateTierParam` proposal changes.
#[contracttype]
#[derive(Clone, PartialEq)]
pub enum TierParam {
    MinDeposit,
    Multiplier,
    ExitFeeBps,
    MaxTvl,
}

// ── Generalised proposal action ───────────────────────────────────────────────

/// Replaces the old `AllocationAction` struct with a proper enum so that
/// governance can vote on more than just pool-allocation changes.
#[contracttype]
#[derive(Clone)]
pub enum ProposalAction {
    /// Adjust the allocation weight (bps) of a pool inside StrategyVault.
    /// Backward-compatible replacement for the old `AllocationAction`.
    SetAllocation(Address, i128),
    /// Rotate the Strategist address stored in this Governance contract.
    UpdateStrategist(Address),
    /// Rotate the Guardian address stored in this Governance contract.
    UpdateGuardian(Address),
    /// Update a single parameter on a tier vault.
    /// The tier vault must expose a governance-gated `set_tier_param` setter
    /// (tracked as a dependency; the proposal can be queued before the setter
    /// is deployed and executed once it is live).
    UpdateTierParam(Address, TierParam, i128),
}

// ── Status ────────────────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, PartialEq)]
pub enum ProposalStatus {
    Pending,
    Executed,
    Vetoed,
}

// ── Proposal record ───────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone)]
pub struct Proposal {
    pub id: u32,
    pub action: ProposalAction,
    pub proposed_at_ledger: u32,
    pub status: ProposalStatus,
}

// ── Storage keys ──────────────────────────────────────────────────────────────

#[contracttype]
pub enum DataKey {
    Strategist,
    Guardian,
    StrategyVault,
    Proposal(u32),
    NextId,
}

// ── Cross-contract client for StrategyVault ───────────────────────────────────

#[contractclient(name = "StrategyVaultClient")]
pub trait StrategyVaultInterface {
    fn set_allocation(env: Env, pool_id: Address, target_bps: i128);
}

// ── Cross-contract client for tier vaults ────────────────────────────────────

/// Tier vaults must expose this setter (dependency noted in issue NF-09).
#[contractclient(name = "TierVaultGovClient")]
pub trait TierVaultGovInterface {
    fn set_tier_param(env: Env, param: TierParam, value: i128);
}

// ── Contract ─────────────────────────────────────────────────────────────────

#[contract]
pub struct Governance;

#[contractimpl]
impl Governance {
    /// One-time initialisation.
    ///
    /// `strategy_vault` **must** be the address of the deployed StrategyVault
    /// contract. Passing an EOA here would allow the timelock/veto flow to be
    /// bypassed — callers should assert this before calling `initialize`.
    pub fn initialize(
        env: Env,
        strategist: Address,
        guardian: Address,
        strategy_vault: Address,
    ) {
        if env.storage().instance().has(&DataKey::Strategist) {
            panic!("already initialized");
        }
        env.storage().instance().set(&DataKey::Strategist, &strategist);
        env.storage().instance().set(&DataKey::Guardian, &guardian);
        env.storage().instance().set(&DataKey::StrategyVault, &strategy_vault);
        env.storage().instance().set(&DataKey::NextId, &0u32);
    }

    // ── Proposal submission ───────────────────────────────────────────────────

    /// Submit a new governance proposal. Only the Strategist may call this.
    /// Returns the new proposal id.
    pub fn propose(env: Env, action: ProposalAction) -> u32 {
        let strategist: Address = env
            .storage()
            .instance()
            .get(&DataKey::Strategist)
            .expect("not initialized");
        strategist.require_auth();

        // Validate bps range for SetAllocation proposals up-front.
        if let ProposalAction::SetAllocation(_, bps) = &action {
            if *bps < 0 || *bps > 10_000 {
                panic!("target_bps must be in [0, 10000]");
            }
        }

        let id: u32 = env.storage().instance().get(&DataKey::NextId).unwrap();
        let proposal = Proposal {
            id,
            action,
            proposed_at_ledger: env.ledger().sequence(),
            status: ProposalStatus::Pending,
        };
        env.storage().persistent().set(&DataKey::Proposal(id), &proposal);
        env.storage().instance().set(&DataKey::NextId, &(id + 1));
        id
    }

    // ── Veto ─────────────────────────────────────────────────────────────────

    /// Cancel a pending proposal. Only the Guardian may call this.
    pub fn veto(env: Env, proposal_id: u32) {
        let guardian: Address = env
            .storage()
            .instance()
            .get(&DataKey::Guardian)
            .expect("not initialized");
        guardian.require_auth();

        let mut proposal: Proposal = env
            .storage()
            .persistent()
            .get(&DataKey::Proposal(proposal_id))
            .expect("proposal not found");

        if proposal.status != ProposalStatus::Pending {
            panic!("proposal is not pending");
        }

        proposal.status = ProposalStatus::Vetoed;
        env.storage()
            .persistent()
            .set(&DataKey::Proposal(proposal_id), &proposal);
    }

    // ── Execute ───────────────────────────────────────────────────────────────

    /// Execute a proposal after the timelock has elapsed.
    /// Anyone may call `execute` — the timelock itself is the guard.
    pub fn execute(env: Env, proposal_id: u32) {
        let mut proposal: Proposal = env
            .storage()
            .persistent()
            .get(&DataKey::Proposal(proposal_id))
            .expect("proposal not found");

        if proposal.status != ProposalStatus::Pending {
            panic!("proposal is not pending");
        }

        let elapsed = env
            .ledger()
            .sequence()
            .saturating_sub(proposal.proposed_at_ledger);
        if elapsed < TIMELOCK_LEDGERS {
            panic!("timelock not elapsed");
        }

        match proposal.action.clone() {
            ProposalAction::SetAllocation(pool_id, target_bps) => {
                let vault_id: Address = env
                    .storage()
                    .instance()
                    .get(&DataKey::StrategyVault)
                    .expect("not initialized");
                let vault = StrategyVaultClient::new(&env, &vault_id);
                vault.set_allocation(&pool_id, &target_bps);
            }
            ProposalAction::UpdateStrategist(new_strategist) => {
                env.storage()
                    .instance()
                    .set(&DataKey::Strategist, &new_strategist);
            }
            ProposalAction::UpdateGuardian(new_guardian) => {
                env.storage()
                    .instance()
                    .set(&DataKey::Guardian, &new_guardian);
            }
            ProposalAction::UpdateTierParam(tier_vault, param, value) => {
                let client = TierVaultGovClient::new(&env, &tier_vault);
                client.set_tier_param(&param, &value);
            }
        }

        proposal.status = ProposalStatus::Executed;
        env.storage()
            .persistent()
            .set(&DataKey::Proposal(proposal_id), &proposal);
    }

    // ── Read helpers ──────────────────────────────────────────────────────────

    pub fn proposal(env: Env, proposal_id: u32) -> Proposal {
        env.storage()
            .persistent()
            .get(&DataKey::Proposal(proposal_id))
            .expect("proposal not found")
    }

    pub fn strategist(env: Env) -> Address {
        env.storage()
            .instance()
            .get(&DataKey::Strategist)
            .expect("not initialized")
    }

    pub fn guardian(env: Env) -> Address {
        env.storage()
            .instance()
            .get(&DataKey::Guardian)
            .expect("not initialized")
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{
        testutils::{Address as _, Ledger},
        Address, Env,
    };

    // ── Mock contracts ────────────────────────────────────────────────────────

    #[contract]
    struct MockVault;

    #[contractimpl]
    impl MockVault {
        pub fn set_allocation(_env: Env, _pool_id: Address, _target_bps: i128) {}
    }

    #[contract]
    struct MockTierVault;

    #[contractimpl]
    impl MockTierVault {
        pub fn set_tier_param(_env: Env, _param: TierParam, _value: i128) {}
    }

    // ── Helpers ───────────────────────────────────────────────────────────────

    fn setup(env: &Env) -> (Address, Address, Address, GovernanceClient) {
        let strategist = Address::generate(env);
        let guardian = Address::generate(env);
        let vault_id = env.register_contract(None, MockVault);
        let contract_id = env.register_contract(None, Governance);
        let client = GovernanceClient::new(env, &contract_id);
        env.mock_all_auths();
        client.initialize(&strategist, &guardian, &vault_id);
        (strategist, guardian, vault_id, client)
    }

    // ── Basic instantiation ───────────────────────────────────────────────────

    #[test]
    fn contract_instantiates() {
        let env = Env::default();
        let _id = env.register_contract(None, Governance);
    }

    // ── SetAllocation (backward-compatible) ───────────────────────────────────

    #[test]
    fn propose_set_allocation_returns_incrementing_ids() {
        let env = Env::default();
        let pool = Address::generate(&env);
        let (_, _, _, client) = setup(&env);
        let id0 = client.propose(&ProposalAction::SetAllocation(pool.clone(), 500));
        let id1 = client.propose(&ProposalAction::SetAllocation(pool.clone(), 600));
        assert_eq!(id0, 0);
        assert_eq!(id1, 1);
    }

    #[test]
    fn full_lifecycle_set_allocation_executes_after_timelock() {
        let env = Env::default();
        env.ledger().with_mut(|l| l.sequence_number = 1_000);
        let pool = Address::generate(&env);
        let (_, _, _, client) = setup(&env);
        let id = client.propose(&ProposalAction::SetAllocation(pool.clone(), 1_000));
        env.ledger()
            .with_mut(|l| l.sequence_number = 1_000 + TIMELOCK_LEDGERS);
        client.execute(&id);
        let p = client.proposal(&id);
        assert_eq!(p.status, ProposalStatus::Executed);
    }

    // ── UpdateStrategist ──────────────────────────────────────────────────────

    #[test]
    fn update_strategist_rotates_stored_address() {
        let env = Env::default();
        env.ledger().with_mut(|l| l.sequence_number = 1_000);
        let new_strategist = Address::generate(&env);
        let (_, _, _, client) = setup(&env);
        let id = client.propose(&ProposalAction::UpdateStrategist(new_strategist.clone()));
        env.ledger()
            .with_mut(|l| l.sequence_number = 1_000 + TIMELOCK_LEDGERS);
        client.execute(&id);
        assert_eq!(client.strategist(), new_strategist);
        let p = client.proposal(&id);
        assert_eq!(p.status, ProposalStatus::Executed);
    }

    // ── UpdateGuardian ────────────────────────────────────────────────────────

    #[test]
    fn update_guardian_rotates_stored_address() {
        let env = Env::default();
        env.ledger().with_mut(|l| l.sequence_number = 1_000);
        let new_guardian = Address::generate(&env);
        let (_, _, _, client) = setup(&env);
        let id = client.propose(&ProposalAction::UpdateGuardian(new_guardian.clone()));
        env.ledger()
            .with_mut(|l| l.sequence_number = 1_000 + TIMELOCK_LEDGERS);
        client.execute(&id);
        assert_eq!(client.guardian(), new_guardian);
        let p = client.proposal(&id);
        assert_eq!(p.status, ProposalStatus::Executed);
    }

    // ── UpdateTierParam ───────────────────────────────────────────────────────

    #[test]
    fn update_tier_param_calls_tier_vault() {
        let env = Env::default();
        env.ledger().with_mut(|l| l.sequence_number = 1_000);
        let (_, _, _, client) = setup(&env);
        let tier_vault = env.register_contract(None, MockTierVault);
        let id = client.propose(&ProposalAction::UpdateTierParam(
            tier_vault.clone(),
            TierParam::MaxTvl,
            5_000_000_0_000_000i128,
        ));
        env.ledger()
            .with_mut(|l| l.sequence_number = 1_000 + TIMELOCK_LEDGERS);
        // Should not panic — MockTierVault accepts the call.
        client.execute(&id);
        let p = client.proposal(&id);
        assert_eq!(p.status, ProposalStatus::Executed);
    }

    // ── Veto blocks all proposal types equally ────────────────────────────────

    #[test]
    fn veto_cancels_set_allocation_proposal() {
        let env = Env::default();
        let pool = Address::generate(&env);
        let (_, _, _, client) = setup(&env);
        let id = client.propose(&ProposalAction::SetAllocation(pool, 500));
        client.veto(&id);
        assert_eq!(client.proposal(&id).status, ProposalStatus::Vetoed);
    }

    #[test]
    fn veto_cancels_update_strategist_proposal() {
        let env = Env::default();
        let (_, _, _, client) = setup(&env);
        let id = client.propose(&ProposalAction::UpdateStrategist(Address::generate(&env)));
        client.veto(&id);
        assert_eq!(client.proposal(&id).status, ProposalStatus::Vetoed);
    }

    #[test]
    fn veto_cancels_update_guardian_proposal() {
        let env = Env::default();
        let (_, _, _, client) = setup(&env);
        let id = client.propose(&ProposalAction::UpdateGuardian(Address::generate(&env)));
        client.veto(&id);
        assert_eq!(client.proposal(&id).status, ProposalStatus::Vetoed);
    }

    #[test]
    fn veto_cancels_update_tier_param_proposal() {
        let env = Env::default();
        let (_, _, _, client) = setup(&env);
        let tier_vault = env.register_contract(None, MockTierVault);
        let id = client.propose(&ProposalAction::UpdateTierParam(
            tier_vault,
            TierParam::ExitFeeBps,
            100,
        ));
        client.veto(&id);
        assert_eq!(client.proposal(&id).status, ProposalStatus::Vetoed);
    }

    // ── Execute-after-veto must panic ─────────────────────────────────────────

    #[test]
    #[should_panic(expected = "proposal is not pending")]
    fn execute_after_veto_panics() {
        let env = Env::default();
        env.ledger().with_mut(|l| l.sequence_number = 1_000);
        let pool = Address::generate(&env);
        let (_, _, _, client) = setup(&env);
        let id = client.propose(&ProposalAction::SetAllocation(pool, 500));
        client.veto(&id);
        env.ledger()
            .with_mut(|l| l.sequence_number = 1_000 + TIMELOCK_LEDGERS);
        client.execute(&id);
    }

    // ── Timelock enforcement ──────────────────────────────────────────────────

    #[test]
    #[should_panic(expected = "timelock not elapsed")]
    fn execute_before_timelock_panics() {
        let env = Env::default();
        env.ledger().with_mut(|l| l.sequence_number = 1_000);
        let pool = Address::generate(&env);
        let (_, _, _, client) = setup(&env);
        let id = client.propose(&ProposalAction::SetAllocation(pool, 500));
        env.ledger()
            .with_mut(|l| l.sequence_number = 1_000 + TIMELOCK_LEDGERS - 1);
        client.execute(&id);
    }

    // ── Double-execute must panic ─────────────────────────────────────────────

    #[test]
    #[should_panic(expected = "proposal is not pending")]
    fn double_execute_panics() {
        let env = Env::default();
        env.ledger().with_mut(|l| l.sequence_number = 1_000);
        let pool = Address::generate(&env);
        let (_, _, _, client) = setup(&env);
        let id = client.propose(&ProposalAction::SetAllocation(pool, 1_000));
        env.ledger()
            .with_mut(|l| l.sequence_number = 1_000 + TIMELOCK_LEDGERS);
        client.execute(&id);
        client.execute(&id);
    }
}
