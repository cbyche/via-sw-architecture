# Gate 1 — 근거 원장과 방법론 보정

> 조회·검토일: 2026-09-21. 이 문서는 외부 사실과 VIA의 제품 예산/분석 제안을 분리한다.

## 1. 직접 확인한 외부 근거

| ID | 1차 출처 / 고정 위치 | 이 자료가 뒷받침하는 범위 | 뒷받침하지 않는 범위 |
|---|---|---|---|
| E1 | SEI, Architecture Tradeoff Analysis Method Collection, 2018-02-14: https://www.sei.cmu.edu/library/architecture-tradeoff-analysis-method-collection/ | 품질 목표에 대한 architecture 평가와 목표 사이 상호작용/trade-off 분석 | VIA의 12개 ASR·3~4개 선정 수·score band·사후 선택 규칙이 ATAM 공식 규칙이라는 주장 |
| E2 | A2A specification **v0.3.0**, Task/streaming/push/AgentCard: https://a2a-protocol.org/v0.3.0/specification/ | 이질적 Agent 계약과 polling/stream/push capability의 존재 | 모든 Agent가 같은 기능을 지원한다거나 webhook이 개인 PC에 바로 도달한다는 가정 |
| E3 | AWS Prescriptive Guidance, Transactional outbox: https://docs.aws.amazon.com/en_en/prescriptive-guidance/latest/cloud-design-patterns/transactional-outbox.html | DB 기록과 외부 통지의 dual-write 문제, outbox의 역할 | 로컬 outbox commit=외부 Agent 접수, 또는 일반적인 exactly-once 외부 action 보장 |
| E4 | AWS Reliability, REL05-BP01: https://docs.aws.amazon.com/wellarchitected/latest/reliability-pillar/rel_mitigate_interaction_failure_graceful_degradation.html | 의존성 장애의 전체 기능 영향 축소 / degraded service 설계 | 모든 bulkhead 후보가 특정 VIA fault set에서 다른 점수를 낸다는 보장 |
| E5 | Microsoft Azure Architecture Center, Bulkhead: https://learn.microsoft.com/en-us/azure/architecture/patterns/bulkhead | 자원·실행의 격리와 장애 전파 범위라는 설계 관심사 | 프로세스만 나누면 공유 GPU/저장소/중앙 queue가 자동 격리된다는 주장 |
| E6 | NIST SP 800-63C-4, Privacy / Data Minimization: https://pages.nist.gov/800-63-4/sp800-63c/privacy/ | 필요한 최소 정보·derived attribute 전달의 privacy 방향성 | 문맥이 federation이므로 VIA에 직접 적용되는 법적 의무, 또는 W-11의 25% cutoff |
| E7 | Maslych et al., CUI 2025, arXiv **2507.22352**, abstract: https://arxiv.org/abs/2507.22352 | VR virtual-agent 실험에서 4초 초과 지연과 경험 저하, filler 효과를 보고 | PC VIA에 보편적인 2초/3초/1초 경계가 입증됐다는 주장 |
| E8 | Qwen3-Omni Technical Report, arXiv **2509.17765**, abstract: https://arxiv.org/abs/2509.17765 | 234ms를 theoretical first-packet latency로 명시 | 실제 PC/S2S 서버 하드웨어 p95, 사용자 소요시간, TTFT=응답 의미 완성 |
| E9 | LocalLLM.in, RTX4060 실제 benchmark 절, 게시 2025-11-30 / 갱신 2026-02-03: https://localllm.in/blog/ollama-vram-requirements-for-local-llms | 저자 보고 Windows11, Qwen3 8B Q4_K_M, 16k, input2957/output1225, prompt2103.19/decode40.58 tok/s | 제조사 공식 SLA, 동일 Intel CPU 확인, 공식 Qwen GGUF와 동일 artifact digest, 여러 반복의 tail 분포 |

E9는 직접 수행했다고 보고한 공개 사례이지만 세부 runtime build/model hash와 반복 분포가 불완전하다. 기존 source 숫자를 재확인했으며 **p95나 공식 artifact의 완전 일치 증거로 승격하지 않는다**. 같은 표에 있는 CPU-only i7 결과를 GPU run의 CPU metadata로 복사하지 않는다.

## 2. 제품 예산과 외부 사실의 분리

