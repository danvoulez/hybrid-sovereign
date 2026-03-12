# RFC-0003 — Sovereign Atom Space (Storage under Epistemic Law)

---

## Contexto

Se a **RFC-0001** definiu a *Constituição da Memória* (Gate, Budget, Frugalidade) e a **RFC-0002** definiu o *Processo Penal* (State Machine, Transcript, Proofs), falta definir o **Solo Epistêmico** onde essas provas existem e persistem.

A RFC-0003 resolve o problema do "Estado" e do "Banco de Dados" em um sistema onde as inferências rodam na borda (edge), caem offline e precisam garantir auditoria criptográfica.

---

## 0. Tese central

O banco de dados tradicional (mutável, CRUD) é um anti-padrão para sistemas baseados em prova.

Se um registro pode sofrer `UPDATE`, o histórico criptográfico é destruído, e o replay torna-se impossível.

Neste sistema:

- **O disco local não é um arquivo morto; é um cache termodinâmico.**
- **A verdade não reside em tabelas; reside em Grafos Direcionados Acíclicos (DAGs) endereçados por conteúdo (CAS).**
- Tudo — desde os pesos de um modelo de IA (gigante), o contrato WASM (médio), até o input de um usuário (pequeno) — é submetido à mesma lei gravitacional: **O Átomo Universal**.

Logo:

- Mutação é apenas a criação de um novo átomo e o avanço de um ponteiro.
- O esquecimento (eviction) não é um erro do sistema, é um mecanismo de sobrevivência.

---

## 1. Objetivo

Definir um subsistema de armazenamento local e rede onde:

- Endereçamento por conteúdo (CID) é a **única** forma de acessar conhecimento.
- O ciclo de vida dos dados obedece rigorosamente ao *Heat Model* (Cold, Warm, Hot).
- A sincronização entre dispositivos (Tablet ↔ Cloud ↔ Tablet) não requer conciliação de banco de dados, apenas *gossip* de CIDs faltantes.
- O `ProofPack` da RFC-0002 pode ter suas dependências resolvidas de forma cega, sem saber se vêm do disco, da rede ou de um pendrive air-gapped.

---

## 2. Invariantes constitucionais

### I1 — Verdade é Hash (Sem CID, sem existência)

Nenhum subsistema pode solicitar um dado por "ID semântico" (ex: `user_123`). O runtime só pede CIDs. A resolução Semântica → CID é isolada.

### I2 — Mutação é estritamente proibida

Uma vez que um Átomo entra no `AtomSpace`, seus bytes nunca mudam. Para alterar um estado, emite-se um novo Átomo.

### I3 — Frugalidade exige Omissão (Absent State)

É perfeitamente legal o dispositivo local conhecer o *Hash* de um Átomo, mas não possuir seu *Payload*. O sistema deve ser capaz de provar a cadeia, mesmo que precise paginar (Ghost) blocos da rede.

### I4 — Eviction é ditatorial

Quando o `BudgetContract` acusa limite de disco ou RAM, o sistema rebaixa a temperatura dos Átomos mais ociosos. Se um modelo de IA não couber, ele é fragmentado ou ejetado.

---

## 3. A Anatomia do Átomo Universal

Não existem "tipos de tabelas". Existe apenas um formato universal de empacotamento.

```rust
pub struct UniversalAtom {
    pub header: AtomHeader,
    pub links: Vec<String>,     // CIDs que este átomo referencia
    pub payload: Vec<u8>,       // Os bytes brutos do dado
}

pub struct AtomHeader {
    pub kind: AtomKind,
    pub size_bytes: u64,
    pub producer_hash: String,
    pub signature: Option<String>,
}

pub enum AtomKind {
    Weights,
    WasmContract,
    PromptText,
    ProofPack,
    StateRoot,
    WitnessData,
}
```

---

## 4. Termodinâmica Epistêmica (O Pager)

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EpistemicHeat {
    Absent,     // Conhecemos o CID, mas zero bytes locais.
    Cold,       // Temos o Header e Links no SSD, mas Payload vazio/ignorado.
    Warm,       // Temos um resumo (Embedding / Metadado) em memória rápida.
    Hot,        // Payload completo carregado, validado e residente em RAM/VRAM.
}
```

**Regra de Transição:** O Runtime (RFC-0002) nunca faz I/O direto. Ele envia uma `MaterializeAction::RehydrateAtom`. O `AtomSpace` executa o *Heat Up*. Se estourar a RAM, o `AtomSpace` força o *Cool Down* de átomos não travados pelo caso atual, antes de servir o novo.

---

## 5. Contrato de Armazenamento (Interface)

```rust
pub trait AtomSpace {
    fn current_heat(&self, cid: &str) -> EpistemicHeat;

