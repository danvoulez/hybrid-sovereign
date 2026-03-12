use std::collections::HashMap;

use crate::action::{
    ComputeAction, FinalOutcome, MaterializeAction, RejectReason, StepAction, StepDecision,
    WitnessAction,
};
use crate::contract::Contract;
use crate::host_env::AtomSpaceHostEnv;
use crate::proof::ProofPack;
use crate::receipt::StepReceipt;
use crate::session::{Session, SessionView};
use epistemic_storage::{AtomSpace, EpistemicHeat};
use sovereign_core::{hash_canonical, Cid, ProofPackCid};
use worker_abi::{WorkerAbi, WorkerResult};

pub trait RuntimeOps {
    fn execute(
        &mut self,
        session: &mut Session,
        action: &StepAction,
    ) -> Result<Vec<StepReceipt>, String>;
    fn current_state_root(&self, session: &Session) -> Cid;
}

pub struct SovereignRuntime<'a> {
    pub atom_space: &'a mut dyn AtomSpace,
    pub workers: HashMap<Cid, Box<dyn WorkerAbi>>,
    pub auto_heat_on_yield: bool,
}

impl<'a> SovereignRuntime<'a> {
    pub fn new(atom_space: &'a mut dyn AtomSpace) -> Self {
        Self {
            atom_space,
            workers: HashMap::new(),
            auto_heat_on_yield: true,
        }
    }

    pub fn register_worker(&mut self, worker_cid: Cid, worker: Box<dyn WorkerAbi>) {
        self.workers.insert(worker_cid, worker);
    }
}

impl RuntimeOps for SovereignRuntime<'_> {
    fn execute(
        &mut self,
        session: &mut Session,
        action: &StepAction,
    ) -> Result<Vec<StepReceipt>, String> {
        match action {
            StepAction::Compute(compute_action) => match compute_action {
                ComputeAction::RunWorker {
                    worker_cid,
                    task_cid,
                } => {
                    let Self {
                        atom_space,
                        workers,
                        auto_heat_on_yield,
                    } = self;

                    let Some(worker) = workers.get_mut(worker_cid) else {
                        return Err(format!("missing worker: {}", worker_cid.as_str()));
                    };

                    let mut out = Vec::new();
                    let mut result = {
                        let mut env = AtomSpaceHostEnv {
                            atom_space: &mut **atom_space,
                            gas_remaining: &mut session.budget_remaining,
                        };
                        worker.execute(task_cid, &mut env)
                    };
                    for _ in 0..8 {
                        match result {
                            WorkerResult::Complete(receipt_cid) => {
                                out.push(StepReceipt::WorkerCompleted {
                                    worker_cid: worker_cid.clone(),
                                    task_cid: task_cid.clone(),
                                    receipt_cid,
                                });
                                return Ok(out);
                            }
                            WorkerResult::Yield(yielded) => {
                                out.push(StepReceipt::WorkerYielded {
                                    worker_cid: worker_cid.clone(),
                                    task_cid: task_cid.clone(),
                                    missing_cids: yielded.missing_cids.clone(),
                                    continuation_cid: yielded.continuation_cid.clone(),
                                });

                                if *auto_heat_on_yield {
                                    for cid in &yielded.missing_cids {
                                        atom_space.heat_up(cid, EpistemicHeat::Hot).map_err(
                                            |fault| {
                                                format!(
                                                    "heat_up failed for {}: {fault:?}",
                                                    cid.as_str()
                                                )
                                            },
                                        )?;
                                    }
                                }

                                result = {
                                    let mut env = AtomSpaceHostEnv {
                                        atom_space: &mut **atom_space,
                                        gas_remaining: &mut session.budget_remaining,
                                    };
                                    worker.resume(&yielded.continuation_cid, &mut env)
                                };
                            }
                            WorkerResult::Fail(err) => {
                                return Err(format!("worker failure: {err:?}"));
                            }
                        }
                    }

                    Err("worker exceeded continuation depth".to_string())
                }
                ComputeAction::Propose {
                    proposer_id,
                    input_set_cid,
                } => Ok(vec![StepReceipt::ProposalCreated {
                    proposal_cid: Cid::new(format!(
                        "cid:proposal:{}",
                        hash_canonical(&[proposer_id.as_str(), input_set_cid.as_str()]).as_str()
                    )),
                    producer_hash: sovereign_core::Hash::from(proposer_id.as_str()),
                }]),
                ComputeAction::RunExpert {
                    expert_id,
                    input_set_cid,
                } => Err(format!(
                    "run expert is not implemented in SovereignRuntime yet (expert_id={} input_set_cid={})",
                    expert_id,
                    input_set_cid.as_str()
                )),
                ComputeAction::RecomputePath { derivation_cid } => Err(format!(
                    "recompute path is not implemented in SovereignRuntime yet (derivation_cid={})",
                    derivation_cid.as_str()
                )),
            },
            StepAction::Materialize(materialize_action) => match materialize_action {
                MaterializeAction::RehydrateAtom { cid } => {
                    self.atom_space
                        .heat_up(cid, EpistemicHeat::Hot)
                        .map_err(|fault| format!("materialize failed: {fault:?}"))?;
                    Ok(vec![StepReceipt::AtomMaterialized {
                        atom_cid: cid.clone(),
                    }])
                }
                MaterializeAction::RetrieveEvidence { query_cid, top_k: _ } => {
                    self.atom_space
                        .heat_up(query_cid, EpistemicHeat::Hot)
                        .map_err(|fault| format!("retrieve evidence failed: {fault:?}"))?;
                    Ok(vec![StepReceipt::AtomMaterialized {
                        atom_cid: query_cid.clone(),
                    }])
                }
                MaterializeAction::LoadModule { module_id } => {
                    Err(format!("load module is not implemented yet (module_id={module_id})"))
                }
            },
            StepAction::Witness(witness_action) => match witness_action {
                WitnessAction::AskUserBit {
                    question_id,
                    left,
                    right,
                } => Ok(vec![StepReceipt::HumanWitnessed {
                    witness_kind: format!("ask-user-bit:{question_id}:{left}|{right}"),
                    answer_cid: Cid::new(format!("cid:witness:bit:{question_id}")),
                }]),
                WitnessAction::AskUserField { field_id } => Ok(vec![StepReceipt::HumanWitnessed {
                    witness_kind: format!("ask-user-field:{field_id}"),
                    answer_cid: Cid::new(format!("cid:witness:field:{field_id}")),
                }]),
                WitnessAction::GetTime { oracle_id } => Ok(vec![StepReceipt::HumanWitnessed {
                    witness_kind: format!("get-time:{oracle_id}"),
                    answer_cid: Cid::new(format!("cid:witness:time:{oracle_id}")),
                }]),
                WitnessAction::FetchExternalAtom {
                    locator,
                    expected_cid,
                } => Ok(vec![StepReceipt::HumanWitnessed {
                    witness_kind: format!("fetch-external-atom:{locator}"),
                    answer_cid: expected_cid
                        .clone()
                        .unwrap_or_else(|| Cid::new(format!("cid:witness:fetch:{locator}"))),
                }]),
            },
        }
    }

    fn current_state_root(&self, session: &Session) -> Cid {
        let metrics = self.atom_space.get_thermal_metrics();
        let transcript_head = session.transcript_head();
        Cid::new(
            hash_canonical(&[
                session.state_root.as_str(),
                transcript_head.as_str(),
                &metrics.hot_atoms.to_string(),
                &metrics.warm_atoms.to_string(),
                &metrics.cold_atoms.to_string(),
            ])
            .as_str(),
        )
    }
}

