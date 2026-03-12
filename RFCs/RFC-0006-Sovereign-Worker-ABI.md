# RFC-0006 — Sovereign Worker ABI & Sandboxed Execution

---

## Contexto

Se a **RFC-0001** ditou a lei, a **RFC-0002** criou o tribunal (prova), a **RFC-0003** definiu a física (espaço/memória), a **RFC-0004** formou a sociedade (federação) e a **RFC-0005** instaurou o governo (Manager Plane)...

Falta definir **os operários**. Como o código que faz o trabalho pesado é empacotado, executado e contido? Se o sistema é *Zero-Trust*, não podemos confiar cegamente no código que processa a IA ou a regra de negócio.

Esta RFC fecha a máquina.

---

## 0. Status

**Draft normativo.**

Esta RFC define a Interface Binária de Aplicação (ABI), o modelo de isolamento (sandbox) e o contrato de determinismo para os "Workers" delegados pelo Manager Plane (RFC-0005).

---

## 1. Tese normativa

O Worker não é um microsserviço com vida própria. O Worker é uma **função pura e isolada** que mapeia um Grafo de CIDs de entrada para um Recibo (Receipt) de saída.

Para garantir o *replay* criptográfico (RFC-0002), o Worker **MUST NOT** possuir acesso direto a disco, rede, relógio de sistema ou entropia oculta.

- Tudo que o Worker sabe vem de **Atoms**.
- Tudo que ele produz vira um **Atom**.
- Se o Worker precisar de um dado que não está na RAM (`Hot`), ele **não faz I/O**. Ele entra em *Epistemic Yield* (suspensão), devolvendo o controle para o Host reidratar a memória.

---

## 2. Invariantes

### I1 — Statelessness absoluto

O Worker não possui estado residual entre execuções. O mesmo `TaskCid` executado no mesmo `WorkerCid` **MUST** produzir o mesmo `ReceiptCid` (dentro das margens de ε contratadas).

### I2 — I/O cego (No-Syscall Rule)

O Worker **MUST NOT** realizar chamadas de sistema (syscalls) para a máquina hospedeira. O único "disco" que ele enxerga é uma interface de requisição de CIDs provida pelo Runtime.

### I3 — Epistemic Yield (co-rotinas de dados)

Se o Worker pedir um CID ao Runtime e o `AtomSpace` (RFC-0003) disser que o dado está `Absent` ou `Cold`, o Worker **MUST** pausar sua execução e retornar um `Yield(MissingCids)`. O Host reidrata os CIDs e acorda o Worker depois.

### I4 — Separação fisiológica (Silício vs. Lógica)

Existem estritamente duas classes de Workers:

1. **Strict Logic (WASM):** Matematicamente exato. Bit-a-bit determinístico.
2. **Statistical Compute (wgpu / Tensor):** Executado em GPU/NPU. Sujeito a variações de hardware (floating point math), mas contido pela margem de erro ε.

---

## 3. O "Manifesto" do Worker (Worker-as-an-Atom)

Um Worker não é um binário solto no SO. Ele é, em si mesmo, um `UniversalAtom` armazenado no `AtomSpace`. O Host carrega o Worker via CID.

```rust
pub struct WorkerManifest {
    pub name: String,
    pub version: String,
    pub class: WorkerClass,
    pub bytecode_cid: String,     // Aponta para o WASM ou WGSL/ONNX
    pub required_capabilities: Vec<Capability>,
    pub determinism_profile: DeterminismProfile, // Vem da RFC-0002
}

pub enum WorkerClass {
    /// O código é a lei. Execução bit-for-bit (ex: compilado para WASM).
    ChipAsCode,
    /// O código aproxima. Executado via wgpu/CUDA/NPU.
    SiliconAsCompute {
        epsilon_bounds: f32,
        quantization: QuantizationLevel, // ex: Q16, FP16, INT8
    },
}
```

---

## 4. A ABI (A interface entre Host e Worker)

O Host (Runtime da RFC-0002) e o Worker se comunicam através de uma fronteira linear e rigorosa de memória.

```rust
pub trait WorkerHostEnv {
    /// Pede um payload associado a um CID.
    /// Retorna Ok(bytes) se estiver HOT na VRAM/RAM.
    /// Retorna Err(PageFault) se estiver COLD ou ABSENT.
    fn request_atom(&self, cid: &str) -> Result<&[u8], PageFault>;

    /// Custos são pagos instrução a instrução ou por batch.
    fn consume_gas(&mut self, amount: u64) -> Result<(), OutOfGas>;
}

pub enum WorkerResult {
    /// Sucesso: O worker finalizou o trabalho e gerou um Receipt.
    Complete(ReceiptCid),

    /// Epistemic Yield: Falta dado em RAM. O Worker devolve o controle pro Host buscar.
    Yield(Vec<String>),

    /// Falha Terminal: O input é inválido ou ocorreu erro interno.
    Fail(WorkerError),
}

/// A assinatura exportada que todo Worker (seja WASM ou nativo) deve implementar.
pub trait WorkerAbi {
    fn execute(
        &mut self,
        task_cid: &str,
        env: &mut dyn WorkerHostEnv
    ) -> WorkerResult;
}
```

---

## 5. Lidando com a fraude do hardware (O problema do ponto flutuante)

