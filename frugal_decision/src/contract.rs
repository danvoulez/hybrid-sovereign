use sovereign_core::{Cid, Hash};

#[derive(Debug, Clone)]
pub struct ErrorContractQ16 {
    pub epsilon_q16: u32,
    pub zero_guess_domains: Vec<String>,
    pub max_questions_per_case: u8,
    pub max_ghosts_per_epoch: u32,
    pub ok_min_q16: u32,
    pub reject_max_q16: u32,
    pub max_risk_q16: u32,
}

#[derive(Debug, Clone)]
pub struct BudgetContract {
    pub max_ram_mb: u32,
    pub max_vram_mb: u32,
    pub max_loaded_params_mb: u32,
    pub max_live_context_tokens: u32,
    pub max_rehydrations: u32,
    pub max_escalations: u8,
    pub max_hot_atoms: u32,
}

#[derive(Debug, Clone)]
pub struct ProposalEnvelope {
    pub hypothesis_cid: Cid,
    pub score_q16: u32,
    pub risk_q16: u32,
    pub required_atoms: Vec<Cid>,
    pub required_workers: Vec<String>,
    pub estimated_ram_mb: u32,
    pub estimated_vram_mb: u32,
    pub estimated_params_mb: u32,
    pub producer_hash: Hash,
}
