# 구조 중심 Decision Reconstruction 작업 계획

> 상태: COMPLETE · 자료 작성·독립 검토·Git 전달 완료 · 시작: 2026-10-01 · 사용자 요청: 중간 승인 없이 최종 자료까지 수행
> 재개 규칙: compaction·세션 재개 시 AGENTS.md 다음으로 이 파일을 읽고, 아래 체크포인트와 실제 git diff를 확인한다. 완료한 단계를 반복하거나 기존의 동일 격자 그림으로 돌아가지 않는다.

## 1. 이번 작업의 목적과 고정 조건

VIA의 중요한 사용자 문제를 해결하는 **서로 다른 SW Architecture**를 비교한다. 방안 1은 REVIEWED_BASELINE target의 실제 구조다. 방안 2는 같은 기능·완료 조건을 달성하는 독립적인 설계이며 target의 Component 목록·책임 배치에 갇히지 않는다. 새 Component가 필요한 이유, 없어지거나 대체되는 책임, 별도 저장소·실행체·인터페이스가 실제로 설명되어야 한다.

Component 이름만 바꾸기, 기존 함수를 둘로 나누기, 호출 순서·알고리즘·정책·설정값만 바꾸기는 선발 근거가 아니다. Component 수를 늘리거나 그림을 비대칭으로 꾸미기 위한 구조도 제외한다. 대안을 채택했을 때 유지해야 할 독립 계약·상태·수명·장애 처리가 실제로 달라져야 한다.

고정: VIA는 interaction·orchestration, Agent는 업무 planning·execution. 한 on-device Omni의 가중치를 공유하고 음성 입력 지속성을 유지한다. 추가 helper dependency는 목적과 전체 비용을 명시한다. 사용자 기능·권한·삭제·중복 실행 방지 요구는 유지하되 이를 구현하는 책임 배치는 대안으로 바꿀 수 있다. Target 본문·accepted/deferred ADR·기존 증거는 수정하지 않는다. 구현·모델 실행·성능 측정·수치 freeze·우열 확정은 하지 않는다.

## 2. 순서와 방법

| 단계 | 수행 방법 | 완료 조건 |
| --- | --- | --- |
| A. 근거 확인 | AGENTS 및 읽기 순서의 active baseline, QA, target 전체 계약·기존 후보와 reference 아이디어 확인 | 기능 요구와 현재의 구조 선택을 구별한 문제 목록 |
| B. 문제부터 발굴 | VIA의 사용자 실패 상황 → 중요한 품질 → 가능한 다른 해결 전략; 기존 8개 수에 맞추지 않음 | 선발·통합·제외 이유가 있는 discovery 기록 |
| C. 대안 구조 설계 | 1안 target 근거 대조; 2안 Component inventory·책임 이동·state owner·정상/정정/삭제/장애/복구 작성 | 이름을 가려도 설명 가능한 구성 차이 및 합리적인 대안 선택 조건 |
| D. 상세 설명 작성 | 후보별 배경 1장 원고, 설계 비교 1장 원고, 실행 사례·인터페이스·수명·ASR/QA 장단점 표·반증 조건 | 현재 ASR 4개 적용 범위 및 메모리/관련 QA의 비용을 숨기지 않는 독립 문서 |
| E. 구조도 제작 | 후보별 실제 topology에 맞게 별도 배치. 1안/2안 Component 존재·부재·대체·배치·저장 경계 명시 | paired SVG/draw.io; 공통 검정·양안 차이 파랑; 같은 2×2 격자 금지; 부재를 가짜 박스로 채우지 않음 |
| F. 독립 검토 | 별도 검토 에이전트로 baseline 충실성·steelman·품질 인과·그림 topology 검토. 발견사항 수정 후 재검토 | 실질 결함 해결 기록. 단순 링크/좌표 검사를 설계 검증으로 취급하지 않음 |
| G. 시각·저장소 검증 | 전 SVG 실제 렌더, 잘림/겹침 확인, draw.io XML/연결, active 링크·용어·QA·generator drift·diff 점검 | 모든 검사 통과, 보관·활성 navigation 일치 |
| H. 전달 | 기존 세대 archive와 철회 이유, README·선발 원칙·진행 기록 정리. 완료 문서 commit & push, CI 확인 | 처음 읽을 파일·최종 개수·검증 범위·미측정 사항과 SHA 보고 |

## 3. 후보 선발 gate

