# 03-02와 두 QA 발표자료 교차 검수 — v6

2026-10-11 사용자 지적과 승인한 개선 방향에 따라 작성자가 사례 선정 → 공통 모집단 → 목표/산식 → 두 발표자료를 반복 대조했다. 독립 에이전트 심사나 실제 후보 시험을 뜻하지 않는다.

원본은 [03-02](./03-02-quality-attribute-definitions.md), 입력은 [P21](./03-02-common-workload.json), 전체 대응은 [manifest](./03-02-workload-manifest.json)다. 발표는 [요약 6장](../../../presentations_files/quality-attributes/README.md)과 [QA 부록 6장](../../../presentations_files/quality-metrics/README.md)이다. 이전 v5의 38파일은 [보존본](../../../archive/qa-common-workload-before-v6-20261011/README.md)에 bytes/hash로 남겼다.

## 1차: 사례 선정과 누락

| 발견 | 수정 | 검수 |
| --- | --- | --- |
| 균등 3개/상황군의 근거가 약함 | 기제/오류 경로로 2/5/3/3/4/4개 선정 | 기존 17개 checkpoint oracle 보존, 원래 S1-03은 M-06 기능 회귀, 4개 추가 |
| DP에 중요한 재사용/최초 생산/집중/정상 답변이 대표 관측 밖에 있음 | S2-04/05, S5-04, S6-04를 채점 사례로 편입 | 41/42/44/45마다 구조 민감 사례 포함, strong A/cache/병렬과 B 추가 비용 유지 |
| 신규 질문/결과의 payload가 모호 | 질문 원문, Execution/revision, 결과 14일/20개와 권한 고정 | 입력·사건·기대값·6조건으로 판별 가능 |
| 내부 구조를 정답으로 강제할 위험 | 원본/게시 경로 선택은 구조 증거로 분리 | 같은 목표/사실/허용 결과를 정확성으로 채점 |

## 2차: 한 묶음과 모든 QA 대응

| 발견 | 수정 | 검수 |
| --- | --- | --- |
| 시간의 각 6개 선정/나머지 사례 제외가 불충분 | 21개 전체 대응표와 각 사건/사용자 목표 관측 | 반응 27개, 사이클 19개. Agent 단독 2개만 새 목표 없어 QA-03 제외 |
| 재사용/집중 부하의 정확성 제외 | 모든 실행의 C1~C6와 최종 tail 채점 | 105시나리오/630조건, 비용·메모리와 같은 활동 |
| 내부5회/외부3회/첫 run만 채점 혼동 | P21 전체만 5회 독립 초기화 | 반응135/처리95, 비용5회 평균/메모리5회 최대 |
| cache의 순서/정답 준비가 불명확 | 실제 S2-02→04, 늦게 오는 S2-05 원천, 동일 before-state | 선행 실패 덮기/파생 정답 선주입 금지, provider cache 별도 기록 |
| Task 10개가 미답변 기록 삭제로 오해될 수 있음 | 동시 처리 슬롯과 논리적 미완료 기록 구별 | S5-04 동시7개, 이전 scope의 상태/이력/할당 보존, 준비/회수도 원장 |

## 3차: 목표와 수치의 교정

| 발견 | 수정 | 검수 |
| --- | --- | --- |
| 변경된 정확성 분모에 이전 문턱이 남을 위험 | 618/630, 전체/조건100/105와 상황군별 정수 목표 | 정수 경계와 등급 검산 |
| 동시에 도착한 사건을 2초 안에 모두 음성 제시하도록 요구하면 모순 | 일반115관측 중110개 ≤2초, 집중30개 전부 ≤5초 | 평균1초는 유지, stop10개 모두0.1초, 순차 voice queue 포함 |
| 사용자가 말하는 중 제시 금지와 Agent 도착 경계 충돌 | 현재 필수 사용자 발화 구간을 외부 입력으로 표시 | DAG에서 VIA 병렬 처리/전사/queue는 유지, wall-clock 병기 |
| 이전 W100 빈도와 비용 목표를 새 활동에 그대로 적용할 위험 | P21 입력50초/출력179초/의미62호출 예산 | 기본$0.223167+여유$0.076833=$0.30/P21, 실제 청구 우선 |
| 45의 예시40/10miss·20생산이 P21과 다름 | S2-02/04/05의 A3miss/강한cache1miss, B4생산 지역 가정 | 원천갱신4가 모델생산4를 강제하지 않음, 비용 중복 방지 |
| 5회만으로 95% 확증을 요구하는 해석 | 실측5쌍의 방향과 평균/최소/최대 공개 | 실용 차이+5쌍 같은 방향의 PoC 기준, 통계적 확증 아님 |
| 41 메모리의 frame 수가 모호 | 4×16MiB 대4×8MiB 명시 | -32MiB 산술과 할당 가정 구별 |

