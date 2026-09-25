# 이전 DP 계열 → VIA-DP 매핑과 누락 점검

> 2026-09-25 · 현행 설명은 VIA-DP-01~18 · 옛 ID는 이력 확인용

이 문서는 이전 후보를 정답으로 삼지 않고 **질문이 사라졌는지 확인하는 원장**이다. 기존 active candidate/ADR과 historical catalog를 구분해 대조했다. 이력의 분류·QA 번호·점수는 현행 요구나 측정 근거로 승계하지 않는다. 파일 이름·A/B 문자만으로 대응시키지 않는다.

## 1. 기존 9개 계열 질문

| 이전 ID·질문 | 현행 담당 | 정확한 관계·A/B 대응 | 처리 |
| --- | --- | --- | --- |
| INT-DP01 Turn routing·fast path | VIA-DP-04 중심, 01·06 연결 | 직접 응답 route release는 04 A의 제한 위임 vs B의 요청별 승인. bounded 실행 범위는 01, 요청 의미·handling 판별은 06. 외부 Action을 Voice가 독립 승인한다는 뜻 아님 | 기존 넓은 질문을 책임별 분리; 별도 중복 ID 없음 |
| IR-DP01 semantic authority | VIA-DP-06 | A 통합·B 단계 권한 유지. 같은 LLM 1개 고정 | 상세 구현도·prompt 근거 보완 |
| TASK-DP01 Task writer | VIA-DP-14 | A 공유 transaction service·B Task별 supervisor 유지 | 새 보고서 추가; B accepted caveat 내장 |
| TASK-DP02 상태 synchronization | VIA-DP-15 | 과거 B의 event 확정+query 보완은 현행 A, 과거 A의 query 확정은 현행 B. 단순 push/poll 대비 폐기 | event+query 공통 tactic과 별개인 확정 권한만 복원 |
| AGENT-DP01 의미 경계 | VIA-DP-09 | A edge 의미 확정·B Core typed handler 유지. 손실 없는 extension은 양쪽 허용 | 기능 손실 대 정확성이라는 가짜 대립 제거 |
| EXEC-DP01 Process 경계 | VIA-DP-11 | 과거 A 같은 Process → 현행 B, 과거 B 격리 → 현행 A | 대상은 VIA Client·SDK. 외부 Runtime embed 아님 |
| CTX-DP01 materialization owner | VIA-DP-17 | 과거 A 중앙 변환·B consumer 변환을 재구성. snapshot/handle만으로 배타성 주장 안 함 | 05의 read-set 질문과 달라 별도 추가 |
| CTX-DP02 model-facing history | VIA-DP-16 | A 요청별 재구성·B 증분 working state, cache/rebuild 양쪽 허용 | 10의 session과 달라 별도 추가 |
| SEC-DP01 protected-use 권한 | VIA-DP-18 | A use별 중앙 승인·B 철회 가능한 local capability 검증 | 기존 privacy 요구만 보존; 새 QA·범위 확대 없음 |

사용자가 지칭한 EXE 계열은 저장소에서 **EXEC-DP01**로 확인했다. EXE-DP라는 별도 정의는 발견하지 못했으며 같은 항목으로 연결한다. 옛 INT/TASK tactic의 ‘항상 고정’ 문구는 현재 전수 검토를 제한하지 않는다.

## 2. 별도 이름이 있었으나 중복·공통 조건이었던 질문

| 이전 이름 | 현행 처리 | 누락이 아닌 이유 |
| --- | --- | --- |
| MODEL-DP01 배치 | 공통 FA-10 및 M-04~06 변화 조건 | local/remote는 명시하지만 동일 DP 비교에서 고정. S2S 1개·semantic LLM 1개라는 현재 범위를 바꾸는 후보는 추가하지 않음 |
| STATE-DP01 journal/snapshot | VIA-DP-08 | 저장 기술 자체가 아니라 복구의 최종 기준 기록으로 재정의 |
| ORCH-DP01 bounded 실행 배치 | VIA-DP-01 | 전역 Core/Agent 양자택일을 버리고 선택적 직접 처리 hybrid 대 직접 기능 미소유로 범위 명확화 |
| FP-INT01 | VIA-DP-04 A의 한정 fast path | 구현 가능한 참조 경로이며 모든 VIA-DP를 미리 결정하는 규범 아님 |
| TASK-T01 | VIA-DP-15 A의 event+query 보완 | 두 transport 병용 자체는 tactic; event 확정 권한은 별도 검토 |

