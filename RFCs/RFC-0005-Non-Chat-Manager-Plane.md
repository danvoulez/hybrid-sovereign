# RFC-0005 — Non-Chat Manager Plane

---

## 0. Status

**Draft normativo.**

Esta RFC define o plano de gerenciamento não conversacional para sistemas orientados a prova, onde o manager não é modelado como chatbot, mas como control plane semântico orientado a eventos, responsável por delegação, orçamento, evidência, escalonamento e avanço de estado.

---

## 1. Tese

O manager **MUST NOT** ser modelado primariamente como interface de chat.

O manager **MUST** ser modelado como uma máquina de transições que:

- observa eventos e receipts
- mantém contexto operacional mínimo
- decide o próximo passo admissível
- delega trabalho a workers especializados
- pede evidência quando necessário
- escalona exceções
- avança pointers quando a prova é suficiente

Chat, quando existir, **MUST** ser tratado como interface periférica de: supervisão, witness humano, inspeção, override explícito.

> Gerenciamento não é conversa; gerenciamento é coordenação sob política, orçamento e prova.

---

## 2. Objetivo

Definir um manager plane em que:

- inputs e outputs são tipados
- o loop principal é orientado a eventos
- o LLM atua como mecanismo de interpretação e priorização, não como autoridade opaca
- humanos entram apenas em exceções ou witness points
- toda transição relevante é replayável
- chat não é a fonte primária de estado nem de decisão

---

## 3. Escopo

**Esta RFC especifica:**

- o papel do manager
- o papel do LLM dentro do manager
- o papel dos workers
- o papel do humano
- o modelo de eventos e comandos
- o loop operacional do manager
- a UI operacional não-chat
- a integração com RFC-0001 a RFC-0004

**Esta RFC não especifica:**

- UX conversacional detalhada
- ranking algorítmico interno do LLM
- implementação de scheduling distribuído
- policy business-specific de cada domínio
- formato visual final de dashboard

---

## 4. Invariantes

### I1 — Chat não é plano primário

O sistema MUST NOT depender de conversa livre para executar o fluxo normal.

### I2 — Toda ação relevante é tipada

O manager MUST NOT emitir comandos operacionais críticos em texto livre como forma canônica.

### I3 — O manager não é o worker

O manager MUST NOT assumir execução detalhada do trabalho quando esta puder ser delegada a worker, expert ou mecanismo especializado.

### I4 — O manager decide continuidade

O papel central do manager MUST ser decidir: continuar, delegar, pedir evidência, escalonar, rejeitar, consolidar.

### I5 — O humano é exceção governada

O humano MUST entrar no plano como witness, árbitro ou aprovador explícito, não como compensação silenciosa para falhas do manager.

### I6 — Toda continuidade relevante deve deixar trilha

Toda decisão do manager que altere o estado do caso MUST produzir artifacts replayáveis via receipts, events ou proofs.

---

## 5. Modelo mental

O manager **não** deve ser entendido como: assistente, chatbot, agente conversacional generalista.

O manager **deve** ser entendido como: despachante, escalonador, supervisor de workflow, intérprete de política operacional, árbitro de orçamento e evidência, controlador semântico de transições.

---

## 6. Fronteiras do manager

**O manager MUST fazer apenas:**

1. interpretar o estado do caso
2. escolher o próximo passo admissível
3. selecionar o worker/expert adequado
4. pedir evidência faltante
5. controlar budget e risco
6. decidir escalonamento
7. consolidar prova suficiente para avanço de pointer ou terminal

**O manager MUST NOT:**

- esconder raciocínio crítico apenas em contexto efêmero
- executar arbitrariamente tarefas detalhadas que pertencem a workers
- conversar por padrão como se conversa fosse o protocolo
- promover output a decisão sem passar pelas leis das RFCs anteriores

---

## 7. Entradas do manager

```rust
pub enum ManagerInput {
    Event(EventCid),
    Receipt(ReceiptCid),
    PointerAdvanced(StatePointer),
    BudgetTick(BudgetState),
    Deadline(DeadlineSignal),
    Witness(WitnessReceipt),
    ForkDetected(PointerFork),
    PolicyUpdate(PolicyCid),
}
```

O manager MUST NOT depender de texto livre como input primário quando um equivalente tipado existir.

---

## 8. Saídas do manager

