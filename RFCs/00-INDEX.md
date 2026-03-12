# Índice das RFCs — Sistema Híbrido Orientado a Prova

---

## Visão geral

Este conjunto de 6 RFCs + índice define a arquitetura de um sistema **local-first**, **zero-trust** e **orientado a prova**, onde:

- O silício pode ser estatístico; o mundo não pode ser estatístico sem contrato.
- Toda decisão é replayável e auditável.
- O estado é imutável; a mutação é apenas avanço de ponteiros.
- O manager não é chat; é control plane orientado a eventos.

---

## Os 6 RFCs

| # | Documento | Tema | Resumo |
|---|-----------|------|--------|
| **1** | [RFC-0001 — Constituição do Híbrido](./RFC-0001-Constituicao-Hibrido.md) | Lei | Lei do Resultado (OK/GHOST/REJECT), erro contratado (ε), no-guess, prova replayável, alocação silício vs gate |
| **2** | [RFC-0002 — Decision as Proof Process](./RFC-0002-Decision-as-Proof-Process.md) | Tribunal | Runtime mínimo, transcript encadeado, ProofPack, sessão limpa, contrato como programa |
| **3** | [RFC-0003 — Sovereign Atom Space](./RFC-0003-Sovereign-Atom-Space.md) | Física | CAS, EpistemicHeat (Absent/Cold/Warm/Hot), AtomSpace, StatePointer, gossip local-first |
| **4** | [RFC-0004 — Federation](./RFC-0004-Federation-under-Proof-Carrying-State.md) | Sociedade | StatePointers assinados, anti-rewind, fork explícito, aceitação local, quorum opcional |
| **5** | [RFC-0005 — Non-Chat Manager Plane](./RFC-0005-Non-Chat-Manager-Plane.md) | Governo | Manager orientado a eventos, inputs/saídas tipados, LLM como componente (não autoridade), chat como exceção |
| **6** | [RFC-0006 — Sovereign Worker ABI](./RFC-0006-Sovereign-Worker-ABI.md) | Operários | ABI Worker, sandbox, Epistemic Yield, ChipAsCode vs SiliconAsCompute, prova de bounding (ε) |

---

## Dependências entre RFCs

```
RFC-0001 (Lei)
    │
    ├──► RFC-0002 (Tribunal)
    │         │
    │         ├──► RFC-0003 (Física)
    │         │
    │         └──► RFC-0004 (Sociedade)
    │
    ├──► RFC-0005 (Governo)
    │
    └──► RFC-0006 (Operários) ◄── Worker ABI, sandbox, yield
```

- **RFC-0001** é a base: define quem decide e como (Gate, ε, no-guess).
- **RFC-0002** define o transcript e o ProofPack; depende da constituição.
- **RFC-0003** define onde os atoms vivem; permite resolver CIDs para o ProofPack.
- **RFC-0004** define como nós federam sem sincronizar estado mutável.
- **RFC-0005** define o manager; opera sobre eventos, receipts e pointers das RFCs anteriores.
- **RFC-0006** define os Workers: ABI, sandbox, Epistemic Yield, prova de bounding para silício.

---

## Mantras por RFC

| RFC | Mantra |
|-----|--------|
| 0001 | `if ok → commit` / `if doubt → ghost` / `if not → reject` — ε é contrato. replay é lei. silício é livre. |
| 0002 | decidir = provar / progredir = transacionar / executar = emitir recibo |
| 0003 | State is a pointer. Knowledge is a graph. Disk is a cache. Forgetfulness is survival. |
| 0004 | state does not merge / heads compete / proof decides / policy accepts |
| 0005 | management is not chat / management is typed continuation / chat is exception |
| 0006 | logic is exact / silicon is bounded / workers cannot speak, they only yield / time is an input, not a state |

---

## Ordem de leitura sugerida

1. **RFC-0001** — Entender a constituição (OK/GHOST/REJECT, ε, no-guess).
2. **RFC-0002** — Entender o processo de prova (Session, StepAction, ProofPack).
3. **RFC-0003** — Entender o armazenamento (AtomSpace, Heat, Pointers).
4. **RFC-0004** — Entender a federação (anúncios, aceitação, forks).
5. **RFC-0005** — Entender o manager (eventos, workers, chat como exceção).
6. **RFC-0006** — Entender os Workers (ABI, sandbox, Epistemic Yield, bounding).

---

## Status

Todos os documentos estão em **draft normativo**. Termos MUST, MUST NOT, SHOULD, SHOULD NOT e MAY têm sentido normativo conforme RFC 2119.
