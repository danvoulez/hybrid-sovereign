use sovereign_core::{Hash, NodeId};

#[derive(Debug, Clone)]
pub struct NodeIdentity {
    pub node_id: NodeId,
    pub public_key: Hash,
    pub roles: Vec<NodeRole>,
}

#[derive(Debug, Clone)]
pub enum NodeRole {
    EdgeExecutor,
    PointerAuthority,
    WitnessAuthority,
    ContractPublisher,
    Mirror,
}
