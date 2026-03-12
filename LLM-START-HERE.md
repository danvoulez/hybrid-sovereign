# LLM Start Here

**For AI assistants and code navigators.** Quick context to work effectively in this codebase.

---

## What This Is

A **proof-carrying runtime** for edge AI and local-first systems. Every decision is replayable. Zero-trust by default. Six RFCs define the architecture; Rust crates implement it.

---

## Architecture in 30 Seconds

1. **Gate** (frugal_decision): Commit / Ghost / Reject. ε is contract. No guessing.
2. **Runtime** (proof_runtime): Session → StepAction → Receipt → transcript. `run()` loop.
3. **Storage** (epistemic_storage): Content-addressed atoms. Heat: Absent → Cold → Warm → Hot.
4. **Federation** (proof_federation): Signed pointers, no mutable state sync. Forks explicit.
5. **Manager** (manager_plane): Event-driven, typed I/O. Chat is exception.
6. **Workers** (worker_abi): Sandboxed, yield on cold. ChipAsCode (WASM) or SiliconAsCompute (ε-bounding).

---

## Key Files

| Want to… | Look at |
|----------|---------|
| Understand the spec | `RFCs/00-INDEX.md` → RFC-0001 through RFC-0006 |
| See the flow | `sovereign_demo/src/main.rs`, `sovereign_demo/tests/one_sovereign_case.rs` |
| Gate logic | `frugal_decision/src/gate.rs` |
| Runtime loop | `proof_runtime/src/runtime.rs` |
| Worker yield | `proof_runtime/src/runtime.rs` (heat_up on yield), `worker_abi/src/yield_model.rs` |
| Atom space | `epistemic_storage/src/space.rs`, `epistemic_storage/src/heat.rs` |

---

## Canonical Test

```bash
cargo test -p sovereign_demo --test one_sovereign_case
```

Proves: worker yields on cold → runtime heats up → resume → Commit → manager advances pointer → federation accepts.

---

## Conventions

- **Traits over structs:** Contract, RuntimeOps, WorkerAbi, AtomSpace, PointerValidator.
- **CIDs everywhere:** Content-addressed. No semantic IDs in core.
- **Receipts, not logs:** Every step produces a verifiable receipt.
- **No chat in control flow:** Manager uses events and typed outputs.

---

## Vocabulary

- **ε (epsilon):** Contracted error margin. Silicon can approximate within ε.
- **Ghost:** Verdict = "need one question before deciding."
- **Epistemic Yield:** Worker pauses when data is cold; returns `Yield(missing_cids)`.
- **ProofPack:** Final certificate of a case. Verifiable by third parties.
- **StatePointer:** Mutable pointer to immutable head. Only "mutation" allowed.

---

## Don’t

- Add syscalls or arbitrary I/O to workers.
- Use mutable state sync in federation.
- Make the manager a chatbot for control flow.
- Ignore the RFCs when changing core behavior.
