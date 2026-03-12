# RFC-0004 — Federation under Proof-Carrying State (Revisada)

---

## 0. Status

**Draft normativo**

Esta RFC define a camada federativa de um sistema local-first, zero-trust, orientado a prova, no qual o estado compartilhado é representado por ponteiros assinados sobre grafos imutáveis endereçados por conteúdo.

---

## 1. Escopo

**Esta RFC especifica:**

- identidade de nós e authorities
- avanço de StatePointer
- anúncio de avanço
- aceitação local de avanços remotos
- detecção de fork
- resolução de fork por política
- anti-rewind
- quorum opcional
- integração com ProofPack, AtomSpace e Contract

**Esta RFC não especifica:**

- transporte físico específico
- criptografia concreta de assinatura
- protocolo de descoberta peer-to-peer
- consenso global universal
- política de negócio de domínio

---

## 2. Tese normativa

A federação **MUST NOT** sincronizar estado mutável.

A federação **MUST** trocar apenas:

- CIDs
- ProofPacks
- StatePointers assinados
- anúncios
- recibos de aceitação
- artefatos de conflito

Consenso federativo **MUST** ser tratado como: acordo local e verificável sobre qual prova tem direito de avançar qual ponteiro em qual namespace.

---

## 3. Termos normativos

As palavras MUST, MUST NOT, SHOULD, SHOULD NOT e MAY têm sentido normativo.

| Termo | Definição |
|-------|-----------|
| **Accept** | Reconhecer localmente um avanço como válido segundo política local. |
| **Announce** | Publicar um candidato de avanço. |
| **Authority** | Entidade autorizada a avançar ou testemunhar em um namespace. |
| **Fork** | Existência de dois ou mais heads incompatíveis para o mesmo alias. |
| **Pointer** | Registro mutável mínimo que aponta para um head imutável. |
| **Proof-Carrying State** | Estado cujo avanço só é válido quando acompanhado de prova replayável. |

---

## 4. Invariantes

### I1 — Verdade continua fora da federação

A federação MUST NOT tratar nenhum nó como autoridade sobre a verdade factual. A verdade continua residindo em: atoms imutáveis, transcript, ProofPack, contrato hashado, receipts verificáveis.

### I2 — Avanço de pointer exige prova

Um StatePointer novo MUST NOT ser aceito sem: assinatura válida, ProofPack verificável ou resolvível, contrato reconhecido, política de namespace satisfeita.

### I3 — Recepção não implica aceitação

Receber anúncio, bytes, CIDs ou proofs MUST NOT implicar aceitação.

### I4 — Conflito é objeto de primeira classe

Heads concorrentes MUST ser preservados como fork explícito até resolução ou política de convivência.

### I5 — Anti-rewind é obrigatório

Nenhum nó MUST aceitar regressão silenciosa de sequência ou head.

### I6 — Política é local

Aceitação federativa MUST ser decidida por política local, mesmo quando os dados vierem de fonte confiável.

---

## 5. Identidade e autoridade

### 5.1 NodeIdentity

```rust
pub struct NodeIdentity {
    pub node_id: String,
    pub public_key: String,
    pub roles: Vec<NodeRole>,
}

pub enum NodeRole {
    EdgeExecutor,
    PointerAuthority,
    WitnessAuthority,
    ContractPublisher,
    Mirror,
}
```

**Regras:** Cada `node_id` MUST mapear para exatamente uma chave pública ativa. Um nó MAY possuir múltiplos papéis. Papéis MUST NOT implicar autoridade universal.

### 5.2 Authority namespace

Autoridade MUST ser namespaceada. Uma authority válida para `contracts:*` MUST NOT ser assumida válida para `cases:*` sem política explícita.

```rust
pub struct AuthorityGrant {
    pub authority_id: String,
    pub namespace_prefix: String,
    pub allowed_roles: Vec<NodeRole>,
    pub valid_from_epoch: u64,
    pub valid_until_epoch: Option<u64>,
}
```

---

## 6. StatePointer

```rust
pub struct StatePointer {
    pub alias: String,
    pub prev_head_cid: Option<String>,
    pub head_cid: String,
    pub sequence_number: u64,
    pub authority_id: String,
    pub authority_signature: String,
}
```

### 6.1 Regras normativas

Um StatePointer candidato MUST satisfazer:

1. alias não vazio
2. head_cid sintaticamente válido
3. sequence_number > 0
4. assinatura válida sobre o conteúdo canônico do pointer
5. authority_id reconhecida para o namespace do alias
6. prev_head_cid consistente com política local ou explícita condição de fork

### 6.2 Anti-rewind

Dado um `last_seen_pointer` para o mesmo alias, um novo pointer MUST ser rejeitado se:

- `sequence_number < last_seen.sequence_number`
- `sequence_number == last_seen.sequence_number` e `head_cid != last_seen.head_cid`
- `prev_head_cid != Some(last_seen.head_cid)` em namespace que não permita fork
- assinatura inválida
- autoridade não autorizada

