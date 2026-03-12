use sovereign_core::{Cid, NodeId, PointerAlias, Signature};

#[derive(Debug, Clone)]
pub struct StatePointer {
    pub alias: PointerAlias,
    pub prev_head_cid: Option<Cid>,
    pub head_cid: Cid,
    pub sequence_number: u64,
    pub authority_id: NodeId,
    pub authority_signature: Signature,
}
