use manager_plane::{ManagerInput, ManagerOutput, ManagerPlane};
use proof_runtime::StepReceipt;
use sovereign_core::{CaseId, Cid};

use crate::app_state::{
    AppState, CaseStatus, PendingCompletion, WitnessKind, WitnessTask, WorkerLedgerEntry,
};
use crate::demo_domain::{
    entry_task_cid, entry_worker_cid, execute_document_case, stage_name_from_worker,
};

#[derive(Debug, Clone)]
pub struct CaseRunReport {
    pub case_id: CaseId,
    pub delegated_worker: Cid,
    pub delegated_task: Cid,
    pub had_yield: bool,
    pub witness_required: bool,
    pub proof_pack_cid: sovereign_core::ProofPackCid,
}

pub struct CaseService;

fn stage_insight(stage: &str, action: &str) -> &'static str {
    match (stage, action) {
        ("intake", "yield") => "atom de payload estava frio; runtime precisa heat-up para retomar",
        ("intake", "complete") => "classificacao inicial concluida e trilha de evidencia aberta",
        ("extract", "complete") => "campos chave extraidos e lacunas do documento explicitadas",
        ("validate", "complete") => "regras obrigatorias avaliadas e bloqueio do caso determinado",
        ("decision-pack", "complete") => "resultado consolidado para decisao final governada",
        (_, "yield") => "worker pausou aguardando materializacao de evidencia",
        _ => "worker concluiu etapa operacional",
    }
}

fn build_operational_projection(
    proof: &proof_runtime::ProofPack,
) -> (Vec<String>, Vec<WorkerLedgerEntry>) {
    let mut timeline = Vec::new();
    let mut ledger = Vec::new();
    for (idx, entry) in proof.transcript_receipts.iter().enumerate() {
        match entry {
            StepReceipt::WorkerYielded {
                worker_cid,
                task_cid,
                missing_cids,
                continuation_cid,
            } => {
                let stage = stage_name_from_worker(worker_cid);
                let insight = stage_insight(stage, "yield").to_string();
                let missing = missing_cids
                    .iter()
                    .map(|c| c.as_str())
                    .collect::<Vec<_>>()
                    .join(",");
                timeline.push(format!(
                    "#{idx:02} stage={} action=yield worker={} task={} missing=[{}] continuation={} insight='{}'",
                    stage,
                    worker_cid.as_str(),
                    task_cid.as_str(),
                    missing,
                    continuation_cid.as_str(),
                    insight
                ));
                ledger.push(WorkerLedgerEntry {
                    stage: stage.to_string(),
                    action: "yield".to_string(),
                    worker_cid: worker_cid.clone(),
                    task_cid: task_cid.clone(),
                    insight,
                    artifact_cids: {
                        let mut artifacts = vec![continuation_cid.clone()];
                        artifacts.extend(missing_cids.iter().cloned());
                        artifacts
                    },
                });
            }
            StepReceipt::WorkerCompleted {
                worker_cid,
                task_cid,
                receipt_cid,
            } => {
                let stage = stage_name_from_worker(worker_cid);
                let insight = stage_insight(stage, "complete").to_string();
                timeline.push(format!(
                    "#{idx:02} stage={} action=complete worker={} task={} receipt={} insight='{}'",
                    stage,
                    worker_cid.as_str(),
                    task_cid.as_str(),
                    receipt_cid.as_str(),
                    insight
                ));
                ledger.push(WorkerLedgerEntry {
                    stage: stage.to_string(),
                    action: "complete".to_string(),
                    worker_cid: worker_cid.clone(),
                    task_cid: task_cid.clone(),
                    insight,
                    artifact_cids: vec![Cid::new(receipt_cid.as_str())],
                });
            }
            other => timeline.push(format!("#{idx:02} action=other receipt={other:?}")),
        }
    }
    (timeline, ledger)
}

impl CaseService {
    pub fn enqueue_document_event(
        state: &mut AppState,
        case_id: &str,
        task_cid: Cid,
    ) -> Result<(), String> {
        state
            .manager
            .ingest(ManagerInput::Event(task_cid))
            .map_err(|e| format!("ingest event failed: {e}"))?;
        state
            .statuses
            .insert(case_id.to_string(), CaseStatus::Running);
        Ok(())
    }

