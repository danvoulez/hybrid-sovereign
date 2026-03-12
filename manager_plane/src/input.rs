use epistemic_storage::StatePointer;
use proof_federation::PointerFork;
use sovereign_core::{Cid, ProofPackCid, ReceiptCid};

use crate::budget::BudgetState;

#[derive(Debug, Clone)]
pub enum ManagerInput {
    Event(Cid),
    WorkerCompleted {
        receipt_cid: ReceiptCid,
        proof_pack_cid: ProofPackCid,
    },
    PointerAdvanced(StatePointer),
    BudgetTick(BudgetState),
    Deadline(String),
    Witness(Cid),
    ForkDetected(PointerFork),
    PolicyUpdate(String),
}
