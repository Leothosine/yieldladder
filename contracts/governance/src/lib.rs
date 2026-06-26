#![no_std]
use soroban_sdk::{
    contract, contractclient, contractimpl, contracttype, Address, Env,
};

const TIMELOCK_LEDGERS: u32 = 51_840; // 72 hours at ~5 s/ledger

#[contracttype]
#[derive(Clone)]
pub struct AllocationAction {
    pub pool_id: Address,
    pub target_bps: i128,
}

#[contracttype]
#[derive(Clone, PartialEq)]
pub enum ProposalStatus {
    Pending,
    Executed,
    Vetoed,
}

#[contracttype]
#[derive(Clone)]
pub struct Proposal {
    pub id: u32,
    pub action: AllocationAction,
    pub proposed_at_ledger: u32,
    pub status: ProposalStatus,
}

#[contracttype]
pub enum DataKey {
    Strategist,
    Guardian,
    StrategyVault,
    Proposal(u32),
    NextId,
}

#[contractclient(name = "StrategyVaultClient")]
pub trait StrategyVaultInterface {
    fn set_allocation(env: Env, pool_id: Address, target_bps: i128);
}

#[contract]
pub struct Governance;

#[contractimpl]
impl Governance {
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

    pub fn propose(env: Env, pool_id: Address, target_bps: i128) -> u32 {
        let strategist: Address = env
            .storage()
            .instance()
            .get(&DataKey::Strategist)
            .expect("not initialized");
        strategist.require_auth();

        if target_bps < 0 || target_bps > 10_000 {
            panic!("target_bps must be in [0, 10000]");
        }

        let id: u32 = env.storage().instance().get(&DataKey::NextId).unwrap();
        let proposal = Proposal {
            id,
            action: AllocationAction { pool_id, target_bps },
            proposed_at_ledger: env.ledger().sequence(),
            status: ProposalStatus::Pending,
        };
        env.storage().persistent().set(&DataKey::Proposal(id), &proposal);
        env.storage().instance().set(&DataKey::NextId, &(id + 1));
        id
    }

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

        let vault_id: Address = env
            .storage()
            .instance()
            .get(&DataKey::StrategyVault)
            .expect("not initialized");
        let vault = StrategyVaultClient::new(&env, &vault_id);
        vault.set_allocation(&proposal.action.pool_id, &proposal.action.target_bps);

        proposal.status = ProposalStatus::Executed;
        env.storage()
            .persistent()
            .set(&DataKey::Proposal(proposal_id), &proposal);
    }

    pub fn proposal(env: Env, proposal_id: u32) -> Proposal {
        env.storage()
            .persistent()
            .get(&DataKey::Proposal(proposal_id))
            .expect("proposal not found")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{
        testutils::{Address as _, Ledger},
        Address, Env,
    };

    #[contract]
    struct MockVault;

    #[contractimpl]
    impl MockVault {
        pub fn set_allocation(_env: Env, _pool_id: Address, _target_bps: i128) {}
    }

    fn setup(env: &Env) -> (Address, Address, GovernanceClient) {
        let strategist = Address::generate(env);
        let guardian = Address::generate(env);
        let vault_id = env.register_contract(None, MockVault);
        let contract_id = env.register_contract(None, Governance);
        let client = GovernanceClient::new(env, &contract_id);
        env.mock_all_auths();
        client.initialize(&strategist, &guardian, &vault_id);
        (strategist, guardian, client)
    }

    #[test]
    fn contract_instantiates() {
        let env = Env::default();
        let _id = env.register_contract(None, Governance);
    }

    #[test]
    fn propose_returns_incrementing_ids() {
        let env = Env::default();
        let pool = Address::generate(&env);
        let (_, _, client) = setup(&env);
        let id0 = client.propose(&pool, &500i128);
        let id1 = client.propose(&pool, &600i128);
        assert_eq!(id0, 0);
        assert_eq!(id1, 1);
    }

    #[test]
    fn full_lifecycle_executes_after_timelock() {
        let env = Env::default();
        env.ledger().with_mut(|l| l.sequence_number = 1_000);
        let pool = Address::generate(&env);
        let (_, _, client) = setup(&env);
        let id = client.propose(&pool, &1_000i128);
        env.ledger()
            .with_mut(|l| l.sequence_number = 1_000 + TIMELOCK_LEDGERS);
        client.execute(&id);
        let p = client.proposal(&id);
        assert_eq!(p.status, ProposalStatus::Executed);
    }

    #[test]
    fn veto_cancels_proposal() {
        let env = Env::default();
        let pool = Address::generate(&env);
        let (_, _, client) = setup(&env);
        let id = client.propose(&pool, &500i128);
        client.veto(&id);
        let p = client.proposal(&id);
        assert_eq!(p.status, ProposalStatus::Vetoed);
    }

    #[test]
    #[should_panic(expected = "proposal is not pending")]
    fn execute_after_veto_panics() {
        let env = Env::default();
        env.ledger().with_mut(|l| l.sequence_number = 1_000);
        let pool = Address::generate(&env);
        let (_, _, client) = setup(&env);
        let id = client.propose(&pool, &500i128);
        client.veto(&id);
        env.ledger()
            .with_mut(|l| l.sequence_number = 1_000 + TIMELOCK_LEDGERS);
        client.execute(&id);
    }

    #[test]
    #[should_panic(expected = "timelock not elapsed")]
    fn execute_before_timelock_panics() {
        let env = Env::default();
        env.ledger().with_mut(|l| l.sequence_number = 1_000);
        let pool = Address::generate(&env);
        let (_, _, client) = setup(&env);
        let id = client.propose(&pool, &500i128);
        env.ledger()
            .with_mut(|l| l.sequence_number = 1_000 + TIMELOCK_LEDGERS - 1);
        client.execute(&id);
    }

    #[test]
    #[should_panic(expected = "proposal is not pending")]
    fn double_execute_panics() {
        let env = Env::default();
        env.ledger().with_mut(|l| l.sequence_number = 1_000);
        let pool = Address::generate(&env);
        let (_, _, client) = setup(&env);
        let id = client.propose(&pool, &500i128);
        env.ledger()
            .with_mut(|l| l.sequence_number = 1_000 + TIMELOCK_LEDGERS);
        client.execute(&id);
        client.execute(&id);
    }
}