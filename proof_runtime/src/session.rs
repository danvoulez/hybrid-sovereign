use crate::receipt::StepReceipt;
use sovereign_core::{hash_canonical, CaseId, Cid, Hash, ProofPackCid, ReceiptCid};

#[derive(Debug, Clone)]
pub struct TranscriptEntry {
    pub receipt: StepReceipt,
    pub prev_hash: Option<Hash>,
    pub entry_hash: Hash,
}

#[derive(Debug, Clone)]
pub struct Session {
    pub case_id: CaseId,
    pub contract_hash: Hash,
    pub initial_budget: u64,
    pub budget_remaining: u64,
    pub state_root: Cid,
    pub transcript: Vec<TranscriptEntry>,
    pub final_receipt_cid: Option<ReceiptCid>,
    pub final_proof_pack_cid: Option<ProofPackCid>,
    pub last_worker_cid: Option<Cid>,
    pub last_task_cid: Option<Cid>,
    pub continuation_cids_used: Vec<Cid>,
    pub manager_receipt_cids: Vec<ReceiptCid>,
}

impl Session {
    pub fn append_receipt(&mut self, receipt: StepReceipt) {
        let prev_hash = self.transcript.last().map(|entry| entry.entry_hash.clone());
        let payload = receipt.canonical();
        let next_hash = match &prev_hash {
            Some(prev) => hash_canonical(&[prev.as_str(), payload.as_str()]),
            None => hash_canonical(&[payload.as_str()]),
        };
        self.transcript.push(TranscriptEntry {
            receipt,
            prev_hash,
            entry_hash: next_hash,
        });
    }

    pub fn transcript_head(&self) -> Hash {
        self.transcript
            .last()
            .map(|entry| entry.entry_hash.clone())
            .unwrap_or_else(|| {
                hash_canonical(&[self.case_id.as_str(), self.contract_hash.as_str()])
            })
    }
}

#[derive(Debug, Clone)]
pub struct SessionView {
    pub case_id: CaseId,
    pub contract_hash: Hash,
    pub budget_remaining: u64,
    pub state_root: Cid,
    pub transcript_len: usize,
    pub worker_completed_count: usize,
    pub worker_yielded_count: usize,
    pub final_receipt_cid: Option<ReceiptCid>,
}

impl From<&Session> for SessionView {
    fn from(value: &Session) -> Self {
        let worker_completed_count = value
            .transcript
            .iter()
            .filter(|entry| matches!(entry.receipt, StepReceipt::WorkerCompleted { .. }))
            .count();
        let worker_yielded_count = value
            .transcript
            .iter()
            .filter(|entry| matches!(entry.receipt, StepReceipt::WorkerYielded { .. }))
            .count();
        Self {
            case_id: value.case_id.clone(),
            contract_hash: value.contract_hash.clone(),
            budget_remaining: value.budget_remaining,
            state_root: value.state_root.clone(),
            transcript_len: value.transcript.len(),
            worker_completed_count,
            worker_yielded_count,
            final_receipt_cid: value.final_receipt_cid.clone(),
        }
    }
}
