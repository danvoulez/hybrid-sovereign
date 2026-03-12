use sovereign_core::{CaseId, Cid, PointerAlias, ReasonCode};

#[derive(Debug, Clone)]
pub enum ManagerReceipt {
    Delegated {
        case_id: CaseId,
        worker_cid: Cid,
        task_cid: Cid,
    },
    EvidenceRequested {
        case_id: CaseId,
        cid: Cid,
    },
    Escalated {
        case_id: CaseId,
        queue: String,
        reason_code: ReasonCode,
    },
    HumanWitnessRequested {
        case_id: CaseId,
        witness_kind: String,
        prompt_cid: Cid,
    },
    PointerAdvanced {
        case_id: CaseId,
        alias: PointerAlias,
        head_cid: Cid,
    },
    ManagerRejected {
        case_id: CaseId,
        reason_code: ReasonCode,
    },
}
