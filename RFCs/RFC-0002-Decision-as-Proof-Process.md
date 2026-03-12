# RFC-0002 — Decision as Proof Process

---

## 0. Tese

O sistema não "roda uma inferência e depois tenta justificá-la".

O sistema executa um **protocolo de prova incremental**. Cada passo gera um recibo verificável. A decisão final é apenas o terminal de uma cadeia de transições replayáveis.

Logo:

- o runtime não é autoridade
- o transcript é a autoridade
- o contrato governa os próximos passos
- o output vale apenas se vier acompanhado de prova suficiente

---

## 1. Objetivo

Definir um runtime onde:

- cada caso é uma sessão limpa
- não existe estado residual entre casos
- cada passo consome orçamento explícito
- o contrato é executável e versionado
- a prova final pode ser verificada por terceiros sem confiar no executor original

---

## 2. Mudança semântica principal

Na v1, o sistema pensava em: Commit, Ghost, Reject.

Na v2 radical:

```rust
pub enum StepDecision {
    Commit,
    Continue(StepAction),
    Reject(RejectReason),
}
```

- **Commit** continua sendo terminal
- **Reject** continua sendo terminal
- o intermediário não é necessariamente "dúvida"; é próximo passo do protocolo

O nome Ghost continua útil, mas melhor reservado para o tipo de ação/transação, não para o enum principal de decisão.

---

## 3. Invariantes

### I1 — Sessão limpa

Cada caso inicia com:

- `case_id`
- `contract`
- `initial_budget`
- inputs iniciais
- transcript vazio

Nenhum cache entre casos pode influenciar a validade.

### I2 — Toda progressão é transacional

Toda evolução da sessão acontece por um único `StepAction`, com:

- custo
- receipt
- novo estado
- novo root

### I3 — Sem transcript, não há decisão válida

Toda decisão exige transcript verificável.

### I4 — Sem contrato hashado, não há lei

Toda sessão aponta para um contrato versionado e hashado.

### I5 — Sem state root, histórico é fraco

Cada evento deve registrar o root do estado pós-transição.

### I6 — Orçamento é vinculante

Se o próximo passo excede o orçamento, o sistema não pode prosseguir.

---

## 4. Fronteira do runtime

O runtime deve fazer apenas quatro coisas:

1. executar ação atômica
2. medir custo
3. emitir recibo
4. entregar o novo estado ao contrato

O runtime não define a política. A política mora no contrato.

---

## 5. Fronteira do contrato

O contrato deve fazer apenas duas coisas:

1. validar o transcript e o estado atual
2. decidir o próximo passo ou terminal

Ou seja: **runtime executa**, **contrato governa**. Essa separação é central.

---

## 6. Estado da sessão

```rust
pub struct Session {
    pub case_id: String,
    pub contract_hash: String,
    pub proof_mode: ProofMode,

    pub initial_budget: u64,
    pub budget_remaining: u64,

    pub state_root: String,
    pub atoms: AtomStoreView,

    pub transcript_head: Option<String>,
    pub event_count: u64,

    pub current_proposal: Option<ProposalRef>,
}
```

**Observações:**

- Session não precisa carregar tudo materializado
- `atoms` pode ser só visão parcial com refs
- `state_root` é o resumo canônico do estado atual
- `current_proposal` pode existir ou não

---

## 7. Tipos de ação

```rust
pub enum StepAction {
    Compute(ComputeAction),
    Materialize(MaterializeAction),
    Witness(WitnessAction),
}
```

### 7.1 ComputeAction

```rust
pub enum ComputeAction {
    Propose {
        proposer_id: String,
        input_set_cid: String,
    },
    RunExpert {
        expert_id: String,
        input_set_cid: String,
    },
    RecomputePath {
        derivation_cid: String,
    },
}
```

### 7.2 MaterializeAction

```rust
pub enum MaterializeAction {
    RehydrateAtom { cid: String },
    RetrieveEvidence { query_cid: String, top_k: u8 },
    LoadModule { module_id: String },
}
```

