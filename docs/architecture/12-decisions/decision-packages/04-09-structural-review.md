# 4단계 — 구조 탐색 검토 기록

> 상태: **독립 검토, 수정 및 재확인 완료 / 사용자 리뷰 대기** / 2026-10-01
> 대상: [04-00 전체 지도](./04-00-structural-alternatives.md)와 04-01~06 개별 구조 설명 및 비교 그림.
> §2~4는 최초 04-00 검토 기록이며, 개별 문서와 그림으로 정리한 후속 리뷰는 §5에 있다.

## 1. 범위와 진행

사용자는 03 리뷰 뒤 04 진행을 승인하고, 품질 우선순위와 직결되는 P에 탐색 노력을 집중하도록 지시했다. 작성자는 승인된 02, 현재 03의 13개 관점, 실제 target의 구조, 제어와 수명, 기억, shared Omni 및 완결성 문서를 대조했다. 기존 네 비교 자료는 재평가 입력으로만 사용했다.

P-01/02/03/04/05/06/14를 집중 탐색하고, 중첩된 메커니즘을 6개 S 질문으로 정리했다. Target 밖 대안은 8개이며 일부 기능을 줄인 세 안도 포함했다. 다른 P에는 연계 내용과 별도 구체화 조건을 남겼다. 이는 새로운 DP 선발이나 전체 설계 공간을 망라했다는 선언이 아니다.

이 절과 §2는 초안 시점의 작성자 자기 검토다. 사용자 요청으로 수행한 별도 에이전트의 독립 검토와 수정 후 재확인은 §3에 기록한다. 초안 작성과 링크 검사가 독립적인 설계 리뷰를 대신하지 않는다.

## 2. 작성자 대조와 보완

| 대조 항목 | 확인한 문제 또는 과장 위험 | 처리와 재확인 |
| --- | --- | --- |
| 요구와 기능 손실 | 02의 완전 지원 전제를 되살려 제한된 대안을 사전 탈락시킬 위험 | 이후 사용자 합의가 탐색에 우선함을 명시. S-02 B의 자연 지칭, S-04 B의 요청과 질문 복구, S-05 A의 S2S 직접 기능 손실을 V-02/03에 기록 |
| P별 노력 | 모든 P에 V-03/04/05가 연결됐다는 이유로 균등 탐색할 위험 | §2에 실제 인과와 깊이 기록. P-08/09/15는 낮은 중요도가 아니라 실행 경로를 공유하는 연계 대상으로 표시 |
| S-01 target 표현 | target의 다음 semantic 호출이 항상 새 model session이라는 근거 없는 추정 가능 | ‘host가 해석 제안과 다음 호출을 번갈아 조정’으로 수정. 대안의 차이는 임시 continuation과 읽기 실행 소유권 |
| S-02 생산 비용 | 사전 객체 이력을 무료로 얻거나 객체 ID를 source의 영구 ID로 간주할 위험 | producer, 파생 저장, background vision, 제한된 raw fallback과 gap 명시. 객체 추적과 시간 정렬은 미확인 |
| S-03 target 약화 | 기존 target에 index/cache/summary가 없는 것처럼 비교할 위험 | 현재 metadata/keyword index와 cache를 명시. 새 helper, 갱신, 삭제와 fallback 비용 포함 |
| S-04 권위와 crash | workflow 이름만 추가하고 기존 graph도 별도 writer로 남기거나 signal/activity를 비내구 호출로 연결할 위험 | continuation과 질문 원본을 runtime으로 이동. 로컬 signal 수락, 전이, intent transaction과 host admission 재검사 명시. Task와 외부 효과 권위는 별개 |
| S-05 출력 | target이 전체 답을 기다린다고 가정하거나 Core-only를 같은 S2S 기능이라고 부를 위험 | target의 문장 단위 출력 가능성 보존. S2S 미지원과 publication/실제 전달의 비원자성 명시 |
| S-06 음성 | native event adapter가 없는 시각을 만들어내거나 일치율을 정확성으로 오인할 위험 | 실제 native capability 미확인, 오류 상관, shared runtime 장애와 지속 인식 제한 명시. target도 이미 Omni에 원음을 보낸다는 사실 보존 |
| 전역 QA와 시간 | 질문별 진단이 다른 QA 정의가 되거나 외부 Agent 시간이 VIA 성능에 포함될 위험 | 모든 S의 13개 관점 검토. 공통 정의와 측정 계약 원칙 유지. 미지원 안내와 최종 결과 전달, 사람 및 Agent 대기를 구별 |
| 기존 네 자료 | 기존 네 안을 자동 재선발할 위험 | 세 안은 해당 S에서 재평가. 복구 journal은 연계 및 추가 탐색 대상으로 남김. 이전 QA 표와 DP 번호 승계 없음 |