    fn heat_up(&mut self, cid: &str, target: EpistemicHeat) -> Result<(), PageFault>;

    fn cool_down(&mut self, cid: &str) -> Result<(), ()>;

    fn materialize(&mut self, atom: UniversalAtom) -> Result<String, StorageError>;

    fn get_thermal_metrics(&self) -> ThermalMetrics;
}

pub enum PageFault {
    NetworkRequired { cid: String },
    BudgetExhausted { required: u64, available: u64 },
    CorruptedCid { expected: String, actual: String },
}
```

---

## 6. Ponteiros Epistêmicos (A única Mutabilidade Permitida)

Se o mundo é imutável, como sabemos qual é a última sessão do usuário ou a versão mais nova do contrato? Através de um **Pointer Registry** isolado.

```rust
pub struct StatePointer {
    pub alias: String,
    pub head_cid: String,
    pub sequence_number: u64,
    pub authority_signature: String,
}
```

**Lei:** O Pointer é apenas uma placa de rua. Ele não contém verdade, apenas direciona o sistema para o CID correto. Se o banco de Pointers for destruído, o estado do sistema pode ser totalmente recriado recalculando os grafos a partir dos `ProofPacks` do `AtomSpace`.

---

## 7. Sincronização Local-First (Gossip Protocol)

Sistemas na borda perdem conectividade. A arquitetura de CIDs torna a sincronização entre dispositivos magicamente simples e *Zero-Trust*:

1. O Tablet emite um `ProofPack` offline e avança seu `StatePointer`.
2. O Tablet ganha Wi-Fi. Ele não manda um JSON gigante para a API.
3. Ele envia apenas: *"Meu ponteiro 'case_01' está no CID X"*.
4. O Cloud Server verifica se possui o CID X.
5. Se não, o Servidor pede: *"Me mande os blocos do CID X"*.
6. O Servidor recebe, verifica o hash e re-executa a prova (UniversalVerifier).
7. Se a prova for válida, o Cloud aceita a decisão feita no Tablet.

Não há conflito de banco de dados (Merge Conflicts). Só existe replicação matemática.

---

## 8. Integração com as RFCs anteriores (O Loop Supremo)

1. **(RFC-0003)** O dispositivo acorda e lê o `StatePointer` do Contrato, descobrindo o `contract_cid`.
2. **(RFC-0003)** O `AtomSpace` garante que o contrato está `Hot` (RAM).
3. **(RFC-0002)** A `Session` é iniciada.
4. **(RFC-0002)** O contrato determina a primeira ação: `ComputeAction::Propose`.
5. **(RFC-0001)** O *Proposer* olha os dados locais e sugere um caminho (score + risco).
6. **(RFC-0001)** O *Gate* avalia. Detecta que falta evidência em RAM. Rejeita com `GhostAction::RehydrateAtom(cid_XYZ)`.
7. **(RFC-0002)** O Runtime gera um recibo dessa suspensão e a envia ao `AtomSpace`.
8. **(RFC-0003)** O `AtomSpace` busca o `cid_XYZ` (talvez via rede), joga para `Hot`.
9. **(RFC-0002)** A sessão retoma (resume). O *Gate* agora aprova a inferência.
10. **(RFC-0002)** Um `ProofPack` final é emitido e armazenado no disco.
11. **(RFC-0003)** O `StatePointer` do caso é atualizado para apontar para o `ProofPack`.

---

## 9. Layout sugerido da crate v4

```
epistemic_storage/
  src/
    lib.rs
    atom.rs          # UniversalAtom, AtomHeader
    heat.rs          # EpistemicHeat, PageFault
    pager.rs         # Lógica de RAM/VRAM/Disk eviction
    cas.rs           # Content-Addressable Storage (o disco)
    pointers.rs      # Name resolution (Alias -> CID)
    network.rs       # Interface para gossip de CIDs
```

---

## 10. Mantra do Sistema Atualizado

> **State is a pointer.**  
> **Knowledge is a graph.**  
> **Disk is a cache.**  
> **Forgetfulness is survival.**

---

## Veredito sobre a SPEC 003

Com a **RFC-0001** você calibrou a governança (quem decide e como).

Com a **RFC-0002** você blindou o histórico (o recibo inquestionável).

Com esta **RFC-0003**, você resolveu a **Física e o Espaço-Tempo** da sua máquina. O disco deixa de ser um local de ansiedade de estado e passa a ser apenas um canal de reidratação criptográfica.

Você acabou de desenhar o kernel de um Sistema Operacional nativo para Agentes Fisiológicos e Zero-Trust.
