# ASR-01 근거 원장 — Windows Consumer PC / Qwen3-8B reference profile

> 재정리일: 2026-09-21
> 상태: **11-A planning evidence**. 실제 VIA 후보의 measured latency가 아니며 11-C target/score도 아직 아니다.
> 목적: VIA Semantic Model을 사용자가 일반적인 Windows PC에서 local inference한다고 가정할 때 사용할 수 있는 근거 있는 reference profile을 고정한다.

## 1. 왜 Qualcomm profile을 대표 근거에서 제외하는가

기존 Qualcomm Snapdragon X Elite 자료는 Qwen3-8B의 공개 TTFT/generation 수치를 제공한다는 장점이 있었지만 VIA의 기본 가정인 **Windows PC + Intel-class CPU + 일반 consumer discrete GPU**와 실행 환경이 다르다. 따라서 Qualcomm 수치는 교차 참고로만 남기고 ASR-01의 주 planning profile에는 사용하지 않는다.

대표 planning evidence는 다음 우선순위로 사용한다.

1. 공식 Qwen model/tokenizer/GGUF artifact
2. Windows + consumer NVIDIA GPU에서 Qwen3-8B Q4_K_M을 실제 실행한 공개 측정
3. 동일 모델/quant/runtime family의 다른 consumer GPU 측정으로 sanity check
4. 실제 VIA 후보를 실행할 수 있게 되면 동일 benchmark contract의 local measurement로 최종 교체

## 2. 공식 Model / Tokenizer snapshot

### Model

- Model: **Qwen/Qwen3-8B**
- Parameters: 8.2B
- Native context: 32,768 tokens
- Non-thinking hard switch: `enable_thinking=False`
- Official model card:
  - https://huggingface.co/Qwen/Qwen3-8B

Qwen 공식 model card는 non-thinking mode를 효율적인 일반 대화용으로 명시하며 `enable_thinking=False`로 thinking output을 비활성화할 수 있다고 설명한다. VIA의 bounded semantic 판단/structured output benchmark에서는 **non-thinking mode**를 reference로 사용한다.

### Tokenizer / chat template

- Repository: `Qwen/Qwen3-8B`
- `tokenizer.json`
  - source commit lineage: initial tokenizer upload `47719a2`
  - SHA256: `aeb13307a71acd8fe81861d94ad54ab689df773318809eed3cbe794b4492dae4`
- `tokenizer_config.json`
  - frozen template commit: `895c8d171bc03c30e113cd7a28c02494b5e068b7`
  - source: https://huggingface.co/Qwen/Qwen3-8B/commit/895c8d171bc03c30e113cd7a28c02494b5e068b7
- tokenizer source:
  - https://huggingface.co/Qwen/Qwen3-8B/blob/main/tokenizer.json

정확 token count는 위 tokenizer와 chat template snapshot으로 실제 직렬화한 전체 prompt를 tokenize한다. 글자 수/단어 수 비례 추정은 사용하지 않는다.

### GGUF artifact

- Repository: **Qwen/Qwen3-8B-GGUF**
- Quantization: **Q4_K_M**
- File size: **5.03 GB**
- frozen file revision: `6a569868d07d3bd59e8b97fb001bf8c0b254bb20`
- SHA256: `d98cdcbd03e17ce47681435b5150e34c1417f50b5c0019dd560e4882c5745785`
- source:
  - https://huggingface.co/Qwen/Qwen3-8B-GGUF/blob/6a569868d07d3bd59e8b97fb001bf8c0b254bb20/Qwen3-8B-Q4_K_M.gguf

공식 page는 Windows에서 `winget install llama.cpp`와 `llama serve -hf Qwen/Qwen3-8B-GGUF:Q4_K_M` 사용법도 제공한다. 즉 추후 실제 VIA benchmark를 할 경우 별도 proprietary runtime/API가 필수는 아니다.

## 3. 주 Planning Reference — Windows 11 + RTX 4060 8GB

출처:
https://localllm.in/blog/ollama-vram-requirements-for-local-llms

공개 test configuration:

| 항목 | 공개 값 |
| --- | --- |
| OS | Windows 11 |
| GPU | NVIDIA RTX 4060, 8 GB |
| Model | Qwen3 8B Q4_K_M |
| Context setting | 16k |
| GPU offload | full GPU offload |
| 실제 prompt | 2,957 tokens |
| 실제 output | 1,225 tokens |
| prompt eval | 1.4059591 s = **2,103.19 tok/s** |
| generation eval | 30.1853166 s = **40.58 tok/s** |
| observed VRAM | 약 7.2 GB |

같은 글의 CPU-only fallback은 12th Gen Intel Core i7-12700H를 사용한다. GPU run의 전체 motherboard/driver/Ollama build hash가 모두 고정되어 있지는 않으므로 이 수치를 VIA 실측값이라 부르지 않는다. 다만 Snapdragon/NPU가 아니라 **Windows consumer PC + RTX 4060 + exact Qwen3-8B Q4_K_M** 조합의 공개 관측이라는 점에서 현재 product planning에 더 직접적인 근거다.

## 4. Corroborating Reference — Intel + RTX 4070 Laptop

출처:
https://github.com/SearchSavior/OpenArc/issues/43

공개 환경:

