use sovereign_core::Cid;

#[derive(Debug, Clone)]
pub enum ComputeAction {
    RunWorker { worker_cid: Cid, task_cid: Cid },
}

#[derive(Debug, Clone)]
pub enum StepAction {
    Compute(ComputeAction),
    Materialize {
        cid: Cid,
    },
    Witness {
        witness_kind: String,
        prompt_cid: Cid,
    },
}

#[derive(Debug, Clone)]
pub enum StepDecision {
    Commit,
    Continue(StepAction),
    Reject(RejectReason),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RejectReason {
    OutOfBudget,
    InvalidTranscript,
    InvalidWitness,
    InternalExecutionFailure,
    MissingMinimumEvidence,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FinalOutcome {
    Commit,
    Reject { reason: RejectReason },
}
