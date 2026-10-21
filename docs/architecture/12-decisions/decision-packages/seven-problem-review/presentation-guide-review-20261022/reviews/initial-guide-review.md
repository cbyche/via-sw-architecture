# 통합 발표자료 v10 독립 가이드 검토 초안

- 작성일: 2026-10-22
- 검토자: `/root/review_existing_deck_guide`
- 상태: **중간 검토 기록. 최종 v9/v10 PPTX 검증 완료 기록이 아님**
- 목적: 기존 29쪽과 추가 DP44 7쪽의 가이드 준수, 실제 구조 가시성, 보존 계약을 점검하고 root의 최종 검토 기록에 근거를 제공한다.
- 작업 경계: 저장소 파일을 직접 수정하지 않았다. DP5 수정 Scene과 검토 렌더는 `/private/tmp/dp3-dp44-whole-review/` 아래에만 작성했다. 최종 통합, 검증, commit 및 push는 root 또는 DP44 작성 세션의 작업이다.

## 1. 적용한 기준과 읽은 자료

1. `INTENT.md`: 실제 구조 차이를 통한 QA trade-off, 기본 설계와 보완 구분, 정확성 QA 경계, 근거 상태 및 선택 상태를 보존한다.
2. `AGENTS.md` §9: Windows PowerPoint 전달, 선언 글꼴 Calibri, 정수 PowerPoint pt, `ko-KR`, 주 본문 검정, native 개체 유지, **도형 자체 text body에 이름을 넣는 규칙**, 기존 판본 보존, 정책 검사와 실제 영향 페이지 렌더, Open XML SDK와 PowerPoint 자체 실행의 구분.
3. `docs/architecture/12-decisions/decision-packages/diagram-design-guide.md`: 이름은 Component/Module 상자, 동작과 조건은 선, Module의 실제 소유 Component 포함, RAM/영속 상태, 실제 요청/반환 endpoint, 반환 점선, 정정/확장/재허가 루프, 생산자/소비자/상태 소유권, 색 범례, 모델 연동 책임과 모델 의존성 구분.
4. `docs/presentations_files/dp1-dp4-reference-style-20261019/reference-checklist-and-review.md`: 기존 reference-style의 배경 → 비교 → 선택/보완 → 상세 흐름, 선택 열 강조, 대안 순서와 비선택 대안 보존, 장단점과 QA 행, 보완 비용/한계 공개, 편집 가능성, 시각 검토 기준. 이 문서의 **16쪽 및 이전 글꼴/QA 정의는 당시 캠페인의 기록**이다. 현 36쪽, Windows 글꼴 규칙, DP3와 DP44 확장 구조에 그대로 덮어쓰지 않았다.
5. 기존 DP5 source와 설계: `scripts/presentations/add_dp5_reference_style_v5.mjs`, `docs/architecture/12-decisions/decision-packages/seven-problem-review/dp5-delegation-memory-20261020.md`.
6. DP44 source와 설계: `scripts/presentations/dp44_v9_pages.mjs`, `scripts/presentations/dp44_v9_scene.mjs`, `scripts/presentations/build_dp44_v9.mjs`, `docs/architecture/12-decisions/decision-packages/seven-problem-review/dp44-local-voice-execution-20261021/presentation-architecture-contract.md`.
7. Presentations skill: `/Users/changbae/.codex/plugins/cache/openai-primary-runtime/presentations/26.1017.22135/skills/presentations/SKILL.md`.

Reference style는 기존 통합본의 native 개체 및 실제 PNG, 위 체크리스트를 기준으로 대조했다. 이번 독립 검토에서 원본 참고 발표자료의 모든 슬라이드를 다시 개별 열람한 것은 아니다.

## 2. 실제 확인 범위

