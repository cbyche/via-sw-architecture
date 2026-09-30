# 8개 설계 후보 독립 검토 및 보완 기록

> 검토일: 2026-10-01 · 검토 시작본: `ede671ed0734f397f7ce3a7ed955c0700819dfd5`
> 범위: 개별 후보 8개, 배경·비교 그림 16개와 편집 원본, ASR·QA 표, 관련 target 계약
> 결과: 아래 발견사항을 수정하고 재검토했다. 후보 채택·ASR 동결·측정된 우위 판정은 아니다.

## 검토 방법

작성 에이전트와 별도로 세 검토 에이전트가 파일을 수정하지 않고 독립적으로 읽었다. 외부 사람이나 Senior Architect의 인증을 받은 것은 아니다. 역할별로 근거 위치·반례·최소 수정안을 보고하게 하고, 작성자가 수정한 뒤 같은 검토자에게 다시 대조하도록 했다.

| 검토 역할 | 확인한 내용 | 재검토 결과 |
| --- | --- | --- |
| 설계 계약 | 8개 대안의 실질적 메커니즘, target 책임·권한, 단계 수명, 확정·복구 원본 | 단계 import·이력 보관·제작 상태의 지적사항 해결 확인 |
| 품질 비교 | 8개 ASR·QA 표의 직접 인과, 공통 조건, 불리한 경우, 적용 범위·측정 의미 | 적용 분류·공정성 수정 확인; 정상 정정과 장애 trial 구별 문구까지 반영 |
| 구조도 | 16개 SVG 시각 검토, draw.io 연결·부모 구조, owner·제어·데이터 경로·색상 | 수정된 6개 비교도 재렌더링 대조에서 기존 의미 결함 해결 확인; 추가 제목 겹침 수정 |

작성자는 수정된 그림을 Chrome으로 다시 렌더링해 글자 경계와 배치를 확인했다. 도식의 XML·좌표 검사는 의미 검증을 대신하지 않으므로, 아래 사건이 그림과 본문에서 같은 경로를 밟는지를 따로 대조했다.

## 발견사항과 처리

| 대상 | 반례 또는 오해 가능성 | 보완 내용 |
| --- | --- | --- |
| 직접 응답: 출력 세대 | Policy Manager가 local output epoch를 소유하는 것으로 보임 | 정책 revision과 Interaction Manager의 output epoch를 분리하고 각 입력선을 수정 |
| 직접 응답: 전달 기록 | Channel I/O가 Response Manager를 거치지 않고 State Store에 receipt를 쓰는 것으로 보임 | Interaction Manager → Response Manager → State Store 기록 경로로 수정 |
| 직접 응답: 허용과 게시 | permit만 받고 생성한 내용이 host 검사를 거치지 않는 것으로 보임 | permit→binding, 생성 content→Request Controller, fence→release/stop 연결을 명시 |
| Agent 상태 | query가 실패할 때 필수 질문·완료도 알릴 수 없는 그림 | 질문·결과 결합→Request Controller→Response Manager의 독립 공통 경로 추가 |
| Context 획득 | 기본 read 전에 통합 해석을 시작하거나, ReadPlan을 최종 해석으로 오인 | 계획 호출·반환은 2안에만 두고 양안에 Final+receipt→통합 해석→proposal 반환을 표시 |
| 대화 Context | 새 delta가 없는 요청이 barrier에 들어갈 수 없거나 payload 전에 vector만 갱신 | 요청→Revision Read Barrier를 분리하고 변경→Delta Consumer→Projector→view/vector 원자 갱신을 명시 |
| 자료 표현 | FactView가 있으면 사용자 요청의 의미 해석 모델 작업이 없어지는 것으로 보임 | 소비자→Model Access 공통 경로와 2안의 추가 extraction 경로를 함께 표시 |
| 공통·차이 색상 | 공통 Agent Gateway는 파랑, 바뀐 ReadPlan 계약은 검정 | 복구의 공통 Agent Gateway는 검정, Context 획득의 다른 해석 계약은 양안 파랑으로 정정 |
| 해석 단계 수명 | attempt 종료 때 회수한 결과를 새 사용자 정정에서 재사용한다고 주장 | 완료 immutable 결과만 새 attempt에 검증·import; dependency 불명·crash 시 재계산, old job/KV/permit 재사용 금지 |
| 복구 이력 보관 | 파생 checkpoint를 믿고 prefix를 지운 뒤 checkpoint를 무효화하면 원본이 없음 | 필요한 prefix 보존, 저장 상한 admission 제한, 삭제 후 읽을 metadata 계약 명시; 권위 base를 만드는 compaction은 이번 대안에서 제외 |
| ASR 적용 | 악화 가능성이 있는 직접 경로를 “개선을 주장하지 않음”만으로 회귀에 둠 | P/R을 직접 인과 기준으로 정의; 직접 응답 QA-19·39, 해석 QA-39의 직접 비교 범위와 공통 회귀 범위를 구별 |
| Context 획득의 공정성 | 양안 공통 ReadReceipt·warm cache를 한쪽만의 장점으로 제시 | 공통 조건을 명시하고, 단절 전 선행 read와 계획의 실패 의존 조정이라는 실제 차이로 수정 |
| 대화 Context 정확성 | lag 자체가 검증을 뚫고 틀린 답변으로 나가는 것으로 오인 | 정상 read-set 검사는 대기·재조회·보류, dependency 누락·잘못된 통과는 정확성 실패로 구별 |
| 문서 제공 상태 | 이미 제공한 그림을 “후보 검토 후 제작”으로 표기 | 실제 제공 상태로 정정하고 README에 이 검토 기록을 연결 |
| 렌더링 | 신규 원자 갱신 설명이 그룹 제목과 겹침 | 제목·설명 baseline을 분리하고 생성기와 SVG/draw.io를 함께 재생성 |

## 확인한 범위와 남은 조건

- 로컬 검사 통과: 활성 Markdown 104개 링크, 활성 용어·정식 Component 이름, QA 카탈로그(23개 ID·4개 core ASR), target 13쌍·후보 16쌍의 도식 원본 일치 검사. 후보 도식 XML·identity·bounds·route 검사와 `git diff --check`도 통과했다. 원격 CI 상태는 이 변경의 GitHub 기록에서 확인할 수 있다.
- SVG 16개를 Chrome에서 렌더링하고 모듈 내 글자 넘침을 검사했다. 수정 경로는 이미지로 다시 확인했다. draw.io는 편집 가능한 XML의 부모·자식과 source/target 연결을 확인했으며 draw.io 애플리케이션에서의 렌더링까지 검증한 것은 아니다.
- 검토 범위에서 발견한 문제를 수정했다는 뜻이며 모든 설계 오류가 없음을 증명하지 않는다. 구조적 예상은 측정 결과로 승격하지 않는다.
- Native speech evidence, classification-only, held generation 등 provider 기능은 문서에 적힌 필요 capability다. 실제 지원·동시 진행·취소 동작을 이번 작업으로 확인하지 않았다.
- 구현·모델 실행·성능 측정·수치 freeze는 수행하지 않았다. Core ASR 변경, 후보 채택, 현재 대안의 우위도 확정하지 않았다.
- REVIEWED_BASELINE target, accepted/deferred ADR, archive의 기존 자료와 증거는 변경하지 않았다.