**Exceção:** Se a política do alias permitir fork, o candidato MAY ser registrado como fork em vez de rejeitado.

---

## 7. Classes de ponteiro

```rust
pub enum PointerClass {
    Personal,
    SharedCase,
    ContractHead,
    WitnessLog,
    MirrorIndex,
}

pub struct PointerPolicy {
    pub alias_prefix: String,
    pub class: PointerClass,
    pub accepted_authorities: Vec<String>,
    pub requires_quorum: bool,
    pub quorum_size: u32,
    pub allow_forks: bool,
    pub require_proof_pack: bool,
}
```

**Regras:** Todo alias MUST casar com exatamente uma política efetiva. Se múltiplas políticas casarem, a mais específica MUST prevalecer. Se nenhuma casar, o alias MUST ser tratado como não autorizado.

---

## 8. Announcement protocol

```rust
pub struct PointerAnnouncement {
    pub pointer: StatePointer,
    pub proof_pack_cid: String,
    pub contract_hash: String,
    pub announcer_node_id: String,
    pub announcement_signature: String,
}
```

**Regras:** Todo anúncio MUST ser assinado. O anúncio MUST referenciar `proof_pack_cid` quando a política exigir prova. O anúncio MUST NOT carregar "estado final" mutável inline como autoridade primária.

---

## 9. Dependency resolution

```rust
pub enum DependencyStatus {
    Complete,
    Missing(Vec<String>),
}

pub trait FederationTransport {
    fn announce(&mut self, msg: PointerAnnouncement) -> anyhow::Result<()>;
    fn request_atoms(&mut self, cids: &[String]) -> anyhow::Result<Vec<UniversalAtom>>;
    fn request_proof_pack(&mut self, cid: &str) -> anyhow::Result<ProofPack>;
}
```

**Regras:** Dependências MUST ser resolvidas por CID. Um nó MUST NOT aceitar um head cujo ProofPack necessário não seja verificável. Um nó MAY marcar um anúncio como Deferred enquanto dependências estiverem faltando.

---

## 10. Acceptance

```rust
pub enum AcceptanceVerdict {
    Accepted,
    Rejected(AcceptRejectReason),
    ForkDetected,
    Deferred,
}

pub enum AcceptRejectReason {
    MissingDependencies,
    InvalidSignature,
    InvalidProof,
    InvalidContract,
    AuthorityViolation,
    RewindAttempt,
    SequenceGap,
    PolicyViolation,
}
```

**Regras:** Um anúncio MUST resultar em exatamente um veredito local. Um anúncio MUST NOT ficar implicitamente aceito por timeout ou silêncio.

---

## 11. AcceptanceReceipt

```rust
pub struct AcceptanceReceipt {
    pub pointer_alias: String,
    pub head_cid: String,
    pub verifier_node_id: String,
    pub verdict: AcceptanceVerdict,
    pub reason: Option<AcceptRejectReason>,
    pub verifier_signature: String,
}
```

---

## 12. Pointer validation

```rust
pub trait PointerValidator {
    fn validate_pointer(
        &self,
        new_pointer: &StatePointer,
        previous: Option<&StatePointer>,
        fed: &FederationView,
    ) -> AcceptanceVerdict;
}
```

O validador MUST checar: assinatura, monotonicidade de sequência, consistência de prev_head_cid, autoridade para o namespace, existência de política aplicável.

---

## 13. Proof verification before acceptance

Antes de aceitar um pointer cujo policy exija prova, o receptor MUST:

1. resolver o ProofPack
2. resolver as dependências necessárias
3. verificar o ProofPack com UniversalVerifier
4. confirmar que o contract_hash é reconhecido
5. confirmar que o head anunciado casa com o outcome/proof esperado

Se qualquer etapa falhar, o anúncio MUST ser rejeitado ou adiado, nunca aceito.

---

## 14. FederationView

```rust
pub struct FederationView {
    pub recognized_nodes: Vec<NodeIdentity>,
    pub accepted_contract_hashes: Vec<String>,
    pub pointer_policies: Vec<PointerPolicy>,
    pub acceptance_receipts: Vec<AcceptanceReceipt>,
}
```

---

## 15. Forks

```rust
pub struct PointerFork {
    pub alias: String,
    pub base_head_cid: Option<String>,
    pub competing_heads: Vec<String>,
    pub detected_by: String,
}
```

**Regras:** Ao detectar heads concorrentes válidos, um nó MUST registrar PointerFork ou rejeitar segundo política. Um fork MUST NOT ser resolvido por overwrite silencioso.

---

## 16. ResolutionPolicy

```rust
pub trait ResolutionPolicy {
    fn resolve(&self, fork: &PointerFork, ctx: &FederationView) -> ResolutionOutcome;
}

pub enum ResolutionOutcome {
    ChooseHead { head_cid: String },
    PreserveFork,
    RequireHumanWitness,
    RequireQuorum,
    RejectAll,
}
```

