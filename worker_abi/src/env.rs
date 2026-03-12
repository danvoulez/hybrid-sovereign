use epistemic_storage::PageFault;

#[derive(Debug, Clone)]
pub struct OutOfGas {
    pub required: u64,
    pub available: u64,
}

pub trait WorkerHostEnv {
    fn request_atom(&mut self, cid: &sovereign_core::Cid) -> Result<Vec<u8>, PageFault>;
    fn consume_gas(&mut self, amount: u64) -> Result<(), OutOfGas>;
}
