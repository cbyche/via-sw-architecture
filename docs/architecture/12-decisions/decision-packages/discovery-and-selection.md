# 문제에서 출발한 발굴·선발 기록

> 이번 결과: 상세 비교안 4개. 사용자 최종 DP 선정 또는 Architecture 변경이 아니다. 숫자를 유지하기 위해 후보를 만들지 않았다.

## 발굴 방법과 근거

[제품 경계](../../01-system-mission-and-boundary.md), [고정 기능](../../03-fixed-architecture-scope.md), [대표 UC](../../05-representative-use-cases.md), target의 전체 구조·제어·기억·공유 runtime·완결성 문서를 먼저 읽었다. 이어 기존 후보·VIA-DP의 문제를 보되 과거 A/B 정의나 ID를 새 분해 기준으로 쓰지 않았다.

각 문제에 대해 ① 사용자 실패 ② target의 실제 해결 ③ 다른 생산/실행/저장 방식 ④ 새로 필요하거나 제거되는 체계 ⑤ 품질 이득과 대가 ⑥ 남아 있는 기능 누락을 작성했다. 1차 탐색의 다섯 구조 중 conversation projection은 별도 주력으로 세우는 근거가 약해 보류했고, 나머지 네 개를 독립 검토와 함께 구체화했다.

## 탐색 결과

| VIA 문제·탐색 방향 | 구조적 대안의 실체 | 판정과 이유 |
| --- | --- | --- |
| 주제·이전 대화로 과거 자료 찾기 | 지속 Indexing Worker·Embedding Runtime·Vector Index·Semantic Retrieval Service | **선발.** 원본 조회만으로 후보를 생산하는 책임을 바꾸며 정확성·응답 대기와 별도 helper·삭제 수명 비용이 충돌 |
| 기다리던 요청을 정정·재시작 뒤 이어가기 | Interaction Workflow Runtime·내구 signal/timer/activity·Continuation Store | **선발.** 단순 controller 분리가 아니라 진행 상태의 writer와 실행 substrate 교체. 양안 모두 이미 durable이라는 조건 유지 |
| 의미 추론 중 연속 발화 근거 생산 | 독립 recognizer 제거, Omni native evidence adapter로 대체 | **선발.** helper/process의 실제 제거와 인식·추론의 공유 장애/자원 경계. Native capability 지원은 조건이며 미검증 |
| 재시작 후 대화·업무 관계 복구 | 권위 journal + projection/replay/checkpoint 서브시스템 | **선발.** 현재 row를 권위에서 파생물로 바꾸고 재구축 능력과 장기 reader/삭제 비용을 교환 |
| 대화 Context를 요청 전에 준비 | 지속 Conversation Projector·변경 subscription·Working View Store | **보류.** 가능한 구조지만 target도 summary/cache가 있음. 별도 상시 생산체를 필요로 하는 반복 조합 부하·독립 소비자가 현재 자료에서 충분하지 않음. 검색 색인과 다른 원천·소비·수명 경계를 입증하면 재검토 |
| 자료를 여러 목적에 재사용 | Fact Extraction Worker·typed Fact View Store·materialization service | **보류.** 안정된 반복 자료에 유효할 수 있으나 현재 UC의 다양성에서 선행 추출을 필수화할 근거 부족. 추가 helper·schema·원문 fallback을 구체화할 때만 독립 후보 가치 |
| 요청 해석과 확정 | 단일 semantic call 대 다단계 해석 | **이번 주력 제외.** 현재 제안은 주로 모델 호출 분해·중간 payload 차이. 독립 생산체나 권위 교체의 가치보다 알고리즘적 분해가 중심 |
| Context 준비 주체 | 계획 후 조회 대 해석 중 demand read | **이번 주력 제외.** 읽기 전략과 dependency는 중요하나 같은 Component의 계획·호출 순서 차이를 넘어서는 구조가 부족. 검색 서브시스템 후보에서 다른 획득 구조를 다룸 |
| Agent 상태 받기 | event 중심 projection 대 필요 시 query | **이번 주력 제외.** source capability·freshness 계약은 중요하지만 제출안에서는 같은 adapter/Task owner 안의 획득 방식 차이. 외부 event broker 도입만으로 부풀리지 않음 |
| 직접 응답 준비 | 해석 후 생성 대 speculative draft | **이번 주력 제외.** 작업 선행과 취소 정책이 중심이며 publication 권위는 공통. 단순 병렬화·큐 추가를 새 Architecture로 포장하지 않음 |
| 외부 연동 장애 전파 | in-process adapter 대 별도 worker | **보류.** process-fatal 의존성이 실제 범위에 들어오면 독립 배치 결정이 된다. 현재 문제·QA 인과 없이 일반적인 격리 효과만 주장하지 않음 |
| 모델 역할 배치·포트폴리오 | 역할별 모델/서버 분리 | **이번 범위 제외.** 한 on-device Omni weights 공유라는 현 비교 고정 조건을 별도 승인 없이 바꾸지 않음. 음성 helper 존폐는 명시적으로 전체 비용을 비교 |

## 선발한 네 가지는 왜 독립적인가

- 검색은 **후보를 생산하는 자산과 서비스**를 바꾼다. 업무 의미 확정이나 Context memory owner는 그대로다.
- Workflow는 **요청 진행을 실행하는 주체**를 바꾼다. 현재 row 기반 State Store를 유지하며 event sourcing을 함께 도입하지 않는다.
- 음성은 **입력 근거를 생산하는 recognizer와 장애 경계**를 바꾼다. semantic 역할·host 확정 권한은 유지한다.
- 복구는 **운영 상태의 권위 저장과 재구축 체계**를 바꾼다. Request Controller의 domain 진행을 workflow로 옮기지 않는다.

검색 helper의 자원 경합과 음성 인식, workflow의 저장과 event sourcing은 실제 제품에서 상호작용한다. 이번 문서는 한 번에 한 구조를 비교한다. 네 대안을 동시에 조합한 제품의 이득을 합산하지 않는다.

## 같은 기존 문제를 살렸다는 뜻

음성·복구의 사용자 문제는 중요해서 유지했다. 이전 그림의 격자를 그대로 재사용한 것이 아니라 worker/helper 제거와 journal/replay 체계로 실제 차이를 다시 그렸다. 대화·Context·요청 문제도 사라진 것이 아니다. 그 문제를 푸는 더 강한 구조가 검색 생산체와 workflow 실행체이며, 예전 후보 번호의 후계 관계는 만들지 않았다.

기존 세대의 정확한 보관 위치와 과거 자료 검토 범위는 [reference 검토 기록](./reference-idea-review.md)을 따른다. 보류는 영구 부정이 아니며 필요한 조건을 확보할 때 재검토한다.
