# 04-44. 지속 대화 실행 구조의 독립 리뷰 기록

> 2026-10-06 / 문서·구조·실제 MAIN 렌더 검토 / 구현·모델 실행·품질 측정 없음
> [본문](./04-44-continuous-interaction.md) · [MAIN](./diagrams/choice44-structure.svg) · [확대 보기](./diagrams/choice44-review.html)

## 0. 최신 사용자 수정 — 좌우 독립 블록 (2026-10-06)

사용자는 이전 MAIN의 공통 생산자와 모델이 양안 사이에 걸쳐 있는 배치를 거절하고 **A는 왼쪽 블록만, B는 오른쪽 블록만** 사용하도록 요청했다. 이전 세 차례 리뷰는 당시 배치에 대한 검토이며 이 후속 사용자 기준을 충족했다는 승인으로 읽지 않는다.

- MAIN의 Interaction Manager, Input Control Gate, Task Manager, Agent Gateway, Downstream Agent, Model Access와 Omni를 양안의 칸 안에 각각 배치했다. 각 안의 입력·명령·질문/결과·모델·실제 전달이 자기 블록 안에서 완결된다.
- 보충 사건도도 기존 상하 비교에서 좌우 독립 참여자/sequence로 바꿨다. 칸 사이의 연결이나 공통 실행 노드는 없다.
- 반복된 모델/Component는 서로 배타적인 대안의 같은 기능이며 모델 복제나 새로운 기능·process·권한·A/B 의미 변경이 아니다. Direct passthrough, currentness, actual receipt와 Guard credit/State snapshot 계약을 유지했다.
- 전용 생성기에서 모든 participant가 A/B 자기 범위 안에 있고, 모든 연결의 양 끝과 중간 경로가 같은 칸 안에 있는지 검사한다. SVG/draw.io/PNG를 함께 갱신하고 두 실제 렌더와 720p 축소본을 확인했다. 실제 text bounding boxes에서 텍스트 겹침·canvas 밖·자기 블록 밖 텍스트는 두 SVG 모두 0건이었다.

- presentation_critic가 최신 메인/사건도의 실제 720p를 먼저 읽고 본문·source와 대조해 새 좌우 독립 요구 충족과 주요 경로/명칭 보존을 확인했다. 이 수정 범위에서 새 P1/P2는 보고되지 않았다. 전용 생성기와 활성 용어/링크/QA 검사를 통과했다. 이전 세 리뷰의 판정을 이번 배치에 자동 적용한 것이 아니다.

## 1. 사용자 요청과 완료 범위

사용자는 04-50 영역 6에서 계속 듣기, 현재 요청의 응답, 기존 업무의 질문/결과와 정정을 한 대화로 잇는 SW 구조를 44로 완성하도록 요청했다. A/B는 전략·알고리즘·정책 차이를 넘어 architectural style에서 달라야 하고, 실제 발표는 **메인 구조 비교 한 장**만으로 끝나야 한다. 완성 → 제3자 리뷰 → 수정 → 재리뷰와 이번 작업 파일만 commit/push하는 범위를 승인했다.

44는 **A 중앙 비동기 Orchestration/Mediator**, **B 반응형 Dataflow/Pipes-and-Filters**로 정제했다. 중앙 명령/반환/continuation 상태를 producer/consumer activation, 국소 windows, join, credit/cancel 계약으로 실제 이전한다. A는 async·priority·cache/prefetch·부분 수정·handler·후보 병행 준비를 모두 허용한다. 두 안에 같은 입력·의미 생산·Task 원본·직접 후보 admission·실제 전달·Omni/ASR·process/storage 조건을 적용한다.

34는 40번대로 승격하지 않고 설계 참고로 유지한다. 41의 의미 해결 주체, 42의 서비스/원본 수명, 43의 의미 생산 분해와 구별한다. 기존 후보의 본문/그림, 참조 Architecture, QA/ADR와 구현·측정 상태를 변경하지 않았다. 병행 세션의 45 작업은 별도다.

## 2. 리뷰 방식

세 개의 독립 agent context가 **읽기 전용**으로 검토했다. 실제 사람 심사위원이나 실측 검증을 뜻하지 않는다.

