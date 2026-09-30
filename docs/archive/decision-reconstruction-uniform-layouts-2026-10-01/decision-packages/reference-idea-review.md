# 기존 후보에서 살릴 것과 이전 설계 자료에서 얻은 아이디어

> **2026-09-30 · 재발굴 보강 기록 / 후보·우열 미선정** · [현재 목록](./README.md)
> 이전 7개 추천은 철회했지만 문제 상황까지 폐기한 것은 아니다. 이전 VIA-DP-01~18은 아이디어와 반론의 참고 자료로 검토했다. 기존 ID에 새 후보를 배정하거나 과거 결과를 새 설계의 근거로 승계하지 않는다.

**후속 리뷰 반영:** 처음에는 네 개만 상세화하고 아래 가능성을 보류했으나, 사용자 지적에 따라 음성 근거·기억 유지·자료 표현·복구 원본도 개별 문서로 구체화했다. 이 문서는 현재 8개 목록에 맞춰 반영 상태를 갱신한다. 이전 판단은 Git 이력에 남으며 archive 원본은 변경하지 않는다.

## 1. 기존 7개 중 살릴 것은 있었는가?

**있었다. 잘못된 것은 문제의 존재보다 비교 질문을 잡은 방식이다.** 다만 제목만 바꿔 원래 비교를 복구하지 않는다. 아래의 ‘살릴 내용’은 새 문서에 실제로 반영할 수 있는 문제·계약이며, 원래 두 방안의 우열을 인정한다는 뜻이 아니다.

| 기존 후보 | 살릴 내용 | 이번 처리와 이유 |
| --- | --- | --- |
| 요청의 해석과 확정 경계 | 지칭·목표·Task의 상호 의존, 정정 후 늦은 결과 차단 | [해석 방식](./request-interpretation-topology.md)에 반영. Request Interpreter의 소속 대신 통합 의미 조정과 단계별 중간 계약·정정 경로를 비교한다. 최종 host 검증은 공통이다. |
| Context 준비 주체 | 당시 화면과 현재 화면의 구별, source revision·coverage, 불필요한 준비 비용 | [근거 획득](./context-acquisition-strategy.md)에 반영. Context Manager는 양쪽에 유지하고, 발화 중 선행 준비와 요청별 계획 후 조회의 의존 관계를 비교한다. |
| 대화와 업무 상태 관리자 | 대화가 바뀌어도 유지되는 Task identity, 질문·승인·취소·완료의 교차 | [Agent 상태 구성](./agent-state-observation.md)의 공통 조건·반례로 반영. Request Controller와 Task Manager를 합치는 안은 복원하지 않는다. 교차 transaction 대 독립 commit은 별도 실질 선택이나 이번 우선 목록에는 넣지 않는다. |
| 응답 전달 | 생성·게시·실제 전달의 구별, 새 발화 후 오래된 음성 차단 | [직접 응답 생성](./direct-response-generation.md)에 반영. Response Manager 존재 여부 대신 선생성·보류·폐기와 경로 허용 뒤 생성의 작업 그래프를 비교한다. 일반 Agent 결과 전달 전체를 다루는 후보라고 확대하지 않는다. |
| Agent 연동 | 외부 접수와 완료의 구별, source identity, 재연결·역순·중복 | [Agent 상태 구성](./agent-state-observation.md)에 반영. adapter 소속 대신 일반 progress를 event로 접어 만드는지 snapshot으로 구성하는지 비교한다. 연동 의미 정규화 결정은 그대로 유지한다. |
| 음성 입력 근거 | 별도 ASR 의존성·중복 인식·전사 충돌과 독립 입력 진행의 대가 | [음성 근거 후보](./speech-evidence-source.md)로 상세화했다. Native evidence의 필수 기능·동시 인식·단일 runtime 장애 비용을 명시하며 실제 capability 검증은 남긴다. |
| process 경계 | native 연동 실패의 전파 범위, IPC·buffer·worker 수명 비용 | **보조 설계로 보존.** 분리 대상은 외부 Agent 자체가 아니라 VIA의 client/SDK 또는 runtime 코드여야 한다. 실제 치명적 실패 경로 없이 장애 이점을 일반화하거나 정상 latency·accuracy 이점을 붙이지 않는다. |

