use manager_plane::{ManagerInput, ManagerOutput, ManagerPlane};

use crate::app_state::{AppState, CaseStatus, WitnessKind, WitnessTask};

pub struct WitnessService;

impl WitnessService {
    fn resolve_one<F>(
        state: &mut AppState,
        case_id: &str,
        matcher: F,
        response_label: String,
    ) -> Result<String, String>
    where
        F: Fn(&WitnessTask) -> bool,
    {
        let witness_pos = state
            .witness_inbox
            .iter()
            .position(|w| w.case_id.as_str() == case_id && matcher(w))
            .ok_or_else(|| format!("matching witness not found for {case_id}"))?;
        let witness = state.witness_inbox.remove(witness_pos);

        state
            .manager
            .ingest(ManagerInput::Witness(witness.prompt_cid.clone()))
            .map_err(|e| format!("witness ingest failed: {e}"))?;

        state
            .case_timeline
            .entry(case_id.to_string())
            .or_default()
            .push(format!(
                "witness action=resolve witness_id={} kind={} response={}",
                witness.witness_id.as_str(),
                witness.kind_label(),
                response_label
            ));

        let (should_finalize, remaining_after) = {
            let pending = state
                .pending_completions
                .get_mut(case_id)
                .ok_or_else(|| format!("no pending completion for {case_id}"))?;

            if !pending
                .required_witness_ids
                .iter()
                .any(|w| w == &witness.witness_id)
            {
                return Err(format!(
                    "witness {} is not required for case {case_id}",
                    witness.witness_id.as_str()
                ));
            }

            if !pending
                .resolved_witness_ids
                .iter()
                .any(|w| w == &witness.witness_id)
            {
                pending
                    .resolved_witness_ids
                    .push(witness.witness_id.clone());
            }

            let remaining = pending
                .required_witness_ids
                .len()
                .saturating_sub(pending.resolved_witness_ids.len());
            (remaining == 0, remaining)
        };

        if !should_finalize {
            state
                .statuses
                .insert(case_id.to_string(), CaseStatus::BlockedOnWitness);
            return Ok(format!(
                "witness recorded; waiting {} additional witness(es)",
                remaining_after
            ));
        }

        let pending = state
            .pending_completions
            .remove(case_id)
            .ok_or_else(|| format!("pending completion disappeared for {case_id}"))?;

        state
            .manager
            .ingest(ManagerInput::WorkerCompleted {
                receipt_cid: pending.receipt_cid,
                proof_pack_cid: pending.proof_pack_cid,
            })
            .map_err(|e| format!("completion ingest failed: {e}"))?;

        let output = state
            .manager
            .evaluate_next(case_id)
            .map_err(|e| format!("manager evaluate after witness failed: {e}"))?;
        match output {
            ManagerOutput::AdvancePointer {
                alias,
                head_cid,
                proof_pack_cid,
            } => {
                if let Some(case) = state.manager.cases.get_mut(case_id) {
                    case.current_head_cid = Some(head_cid.clone());
                }
                state
                    .case_timeline
                    .entry(case_id.to_string())
                    .or_default()
                    .push(format!(
                        "witness-resolved action=advance-pointer alias={} head={} proof_pack={}",
                        alias.as_str(),
                        head_cid.as_str(),
                        proof_pack_cid.as_str()
                    ));
                state
                    .statuses
                    .insert(case_id.to_string(), CaseStatus::Committed);
                Ok("all witnesses resolved; pointer advanced".to_string())
            }
            other => Err(format!(
                "expected AdvancePointer after witness completion, got {other:?}"
            )),
        }
    }

    pub fn resolve_binary_confirm(
        state: &mut AppState,
        case_id: &str,
        accepted: bool,
    ) -> Result<String, String> {
        let label = if accepted { "A" } else { "B" }.to_string();
        Self::resolve_one(
            state,
            case_id,
            |w| matches!(w.kind, WitnessKind::BinaryConfirm { .. }),
            label,
        )
    }

    pub fn resolve_field_fill(
        state: &mut AppState,
        case_id: &str,
        value: &str,
    ) -> Result<String, String> {
        Self::resolve_one(
            state,
            case_id,
            |w| matches!(w.kind, WitnessKind::FieldFill { .. }),
            format!("value={value}"),
        )
    }

    pub fn resolve_approval(
        state: &mut AppState,
        case_id: &str,
        approved: bool,
    ) -> Result<String, String> {
        let label = if approved { "approved" } else { "rejected" }.to_string();
        Self::resolve_one(
            state,
            case_id,
            |w| matches!(w.kind, WitnessKind::ApproveReject { .. }),
            label,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::WitnessService;
    use crate::app_state::{AppState, CaseStatus};
    use crate::case_service::CaseService;
    use crate::demo_domain::entry_task_cid;

    #[test]
    fn case_only_unblocks_after_all_typed_witnesses() {
        let mut state = AppState::new_seeded();
        let case_id = "case-doc-001";

        CaseService::enqueue_document_event(&mut state, case_id, entry_task_cid()).unwrap();
        CaseService::run_case_once(&mut state, case_id).unwrap();
        assert_eq!(
            state.statuses.get(case_id),
            Some(&CaseStatus::BlockedOnWitness)
        );

        let r1 = WitnessService::resolve_binary_confirm(&mut state, case_id, true).unwrap();
        assert!(r1.contains("waiting 2"));
        assert!(state
            .manager
            .cases
            .get(case_id)
            .and_then(|c| c.current_head_cid.clone())
            .is_none());

        let r2 = WitnessService::resolve_field_fill(&mut state, case_id, "DOC-99821").unwrap();
        assert!(r2.contains("waiting 1"));
        assert!(state
            .manager
            .cases
            .get(case_id)
            .and_then(|c| c.current_head_cid.clone())
            .is_none());

        let r3 = WitnessService::resolve_approval(&mut state, case_id, true).unwrap();
        assert!(r3.contains("all witnesses resolved"));
        assert_eq!(state.statuses.get(case_id), Some(&CaseStatus::Committed));
        assert!(state
            .manager
            .cases
            .get(case_id)
            .and_then(|c| c.current_head_cid.clone())
            .is_some());
    }
}