A IA tradicional sofre de não-determinismo: rodar um modelo de rede neural em uma GPU NVIDIA RTX 4090 e rodar o mesmo modelo no chip de um iPhone gera *outputs ligeiramente diferentes* por causa da matemática de ponto flutuante.

**Como validar o ProofPack (RFC-0002) se a resposta muda de hardware para hardware?**

A RFC-0006 decreta: O output de um `SiliconAsCompute` Worker não é verificado por *Hash de Conteúdo Exato*, mas por **Prova de Bounding**.

```rust
pub struct SiliconReceipt {
    pub task_cid: String,
    pub result_vector: Vec<i32>, // Sempre quantizado (ex: Q16)
    pub hardware_signature: String, // ex: "Apple M2 GPU"
}

/// Quando o Verificador (Nuvem ou outro Tablet) for auditar a prova:
pub fn verify_silicon_execution(
    expected_receipt: &SiliconReceipt,
    recomputed_receipt: &SiliconReceipt,
    epsilon: f32
) -> bool {
    let distance = calculate_vector_distance(
        &expected_receipt.result_vector,
        &recomputed_receipt.result_vector
    );
    distance <= epsilon
}
```

**Efeito:** O sistema não quebra porque os hardwares pensam diferente. Ele aceita variações fisiológicas do silício, *desde que respeitem a Lei do Erro Contratado (ε)*.

---

## 6. Isolando o oráculo (Testemunhas de tempo e rede)

Se o Worker não tem relógio e não tem internet, como ele sabe que horas são ou pega um dado de uma API externa (como a cotação de uma moeda)?

Ele não faz isso. **O Manager (RFC-0005) faz.**

O Manager injeta no `TaskCid` os "Witnesses" (Testemunhas) necessários.

```json
{
  "t": "atom.task",
  "worker": "cid_worker_pricing_v1",
  "witnesses": [
    { "type": "time", "ms": 1741753200, "oracle_sig": "0x..." },
    { "type": "fetch", "url": "api.price", "response_cid": "cid_..." }
  ]
}
```

O Worker roda sobre uma *foto estática do universo*. Isso garante que se o caso for reexecutado daqui a 10 anos, o Worker produzirá o mesmo exato recibo.

---

## 7. O fluxo de vida de uma tarefa (The Yield Loop)

Para entender o poder do Epistemic Yield, veja como a CPU do Tablet opera sem nunca bloquear (Non-Blocking Data Fetch):

1. **Manager (RFC-0005)** emite `Delegate { worker: "SummarizeText", task: "CID_1" }`.
2. **Runtime (RFC-0002)** inicializa a sandbox do WASM/GPU e chama `execute("CID_1")`.
3. Worker lê `CID_1`. Descobre que precisa processar o texto em `CID_2`.
4. Worker chama `env.request_atom("CID_2")`.
5. **AtomSpace (RFC-0003)** percebe que `CID_2` está no SSD (`Cold`), não na RAM. Retorna `Err(PageFault)`.
6. Worker salva seu estado interno mínimo e retorna `Yield(vec!["CID_2"])`.
7. Runtime destrói temporariamente a sandbox (liberando a CPU). Pede ao AtomSpace para fazer o *Heat Up* (subir pro Hot) do `CID_2`.
8. (10 milissegundos depois) `CID_2` está na RAM. Runtime acorda o Worker de novo.
9. Worker processa tudo, e devolve `Complete(ReceiptCid)`.
10. Manager pega o Receipt e avança o state do caso.

---

## 8. Layout sugerido da crate

```
worker_abi/
  src/
    lib.rs
    manifest.rs     # WorkerManifest, WorkerClass
    env.rs          # WorkerHostEnv, GasMetering
    yield_model.rs  # EpistemicYield structures
    sandbox/
      wasm.rs       # Wasmtime/Wasmer implementation
      wgpu.rs       # WebGPU compute shader implementation
    bounding.rs     # Silicon ε-math validation
```

---

## 9. Mantra normativo

```
Logic is exact.
Silicon is bounded.
Workers cannot speak, they only yield.
Time is an input, not a state.
```

Versão curta (repo-core):

```
no syscalls
no arbitrary I/O
pure functions only
yield on cold memory
```

---

## 10. Veredito arquitetural

Com a **RFC-0006**, o círculo criptográfico está perfeitamente selado.

Você não precisa mais confiar no modelo de linguagem, nem no pipeline RAG, nem no script de validação de negócios. Você só precisa confiar no **Runtime** (que é auditável e *open-source*) e no **Math** (Hashing + Assinaturas).

| Situação | Contenção |
|----------|-----------|
| Modelo alucina | O Gate (RFC-0001) rejeita. |
| Worker tenta roubar dados | A Sandbox (RFC-0006) bloqueia — não tem acesso a rede/disco. |
| Sistema fica sem RAM | O AtomSpace (RFC-0003) ejeta memórias quentes. |
| Usuário cai offline | Continua operando; a Federação (RFC-0004) resolve por gossip e fork depois. |

---

## Resumo das 6 RFCs

| RFC | Papel |
|-----|-------|
| 0001 | Lei — Constituição |
| 0002 | Tribunal — Prova |
| 0003 | Física — Espaço |
| 0004 | Sociedade — Federação |
| 0005 | Governo — Manager |
| 0006 | Operários — Workers |
