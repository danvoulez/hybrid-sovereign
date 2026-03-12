use std::collections::HashMap;

use epistemic_storage::EpistemicHeat;
use manager_plane::{BudgetState, DemoManagerPlane, ManagedCase};
use proof_federation::{AcceptanceReceipt, PointerAnnouncement, PointerFork};
use proof_runtime::ProofPack;
use sovereign_core::{BudgetAmount, CaseId, Cid};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum CaseStatus {
    New,
    Running,
    BlockedOnEvidence,
    BlockedOnWitness,
    BlockedOnFederation,
    Committed,
    Rejected,
    Federated,
    Forked,
}

#[derive(Debug, Clone)]
pub enum WitnessKind {
    BinaryConfirm {
        question: String,
        positive_label: String,
        negative_label: String,
    },
    FieldFill {
        field_name: String,
        prompt: String,
    },
    ApproveReject {
        reason: String,
    },
}

#[derive(Debug, Clone)]
pub struct WitnessTask {
    pub witness_id: Cid,
    pub case_id: CaseId,
    pub prompt_cid: Cid,
    pub kind: WitnessKind,
}

impl WitnessTask {
    pub fn kind_label(&self) -> &'static str {
        match self.kind {
            WitnessKind::BinaryConfirm { .. } => "binary_confirm",
            WitnessKind::FieldFill { .. } => "field_fill",
            WitnessKind::ApproveReject { .. } => "approve_reject",
        }
    }
}

#[derive(Debug, Clone)]
pub struct PendingCompletion {
    pub receipt_cid: sovereign_core::ReceiptCid,
    pub proof_pack_cid: sovereign_core::ProofPackCid,
    pub required_witness_ids: Vec<Cid>,
    pub resolved_witness_ids: Vec<Cid>,
}

#[derive(Debug, Clone)]
pub struct WorkerLedgerEntry {
    pub stage: String,
    pub action: String,
    pub worker_cid: Cid,
    pub task_cid: Cid,
    pub insight: String,
    pub artifact_cids: Vec<Cid>,
}

#[derive(Debug, Clone)]
pub struct CaseSummary {
    pub case_id: CaseId,
    pub status: CaseStatus,
    pub budget_remaining: BudgetAmount,
    pub head: Option<Cid>,
    pub blocked_reason: Option<String>,
    pub last_receipt: Option<sovereign_core::ReceiptCid>,
}

#[derive(Debug, Default)]
pub struct AppState {
    pub manager: DemoManagerPlane,
    pub statuses: HashMap<String, CaseStatus>,
    pub proofs: HashMap<String, ProofPack>,
    pub hot_atoms_by_case: HashMap<String, Vec<Cid>>,
    pub case_atom_heat: HashMap<String, HashMap<Cid, EpistemicHeat>>,
    pub case_timeline: HashMap<String, Vec<String>>,
    pub worker_ledger_by_case: HashMap<String, Vec<WorkerLedgerEntry>>,
    pub pending_completions: HashMap<String, PendingCompletion>,
    pub witness_inbox: Vec<WitnessTask>,
    pub federation_log: Vec<String>,
    pub pointer_announcements: Vec<PointerAnnouncement>,
    pub acceptance_receipts: Vec<AcceptanceReceipt>,
    pub fork_registry: Vec<PointerFork>,
    pub replay_log: Vec<String>,
}

impl AppState {
    pub fn new_seeded() -> Self {
        let mut state = Self::default();
        let case_id = "case-doc-001".to_string();
        state.manager.cases.insert(
            case_id.clone(),
            ManagedCase {
                case_id: CaseId::from(case_id.as_str()),
                state_root: Cid::from("cid:state:case-doc-001"),
                current_head_cid: None,
                active_budget: BudgetState {
                    gas_remaining: BudgetAmount(80),
                    max_parallel_workers: 2,
                    max_open_cases: 16,
                    max_human_interrupts: 2,
                },
                pending_events: vec![],
                pending_actions: vec![],
                latest_proof_pack_cid: None,
                blocked_on: None,
            },
        );
        state.statuses.insert(case_id, CaseStatus::New);
        state
    }

    pub fn case_summary(&self, case_id: &str) -> Option<CaseSummary> {
        let case = self.manager.cases.get(case_id)?;
        let status = self
            .statuses
            .get(case_id)
            .copied()
            .unwrap_or(CaseStatus::New);
        let last_receipt = self
            .proofs
            .get(case_id)
            .and_then(|p| p.final_receipt_cid.clone());

        Some(CaseSummary {
            case_id: case.case_id.clone(),
            status,
            budget_remaining: case.active_budget.gas_remaining,
            head: case.current_head_cid.clone(),
            blocked_reason: case.blocked_on.as_ref().map(|r| format!("{r:?}")),
            last_receipt,
        })
    }

    pub fn queue(&self) -> Vec<CaseSummary> {
        self.manager
            .cases
            .keys()
            .filter_map(|id| self.case_summary(id))
            .collect()
    }
}