## 3. 독립 검토

### 3.1 검토 방식과 결과

사용자가 독립 에이전트 검토와 수정 후 commit/push를 명시적으로 요청했다. 초안 커밋 `c0f9600021820b65b06a647eca86fff999c88e66`을 기준으로 별도 에이전트 세 명이 읽기 전용 검토를 수행했다. 작성자가 발견사항을 통합하고 수정했으며 같은 검토자들에게 실제 수정문을 다시 확인하도록 요청했다.

| 독립 검토 | 담당 범위 | 1차 결과 |
| --- | --- | --- |
| target_review | 실제 target 대조, 실행과 상태, 실패 및 복구 계약 | P1 없음, P2 2건. S-05 공통 Core 의존성, S-02 관측 process 감소 표현 |
| alternatives_review | 우선순위에 따른 탐색 충분성, 강한 대안 누락, 실질 구조 차이 | P1 없음, P2 1건. S-01의 교체되는 실행 계약 불명확. P-08/09/15의 연계 분류를 뒤집을 중대한 누락은 찾지 못함 |
| quality_review | 8개 대안의 13개 관점, 기능 손실, 시간 귀속, 공통 QA 의미 | P1 없음, P2 1건. S-05의 ASR 및 Core 공통 의존성. 시험 경계와 공존성 설명 보완 권고 |

중복을 합치면 **P2 발견사항은 3건**이다. 검토자의 같은 지적을 여러 독립 결함처럼 세지 않는다. 기능을 일부 포기한 세 안은 사용자 합의에 따라 그 자체를 결함으로 처리하지 않았다.

### 3.2 지적과 반영

| ID | 문제와 반례 | 수정 |
| --- | --- | --- |
| IR-01 / P2 | S-05가 정규 전사와 Core 전체 장애를 Core-only 대안에 새로 생긴 불이익처럼 설명했다. Target S2S도 정규 전사 및 input_echo 대조, Request Controller admission과 Response Manager 게시를 요구하므로 Core 종료는 양안에 영향을 준다. | S-05 V-01/04/07에 공통 ASR와 host 확정을 명시했다. VoiceProposal 기반 판단과 Request Interpreter의 semantic 처리 차이, semantic → SpeechRender 의존성, 국소 semantic 실패를 분리했다. 실제 국소 장애 격리는 미확인이다. |
| IR-02 / P2 | S-02 B의 ‘관측 process 감소’는 target의 UI/Core 배치와 맞지 않는다. 연속 화면 관측을 제거해도 UI/Core process 자체는 남는다. | V-07을 연속 관측 작업과 임시 상태 감소로 수정하고 UI/Core의 장애 경계를 보존했다. 선택 근거 손실과 자연 연속 지칭 미지원은 그대로 표시했다. |
| IR-03 / P2 | S-01에서 동일한 generation, 읽기와 Context를 그대로 두고 loop만 Worker로 옮기면 ‘host 왕복 감소’는 실제 구조 이익이 아니다. Target에도 session/KV가 있다. | 중간 proposal 종료 및 host 재조립/재호출 경계를 `READ_REQUIRED`, 재개 handle, 근거 주입과 실행 재개 계약으로 바꾸는 스케치를 명시했다. 연산 양보, 유한 KV, handle 무효, 재시도 예산을 설명했다. 단순 wrapper는 구조 효과의 근거가 없고 실제 runtime 지원은 미확인이라고 밝혔다. 요약, V-04/05/06과 가설이 약해지는 조건을 동기화했다. |

명료화 권고도 함께 반영했다. S-01은 같은 input revision의 누적 읽기 및 연산 예산과 deadline을 재시도로 초기화하지 않는다. 각 대안의 V-09에는 분석용 기록과 별도로 지연, 사건 순서 또는 장애를 통제할 시험 경계를 적었다. S-04 A의 V-11에는 timer와 signal/activity 재개가 다른 PC 앱과 CPU 및 IO를 공유하는 공존성 비용을 보완했다. 시험 도구를 구현하거나 실행했다는 뜻은 아니다.