42 정상 답변의 지역0.015초, 44 전이의0.010초 같은 작은 차이를 전체 QA 평균/유의 차이로 확대하지 않는다. 지역 예시와 민감도 계산은 후보 측정 결과가 아니다.

## 4차: 발표 원본과 렌더

요약을 핵심 표2장/공통 활동1장/전체21개 대응표3장으로 개편했다. 부록은 여섯 QA 각1장이다. 두 JSON은 같은 원본 SHA-256, 공통 원장/manifest hash와 전체 명세를 포함한다. 대응표 슬라이드 노트에도 21개의 완전한 선정/관측 경계를 넣었다.

별도 build/candidate/final로 export/finalize하고 최종 PPTX에서 재가져와 렌더한 **12장을 각각 육안 확인**했다. 제공한 레퍼런스의 흰 바탕/파란 강조/크림 요약표/목표 근거와 오른쪽 등급표를 유지했다. 최종 렌더에 잘림/겹침을 발견하지 않았다. 요약 native table6개, 부록18개이며 슬라이드 이미지를 삽입한 페이지는 없다. native table의 PowerPoint 자동 행 확장 추정에서 부록 6개 계산표가 주석 쪽으로 넘칠 수 있다는 경고를 확인했다. 계산표의 상하 여백을 6px에서 2px로 줄이고 별도 새 build/final로 다시 export/finalize했다. 수정 뒤 package/layout/font/native table/Artifact Tool import 검사를 통과했고 두 deck의 layout finding/warning은 모두 0이다. portable 확인 기록은 각 발표 폴더의 validation-summary.json에 있다.

## 5차: 자동 재현성과 확인 범위

공통 활동의 사례 누락/반응 누락/반복15회/호출상한 변경/의존순서 역전/Agent 분류 오류/집중 분모 변경/VIA 시간을 외부 제외에 추가한 입력은 모두 거부됐다. 검산은 실제 단가 합계, 병렬 Agent·사람 발화의 귀속, 공유 allocation identity/동시 peak와 모든 등급 경계를 검사한다. 두 발표자료의 표/문안/JSON/노트/hash/PNG/링크와 archive **103파일**의 bytes/hash도 일치했다. 기존 17개의 checkpoint 사실은 보존하고 S5-03의 낡은 §3.6.3 연결만 현행 §3.5로 고쳤다.

```sh
python3 scripts/architecture/qa_common_workload.py --check
python3 scripts/architecture/qa_poc_reference.py --check
python3 scripts/presentations/check_quality_attributes.py
python3 scripts/presentations/check_quality_metrics.py
python3 scripts/architecture/check_active_terminology.py
python3 scripts/architecture/check_active_markdown_links.py
python3 scripts/architecture/check_qa_catalog.py
```

위 검사는 통과했다. 실제 후보/유료 모델 API/Windows 메모리/Microsoft PowerPoint 앱 저장·재열기는 수행하지 않았다. 녹음·화면 fixture 파일과 후보 trace는 정의 이후 실행 작업이며 현재 측정 결과로 만들지 않는다. 우선순위 변경 추천은 미채택 상태로 보존했고 기존 번호순을 유지했다. 기존 DP/참조 Architecture/QA catalog/ADR와 병행 작업은 수정하지 않았다. 저장소 전체 CI의 결과는 이 QA 범위 검수와 구별한다.
