use epistemic_storage::StatePointer;
use sovereign_core::{Cid, Hash, NodeId, ProofPackCid, Signature};

#[derive(Debug, Clone)]
pub struct PointerAnnouncement {
    pub pointer: StatePointer,
    pub proof_pack_cid: ProofPackCid,
    pub contract_hash: Hash,
    pub announcer_node_id: NodeId,
    pub announcement_signature: Signature,
}

#[derive(Debug, Clone)]
pub struct ContractAnnouncement {
    pub contract_hash: Hash,
    pub contract_cid: Cid,
    pub publisher_id: NodeId,
    pub publisher_signature: Signature,
}