원래 후보의 일곱 상세 문서와 그림은 수정해 덮지 않았다. 이전 설명의 어느 부분이 부족했는지도 다시 확인할 수 있다.

## 2. VIA-DP-01~18에서 무엇을 검토했는가?

각 문서의 문제·대안·구조 판별 규칙을 검토했고, 관련 ADR의 accepted/deferred 상태도 확인했다. 아래는 **참고 자료 감사표**다. 새 DP의 분해 기준이나 ID 대응표가 아니다. 옛 문서의 Core/Non-core 분류는 당시 작업의 지위이며 이번 후보의 선정 등급으로 가져오지 않는다.

| 참고 문서 | 얻을 수 있는 아이디어 | 이번 반영 또는 유보 이유 |
| --- | --- | --- |
| [VIA-DP-01](../via-dp-01-direct-handling.md) | 제한된 정보 처리의 VIA 실행과 Agent 일원화는 실제 실행 계약의 차이 | 기능 범위를 바꾸는 큰 선택이다. 현재 합의한 한정 자료 직접 설명을 조용히 다시 열지 않는다. 직접 응답 후보는 좁은 S2S 생성 순서만 다룬다. |
| [VIA-DP-02](../via-dp-02-state-consistency.md) | 공동 원자 commit 대 독립 commit·관계 조정은 관리자 수보다 강한 질문 | 상태 후보에서 transaction·질문 결합을 공통으로 고정한다. 별도 후보는 교차 전이의 중요성과 중간 상태 비용을 더 좁힐 때 검토한다. 과거 작은 지연을 새 설계의 제외 증거로 쓰지 않는다. |
| [VIA-DP-03](../via-dp-03-voice-evidence.md) | 별도 timestamp-capable ASR 대 S2S native evidence는 dependency·정보 도착 경로의 선택 | 음성 근거의 상세 대안에 native streaming을 허용하고 공유 Omni의 동시 입력·장애 계약을 적었다. 이전 turn-final-only 제한·두 모델 구성·mock 계획은 가져오지 않는다. |
| [VIA-DP-04](../via-dp-04-response-authority.md) | 요청별 승인과 제한된 게시 lease, 생성과 승인의 병렬화 | 직접 응답 문서에 ‘승인 왕복 제거와 생성 순서는 다른 축’이라는 경계를 추가한다. 중앙 승인을 매번 LLM 호출로 과장하지 않는다. lease 비교까지 한 후보에 섞지 않는다. |
| [VIA-DP-05](../via-dp-05-context-contract.md) | 사전 read-set의 폐쇄성과 처리 중 추가 source 허용은 다른 선택 | 새 Context 후보의 조회 시작 시점과 혼동하지 않도록 명시한다. 계획안도 lazy read·cache·한정 보완을 허용하며 전체 자료 재수집을 강제하지 않는다. |
| [VIA-DP-06](../via-dp-06-semantic-authority.md) | 단계 수보다 중간 의미의 정정 권한·version 연결이 중요 | 해석 문서에 통합 조정과 단계별 정정 계약의 판별 규칙을 보강한다. 마지막 전체 재판단을 붙이면 대안 차이가 사라지는 조건도 명시한다. |
| [VIA-DP-07](../via-dp-07-compound-orchestration.md) | 사용자 명시 dependency의 VIA 실행과 Agent bundle 실행을 구별 | 한 업무 전체를 Agent에 맡긴다는 현재 합의를 우선한다. 사용자 목표 사이 관계와 Agent 내부 계획을 혼동하지 않는다. 현재 후보에 이전 node graph를 이식하지 않는다. |
| [VIA-DP-08](../via-dp-08-recovery-source.md) | event log+checkpoint 대 current state+pending command는 복구 원본 선택 | 독립적인 복구 후보로 상세화했다. 기존 Agent 상태 관측과 분리하고 DB·owner·교차 transaction을 유지한 채 권위 저장·replay·삭제·effect 조정을 비교한다. |
| [VIA-DP-09](../via-dp-09-agent-semantics.md) | 최소 공통분모로 기능을 잃지 않는 canonical contract·명시적 확장 | Agent 상태 두 안에서 동일한 의미 정규화·capability를 유지한다. adapter 위치 변경만으로 속도·정확도 차이를 주장하지 않는다. ADR-001 유지. |
| [VIA-DP-10](../via-dp-10-model-session-authority.md) | session control과 큰 audio/token data path는 분리 가능 | 공통 관리자가 있다고 모든 데이터를 중계시키지 않는다. 여러 호출이 여러 가중치 사본을 뜻하지 않는다는 조건을 유지한다. session manager 분리는 새 독립 후보로 선정하지 않는다. |
| [VIA-DP-11](../via-dp-11-process-isolation.md) | 외부 Agent 장애와 VIA client의 잡을 수 없는 치명적 실패를 구별 | 기존 process 후보의 질문에 답할 구체적 경계다. 대상 코드·fault가 미정인 상태에서 강한 효과를 주장하지 않는다. ADR-003을 철회하거나 새 성능 근거로 재해석하지 않는다. |
| [VIA-DP-12](../via-dp-12-evidence-commit.md) | 생성·게시·durable evidence·물리 출력은 서로 다른 사건 | 직접 응답 두 안의 게시와 실제 재생 근거를 공통으로 유지한다. business durability를 줄여 빠른 안을 만들지 않는다. evidence flush 순서는 별도 보조 주제다. |
| [VIA-DP-13](../via-dp-13-control-reservation.md) | 우선순위 queue만으로 이미 실행 중인 비선점 작업을 멈출 수 없음 | 추가 semantic 단계·분류·speculation의 비용을 공유 scheduler 계약 아래 설명한다. 예약량 조절을 새 구조 후보로 세지 않는다. |
| [VIA-DP-14](../via-dp-14-task-state-authority.md) | revision 기반 handler와 Task별 fenced writer는 실제 쓰기 계약의 선택 | 관측한 상태를 만드는 방식과 Task 쓰기 직렬화 방식을 분리한다. 상태 후보 양쪽의 최종 쓰기 계약은 동일하게 두고 ADR-002 지위 유지. |
| [VIA-DP-15](../via-dp-15-agent-observation-authority.md) | 이벤트 projection과 query snapshot의 강한 대안, query 합치기·freshness | 새 Agent 상태 후보와 직접 관련 있는 선행 아이디어다. 이번에는 필수 질문·승인·terminal을 일시 hint로만 남기지 않도록 강화했다. 옛 전체 progress 범위와 동일 후보라고 취급하지 않는다. |
| [VIA-DP-16](../via-dp-16-model-context-state.md) | 요청별 canonical 재구성과 증분 working view, watermark·삭제 전파 | 기억 유지의 상세 문서에 owner 변경 전달·revision barrier·gap rebuild를 정의했다. target의 기존 cache를 인정하고 ‘전체 rebuild 대 cache’라는 약한 비교는 제외한다. |
| [VIA-DP-17](../via-dp-17-context-materialization-authority.md) | 공통 source 표현과 목적별 view, 변환 재사용·정보 손실의 긴장 | target이 최종 의미를 공통 변환기로 닫지 않음을 확인했다. 자료 표현 후보는 원문 중심 package 대 선행 사실 view로 다시 구성했다. 조회 시점·parser·원문 접근은 고정하고 실제 변환 산출물과 소비 계약을 비교한다. |
| [VIA-DP-18](../via-dp-18-authorization-enforcement.md) | use별 중앙 승인과 철회 가능한 capability는 revoke barrier가 다름 | 새 후보 모두 권한 철회·실제 사용 검사를 유지한다. 로컬 함수 호출을 원격 왕복처럼 부풀리지 않는다. 보호 경계 재설계는 이번 핵심 기능 비교와 별도로 유보한다. |