```rust
pub enum ManagerOutput {
    Delegate { worker_id: String, task_cid: String },
    RequestEvidence { cid: String },
    LoadExpert { expert_id: String, input_set_cid: String },
    Escalate { queue: String, reason_code: u32 },
    AskHumanWitness { witness_kind: String, prompt_cid: String },
    AdvancePointer { alias: String, head_cid: String },
    Reject { reason_code: u32 },
    NoOp,
}
```

Toda saída relevante SHOULD poder ser serializada como atom/receipt e entrar no transcript do caso.

---

## 9. Papel do LLM

O LLM **não** é o manager em si. O LLM é um componente do manager plane.

**O LLM MAY ser usado para:**

- interpretar contexto imperfeito
- resumir estado disperso
- decompor metas em subtarefas
- escolher entre estratégias viáveis
- priorizar evidências
- selecionar workers prováveis
- explicar decisões depois

**O LLM MUST NOT ser tratado como autoridade final sobre:**

- commit
- accept federativo
- validade de proof
- integridade de pointer
- orçamento efetivo
- zero-guess domains

> O LLM propõe estrutura de continuidade; o manager plane governa a legitimidade da continuidade.

---

## 10. Papel dos workers

```rust
pub trait Worker {
    fn id(&self) -> &str;
    fn capabilities(&self) -> Vec<String>;
    fn execute(&self, task_cid: &str) -> anyhow::Result<ReceiptCid>;
}
```

Worker MUST ser tratável como executor de tarefa delimitada. Worker MUST NOT avançar pointer global sem comando ou política explícita. Worker MUST retornar receipt tipado.

---

## 11. Papel do humano

O humano não é o loop principal. O humano é uma autoridade de exceção.

O humano MAY entrar como: witness binário, witness textual/estruturado, aprovador, resolvedor de fork, autorizador de budget extra, supervisor de política.

O humano MUST NOT ser usado como "patch invisível" para toda fragilidade do sistema. Toda intervenção humana relevante MUST gerar witness receipt.

---

## 12. Chat como interface periférica

Chat continua permitido, mas com estatuto restrito.

**Chat MAY ser usado para:** inspeção do caso, explicação do estado atual, resposta a witness requests, override explícito, debugging/postmortem.

**Chat MUST NOT ser:** o mecanismo normal de dispatch, o repositório oculto de estado, o protocolo primário entre manager e worker, o canal canônico de decisão operacional.

> Chat é interface de exceção e observabilidade, não o barramento principal do manager.

---

## 13. Modelo operacional do caso

```rust
pub struct ManagedCase {
    pub case_id: String,
    pub state_root: String,
    pub current_head_cid: Option<String>,
    pub active_budget: BudgetState,
    pub pending_events: Vec<String>,
    pub pending_actions: Vec<String>,
    pub blocked_on: Option<BlockReason>,
}

pub enum BlockReason {
    WaitingForWorker,
    WaitingForEvidence,
    WaitingForHumanWitness,
    WaitingForBudget,
    WaitingForPolicy,
    WaitingForForkResolution,
}
```

A unidade primária da operação MUST ser o caso, não a thread de chat.

---

## 14. Event loop do manager

```rust
pub trait ManagerPlane {
    fn ingest(&mut self, input: ManagerInput) -> anyhow::Result<()>;
    fn evaluate_next(&mut self, case_id: &str) -> anyhow::Result<ManagerOutput>;
}
```

Ao receber novo input, o manager MUST: anexar ao estado do caso, atualizar status, verificar constraints, decidir próximo passo, emitir output tipado ou NoOp.

---

## 15. Planejamento e decomposição

```rust
pub struct PlanStep {
    pub step_id: String,
    pub task_cid: String,
    pub intended_worker: Option<String>,
    pub dependency_cids: Vec<String>,
}
```

O plano MUST ser representável como artifacts verificáveis. Uma subtarefa MUST NOT ser executada sem budget e política admissíveis.

---

## 16. Seleção de worker

```rust
pub struct WorkerSelection {
    pub worker_id: String,
    pub reason_cid: String,
    pub confidence_q16: u32,
}
```

Seleção de worker SHOULD considerar capability, custo, risco e política. O manager MAY consultar LLM para recomendar seleção. A decisão final MUST continuar sujeita a Gate/contract/proof das RFCs anteriores.

---

## 17. Budget e atenção gerencial

```rust
pub struct BudgetState {
    pub gas_remaining: u64,
    pub max_parallel_workers: u32,
    pub max_open_cases: u32,
    pub max_human_interrupts: u32,
}
```

O manager MUST respeitar budget operacional do caso. O manager MUST NOT abrir delegações ilimitadas.

---

## 18. Manager receipts

