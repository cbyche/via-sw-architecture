# QA-01~QA-03 근거 원장 — Target Mac / Local Qwen3-8B profile

> 기준일: 2026-09-26
> 상태: **Local runtime smoke-tested / 공식 QA benchmark NOT_RUN**
> 목적: VIA semantic LLM을 실제 평가 target에서 동일하게 실행할 model artifact와 runtime contract를 고정한다.

## 1. Frozen model

| 항목 | 고정값 |
| --- | --- |
| Model | `Qwen/Qwen3-8B-GGUF` |
| Quantization | `Q4_K_M` |
| Cached repository revision | `7c41481f57cb95916b40956ab2f0b139b296d974` |
| GGUF SHA-256 | `d98cdcbd03e17ce47681435b5150e34c1417f50b5c0019dd560e4882c5745785` |
| Runtime | `llama.cpp` 0.5.0 build 11146, Metal |
| Mode | non-thinking |
| Context | 16,384 tokens |
| Endpoint | local OpenAI-compatible HTTP, `127.0.0.1:18080` |

Official sources:

- https://huggingface.co/Qwen/Qwen3-8B
- https://huggingface.co/Qwen/Qwen3-8B-GGUF

모든 Component와 Task는 이 semantic LLM 한 instance를 공유한다. Prompt·schema·session과 호출 graph는 후보 구조에 따라 달라질 수 있으나 모델을 추가 적재하지 않는다.

## 2. Target device

| 항목 | 값 |
| --- | --- |
| PC | MacBook Air 15-inch, model identifier `Mac17,4` |
| SoC | Apple M5, 10-core CPU(4P+6E), 10-core GPU |
| Memory | 24 GB unified memory |
| OS | macOS 26.5.1 (25F80) |
| Power | AC power; scored run마다 상태 기록 |

이 장비가 VIA Architecture 평가의 target이다. 다른 장비 수치로 보정하거나 환산하지 않는다.

## 3. 설치와 기동

Homebrew의 `llama.cpp`를 설치하고 공식 GGUF를 한 번 받았다. 이후 실행은 SHA-256을 확인하는 저장소 script를 사용한다.

```bash
brew install llama.cpp
llama-server -hf Qwen/Qwen3-8B-GGUF:Q4_K_M
scripts/architecture/start_local_semantic_model.sh
```

모델 파일은 local Hugging Face cache에 있으며 Git에 포함하지 않는다. `VIA_SEMANTIC_MODEL_PATH`로 같은 SHA-256의 다른 위치를 지정할 수 있다.

## 4. 2026-09-26 smoke evidence

JSON schema로 `intent`, `target`, `delegate`를 요구한 한국어 의미판단 요청이 HTTP 200과 유효 JSON으로 반환됐다.

| 항목 | 관측값 |
| --- | ---: |
| Prompt tokens | 43 |
| Output tokens | 37 |
| Prompt processing | 374.845 ms, 114.71 tok/s |
| Generation | 1,906.335 ms, 18.88 tok/s |
| HTTP total | 2.289 s |

Evidence label은 `MEASURED_MODEL`이다. 이 값은 한 번의 기능 smoke test이므로 p95, QA target 충족, VIA E2E 또는 A/B 차이를 주장하지 않는다.

## 5. 공식 benchmark contract

QA-01~QA-03에서 model 시간을 추정식으로 대체하지 않고 실제 호출 trace로 측정한다. 각 호출은 다음을 기록한다.

- model/runtime revision과 GGUF SHA-256
- serialized prompt SHA-256, exact input/output token 수
- system prompt, output schema, Conversation·Task·Context version
- queue start/end, inference start, first meaningful output, structured completion
- retry/repair와 shared-model contention
- call dependency와 sequential/parallel 관계

Network, IPC, Context access, validation, speech generation, playback queue와 device onset은 model span 밖의 별도 span으로 유지한다. Model load를 scored request path에 포함하는 cold case와 미리 적재한 warm case를 구분한다.

## 6. Evidence 상태

| 주장 | 상태 |
| --- | --- |
| Frozen GGUF가 target Mac에서 적재된다 | `MEASURED_MODEL` |
| 한국어 structured semantic output을 반환한다 | `MEASURED_MODEL` smoke evidence |
| QA workload의 semantic 정확도·p95 | `NOT_RUN` |
| DP A/B의 QA-01~03 차이 | `NOT_RUN` |
| 실제 Voice user-experienced latency | `NOT_RUN` |

Prompt corpus, repetition, run order와 failure treatment는 [VIA Core Evaluation Profile](../../11-measurement/evaluation-profile.md)과 machine-readable freeze에서 확정한다.
