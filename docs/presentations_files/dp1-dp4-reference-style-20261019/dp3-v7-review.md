# DP3 v7: SW Component와 실행 구조 그림 재작성

2026-10-21. 사용자 요청에 따라 DP3 10~12쪽 설계 그림을 다시 작성했다.

- [편집 가능한 PowerPoint v7](VIA-DP1-DP5-reference-style-29slides-v7-dp3-architecture-windows.pptx)
- 요청한 기존 `21slides-v5-r2-windows.pptx` 경로도 같은 v7 내용으로 갱신했다.
- 이전 v6와 수정 전 21장 snapshot은 보존했다. v6 검토 기록은 그 판본의 기록이다.

## 설계와 표현 기준

[INTENT](../../../INTENT.md), [그림 작성 기준](../../architecture/12-decisions/decision-packages/diagram-design-guide.md), [기존 발표 체크리스트](reference-checklist-and-review.md)를 읽고 DP1 p2, DP2 p6, DP4 p17의 실제 렌더를 대조했다. 연녹색 대안 머리, 옅은 노랑 Component 경계·제목 칸, 흰색 내부 Module, 소유 상태 원통을 계승했다. 파란 점선은 실제 OS process, 육각형은 외부 의미 모델, 실선/점선은 요청/반환으로 구분했다.

| 페이지 | 재작성 내용 |
| --- | --- |
| 10 | 원본 공동 해석 A와 후보 생산 B-R1의 책임. Request Controller의 허가·최종 채택, Context Manager의 원본 ref/cache, Request Interpreter의 각 해석 Module, Screen Target Producer의 후보 권위, Model Access의 cloud/local 실행. |
| 11 | B-R2 단일 process의 shared queue/runtime와 B-R3의 두 queue/runtime/장애 경계. UI 결과 host 반환 이후 조건부 SAM 실행, 동일 CPU 순차 admission. |
| 12 | host 실행 제어기와 worker IPC endpoint, adapter/runtime, model session/episode tensor/disk 파일. ReleaseModel과 CloseEpisode를 별도로 표현. 기존 RAM 원장 수치 보존. |

논리 책임 매핑은 [발표 구조 계약](../../architecture/12-decisions/decision-packages/seven-problem-review/dp3-grounding-architecture-20261021/presentation-architecture-contract.md)에 기록했다. Model Access의 여러 상자는 한 논리 Component의 분산 배치이며 독립적인 의미 채택 권위가 아니다. 원본/후보 범위 확대는 Request Controller의 허가를 받는다.

## 검토와 수정

독립 설계 검토자가 기존 v6와 새 실제 렌더를 대조했다. R3의 UI/SAM 동시 fanout처럼 보이던 흐름을 host 반환 후 조건부 요청으로 바꿨다. 후보 기록과 worker tensor를 분리하고, 모델 session을 worker 뒤의 실행 주체로 그리던 표현을 runtime 소유 상태로 수정했다. 최종 의미 채택은 Request Controller로 돌아온다.

첫 새 렌더에서 R3 상태 라벨의 넘침, p12 명령 출발점, RAM 표의 A 32MB 귀속과 시각 runtime 범위를 바로잡았다. A의 32MB는 cloud 입력, B의 24/32MB는 IPC다. Episode 읽기·갱신과 model load/use/release를 연결했다. 수정 렌더 재검토에서 새 P1/P2와 눈에 띄는 겹침은 보고되지 않았다. 마지막으로 p11에 동일 Component의 분산 배치, process 경계 연결의 IPC Endpoint 의미, 동일 heavy stage 순차 조건을 명시하고 주 작성자가 최종 렌더를 확인했다.

## 보존과 검증 범위

29장 모두 Artifact Tool로 렌더했다. 변경한 것은 10~12쪽뿐이며 나머지 26장의 XML·notes·relationships는 byte-identical, 렌더 PNG는 pixel-identical이다. QA 수치·시나리오·결론과 이전 DP3 appendix는 그대로다. 새 그림은 native PowerPoint 도형이며 표도 native table이다. 동일 scene 모델로 SVG/draw.io를 생성하고 XML 유효성을 확인했다.

Package integrity, layout geometry, Calibri font policy, native table 존재, Artifact Tool 재import가 통과했다. QA06 합계 592/1,954/2,374/2,662MB를 다시 확인했다. 이는 도형·패키지·보존 검증이며 Windows PowerPoint 실행, 실제 모델 추론이나 성능 측정은 하지 않았다.

[패키지 보존](dp3-v7-preservation.json) / [렌더·수치 보존](dp3-v7-render-preservation.json) / [파일 검사](dp3-v7-validation.json) / [수정 세 페이지 preview](previews-v7-dp3-architecture/).

생성기: `scripts/presentations/build_dp3_cpu_v7.mjs`, 패키징: `scripts/presentations/package_dp3_cpu_v7.py`. 재생성 입력은 독립 build의 `source-v6.pptx`로 복사한 보존 v6이며, 갱신된 legacy 경로를 v6 입력으로 사용하지 않는다. 모델 실행·측정·commit·push는 수행하지 않았다.

## GitHub 발행 전 형식 확인

AGENTS의 도형 라벨 규칙에 따라 새 10~12쪽의 Component·Module·원통·process 제목 등을 해당 native 도형 자체의 text body로 결합했다. 텍스트 사각형은 inset으로 유지했으며 원통 geometry에 따른 작은 내부 위치 차이는 최종 렌더에서 다시 확인했다. 새 페이지별 58/50/21개 라벨을 결합했고 기존 26장의 byte·pixel 보존은 재확인했다. v7 PPTX와 preview·hash·검사 보고서를 함께 갱신했다.

발행 전 local 검사에서 DP3 관련 링크와 그림 동기화·용어·PPTX 검사는 통과했다. 전체 active-link 검사에서는 이 변경 범위 밖의 기존 `four-dp-presentation-20261019` frozen source 복사본에서 14개 누락 상대 링크가 보고됐다. 해당 기존 근거 파일은 수정하지 않았다.

## Windows Open XML 검사 후 master registry 보정

발행한 v7을 직접 지정한 [Windows CI](https://github.com/cbyche/via-sw-architecture/actions/runs/39619401280)는 package를 열었으나 master registry 오류 3개를 검출했다. 이전 합본에서 계승하고 import가 추가한 `slideMasterId` 태그를 정식 `sldMasterId`로 보정했으며, PresentationPart의 notesMaster 등록을 규격상 허용되는 한 개로 정리했다. 실제 master part, 슬라이드, 노트, 개별 노트의 관계 파일은 그대로 보존했다. 패키징 생성기에 보정을 포함하고 새 검증 보고서를 생성했다. 29장 전체의 보정 전후 렌더 픽셀이 동일하다. Windows 재검사는 후속 커밋의 CI 결과로 확인한다.
