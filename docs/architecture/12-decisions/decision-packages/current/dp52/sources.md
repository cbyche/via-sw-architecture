# DP52 근거와 적용 경계

2026-10-23 열람. 저장소 정적 읽기와 공식 문서 조사만 수행했다. 제품 실행이나 모델 호출은 없다.

## 1. 제품 요구와 현재 작업 기준

| 출처 | 확인한 사실 | DP52 적용 |
| --- | --- | --- |
| [INTENT](../../../../../../INTENT.md) | 실제 사용자 문제와 SW 구조에서 대안을 도출하고 예상과 관측을 구별 | 기존 A/B를 전제하지 않고 상태 적용 코드를 따라감 |
| [고정 범위](../../../../03-fixed-architecture-scope.md) | VIA와 Agent 책임, Task identity의 Agent 중립성 | 외부 Agent 내부 계획을 설계하지 않음 |
| [대표 UC](../../../../05-representative-use-cases.md) UC-06/10/13/14/15/18 | 질문 답변, 같은 Agent 복수 실행, 비동기 결과, Voice 연결 분리, 재시작 재연결 | 정상 및 실패 추적의 공통 완료 조건 |
| [참조 설계 상태와 수명](../../../target-architecture/control-and-lifecycle.md) 4~7절 | 질문 결합, 전송 및 정정, outbox와 외부 실행의 보장 구별 | 공통 정확성 조건의 참조. 참조 구현을 선택한 것으로 보지 않음 |
| [현재 DP2](../dp02/design.md) | 대화 의미 실행과 요청별 논리 실행 비교, 외부 Agent 종료까지 무조건 기다리는 안이 아님 | 이번 두 저장 경계와 동일 비교로 재명명하지 않음 |
| [현재 DP44](../dp44/design.md) | 음성 준비와 실제 전달 기록의 분리 | 질문 생성과 실제 제시를 구분하는 경계만 사용 |
| [현재 QA](../qa/README.md), [시나리오](../scenarios/README.md), [평가](../evaluation/README.md) | QA 의미와 판본, 관측과 합성의 구별 | 새 모집단, 점수, 성능 결과를 만들지 않음 |

## 2. 실제 코드에서 확인한 범위

| 파일 | 확인 사실 | 주장할 수 없는 것 |
| --- | --- | --- |
| [상태 코드](../../../../../../prototypes/candidates/four_dp_poc_v2_dp05_reuse_corrected/src/dp02/state.rs) | Conversation에 requests/pending/questions/associations를 보유. Arc, Mutex, Condvar 사용. wait_relations는 pending 전체 해소 대기 | 제품의 영속 세션과 복구가 완성되었다는 주장 |
| [실행 코드](../../../../../../prototypes/candidates/four_dp_poc_v2_dp05_reuse_corrected/src/dp02/execution.rs) | 현재 요청 revision 검사, ANSWER의 task/question 검사, 고정 답변 수신 shim | 실제 외부 Agent reply, 음성 실제 전달과 외부 exactly-once |
| [의존성](../../../../../../prototypes/candidates/four_dp_poc_v2_dp05_reuse_corrected/Cargo.toml) | 확인한 후보 crate는 serde_json 의존성을 사용 | 이 crate가 Tokio 또는 SQLite 제품 runtime이라는 주장 |

이는 지정한 코드 경로의 정적 관찰이다. 저장소 전체에 다른 구현이 전혀 없다는 전수조사 결론은 아니다. 기존 PoC의 실험 목적과 결과를 이번 구조의 실패 또는 성능 증거로 재해석하지 않는다.

## 3. 공식 API 및 원본 코드

| 출처 | 직접 확인한 메커니즘 | VIA 적용 설계 및 한계 |
| --- | --- | --- |
| [Tokio mpsc 공식 문서](https://docs.rs/tokio/latest/tokio/sync/mpsc/index.html) | bounded channel의 수용량 및 backpressure, receiver 종료와 미처리 항목 drop, close 후 drain | mailbox를 유한하게 두고 성공적인 send를 업무 적용과 구별. 영속성은 VIA가 추가 |
| [Tokio bounded.rs 원본](https://github.com/tokio-rs/tokio/blob/master/tokio/src/sync/mpsc/bounded.rs) | send가 reserve 후 permit.send를 호출하고 반환 | 처리 receipt를 별도로 저장해야 함. 원본은 mutable master이며 재현 빌드 버전으로 freeze하지 않음 |
| [SQLite Isolation](https://sqlite.org/isolation.html) | 쓰기는 직렬화되고 한 writer만 존재. commit 전 다른 connection에는 변경이 보이지 않는 기본 격리 | 짧은 transaction으로 교차 상태 적용. WAL 또는 connection 증가를 병렬 쓰기 근거로 사용하지 않음 |

Tokio 공식 문서 열람 화면에는 1.54.1이 표시되었다. 실제 제품 의존성 버전을 선택하거나 설치한 것은 아니다. 메커니즘 확인과 성능 검증은 다르다. 공식 API의 존재가 DP52의 지연, RAM 또는 정확도 개선을 입증하지 않는다.

## 4. 환경과 미확인 capability

현재 읽기 작업의 개발 호스트는 macOS 경로를 사용한다. 제품 대상 Samsung Windows PC의 음성 드라이버, 실제 Agent SDK와 storage 설정을 검증하지 않았다. Rust형 스케치가 Windows에서 컴파일되거나 실제 audio endpoint를 관측한다고 주장하지 않는다.

추가 확인은 Agent별 독립 세션/실행 ID, 질문 revision과 철회 이벤트, client correlation, reply idempotency, 상태 조회 및 재연결, 재생 범위 관측과 watermark, 대상 장비의 backlog 한도다. 이 확인 없이 provider의 capability를 가정해 사용자 완료를 보장하지 않는다.

## 5. 공용 문서 담당에게 남긴 연결 사항

공용 current README에 DP52와 이 폴더 README를 연결하고, 새 DP51~55 안내의 범위를 설명할 필요가 있다. package manifest가 새 저작 자료도 관리할지 담당자가 결정해야 한다. 이 세션은 공용 파일을 수정하지 않았다. 다른 DP에 제안한 인터페이스는 합의 전이며 기존 공통 계약을 자동 변경하지 않는다.
