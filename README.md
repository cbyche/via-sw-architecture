# VIA Software Architecture

이 저장소는 Samsung PC용 **Voice Interaction Agent(VIA)**의 소프트웨어 아키텍처를 정의하고 검증한다. VIA는 사용자 PC에서 Voice·Text·화면 interaction을 하나의 대화로 관리하고, 직접 답할 수 있는 요청은 처리하며, 실제 업무는 Downstream Agent에 위임한 뒤 진행 상황과 결과를 다시 사용자에게 연결한다.

이 과제의 핵심은 특정 모델이나 Agent를 고르는 것이 아니다. **VIA 내부의 책임·상태·계약·process boundary를 어디에 둘지** Decision Point(DP)별 A/B 대안을 만들고, 동일 조건의 측정으로 trade-off를 비교해 근거 있는 Architecture Decision을 남기는 것이다.

## System boundary

| 영역 | 책임 |
| --- | --- |
| VIA | Voice/Text interaction, 화면·대화 Context, direct response, 요청·Agent orchestration, Conversation/Task 상태, progress/result 전달 |
| Downstream Agent | 업무별 reasoning, planning, tool 선택·실행, 외부 업무 상태 변경 |
| AI Model Runtime | S2S와 VIA semantic inference에 사용하는 local/remote dependency. 연결 구조는 범위 안, 모델 내부 구현·학습은 범위 밖 |
| 제품 범위 밖 | Downstream Agent 내부 모델의 성능과 실행시간, VIA가 직접 수행하지 않는 외부 업무 구현 |

정확한 경계는 [System Mission & Boundary](docs/architecture/01-system-mission-and-boundary.md)가 기준이다.

## Current architecture work

현재 비교하는 Core Decision Point는 네 개다.

| DP | Architecture question | Current decision state |
| --- | --- | --- |
| IR-DP01 | semantic authority를 통합할 것인가, 단계별로 분리할 것인가 | Deferred; A는 interim reference |
| TASK-DP01 | Task 상태를 공유 service가 소유할 것인가, Task별 supervisor가 소유할 것인가 | B accepted; 새 Voice metric으로 재검증 필요 |
| AGENT-DP01 | Agent 차이를 경계에서 canonical contract로 정규화할 것인가 | A accepted |
| EXEC-DP01 | integration runtime을 단일 process에 둘 것인가, process 격리할 것인가 | B accepted; 새 Voice metric으로 재검증 필요 |

각 DP는 **다른 DP 조건을 고정한 A/B paired comparison**으로 평가한다. 16개 조합의 global winner를 먼저 고르는 방식이 아니며, full-factorial 분석은 interaction 확인을 위한 secondary analysis일 뿐이다.

## Current measurement status

현재 단계는 **Measurement Contract Definition**이다. QA catalog 자체를 사용자와 재검토하고 있으며, 아직 Candidate Implementation이나 A/B Measurement를 수행하는 단계가 아니다.

| Item | Status |
| --- | --- |
| System boundary, representative use cases, change scenarios | Defined |
| QC taxonomy | QC-01~QC-10 defined |
| QA catalog | **User-review draft**; category ranges and 19 active single-metric QA definitions recorded |
| ASR classification | **Not selected; QA별 Architecture 영향 검토 전** |
| QA-01~QA-05 event and software-boundary contract | Draft defined; machine freeze pending |
| QA-05 Task control contract | Draft defined; machine freeze pending |
| QA-11~15, QA-31/32/41 semantic contract | Draft defined; machine freeze pending |
| QA-61/62 observability contract | Draft defined; machine freeze pending |
| Machine-readable contract and harness | **Not implemented** |
| Current QA results | **Not run** |
| Actual S2S/VIA LLM/product latency evidence | **Not measured** |
| Previous measurement code and results | Historical/superseded archive |

새 Voice responsiveness 정의는 다음과 같다.

- **QA-01 — Delegated Path VIA Responsiveness:** Agent 실행시간을 제외하고, Voice 요청의 위임 준비와 Agent 결과의 Voice 전달에 VIA가 소비한 시간
- **QA-02 — VIA Direct Voice Response Responsiveness:** Downstream Agent 없이 VIA가 직접 답하는 Voice 요청의 전체 반응시간
- **QA-03 — Agent Progress Voice Feedback Responsiveness:** Agent status가 source에서 제공 가능해진 뒤 사용자에게 audible Voice로 전달되기까지의 시간
- **QA-04 — Voice Interruption Responsiveness:** 사용자의 실제 barge-in 발화 시작부터 중단 대상 음성의 마지막 audible sample까지의 시간
- **QA-05 — Task Control Responsiveness:** 사용자 제어 입력 종료부터 올바른 Task 처리 상태가 보이거나 들릴 때까지의 시간

QA-01~04의 Voice endpoint, 포함·제외 구간, S2S 234 ms와 VIA LLM 추정치의 허용 범위는 [Voice Responsiveness](docs/architecture/08-quality-attributes/voice-responsiveness.md)를 따른다. QA-05는 [Task Control Responsiveness](docs/architecture/08-quality-attributes/interaction-control-responsiveness.md)를 따른다. 과거 harness의 수치나 이름을 현재 결과로 해석하면 안 된다.

