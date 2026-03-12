use crate::contract::{BudgetContract, ErrorContractQ16, ProposalEnvelope};
use crate::verdict::Verdict;
use sovereign_core::ReasonCode;

#[derive(Debug, Clone)]
pub struct GateInputs<'a> {
    pub domain: &'a str,
    pub has_intent: bool,
    pub has_minimum_evidence: bool,
    pub evidence_anchored: bool,
    pub deterministic_proof: bool,
    pub questions_used: u8,
    pub ghosts_used_in_epoch: u32,
    pub escalations_used: u8,
    pub rehydrations_used: u32,
    pub live_ram_mb: u32,
    pub live_vram_mb: u32,
    pub loaded_params_mb: u32,
    pub live_context_tokens: u32,
    pub hot_atoms: u32,
    pub err: &'a ErrorContractQ16,
    pub budget: &'a BudgetContract,
}

#[derive(Debug, Clone)]
pub struct GateDecision {
    pub verdict: Verdict,
    pub reason_code: ReasonCode,
    pub next_action: Option<String>,
}

pub fn gate_run(inp: GateInputs<'_>, prop: &ProposalEnvelope) -> GateDecision {
    if !inp.has_intent || !inp.has_minimum_evidence {
        if inp.questions_used < inp.err.max_questions_per_case
            && inp.ghosts_used_in_epoch < inp.err.max_ghosts_per_epoch
        {
            return GateDecision {
                verdict: Verdict::Ghost,
                reason_code: ReasonCode::MISSING_EVIDENCE,
                next_action: Some(format!("retrieve:{}", prop.hypothesis_cid.as_str())),
            };
        }
        return GateDecision {
            verdict: Verdict::Reject,
            reason_code: ReasonCode::MISSING_EVIDENCE,
            next_action: None,
        };
    }

    if !inp.evidence_anchored {
        return GateDecision {
            verdict: Verdict::Reject,
            reason_code: ReasonCode::UNANCHORED,
            next_action: None,
        };
    }

    if inp.err.zero_guess_domains.iter().any(|d| d == inp.domain) && !inp.deterministic_proof {
        return GateDecision {
            verdict: Verdict::Reject,
            reason_code: ReasonCode::ZERO_GUESS,
            next_action: None,
        };
    }

    let ram_after = inp.live_ram_mb.saturating_add(prop.estimated_ram_mb);
    let vram_after = inp.live_vram_mb.saturating_add(prop.estimated_vram_mb);
    let params_after = inp
        .loaded_params_mb
        .saturating_add(prop.estimated_params_mb);

    if ram_after > inp.budget.max_ram_mb
        || vram_after > inp.budget.max_vram_mb
        || params_after > inp.budget.max_loaded_params_mb
        || inp.live_context_tokens > inp.budget.max_live_context_tokens
        || inp.hot_atoms > inp.budget.max_hot_atoms
    {
        return GateDecision {
            verdict: Verdict::Reject,
            reason_code: ReasonCode::RESOURCE_VIOLATION,
            next_action: None,
        };
    }

    if prop.score_q16 >= inp.err.ok_min_q16 && prop.risk_q16 <= inp.err.max_risk_q16 {
        return GateDecision {
            verdict: Verdict::Commit,
            reason_code: ReasonCode::NONE,
            next_action: None,
        };
    }

    if prop.score_q16 <= inp.err.reject_max_q16 {
        return GateDecision {
            verdict: Verdict::Reject,
            reason_code: ReasonCode::SILICON_NOT_OK,
            next_action: None,
        };
    }

    GateDecision {
        verdict: Verdict::Ghost,
        reason_code: ReasonCode::SILICON_DOUBT,
        next_action: Some("ask_human_witness".to_string()),
    }
}