### 3.3 탐색의 충분성과 재검토

독립 검토는 기존 네 자료의 반복에 갇히지 않았고 S-02~06에 실제 데이터 생산, 실행 경로, 상태 권위 또는 복구 범위 차이가 있음을 확인했다. P-08은 연결된 S의 정정 및 전송 경계에서 검토한다. P-09의 client Delivery Agent와 P-15의 제공자 전용 실행 package는 추가 탐색 가능성이지만, 현재 연계 분류와 재검토 조건을 뒤집을 중대한 누락 근거는 발견하지 못했다. 모든 P에 같은 대안 수를 만들거나 기존 event-sourcing 안을 자동 승계하지 않는다.

수정된 04-00의 실제 본문과 diff를 세 검토자가 다시 읽었다. target_review는 P2 2건, alternatives_review는 P2 1건, quality_review는 P2 1건의 해결을 각각 확인했다. **중복을 합친 P2 3건 모두 해결됐고, 최종 재검토에서 미해결 P1/P2 또는 새 중대 결함은 보고되지 않았다.** 독립 검토에서 결함을 찾지 못했다는 사실은 설계 공간을 망라했다거나 runtime 지원, 품질 우위가 입증됐다는 뜻이 아니다. S-01은 특히 미검증 실행 계약을 전제로 한 탐색안이며 강한 방안 2로 확정하지 않았다.

## 4. 최초 04-00 검사와 전달

| 검사 | 결과 |
| --- | --- |
| Active Markdown 링크 | PASS — active Markdown 113개. 새 04 문서에 archive 규범 참조 없음 |
| Active 용어 | PASS — canonical target Component 명칭과 기존 의미 유지 |
| QA catalog | PASS — 기존 QA registry와 4개 ASR 지위 보존 |
| 탐색 및 품질 추적 | PASS — P-01~16 배분, 집중 P 7개, S anchor 6개, 각 S의 V-01~13 순서와 이름이 03과 일치. A/B가 있는 두 표를 포함해 대안 8개 검토 |
| 표, anchor와 표현 | PASS — 표 열 수, 새 local anchor와 대상 문서 anchor 확인. 새 04 문서에 가운데점 없음 |
| 변경 범위와 diff | PASS — 새 04 문서 두 개와 AGENTS, 작업 계획, README만 변경. 공백 오류 없음 |

위 검사는 문서 일관성 확인이다. 대안의 충분성, 모델 지원, 실제 품질 우위를 실증하지 않는다. 독립 검토에 따른 수정 후 같은 검사를 다시 수행해 통과했다.

Git 전달은 이 기록을 포함한 커밋의 push와 해당 SHA의 CI 상태로 확인한다. 다음 행동은 사용자의 04 문서 리뷰다.

Target, 승인된 02/03 본문, 기존 QA 원본과 ADR, archive, 구현과 측정 자료는 수정하지 않았다. 05 비교와 DP 선발, QA freeze, 모델 실행과 측정은 미착수다.

## 5. 사용자 리뷰 후 개별 문서와 비교 그림

### 5.1 설명 구성과 참고 자료의 취사선택

사용자는 S-01~06이 줄글 위주여서 이해하기 어렵다고 지적하고 개별 문서, 직접 대조할 그림, 기존 네 자료의 선택적 참고를 요청했다. 04-00은 탐색 깊이와 전체 지도, 조합 관계를 보존하고 상세 설명은 04-01~06으로 이동했다. 기존 `s-01`~`s-06` anchor도 유지했다.

각 문서는 사용자 상황, 핵심 차이표, T/A/B 구조도, 같은 사건의 처리 흐름, 상태와 실패, 13개 품질 관점, 성립 및 반증 조건, 참고 자료의 취사선택 순서다. A/B 수는 기존 탐색 결과를 유지했다. 여섯 SVG와 편집 가능한 draw.io 원본은 같은 생성기 좌표에서 만들며, 전체 그림을 보는 HTML도 추가했다.

