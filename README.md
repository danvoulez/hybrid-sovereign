# Hybrid Sovereign Workspace v6

**A proof-carrying runtime for edge AI and local-first systems.** Every decision is replayable. Nothing is trusted by default.

**Repository:** [github.com/danvoulez/hybrid-sovereign](https://github.com/danvoulez/hybrid-sovereign)

---

## The problem

You want to run AI, RAG, or business logic on a tablet, a phone, or a Raspberry Pi. Offline. Without a central server. But:

- **AI is stochastic** — same input, different output. How do you audit it?
- **Hardware varies** — GPU on laptop ≠ GPU on phone. Same model, different floats.
- **State is messy** — caches, embeddings, "context" scattered everywhere. Replay is impossible.
- **Chat is a trap** — if the manager is a chatbot, decisions hide in conversation. No proof.

This workspace is an architecture that says: *silicon can be approximate; the world cannot be approximate without a contract.*

---

## The idea in one paragraph

Every computation ends in one of three states: **Commit** (accepted), **Ghost** (needs one question), or **Reject** (denied). Error margins (ε) are declared upfront. Workers are sandboxed functions that *yield* when they need data instead of blocking. Storage is content-addressed (CIDs) with a heat model (Absent → Cold → Warm → Hot). The manager is not a chat; it's an event-driven control plane. Federation syncs pointers over proofs, not mutable state. Result: a system where you can re-run any case years later and get the same cryptographic proof.

---

## The six layers (metaphor)

Think of it as building a small sovereign:

| Layer | RFC | Role | In plain terms |
|-------|-----|------|----------------|
| **1. Law** | RFC-0001 | Constitution | Three verdicts only. ε is contract. No guessing without evidence. |
| **2. Tribunal** | RFC-0002 | Court | Every step emits a receipt. Transcript is chain of hashes. ProofPack = certificate. |
| **3. Physics** | RFC-0003 | Storage | Data lives by CID. Disk is a cache. "Forgetting" (eviction) is survival. |
| **4. Society** | RFC-0004 | Federation | Nodes exchange signed pointers, not state. Forks are explicit. No silent overwrite. |
| **5. Government** | RFC-0005 | Manager | Event loop. Typed inputs/outputs. Chat is exception, not protocol. |
| **6. Workers** | RFC-0006 | Labor | Sandboxed, no syscalls. Yield on cold memory. Silicon verified by ε-bounding. |

---

## How it flows

```
User/Event → Manager (Delegate) → Runtime (execute) → Worker
                                        ↓
                              Worker needs CID_2 (cold)
                                        ↓
                              Yield(missing_cids) → Runtime heats up → Resume
                                        ↓
                              Complete(receipt) → Manager (AdvancePointer) → Federation
```

The manager never "chats" to decide. It receives typed events, delegates to workers, and advances pointers when proof is sufficient.

---

## Key concepts (cheat sheet)

| Term | Meaning |
|------|---------|
| **Gate** | The constitutional decision function. Inputs + proposal → Commit / Ghost / Reject. |
| **ε (epsilon)** | Contracted error margin. Silicon can approximate within ε; outside = reject. |
| **CID** | Content ID. Hash of data. The only way to address knowledge. |
| **ProofPack** | Final certificate of a case: transcript head, receipts, outcome. Verifiable by third parties. |
| **Epistemic Heat** | Absent (know hash, no bytes) → Cold (on disk) → Warm (summary in RAM) → Hot (full payload in RAM). |
| **Epistemic Yield** | Worker pauses and returns "I need CID_X" instead of blocking. Host fetches, resumes. |
| **StatePointer** | Mutable pointer to an immutable head. The only "mutation" allowed. |
| **No-Guess** | If evidence is missing, you get Ghost (one question) or Reject. Never fake-complete. |

---

## Getting started

```bash
# Run the canonical demo (worker yield → heat up → resume → proof → federation)
cargo run -p sovereign_demo

# Run the integration test
cargo test -p sovereign_demo --test one_sovereign_case
```

**v6.1 mandate:** Prove that one sovereign case survives yield, proof, and federated acceptance without hidden state. No new abstractions until that holds.

---

## Crates

| Crate | Purpose |
|-------|---------|
| `sovereign_core` | CIDs, hashes, signatures, reason codes |
| `frugal_decision` | Gate, Verdict, ErrorContract, BudgetContract |
| `proof_runtime` | Session, StepAction, ProofPack, `run()` loop |
| `epistemic_storage` | AtomSpace, EpistemicHeat, StatePointer |
| `proof_federation` | PointerAnnouncement, AcceptanceVerdict, Fork |
| `manager_plane` | ManagerInput, ManagerOutput, ManagedCase |
| `worker_abi` | WorkerHostEnv, WorkerAbi, Yield, SiliconReceipt |
| `sovereign_demo` | End-to-end demo binary |
| `apps/sovereign_manager_app` | CLI shell: queue, witness, replay, federation |

---

## RFCs

Full spec: `RFCs/00-INDEX.md`. Read in order 0001 → 0006.

---

## Vocabulário (curiosidade)

Termos que podem soar estranhos ou carregar sentido específico neste contexto:

| Termo | Origem / sentido |
|-------|------------------|
| **Epistemic** | Do grego *episteme* (conhecimento). Aqui: "relativo ao que sabemos". Epistemic Heat = temperatura do *conhecimento* (não da CPU). Epistemic Yield = pausar por *falta de dado*, não por I/O bloqueante. |
| **Ghost** | Um dos três veredictos. Não é "erro" nem "sucesso" — é "preciso de uma pergunta objetiva antes de decidir". Evita adivinhar. |
| **ε (epsilon)** | Margem de erro contratada. O silício pode aproximar; o contrato diz até onde. Fora de ε = Reject. |
| **Content-addressed** | Dado identificado pelo *hash* do seu conteúdo, não por nome ou ID. CID = Content ID. Mesmo conteúdo → mesmo CID, em qualquer máquina. |
| **Proof-carrying** | O estado "carrega" a prova de como chegou ali. Não é "confie em mim"; é "aqui está o certificado, verifique". |
| **Reidratar** | Trazer dado de Cold/Absent para Hot (RAM). O inverso de "serializar e esquecer". |
| **Eviction** | Ejetar dado da RAM/VRAM para liberar espaço. Não é bug; é sobrevivência. O sistema *esquece* de propósito. |
| **Bounding** | Para silício (GPU/NPU): não comparar hashes exatos; comparar se a distância entre vetores ≤ ε. Aceita variação fisiológica do hardware. |
| **ChipAsCode** | Worker determinístico bit-a-bit (ex: WASM). O código é a lei. |
| **SiliconAsCompute** | Worker estatístico (GPU, NPU). Aproxima; validado por bounding. |
| **Gossip** | Sincronização por "fofoca": "meu pointer está no CID X". Quem não tem o CID pede. Sem merge de estado mutável. |
| **Witness** | Testemunha externa injetada no task (hora, cotação, clique do usuário). O Worker não busca; o Manager injeta. Garante replay idêntico. |
| **Gas** | Unidade de custo por execução. Worker consome gas; orçamento limita. Evita loops infinitos e abuso. |
| **PageFault** | Falha ao acessar dado: está Cold ou Absent. No nosso caso, vira Yield, não crash. |
| **No-Guess** | Regra constitucional: sem evidência mínima, não inventar. Ghost (pergunta) ou Reject. Nunca "completude falsa". |

---

## Status

Lightweight, trait-first. Clean starting point, not production. v7 planning: `docs/v7/KANBAN.md`.