    pub fn run_case_once(state: &mut AppState, case_id: &str) -> Result<CaseRunReport, String> {
        let decision = state
            .manager
            .evaluate_next(case_id)
            .map_err(|e| format!("manager evaluate failed: {e}"))?;

        let (worker_cid, task_cid) = match decision {
            ManagerOutput::Delegate {
                worker_cid,
                task_cid,
            } => (worker_cid, task_cid),
            other => return Err(format!("expected delegate, got {other:?}")),
        };
        if worker_cid != entry_worker_cid() || task_cid != entry_task_cid() {
            return Err(format!(
                "unexpected delegate route worker={} task={}",
                worker_cid.as_str(),
                task_cid.as_str()
            ));
        }

        let case_id_t = CaseId::from(case_id);
        let execution = execute_document_case(&case_id_t)?;
        let proof = execution.proof;
        let needed_witness = execution.needed_witness;
        let missing_field_prompt = execution.missing_field_prompt;
        let missing_field_name = execution.missing_field_name;
        let had_yield = execution.had_yield;
        let hot_atoms_after_run = execution.hot_atoms_after_run;
        let atom_heat_after_run = execution.atom_heat_after_run;
        let (timeline, ledger) = build_operational_projection(&proof);
        state
            .hot_atoms_by_case
            .insert(case_id.to_string(), hot_atoms_after_run);
        state.case_atom_heat.insert(
            case_id.to_string(),
            atom_heat_after_run.into_iter().collect(),
        );
        let proof_pack_cid = proof.proof_pack_cid.clone();
        let final_receipt = proof
            .final_receipt_cid
            .clone()
            .ok_or_else(|| "proof missing final receipt cid".to_string())?;
        state.proofs.insert(case_id.to_string(), proof);
        state.case_timeline.insert(case_id.to_string(), timeline);
        state
            .worker_ledger_by_case
            .insert(case_id.to_string(), ledger);

        if needed_witness {
            let witness_confirm = WitnessTask {
                witness_id: Cid::new(format!("cid:witness:confirm:{case_id}")),
                case_id: case_id_t.clone(),
                prompt_cid: Cid::new(format!("cid:witness:prompt:confirm:{case_id}")),
                kind: WitnessKind::BinaryConfirm {
                    question: "Documento pertence ao caso correto?".to_string(),
                    positive_label: "Sim".to_string(),
                    negative_label: "Nao".to_string(),
                },
            };
            let witness_field_fill = WitnessTask {
                witness_id: Cid::new(format!("cid:witness:field-fill:{case_id}")),
                case_id: case_id_t.clone(),
                prompt_cid: missing_field_prompt,
                kind: WitnessKind::FieldFill {
                    field_name: missing_field_name,
                    prompt: "Informe numero do documento".to_string(),
                },
            };
            let witness_approval = WitnessTask {
                witness_id: Cid::new(format!("cid:witness:approval:{case_id}")),
                case_id: case_id_t.clone(),
                prompt_cid: Cid::new(format!("cid:witness:prompt:approval:{case_id}")),
                kind: WitnessKind::ApproveReject {
                    reason: "Autorizar excecao de politica para documento incompleto".to_string(),
                },
            };
            let required_witness_ids = vec![
                witness_confirm.witness_id.clone(),
                witness_field_fill.witness_id.clone(),
                witness_approval.witness_id.clone(),
            ];
            state.witness_inbox.push(WitnessTask { ..witness_confirm });
            state.witness_inbox.push(WitnessTask {
                ..witness_field_fill
            });
            state.witness_inbox.push(WitnessTask { ..witness_approval });
            state.pending_completions.insert(
                case_id.to_string(),
                PendingCompletion {
                    receipt_cid: final_receipt,
                    proof_pack_cid: proof_pack_cid.clone(),
                    required_witness_ids,
                    resolved_witness_ids: vec![],
                },
            );
            state
                .statuses
                .insert(case_id.to_string(), CaseStatus::BlockedOnWitness);
        } else {
            state
                .manager
                .ingest(ManagerInput::WorkerCompleted {
                    receipt_cid: final_receipt,
                    proof_pack_cid: proof_pack_cid.clone(),
                })
                .map_err(|e| format!("ingest completion failed: {e}"))?;
            state
                .statuses
                .insert(case_id.to_string(), CaseStatus::Committed);
        }

        Ok(CaseRunReport {
            case_id: case_id_t,
            delegated_worker: worker_cid,
            delegated_task: task_cid,
            had_yield,
            witness_required: needed_witness,
            proof_pack_cid,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use sovereign_core::Cid;

    use super::CaseService;
    use crate::app_state::AppState;
    use crate::demo_domain::entry_task_cid;

    #[test]
    fn timeline_is_operationally_legible_across_workers() {
        let mut state = AppState::new_seeded();
        let case_id = "case-doc-001";

        CaseService::enqueue_document_event(&mut state, case_id, entry_task_cid()).unwrap();
        let report = CaseService::run_case_once(&mut state, case_id).unwrap();
        assert!(report.had_yield);
        assert_eq!(
            report.delegated_worker,
            Cid::from("worker:document-intake:v7")
        );

        let timeline = state
            .case_timeline
            .get(case_id)
            .expect("timeline should exist");
        assert!(timeline
            .iter()
            .any(|line| line.contains("stage=intake action=yield")));
        assert!(timeline
            .iter()
            .any(|line| line.contains("stage=extract action=complete")));
        assert!(timeline
            .iter()
            .any(|line| line.contains("stage=validate action=complete")));
        assert!(timeline
            .iter()
            .any(|line| line.contains("stage=decision-pack action=complete")));
    }

    #[test]
    fn each_worker_stage_adds_distinct_operational_insight() {
        let mut state = AppState::new_seeded();
        let case_id = "case-doc-001";

        CaseService::enqueue_document_event(&mut state, case_id, entry_task_cid()).unwrap();
        CaseService::run_case_once(&mut state, case_id).unwrap();

        let ledger = state
            .worker_ledger_by_case
            .get(case_id)
            .expect("worker ledger should exist");

        let complete_entries = ledger
            .iter()
            .filter(|entry| entry.action == "complete")
            .collect::<Vec<_>>();
        assert_eq!(complete_entries.len(), 4);

        let unique_insights = complete_entries
            .iter()
            .map(|entry| entry.insight.clone())
            .collect::<HashSet<_>>();
        assert_eq!(unique_insights.len(), 4);
    }
}
