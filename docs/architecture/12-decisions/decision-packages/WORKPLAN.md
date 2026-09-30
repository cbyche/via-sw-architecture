# 구조 중심 Decision Reconstruction 작업 계획

> 상태: DOCUMENTS_COMPLETE · Git 전달 진행 · 시작: 2026-10-01 · 사용자 요청: 중간 승인 없이 최종 자료까지 수행
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

## 5. 체크포인트 / 재개 메모

- [x] 계획 저장 및 사용자 조건 명문화
- [x] A–B: baseline 확인·후보 선발 근거 작성
- [x] C–D: 4개 대안 구성·개별 문서 작성 및 1차 독립 검토 수정 (최종 재검토는 F)
- [x] E: 새 topology 비교 그림·배경 그림 완성
- [x] F: 독립 검토·수정·재검토
- [x] G: 전 렌더·일관성 검사
- [ ] H: archive/navigation·commit/push·CI

초기 탐색 기록: semantic retrieval 서브시스템, durable request workflow, conversation projection, event-sourced recovery, native speech evidence. 이 5개를 자동 확정하지 않는다. 기존 8개 및 추가 아이디어와 비교해 선발하고 근거를 남긴다. 현재 작업 시작 커밋은 `0e2d15f9e71c7d73b434831a021df16822ac939f`이며 작업 시작 시 main working tree는 clean이다.

### 현재 체크포인트

- 선발: semantic-retrieval-subsystem, durable-request-orchestration, speech-evidence-source, recovery-state-source. 총 4개이며 사용자 최종 선택 또는 우열 확정이 아니다.
- conversation projection은 기존 cache와의 독립 비용·가치가 충분하지 않아 주력에서 보류한다. 나머지 옛 후보도 discovery 문서에 판단 이유를 남긴다.
- 기존 8개 세대는 `docs/archive/decision-reconstruction-uniform-layouts-2026-10-01/` 및 대응 scripts archive에 원본 48파일 SHA-256 manifest와 함께 보관했다.
- 독립 구조·품질 검토의 1차 지적: UC-17 memory activity 누락, retrieval policy 권위 혼동, top-1 대상 확정 위험, 양안 durable/KV 공통 기능의 편향, replay 효력 과장. 개별 문서에 수정 반영했다.
- 재개 다음 작업: 새 generator 완성·전 SVG 렌더/보완 → discovery/quality/navigation 기록 → 독립 재검토 → 전체 검사 → commit/push/CI.

- 4개 개별 문서·공통 품질/선발/discovery/reference/README와 8개 SVG/draw.io 완성. 전 SVG 실제 렌더 확인. 독립 1차 그림 결함(검색 owner·embedding·gate·cache, 음성 호출 주체) 수정. 현재 세 검토자의 최종 재검토 및 저장소 검사 단계.

### 문서 완료 체크포인트

세 검토자 최종 재검토에서 미해결 P1/P2 없음. 4개 상세 문서·배경/비교 8장·선발/보류/품질/검토 기록 완료. 102개 active 문서 링크·용어·QA·target 13쌍·후보 8쌍 검사 PASS. 보관 48파일 byte/SHA 확인. 다음 단계는 scoped commit → main push → 해당 SHA CI 확인이며, 구현·측정 작업으로 넘어가지 않는다.
