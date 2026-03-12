use std::collections::HashMap;

use sovereign_core::{Cid, PointerAlias};

use crate::case::ManagedCase;
use crate::input::ManagerInput;
use crate::output::ManagerOutput;

pub trait Worker {
    fn id(&self) -> &str;
    fn capabilities(&self) -> Vec<String>;
    fn execute(&self, task_cid: &Cid) -> Result<Cid, String>;
}

pub trait ManagerPlane {
    fn ingest(&mut self, input: ManagerInput) -> Result<(), String>;
    fn evaluate_next(&mut self, case_id: &str) -> Result<ManagerOutput, String>;
}

#[derive(Debug, Default)]
pub struct DemoManagerPlane {
    pub cases: HashMap<String, ManagedCase>,
    pub inbox: Vec<ManagerInput>,
}

impl ManagerPlane for DemoManagerPlane {
    fn ingest(&mut self, input: ManagerInput) -> Result<(), String> {
        self.inbox.push(input);
        Ok(())
    }

    fn evaluate_next(&mut self, case_id: &str) -> Result<ManagerOutput, String> {
        let Some(case) = self.cases.get_mut(case_id) else {
            return Err("unknown case".to_string());
        };
        if case.current_head_cid.is_some() {
            return Ok(ManagerOutput::NoOp);
        }
        if let Some(last) = self.inbox.last() {
            return Ok(match last {
                ManagerInput::Event(task_cid) => ManagerOutput::Delegate {
                    worker_cid: Cid::from("worker:document-intake:v7"),
                    task_cid: task_cid.clone(),
                },
                ManagerInput::WorkerCompleted {
                    receipt_cid: _,
                    proof_pack_cid,
                } => {
                    case.latest_proof_pack_cid = Some(proof_pack_cid.clone());
                    ManagerOutput::AdvancePointer {
                        alias: PointerAlias::new(format!("cases:{case_id}:latest")),
                        head_cid: Cid::new(proof_pack_cid.as_str()),
                        proof_pack_cid: proof_pack_cid.clone(),
                    }
                }
                _ => ManagerOutput::NoOp,
            });
        }
        Ok(ManagerOutput::NoOp)
    }
}
