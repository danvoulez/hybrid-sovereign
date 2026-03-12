mod app_state;
mod case_service;
mod demo_domain;
mod federation_service;
mod heat_service;
mod replay_service;
mod witness_service;

use app_state::{AppState, WitnessKind};
use case_service::CaseService;
use demo_domain::entry_task_cid;
use federation_service::FederationService;
use heat_service::HeatService;
use replay_service::{ReplayReport, ReplayService, WipeReport};
use sovereign_core::Cid;
use witness_service::WitnessService;

fn print_case_queue(state: &AppState) {
    println!("=== Case Queue ===");
    for item in state.queue() {
        println!(
            "case={} status={:?} budget={} head={} blocked={} last_receipt={}",
            item.case_id,
            item.status,
            item.budget_remaining.0,
            item.head.as_ref().map(|c| c.as_str()).unwrap_or("-"),
            item.blocked_reason.as_deref().unwrap_or("-"),
            item.last_receipt
                .as_ref()
                .map(|c| c.as_str())
                .unwrap_or("-")
        );
    }
}

fn print_witness_inbox(state: &AppState) {
    println!("=== Witness Inbox ===");
    if state.witness_inbox.is_empty() {
        println!("empty");
        return;
    }
    for w in &state.witness_inbox {
        let details = match &w.kind {
            WitnessKind::BinaryConfirm {
                question,
                positive_label,
                negative_label,
            } => format!("question='{question}' options={positive_label}/{negative_label}"),
            WitnessKind::FieldFill { field_name, prompt } => {
                format!("field={field_name} prompt='{prompt}'")
            }
            WitnessKind::ApproveReject { reason } => format!("reason='{reason}'"),
        };
        println!(
            "case={} witness_id={} kind={} prompt={} details={}",
            w.case_id,
            w.witness_id,
            w.kind_label(),
            w.prompt_cid,
            details
        );
    }
}

fn print_replay_panel(state: &AppState) {
    println!("=== Replay Inspector ===");
    if state.replay_log.is_empty() {
        println!("no replay yet");
        return;
    }
    for line in &state.replay_log {
        println!("{line}");
    }
}

fn print_federation_panel(state: &AppState) {
    println!("=== Federation Panel ===");
    if state.pointer_announcements.is_empty()
        && state.acceptance_receipts.is_empty()
        && state.fork_registry.is_empty()
    {
        println!("no federation events yet");
        return;
    }
    println!("node-a (announcer):");
    for ann in &state.pointer_announcements {
        println!(
            "case_alias={} announcer={} head={} proof_pack={}",
            ann.pointer.alias.as_str(),
            ann.announcer_node_id.as_str(),
            ann.pointer.head_cid.as_str(),
            ann.proof_pack_cid.as_str()
        );
    }
    println!("node-b (verifier):");
    for rec in &state.acceptance_receipts {
        println!(
            "alias={} verifier={} head={} verdict={:?}",
            rec.pointer_alias.as_str(),
            rec.verifier_node_id.as_str(),
            rec.head_cid.as_str(),
            rec.verdict
        );
    }
    println!("fork registry:");
    if state.fork_registry.is_empty() {
        println!("(empty)");
    } else {
        for fork in &state.fork_registry {
            let heads = fork
                .competing_heads
                .iter()
                .map(|h| h.as_str())
                .collect::<Vec<_>>()
                .join(",");
            println!(
                "alias={} detected_by={} heads=[{}]",
                fork.alias.as_str(),
                fork.detected_by.as_str(),
                heads
            );
        }
    }
    println!("raw federation log:");
    for line in &state.federation_log {
        println!("  {line}");
    }
}