1. VIA의 고정 UC와 실제 실패를 설명하는가? 범용 기술 도입 자체를 문제로 삼지 않는다.
2. 방안 1의 존재 요소·기능을 누락해 약하게 만들지 않았는가?
3. 방안 2에 실제로 새로 생기거나 대체·제거되는 Component/저장소/실행체가 있는가? 어떤 기존 책임을 대신하는지 명확한가?
4. 그 변화가 데이터 획득·상태 소유·실행·장애·복구의 다른 메커니즘을 만드는가?
5. 대안을 선택할 현실적인 조건과 불리한 조건을 모두 설명하는가?
6. ASR/QA 영향이 구조에서 이어지는가? 단순 유지보수·장애격리 주장으로 끝나지 않는가?
7. 후보 간 같은 효과를 중복 계산하지 않는가? 남는 의존성은 명시했는가?

## 4. 그림 검수 질문

- 문장 설명을 읽기 전에 1안에만 있는 요소, 2안에만 있는 요소, 책임이 이동한 위치를 지목할 수 있는가?
- 공통 입력·사용자 출력·권한 요구를 유지하면서 해결 방식이 어떻게 다른지 보이는가?
- 새 서브시스템 안의 구성·저장 상태·외부 port가 구체적인가? 박스 이름만 그럴듯하지 않은가?
- 같은 SW를 일부러 다른 위치에 옮겨 차이를 과장하지 않았는가?
- process/Component/자료 저장소/모델 dependency의 의미를 구분했는가? 파랑은 양안 차이이지 2안 추천 표시가 아니다.

## 5. 체크포인트 / 재개 메모 (진행 이력)

- [x] 계획 저장 및 사용자 조건 명문화
- [x] A–B: baseline 확인·후보 선발 근거 작성
- [x] C–D: 4개 대안 구성·개별 문서 작성 및 1차 독립 검토 수정 (최종 재검토는 F)
- [x] E: 새 topology 비교 그림·배경 그림 완성
- [x] F: 독립 검토·수정·재검토
- [x] G: 전 렌더·일관성 검사
- [x] H: archive/navigation·commit/push·CI

초기 탐색 기록: semantic retrieval 서브시스템, durable request workflow, conversation projection, event-sourced recovery, native speech evidence. 이 5개를 자동 확정하지 않는다. 기존 8개 및 추가 아이디어와 비교해 선발하고 근거를 남긴다. 현재 작업 시작 커밋은 `0e2d15f9e71c7d73b434831a021df16822ac939f`이며 작업 시작 시 main working tree는 clean이다.

### 설계 단계 체크포인트 (당시 기록)

- 선발: semantic-retrieval-subsystem, durable-request-orchestration, speech-evidence-source, recovery-state-source. 총 4개이며 사용자 최종 선택 또는 우열 확정이 아니다.
- conversation projection은 기존 cache와의 독립 비용·가치가 충분하지 않아 주력에서 보류한다. 나머지 옛 후보도 discovery 문서에 판단 이유를 남긴다.
- 기존 8개 세대는 `docs/archive/decision-reconstruction-uniform-layouts-2026-10-01/` 및 대응 scripts archive에 원본 48파일 SHA-256 manifest와 함께 보관했다.
- 독립 구조·품질 검토의 1차 지적: UC-17 memory activity 누락, retrieval policy 권위 혼동, top-1 대상 확정 위험, 양안 durable/KV 공통 기능의 편향, replay 효력 과장. 개별 문서에 수정 반영했다.
- 재개 다음 작업: 새 generator 완성·전 SVG 렌더/보완 → discovery/quality/navigation 기록 → 독립 재검토 → 전체 검사 → commit/push/CI.

- 4개 개별 문서·공통 품질/선발/discovery/reference/README와 8개 SVG/draw.io 완성. 전 SVG 실제 렌더 확인. 독립 1차 그림 결함(검색 owner·embedding·gate·cache, 음성 호출 주체) 수정. 현재 세 검토자의 최종 재검토 및 저장소 검사 단계.

### 문서 완료 체크포인트 (당시 기록)

세 검토자 최종 재검토에서 미해결 P1/P2 없음. 4개 상세 문서·배경/비교 8장·선발/보류/품질/검토 기록 완료. 102개 active 문서 링크·용어·QA·target 13쌍·후보 8쌍 검사 PASS. 보관 48파일 byte/SHA 확인. 다음 단계는 scoped commit → main push → 해당 SHA CI 확인이며, 구현·측정 작업으로 넘어가지 않는다.

### 최종 완료 / 다음 세션의 시작점

