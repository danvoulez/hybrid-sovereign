# v7 Kanban

## P0
- [x] V7-E1-I1 Create `apps/sovereign_manager_app` shell.
- [x] V7-E1-I3 Implement case queue surface.
- [x] V7-E2-I1 Define canonical managed-case projection.
- [x] V7-E3-I1 Create `CaseService` bridge (`manager_plane -> proof_runtime`).
- [x] V7-E4-I1 Define document-intake task atom schema.
- [x] V7-E4-I2 Implement document-intake worker.
- [x] V7-E5-I2 Surface `WorkerYielded` and `continuation_cid` in timeline.
- [x] V7-E6-I1 Create pending witness projection.

## P1
- [x] V7-E4-I3 Implement document-extract worker.
- [x] V7-E4-I4 Implement document-validate worker.
- [x] V7-E4-I5 Implement decision-pack worker.
- [x] V7-E4-I7 Add worker operational ledger (stage-specific insight per worker).
- [x] V7-E6-I2 Implement A/B witness handler (CLI typed flow).
- [x] V7-E6-I3 Implement field-fill witness handler (CLI typed flow).
- [x] V7-E6-I4 Implement approve/reject witness handler (CLI typed flow).
- [x] V7-E7-I1 Render hash-chained transcript.
- [x] V7-E7-I3 Implement replay action (`replay <case-id>` with field-level diff).
- [x] V7-E8-I1 Build node A/B federation panel.

## P2
- [x] V7-E5-I4 Add debug heat-up controls (`heat`, `heat-up`, `cool-down`).
- [x] V7-E8-I4 Add fork registry view (`fork <case-id>` + federation panel section).
- [x] V7-E9-I1 Add wipe-hot-state action (`wipe <case-id>`).
- [x] V7-E9-I3 Add post-wipe replay flow (auto replay report after wipe).

## Sprint 1 (current)
- [x] V7-E1-I1 Create app shell.
- [x] V7-E1-I3 Implement queue output.
- [x] V7-E2-I1 Define managed-case projection.
- [x] V7-E3-I1 Create case execution bridge.
- [x] V7-E4-I1 Define document task atom baseline.
- [x] V7-E4-I2 Implement first domain worker (`document-extract`, yield/resume).
- [x] Wire witness inbox flow and federated acceptance in CLI demo.
- [x] Add replay-from-ashes check in CLI demo.
