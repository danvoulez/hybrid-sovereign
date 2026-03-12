use epistemic_storage::{
    AtomBody, AtomHeader, AtomKind, AtomSpace, InMemoryAtomSpace, StatePointer, UniversalAtom,
};
use frugal_decision::{gate_run, BudgetContract, ErrorContractQ16, GateInputs, ProposalEnvelope};
use manager_plane::{BudgetState, DemoManagerPlane, ManagedCase, ManagerInput, ManagerPlane};
use proof_federation::{
    BasicPointerValidator, FederationView, PointerClass, PointerPolicy, PointerValidator,
};
use proof_runtime::{
    run, ComputeAction, Contract, DeterminismProfile, ExecutionTarget, ProofMode, Session,
    SovereignRuntime, StepAction, StepDecision,
};
use sovereign_core::{
    hash_canonical, BudgetAmount, CaseId, Cid, Hash, NodeId, PointerAlias, Signature,
};
use worker_abi::{
    verify_silicon_execution, SiliconReceipt, WorkerAbi, WorkerError, WorkerHostEnv, WorkerResult,
    WorkerYield,
};

#[derive(Debug)]
struct DemoContract {
    worker_cid: Cid,
    task_cid: Cid,
}

impl Contract for DemoContract {
    fn eval_step(&self, session: &proof_runtime::SessionView) -> StepDecision {
        match session.transcript_len {
            0 => StepDecision::Continue(StepAction::Compute(ComputeAction::RunWorker {
                worker_cid: self.worker_cid.clone(),
                task_cid: self.task_cid.clone(),
            })),
            _ => StepDecision::Commit,
        }
    }