| 리뷰 | 초점 | 확인 대상 |
| --- | --- | --- |
| architecture_challenge | 실제 style 차이, strong A, 상태/소유 계약, 가장 싼 전환과 혼합 반론 | 본문·기준 문서·MAIN 실제 1280×720 렌더·보충 사건도 source |
| presentation_critic | 그림만 먼저 보고 한 장 발표가 가능한지, 구조/상태/입력·출력과 품질 인과, 배치·글자 | 실제 MAIN 렌더 먼저, 이후 source와 본문 대조 |
| independent_jury | 기능 완결성, 경합/권한/현재성, 비교 공정성과 주요 후보의 조건부 논거 | 본문·MAIN 실제 1280×720 렌더·보충 사건도 source/경로 |

세 관점을 병행 검토했지만 수정은 작성자가 순차 적용했다. 그림만 읽은 판단을 먼저 받았으며 본문으로 그림의 누락을 덮지 않았다. 최종 비교 상태는 **미선정 정제 후보**다.

## 3. 1차 리뷰와 보완

P1은 보고되지 않았다. 아래는 중복 지적을 내용별로 모은 P2와 수정이다.

| 발견 | 실제 문제 | 보완 |
| --- | --- | --- |
| 미해결 입력의 확인 질문 | 질문에 답해야 의미가 해소되는데 후보 게시가 InputSettled를 기다려 순환 대기 | InteractionNotice(u, question ID, admission class). 현재 입력/질문/권한/발화 종료로 제한 게시하고 외부 효과 hold 유지 |
| 최초 게시와 receipt | 자기 candidate의 실제 receipt를 선행조건으로 읽을 수 있었고 초기 상태 불명 | 과거 확정 snapshot과 known-empty 초기화, UNKNOWN 복구, 자기 receipt는 게시 후 동일 publication ID의 actual-range 갱신으로 분리 |
| stop/hold의 외부 효과 경계 | Gate가 sink에만 닿고 admission epoch 확인만으로 전송 경쟁을 해결하지 못함 | MAIN Gate→Task hold; Request Controller admission/hold, Task Manager command/epoch, Agent Gateway transmission CAS를 같은 local transaction으로 확정. DISPATCHING 선행이면 실제 전송/UNKNOWN 보존 |
| B 제어 출처 | Join→Input의 u2/cancel이 새 입력 생산처럼 보임 | Gate→Reactive Runtime priority control barrier. 역방향선은 소비자의 credit/cancel. 사용자 Task 취소와 구별 |
| 현재성과 실제 제시 소유 | eligibility 이름만 있고 source revision/최종 검사/focus 소유가 충분히 보이지 않음 | Task Notice→Join source revisions, Output Arbiter 안의 Publication Guard/Presentation State, owner 현재성 조회와 실제 receipt 경로 |
| Direct admission | Voice Runtime의 후보가 좁은 허용 검사 없이 중앙/Join에 직결 | Voice Runtime→Request Controller admission을 양안 공통으로 명시 |
| 주요 입력/헤더 중첩 | B Runtime/입력 라벨, A 입력선/header 관통, Task 경로와 Join 라벨 혼잡 | 독립 입력 경로를 owner 헤더 밖으로 재배치하고 라벨/여백 조정. 실제 720p로 재검수 |

## 4. 2차 재리뷰와 보완

1차의 확인 질문·초기 전달·hold/CAS 계약은 해결 확인을 받았다. architecture_challenge는 새로운 P1/P2를 찾지 못했고, 나머지 관점에서 다음 P2를 추가 발견했다.

| 발견 | 실제 문제 | 보완 |
| --- | --- | --- |
| 승인된 S2S 후보의 후속 전달 | admission까지만 있고 기존 Text/audio가 sink에 도달하지 않아 Core에서 재생성하는 그림으로 읽힘 | AdmittedDirectCandidate를 A의 후보 반환→Dispatcher→Guard로 전달. B의 Response Manager adapter는 기존 후보 passthrough→Join→Guard. 본문과 MAIN에 Direct 통과 명시 |
| credit의 생산자 | Presentation State→Join의 snapshot/credit이 저장소를 admission authority처럼 표현 | Publication Guard→Join credit과 Presentation State→Join past snapshot을 서로 다른 실제 edge로 분리 |
| 원통 이름 관통 | 낮은 Presentation State 원통의 곡선이 이름을 가로지름 | 원통 높이 64, owner 높이 200으로 확대해 곡선 사이에 이름 배치 |

