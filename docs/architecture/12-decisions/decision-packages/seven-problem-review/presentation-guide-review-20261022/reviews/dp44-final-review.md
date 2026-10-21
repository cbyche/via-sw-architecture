# DP44 v9 최종 7쪽 독립 재검토

- 작성일: 2026-10-22
- 검토자: `/root/review_existing_deck_guide`
- 검토 대상 commit: `30a5cd1667b1ecb81db1eaa2d17de3b33a84aa8f`
- 판정: **초안 D01–D05 해결 확인. 이번 검토 범위에서 남은 P1/P2를 발견하지 못함.**
- 파일 및 저장소 수정: 없음. 이 검토 기록과 XML 확인 결과만 `/private/tmp/dp3-dp44-whole-review/`에 기록했다.

## 1. 실제 확인 대상

- 최종 PPTX: `docs/presentations_files/dp1-dp4-reference-style-20261019/VIA-DP1-DP5-DP44-reference-style-36slides-v9-windows.pptx`.
- 직접 계산한 SHA-256: `ff3e7849df698c49e97e72a9e2a4dc61858b57d07b72a07702951dcaf87ad08d`. 저장소의 `dp44-v9-final-checks.json` 및 `dp44-v9-review.md`와 일치한다.
- 실제 최종 PPTX preview: `previews-v9-dp44/slide-24.png`부터 `slide-30.png`까지 **7장 전부 실제 열람**했다. 임시 초안 렌더로 최종 확인을 대신하지 않았다.
- 현재 source: `scripts/presentations/dp44_v9_pages.mjs`의 구조/edge/라벨/범례 및 반환선 집합.
- 현재 설계 문서: `docs/architecture/12-decisions/decision-packages/seven-problem-review/dp44-local-voice-execution-20261021/presentation-architecture-contract.md`.
- 현재 검증 기록: `dp44-v9-final-checks.json`, `dp44-v9-review.md`.
- 기준: 이전 독립 검토 초안에 기록한 `AGENTS.md` §9, `diagram-design-guide.md`, 기존 reference-style 체크리스트 및 보존 계약.

## 2. 기존 지적 닫힘

| ID | 초안 문제 | 최종 확인 결과 | 판정 |
| --- | --- | --- | --- |
| D01 | p25–27 반환선이 요청선과 같은 실선 | 실제 preview에서 audio/model 결과, chunk 반환, 상태/구간 반환, 출력 확인 경로가 점선이다. source의 반환 edge 집합과 PPTX XML의 dash를 대조했다. 요청/전달은 실선을 유지한다. | 해결 |
| D02 | p27 정정 입력이 노트에만 있고 실제 trigger 연결이 없음 | p27에 Response Registry / Revision Admission을 추가했다. Generation Lifecycle, Turn Execution 및 Playback Selection으로 실제 선이 연결된다. p26과 같은 인스턴스라는 의미는 설계 §7과 발표자 노트에 명시된다. | 해결 |
| D03 | p27 B self-loop가 구판 거절 문구를 가로지름 | 문구가 y480으로 이동하여 y465 루프와 분리됐다. 최종 PNG에서 텍스트 관통이 없다. | 해결 |
| D04 | p25 chunk 반환 라벨이 실제 선에서 떨어져 있음 | A 라벨은 x300/y612, B 라벨은 x1170/y610으로 이동했다. 최종 PNG에서 해당 점선 반환 구간과 연결해 추적할 수 있다. | 해결 |
| D05 | p26/27 그림 자체의 색·배타성·선 범례 없음 | 두 페이지 하단에 공통/A/B, A/B 배타 구성, 실선 요청/전달 및 점선 결과/복원 범례가 들어갔다. | 해결 |

이전 입력 확정 bypass도 다시 확인했다. 원격 S2S Runtime → Input Session Client → Input Finalizer → DP2이며, Local VAD 종료가 Input Finalizer에 연결된다. Runtime에서 DP2로 직접 우회하는 선은 없다.

## 3. 구조와 의미 확인