과거 DP-00/13~16의 범용 Agent·로컬 실행·제어·복구·장애 경계는 다른 세대 번호다. VIA-DP-13~16과 숫자만으로 대응하지 않는다. Agent-neutral 책임 경계는 유지하고 복합 조정은 07, Task writer는 14, 복구는 08, VIA Client 배치는 11에서 다룬다. 외부 domain planning을 VIA 내부로 가져오는 과거 범위는 재개하지 않는다.

## 3. 원래 25개 주제의 전수 귀속

| 원래 주제 | 현행 귀속 | 처리 이유 |
| --- | --- | --- |
| S01 Agent-neutral 경계 | 공통 System Mission | 외부 업무 책임은 비교로 바꾸지 않음 |
| S02 S2S/Core 중계 | 04·06 | 게시와 의미 권한 분리 |
| S03 barge-in·audio buffer | 04·13 | 필수 물리 제어·자원 계약, buffer 크기는 tuning |
| S04 model adapter·helper | 03·06·10 | 두 모델 고정; 추가 helper 모델은 범위 밖 |
| S05 시각·선택·trajectory | 03·05·06 | 관측 근거·전달·해석 구분 |
| S06 Source 값 변환 | 17 | 누락 owner 질문 복원 |
| S07 대화·Task 모델 입력 | 16 | working state 책임 복원 |
| S08 refinement·association | 06 | 의미 권한 |
| S09 Agent 선택 | 06·09 | capability 판단과 외부 의미 계약 |
| S10 bounded 직접 처리 | 01 | 지원 기능의 실행 책임 |
| S11 VIA-local tracked path | 01·14 | 직접 기능의 추적은 VIA Task 계약; 제3자 Runtime embed 아님 |
| S12 복합 관계 | 06·07·02 | 관계 이해·실행·교차 확정 |
| S13 Task state writer | 14 | 새 번호로 독립 설명 |
| S14 persistence·outbox | 08·02·14 | 복구 원본과 원자성·writer 분리; 멱등은 공통 |
| S15 Agent 계약 | 09 | 수명 의미 경계 |
| S16 push/poll·feedback | 15·04 | source 확정과 사용자 게시 |
| S17 동의·승인·egress | 18·09 | use-time 권한과 외부 승인 의미 |
| S18 Process·queue·scheduler | 11·13 | crash 경계와 정상 자원 점유 분리 |
| S19 User Memory | 05·16·17·18 | 저장·삭제·조회 요구는 필수, 독립 점수용 DP 아님 |
| S20 재시작 재연결 | 08·10·14·15 | 상태·모델 세션·owner·외부 관측 복원 |
| S21 응답·알림 | 04·12·13 | 게시·기록 순서·실행 자원 |
| S22 telemetry | 12 및 모든 DP trace | QA-23/61/62 전수 점검 |
| S23 OS/framework/DB 제품 | 해당 DP 구현 조건 | 계약·배치 변화 없으면 제품 선택만으로 DP 추가 안 함 |
| S24 domain planning·Tool 실행 | Downstream Agent | VIA 범위 밖 |
| S25 Mobile/TV/Robot | 현 PC 범위 밖 | 제품 범위 확대 안 함 |

이 표는 질문 귀속 확인이지 모든 UC의 구현·시험 통과 주장이 아니다. 아직 발견하지 못한 미래의 모든 DP까지 완전하다는 뜻도 아니다. 새로운 독립 권한·상태·계약 질문이 나오면 기존 항목에 억지로 접지 말고 같은 protocol로 추가 검토한다.

## 4. provenance와 상태

대조 대상은 기존 candidates의 IR/TASK/AGENT/EXEC 정의, ADR-001~004, docs/archive/w12-g1/12-01-dp-master-catalog.md의 9개 질문과 12-01a-scope-and-coverage-ledger.md의 25개 주제다. archive는 **historical provenance only**이며 현재 요구·QA·결정의 normative source가 아니다. 현재 번호·정의는 [18개 VIA-DP index](./README.md), QA는 [현행 catalog](../08-quality-attributes/quality-model.md)를 따른다.

기존 승인 이력은 06 Deferred/A interim, 09 A accepted, 11의 A 방향에 대응하는 기존 격리 B accepted, 14 B accepted로 유지한다. 새 15~18은 검토 초안이고 승자·ASR 확정은 없다.