### 7.3 WitnessAction

```rust
pub enum WitnessAction {
    AskUserBit { question_id: String, left: String, right: String },
    AskUserField { field_id: String },
    GetTime { oracle_id: String },
    FetchExternalAtom { locator: String, expected_cid: Option<String> },
}
```

Nem todo passo é "inferência"; alguns são testemunhos.

---

## 8. Custo e orçamento

```rust
pub struct ActionCost {
    pub gas: u64,
}
```

O custo pode ser definido pelo contrato ou calculado pelo runtime segundo tabela declarada pelo contrato.

```rust
pub trait Contract {
    fn eval_step(&self, session: &SessionView) -> StepDecision;
    fn cost_of(&self, action: &StepAction, session: &SessionView) -> ActionCost;
    fn determinism_profile(&self) -> DeterminismProfile;
}
```

**Regra:** O custo é debitado antes da execução. Se não houver saldo suficiente → `Reject(OutOfBudget)`.

---

## 9. Perfil de determinismo

```rust
pub struct DeterminismProfile {
    pub fixed_point_only: bool,
    pub allow_user_input: bool,
    pub allow_time_oracle: bool,
    pub allow_external_fetch: bool,
    pub wasm_abi_version: u32,
}
```

**Leis:**

- nada de relógio implícito
- nada de RNG implícito
- nada de rede implícita
- toda fonte externa vira `WitnessAction`
- toda computação relevante deve ser replayável sob o mesmo perfil

---

## 10. Transcript encadeado

```rust
pub struct StepEvent {
    pub prev_event_cid: Option<String>,
    pub action_cid: String,
    pub receipt_cid: String,

    pub budget_before: u64,
    pub budget_after: u64,

    pub state_root_before: String,
    pub state_root_after: String,
}
```

Cada evento precisa permitir verificar: qual ação foi pedida, qual recibo foi produzido, quanto orçamento foi consumido, qual root saiu da transição.

---

## 11. Recibos

```rust
pub enum StepReceipt {
    ProposalCreated { proposal_cid: String, proposer_hash: String },
    ExpertOutput { output_set_cid: String, expert_hash: String },
    AtomRehydrated { atom_cid: String, bytes: u64 },
    EvidenceRetrieved { result_set_cid: String },
    UserBitWitnessed { question_id: String, answer: bool },
    UserFieldWitnessed { field_id: String, value_cid: String },
    TimeWitnessed { oracle_id: String, timestamp_ms: u64 },
    ExternalAtomFetched { atom_cid: String },
}
```

O recibo é a unidade mínima de prova operacional.

---

## 12. Proposal como artefato

```rust
pub struct FrugalProposal {
    pub hypothesis_cid: String,
    pub score_q16: u32,
    pub risk_q16: u32,
    pub required_atoms: Vec<String>,
    pub required_modules: Vec<String>,
    pub producer_hash: String,
}

pub struct ProposalRef {
    pub proposal_cid: String,
}
```

A proposta entra no transcript como artefato verificável, não como sombra interna.

---

## 13. Proof modes

```rust
pub enum ProofMode {
    FullSelfContained,
    AnchoredImmutableRefs,
}
```

### 13.1 FullSelfContained

O ProofPack carrega: contrato ou bytecode, transcript completo, witness payloads, atoms necessários, módulos relevantes ou hashes + blobs necessários.

**Objetivo:** replay offline completo.

### 13.2 AnchoredImmutableRefs

O ProofPack carrega: transcript, CIDs, hashes, refs imutáveis, witnesses mínimos.

**Objetivo:** prova leve, auditável contra storage imutável externo.

---

## 14. ProofPack

```rust
pub struct ProofPack {
    pub case_id: String,
    pub proof_mode: ProofMode,

    pub contract_hash: String,
    pub initial_budget: u64,

    pub transcript_head: Option<String>,
    pub event_count: u64,

    pub final_state_root: String,
    pub final_outcome: FinalOutcome,
}

pub enum FinalOutcome {
    Commit { output_cid: String },
    Reject { reason: RejectReason },
}
```

