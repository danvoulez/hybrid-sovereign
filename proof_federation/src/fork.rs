use sovereign_core::{Cid, PointerAlias};

#[derive(Debug, Clone)]
pub struct PointerFork {
    pub alias: PointerAlias,
    pub base_head_cid: Option<Cid>,
    pub competing_heads: Vec<Cid>,
    pub detected_by: sovereign_core::NodeId,
}