작성자도 Task 아래 설명과 Q1/r1의 겹침, 바닥 legend와 Interaction Manager의 겹침을 발견해 제거했다. 참조 §5의 잘못된 heading anchor도 고쳤다. 보충 사건도는 설명표에서 이름이 있는 실제 참여자의 sequence로 재작성했다.

## 5. 3차 최종 재검토

세 리뷰는 **현재 본문·MAIN에서 남은 P1/P2를 발견하지 못했다**고 회신했다. architecture_challenge는 새 보충 사건도와 계약 사이의 실질 불일치도 발견하지 못했다. independent_jury의 보충 사건도 확인은 source/경로에 한정되며 실제 보충 PNG를 직접 확인한 것으로 기록하지 않는다.

공통 최종 판단은 다음과 같다.

- A의 중앙 명령·반환 회수·후속 상태와 B의 직접 소비자 활성화·국소 window·join·feedback은 실제 architectural style 차이다.
- 공통 단일 sink는 B 전체를 중앙 orchestration으로 바꾸지 않는다. Guard가 모든 준비 순서를 정해 worker를 부르면 별도 혼합이다.
- Direct 후보, 실제 질문 focus, owner 현재성, credit 생산자와 hold/전송 경합이 본문과 MAIN에서 일치한다.
- 강한 A의 async·priority·부분 재사용과 작은 혼합 반론을 유지했다. B의 무료 병렬 추론·자동 의미 정확성·process 장애 격리를 주장하지 않는다.
- 720p에서 핵심 이름과 주 흐름을 읽고, 중앙 왕복/전이 집중과 B의 직접 분기/추가 windows·join 비용으로 품질 인과를 설명할 수 있다. m/t/v는 보조 서비스 포트다.

이는 검토 범위 안에서 발견한 결함을 해결했다는 판단이다. **B의 실제 품질 우위, 최종 DP/A/B 선택, 결함이 전혀 없음, 사람 심사 결과 또는 41~43 대비 객관적 우위의 인증이 아니다.**

## 6. 작성자 검수와 재현 범위

- [전용 생성기](../../../../scripts/architecture/generate_continuous_interaction_diagrams.py)가 MAIN/사건도의 SVG·편집 가능한 draw.io를 같은 scene에서 생성한다. `--check`는 XML, ID/참조, owner·canvas, 직교 배선, 다른 node 관통, 이름만 있는 header와 생성물 일치를 검사한다.
- MAIN/사건도의 2560×1440 PNG를 macOS Chrome의 실제 SVG/font 렌더에서 생성하고 눈으로 확인했다. MAIN은 별도 1280×720 축소본도 검토했다. SVG와 PNG의 마지막 수정 상태를 동기화했다.
- 같은 브라우저의 실제 text bounding boxes로 두 SVG의 텍스트 간 겹침 및 canvas 밖 텍스트 **0건**을 확인했다. 이는 배치 보조 검사이며 배선의 뜻·구조 우위·모든 해상도의 가독성을 자동 판정하지 않는다.
- 구조도는 이점 문장/점수 패널 없이 명령·자료·조건 화살표, 이름만 있는 Component/Module, 상태 원통으로 구성했다. 메인 16:9 한 장과 본문의 같은 사건·소유·V-01~13·조건/반증을 맞췄다.
- 저장소의 활성 용어/151개 Markdown의 local 링크/QA catalog, target·기존 package·Stage 4·mechanism·functional·여섯 비교·41/42/43/45 diagram checks와 새 44 check를 모두 통과했다. 새 44 check를 CI에 추가했다. 구체적 실행 결과는 최종 게시 응답과 GitHub CI에서 확인한다.

다른 후보/참조 설계의 품질 결과를 가져오지 않았고 Architecture candidate를 실행하지 않았다. 후속 측정의 harness/freeze·수치·장비·승자 선정은 이번 요청의 범위가 아니다. 이번 게시 파일의 세부 범위는 해당 Git commit이 원본이다.
