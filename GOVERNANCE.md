# Governance

## Project Status

Lightweight, trait-first. Clean starting point for the architecture defined in RFCs 0001–0006. **Not production-ready.**

## Decision Making

- **RFCs** define the normative spec. Changes to core concepts go through RFC proposal.
- **Code** follows the RFCs. If code and spec diverge, the RFC is the source of truth until updated.
- **v6.1 mandate:** No new abstractions until one sovereign case survives yield, proof, and federated acceptance.

## Contributing

1. Read `README.md` and `RFCs/00-INDEX.md`.
2. Align changes with the relevant RFC (0001–0006).
3. Run `cargo test -p sovereign_demo --test one_sovereign_case` before submitting.
4. Keep the architecture trait-first; avoid unnecessary dependencies.

## Crates and Ownership

| Crate | RFC | Role |
|-------|-----|------|
| sovereign_core | — | Primitives (CIDs, hashes, reason codes) |
| frugal_decision | 0001 | Gate, Verdict, contracts |
| proof_runtime | 0002 | Session, ProofPack, run loop |
| epistemic_storage | 0003 | AtomSpace, Heat, Pointers |
| proof_federation | 0004 | Announcements, acceptance, forks |
| manager_plane | 0005 | Manager inputs/outputs, cases |
| worker_abi | 0006 | Worker ABI, yield, bounding |

## Planning

v7 roadmap: `docs/v7/KANBAN.md`, `docs/v7/EPICS.md`.
