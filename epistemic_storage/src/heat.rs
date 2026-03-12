use sovereign_core::Cid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EpistemicHeat {
    Absent,
    Cold,
    Warm,
    Hot,
}

#[derive(Debug, Clone)]
pub enum PageFault {
    AtomNotHot { cid: Cid, current: EpistemicHeat },
    NetworkRequired { cid: Cid },
    BudgetExhausted { required: u64, available: u64 },
    CorruptedCid { expected: Cid, actual: Cid },
}

#[derive(Debug, Clone, Default)]
pub struct ThermalMetrics {
    pub hot_atoms: u32,
    pub warm_atoms: u32,
    pub cold_atoms: u32,
}
