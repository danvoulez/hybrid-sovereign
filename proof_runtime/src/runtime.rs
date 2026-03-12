use std::collections::HashMap;

use crate::action::{ComputeAction, FinalOutcome, RejectReason, StepAction, StepDecision};
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
            StepAction::Compute(ComputeAction::RunWorker {
                worker_cid,
                task_cid,
            }) => {
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
            StepAction::Materialize { cid } => {
                self.atom_space
                    .heat_up(cid, EpistemicHeat::Hot)
                    .map_err(|fault| format!("materialize failed: {fault:?}"))?;
                Ok(vec![StepReceipt::AtomMaterialized {
                    atom_cid: cid.clone(),
                }])
            }
            StepAction::Witness {
                witness_kind,
                prompt_cid,
            } => Ok(vec![StepReceipt::HumanWitnessed {
                witness_kind: witness_kind.clone(),
                answer_cid: prompt_cid.clone(),
            }]),
        }
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
                let proof = build_proof(session, FinalOutcome::Commit);
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
                session.budget_remaining -= cost;

                match rt.execute(session, &action) {
                    Ok(receipts) => {
                        for receipt in receipts {
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
                            session.append_receipt(receipt);
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
