use sovereign_core::{canonical_join, Cid};

#[derive(Debug, Clone)]
pub enum ComputeAction {
    RunWorker { worker_cid: Cid, task_cid: Cid },
    Propose { proposer_id: String, input_set_cid: Cid },
    RunExpert { expert_id: String, input_set_cid: Cid },
    RecomputePath { derivation_cid: Cid },
}

#[derive(Debug, Clone)]
pub enum MaterializeAction {
    RehydrateAtom { cid: Cid },
    RetrieveEvidence { query_cid: Cid, top_k: u8 },
    LoadModule { module_id: String },
}

#[derive(Debug, Clone)]
pub enum WitnessAction {
    AskUserBit {
        question_id: String,
        left: String,
        right: String,
    },
    AskUserField { field_id: String },
    GetTime { oracle_id: String },
    FetchExternalAtom {
        locator: String,
        expected_cid: Option<Cid>,
    },
}

impl ComputeAction {
    pub fn canonical(&self) -> String {
        match self {
            Self::RunWorker {
                worker_cid,
                task_cid,
            } => canonical_join(&["run_worker", worker_cid.as_str(), task_cid.as_str()]),
            Self::Propose {
                proposer_id,
                input_set_cid,
            } => canonical_join(&["propose", proposer_id, input_set_cid.as_str()]),
            Self::RunExpert {
                expert_id,
                input_set_cid,
            } => canonical_join(&["run_expert", expert_id, input_set_cid.as_str()]),
            Self::RecomputePath { derivation_cid } => {
                canonical_join(&["recompute_path", derivation_cid.as_str()])
            }
        }
    }
}

impl MaterializeAction {
    fn canonical(&self) -> String {
        match self {
            Self::RehydrateAtom { cid } => canonical_join(&["rehydrate_atom", cid.as_str()]),
            Self::RetrieveEvidence { query_cid, top_k } => {
                canonical_join(&["retrieve_evidence", query_cid.as_str(), &top_k.to_string()])
            }
            Self::LoadModule { module_id } => canonical_join(&["load_module", module_id]),
        }
    }
}

impl WitnessAction {
    fn canonical(&self) -> String {
        match self {
            Self::AskUserBit {
                question_id,
                left,
                right,
            } => canonical_join(&["ask_user_bit", question_id, left, right]),
            Self::AskUserField { field_id } => canonical_join(&["ask_user_field", field_id]),
            Self::GetTime { oracle_id } => canonical_join(&["get_time", oracle_id]),
            Self::FetchExternalAtom {
                locator,
                expected_cid,
            } => canonical_join(&[
                "fetch_external_atom",
                locator,
                expected_cid.as_ref().map(|v| v.as_str()).unwrap_or("-"),
            ]),
        }
    }
}

#[derive(Debug, Clone)]
pub enum StepAction {
    Compute(ComputeAction),
    Materialize(MaterializeAction),
    Witness(WitnessAction),
}

impl StepAction {
    pub fn canonical(&self) -> String {
        match self {
            Self::Compute(action) => canonical_join(&["compute", &action.canonical()]),
            Self::Materialize(action) => canonical_join(&["materialize", &action.canonical()]),
            Self::Witness(action) => canonical_join(&["witness", &action.canonical()]),
        }
    }
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
    UnanchoredEvidence,
    ZeroGuessViolation,
    DeterminismViolation,
    ContractViolation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FinalOutcome {
    Commit { output_cid: Cid },
    Reject { reason: RejectReason },
}
