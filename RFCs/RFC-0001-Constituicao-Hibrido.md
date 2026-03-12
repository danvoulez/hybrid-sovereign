# RFC-0001 — Constituição do Híbrido

---

## Preâmbulo

**Este sistema existe para democratizar poder de processamento sem sacrificar confiança.**

**O silício pode ser estatístico. O mundo não pode ser estatístico "sem contrato".**

---

## Artigo I — Lei do Resultado

Toda computação deve terminar em um de três estados:

- **OK** → COMMIT
- **DOUBT** → GHOST (1 pergunta)
- **NOT** → REJECT

Não existe "talvez commit". Não existe "quase ok".

---

## Artigo II — Lei do Erro Contratado

Erro permitido é cláusula explícita, nunca acidente.

O sistema deve declarar antes de rodar:

- margem de erro ε
- budget de dúvida (quantos casos podem virar GHOST)
- domínios proibidos (onde ε = 0)

Se não há contrato, o sistema cai para modo conservador.

---

## Artigo III — Lei do No-Guess

Sem evidência suficiente, é proibido adivinhar.

Se faltar dado mínimo exigido:

- GHOST (pergunta única e objetiva)
- é ilegal gerar COMMIT por completude falsa

---

## Artigo IV — Lei do Silício Criativo

O silício é livre para aproximar, desde que o Gate consiga provar segurança.

O silício pode:

- interpolar, prever, comprimir, estimar
- usar heurística, sampling, modelos, atalhos

Mas só é aceito se:

- respeitar ε e as proibições
- produzir score + explicação mínima de risco
- passar pelo Gate

---

## Artigo V — Lei da Prova

Toda decisão deve ser replayável e auditável.

Para qualquer output:

- existe identidade de conteúdo (CID)
- existe derivation que re-roda e bate CID
- existe trilha mínima de inputs/params

Sem replay, é apenas opinião.

---

## Artigo VI — Lei da Alocação

Cada tarefa deve ir para onde ela é melhor:

- **Silício:** throughput, paralelismo, aproximação
- **Chip-as-Code:** política, invariantes, auditabilidade, no-guess
- **Híbrido:** "silício propõe → gate decide"

O objetivo é eficiência com governança, não velocidade cega.

---

## Artigo VII — Lei do Mínimo Hardware

O hardware mínimo aceitável é o que preserva a Constituição.

Se o dispositivo não sustenta:

- replay mínimo
- CIDs
- `gate_run` determinístico

Ele pode até computar, mas não pode "decidir".

---

## Artigo VIII — Lei do Upgrade Sem Traição

Atualizações não podem invalidar a história.

Uma mudança é permitida apenas se:

- muda o kernel → muda o `kernel_hash` / `shader_hash`
- mantém replay para o que foi emitido
- preserva compatibilidade ou explicita ruptura

---

## Artigo IX — Lei do Poder Democrático

O sistema deve escalar para os fracos, não só para os fortes.

A arquitetura é considerada bem-sucedida quando:

- roda em 1 dispositivo barato
- e, se federar, federar sem perder prova

---

## Mantra (versão LogLine-core)

```
if ok   → commit
if doubt → ghost
if not  → reject
```

**E abaixo:** ε é contrato. replay é lei. silício é livre.

---

## Referências de Código (Rust)

Materialização da Constituição:

- JSON✯Atomic CID
- `gate_run` (OK/DOUBT/NOT)
- `silicon_propose` (estatístico com ε contratado)
- NDJSON atoms (fact/set/derivation)
- verify replay (recalcula e bate CID)

---

### 1. Tipos base: Verdict + Contrato de erro (ε)

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    Commit,
    Ghost,
    Reject,
}

#[derive(Debug, Clone)]
pub struct ErrorContract {
    /// epsilon permitido (ex: 0.05 = 5%)
    pub epsilon: f32,
    /// domínios onde epsilon=0 (proibido chutar)
    pub zero_guess_domains: Vec<&'static str>,
    /// máximo de perguntas (ghost) por decisão
    pub max_questions: u8,
}

impl ErrorContract {
    pub fn forbids_guess(&self, domain: &str) -> bool {
        self.zero_guess_domains.iter().any(|d| *d == domain)
    }
}
```

---

### 2. JSON✯Atomic CID: canonize → BLAKE3

```rust
use anyhow::Result;
use blake3;
use serde::Serialize;

pub fn cid_for_value<T: Serialize>(value: &T) -> Result<String> {
    let canon = logline::json_atomic::canonize(value)?;
    Ok(hex::encode(blake3::hash(&canon).as_bytes()))
}

use serde_json::Value;