---

## 17. Quorum

```rust
pub struct QuorumProof {
    pub alias: String,
    pub head_cid: String,
    pub acceptance_receipt_cids: Vec<String>,
}

pub enum PointerStatus {
    LocalOnly,
    Announced,
    FederallyAccepted,
    Rejected,
    Forked,
}
```

Quando `requires_quorum = true`, um head MUST NOT ser marcado FederallyAccepted sem receipts suficientes.

---

## 18. Sequence gaps

Se um nó receber um pointer com `sequence_number > last_seen + 1`, ele MAY: rejeitar por SequenceGap, adiar por Deferred, ou solicitar ponteiros intermediários. Ele MUST NOT assumir automaticamente que a cadeia intermediária é irrelevante, exceto se a política permitir saltos explícitos.

---

## 19. Contract federation

```rust
pub struct ContractAnnouncement {
    pub contract_hash: String,
    pub contract_cid: String,
    pub publisher_id: String,
    pub publisher_signature: String,
}
```

---

## 20. Witness federation

```rust
pub struct WitnessReceipt {
    pub witness_kind: String,
    pub payload_cid: String,
    pub witness_authority_id: String,
    pub witness_signature: String,
}
```

---

## 21. Announcement processing pipeline

**Pipeline normativo:**

1. verificar assinatura do anúncio
2. validar sintaxe do pointer
3. localizar política do alias
4. comparar com pointer anterior conhecido
5. resolver dependências mínimas
6. verificar ProofPack se exigido
7. avaliar conflito/fork
8. emitir AcceptanceReceipt

---

## 22. ForkRegistry

```rust
pub trait ForkRegistry {
    fn register_fork(&mut self, fork: PointerFork) -> anyhow::Result<()>;
    fn list_forks(&self, alias: &str) -> Vec<PointerFork>;
}
```

---

## 23. ResolutionEngine

```rust
pub trait ResolutionEngine {
    fn resolve_fork(
        &self,
        fork: &PointerFork,
        fed: &FederationView,
    ) -> anyhow::Result<ResolutionOutcome>;
}
```

---

## 24. Local sovereignty

A federação MUST preservar soberania local:

- prova local válida MUST NOT ser destruída por rejeição remota
- cloud MUST NOT ser tratada como autoridade implícita
- um nó local MAY manter estado LocalOnly
- aceitação remota MAY promover status federativo, mas MUST NOT redefinir a validade criptográfica intrínseca do ProofPack

---

## 25. Failure modes

Implementações MUST distinguir ao menos: falha de transporte, assinatura, política, dependência, replay/prova, tentativa de rewind, fork detectado, contrato desconhecido.

---

## 26. Segurança

**Ataques que a arquitetura deve resistir:**

- overwrite silencioso de head
- replay de pointer antigo
- anúncio com assinatura inválida
- contrato malicioso não reconhecido
- proof pack inválido mas bem formatado
- injeção de dependência errada
- quorum forjado por nós não reconhecidos

---

## 27. Layout sugerido da crate

```
proof_federation/
  src/
    lib.rs
    node.rs
    pointer.rs
    announcement.rs
    acceptance.rs
    fork.rs
    policy.rs
    resolution.rs
    transport.rs
    validator.rs
    quorum.rs
```

---

## 28. Fluxo canônico

### Caso: avanço simples aceito

1. Node A gera ProofPack
2. Node A cria StatePointer
3. Node A emite PointerAnnouncement
4. Node B recebe anúncio
5. Node B valida assinatura e política
6. Node B resolve ProofPack e dependencies
7. Node B roda UniversalVerifier
8. Node B aceita localmente
9. Node B emite AcceptanceReceipt

### Caso: fork

1. Node A anuncia head_1
2. Node C anuncia head_2 para o mesmo alias e mesma base
3. Node B detecta incompatibilidade
4. Node B registra PointerFork
5. Node B aplica ResolutionPolicy
6. resultado: escolhe um / preserva ambos / pede witness humano / aguarda quorum

---

## 29. Conformidade mínima

Uma implementação só é minimamente conforme se:

- valida assinatura de pointer
- aplica anti-rewind
- resolve dependencies por CID
- verifica ProofPack quando exigido
- trata fork como objeto explícito
- emite AcceptanceReceipt
- separa recepção de aceitação

---

## 30. Mantra normativo

```
state does not merge
heads compete
proof decides
policy accepts
```

Versão dura:

```
no blind trust
no silent overwrite
only signed heads
only verifiable advances
```

---

## 31. Veredito arquitetural

A RFC-0004 revisada fecha a camada política da máquina:

- RFC-0001 define quem pode decidir
- RFC-0002 define como a prova é construída
- RFC-0003 define onde o conhecimento imutável vive
- RFC-0004 define como soberanos locais coexistem sem regredir para banco distribuído mutável

> Federação não é sincronizar memória; é arbitrar avanços assinados sobre prova verificável.