기존 네 자료에서 revision, 권한과 삭제 fence, 재개 identity, 활동과 외부 효과의 구별, 원자 색인 게시, producer incarnation 같은 계약을 대조했다. S-01의 runtime 재개와 S-03의 검색 자료 생산은 별개로 유지했다. S-04에 전체 domain event sourcing을 합치지 않았고, S-06의 native capability는 여전히 미확인이다. 이전 QA 수식과 선정 판단은 승계하지 않는다. 적용한 것과 제외한 것은 각 문서 §8에 기록했다.

### 5.2 그림과 상세 계약의 독립 검토

앞서 사용자 요청으로 시작한 세 독립 검토자에게 개별 문서와 생성 원본을 읽기 전용으로 검토하도록 했다. target_review는 실제 target 및 책임 경계를, alternatives_review는 강한 대안과 참고 자료의 취사선택을, quality_review는 품질 인과와 기능 손실을 검토했다. 그림의 새 발견사항은 중복 제외 **P2 3건**이며, §3의 최초 본문 발견사항과 별개다.

| ID | 문제 | 수정 및 재확인 |
| --- | --- | --- |
| DG-01 / P2 | S-05 A 입력이 곧바로 Request Interpreter로 가는 그림은 초기 Request Controller의 입력, Context와 예산 결합을 생략했다. | 초기 호출과 proposal 반환을 Request Controller에 연결했다. 본문 축약 흐름도 같은 순서로 명료화했다. target_review와 quality_review가 그림의 해결을 확인했다. |
| DG-02 / P2 | S-05 A의 단일 전달선은 Text도 음성 생성을 기다리는 것으로 보였다. | Response Manager에서 Text 직접 게시와 Model Access의 음성 생성, 결과 반환, 검사 후 Voice release를 구별했다. 실제 전달 receipt는 Interaction Manager에 표시했다. 세 검토자가 해결을 확인했다. |
| DG-03 / P2 | S-04 B coordinator에서 Task로 직접 향하는 선은 후속 요청의 현재 admission을 우회하는 인상을 주었다. | 후속 제안을 Request Controller에 반환하고 현재 admission 및 내구 원장 뒤 Task Manager와 Agent Gateway로 전달하도록 수정했다. target_review와 alternatives_review가 해결을 확인했다. |

추가 명료화로 S-05 T의 VoiceProposal 반환도 Interaction Manager를 경유하도록 고쳤다. S-02의 공통 후속 설명에 Request Controller의 입력 및 예산 결합과 Request Interpreter 호출을 적었다. 세 검토자는 수정된 실제 생성 원본과 SVG/draw.io를 다시 확인했고, **미해결 또는 신규 P1/P2를 보고하지 않았다.** 이는 runtime 지원이나 품질 우위의 입증이 아니다.

### 5.3 렌더링과 최종 문서 검사

| 검사 | 결과와 범위 |
| --- | --- |
| 실제 그림 렌더링 | PASS — headless Chrome으로 SVG 6장 렌더링 후 작성자가 전부 시각 확인. 노드의 글자 넘침 검사도 통과. PNG는 임시 검수용이며 측정 evidence가 아님 |
| 편집 원본과 SVG | PASS — 새 생성기의 6쌍 및 gallery 생성물 일치, XML ID, 좌표 경계, 직교 연결과 무관한 node 관통 검사. CI에 같은 `--check` 추가 |
| 기존 그림 보존 | PASS — target 13쌍과 기존 참고 자료 8쌍의 생성 일치 검사 |
| Active 문서 | PASS — 119개 Markdown의 local link, canonical 용어, QA catalog |
| 추적과 표 | PASS — P-01~16의 배분과 집중 P 7개, S anchor 6개, 개별 문서 6개. 기존 8개 대안의 13개 관점 전체 104개 셀 보존. 관점 이름이 03과 일치 |
| 표현과 링크 | PASS — 04 문서의 표 열 수와 fragment, 가운데점 없음, archive 규범 링크 없음 |
| 변경 경계 | Target, 승인된 02/03, 기존 네 참고 문서, QA 원본, ADR, archive와 실행 및 측정 자료 보존. 05 및 DP 선발 미착수 |

다음은 개별 문서에 대한 사용자 리뷰다. 전달 SHA와 CI는 이 기록을 포함한 커밋의 Git 이력과 해당 workflow에서 확인한다.
