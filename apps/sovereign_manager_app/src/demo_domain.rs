use epistemic_storage::{
    AtomBody, AtomHeader, AtomKind, AtomSpace, EpistemicHeat, InMemoryAtomSpace, UniversalAtom,
};
use proof_runtime::{
    run, ComputeAction, Contract, DeterminismProfile, ExecutionTarget, ProofMode, Session,
    SovereignRuntime, StepAction, StepDecision,
};
use sovereign_core::{hash_canonical, CaseId, Cid, Hash, ReceiptCid, Signature};
use worker_abi::{WorkerAbi, WorkerError, WorkerHostEnv, WorkerResult, WorkerYield};

pub const WORKER_INTAKE: &str = "worker:document-intake:v7";
pub const WORKER_EXTRACT: &str = "worker:document-extract:v7";
pub const WORKER_VALIDATE: &str = "worker:document-validate:v7";
pub const WORKER_DECISION_PACK: &str = "worker:decision-pack:v7";

pub const TASK_INTAKE: &str = "cid:task:doc-intake:001";
pub const TASK_EXTRACT: &str = "cid:task:doc-extract:001";
pub const TASK_VALIDATE: &str = "cid:task:doc-validate:001";
pub const TASK_DECISION_PACK: &str = "cid:task:decision-pack:001";

pub fn entry_worker_cid() -> Cid {
    Cid::from(WORKER_INTAKE)
}

pub fn entry_task_cid() -> Cid {
    Cid::from(TASK_INTAKE)
}

pub fn stage_name_from_worker(worker_cid: &Cid) -> &'static str {
    match worker_cid.as_str() {
        WORKER_INTAKE => "intake",
        WORKER_EXTRACT => "extract",
        WORKER_VALIDATE => "validate",
        WORKER_DECISION_PACK => "decision-pack",
        _ => "unknown-stage",
    }
}

#[derive(Debug, Clone)]
pub struct DomainExecution {
    pub proof: proof_runtime::ProofPack,
    pub needed_witness: bool,
    pub missing_field_name: String,
    pub missing_field_prompt: Cid,
    pub had_yield: bool,
    pub hot_atoms_after_run: Vec<Cid>,
    pub atom_heat_after_run: Vec<(Cid, EpistemicHeat)>,
}

#[derive(Debug)]
struct DocumentContract {
    stages: Vec<(Cid, Cid)>,
}

impl DocumentContract {
    fn new() -> Self {
        Self {
            stages: vec![
                (Cid::from(WORKER_INTAKE), Cid::from(TASK_INTAKE)),
                (Cid::from(WORKER_EXTRACT), Cid::from(TASK_EXTRACT)),
                (Cid::from(WORKER_VALIDATE), Cid::from(TASK_VALIDATE)),
                (
                    Cid::from(WORKER_DECISION_PACK),
                    Cid::from(TASK_DECISION_PACK),
                ),
            ],
        }
    }
}

impl Contract for DocumentContract {
    fn eval_step(&self, session: &proof_runtime::SessionView) -> StepDecision {
        let stage_index = session.worker_completed_count;
        if stage_index < self.stages.len() {
            let (worker_cid, task_cid) = &self.stages[stage_index];
            StepDecision::Continue(StepAction::Compute(ComputeAction::RunWorker {
                worker_cid: worker_cid.clone(),
                task_cid: task_cid.clone(),
            }))
        } else {
            StepDecision::Commit
        }
    }

    fn cost_of(&self, _action: &StepAction, _session: &proof_runtime::SessionView) -> u64 {
        3
    }

    fn determinism_profile(&self) -> DeterminismProfile {
        DeterminismProfile {
            fixed_point_only: false,
            allow_user_input: false,
            allow_time_oracle: false,
            allow_external_fetch: false,
            execution_target: ExecutionTarget::Wasm { abi_version: 1 },
        }
    }
}

#[derive(Debug)]
struct StageWorker {
    stage_name: &'static str,
    stage_worker_cid: Cid,
    stage_task_cid: Cid,
    required_atom_cid: Cid,
    allow_yield: bool,
    pending_continuation: Option<Cid>,
}

impl StageWorker {
    fn new(
        stage_name: &'static str,
        stage_worker_cid: Cid,
        stage_task_cid: Cid,
        required_atom_cid: Cid,
        allow_yield: bool,
    ) -> Self {
        Self {
            stage_name,
            stage_worker_cid,
            stage_task_cid,
            required_atom_cid,
            allow_yield,
            pending_continuation: None,
        }
    }

    fn continuation(&self) -> Cid {
        Cid::new(format!(
            "cid:continuation:{}:{}",
            self.stage_name,
            hash_canonical(&[
                self.stage_task_cid.as_str(),
                self.required_atom_cid.as_str(),
                self.stage_worker_cid.as_str(),
            ])
            .as_str()
        ))
    }

    fn build_receipt(&self, doc_bytes: &[u8]) -> ReceiptCid {
        ReceiptCid::new(format!(
            "cid:receipt:{}:{}",
            self.stage_name,
            hash_canonical(&[
                self.stage_task_cid.as_str(),
                &String::from_utf8_lossy(doc_bytes),
                self.stage_worker_cid.as_str(),
            ])
            .as_str()
        ))
    }
}