| 범위 | 실제 확인한 대상 | 확인 방법과 한계 |
| --- | --- | --- |
| 기존 v8 1–9, 13–29쪽 | `previews-v6-dp3-cpu/slide-XX.png` 26장 | 각 PNG를 실제 열람했다. v8 보존 기록에서 이 26장은 v6와 같은 렌더임을 확인하여 사용했다. |
| 기존 v8 10–12쪽 | `previews-v8-dp3-guide-fixed/slide-10.png`부터 `slide-12.png` | 세 장을 추가로 실제 열람했다. 모델/worker/IPC/상태 수명, 확장·재허가 흐름을 확인했으며 이번 검토에서 고신뢰 신규 결함을 찾지 못했다. |
| 기존 v8 29쪽 패키지 | `VIA-DP1-DP5-reference-style-29slides-v8-dp3-guide-fixed-windows.pptx` | XML을 통한 도형 이름/좌표/텍스트 소유 관계와 기존 보존·검토 기록을 확인했다. 이 단계에서 최종 v10 패키지 검사를 수행한 것은 아니다. |
| DP5 수정 초안 p21/p22 | `/private/tmp/dp3-dp44-whole-review/dp5-page-21.png`, `dp5-page-22.png` | 실제 native renderer PNG를 검토하고 라벨을 수정한 뒤 재렌더했다. 최종 수정 PNG도 다시 열람했다. 아직 원본 native 표와 결합한 최종 패키지 렌더는 아니다. |
| 추가 DP44 초안 p24–30 | `/private/tmp/dp3-dp44-whole-review/dp44-review/page-24.png`부터 `page-30.png` | 현재 source를 개인 임시 폴더에 복사하고 DP44 build의 동일 native renderer로 7장을 렌더하여 전부 실제 열람했다. 작성 중 source의 스냅샷이므로 최종 v9 PPTX와 같다고 확정하지 않는다. |

기존 29쪽 전체와 DP44 초안 7쪽 전체에 대한 이미지 검토를 수행했다. **최종 통합 36쪽 PPTX의 전체 렌더 검토와 동일한 진술은 아니다.** v9에서 기존 appendix 24–29쪽은 DP44 7쪽 뒤인 31–36쪽으로 이동한다. 실제 최종 페이지 대응은 패키징 완료 후 다시 확인해야 한다.

## 3. 기존 페이지에서 확인한 수정 사항

### G01 — 도형 이름의 자체 text body 소유 위반

- 위치: 기존 26쪽 전반, 특히 p2/3/6/7/17/18/21/22의 Component 및 내부 도형.
- 실제 근거: v8 `ppt/slides/slide2.xml`의 `Request Interpreter Component`(id 27)는 이름을 담지 않은 배경 도형이고, 별도 투명 textbox `DP1 object 67`(id 28)이 `Request Interpreter`를 겹쳐 표시한다. 내부 처리 상자도 같은 방식이다.
- in-memory 인접 pair 탐지에서는 기존 26쪽에 422개가 발견되었다. 이 수에는 매쪽 navigation chevron 5개가 포함되며, 모든 pair가 독립적인 사용자 보임 오류라는 뜻은 아니다. p10–12는 이 탐지에서 0개였다.
- 수정 방향: native 도형의 자체 text body로 라벨을 옮기고 overlay textbox를 제거한다. flow annotation은 별도로 보존한다.
- 상태: root 담당 수정. 독립 검토자는 문제와 탐지 근거를 전달했으며 최종 XML/렌더 확인은 아직 수행하지 않았다.

### G02 — p21 DP5의 실제 조회·반환·보완 루프 부족

- A는 `필요 범위 추가 읽기` 설명이 있으나 실제 Controller 허가 → Context Manager 조회 → 결과 반환 → 해석 재개를 추적하기 어렵다.
- B의 부족 근거 보완 dotted edge는 원본 상태 쪽에서 끝나 실제 반환과 재허가·재개가 닫히지 않는다.
- 수정 방향: 기존 기능을 유지하면서 Module endpoint를 연결하고, 허가 범위 안의 원본 보완과 더 넓은 범위의 Controller 재허가를 구분한다. 관계 생산 경로의 원본 조회/반환도 연결한다.
- 상태: 독립 검토자가 임시 Scene 수정본 작성 및 렌더 검토 완료. root가 최종 통합할 예정이다.

