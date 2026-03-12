#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    Commit,
    Ghost,
    Reject,
}
