use std::collections::HashMap;

use epistemic_storage::EpistemicHeat;
use sovereign_core::Cid;

use crate::app_state::AppState;

pub struct HeatService;

impl HeatService {
    fn ensure_case_heat_map<'a>(
        state: &'a mut AppState,
        case_id: &str,
    ) -> &'a mut HashMap<Cid, EpistemicHeat> {
        state
            .case_atom_heat
            .entry(case_id.to_string())
            .or_insert_with(HashMap::new)
    }

    fn rebuild_hot_projection(state: &mut AppState, case_id: &str) {
        let hot = state
            .case_atom_heat
            .get(case_id)
            .map(|heat_map| {
                heat_map
                    .iter()
                    .filter_map(|(cid, heat)| {
                        if *heat == EpistemicHeat::Hot {
                            Some(cid.clone())
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        state.hot_atoms_by_case.insert(case_id.to_string(), hot);
    }

    pub fn heat_up(state: &mut AppState, case_id: &str, cid: Cid) -> Result<(), String> {
        let heat_map = Self::ensure_case_heat_map(state, case_id);
        heat_map.insert(cid.clone(), EpistemicHeat::Hot);
        Self::rebuild_hot_projection(state, case_id);
        state
            .case_timeline
            .entry(case_id.to_string())
            .or_default()
            .push(format!(
                "heat action=heat-up cid={} target=Hot",
                cid.as_str()
            ));
        state
            .replay_log
            .push(format!("case={case_id} heat_up={}", cid.as_str()));
        Ok(())
    }

    pub fn cool_down(state: &mut AppState, case_id: &str, cid: &Cid) -> Result<(), String> {
        let heat_map = Self::ensure_case_heat_map(state, case_id);
        if !heat_map.contains_key(cid) {
            return Err(format!(
                "cid {} is not tracked for case {case_id}",
                cid.as_str()
            ));
        }
        heat_map.insert(cid.clone(), EpistemicHeat::Cold);
        Self::rebuild_hot_projection(state, case_id);
        state
            .case_timeline
            .entry(case_id.to_string())
            .or_default()
            .push(format!(
                "heat action=cool-down cid={} target=Cold",
                cid.as_str()
            ));
        state
            .replay_log
            .push(format!("case={case_id} cool_down={}", cid.as_str()));
        Ok(())
    }

    pub fn snapshot(state: &AppState, case_id: &str) -> Vec<(Cid, EpistemicHeat)> {
        let mut out = state
            .case_atom_heat
            .get(case_id)
            .map(|m| {
                m.iter()
                    .map(|(cid, heat)| (cid.clone(), *heat))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        out.sort_by(|a, b| a.0.as_str().cmp(b.0.as_str()));
        out
    }
}

#[cfg(test)]
mod tests {
    use sovereign_core::Cid;

    use super::HeatService;
    use crate::app_state::AppState;
    use crate::case_service::CaseService;
    use crate::demo_domain::entry_task_cid;

    #[test]
    fn heat_debug_controls_update_projection_and_snapshot() {
        let mut state = AppState::new_seeded();
        let case_id = "case-doc-001";

        CaseService::enqueue_document_event(&mut state, case_id, entry_task_cid()).unwrap();
        CaseService::run_case_once(&mut state, case_id).unwrap();

        let atom = Cid::from("cid:doc:intake:payload");
        HeatService::cool_down(&mut state, case_id, &atom).unwrap();
        assert!(state
            .hot_atoms_by_case
            .get(case_id)
            .map(|v| v.iter().all(|cid| cid != &atom))
            .unwrap_or(true));

        HeatService::heat_up(&mut state, case_id, atom.clone()).unwrap();
        let snapshot = HeatService::snapshot(&state, case_id);
        assert!(snapshot
            .iter()
            .any(|(cid, heat)| cid == &atom && *heat == epistemic_storage::EpistemicHeat::Hot));
    }
}