### G03 — p22 DP5 보완 도식의 이름/동작/상태 구분 부족

- A1/B1 보완 그림이 Module 이름 대신 동작 문장을 큰 사각형으로 표시하고, 추가 색인 및 RAM 대기 사건의 보관 형태가 드러나지 않는다.
- 수정 방향: Component 안의 이름 있는 Module, 상태 원통, 동작 화살표로 최소 재작성한다. A1의 요청 간 색인 보존과 B1의 RAM 대기 사건, 미게시 범위의 원본 보완을 표현한다.
- 상태: 독립 검토자가 임시 Scene 수정본 작성 및 렌더 검토 완료. 원본 선택표/장단점/미선택 결론과 결합한 검토는 아직 남아 있다.

### G04 — p2/3, p6/7의 핵심 공통 경계 가시성

- p2/3: Request Controller의 허가·최종 채택이 각주 중심이라 주요 조회/반환 경로에서 실제 권한 경계를 추적하기 어렵다.
- p6/7: 공유 모델 연동과 호출 한도 K가 각주에 머물러, 대화 실행과 요청별 실행의 병행성이 모델 복제인지 공유 실행인지 그림만으로 확인하기 어렵다.
- 수정 방향: 원래 설계와 측정 수치를 유지하면서 실제 공통 경계를 펼친다. model/프로세스 복제를 새로 만들지 않는다.
- 상태: root에게 전달. 다른 담당자가 수정 중인 결과를 이 검토자가 아직 최종 재검증하지 않았다.

기존 26장 초기 이미지 검토에서 대규모 잘림, 표 파손, 내용 소실은 발견하지 못했다. 이 결론은 G01의 구조적 편집 정책 위반을 없애지 않으며, 수정 후 최종 검토를 대체하지 않는다.

## 4. DP5 임시 수정본과 보존 계약

수정 source: `/private/tmp/dp3-dp44-whole-review/dp5_v10_grounding_loops.mjs`.

- `dp5GroundingLoopsPage(Scene)`: p21 그림 영역 `{x:24, y:180, w:1872, h:773}`. y959 이하 원본 구조적 대가 native 표와 footer는 교체하지 않는다.
- `dp5TacticFragmentsPage(Scene)`: p22 `{x:75, y:606, w:810, h:168}` 및 `{x:1025, y:606, w:810, h:168}`. 위 판단표, 보완 제목, 장단점, 미선택 결론, 하단 주의 문구는 교체하지 않는다.
- 자동 Scene 검증: p21 22 nodes / 25 edges, p22 10 nodes / 7 edges. Component containment, Module owner, endpoint, 직교 경로, 무관한 노드 관통, 금지 구두점 검사를 통과했다.
- 실제 렌더 검토: p22 return 경로와 라벨이 겹치던 부분을 고쳤고, 최종 PNG에서 고신뢰 추가 잘림/텍스트 관통을 발견하지 못했다.

보존한 계약:

1. QA 수치, 설계 질문, 대안 기능, 원본과 확정 사실의 기존 소유 권위, 기본안 미선택 상태를 변경하지 않았다.
2. 원본 소유자는 공통 외부 참조로 남기며 Context Manager가 새 권위를 갖는 것으로 옮기지 않았다.
3. A는 요청 경로에서 관계를 해석한다. B는 사건 갱신 경로에서 관계를 생산하고 요청 시 조회·적용하며 원본 부족 시 보완한다.
4. B의 생산과 요청 경로를 분리하되 기존 원본 확인/권한/현재성/최종 채택 기능을 보존했다.
5. A1은 `요청 간 보존`으로 표현했다. 문서가 보장하지 않은 디스크 내구 저장이나 crash recovery를 새로 주장하지 않았다.
6. B1 추가 대기열은 RAM으로 표시했다. 공통 관계 생산·조회·원본 조회 Module과 추가 조정 Module을 색으로 구분했다.
7. 새 process, 추가 모델 복제, 의무 호출 횟수, 개선 수치, 새 선택 결론을 만들지 않았다.

