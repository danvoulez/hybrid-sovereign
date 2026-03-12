# v7 — Sovereign Manager Vertical

## Mandate

Build the first machine someone can operate.
No chat-first control. No hidden state. Visible delegation, visible proof, visible replay.

## Epics

### V7-E1 App Shell & Case Queue
- Deliverables: app shell, navigation surfaces, seed fixtures, queue view.
- Acceptance: app boots locally; queue shows status, budget, head, blocked reason.

### V7-E2 Managed Case Model
- Deliverables: canonical case states, case projection, timeline summary.
- Acceptance: derived status from real receipts/events and blocked reason rendering.

### V7-E3 Runtime-backed Case Execution
- Deliverables: app -> manager -> runtime wiring, real `RunWorker`, receipt projection.
- Acceptance: `Delegate -> Yield -> Complete` visible without fake flow.

### V7-E4 Document Intake & Approval Domain
- Deliverables: intake/extract/validate/decision workers, domain fixtures, operational insight per worker stage.
- Acceptance: one document case runs end-to-end, including at least one yield and one witness.

### V7-E5 Visible Yield & Heat Model
- Deliverables: heat panel (`Absent/Cold/Warm/Hot`), continuation visibility.
- Acceptance: operator sees why execution paused and resumed.

### V7-E6 Witness Inbox
- Deliverables: typed witness widgets (A/B, field fill, approve/reject).
- Acceptance: witness response unblocks case and lands in transcript.

### V7-E7 Replay Inspector
- Deliverables: hash-chained timeline, proof pack view, replay action and diff.
- Acceptance: replay in fresh storage reproduces outcome deterministically.

### V7-E8 Federation Panel
- Deliverables: A/B node view, announcements, acceptance receipts, fork status.
- Acceptance: case committed in node A appears accepted in node B.

### V7-E9 Replay From Ashes
- Deliverables: wipe-hot-state action, rehydrate/replay.
- Acceptance: case remains verifiable after hot-state wipe.