- A의 응답별 생성 준비/보관과 별도 재생 선택, B의 현재 turn 소유 생성/버퍼가 실제 Component, Module, RAM 상태 및 호출/반환으로 구분된다.
- 공통 원격 S2S Runtime과 VIA adapter, 업무 문안을 작성하는 Downstream Agent, 실제 출력과 제시 이력이 분리되어 있다. 모델 복제 또는 추가 문안 작성 LLM을 새로 도입한 표현은 없다.
- VAD의 로컬 재생 halt와 생성 취소를 같은 사건으로 합치지 않았다. 출력 위치 확인이 Delivery History로 돌아가며, 생성 완료 자체가 사용자 전달 기록으로 이어지지 않는다.
- p27의 구판/철회 tail 거절과, 유효한 판본의 단순 발화 전환 취소 시 tail 재사용을 문서와 노트에서 구분한다. B가 모든 기존 음성을 버리고 전체 재생성한다는 잘못된 강제 조건은 없다.
- p28–30에서 원 수치, 합성 수치 표시, QA01 UI와 별도 첫 음성 종료점, QA02/07 고정 정답 fixture, QA04 개념 Module 가정, QA03 인과 경로 경계, RAM/비용 한계, 조건부 A 권고 및 A1 미적용·미평가를 확인했다.
- 이 재검토에서 수정할 고신뢰 신규 P1/P2 구조·표현·레이아웃 결함을 발견하지 못했다. 본 판정은 구현 정확성이나 측정 성능을 입증하지 않는다.

## 4. 직접 수행한 패키지 표본 확인

최종 PPTX의 presentation relationship 순서로 실제 slide part를 찾아 XML을 읽었다. 이 패키지는 DP44 페이지에 `slide624.xml`부터 `slide630.xml`을 사용하므로 파일명의 숫자를 발표 페이지 번호로 가정하지 않았다.

| 발표 쪽 | native shape 수 | native table 수 | picture 수 | 텍스트가 없는 `node:` 도형 | 점선 edge 수 |
| --- | ---: | ---: | ---: | ---: | ---: |
| 24 | 42 | 1 | 0 | 0 | 0 |
| 25 | 108 | 0 | 0 | 0 | 7 |
| 26 | 104 | 0 | 0 | 0 | 10 |
| 27 | 92 | 0 | 0 | 0 | 8 |
| 28 | 12 | 2 | 0 | 0 | 0 |
| 29 | 12 | 1 | 0 | 0 | 0 |
| 30 | 22 | 1 | 0 | 0 | 0 |

새 `node:` 도형에 이름이 자체 text body로 들어 있는 것을 확인했다. SVG를 통째로 그림으로 넣은 페이지가 아니며 native 표를 유지한다. XML 직접 확인 결과는 `/private/tmp/dp3-dp44-whole-review/dp44-final-xml-check.json`에 있다.

저장소의 자동 검사 기록은 Windows typography 위반 0, 새 7쪽 native 개체와 수치 확인, 기존 29쪽 footer 밖 픽셀 보존 PASS를 보고한다. 이번 독립 재검토는 그 전체 검사 스크립트를 다시 수행한 것은 아니며, 최종 hash와 새 페이지 XML/이미지를 직접 확인했다.

## 5. 검증 범위와 남은 root 작업

- 이 재검토 범위는 **최종 v9 DP44 24–30쪽**이다. 기존 29쪽의 이전 독립 이미지 검토는 `v10-review-draft.md`에 별도 기록되어 있다.
- 최종 v10에서 root가 수정하는 기존 페이지와 도형 자체 텍스트 이관 결과는 아직 이 기록의 확인 대상이 아니다. 조립 후 실제 최종 렌더와 보존 대조가 필요하다.
- Microsoft PowerPoint 앱 자체 또는 Windows native rendering을 직접 실행하지 않았다. Open XML SDK 결과와 실제 PowerPoint 실행 여부도 서로 구분해야 한다.
- 새로운 VIA 구현, 모델 호출, 성능 측정 또는 synthetic backdata의 전면 재계산을 수행하지 않았다.
- D01–D05는 최종 v9에 대해 닫는다. v10에서 해당 7쪽을 보존하는 경우 동일성 검사가 이 결론의 적용 범위를 이어준다.