### ADR의 지위를 어떻게 보존했는가

- [ADR-001](../../../adr/ADR-001-agent-integration-contract-boundary.md): Agent 의미 정규화 A accepted. 새 관측 방식 두 안에서 동일하게 유지한다. 과거 변경 국소화 결과는 현 ASR 실측이 아니다.
- [ADR-002](../../../adr/ADR-002-task-state-authority.md): Task별 durable supervisor B accepted, 현 계약 재검증 필요. 새 상태 후보는 쓰기 소유권을 다시 선정하지 않는다.
- [ADR-003](../../../adr/ADR-003-runtime-fault-isolation-boundary.md): 연동 process 격리 B accepted, 새 Voice·장비 조건 재검증 필요. shortlist에서 제외했다고 이 결정을 취소한 것이 아니다.
- [ADR-004](../../../adr/ADR-004-semantic-decision-ownership.md): deferred, A는 interim reference. target에 통합 해석이 있어도 비교 승자가 입증되었다는 뜻은 아니다. 새 해석 후보의 정정 계약은 이 선행 쟁점을 참고했으며 이전 모델·호출 수·실험값은 승계하지 않는다.

## 3. 보류했던 아이디어를 실제로 상세화한 결과

| 새 상세 문서 | 이전 보류 이유를 해소한 설계 작업 | 검증·공동 판단에 남긴 것 |
| --- | --- | --- |
| [음성 근거](./speech-evidence-source.md) | Native transcript revision·시간·gap 계약, 동시 인식·입력 확정·불일치 검증의 변경, 동일 fault에서의 기능 손실 명시 | 실제 build capability와 전체 자원, Omni 장애 중 전사 지속을 필수로 볼지 여부 |
| [기억 유지](./conversation-context-maintenance.md) | 활성 대화 view의 범위, owner별 변경 전달·revision vector·read barrier, 삭제 fence·gap rebuild 정의 | cache 최적화 후에도 차이가 남는지, 사용 빈도 대비 갱신·상주 비용 |
| [자료 표현](./context-representation-pipeline.md) | 현재 원문 중심 근거를 보존하고 별도 materialization·view 재사용·원문 복귀·오류 전파 계약 정의 | 반복 소비의 실제 가치, 추출 오류·첫 사용 비용·schema 확장 부담 |
| [복구 원본](./recovery-state-source.md) | 같은 DB와 owner 아래 이력 append·checkpoint·side-effect 없는 replay·삭제 payload·UNKNOWN 처리 정의 | 필요한 복구 범위, migration·삭제·tail replay 비용 대비 이점 |