O ProofPack deve permitir que um terceiro: reconstitua a sessão inicial, percorra o transcript, reexecute as transições, confirme o outcome final.

---

## 15. Rejeição tipada

```rust
pub enum RejectReason {
    OutOfBudget,
    MissingMinimumEvidence,
    UnanchoredEvidence,
    ZeroGuessViolation,
    DeterminismViolation,
    ContractViolation,
    InvalidWitness,
    InvalidTranscript,
    InternalExecutionFailure,
}
```

---

## 16. Loop normativo

```rust
pub fn run(mut session: Session, contract: &dyn Contract, rt: &mut dyn RuntimeOps)
    -> Result<ProofPack, RejectReason>
{
    loop {
        let view = SessionView::from(&session);
        match contract.eval_step(&view) {
            StepDecision::Commit => { /* build proof pack, return */ }
            StepDecision::Reject(reason) => { /* build proof pack, return */ }
            StepDecision::Continue(action) => {
                let cost = contract.cost_of(&action, &view);
                if session.budget_remaining < cost.gas { /* Reject OutOfBudget */ }
                // debit, execute, apply receipt, append event
            }
        }
    }
}
```

---

## 17. RuntimeOps

```rust
pub trait RuntimeOps {
    fn execute(&mut self, session: &Session, action: &StepAction) -> anyhow::Result<StepReceipt>;
}
```

Só isso. Nada de política. Nada de decidir. Nada de "talvez".

---

## 18. Verificador universal

```rust
pub trait UniversalVerifier {
    fn verify(&self, pack: &ProofPack) -> anyhow::Result<()>;
}
```

Ele precisa: resolver o contrato, reconstruir a sessão, percorrer o transcript, reexecutar cada ação/receipt, comparar budget/state_root/receipt_cid, validar o final_outcome.

---

## 19. Sessão sem memória residual

O runtime não deve depender de:

- cache global
- warm state entre casos
- embeddings persistidos invisíveis
- experts carregados de sessão anterior

Qualquer caso deve poder ser reexecutado a partir de seu ProofPack e suas referências imutáveis, sem qualquer estado residual do runtime.

---

## 20. Contrato como programa

**Requisitos:** hashável, versionado, sandboxável, deterministicamente reexecutável, incapaz de I/O implícito, incapaz de mutar estado fora da sessão.

**Recomendação:** WASM é um bom target. Contrato é programa governante, não aplicação arbitrária.

---

## 21. Risco principal e contenção

O maior risco é virar uma VM genérica demais.

**Contenção:**

- Runtime: execute, measure, receipt, append transcript
- Contrato: eval_step, cost_of, determinism_profile
- Módulos externos: proposer, expert, oracle adapter — deliberadamente hashados

---

## 22. Consequência filosófica

> A computação relevante não é o forward;  
> a computação relevante é a cadeia de transições que constrói um certificado verificável.

---

## 23. Shape de crate v3

```
proof_runtime/
  src/
    lib.rs
    session.rs
    contract.rs
    decision.rs
    action.rs
    receipt.rs
    event.rs
    proof.rs
    reject.rs
    verifier.rs
    runtime.rs
    cid.rs
    state_root.rs
    determinism.rs
```

---

## 24. Mantra da RFC-0002

```
decidir = provar
progredir = transacionar
executar = emitir recibo
validade = transcript + contrato + replay
```

Versão curta (repo-core):

```
no hidden state
no silent authority
only steps
only receipts
only proof
```

---

## 25. Veredito final

Esta versão revisada fica mais sólida porque:

- separa terminal de continuação
- explicita witnesses/oracles
- endurece determinismo
- encadeia transcript por hash
- formaliza modos de prova
- reduz o runtime ao mínimo constitucional

Resultado: isso já não é só uma engine frugal. É uma máquina de decisão orientada a prova, com cara própria.
