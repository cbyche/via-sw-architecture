# 후보 도출과 자체 검토 기록

> 상태: **문서·구조 대조에 의한 자체 검토 / 사용자 선정 전**
> 기록일: 2026-09-30 · [선발 원칙](./selection-principles.md) · [후보 목록](./README.md)

## 1. 어떻게 추렸는가

먼저 검토 완료 기준선의 사용자 흐름·Component·상태 소유권과 제어·기억·공유 Omni 계약에서 구조적 선택을 찾았다. 이전 DP inventory에서 번호나 대안을 가져오지 않았다. 이어 같은 사용자 행동을 제공하는 대안을 만들고, Component 변화의 실질성·대안의 강도·후보 중복을 검토했다.

아래 분류는 작성자의 제안이다. 사용자에게 이미 선정받았거나 독립 심사·실험을 통과했다는 뜻이 아니다. ASR 검토는 사용자가 요청한 후속 순서로 남긴다.

## 2. 약한 대안을 어떻게 보완했는가

| 후보 | 배제한 약한 대안 | 현재 steelman에 반영한 보완 | 남은 핵심 질문 |
| --- | --- | --- | --- |
| [해석·확정 경계](./request-resolution-boundary.md) | 모델이 검증 없이 직접 실행 | 통합 Component 내부의 typed module·결정적 검증·권한 경계 유지 | 공개 계약을 내부화하는 실질적 효과가 있는가, 박스만 지웠는가 |
| [Context 준비](./context-preparation-ownership.md) | 소비자가 원본 DB를 직접 읽고 stale 정보 사용 | owner read port, revision, provenance, 삭제·권한 무효화 유지 | 소비자별 요구가 공통 준비의 이점보다 크게 다른가 |
| [대화·업무 owner](./conversation-task-ownership.md) | Conversation 종료와 함께 Task 종료 | 독립 aggregate·identity·mailbox·내구 기록 유지 | 한 owner로 교차 상태 전이가 단순해지는가, 변경 책임만 커지는가 |
| [응답 전달](./response-publication-ownership.md) | 응답마다 마음대로 음성 재생 | 공통 Voice arbiter·epoch·admission과 응답별 내구 원장 유지 | 차례 조정과 전달 이력 조회의 분산 비용을 감수할 이유가 있는가 |
| [Agent 연동](./agent-integration-boundary.md) | Task Manager에 vendor SDK 직접 결합 | 개별 연동 owner와 versioned 최소 port·명시적 extension·공통 library 허용 | 현재 adapter보다 독립된 책임이 실제로 생기는가 |
| [음성 입력 근거](./speech-evidence-boundary.md) | semantic이 끝날 때까지 녹음만 보관 | Omni의 동시 입력 session·시간 근거·bounded service를 필수로 명시 | 그 기능을 제공할 runtime/build가 가능한가; 현재 미확인 |
| [runtime 격리](./runtime-isolation.md) | 단일 thread와 blocking 호출 | 충분한 thread·비동기·bounded queue, 위험 dependency process 격리 유지 | host process-fatal 격리의 가치가 추가 경계 비용을 정당화하는가 |

## 3. Component 변화의 강도를 검토한 결과

| 후보 | 눈에 보이는 변화 | 판정 |
| --- | --- | --- |
| 해석·확정 경계 | Request Interpreter의 독립 경계가 Request Controller 내부로 이동 | 우선 검토. 내부 module로도 동일 경계가 유지되어 차이가 약해지면 탈락 가능 |
| Context 준비 | Context Manager의 구성 책임·view 상태가 소비자 쪽으로 분산 | 우선 검토. Component 수가 같아도 책임·연결이 실질적으로 달라짐 |
| 대화·업무 owner | Task Manager의 상태 권위가 Request Controller로 통합 | 우선 검토. 여러 lifecycle을 다루는 owner 계약의 선택 |
| 응답 전달 | Response Manager를 응답 종류별 Component로 분리하고 공통 출력 계약 확대 | 우선 검토. publication·복구 owner와 연결이 달라짐 |
| Agent 연동 | Agent Gateway의 전송·수신 lifecycle을 Agent별 Component가 소유 | 우선 검토. 기존 adapter를 다시 그린 수준이라면 탈락 가능 |
| 음성 입력 근거 | 독립 ASR dependency와 worker·불일치 계약 제거 | 조건부. 최상위 Component 변화가 아니며 capability 공백 존재 |
| runtime 격리 | UI·Voice·Core process 경계와 IPC 제거 | 조건부. 논리 Component 구성은 그대로 |

핵심 후보 다섯 개를 모두 정식 package로 만들자는 결론은 아니다. 특히 해석 경계와 Agent 연동은 그림의 경계를 바꾼 만큼 실제 독립 책임이 달라지는지 사용자 검토에서 엄격하게 확인한다.

## 4. 별도 후보로 올리지 않은 항목