- 완성 자료 커밋: `e1570e789c4dbb7fbdd01efe03a92dad442e9032`, `origin/main` push 완료.
- [해당 커밋 architecture-ci](https://github.com/cbyche/via-sw-architecture/actions/runs/36751466485): **SUCCESS** 확인.
- 네 비교안과 8장 그림, 보관·선발·품질·검토 문서가 전달됐다. 마지막 변경은 이 완료 기록이다.
- compaction·새 세션에서 A~H를 다시 시작하지 않는다. [README](./README.md)에서 네 문서를 읽고 사용자의 다음 검토에 대응한다. 사용자 최종 선정 전 DP 채택·target 변경으로 해석하지 않는다.
- 구현·모델 실행·성능 측정·ASR 승격은 여전히 미수행이며 별도 승인 없는 다음 작업이 아니다.

## 후속 작업 — 조건별 기호와 추가 QA (2026-10-01)

사용자 요청: 공통 QA-09·19·29·39·메모리를 유지하고 각 DP에서 의미 있는 추가 QA를 발굴해 쉬운 이유와 (+)/(-)/(0)로 정리한 뒤 commit/push한다.

- 범위: 4개 개별 문서의 공통 5항목/추가 QA 표, 품질 비교 규칙, README·검토 기록. 기준선·QA catalog·그림 구조·구현·측정은 변경하지 않는다.
- 순서: 현재 계약 확인 → 조건별 기호 및 DP별 추가 4항목 작성 → 과장·중복 가중·(0) 오용 검토 → 문서 검사·diff → commit/push·CI.
- 문서 완료: 공통 5항목 + DP별 추가 4항목 작성, 독립 품질 검토의 P2 수정·재확인 완료(미해결 P1/P2 없음). 링크·용어·QA·그림 원본·표 구조·diff 검사 통과. Git 전달은 이 후속 편집을 포함한 커밋의 push/CI 기록으로 확인한다. 최초 A~H 설계 재구성은 다시 수행하지 않는다.

## 후속 작업 — ISO/IEC 25010 기반 QA 재작성 (2026-10-01)

사용자 지적: 앞선 추가 QA는 품질속성·관찰 지표·설계 비용을 혼용했다. 해당 검토 완료는 ISO 분류 적합성까지 검증한 것으로 해석하지 않는다. 기존 자료의 구조·그림은 유지하면서 품질 비교를 다시 작성한다.

- 기준 판본: ISO/IEC 25010:2023. 기존 저장소의 change-locality 근거도 이 판본을 사용한다. 공식 표준 소개와 공개 원문으로 특성·부특성을 확인한다.
- 범위: 품질 모델의 분류 근거 보충, 네 문서의 공통/추가 비교표, 공통 비교 규칙·안내·검토 기록. QA ID·metric·ASR 지위·target 구조는 유지한다.
- 방법: 표준의 특성/부특성 → VIA 품질 질문 → 실제 구조 차이 → 조건별 기호와 쉬운 이유 → 확인할 관찰 항목. 비용이나 기능 이름을 별도 품질속성으로 만들지 않는다.
- [x] I. 표준 근거·기존 QA 대응 정리
- [x] J. 네 문서 재작성; 추가 항목 수를 맞추기보다 인과가 분명한 품질만 유지
- [x] K. 작성자와 별도 에이전트의 ISO 분류·구조/steelman·전체 문서/그림 일치 검토 및 보완
- [x] L. 링크·용어·QA·그림 생성 일치·표·diff 검사
- [x] M. commit/push 및 해당 SHA의 CI 확인

재개 시 I~M의 실제 진행과 git diff를 먼저 확인한다. 이전 A~H를 반복하지 않으며 구현·모델 실행·성능 측정으로 넘어가지 않는다. 시작 커밋은 `5cdf8dfc5123ddcb52bb1b82d426bf9b854b0d1d`, 작업 시작 시 main은 clean이다.

후속 완료: ISO 분류 대응·네 문서 재작성 완료. 세 독립 검토자의 지적을 수정·재확인했고 미해결 P1/P2 없음. 링크 103개·QA registry·기존 그림 21쌍·표 구조·diff 검사 PASS. 자료 커밋 `78d3ac3df6de0b205ddd5e9d9e87563f1577839c`를 origin/main에 push했고 [해당 architecture-ci](https://github.com/cbyche/via-sw-architecture/actions/runs/36757477256)의 SUCCESS를 확인했다. 이 완료 기록 외에 미완료 편집은 없다. 이후에는 사용자 검토에 대응하며 구현·측정으로 넘어가지 않는다.