```rust
pub enum ManagerReceipt {
    Delegated { case_id: String, worker_id: String, task_cid: String },
    EvidenceRequested { case_id: String, cid: String },
    Escalated { case_id: String, queue: String, reason_code: u32 },
    HumanWitnessRequested { case_id: String, witness_kind: String, prompt_cid: String },
    PointerAdvanced { case_id: String, alias: String, head_cid: String },
    ManagerRejected { case_id: String, reason_code: u32 },
}
```

Toda ação gerencial com efeito no caso SHOULD produzir receipt persistível.

---

## 19. UI model

A UI primária do manager **não** é caixa de texto.

**A UI SHOULD privilegiar:**

- fila de casos
- timeline de events/receipts
- estado do case
- pendências
- budget consumido
- bloqueios
- workers ativos
- escalations
- forks
- pointers avançados

**Modelos de tela recomendados:** case queue, case detail, blocked cases, pending human witnesses, fork resolution board, audit trail / replay inspector.

---

## 20. Human interaction model

Interações humanas SHOULD ser tipadas. Exemplos: confirmar A/B, preencher campo, aprovar/rejeitar, assinar exceção, selecionar head em fork, autorizar aumento de budget.

Interações como "conversa aberta para o sistema descobrir o que fazer" SHOULD NOT ser o caminho primário em fluxos críticos.

---

## 21–24. Integração com RFC-0001 a RFC-0004

- **RFC-0001:** O manager MUST operar dentro da constituição (Commit, Ghost, Reject, epsilon, no-guess, budget law).
- **RFC-0002:** Toda ação gerencial relevante SHOULD entrar no transcript como step, action, receipt, witness, proof artifact.
- **RFC-0003:** O manager MUST tratar tasks, plans, prompts, receipts, worker outputs, witnesses, proofs como artifacts endereçados por conteúdo.
- **RFC-0004:** Managers distintos MAY federar state pointers, proof packs, witness receipts, fork decisions, acceptance receipts.

---

## 25. Failure modes

Implementações conformes MUST distinguir: worker timeout, worker invalid receipt, insufficient evidence, policy violation, out of budget, blocked on human witness, blocked on fork, blocked on storage/materialization, manager planning failure.

---

## 26. Segurança semântica

O manager plane MUST resistir a: comando operacional em linguagem solta sem receipt, estado crítico escondido em contexto de chat, delegação sem trilha, escalonamento sem motivo tipado, aprovação sem evidência suficiente, fusão silenciosa de casos, override humano não registrado.

---

## 27. Conformidade mínima

Uma implementação só é minimamente conforme se:

- usa eventos tipados como plano primário
- emite outputs tipados
- trata chat como periférico
- separa manager de worker
- registra receipts gerenciais relevantes
- opera por caso, não por thread de conversa
- respeita orçamento e política
- integra com proof/transcript das RFCs anteriores

---

## 28. Layout sugerido da crate

```
manager_plane/
  src/
    lib.rs
    case.rs
    input.rs
    output.rs
    receipt.rs
    worker.rs
    human.rs
    budget.rs
    planner.rs
    loop.rs
    ui_model.rs
```

---

## 29. Fluxo canônico

### Caso normal

1. entra Event
2. manager avalia caso
3. manager emite Delegate
4. worker executa e retorna Receipt
5. manager reavalia
6. pede evidência ou delega novo passo
7. quando a prova é suficiente, avança pointer ou terminaliza

### Caso excepcional

1. entra Receipt ambíguo ou insuficiente
2. manager emite AskHumanWitness
3. humano responde
4. witness vira receipt
5. manager continua ou rejeita

### Caso federado

1. pointer remoto avança
2. entra PointerAdvanced
3. manager/local policy avalia impacto
4. se houver fork, bloqueia em WaitingForForkResolution
5. resolução gera novo input e o caso continua

---

## 30. Mantra normativo

```
management is not chat
management is typed continuation
chat is exception
proof is memory
policy is law
```

Versão curta:

```
no chat-first control
no silent delegation
only events
only receipts
only governed steps
```

---

## 31. Veredito arquitetural

A RFC-0005 fecha uma lacuna decisiva:

- RFC-0001 definiu a legitimidade da decisão
- RFC-0002 definiu o transcript e a prova
- RFC-0003 definiu o espaço soberano dos atoms
- RFC-0004 definiu a federação de state pointers
- RFC-0005 define a forma correta do manager

> LLM as Manager não é um chatbot com tools; é um control plane semântico orientado a eventos, governado por política, orçamento e prova.