| 논의 주제 | 이번 처리와 이유 | 다시 후보가 될 조건 |
| --- | --- | --- |
| 추론 예약량·우선순위·round-robin·quantum | 정책·알고리즘·설정. 동일 Model Access 책임 안의 차이만으로 독립 후보를 만들지 않음 | 자원 admission·session 관리 권한과 Component 계약을 바꾸는 현실적 대안이 생길 때 |
| Agent 선택 순위·matching | 선택 알고리즘만으로는 구조 변화가 약함. 요청 해석 경계와 Agent 연동 문제로 구분 | 독립 선택 Component가 고유 상태·계약·변경 책임을 가져야 하는 근거가 생길 때 |
| semantic 호출 2회·추가 읽기 한 묶음 | 현재 처리 정책. 횟수만 바꾼 비교는 제외 | orchestration 소유권과 Component 경계를 달리해야 해결되는 문제가 확인될 때 |
| cache TTL·보관 기간·queue 크기 | 설정값 변경. Context 준비·수명 계약의 비용으로 남김 | 원본·파생 view의 owner나 저장 경계가 바뀌는 대안이 필요할 때 |
| embedded DB 제품·저장 엔진 | 같은 원자성·복구 계약을 구현하는 제품 선택만으로는 부족 | 저장·복구 책임과 여러 owner의 확정 경계가 달라질 때 |
| 공유 transaction 대 owner별 journal | 구조적 후보는 맞지만 Component 시각 변화가 상대적으로 약하고 업무 owner 후보와 섞일 위험이 있음. 별도 파일 승격 보류 | owner 배치를 고정한 상태에서 독립 persistence 질문으로 다룰 필요가 확인될 때 |
| S2S 직접 응답의 허용 범위 확대 | 현재 사용자가 지정한 좁은 범위를 바꾸는 제품 정책. 이번 대안에 포함하지 않음 | 사용자 지정 변경과 별개로 admission owner의 구조 차이를 구체화할 때 |
| 복합 업무를 단계별로 VIA가 계획·실행 | 한 업무를 통째로 Agent에 맡기는 합의 및 domain planning 경계를 위반 | 현재 범위에서는 제외. 명시된 독립 목표 간 관계 관리는 유지 |
| 모델 weights의 역할별 복제 | 공유 on-device Omni 사용자 지정에 어긋남 | 이번 후보에서는 제외 |
| Policy Manager의 존재 자체 | 필수 정보 통제를 근거로 Component별 후보를 자동 생성하지 않음 | 같은 권한 요구를 다른 상태·강제 경계로 충족하는 구체적인 대안이 필요할 때 |

## 5. 후보 간 중복을 어떻게 통제하는가

- **해석·확정 ↔ Context 준비:** 앞의 후보는 제안·제어 경계, 뒤의 후보는 근거 view 구성 owner다. 첫 비교에서는 Context 준비 위치를, 둘째 비교에서는 해석·확정 경계를 고정한다.
- **해석·확정 ↔ 대화·업무 owner:** 둘 다 Request Controller가 커질 수 있지만 이동하는 책임이 다르다. 두 통합을 동시에 적용한 결과를 한 선택의 효과로 설명하지 않는다.
- **대화·업무 owner ↔ Agent 연동:** 전자는 업무 상태의 의미 권위, 후자는 provider별 전송·수신 lifecycle이다. Agent 업무의 domain reasoning은 모두 외부에 남는다.
- **응답 전달 ↔ Context 준비:** 응답 Component가 나뉘면 조회 소비자 수도 달라질 수 있다. Context 후보에서는 공통 Response Manager를 유지하고, 응답 후보에서는 현재의 근거 제공 계약을 유지한다.
- **논리 구조 ↔ runtime 격리:** Component를 나누었다고 process-fatal 격리가 생기는 것으로 계산하지 않는다. runtime 비교에서는 논리 owner를 유지한다.
- **음성 입력 근거 ↔ runtime 격리:** 입력 경로 후보만 ASR 유무를 바꾼다. runtime 후보는 ASR·추론 dependency 구성을 유지한다.

## 6. 그림 상세화 재검토

사용자의 Senior SW Architect 심사 관점과 제공한 비교 슬라이드를 반영해 일곱 그림을 다시 구성했다. 기존의 단순 Component 연결만으로는 상태·제어·복구 비용을 따라가기 어려웠다. 재작성에서는 각 그림의 시나리오, 내부 책임, 핵심 payload, 내구 기록 또는 파생 view, 번호 흐름과 예외 조건을 드러냈다.

| 재검토 항목 | 반영 내용 |
| --- | --- |
| 색의 의미 | 방안별 색 구분을 제거. 공통 검정 / 차이 파랑을 양쪽에 동일 적용 |
| 책임과 상태 | 확정 상태·해석 임시 상태, 원본·view, 업무 payload·delivery 상태, publication·장치 receipt를 구분 |
| 경계의 종류 | Component·내부 모듈·process·dependency·저장 기록을 분리 표기 |
| 비교의 공정성 | 검증·독립 Task lifecycle·공통 arbiter·provider adapter·비동기 host를 대안에도 유지 |
| 호출 정합성 | 입력 확정은 Interaction Manager를 경유, semantic은 Request Interpreter를 경유, publication 원장은 응답 owner가 변경 |
| 그림과 근거 | 내부 블록은 기준선 책임의 설명용 분해. 승인된 새 내부 설계나 구현으로 취급하지 않음 |
| 검토 방식 | 일곱 SVG 렌더링 확인, XML·중첩·화살표 endpoint·leaf 관통·원본/preview 일치 검사 |

사용자 예시의 수치·점수는 옮기지 않았다. 본문과 그림의 이점·비용은 구조적 가설이며 ASR 선정이나 측정 결과가 아니다. 상세한 [그림 작성 규칙](./diagrams/README.md)을 남겼다.

## 7. 다음 공동 검토에서 결정할 것

각 후보의 문제 중요성, 두 방안의 설득력, 실제 Component 차이와 후보 간 독립성을 확인해 채택·통합·보류한다. 구조적 후보가 좁혀지면 메모리를 포함한 ASR의 추가·변경과 관련성을 논의한다. 지금은 metric·target·device budget이나 정식 package 개수를 정하지 않는다.

근거: [Target Architecture](../target-architecture/architecture.md), [제어 계약](../target-architecture/control-and-lifecycle.md), [기억 계약](../target-architecture/memory-and-context-lifecycle.md), [공유 Omni 계약](../target-architecture/shared-omni-runtime.md), [설계 완결성](../target-architecture/design-completeness.md).