| 항목 | 공개 값 |
| --- | --- |
| PC | Lenovo Yoga Pro 9 16IMH9 |
| CPU | Intel Core Ultra 9 185H |
| GPU | NVIDIA GeForce RTX 4070 Laptop |
| Runtime | Ollama 0.12.6 / CUDA |
| Model | `qwen3:8b-q4_K_M` |
| Input/output | 512 / 128 tokens |
| 반복 | 5회 |
| Avg TTFT | **0.19 s** |
| Avg prompt processing | **2,614.20 tok/s** |
| Avg decode | **44.92 tok/s** |
| Avg duration | **4.38 s** |

개별 5회 결과도 공개되어 있으며 TTFT 0.18~0.20s, decode 44.8~45.0 tok/s로 기록되어 있다.

이 장치는 desktop RTX 4070이 아니라 laptop GPU이므로 4060 profile을 대체하는 target hardware로 쓰지 않는다. 대신 Intel/Windows-class consumer system에서 **40~45 tok/s decode가 실제 관측된다는 sanity check**로 사용한다.

## 5. ASR-01 계산식

실제 serialized token count를 `Nin`, 해당 판단/응답에 필요한 실제 생성 token 수를 `Nout`이라 한다.

RTX 4060 planning profile의 공개 관측률을 그대로 사용하면:

```text
R_prompt = 2103.19 token/s
R_gen    = 40.58 token/s

T_response_start_model_hat ≈ Nin / R_prompt + 1 / R_gen
T_complete_model_hat ≈ Nin / R_prompt + Nout / R_gen
```

공개 자료가 prompt-eval과 generation을 별도 throughput으로 보고하므로 첫 token을 prompt rate에 포함시키지 않는다. 첫 유효 output의 근사치는 prompt processing + 약 1 generation token이며, structured output 완료는 Nout 전체 generation을 포함한다.

이는 **model inference subtotal**이다. 다음은 별도 span으로 추가한다.

- model/server request serialization
- Context access/materialization
- IPC/RPC
- model queueing
- validation / structured-output commit
- response composition
- Voice/Text delivery
- cache miss/model load가 실제 request path에 존재하면 그 비용

공개 prompt/generation rate에 이미 포함된 inference를 다시 더하지 않는다.

### 왜 예전의 `ceil(Nin/128) × TTFT_short`를 쓰지 않는가

Qualcomm profile의 128-token TTFT를 block-linear extrapolation하는 식은 해당 runtime의 한 공개 table을 이용한 approximation이었다. Windows consumer GPU에서 prompt-processing throughput을 직접 공개한 자료가 있으므로, 현재는 **실제 observed prompt tok/s와 generation tok/s를 사용하는 선형 subtotal**이 더 단순하고 target 환경과도 가깝다.

길어진 context, KV pressure, concurrent workload에서 속도가 달라질 수 있으므로 이 식을 runtime p95로 부르지 않는다.

## 6. 실제 benchmark를 나중에 할 때의 runtime/API contract

사용자가 지금 별도의 모델 서버나 API를 준비할 필요는 없다.

실행 검증이 필요해지는 시점에는 다음 하나의 reference runtime을 고정한다.

```text
OS          : Windows 11
Model       : Qwen3-8B Q4_K_M, 위 SHA 고정
Mode        : non-thinking
Runtime     : llama.cpp frozen release/commit + CUDA
Server      : llama-server
API         : local OpenAI-compatible HTTP endpoint
Concurrency : 1
Context     : 11-C에서 canonical workload와 함께 고정
Warm/cold   : 구분 측정
```

Architecture 후보가 내부적으로 다른 Model provider/runtime을 사용할 수 있어도 **평가용 Semantic Model dependency**는 동일 reference runtime/API fixture로 고정해 구조 차이와 모델 성능 차이를 섞지 않는다.

실제 hardware를 준비할 수 없는 단계에서는 위 public profile로 `ESTIMATED_MODEL_ONLY`를 계산하고, 후보 구조의 prompt size/call graph 차이를 비교한다. 이후 실제 PC를 확보하면 같은 prompt ledger와 call graph를 그대로 재생하여 `MEASURED` evidence로 교체한다.

## 7. Prompt token 원장

`benchmark/rebaseline/build_assets.py`의 prompt 초안은 grounding, refinement, compound, task association, Agent selection, handling, response composition, integrated judgment를 다룬다. 이는 반드시 8회 LLM을 직렬 호출해야 한다는 Architecture 제안이 아니다.

후보마다 실제 call graph를 먼저 정의하고 각 호출에 대해 다음을 기록한다.

- frozen tokenizer/template revision
- system prompt
- output schema
- conversation / Task / Context
- current request
- final serialized prompt SHA
- exact input token count
- expected/actual output token count
- dependency / parallelism / resource

공식 tokenizer 실행이 끝나기 전에는 token count를 추정 숫자로 채우지 않는다.

## 8. Evidence level

| 값 | 의미 |
| --- | --- |
| `SOURCE_OBSERVED` | 공개 source가 직접 측정한 값 |
| `ESTIMATED_MODEL_ONLY` | frozen token count + 공개 throughput으로 계산한 subtotal |
| `SYNTHETIC_REPLAY` | fixed fixture/runtime의 재생 시험 |
| `MEASURED` | 실제 VIA candidate/runtime에서 측정 |
| `NOT_RUN` | 아직 수행하지 않음 |

현재 Qwen evidence는 `SOURCE_OBSERVED`와 그에 기반한 planning equation까지이다. 실제 VIA candidate latency는 아직 `NOT_RUN`이다.
