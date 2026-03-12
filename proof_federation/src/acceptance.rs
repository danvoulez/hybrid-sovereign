use sovereign_core::{Cid, NodeId, PointerAlias, Signature};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AcceptRejectReason {
    MissingDependencies,
    InvalidSignature,
    InvalidProof,
    InvalidContract,
    AuthorityViolation,
    RewindAttempt,
    SequenceGap,
    PolicyViolation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AcceptanceVerdict {
    Accepted,
    Rejected(AcceptRejectReason),
    ForkDetected,
    Deferred,
}

#[derive(Debug, Clone)]
pub struct AcceptanceReceipt {
    pub pointer_alias: PointerAlias,
    pub head_cid: Cid,
    pub verifier_node_id: NodeId,
    pub verdict: AcceptanceVerdict,
    pub verifier_signature: Signature,
}
