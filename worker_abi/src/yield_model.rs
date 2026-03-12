use crate::env::WorkerHostEnv;
use sovereign_core::{Cid, ReceiptCid};

#[derive(Debug, Clone)]
pub enum WorkerError {
    InvalidTask,
    InternalFailure,
    InvalidWitness,
}

#[derive(Debug, Clone)]
pub enum WorkerResult {
    Complete(ReceiptCid),
    Yield(WorkerYield),
    Fail(WorkerError),
}

#[derive(Debug, Clone)]
pub struct WorkerYield {
    pub missing_cids: Vec<Cid>,
    pub continuation_cid: Cid,
}

pub trait WorkerAbi {
    fn execute(&mut self, task_cid: &Cid, env: &mut dyn WorkerHostEnv) -> WorkerResult;
    fn resume(&mut self, continuation_cid: &Cid, env: &mut dyn WorkerHostEnv) -> WorkerResult;
}