fn print_case_detail(state: &AppState, case_id: &str) {
    println!("=== Case Detail ({case_id}) ===");
    match state.case_summary(case_id) {
        Some(summary) => {
            println!(
                "status={:?} budget={} head={} last_receipt={}",
                summary.status,
                summary.budget_remaining.0,
                summary.head.as_ref().map(|h| h.as_str()).unwrap_or("-"),
                summary
                    .last_receipt
                    .as_ref()
                    .map(|r| r.as_str())
                    .unwrap_or("-")
            );
        }
        None => {
            println!("unknown case");
            return;
        }
    }
    println!("timeline:");
    if let Some(lines) = state.case_timeline.get(case_id) {
        for line in lines {
            println!("{line}");
        }
    } else {
        println!("(empty)");
    }
    println!("worker_ledger:");
    if let Some(entries) = state.worker_ledger_by_case.get(case_id) {
        for entry in entries {
            let artifacts = entry
                .artifact_cids
                .iter()
                .map(|c| c.as_str())
                .collect::<Vec<_>>()
                .join(",");
            println!(
                "  stage={} action={} worker={} task={} artifacts=[{}] insight='{}'",
                entry.stage,
                entry.action,
                entry.worker_cid.as_str(),
                entry.task_cid.as_str(),
                artifacts,
                entry.insight
            );
        }
    } else {
        println!("(empty)");
    }
    let hot_atoms = state
        .hot_atoms_by_case
        .get(case_id)
        .map(|atoms| {
            atoms
                .iter()
                .map(|c| c.as_str())
                .collect::<Vec<_>>()
                .join(",")
        })
        .unwrap_or_else(|| "none".to_string());
    println!("hot_atoms={hot_atoms}");
    let heat_snapshot = HeatService::snapshot(state, case_id);
    if heat_snapshot.is_empty() {
        println!("heat_snapshot=(empty)");
    } else {
        println!("heat_snapshot:");
        for (cid, heat) in heat_snapshot {
            println!("  cid={} heat={heat:?}", cid.as_str());
        }
    }
}

fn run_demo() -> Result<(), String> {
    let mut state = AppState::new_seeded();
    let case_id = "case-doc-001";

    print_case_queue(&state);

    CaseService::enqueue_document_event(&mut state, case_id, entry_task_cid())?;
    let report = CaseService::run_case_once(&mut state, case_id)?;
    println!(
        "run report: case={} worker={} task={} had_yield={} witness_required={} proof_pack={}",
        report.case_id,
        report.delegated_worker,
        report.delegated_task,
        report.had_yield,
        report.witness_required,
        report.proof_pack_cid,
    );

    print_case_queue(&state);
    print_case_detail(&state, case_id);
    print_witness_inbox(&state);

    if report.witness_required {
        let confirm = WitnessService::resolve_binary_confirm(&mut state, case_id, true)?;
        println!("witness outcome: {confirm}");
        print_witness_inbox(&state);
        let fill = WitnessService::resolve_field_fill(&mut state, case_id, "DOC-99821")?;
        println!("witness outcome: {fill}");
        print_witness_inbox(&state);
        let approval = WitnessService::resolve_approval(&mut state, case_id, true)?;
        println!("witness outcome: {approval}");
        print_witness_inbox(&state);
    }

    let federation_verdict = FederationService::announce_and_validate(&mut state, case_id)?;
    println!("federation verdict: {federation_verdict:?}");

    let wipe = ReplayService::wipe_hot_state(&mut state, case_id)?;
    let wiped_atoms = if wipe.wiped_hot_atoms.is_empty() {
        "none".to_string()
    } else {
        wipe.wiped_hot_atoms
            .iter()
            .map(|c| c.as_str())
            .collect::<Vec<_>>()
            .join(",")
    };
    println!(
        "wipe hot state: case={} wiped_hot_atoms={wiped_atoms}",
        wipe.case_id
    );

    let replay_ok = ReplayService::replay_case_from_ashes(&mut state, case_id)?;
    println!("replay from ashes: {replay_ok}");

    print_case_queue(&state);
    print_case_detail(&state, case_id);
    print_federation_panel(&state);
    print_replay_panel(&state);

    Ok(())
}

fn bootstrap_case_to_committed_or_federated(
    state: &mut AppState,
    case_id: &str,
) -> Result<(), String> {
    if state.proofs.contains_key(case_id) {
        return Ok(());
    }

    CaseService::enqueue_document_event(state, case_id, entry_task_cid())?;
    let report = CaseService::run_case_once(state, case_id)?;
    if report.witness_required {
        WitnessService::resolve_binary_confirm(state, case_id, true)?;
        WitnessService::resolve_field_fill(state, case_id, "DOC-99821")?;
        WitnessService::resolve_approval(state, case_id, true)?;
    }
    let _ = FederationService::announce_and_validate(state, case_id)?;
    Ok(())
}

fn print_replay_report(report: &ReplayReport) {
    println!("=== Replay Report ===");
    println!("case={} match={}", report.case_id, report.matches);
    if report.diffs.is_empty() {
        println!("diffs=none");
        return;
    }
    for diff in &report.diffs {
        println!(
            "field={} original={} replayed={}",
            diff.field, diff.original, diff.replayed
        );
    }
}