fn build_proof(session: &Session, final_outcome: FinalOutcome) -> ProofPack {
    let transcript_head = session.transcript_head();
    let proof_pack_cid = ProofPackCid::new(
        hash_canonical(&[
            session.case_id.as_str(),
            session.contract_hash.as_str(),
            transcript_head.as_str(),
        ])
        .as_str(),
    );

    ProofPack {
        proof_pack_cid,
        case_id: session.case_id.clone(),
        contract_hash: session.contract_hash.clone(),
        initial_budget: session.initial_budget,
        event_count: session.transcript.len() as u64,
        transcript_head,
        transcript_receipts: session
            .transcript
            .iter()
            .map(|entry| entry.receipt.clone())
            .collect(),
        proof_mode: session.proof_mode.clone(),
        final_state_root: session.state_root.clone(),
        final_outcome,
        final_receipt_cid: session.final_receipt_cid.clone(),
        worker_cid: session.last_worker_cid.clone(),
        task_cid: session.last_task_cid.clone(),
        continuation_cids_used: session.continuation_cids_used.clone(),
        manager_receipt_cids: session.manager_receipt_cids.clone(),
    }
}

pub fn run(session: &mut Session, contract: &dyn Contract, rt: &mut dyn RuntimeOps) -> ProofPack {
    loop {
        let view = SessionView::from(&*session);
        match contract.eval_step(&view) {
            StepDecision::Commit => {
                let output_cid = session
                    .final_receipt_cid
                    .as_ref()
                    .map(|r| Cid::new(r.as_str()))
                    .unwrap_or_else(|| session.state_root.clone());
                let proof = build_proof(session, FinalOutcome::Commit { output_cid });
                session.final_proof_pack_cid = Some(proof.proof_pack_cid.clone());
                return proof;
            }
            StepDecision::Reject(reason) => {
                let proof = build_proof(session, FinalOutcome::Reject { reason });
                session.final_proof_pack_cid = Some(proof.proof_pack_cid.clone());
                return proof;
            }
            StepDecision::Continue(action) => {
                let cost = contract.cost_of(&action, &view);
                if session.budget_remaining < cost {
                    let proof = build_proof(
                        session,
                        FinalOutcome::Reject {
                            reason: RejectReason::OutOfBudget,
                        },
                    );
                    session.final_proof_pack_cid = Some(proof.proof_pack_cid.clone());
                    return proof;
                }
                let action_canonical = action.canonical();
                let budget_before = session.budget_remaining;
                session.budget_remaining -= cost;

                match rt.execute(session, &action) {
                    Ok(receipts) => {
                        for receipt in receipts {
                            let budget_after = session.budget_remaining;
                            let state_root_before = session.state_root.clone();
                            let state_root_after = rt.current_state_root(session);
                            match &receipt {
                                StepReceipt::WorkerYielded {
                                    continuation_cid, ..
                                } => {
                                    session
                                        .continuation_cids_used
                                        .push(continuation_cid.clone());
                                }
                                StepReceipt::WorkerCompleted {
                                    worker_cid,
                                    task_cid,
                                    receipt_cid,
                                } => {
                                    session.last_worker_cid = Some(worker_cid.clone());
                                    session.last_task_cid = Some(task_cid.clone());
                                    session.final_receipt_cid = Some(receipt_cid.clone());
                                }
                                _ => {}
                            }
                            session.append_receipt(
                                receipt,
                                action_canonical.clone(),
                                budget_before,
                                budget_after,
                                state_root_before,
                                state_root_after.clone(),
                            );
                            session.state_root = state_root_after;
                        }
                    }
                    Err(_) => {
                        let proof = build_proof(
                            session,
                            FinalOutcome::Reject {
                                reason: RejectReason::InternalExecutionFailure,
                            },
                        );
                        session.final_proof_pack_cid = Some(proof.proof_pack_cid.clone());
                        return proof;
                    }
                }
            }
        }
    }
}
