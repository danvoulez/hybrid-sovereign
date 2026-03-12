use epistemic_storage::StatePointer;
use proof_federation::{
    AcceptanceReceipt, AcceptanceVerdict, BasicPointerValidator, FederationView,
    PointerAnnouncement, PointerClass, PointerFork, PointerPolicy, PointerValidator,
};
use sovereign_core::{Cid, Hash, NodeId, PointerAlias, Signature};

use crate::app_state::{AppState, CaseStatus};

pub struct FederationService;

impl FederationService {
    fn default_federation_view(proof_pack_cid: &sovereign_core::ProofPackCid) -> FederationView {
        FederationView {
            recognized_nodes: vec![],
            accepted_contract_hashes: vec![Hash::from("contract:document-intake:v7")],
            accepted_proof_packs: vec![proof_pack_cid.clone()],
            pointer_policies: vec![PointerPolicy {
                alias_prefix: "cases:".to_string(),
                class: PointerClass::SharedCase,
                accepted_authorities: vec![NodeId::from("node-a"), NodeId::from("node-c")],
                requires_quorum: false,
                quorum_size: 0,
                allow_forks: false,
                require_proof_pack: true,
            }],
            acceptance_receipts: vec![],
        }
    }

    pub fn announce_and_validate(
        state: &mut AppState,
        case_id: &str,
    ) -> Result<AcceptanceVerdict, String> {
        let proof = state
            .proofs
            .get(case_id)
            .ok_or_else(|| format!("missing proof for {case_id}"))?;

        let previous = StatePointer {
            alias: PointerAlias::new(format!("cases:{case_id}:latest")),
            prev_head_cid: None,
            head_cid: Cid::from("cid:head:bootstrap"),
            sequence_number: 1,
            authority_id: NodeId::from("node-a"),
            authority_signature: Signature::from("sig:node-a"),
        };

        let candidate = StatePointer {
            alias: PointerAlias::new(format!("cases:{case_id}:latest")),
            prev_head_cid: Some(previous.head_cid.clone()),
            head_cid: Cid::new(proof.proof_pack_cid.as_str()),
            sequence_number: 2,
            authority_id: NodeId::from("node-a"),
            authority_signature: Signature::from("sig:node-a:next"),
        };

        let federation = Self::default_federation_view(&proof.proof_pack_cid);

        let validator = BasicPointerValidator;
        let verdict = validator.validate_pointer(
            &candidate,
            Some(&previous),
            Some(&proof.proof_pack_cid),
            &federation,
        );

        let announcement = PointerAnnouncement {
            pointer: candidate.clone(),
            proof_pack_cid: proof.proof_pack_cid.clone(),
            contract_hash: proof.contract_hash.clone(),
            announcer_node_id: NodeId::from("node-a"),
            announcement_signature: Signature::from("sig:announcement:node-a"),
        };
        state.pointer_announcements.push(announcement.clone());

        let acceptance_receipt = AcceptanceReceipt {
            pointer_alias: candidate.alias.clone(),
            head_cid: candidate.head_cid.clone(),
            verifier_node_id: NodeId::from("node-b"),
            verdict: verdict.clone(),
            verifier_signature: Signature::from("sig:acceptance:node-b"),
        };
        state.acceptance_receipts.push(acceptance_receipt.clone());

        state.federation_log.push(format!(
            "node-a announced case={case_id} head={} proof_pack={}",
            announcement.pointer.head_cid.as_str(),
            announcement.proof_pack_cid.as_str()
        ));
        state.federation_log.push(format!(
            "node-b validated case={case_id} head={} verdict={:?}",
            acceptance_receipt.head_cid.as_str(),
            acceptance_receipt.verdict
        ));
        state
            .case_timeline
            .entry(case_id.to_string())
            .or_default()
            .push(format!(
                "federation action=announce node=node-a head={} proof_pack={}",
                announcement.pointer.head_cid.as_str(),
                announcement.proof_pack_cid.as_str()
            ));
        state
            .case_timeline
            .entry(case_id.to_string())
            .or_default()
            .push(format!(
                "federation action=validate-pointer node=node-b head={} verdict={verdict:?}",
                candidate.head_cid.as_str()
            ));
        if verdict == AcceptanceVerdict::Accepted {
            state
                .statuses
                .insert(case_id.to_string(), CaseStatus::Federated);
        }
        Ok(verdict)
    }