pub fn cid_for_atom_without_cid(v: &Value) -> Result<String> {
    let mut tmp = v.clone();
    if let Value::Object(ref mut m) = tmp {
        m.remove("cid");
    }
    cid_for_value(&tmp)
}

pub fn attach_cid(mut v: Value) -> Result<Value> {
    let cid = cid_for_atom_without_cid(&v)?;
    if let Value::Object(ref mut m) = v {
        m.insert("cid".into(), Value::String(cid));
    }
    Ok(v)
}
```

---

### 3. Silício pode ser estatístico: `silicon_propose`

```rust
#[derive(Debug, Clone)]
pub struct SiliconProposal {
    pub score_q16: u32,
    pub risk_q16: u32,
    pub kernel_hash: String,
}

pub fn silicon_propose(seed: u64, idx: u64) -> SiliconProposal {
    let mut r = rand_chacha::ChaCha20Rng::seed_from_u64(seed ^ idx.wrapping_mul(0x9E37));
    let score_q16 = (r.next_u32() % 65_536) as u32;
    let mid = 52_000i64;
    let dist = (score_q16 as i64 - mid).abs().min(65_535) as u32;
    let risk_q16 = dist;
    SiliconProposal {
        score_q16,
        risk_q16,
        kernel_hash: "cpu_v1".into(),
    }
}
```

---

### 4. Gate-as-Compute: `gate_run`

```rust
pub const OK_MIN_Q16: u32 = 52_000;
pub const DOUBT_BELOW_Q16: u32 = 48_000;

pub const R_MISSING_EVIDENCE: u32 = 0x01;
pub const R_UNANCHORED: u32 = 0x02;
pub const R_POLICY_VIOLATION: u32 = 0x04;
pub const R_SILICON_DOUBT: u32 = 0x08;
pub const R_SILICON_NOT_OK: u32 = 0x10;

#[derive(Debug, Clone)]
pub struct GateInputs<'a> {
    pub domain: &'a str,
    pub has_intent: bool,
    pub has_evidence: bool,
    pub evidence_anchored: bool,
    pub policy_ok: bool,
    pub epoch: u64,
    pub contract: &'a ErrorContract,
}

#[derive(Debug, Clone)]
pub struct GateDecision {
    pub verdict: Verdict,
    pub reason_code: u32,
    pub question: Option<String>,
}

pub fn gate_run(inp: GateInputs, silicon: SiliconProposal) -> GateDecision {
    let mut reason = 0u32;

    if !inp.has_intent || !inp.has_evidence {
        reason |= R_MISSING_EVIDENCE;
        return GateDecision {
            verdict: Verdict::Ghost,
            reason_code: reason,
            question: Some("Qual evidência está faltando para confirmar?".into()),
        };
    }

    if !inp.evidence_anchored {
        reason |= R_UNANCHORED;
        return GateDecision { verdict: Verdict::Reject, reason_code: reason, question: None };
    }

    if !inp.policy_ok {
        reason |= R_POLICY_VIOLATION;
        return GateDecision { verdict: Verdict::Reject, reason_code: reason, question: None };
    }

    if inp.contract.forbids_guess(inp.domain) {
        if silicon.score_q16 < OK_MIN_Q16 {
            reason |= R_SILICON_NOT_OK;
            return GateDecision { verdict: Verdict::Reject, reason_code: reason, question: None };
        }
        return GateDecision { verdict: Verdict::Commit, reason_code: reason, question: None };
    }

    if silicon.score_q16 < DOUBT_BELOW_Q16 {
        reason |= R_SILICON_DOUBT;
        return GateDecision {
            verdict: Verdict::Ghost,
            reason_code: reason,
            question: Some("Tenho dúvida: confirma X ou Y?".into()),
        };
    }

    if silicon.score_q16 < OK_MIN_Q16 {
        reason |= R_SILICON_NOT_OK;
        return GateDecision { verdict: Verdict::Reject, reason_code: reason, question: None };
    }

    GateDecision { verdict: Verdict::Commit, reason_code: reason, question: None }
}
```

---

### 5. Emitindo átomos NDJSON: fact / set / derivation

*(Ver implementação completa em `gate_run_atoms` — facts, sets, silicon_propose, gate_run, derivation com inputs/outputs.)*

---

### 6. Verify replay

O truque: dado o `atom.derivation` com `op="gate_run"`:

1. resolve inputs
2. reexecuta `silicon_propose` + `gate_run`
3. reconstrói outputs esperados
4. compara cid (e/ou members) com o ledger

---

## Extra: Escalando para "100 milhões no tablet"

- `silicon_propose` vira wgpu compute (muitos casos por dispatch)
- o Gate fica CPU leve
- não grava 100M outputs: grava rollups, amostras (`sample_every`), `batch_hash` (prova compacta)