fn print_wipe_report(report: &WipeReport) {
    println!("=== Wipe Report ===");
    let atoms = if report.wiped_hot_atoms.is_empty() {
        "none".to_string()
    } else {
        report
            .wiped_hot_atoms
            .iter()
            .map(|c| c.as_str())
            .collect::<Vec<_>>()
            .join(",")
    };
    println!("case={} wiped_hot_atoms={atoms}", report.case_id);
}

fn run_replay_command(case_id: &str) -> Result<(), String> {
    let mut state = AppState::new_seeded();
    bootstrap_case_to_committed_or_federated(&mut state, case_id)?;
    let report = ReplayService::replay_case_with_diff(&mut state, case_id)?;
    print_replay_report(&report);
    print_replay_panel(&state);
    Ok(())
}

fn run_wipe_command(case_id: &str) -> Result<(), String> {
    let mut state = AppState::new_seeded();
    bootstrap_case_to_committed_or_federated(&mut state, case_id)?;
    let wipe = ReplayService::wipe_hot_state(&mut state, case_id)?;
    print_wipe_report(&wipe);
    let report = ReplayService::replay_case_with_diff(&mut state, case_id)?;
    print_replay_report(&report);
    print_replay_panel(&state);
    Ok(())
}

fn run_fork_command(case_id: &str) -> Result<(), String> {
    let mut state = AppState::new_seeded();
    bootstrap_case_to_committed_or_federated(&mut state, case_id)?;
    let verdict = FederationService::simulate_fork(&mut state, case_id)?;
    println!("fork simulation verdict: {verdict:?}");
    print_case_queue(&state);
    print_case_detail(&state, case_id);
    print_federation_panel(&state);
    Ok(())
}

fn run_heat_view_command(case_id: &str) -> Result<(), String> {
    let mut state = AppState::new_seeded();
    bootstrap_case_to_committed_or_federated(&mut state, case_id)?;
    print_case_detail(&state, case_id);
    Ok(())
}

fn run_heat_up_command(case_id: &str, cid: &str) -> Result<(), String> {
    let mut state = AppState::new_seeded();
    bootstrap_case_to_committed_or_federated(&mut state, case_id)?;
    HeatService::heat_up(&mut state, case_id, Cid::from(cid))?;
    print_case_detail(&state, case_id);
    print_replay_panel(&state);
    Ok(())
}

fn run_cool_down_command(case_id: &str, cid: &str) -> Result<(), String> {
    let mut state = AppState::new_seeded();
    bootstrap_case_to_committed_or_federated(&mut state, case_id)?;
    HeatService::cool_down(&mut state, case_id, &Cid::from(cid))?;
    print_case_detail(&state, case_id);
    print_replay_panel(&state);
    Ok(())
}

fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    let cmd = args.get(1).map(|s| s.as_str()).unwrap_or("demo");

    let result = match cmd {
        "demo" => run_demo(),
        "queue" => {
            let state = AppState::new_seeded();
            print_case_queue(&state);
            Ok(())
        }
        "replay" => {
            let case_id = args.get(2).map(|s| s.as_str()).unwrap_or("case-doc-001");
            run_replay_command(case_id)
        }
        "wipe" => {
            let case_id = args.get(2).map(|s| s.as_str()).unwrap_or("case-doc-001");
            run_wipe_command(case_id)
        }
        "fork" => {
            let case_id = args.get(2).map(|s| s.as_str()).unwrap_or("case-doc-001");
            run_fork_command(case_id)
        }
        "heat" => {
            let case_id = args.get(2).map(|s| s.as_str()).unwrap_or("case-doc-001");
            run_heat_view_command(case_id)
        }
        "heat-up" => {
            let case_id = args.get(2).map(|s| s.as_str()).unwrap_or("case-doc-001");
            let cid = args
                .get(3)
                .map(|s| s.as_str())
                .unwrap_or("cid:doc:intake:payload");
            run_heat_up_command(case_id, cid)
        }
        "cool-down" => {
            let case_id = args.get(2).map(|s| s.as_str()).unwrap_or("case-doc-001");
            let cid = args
                .get(3)
                .map(|s| s.as_str())
                .unwrap_or("cid:doc:intake:payload");
            run_cool_down_command(case_id, cid)
        }
        _ => Err(format!(
            "unknown command: {cmd}. use: demo | queue | replay <case-id> | wipe <case-id> | fork <case-id> | heat <case-id> | heat-up <case-id> <cid> | cool-down <case-id> <cid>"
        )),
    };

    if let Err(err) = result {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}