| W | 목표 근거 유형 | 판단 |
|---|---|---|
| W-01 | 기존 대화 반응 목표 2s 계승 + 제품 예산 | E7은 방향성만 보조. 분해 후 다른 분모이므로 과거 score 이관 금지. |
| W-02 | **신규 제품 제안 3s** | 즉시 인사보다 실제 접수 확정을 중시. 모델+context+handoff 예산이며 선행 후보 숫자에서 역산하지 않음. |
| W-03 | **신규 제품 제안 1s** | 이미 준비된 질문/결과를 지체 없이 전달하는 경로의 예산. |
| W-04 | **신규 제품 제안 1.25x** | 4 active Task로 늘어도 전경 저하 25% 이내. 자연 사용자 통계·보편 상수 아님. |
| W-05/06 | 기존 95% 검토값 계승 | TC 동일가중 만족도의 허용 잔여 미달 5 percentage points. 실제 전체 성공확률/19 of20 단순 총계와 다름. |
| W-07/08 | **사용자 승인 2/3** | 변경 국소화의 제품 예산. 특정 adapter 구조가 정답이라는 뜻 아님. |
| W-09 | **신규 제품 제안 5s** | 일시 종료 후 업무 제어의 빠른 회복. 무조건 더 낮은 시간을 위해 correctness를 희생하지 않음. |
| W-10 | fixed **28-cell** unaffected suite의 100% 목표 | 24 external-dependency cells + 4 integration-host fatal-fault cells. 관련 없는 capability는 계속 제공해야 한다는 제품 요구의 운영화이며 실제 가용성 확률 아님. |
| W-11 | **신규 제한공유 예산 25%=5/20** | 전체 허용 corpus를 그대로 외부로 보내지 않고 제한된 scope로 처리할 목표. 정보단위/필요량의 타당성은 Gate 1/2 검토 대상. |
| W-12 | 기존 0/24 | 알려진 안전 기회에서 위반을 허용하지 않는 목표. score-only 비교이며 출시 인증 아님. |

각 score band는 제품 목표 대비 단계적 여유/미달을 표현하는 프로젝트 규칙이다. 논문이나 표준이 전체 band를 검증해줬다고 말하지 않는다. 특히 W-02/03/04/09/11은 새로운 제품 예산이므로 승인값으로 위장하지 않는다.

## 3. 이번에 바로잡은 비교 오류

1. **Handoff 종료점:** 로컬 enqueue와 Agent accepted를 OR로 선택하면 queue형 구조가 아직 업무를 넘기지 않고도 빠른 점수를 받는다. 공통 AND endpoint와 restart-safe correlation을 사용한다.
2. **p95 오명칭:** 공개 평균 throughput으로 얻은 model 소계에는 반복 분포가 없다. system p95 score와 분리한다.
3. **N/A 남용:** 구조가 이 QA를 바꾸지 않는다는 뜻은 metric 자체가 없다는 뜻이 아니다. held-constant/미측정과 N/A를 구분한다.
4. **추가 구현량 혼입:** 기본 구조의 Component 개수는 W-08이 아니다. 15개 고정 변화에서 수정·추가·제거되는 요소만 센다.
5. **결과 후 Primary 선정 편향:** 규칙을 사전 작성해도 사후 축 선택의 탐색성이 사라지지는 않는다. 전수 결과·prior relevance를 유지하고 별도 확인 반복/원장 교차 검토를 한다.
6. **Privacy bytes/단위 임의화:** compressed payload와 remote-readable handle을 0 노출로 취급하지 않는다. unit corpus와 의미 annotation을 잠근다.
7. **Coupled DP:** 서로 결합된 DP들의 개별 승자를 합친다고 최종 최적이 되는 것은 아니다. Gate 2에 교차 확인 계획을 포함한다.
8. **On-device baseline과 Privacy 인과:** CTX-DP02의 Model-facing context 크기는 on-device Model 기준에서 그 자체로 remote exposure가 아니다. 따라서 W-11을 primary causal hypothesis에서 내리고 외부 Agent egress 재사용 시 regression으로만 확인한다.
9. **Process-isolation DP와 fault stimulus:** external connection-refused/no-reply와 whole-VIA restart만으로는 EXEC-DP01의 process fault boundary 차이를 충분히 자극하지 못한다. 후보 결과 전에 integration-host fatal fault를 W-09/W-10 공통 fixture에 추가한다.

## 4. 이 시점의 증거 완성도

완료: repository 최신 HEAD 검증, Gate 1 **9개 DP/25개 주제 최종 분류**, 18UC·24change·18RC 연결, 12개 평가 계약의 작성. 기존 계산도우미는 7-ASR 기준선 자산이며 Working-12 machine-readable fixture는 Gate 2 실행자산으로 별도 materialize해야 한다.

미수행: 실제 VIA/Agent adapter 실행, Qwen 모델 inference, candidate별 exact prompt tokenization, 실제 Windows 음성·화면 계측, 후보 latency/accuracy/승자 산출. 기존 generator/실험의 전체 regression을 이번 환경에서 다시 실행했다고 주장하지 않는다.

현재 기준선 변경은 GitHub connector를 통해 `architecture-rebaseline-20260918` branch에 직접 반영한다. Working-12의 machine-readable fault fixtures와 candidate adapter는 아직 실행 자산으로 생성하지 않았으며 Gate 2에서 후보/E-ID와 함께 동결한다.