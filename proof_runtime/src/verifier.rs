use crate::proof::ProofPack;

#[derive(Debug)]
pub enum VerificationError {
    ContractNotFound,
    TranscriptMismatch {
        step: u64,
        detail: String,
    },
    BudgetMismatch {
        step: u64,
        expected: u64,
        actual: u64,
    },
    StateRootMismatch {
        step: u64,
        expected: String,
        actual: String,
    },
    OutcomeMismatch,
    ReceiptMismatch {
        step: u64,
    },
}

pub trait UniversalVerifier {
    fn verify(&self, pack: &ProofPack) -> Result<(), VerificationError>;
}
