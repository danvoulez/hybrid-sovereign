use sovereign_core::BudgetAmount;

#[derive(Debug, Clone)]
pub struct BudgetState {
    pub gas_remaining: BudgetAmount,
    pub max_parallel_workers: u32,
    pub max_open_cases: u32,
    pub max_human_interrupts: u32,
}