## 5. DP44 초안 검토 결과

### 확인한 구조·근거 상태

- Voice Input Controller 내부 Input Finalizer를 거쳐 인식 결과와 발화 종료를 확정한 뒤 DP2 경계로 전달하도록 source와 실제 렌더에 반영되어 있다. 원격 Runtime이 DP2를 직접 호출하는 이전 우회는 이 스냅샷에 없다.
- 로컬 VAD, 재생 제어, 장치 출력, 실제 출력 위치와 Delivery History 경로가 구분된다. 생성 callback을 실제로 들은 내용으로 기록하는 경로는 보이지 않는다.
- A의 응답별 준비 job 및 별도 재생 선택, B의 현재 turn에 속한 생성/버퍼, 공통 원격 모델 adapter가 다르게 펼쳐져 있다.
- QA01 화면 값과 별도 첫 음성 종료점을 구분했다. QA02/07은 고정 정답 조건이며 모델 정확도 실측 증거가 아님을 표시했다.
- QA04는 개념 Module 변경 수 가정이며 실제 Rust diff가 아님을 표시했다.
- 합성 수치 표시는 p28–30에 존재한다. QA03 인과 계산, QA05 폐기와 재개 비용, QA06 로컬 RAM 한계, 조건부 A 권고와 A1 미적용·미평가 상태를 보존한다.
- p28–30에서 수정할 고신뢰 신규 표/수치/레이아웃 결함을 발견하지 못했다. 이는 backdata 재계산이나 실측 검증을 수행했다는 뜻이 아니다.

### D01 — 반환선이 요청선과 같은 실선

- 위치: p25–27. source의 `e()` helper가 `dashed`를 지정하지 않아 `a-return`, `b-return`, `a-read-return`, model result 및 상태 반환 등이 모두 실선으로 렌더된다.
- 기준: diagram-design-guide §5.1의 요청/전달 실선, 결과/복원 점선 구분.
- 수정 방향: 실제 결과/상태/거절 반환 경로만 점선으로 전환하고, 요청/전달 경로는 실선을 유지한다.
- 상태: root가 독립 확인한 뒤 DP44 작성 세션에 수정 요청을 전달했다고 통지받았다. 수정 완료 결과는 아직 미검증이다.

### D02 — p27 정정 입력과 상태 무효화의 시작 경로 부족

- p27은 정정 반영/생성 취소/재개를 다루지만 Registry에서 채택된 판본이 각 실행 책임으로 들어오는 실제 경로가 없고 노트에만 설명되어 있다.
- 현재 그림만 보면 Generation Lifecycle이나 Turn Execution이 어느 채택 사실을 받아 현재 판본을 바꾸는지 추적하기 어렵다.
- 수정 방향: 같은 Revision Admission의 참조 또는 실제 Module을 명시하고 A/B의 판본 반영·취소·유효 재생 경계로 잇는다. 기존 범위/권위 계약을 유지하고 새 정책을 만들지 않는다.
- 상태: root가 확인 후 DP44 작성 세션에 수정 요청을 전달했다고 통지받았다. 최종 경로는 아직 미검증이다.

### D03 — p27 B self-loop가 거절 문구 관통

- 실제 PNG에서 `4 구판 chunk는 거절 후 종료` 텍스트를 self-loop 수평선 y465가 가로지른다.
- 수정 방향: 루프 또는 해당 설명을 이동하고 실제 렌더에서 재확인한다.
- 상태: root에게 전달, 미재검증.

