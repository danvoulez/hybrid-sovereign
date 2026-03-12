use crate::acceptance::AcceptanceReceipt;
use crate::node::NodeIdentity;
use sovereign_core::{Hash, NodeId, ProofPackCid};

#[derive(Debug, Clone)]
pub enum PointerClass {
    Personal,
    SharedCase,
    ContractHead,
    WitnessLog,
    MirrorIndex,
}

#[derive(Debug, Clone)]
pub struct PointerPolicy {
    pub alias_prefix: String,
    pub class: PointerClass,
    pub accepted_authorities: Vec<NodeId>,
    pub requires_quorum: bool,
    pub quorum_size: u32,
    pub allow_forks: bool,
    pub require_proof_pack: bool,
}

#[derive(Debug, Clone, Default)]
pub struct FederationView {
    pub recognized_nodes: Vec<NodeIdentity>,
    pub accepted_contract_hashes: Vec<Hash>,
    pub accepted_proof_packs: Vec<ProofPackCid>,
    pub pointer_policies: Vec<PointerPolicy>,
    pub acceptance_receipts: Vec<AcceptanceReceipt>,
}
