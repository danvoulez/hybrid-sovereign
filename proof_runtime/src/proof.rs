use crate::action::FinalOutcome;
use crate::receipt::StepReceipt;
use sovereign_core::{CaseId, Cid, Hash, ProofPackCid, ReceiptCid};

#[derive(Debug, Clone)]
pub struct ProofPack {
    pub proof_pack_cid: ProofPackCid,
    pub case_id: CaseId,
    pub contract_hash: Hash,
    pub initial_budget: u64,
    pub event_count: u64,
    pub transcript_head: Hash,
    pub transcript_receipts: Vec<StepReceipt>,
    pub final_state_root: Cid,
    pub final_outcome: FinalOutcome,
    pub final_receipt_cid: Option<ReceiptCid>,
    pub worker_cid: Option<Cid>,
    pub task_cid: Option<Cid>,
    pub continuation_cids_used: Vec<Cid>,
    pub manager_receipt_cids: Vec<ReceiptCid>,
}
