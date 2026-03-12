use epistemic_storage::{AtomBody, AtomSpace, EpistemicHeat, PageFault};
use sovereign_core::Cid;
use worker_abi::{OutOfGas, WorkerHostEnv};

pub struct AtomSpaceHostEnv<'a> {
    pub atom_space: &'a mut dyn AtomSpace,
    pub gas_remaining: &'a mut u64,
}

impl WorkerHostEnv for AtomSpaceHostEnv<'_> {
    fn request_atom(&mut self, cid: &Cid) -> Result<Vec<u8>, PageFault> {
        let current = self.atom_space.current_heat(cid);
        if current != EpistemicHeat::Hot {
            return Err(PageFault::AtomNotHot {
                cid: cid.clone(),
                current,
            });
        }

        let Some(atom) = self.atom_space.get_atom(cid) else {
            return Err(PageFault::NetworkRequired { cid: cid.clone() });
        };

        match &atom.body {
            AtomBody::Inline(bytes) => Ok(bytes.clone()),
            AtomBody::Chunked { root_cid, .. } => Ok(root_cid.as_str().as_bytes().to_vec()),
        }
    }

    fn consume_gas(&mut self, amount: u64) -> Result<(), OutOfGas> {
        if *self.gas_remaining < amount {
            return Err(OutOfGas {
                required: amount,
                available: *self.gas_remaining,
            });
        }
        *self.gas_remaining -= amount;
        Ok(())
    }
}