impl WorkerAbi for StageWorker {
    fn execute(&mut self, _task_cid: &Cid, env: &mut dyn WorkerHostEnv) -> WorkerResult {
        if env.consume_gas(2).is_err() {
            return WorkerResult::Fail(WorkerError::InternalFailure);
        }

        match env.request_atom(&self.required_atom_cid) {
            Ok(bytes) => WorkerResult::Complete(self.build_receipt(&bytes)),
            Err(_) if self.allow_yield => {
                let continuation = self.continuation();
                self.pending_continuation = Some(continuation.clone());
                WorkerResult::Yield(WorkerYield {
                    missing_cids: vec![self.required_atom_cid.clone()],
                    continuation_cid: continuation,
                })
            }
            Err(_) => WorkerResult::Fail(WorkerError::InternalFailure),
        }
    }

    fn resume(&mut self, continuation_cid: &Cid, env: &mut dyn WorkerHostEnv) -> WorkerResult {
        if env.consume_gas(1).is_err() {
            return WorkerResult::Fail(WorkerError::InternalFailure);
        }

        if self.pending_continuation.as_ref() != Some(continuation_cid) {
            return WorkerResult::Fail(WorkerError::InvalidTask);
        }

        match env.request_atom(&self.required_atom_cid) {
            Ok(bytes) => {
                self.pending_continuation = None;
                WorkerResult::Complete(self.build_receipt(&bytes))
            }
            Err(_) => WorkerResult::Fail(WorkerError::InternalFailure),
        }
    }
}

pub fn execute_document_case(case_id: &CaseId) -> Result<DomainExecution, String> {
    let required_atom_cid = Cid::from("cid:doc:intake:payload");
    let document_payload = b"doc_type=invoice;owner=".to_vec();

    let mut atom_space = InMemoryAtomSpace::default();
    let atom = UniversalAtom {
        header: AtomHeader {
            kind: AtomKind::Task,
            size_bytes: document_payload.len() as u64,
            producer_hash: Hash::from("producer:document-intake"),
            signature: Some(Signature::from("sig:document-intake")),
        },
        links: vec![],
        body: AtomBody::Inline(document_payload.clone()),
    };
    atom_space
        .materialize(required_atom_cid.clone(), atom)
        .map_err(|e| format!("materialize failed: {e}"))?;

    let mut session = Session {
        case_id: case_id.clone(),
        contract_hash: Hash::from("contract:document-intake:v7"),
        initial_budget: 64,
        budget_remaining: 64,
        state_root: Cid::from("cid:state:document-intake"),
        proof_mode: ProofMode::AnchoredImmutableRefs,
        transcript: vec![],
        final_receipt_cid: None,
        final_proof_pack_cid: None,
        last_worker_cid: None,
        last_task_cid: None,
        continuation_cids_used: vec![],
        manager_receipt_cids: vec![],
    };

    let contract = DocumentContract::new();

    let mut runtime = SovereignRuntime::new(&mut atom_space);
    runtime.register_worker(
        Cid::from(WORKER_INTAKE),
        Box::new(StageWorker::new(
            "intake",
            Cid::from(WORKER_INTAKE),
            Cid::from(TASK_INTAKE),
            required_atom_cid.clone(),
            true,
        )),
    );
    runtime.register_worker(
        Cid::from(WORKER_EXTRACT),
        Box::new(StageWorker::new(
            "extract",
            Cid::from(WORKER_EXTRACT),
            Cid::from(TASK_EXTRACT),
            required_atom_cid.clone(),
            false,
        )),
    );
    runtime.register_worker(
        Cid::from(WORKER_VALIDATE),
        Box::new(StageWorker::new(
            "validate",
            Cid::from(WORKER_VALIDATE),
            Cid::from(TASK_VALIDATE),
            required_atom_cid.clone(),
            false,
        )),
    );
    runtime.register_worker(
        Cid::from(WORKER_DECISION_PACK),
        Box::new(StageWorker::new(
            "decision-pack",
            Cid::from(WORKER_DECISION_PACK),
            Cid::from(TASK_DECISION_PACK),
            required_atom_cid,
            false,
        )),
    );

    let proof = run(&mut session, &contract, &mut runtime);
    let had_yield = !proof.continuation_cids_used.is_empty();
    let needed_witness = document_payload
        .windows(b"owner=".len())
        .any(|w| w == b"owner=");
    let hot_atoms_after_run = atom_space
        .heats
        .iter()
        .filter_map(|(cid, heat)| {
            if *heat == EpistemicHeat::Hot {
                Some(cid.clone())
            } else {
                None
            }
        })
        .collect();
    let atom_heat_after_run = atom_space
        .heats
        .iter()
        .map(|(cid, heat)| (cid.clone(), *heat))
        .collect();

    Ok(DomainExecution {
        proof,
        needed_witness,
        missing_field_name: "owner_document_id".to_string(),
        missing_field_prompt: Cid::new(format!("cid:witness:prompt:{}", case_id.as_str())),
        had_yield,
        hot_atoms_after_run,
        atom_heat_after_run,
    })
}
