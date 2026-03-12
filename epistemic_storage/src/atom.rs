use sovereign_core::{Cid, Hash, Signature};

#[derive(Debug, Clone)]
pub struct UniversalAtom {
    pub header: AtomHeader,
    pub links: Vec<Cid>,
    pub body: AtomBody,
}

#[derive(Debug, Clone)]
pub struct AtomHeader {
    pub kind: AtomKind,
    pub size_bytes: u64,
    pub producer_hash: Hash,
    pub signature: Option<Signature>,
}

#[derive(Debug, Clone)]
pub enum AtomBody {
    Inline(Vec<u8>),
    Chunked {
        root_cid: Cid,
        codec: String,
        total_size_bytes: u64,
    },
}

#[derive(Debug, Clone)]
pub enum AtomKind {
    Weights,
    WasmContract,
    PromptText,
    ProofPack,
    StateRoot,
    WitnessData,
    WorkerManifest,
    Task,
    Receipt,
}
