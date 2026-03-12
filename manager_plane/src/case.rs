use crate::budget::BudgetState;
use sovereign_core::{CaseId, Cid, ProofPackCid};

#[derive(Debug, Clone)]
pub struct ManagedCase {
    pub case_id: CaseId,
    pub state_root: Cid,
    pub current_head_cid: Option<Cid>,
    pub active_budget: BudgetState,
    pub pending_events: Vec<Cid>,
    pub pending_actions: Vec<String>,
    pub latest_proof_pack_cid: Option<ProofPackCid>,
    pub blocked_on: Option<BlockReason>,
}

#[derive(Debug, Clone)]
pub enum BlockReason {
    WaitingForWorker,
    WaitingForEvidence,
    WaitingForHumanWitness,
    WaitingForBudget,
    WaitingForPolicy,
    WaitingForForkResolution,
}