    fn cost_of(&self, _action: &StepAction, _session: &proof_runtime::SessionView) -> u64 {
        5
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
struct YieldingWorker {
    required_atom_cid: Cid,
    pending_continuation: Option<Cid>,
}

impl YieldingWorker {
    fn new(required_atom_cid: Cid) -> Self {
        Self {
            required_atom_cid,
            pending_continuation: None,
        }
    }

    fn continuation_for(&self, task_cid: &Cid) -> Cid {
        Cid::new(format!(
            "cid:continuation:{}",
            hash_canonical(&[task_cid.as_str(), self.required_atom_cid.as_str()]).as_str()
        ))
    }

    fn receipt_for(task_cid: &Cid, atom_bytes: &[u8]) -> sovereign_core::ReceiptCid {
        sovereign_core::ReceiptCid::new(format!(
            "cid:receipt:{}",
            hash_canonical(&[
                task_cid.as_str(),
                &String::from_utf8_lossy(atom_bytes),
                "worker:yielding:v1",
            ])
            .as_str()
        ))
    }
}

impl WorkerAbi for YieldingWorker {
    fn execute(&mut self, task_cid: &Cid, env: &mut dyn WorkerHostEnv) -> WorkerResult {
        if env.consume_gas(2).is_err() {
            return WorkerResult::Fail(WorkerError::InternalFailure);
        }

        match env.request_atom(&self.required_atom_cid) {
            Ok(bytes) => WorkerResult::Complete(Self::receipt_for(task_cid, &bytes)),
            Err(_) => {
                let continuation = self.continuation_for(task_cid);
                self.pending_continuation = Some(continuation.clone());
                WorkerResult::Yield(WorkerYield {
                    missing_cids: vec![self.required_atom_cid.clone()],
                    continuation_cid: continuation,
                })
            }
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
                let task_cid = Cid::from("cid:task:one");
                WorkerResult::Complete(Self::receipt_for(&task_cid, &bytes))
            }
            Err(_) => WorkerResult::Fail(WorkerError::InternalFailure),
        }
    }
}

fn main() {
    let worker_cid = Cid::from("worker:yielding:v1");
    let task_cid = Cid::from("cid:task:one");
    let required_atom_cid = Cid::from("cid:atom:knowledge:one");

    let mut atom_space = InMemoryAtomSpace::default();
    let atom = UniversalAtom {
        header: AtomHeader {
            kind: AtomKind::Task,
            size_bytes: 7,
            producer_hash: Hash::from("producer:demo"),
            signature: Some(Signature::from("sig:demo")),
        },
        links: vec![],
        body: AtomBody::Inline(b"context".to_vec()),
    };
    atom_space
        .materialize(required_atom_cid.clone(), atom)
        .unwrap();

    let err = ErrorContractQ16 {
        epsilon_q16: 32,
        zero_guess_domains: vec!["finance".to_string()],
        max_questions_per_case: 1,
        max_ghosts_per_epoch: 8,
        ok_min_q16: 50000,
        reject_max_q16: 30000,
        max_risk_q16: 10000,
    };
    let budget = BudgetContract {
        max_ram_mb: 512,
        max_vram_mb: 512,
        max_loaded_params_mb: 256,
        max_live_context_tokens: 4096,
        max_rehydrations: 4,
        max_escalations: 1,
        max_hot_atoms: 32,
    };
    let proposal = ProposalEnvelope {
        hypothesis_cid: Cid::from("cid:hypothesis:001"),
        score_q16: 54000,
        risk_q16: 5000,
        required_atoms: vec![required_atom_cid.clone()],
        required_workers: vec![worker_cid.as_str().to_string()],
        estimated_ram_mb: 16,
        estimated_vram_mb: 0,
        estimated_params_mb: 4,
        producer_hash: Hash::from("worker:yielding:v1"),
    };
    let gate = gate_run(
        GateInputs {
            domain: "ops",
            has_intent: true,
            has_minimum_evidence: true,
            evidence_anchored: true,
            deterministic_proof: true,
            questions_used: 0,
            ghosts_used_in_epoch: 0,
            escalations_used: 0,
            rehydrations_used: 0,
            live_ram_mb: 64,
            live_vram_mb: 0,
            loaded_params_mb: 0,
            live_context_tokens: 128,
            hot_atoms: 0,
            err: &err,
            budget: &budget,
        },
        &proposal,
    );
    println!("gate verdict: {:?}", gate.verdict);

    let mut manager = DemoManagerPlane::default();
    manager.cases.insert(
        "case-001".to_string(),
        ManagedCase {
            case_id: CaseId::from("case-001"),
            state_root: Cid::from("cid:state:root:001"),
            current_head_cid: None,
            active_budget: BudgetState {
                gas_remaining: BudgetAmount(40),
                max_parallel_workers: 2,
                max_open_cases: 8,
                max_human_interrupts: 1,
            },
            pending_events: vec![],
            pending_actions: vec![],
            latest_proof_pack_cid: None,
            blocked_on: None,
        },
    );

    manager
        .ingest(ManagerInput::Event(task_cid.clone()))
        .unwrap();
    let delegated = manager.evaluate_next("case-001").unwrap();
    println!("manager delegate: {:?}", delegated);

    let mut session = Session {
        case_id: CaseId::from("case-001"),
        contract_hash: Hash::from("contract:demo:v6.1"),
        initial_budget: 60,
        budget_remaining: 60,
        state_root: Cid::from("cid:state:root:001"),
        proof_mode: ProofMode::AnchoredImmutableRefs,
        transcript: vec![],
        final_receipt_cid: None,
        final_proof_pack_cid: None,
        last_worker_cid: None,
        last_task_cid: None,
        continuation_cids_used: vec![],
        manager_receipt_cids: vec![],
    };

    let contract = DemoContract {
        worker_cid: worker_cid.clone(),
        task_cid: task_cid.clone(),
    };
    let mut runtime = SovereignRuntime::new(&mut atom_space);
    runtime.register_worker(
        worker_cid.clone(),
        Box::new(YieldingWorker::new(required_atom_cid.clone())),
    );
    let proof = run(&mut session, &contract, &mut runtime);
    println!("proof outcome: {:?}", proof.final_outcome);

    let final_receipt = proof
        .final_receipt_cid
        .clone()
        .expect("final receipt cid should be present");
    manager
        .ingest(ManagerInput::WorkerCompleted {
            receipt_cid: final_receipt.clone(),
            proof_pack_cid: proof.proof_pack_cid.clone(),
        })
        .unwrap();
    let pointer_output = manager.evaluate_next("case-001").unwrap();
    println!("manager pointer output: {:?}", pointer_output);

    let previous = StatePointer {
        alias: PointerAlias::from("cases:case-001:latest"),
        prev_head_cid: None,
        head_cid: Cid::from("cid:head:previous"),
        sequence_number: 1,
        authority_id: NodeId::from("node-a"),
        authority_signature: Signature::from("sig-a"),
    };

    let candidate_head = Cid::new(proof.proof_pack_cid.as_str());
    let candidate = StatePointer {
        alias: PointerAlias::from("cases:case-001:latest"),
        prev_head_cid: Some(previous.head_cid.clone()),
        head_cid: candidate_head,
        sequence_number: 2,
        authority_id: NodeId::from("node-a"),
        authority_signature: Signature::from("sig-a2"),
    };

    let federation = FederationView {
        recognized_nodes: vec![],
        accepted_contract_hashes: vec![Hash::from("contract:demo:v6.1")],
        accepted_proof_packs: vec![proof.proof_pack_cid.clone()],
        pointer_policies: vec![PointerPolicy {
            alias_prefix: "cases:".to_string(),
            class: PointerClass::SharedCase,
            accepted_authorities: vec![NodeId::from("node-a")],
            requires_quorum: false,
            quorum_size: 0,
            allow_forks: false,
            require_proof_pack: true,
        }],
        acceptance_receipts: vec![],
    };
    let validator = BasicPointerValidator;
    let verdict = validator.validate_pointer(
        &candidate,
        Some(&previous),
        Some(&proof.proof_pack_cid),
        &federation,
    );
    println!("federation verdict: {:?}", verdict);

    let expected = SiliconReceipt {
        task_cid: task_cid.clone(),
        result_vector: vec![10, 20, 30],
        hardware_signature: Signature::from("gpu:a"),
    };
    let recomputed = SiliconReceipt {
        task_cid,
        result_vector: vec![11, 19, 30],
        hardware_signature: Signature::from("gpu:b"),
    };
    println!(
        "bounded silicon verification: {}",
        verify_silicon_execution(&expected, &recomputed, 3.0)
    );
}