## Start here

새로 참여한 사람과 LLM은 아래 순서로 읽는다.

1. [Architecture baseline guide](docs/architecture/README.md) — 전체 문서의 위계와 현재 상태
2. [System Mission & Boundary](docs/architecture/01-system-mission-and-boundary.md) — VIA가 무엇이고 무엇이 아닌지
3. [Fixed Architecture Scope](docs/architecture/03-fixed-architecture-scope.md) — 모든 후보가 공통으로 만족할 범위
4. [Representative Use Cases](docs/architecture/05-representative-use-cases.md) — 평가할 사용자 상황
5. [Quality Model](docs/architecture/08-quality-attributes/quality-model.md) — QC·QA·ASR의 관계와 현재 QA catalog
6. [Voice Responsiveness](docs/architecture/08-quality-attributes/voice-responsiveness.md) — 현재 QA-01~QA-04 정의
7. [Task Control Responsiveness](docs/architecture/08-quality-attributes/interaction-control-responsiveness.md) — 현재 QA-05 정의
8. [Observability](docs/architecture/08-quality-attributes/observability.md) — 현재 QA-61/62 정의
9. [Event & Boundary Contract](docs/architecture/11-measurement/event-boundary-contract.md) — 실제 사건과 software 진단 event의 경계
10. [Measurement guide](docs/architecture/11-measurement/README.md) — 결과 전 동결해야 할 계약과 evidence level
11. [Architecture decisions](docs/architecture/12-decisions/README.md)과 [ADRs](docs/adr/README.md) — DP 대안, 판단 방법, 현재 결정

저장소를 수정하는 LLM은 먼저 [AGENTS.md](AGENTS.md)를 읽어야 한다.

## Repository map

| 경로 | 현재 역할 |
| --- | --- |
| [`docs/architecture/`](docs/architecture/README.md) | 유일한 active Architecture 기준선 |
| [`docs/adr/`](docs/adr/README.md) | 승인·유예된 Architecture Decision Record |
| [`docs/references/`](docs/references/README.md) | 출처와 참고자료; 요구사항이나 결정 자체는 아님 |
| [`benchmark/architecture/`](benchmark/architecture/README.md) | 다음 measurement contract와 harness의 active 위치 |
| [`prototypes/candidates/`](prototypes/candidates/README.md) | Architecture 후보 구현 |
| [`results/architecture-evaluation/current/`](results/architecture-evaluation/current/README.md) | 현재 계약으로 생성된 evidence만 저장할 위치 |
| [`scripts/architecture/`](scripts/architecture/README.md) | active 문서·구조 검증 도구 |
| [`docs/archive/`](docs/archive/README.md), [`benchmark/archive/`](benchmark/archive/README.md), [`results/gate2/archive/`](results/gate2/archive/README.md) | 퇴역한 세대와 historical evidence; `gate2`는 과거 campaign 이름일 뿐 현재 단계명이 아님 |

## Evidence interpretation

- `NOT_IMPLEMENTED`와 `NOT_RUN`은 결과가 없다는 뜻이다. archive 수치로 빈칸을 채우지 않는다.
- 계산값, scheduled mock/reference 측정, 실제 모델 측정, 제품 end-to-end 측정을 같은 evidence로 취급하지 않는다.
- Qwen3-Omni의 공개 234 ms는 제한된 조건의 theoretical first-audio-packet reference다. QA metric 시작점이나 제품 latency가 아니다.
- VIA LLM token-rate 계산은 `ESTIMATED_MODEL_ONLY`; 실제 component span과 섞어도 `HYBRID_REFERENCE_ESTIMATE`다.
- 유효 endpoint는 무음·earcon·filler가 아니라 첫 **meaningful audible** onset이다.
- 실제 audible latency 주장은 speaker/loopback 수준의 onset 관측이 있어야 한다. instrumented sink 도착과 동일하지 않다.

## Development

`architecture-ci`는 `main` 변경마다 active 문서 정합성과 candidate prototype을 검사한다. `candidate-review-bundle`은 외부 리뷰용 source·lock·test output 묶음이 필요할 때만 수동 실행한다. 두 workflow 모두 QA 측정 campaign이나 Architecture 승자 결정을 수행하지 않는다.

```bash
.venv/bin/python scripts/architecture/check_active_markdown_links.py
.venv/bin/python scripts/architecture/check_active_terminology.py
.venv/bin/python scripts/architecture/check_qa_catalog.py
cargo +1.98.1 fmt --manifest-path prototypes/candidates/Cargo.toml --all -- --check
cargo +1.98.1 test --locked --manifest-path prototypes/candidates/Cargo.toml --workspace --all-targets
```

작업 규칙은 [AGENTS.md](AGENTS.md), 사람 기여 절차는 [CONTRIBUTING.md](CONTRIBUTING.md)를 따른다.
