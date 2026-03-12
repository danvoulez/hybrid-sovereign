use crate::action::{StepAction, StepDecision};
use crate::session::SessionView;

#[derive(Debug, Clone)]
pub enum ExecutionTarget {
    Wasm { abi_version: u32 },
    Native,
}

#[derive(Debug, Clone)]
pub struct DeterminismProfile {
    pub fixed_point_only: bool,
    pub allow_user_input: bool,
    pub allow_time_oracle: bool,
    pub allow_external_fetch: bool,
    pub execution_target: ExecutionTarget,
}

pub trait Contract {
    fn eval_step(&self, session: &SessionView) -> StepDecision;
    fn cost_of(&self, action: &StepAction, session: &SessionView) -> u64;
    fn determinism_profile(&self) -> DeterminismProfile;
}
