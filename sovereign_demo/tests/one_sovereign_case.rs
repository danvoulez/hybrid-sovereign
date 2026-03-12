use epistemic_storage::{
    AtomBody, AtomHeader, AtomKind, AtomSpace, EpistemicHeat, InMemoryAtomSpace, StatePointer,
    UniversalAtom,
};
use manager_plane::{
    BudgetState, DemoManagerPlane, ManagedCase, ManagerInput, ManagerOutput, ManagerPlane,
};
use proof_federation::{
    AcceptanceVerdict, BasicPointerValidator, FederationView, PointerClass, PointerPolicy,
    PointerValidator,
};
use proof_runtime::{
    run, ComputeAction, Contract, DeterminismProfile, ExecutionTarget, FinalOutcome, ProofMode,
    Session, SovereignRuntime, StepAction, StepDecision, StepReceipt,
};
use sovereign_core::{
    hash_canonical, BudgetAmount, CaseId, Cid, Hash, NodeId, PointerAlias, Signature,
};
use worker_abi::{WorkerAbi, WorkerError, WorkerHostEnv, WorkerResult, WorkerYield};

#[derive(Debug)]
struct OneCaseContract {
    worker_cid: Cid,
    task_cid: Cid,
}

impl Contract for OneCaseContract {
    fn eval_step(&self, session: &proof_runtime::SessionView) -> StepDecision {
        if session.transcript_len == 0 {
            StepDecision::Continue(StepAction::Compute(ComputeAction::RunWorker {
                worker_cid: self.worker_cid.clone(),
                task_cid: self.task_cid.clone(),
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
struct YieldingWorker {
    required_atom_cid: Cid,
    task_cid: Cid,
    pending_continuation: Option<Cid>,
}

impl YieldingWorker {
    fn new(required_atom_cid: Cid, task_cid: Cid) -> Self {
        Self {
            required_atom_cid,
            task_cid,
            pending_continuation: None,
        }
    }

    fn continuation(&self) -> Cid {
        Cid::new(format!(
            "cid:continuation:{}",
            hash_canonical(&[self.task_cid.as_str(), self.required_atom_cid.as_str()]).as_str()
        ))
    }

    fn receipt(&self, atom_bytes: &[u8]) -> sovereign_core::ReceiptCid {
        sovereign_core::ReceiptCid::new(format!(
            "cid:receipt:{}",
            hash_canonical(&[
                self.task_cid.as_str(),
                &String::from_utf8_lossy(atom_bytes),
                "worker:yield:v6.1",
            ])
            .as_str()
        ))
    }
}

impl WorkerAbi for YieldingWorker {
    fn execute(&mut self, _task_cid: &Cid, env: &mut dyn WorkerHostEnv) -> WorkerResult {
        let _ = env.consume_gas(1);
        match env.request_atom(&self.required_atom_cid) {
            Ok(bytes) => WorkerResult::Complete(self.receipt(&bytes)),
            Err(_) => {
                let continuation = self.continuation();
                self.pending_continuation = Some(continuation.clone());
                WorkerResult::Yield(WorkerYield {
                    missing_cids: vec![self.required_atom_cid.clone()],
                    continuation_cid: continuation,
                })
            }
        }
    }

    fn resume(&mut self, continuation_cid: &Cid, env: &mut dyn WorkerHostEnv) -> WorkerResult {
        let _ = env.consume_gas(1);
        if self.pending_continuation.as_ref() != Some(continuation_cid) {
            return WorkerResult::Fail(WorkerError::InvalidTask);
        }
        match env.request_atom(&self.required_atom_cid) {
            Ok(bytes) => {
                self.pending_continuation = None;
                WorkerResult::Complete(self.receipt(&bytes))
            }
            Err(_) => WorkerResult::Fail(WorkerError::InternalFailure),
        }
    }
}

fn materialize_cold(space: &mut InMemoryAtomSpace, cid: &Cid) {
    let atom = UniversalAtom {
        header: AtomHeader {
            kind: AtomKind::Task,
            size_bytes: 11,
            producer_hash: Hash::from("producer:test"),
            signature: Some(Signature::from("sig:test")),
        },
        links: vec![],
        body: AtomBody::Inline(b"cold-bytes".to_vec()),
    };
    space.materialize(cid.clone(), atom).unwrap();
}

fn run_case_once() -> (proof_runtime::ProofPack, Session, InMemoryAtomSpace) {
    let case_id = CaseId::from("case:one");
    let worker_cid = Cid::from("worker:yield:v6.1");
    let task_cid = Cid::from("cid:task:one");
    let missing_atom_cid = Cid::from("cid:atom:required:one");

    let mut atom_space = InMemoryAtomSpace::default();
    materialize_cold(&mut atom_space, &missing_atom_cid);

    let mut session = Session {
        case_id,
        contract_hash: Hash::from("contract:v6.1"),
        initial_budget: 40,
        budget_remaining: 40,
        state_root: Cid::from("cid:state:root:one"),
        proof_mode: ProofMode::AnchoredImmutableRefs,
        transcript: vec![],
        final_receipt_cid: None,
        final_proof_pack_cid: None,
        last_worker_cid: None,
        last_task_cid: None,
        continuation_cids_used: vec![],
        manager_receipt_cids: vec![],
    };

    let contract = OneCaseContract {
        worker_cid: worker_cid.clone(),
        task_cid: task_cid.clone(),
    };

    {
        let mut runtime = SovereignRuntime::new(&mut atom_space);
        runtime.register_worker(
            worker_cid,
            Box::new(YieldingWorker::new(missing_atom_cid.clone(), task_cid)),
        );
        let proof = run(&mut session, &contract, &mut runtime);
        return (proof, session, atom_space);
    }
}

#[test]
fn one_sovereign_case() {
    let (proof, session, atom_space) = run_case_once();

    assert!(matches!(
        &proof.final_outcome,
        FinalOutcome::Commit { output_cid } if *output_cid
            == Cid::new(
                proof
                    .final_receipt_cid
                    .as_ref()
                    .expect("final receipt must exist for commit")
                    .as_str()
            )
    ));
    assert!(proof.final_receipt_cid.is_some());
    assert_eq!(proof.event_count, 2);
    assert_eq!(session.transcript.len(), 2);

    match &session.transcript[0].receipt {
        StepReceipt::WorkerYielded {
            missing_cids,
            continuation_cid,
            ..
        } => {
            assert_eq!(missing_cids.len(), 1);
            assert_eq!(continuation_cid, &proof.continuation_cids_used[0]);
        }
        other => panic!("expected WorkerYielded, got {other:?}"),
    }

    match &session.transcript[1].receipt {
        StepReceipt::WorkerCompleted { receipt_cid, .. } => {
            assert_eq!(Some(receipt_cid.clone()), proof.final_receipt_cid.clone());
        }
        other => panic!("expected WorkerCompleted, got {other:?}"),
    }
    assert!(session.transcript[0]
        .action_canonical
        .contains("compute|run_worker"));
    assert_eq!(session.transcript[0].budget_before, 40);
    assert!(session.transcript[0].budget_after <= session.transcript[0].budget_before);
    assert_eq!(
        session.transcript[0].state_root_before,
        Cid::from("cid:state:root:one")
    );
    assert_ne!(
        session.transcript[0].state_root_before,
        session.transcript[0].state_root_after
    );
    assert_eq!(
        session.transcript[1].state_root_before,
        session.transcript[0].state_root_after
    );
    assert_ne!(
        session.transcript[1].state_root_after,
        session.transcript[0].state_root_after
    );

    assert_eq!(
        atom_space.current_heat(&Cid::from("cid:atom:required:one")),
        EpistemicHeat::Hot
    );

    let mut manager = DemoManagerPlane::default();
    manager.cases.insert(
        "case:one".to_string(),
        ManagedCase {
            case_id: CaseId::from("case:one"),
            state_root: Cid::from("cid:state:root:one"),
            current_head_cid: None,
            active_budget: BudgetState {
                gas_remaining: BudgetAmount(10),
                max_parallel_workers: 1,
                max_open_cases: 1,
                max_human_interrupts: 1,
            },
            pending_events: vec![],
            pending_actions: vec![],
            latest_proof_pack_cid: None,
            blocked_on: None,
        },
    );

    manager
        .ingest(ManagerInput::Event(Cid::from("cid:task:one")))
        .unwrap();
    assert!(matches!(
        manager.evaluate_next("case:one").unwrap(),
        ManagerOutput::Delegate { .. }
    ));

    manager
        .ingest(ManagerInput::WorkerCompleted {
            receipt_cid: proof.final_receipt_cid.clone().unwrap(),
            proof_pack_cid: proof.proof_pack_cid.clone(),
        })
        .unwrap();

    let advance = manager.evaluate_next("case:one").unwrap();
    let (head_cid, proof_pack_cid) = match advance {
        ManagerOutput::AdvancePointer {
            head_cid,
            proof_pack_cid,
            ..
        } => (head_cid, proof_pack_cid),
        other => panic!("expected AdvancePointer, got {other:?}"),
    };
    assert_eq!(head_cid, Cid::new(proof.proof_pack_cid.as_str()));
    assert_eq!(proof_pack_cid, proof.proof_pack_cid.clone());

    let previous = StatePointer {
        alias: PointerAlias::from("cases:case:one:latest"),
        prev_head_cid: None,
        head_cid: Cid::from("cid:head:0"),
        sequence_number: 1,
        authority_id: NodeId::from("node-b"),
        authority_signature: Signature::from("sig:b"),
    };

    let candidate = StatePointer {
        alias: PointerAlias::from("cases:case:one:latest"),
        prev_head_cid: Some(previous.head_cid.clone()),
        head_cid: Cid::new(proof.proof_pack_cid.as_str()),
        sequence_number: 2,
        authority_id: NodeId::from("node-b"),
        authority_signature: Signature::from("sig:b2"),
    };

    let federation = FederationView {
        recognized_nodes: vec![],
        accepted_contract_hashes: vec![Hash::from("contract:v6.1")],
        accepted_proof_packs: vec![proof.proof_pack_cid.clone()],
        pointer_policies: vec![PointerPolicy {
            alias_prefix: "cases:".to_string(),
            class: PointerClass::SharedCase,
            accepted_authorities: vec![NodeId::from("node-b")],
            requires_quorum: false,
            quorum_size: 0,
            allow_forks: false,
            require_proof_pack: true,
        }],
        acceptance_receipts: vec![],
    };

    let validator = BasicPointerValidator;
    assert_eq!(
        validator.validate_pointer(
            &candidate,
            Some(&previous),
            Some(&proof.proof_pack_cid),
            &federation
        ),
        AcceptanceVerdict::Accepted
    );

    let (replay_proof, replay_session, replay_space) = run_case_once();
    assert_eq!(replay_proof.final_outcome, proof.final_outcome);
    assert_eq!(replay_proof.final_receipt_cid, proof.final_receipt_cid);
    assert_eq!(replay_proof.proof_pack_cid, proof.proof_pack_cid);
    assert_eq!(replay_session.transcript.len(), 2);
    assert_eq!(
        replay_space.current_heat(&Cid::from("cid:atom:required:one")),
        EpistemicHeat::Hot
    );
}