    pub fn simulate_fork(state: &mut AppState, case_id: &str) -> Result<AcceptanceVerdict, String> {
        let proof = state
            .proofs
            .get(case_id)
            .ok_or_else(|| format!("missing proof for {case_id}"))?;

        let accepted_head = Cid::new(proof.proof_pack_cid.as_str());
        let previous = StatePointer {
            alias: PointerAlias::new(format!("cases:{case_id}:latest")),
            prev_head_cid: Some(Cid::from("cid:head:bootstrap")),
            head_cid: accepted_head.clone(),
            sequence_number: 2,
            authority_id: NodeId::from("node-a"),
            authority_signature: Signature::from("sig:node-a:accepted"),
        };

        let competing_head = Cid::new(format!("cid:fork:{}", case_id));
        let competing = StatePointer {
            alias: PointerAlias::new(format!("cases:{case_id}:latest")),
            prev_head_cid: Some(Cid::from("cid:head:bootstrap")),
            head_cid: competing_head.clone(),
            sequence_number: 2,
            authority_id: NodeId::from("node-c"),
            authority_signature: Signature::from("sig:node-c:competing"),
        };

        let federation = Self::default_federation_view(&proof.proof_pack_cid);
        let validator = BasicPointerValidator;
        let verdict = validator.validate_pointer(
            &competing,
            Some(&previous),
            Some(&proof.proof_pack_cid),
            &federation,
        );

        let competing_announcement = PointerAnnouncement {
            pointer: competing.clone(),
            proof_pack_cid: proof.proof_pack_cid.clone(),
            contract_hash: proof.contract_hash.clone(),
            announcer_node_id: NodeId::from("node-c"),
            announcement_signature: Signature::from("sig:announcement:node-c"),
        };
        state
            .pointer_announcements
            .push(competing_announcement.clone());

        let receipt = AcceptanceReceipt {
            pointer_alias: competing.alias.clone(),
            head_cid: competing.head_cid.clone(),
            verifier_node_id: NodeId::from("node-b"),
            verdict: verdict.clone(),
            verifier_signature: Signature::from("sig:acceptance:node-b:fork"),
        };
        state.acceptance_receipts.push(receipt.clone());

        if verdict == AcceptanceVerdict::ForkDetected {
            let fork = PointerFork {
                alias: competing.alias.clone(),
                base_head_cid: previous.prev_head_cid.clone(),
                competing_heads: vec![previous.head_cid.clone(), competing.head_cid.clone()],
                detected_by: NodeId::from("node-b"),
            };
            state.fork_registry.push(fork.clone());
            state
                .statuses
                .insert(case_id.to_string(), CaseStatus::Forked);
            state.federation_log.push(format!(
                "node-b detected fork case={case_id} alias={} heads={},{}",
                fork.alias.as_str(),
                fork.competing_heads[0].as_str(),
                fork.competing_heads[1].as_str()
            ));
            state
                .case_timeline
                .entry(case_id.to_string())
                .or_default()
                .push(format!(
                    "federation action=fork-detected alias={} heads={},{}",
                    fork.alias.as_str(),
                    fork.competing_heads[0].as_str(),
                    fork.competing_heads[1].as_str()
                ));
        } else {
            state.federation_log.push(format!(
                "node-b fork-simulation case={case_id} verdict={verdict:?}"
            ));
        }

        Ok(verdict)
    }
}

#[cfg(test)]
mod tests {
    use proof_federation::AcceptanceVerdict;

    use super::FederationService;
    use crate::app_state::AppState;
    use crate::case_service::CaseService;
    use crate::demo_domain::entry_task_cid;
    use crate::witness_service::WitnessService;

    #[test]
    fn dual_node_records_are_emitted_for_announcement_and_acceptance() {
        let mut state = AppState::new_seeded();
        let case_id = "case-doc-001";

        CaseService::enqueue_document_event(&mut state, case_id, entry_task_cid()).unwrap();
        CaseService::run_case_once(&mut state, case_id).unwrap();
        WitnessService::resolve_binary_confirm(&mut state, case_id, true).unwrap();
        WitnessService::resolve_field_fill(&mut state, case_id, "DOC-99821").unwrap();
        WitnessService::resolve_approval(&mut state, case_id, true).unwrap();

        let verdict = FederationService::announce_and_validate(&mut state, case_id).unwrap();
        assert_eq!(verdict, AcceptanceVerdict::Accepted);
        assert_eq!(state.pointer_announcements.len(), 1);
        assert_eq!(state.acceptance_receipts.len(), 1);
        assert_eq!(
            state.pointer_announcements[0].announcer_node_id.as_str(),
            "node-a"
        );
        assert_eq!(
            state.acceptance_receipts[0].verifier_node_id.as_str(),
            "node-b"
        );
    }

    #[test]
    fn fork_registry_is_populated_on_competing_head() {
        let mut state = AppState::new_seeded();
        let case_id = "case-doc-001";

        CaseService::enqueue_document_event(&mut state, case_id, entry_task_cid()).unwrap();
        CaseService::run_case_once(&mut state, case_id).unwrap();
        WitnessService::resolve_binary_confirm(&mut state, case_id, true).unwrap();
        WitnessService::resolve_field_fill(&mut state, case_id, "DOC-99821").unwrap();
        WitnessService::resolve_approval(&mut state, case_id, true).unwrap();
        FederationService::announce_and_validate(&mut state, case_id).unwrap();

        let fork_verdict = FederationService::simulate_fork(&mut state, case_id).unwrap();
        assert_eq!(fork_verdict, AcceptanceVerdict::ForkDetected);
        assert_eq!(state.fork_registry.len(), 1);
        assert_eq!(
            state.fork_registry[0].alias.as_str(),
            "cases:case-doc-001:latest"
        );
    }
}
