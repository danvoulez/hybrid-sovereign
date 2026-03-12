use sovereign_core::{CaseId, Cid};

use crate::app_state::AppState;
use crate::demo_domain::execute_document_case;
use epistemic_storage::EpistemicHeat;

#[derive(Debug, Clone)]
pub struct ReplayFieldDiff {
    pub field: String,
    pub original: String,
    pub replayed: String,
}

#[derive(Debug, Clone)]
pub struct ReplayReport {
    pub case_id: CaseId,
    pub matches: bool,
    pub diffs: Vec<ReplayFieldDiff>,
}

#[derive(Debug, Clone)]
pub struct WipeReport {
    pub case_id: CaseId,
    pub wiped_hot_atoms: Vec<Cid>,
}

pub struct ReplayService;

impl ReplayService {
    pub fn wipe_hot_state(state: &mut AppState, case_id: &str) -> Result<WipeReport, String> {
        let wiped_hot_atoms = state.hot_atoms_by_case.remove(case_id).unwrap_or_default();
        if let Some(heat_map) = state.case_atom_heat.get_mut(case_id) {
            for cid in &wiped_hot_atoms {
                if let Some(heat) = heat_map.get_mut(cid) {
                    *heat = EpistemicHeat::Cold;
                }
            }
        }
        let atom_list = if wiped_hot_atoms.is_empty() {
            "none".to_string()
        } else {
            wiped_hot_atoms
                .iter()
                .map(|c| c.as_str())
                .collect::<Vec<_>>()
                .join(",")
        };

        state
            .replay_log
            .push(format!("case={case_id} wipe_hot_atoms={atom_list}"));
        state
            .case_timeline
            .entry(case_id.to_string())
            .or_default()
            .push(format!("replay action=wipe-hot-state atoms=[{atom_list}]"));

        Ok(WipeReport {
            case_id: CaseId::from(case_id),
            wiped_hot_atoms,
        })
    }

    pub fn replay_case_with_diff(
        state: &mut AppState,
        case_id: &str,
    ) -> Result<ReplayReport, String> {
        let original = state
            .proofs
            .get(case_id)
            .ok_or_else(|| format!("missing original proof for {case_id}"))?
            .clone();
        let replay = execute_document_case(&CaseId::from(case_id))?;

        let mut diffs = Vec::new();
        let original_contract_hash = original.contract_hash.as_str().to_string();
        let replay_contract_hash = replay.proof.contract_hash.as_str().to_string();
        if original_contract_hash != replay_contract_hash {
            diffs.push(ReplayFieldDiff {
                field: "contract_hash".to_string(),
                original: original_contract_hash,
                replayed: replay_contract_hash,
            });
        }

        let original_proof_pack = original.proof_pack_cid.as_str().to_string();
        let replay_proof_pack = replay.proof.proof_pack_cid.as_str().to_string();
        if original_proof_pack != replay_proof_pack {
            diffs.push(ReplayFieldDiff {
                field: "proof_pack_cid".to_string(),
                original: original_proof_pack,
                replayed: replay_proof_pack,
            });
        }

        let original_receipt = original
            .final_receipt_cid
            .as_ref()
            .map(|v| v.as_str().to_string())
            .unwrap_or_else(|| "-".to_string());
        let replay_receipt = replay
            .proof
            .final_receipt_cid
            .as_ref()
            .map(|v| v.as_str().to_string())
            .unwrap_or_else(|| "-".to_string());
        if original_receipt != replay_receipt {
            diffs.push(ReplayFieldDiff {
                field: "final_receipt_cid".to_string(),
                original: original_receipt,
                replayed: replay_receipt,
            });
        }

        let original_outcome = format!("{:?}", original.final_outcome);
        let replay_outcome = format!("{:?}", replay.proof.final_outcome);
        if original_outcome != replay_outcome {
            diffs.push(ReplayFieldDiff {
                field: "final_outcome".to_string(),
                original: original_outcome,
                replayed: replay_outcome,
            });
        }

        let original_event_count = original.event_count.to_string();
        let replay_event_count = replay.proof.event_count.to_string();
        if original_event_count != replay_event_count {
            diffs.push(ReplayFieldDiff {
                field: "event_count".to_string(),
                original: original_event_count,
                replayed: replay_event_count,
            });
        }

        let original_transcript_head = original.transcript_head.as_str().to_string();
        let replay_transcript_head = replay.proof.transcript_head.as_str().to_string();
        if original_transcript_head != replay_transcript_head {
            diffs.push(ReplayFieldDiff {
                field: "transcript_head".to_string(),
                original: original_transcript_head,
                replayed: replay_transcript_head,
            });
        }

        let matches = diffs.is_empty();
        state
            .replay_log
            .push(format!("case={case_id} replay_match={matches}"));
        if !matches {
            for diff in &diffs {
                state.replay_log.push(format!(
                    "case={case_id} diff field={} original={} replayed={}",
                    diff.field, diff.original, diff.replayed
                ));
            }
        }

        Ok(ReplayReport {
            case_id: CaseId::from(case_id),
            matches,
            diffs,
        })
    }

    pub fn replay_case_from_ashes(state: &mut AppState, case_id: &str) -> Result<bool, String> {
        Ok(Self::replay_case_with_diff(state, case_id)?.matches)
    }
}

#[cfg(test)]
mod tests {
    use sovereign_core::{CaseId, Cid, Hash};

    use super::ReplayService;
    use crate::app_state::AppState;
    use crate::demo_domain::execute_document_case;

    #[test]
    fn replay_report_contains_field_diffs_when_original_is_tampered() {
        let mut state = AppState::new_seeded();
        let case_id = "case-doc-001";
        let original = execute_document_case(&CaseId::from(case_id)).unwrap().proof;
        state.proofs.insert(case_id.to_string(), original.clone());

        let mut tampered = original;
        tampered.contract_hash = Hash::from("contract:tampered");
        state.proofs.insert(case_id.to_string(), tampered);

        let report = ReplayService::replay_case_with_diff(&mut state, case_id).unwrap();
        assert!(!report.matches);
        assert!(!report.diffs.is_empty());
    }

    #[test]
    fn wipe_hot_state_clears_case_hot_atoms() {
        let mut state = AppState::new_seeded();
        let case_id = "case-doc-001";
        state.hot_atoms_by_case.insert(
            case_id.to_string(),
            vec![Cid::from("cid:atom:1"), Cid::from("cid:atom:2")],
        );

        let report = ReplayService::wipe_hot_state(&mut state, case_id).unwrap();
        assert_eq!(report.case_id.as_str(), case_id);
        assert_eq!(report.wiped_hot_atoms.len(), 2);
        assert!(state.hot_atoms_by_case.get(case_id).is_none());
    }
}
