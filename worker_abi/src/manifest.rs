use sovereign_core::Cid;

#[derive(Debug, Clone)]
pub struct WorkerManifest {
    pub name: String,
    pub version: String,
    pub class: WorkerClass,
    pub bytecode_cid: Cid,
    pub required_capabilities: Vec<Capability>,
    pub determinism_profile: DeterminismProfile,
}

#[derive(Debug, Clone)]
pub enum WorkerClass {
    ChipAsCode,
    SiliconAsCompute {
        epsilon_bounds: f32,
        quantization: QuantizationLevel,
    },
}

#[derive(Debug, Clone)]
pub enum QuantizationLevel {
    Q16,
    Fp16,
    Int8,
}

#[derive(Debug, Clone)]
pub enum Capability {
    RequestAtom,
    ConsumeGas,
    YieldOnColdMemory,
}

#[derive(Debug, Clone)]
pub struct DeterminismProfile {
    pub no_syscalls: bool,
    pub no_hidden_entropy: bool,
    pub yield_on_page_fault: bool,
}
