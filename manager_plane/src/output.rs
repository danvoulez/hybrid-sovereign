use sovereign_core::{Cid, PointerAlias, ProofPackCid, ReasonCode};

#[derive(Debug, Clone)]
pub enum ManagerOutput {
    Delegate {
        worker_cid: Cid,
        task_cid: Cid,
    },
    RequestEvidence {
        cid: Cid,
    },
    LoadExpert {
        expert_id: String,
        input_set_cid: Cid,
    },
    Escalate {
        queue: String,
        reason_code: ReasonCode,
    },
    AskHumanWitness {
        witness_kind: String,
        prompt_cid: Cid,
    },
    AdvancePointer {
        alias: PointerAlias,
        head_cid: Cid,
        proof_pack_cid: ProofPackCid,
    },
    Reject {
        reason_code: ReasonCode,
    },
    NoOp,
}