### D04 — p25 chunk 반환 라벨이 자기 선에서 분리

- A의 `5 chunk 반환`은 실제 반환선 x440/y590과 떨어진 y650 중앙에 있다.
- B의 `4 chunk 반환`도 실제 x1410 반환선에서 멀리 x1490..1790에 있다.
- 기준: diagram-design-guide §5.1의 라벨을 설명하는 선 구간 가까이에 놓고 요청/반환을 각각 추적할 수 있어야 한다는 규칙.
- 수정 방향: 반환선의 빈 수평 또는 수직 구간 가까이에 라벨을 붙인다. 다른 경로를 가리거나 기존 Component 이름 위에 놓지 않는다.
- 상태: root에게 전달, 미재검증.

### D05 — p26/27 그림 자체 범례 누락

- p25에는 공통/파랑 A/초록 B 및 배타적 구성 범례가 있으나 p26/27에는 페이지 자체의 범례가 없다.
- 수정 방향: 공통/A/B와 배타적 구성, 실선/점선 의미를 짧게 표시한다. p26의 두 controller가 동시 실행된다는 오해를 줄인다.
- 상태: root에게 전달, 미재검증.

## 6. 보존해야 할 내용

- 현행 DP 숫자와 발표 번호, 선택/조건부 권고/미선택/미적용 상태를 유지한다.
- 기존 historical appendix와 당시 증거를 없애거나 현행 데이터로 바꾸지 않는다. 이전 정의, 모집단, 측정 범위 및 근거 제한을 원래 상태로 남긴다.
- 원 수치와 분모, 합성/실측 여부, callback/UI/audio 종료점 구분, 전체 경과시간과 VIA 귀속시간 구분을 보존한다.
- DP44 고정 정답 fixture의 100%를 정확도 측정이나 일반 정확도 우세로 서술하지 않는다.
- DP44의 A 조건부 권고는 accepted ADR 또는 측정 승자가 아니다. A1도 미적용·미평가 상태다.
- ASR, shared model, worker, Controller 권위 등 현행 설계 계약을 그림 개선을 이유로 새로 만들거나 바꾸지 않는다.
- 원본 native table을 그림으로 대체하지 않는다. 편집 가능한 PPTX, Scene source, SVG/draw.io와 문서의 책임/상태/흐름을 동기화한다.

## 7. 최종 검증에 남은 작업

1. DP44 작성 중 수정이 끝난 source/PPTX를 받고 D01–D05의 실제 해결 여부를 다시 검토한다.
2. root가 조립한 최종 36쪽 패키지에서 수정 페이지와 이전 보존 페이지의 텍스트, native 표, 수치, 노트, appendix 대응을 비교한다.
3. DP5 두 fragment를 원본 표·제목·각주와 결합한 실제 최종 렌더에서 확인한다.
4. 도형 자체 text body 이관 결과를 XML과 실제 렌더에서 확인한다. 인접 pair 수가 줄었다는 사실만으로 시각 검증을 대신하지 않는다.
5. Windows policy, package integrity, layout, 다시 import, 필요 native table 및 모든 관련 preview 검사를 최종 PPTX에 수행한다.
6. 가용한 Windows Open XML SDK 검증과 실제 PowerPoint 설치/실행 여부를 별도로 기록한다. 현재 독립 검토자는 최종 Windows PowerPoint 렌더를 수행하지 않았다.
7. 최종 source/설계 문서/SVG/draw.io/PNG/PPTX와 리뷰 기록의 일치 및 commit/push 결과는 root가 확인한다.

**현재 결론:** 가이드 위반과 누락 흐름을 구체화하고 DP5 수정 source를 준비했으며 기존 29쪽 및 DP44 초안 7쪽의 실제 이미지 검토를 마쳤다. DP44는 아직 작성 중이고 최종 v9/v10 통합 패키지의 수정 완료·전체 검증·publication 완료는 이 기록에서 확정하지 않는다.
