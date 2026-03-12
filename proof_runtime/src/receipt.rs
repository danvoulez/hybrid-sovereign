use sovereign_core::{canonical_join, Cid, ReceiptCid};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StepReceipt {
    ProposalCreated {
        proposal_cid: Cid,
        producer_hash: sovereign_core::Hash,
    },
    AtomMaterialized {
        atom_cid: Cid,
    },
    HumanWitnessed {
        witness_kind: String,
        answer_cid: Cid,
    },
    WorkerYielded {
        worker_cid: Cid,
        task_cid: Cid,
        missing_cids: Vec<Cid>,
        continuation_cid: Cid,
    },
    WorkerCompleted {
        worker_cid: Cid,
        task_cid: Cid,
        receipt_cid: ReceiptCid,
    },
}

impl StepReceipt {
    pub fn canonical(&self) -> String {
        match self {
            Self::ProposalCreated {
                proposal_cid,
                producer_hash,
            } => canonical_join(&["proposal", proposal_cid.as_str(), producer_hash.as_str()]),
            Self::AtomMaterialized { atom_cid } => {
                canonical_join(&["materialize", atom_cid.as_str()])
            }
            Self::HumanWitnessed {
                witness_kind,
                answer_cid,
            } => canonical_join(&["witness", witness_kind, answer_cid.as_str()]),
            Self::WorkerYielded {
                worker_cid,
                task_cid,
                missing_cids,
                continuation_cid,
            } => {
                let missing = missing_cids
                    .iter()
                    .map(|c| c.as_str())
                    .collect::<Vec<_>>()
                    .join(",");
                canonical_join(&[
                    "worker_yielded",
                    worker_cid.as_str(),
                    task_cid.as_str(),
                    &missing,
                    continuation_cid.as_str(),
                ])
            }
            Self::WorkerCompleted {
                worker_cid,
                task_cid,
                receipt_cid,
            } => canonical_join(&[
                "worker_completed",
                worker_cid.as_str(),
                task_cid.as_str(),
                receipt_cid.as_str(),
            ]),
        }
    }
}
