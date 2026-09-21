# 12-03. Implementation & Test Readiness Plan

> 2026-09-22 / Gate 2 승인 후 실행 계획.
> 대상: IR-DP01, TASK-DP01, AGENT-DP01, EXEC-DP01.
> 고정 원칙: FP-INT01 S2S Direct Fast Path, TASK-T01 Event-first + Query Reconciliation.
> 목적: 발표용 숫자를 만들기 위한 구현이 아니라 **동일 조건에서 두 Architecture가 실제로 어떤 trade-off를 보이는지 검증**하는 최소 executable prototype을 만든다.

## 1. 구현 순서

### Phase 0 — Deterministic structural harness

실제 Model/실제 Agent 없이 먼저 구현한다.

- 공통 Rust workspace와 trace schema
- deterministic Agent P/Q simulator
- Context/Policy/Repository fixture
- 94 canonical TC replay adapter
- W-02/W-03/W-04/W-09/W-10 span/fault instrumentation
- M/A/C 24 change ledger skeleton
- process-fatal fixture와 restart controller

이 단계는 사용자 GPU/API/실제 Agent가 필요 없다.

### Phase 1 — 3개 non-semantic DP executable prototype

우선 TASK-DP01, AGENT-DP01, EXEC-DP01 A/B를 구현한다.

- TASK: shared transactional service vs durable per-Task supervisor
- AGENT: edge normalization vs Core-visible typed contracts
- EXEC: one-process Tokio runtime vs child integration worker + Windows IPC

deterministic Agent fixture로 기능·recovery·concurrency·failure-isolation과 design-analysis change count를 검증한다.

### Phase 2 — IR-DP01 실제 Model 실험

같은 frozen Qwen3-8B reference Model로:

- Integrated Semantic Authority prompt/schema
- Staged Semantic Authorities prompt/schema
- 같은 canonical input/context
- actual token ledger / call graph / latency
- W-05 Completion obligation 결과

를 비교한다. Model accuracy 차이는 Architecture가 미리 정한 결과가 아니라 empirical evidence로만 사용한다.

### Phase 3 — combined sensitivity checks

개별 DP에서 결론을 뒤집을 수 있는 작은 조합만 확인한다.

- IR × AGENT
- TASK × EXEC

전체 2^4 조합을 무조건 실행하지 않는다.

## 2. 사용자 준비물 — 지금 필요한 것 / 나중에 필요한 것

### 지금 당장 필요한 것: 없음

Phase 0/1용 code, fixture, tests, GitHub Actions 가능한 부분은 repository에서 준비한다. 실제 downstream Agent, S2S server, API key는 필요 없다.

### 실제 Windows 구조 성능 측정 전에 필요한 것

1. VIA target에 가까운 **Windows 11 PC 1대**
2. 그 PC에서 repository clone/build/run 권한
3. Rust toolchain은 우리가 exact version을 freeze한 뒤 설치
4. 장비 정보 수집 허용:
   - CPU
   - RAM
   - GPU
   - Windows build
   - storage type
5. background workload를 최소화한 repeatable test window

GPU는 TASK/AGENT/EXEC 구조 시험 자체에는 필수가 아니다. hosted CI의 기능 검사는 가능하지만 p95 performance 숫자는 noisy hosted runner를 대표 장비 실측으로 사용하지 않는다.

### IR-DP01 실제 Model 시험 전에 추가로 필요한 것

1. NVIDIA GPU가 있는 reference Windows PC
2. frozen Qwen3-8B Q4_K_M artifact
3. frozen tokenizer/template
4. frozen llama.cpp/llama-server build
5. local model server 실행 권한

현재 planning evidence는 RTX 4060급 consumer GPU profile이지만, **사용자 장비가 반드시 RTX 4060이어야 한다는 뜻은 아니다.** 실제 평가 장비를 한 번 정하면 A/B 모두 동일 장비·artifact·runtime을 사용하고 manifest에 기록한다.

### 지금 필요하지 않은 것

- Qwen3-Omni S2S 실행 환경: INT-DP01을 score DP에서 내렸으므로 현재 4-DP 검증의 선행조건이 아니다.
- 실제 production downstream Agent: AGENT/TASK 비교는 먼저 deterministic P/Q fixture로 수행한다.
- cloud API key: 초기 구조 비교에 사용하지 않는다.

## 3. 실제 PC 실측을 ChatGPT와 연결하는 방법

현재 이 채팅 환경은 사용자의 로컬 Windows PC에서 직접 명령을 실행하지 않는다. 따라서 repository에 **one-command runner와 result bundle**을 만든 뒤 사용자가 로컬에서 실행하고 결과 JSON/trace를 GitHub branch에 commit하거나 이 대화에 업로드하는 방식이 가장 재현 가능하다.

예정 명령 형태:

```powershell
./scripts/bootstrap-test-env.ps1
./scripts/run-structural-bench.ps1
./scripts/run-ir-model-bench.ps1
```

실행 결과에는 hardware/runtime/model/config hash와 raw trial만 저장하고 score는 동일 scoring tool로 후처리한다.

## 4. 증거 등급

| 단계 | 증거 등급 | 사용 가능 주장 |
|---|---|---|
| C/I/S/D 및 change ledger | DESIGN_ANALYSIS | 구조 변경 범위, 예상 dependency |
| deterministic fixture 실행 | REPLAY / MEASURED_STRUCTURAL | Task/Agent/process 상태·latency·fault 처리 |
| 실제 Qwen 실행 | MEASURED_MODEL | 해당 model/corpus에서 IR 정확도·model-inclusive latency |
| 실제 target PC 반복 측정 | MEASURED_SYSTEM | 해당 장비/config의 p95 |
| 공개 benchmark 추정 | ESTIMATED_MODEL_ONLY | planning subtotal만 |

서로 다른 증거 등급을 하나의 ‘실측값’처럼 섞지 않는다.

## 5. 실행 전에 제가 먼저 완료할 작업

1. 현재 Gate 2 validator를 4-DP 기준으로 실제 실행 가능한 상태로 정합화하고 unit test 재실행.
2. Rust workspace / common contracts / trace collector 생성.
3. Agent P/Q simulator 및 TASK/AGENT/EXEC A/B 구현.
4. Windows process worker + named-pipe IPC prototype.
5. fault/restart/concurrency driver 작성.
6. IR integrated/staged prompt/schema를 candidate 결과 전에 freeze.
7. one-command PowerShell runner와 result manifest 작성.
8. actual measurement 전 GitHub revision/tag를 고정.

이 단계까지는 사용자 환경을 요구하지 않고 진행할 수 있다.