실제 capability를 확인하지 않았다는 사실과 대안을 설계할 수 없다는 판단은 다르다. 조건을 명시해 문서화한 뒤 사용자와 중요성·실현 조건을 검토한다. 현재 네 추가 문서도 우열·정식 DP 채택이 확정된 것은 아니다. 교차 commit 대 독립 commit은 복구 원본과 별도 축이며 여전히 이번 상세 목록에는 넣지 않았다.

## 4. 아카이브의 위치와 보존 방식

이전 7개와 README·원칙·검토 기록·diagram README·그림 7쌍, 총 25개 파일은 `docs/archive/decision-reconstruction-component-boundaries-2026-09-30/decision-packages/`에 원문 그대로 보존했다. 생성기 1개는 `scripts/archive/decision-reconstruction-component-boundaries-2026-09-30/`에 보존했다. [보관 안내](../../../archive/decision-reconstruction-component-boundaries-2026-09-30/README.md)와 [원본 경로·SHA-256 manifest](../../../archive/decision-reconstruction-component-boundaries-2026-09-30/manifest.json)는 역사 기록 확인용이다.

원본 commit은 `7094db96d5e747cfefe5b6a5132edebf00b515b3`이다. 26개 보관 파일을 해당 commit의 원본과 byte 단위로 비교하고 hash를 확인한다. Snapshot 내부의 과거 상대 경로는 보존했으므로 원래 배치가 필요한 링크·생성기는 그 commit에서 확인한다. 현재 문서의 요구·계약 근거는 archive가 아니라 활성 target으로 연결한다.

**VIA-DP-01~18은 이 아카이브에 넣지 않았다.** 기존 활성 참고 문서의 내용·지위와 ADR을 그대로 보존했다. 새 문서는 같은 `decision-packages/`에 평평하게 두었으며 추가 `candidates/` 폴더는 만들지 않았다.
