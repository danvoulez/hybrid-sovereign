use epistemic_storage::StatePointer;
use sovereign_core::ProofPackCid;

use crate::acceptance::{AcceptRejectReason, AcceptanceVerdict};
use crate::policy::FederationView;

pub trait PointerValidator {
    fn validate_pointer(
        &self,
        new_pointer: &StatePointer,
        previous: Option<&StatePointer>,
        proof_pack_cid: Option<&ProofPackCid>,
        fed: &FederationView,
    ) -> AcceptanceVerdict;
}

#[derive(Debug, Default)]
pub struct BasicPointerValidator;

impl PointerValidator for BasicPointerValidator {
    fn validate_pointer(
        &self,
        new_pointer: &StatePointer,
        previous: Option<&StatePointer>,
        proof_pack_cid: Option<&ProofPackCid>,
        fed: &FederationView,
    ) -> AcceptanceVerdict {
        let policy = fed
            .pointer_policies
            .iter()
            .find(|p| new_pointer.alias.as_str().starts_with(&p.alias_prefix));
        let Some(policy) = policy else {
            return AcceptanceVerdict::Rejected(AcceptRejectReason::PolicyViolation);
        };

        if !policy
            .accepted_authorities
            .iter()
            .any(|a| a == &new_pointer.authority_id)
        {
            return AcceptanceVerdict::Rejected(AcceptRejectReason::AuthorityViolation);
        }

        if policy.require_proof_pack {
            let Some(proof_cid) = proof_pack_cid else {
                return AcceptanceVerdict::Rejected(AcceptRejectReason::InvalidProof);
            };
            if !fed.accepted_proof_packs.iter().any(|p| p == proof_cid) {
                return AcceptanceVerdict::Rejected(AcceptRejectReason::InvalidProof);
            }
        }

        if let Some(previous) = previous {
            if new_pointer.sequence_number < previous.sequence_number {
                return AcceptanceVerdict::Rejected(AcceptRejectReason::RewindAttempt);
            }
            if new_pointer.sequence_number == previous.sequence_number
                && new_pointer.head_cid != previous.head_cid
            {
                return AcceptanceVerdict::ForkDetected;
            }
            if !policy.allow_forks && new_pointer.prev_head_cid.as_ref() != Some(&previous.head_cid)
            {
                return AcceptanceVerdict::Rejected(AcceptRejectReason::SequenceGap);
            }
            if policy.allow_forks && new_pointer.prev_head_cid.as_ref() != Some(&previous.head_cid)
            {
                return AcceptanceVerdict::ForkDetected;
            }
        }

        AcceptanceVerdict::Accepted
    }
}
