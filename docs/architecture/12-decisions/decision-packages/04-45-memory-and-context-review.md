# 04-45 독립 리뷰와 수정 기록

> 대상: [45 본문](./04-45-memory-and-context.md), 전용 생성기, 메인/수명 SVG와 draw.io 및 PNG / 2026-10-06
> 독립 리뷰는 작성 작업을 맡지 않은 별도 에이전트의 문서/설계 검토다. 실제 사용자 심사, 실행 시험, 품질 우위나 최종 DP 선정이 아니다.

## 2026-10-13 과거 이력 정의와 설명 설계 — 작성자 검수

사용자는 과거 이력의 정의를 DP-45에 넣고, 모호한 설명과 설계를 심사위원에게 설명할 수 있도록 구체화하라고 요청했다. 이번 작업은 작성자 한 명이 문서와 데이터/그림을 대조한 검수이며 아래 과거 독립 리뷰를 새 판본에 소급하지 않는다.

### 반영한 내용

- 본문 §0.1~0.3: 사용자 입력, 채택 요청과 정정, 업무 연결과 수신 사건, 응답 준비/실제 전달, 사용 자료/결과 참조의 다섯 종류를 정의했다. owner, 기록 시점과 의미 한계, 당시/현재 상태 및 원본/파생/Context/User Memory를 구별했다.
- §0.4~0.6와 [설명 JSON](./contracts/dp45-history-walkthrough.json): 입력에 정답 ID를 넣지 않는 후보 발견 → 좁힌 조회 → 같은 전달 원문/결과 참조를 반환하는 장면을 작성했다. B의 네 관계 종류와 생산 정책, 검색 metadata, 조건부 의미 가공, 게시/조회와 미준비 처리를 구체화했다. 과거 이력을 새 중앙 DB나 현재 Task 권위로 바꾸지 않았다.
- §0.7~0.8: 첫/반복/변경/파생 오류/철회 사건, 설명 원고와 미검증 품질 가설을 분리했다. 새 충분 범위 예시는 양안의 현재 의미 입력이 같으므로 B의 호출/token 감소를 주장하지 않는다.
- [정의 그림](./diagrams/dp45-history-definition.svg)과 [같은 기록의 흐름 그림](./diagrams/dp45-history-walkthrough.svg): 각 SVG와 draw.io, 실제 렌더 PNG를 추가했다. 생성기는 [generate_dp45_history_diagrams.py](../../../../scripts/architecture/generate_dp45_history_diagrams.py)다. 기존 MAIN/수명과 Evidence v1 PARTIAL JSON, PPTX는 보존했다. workplan과 그림 안내에 읽기 경로를 연결했다.

### 확인한 범위와 한계

1. 설명 JSON을 정적으로 대조했다. 최초 입력/조회에 확정 target ID가 없고, A/B의 발췌/결과 version/출처 vector가 같으며, 문구가 실제 원문 `[0,52)` 및 각 채널 전달 구간 안에 있음을 확인했다. B의 게시 revision/producer version, 네 kind, 미래 R23/T23 미게시와 별도 정정 variant를 확인했다. 기존 `[27,52)` PARTIAL 예시는 바꾸지 않았다. timestamp와 record는 작성 예시이며 실행 기록이 아니다.
2. 새 생성기의 `--check`로 두 SVG/draw.io 쌍의 source 일치, XML ID/참조, owner 포함, canvas와 직교 경로를 확인했다. Module과 상태는 Context Manager 안에 있고 원본 종류 그림의 접힌 종이는 자료 구분이다.
3. Chrome으로 2560×1440 PNG 두 장을 렌더하고 직접 열었다. 글자 겹침, canvas 밖 잘림과 자료 도형 내부 글자 넘침은 0건이다. 처음 렌더에서 B의 원본 읽기 선이 Context Manager 제목을 지나고 원통 첫 줄이 윗곡선과 겹쳐 보이는 것을 발견해 경로와 여백을 수정했고 재렌더를 확인했다. draw.io 앱, Microsoft PowerPoint와 발표 거리 가독성은 확인하지 않았다.
4. 이번 변경 문서의 local 링크, active terminology, 정식 target Component 이름, QA catalog와 `git diff --check`를 통과했다. 공유 작업공간 전체 링크 검사에는 병행 작성 중인 DP-47의 `dp47-design-review.md`와 `choice47-structure.png` 미생성 참조가 남아 있었다. 해당 파일을 수정하거나 검사에서 숨기지 않았으며 이번 DP-45의 scoped 링크 통과와 구별한다.

재생성/검사: `.venv/bin/python scripts/architecture/generate_dp45_history_diagrams.py`와 동일 명령의 `--check`. PNG는 생성한 로컬 SVG를 Chrome headless의 2560×1440 viewport에서 렌더한 preview다. 검수는 정의, 데이터와 그림의 정적 일치이며 생산자/retrieval 구현, 의미 정확도, 성능/비용/자원 측정, 독립 심사나 최종 A/B 선택을 확인하지 않는다. 사용자 검토를 위한 설계 제안이다.

## 2026-10-10 실행 계약 재구체화 — 본문과 데이터 예시

사용자는 44 → 45 → 42 → 46의 본문/그림/발표를 단계별로 완성하고 작업 단위마다 commit/push하도록 요청했다. 이번 본문 작업은 04-40의 로컬 VIA, 로컬 VAD, cloud voice/semantic 조건과 41의 읽기 도구 실행기 직접 read를 45에 적용한다. 기본 A/B, 공식 Reference, 같은 사례와 강한 A의 cache/부분 갱신, B의 선택적 생산/미게시/미지원/명시 B+ 계약을 유지했다. 이 절은 이번 본문 작성자의 검수이며 아래 과거 독립 리뷰를 새 판본 전체에 소급하지 않는다.

### 작성 범위와 수정 이유

- 45 본문과 [dp45-evidence-examples.json](./contracts/dp45-evidence-examples.json), 이 검수 기록만 이 작업자가 작성했다. scene/생성기/발표/PPTX와 공통 문서/QA는 별도 통합 writer의 범위다.
- 기존 Shared Omni/온디바이스 KV 전제가 새 cloud 계약과 충돌하여 현재 적용 조건을 수정했다. 참조 Architecture와 과거 QA/ADR를 변경하거나 실제 모델 사용량을 측정하지 않았다.
- RC의 입력/scope/채택 권위와 RI 읽기 실행기의 CM 직접 caller를 구별했다. 동일 caller/job으로 read 반환, A의 추가 owner read 반복과 B의 게시 대기/재조회 및 두 trigger를 상세화했다.
- 고정 필드/타입/유한 kind를 가진 Evidence Contract v1과 JSON 원본 snapshot, Query1, Bundle1/Receipt1, 생산/부분 의미 job, MemoryRecord/Coverage/PublishResult를 추가했다. 실제 Schema/런타임 API가 구현된 것으로 표현하지 않는다.
- 과거 설명의 실제 전달 범위에서 조건/예외 한 구간을 추출하는 `MODEL_DERIVED` 예시와 TM의 `EXPLICIT` result 연결을 구별했다. 좁은 예시를 전체 평가 기준의 완전 생산으로 오해하지 않도록 양안 Bundle은 `PARTIAL`, B Coverage에는 미생산 구간을 기록했다.
- 03-02 공통 QA 원본에 다섯 품질 묶음을 연결하되 DP 전용 대표 지표/분모/정량 우열을 만들지 않았다. 46의 시간 관계 owner는 IM, 과거 owner는 CM이며 중복 게시/청구/QA 이익 합산을 금지했다.
- 독립 생산 수명과 현재 Request/attempt의 read 수명, 삭제/철회 fence와 local purge, 오래된 producer/caller 완료와 cloud 청구를 구별했다. 42 B의 cross-owner fence는 실제 배치 프로토콜을 참조해야 하며 CM transaction만으로 완료됐다고 하지 않는다.

### 본문과 예시의 정적 검수

2026-10-10 작성자가 임시 stdlib 검사로 JSON의 고정 key, 필드 대응, source span `[start,end)`와 원문 일치, 실제 전달 구간 안 포함, query/receipt correlation, 관계와 Coverage/PublishResult의 동일 게시 revision, 동일 A/B 자료와 PARTIAL 표시, 미생산/미지원 상태 구별을 확인했다. 명세 정적 검사이며 retrieval/producer 엔진, semantic oracle 또는 cloud 모델을 실행한 검증은 아니다.

같은 시점에 활성 Markdown 링크 검사 158개, 정식 Component/용어 검사, `git diff --check`를 통과했다. 다른 세션이 공통/46 자료를 계속 편집하므로 최종 통합 시 다시 검사해야 한다.

**그림/PPTX 최신 판본은 통합 writer가 이후 생성/렌더하고 이 기록에 실제 결과를 추가할 예정이다.** 이 본문 완료 시점에 새 MAIN/별도 순서도/PNG/native PPTX를 검증했다고 주장하지 않는다. 이번 작업자는 commit/push하지 않았으며 게시 SHA/CI는 최종 통합 검수와 Git 이력에서 확인한다. A/B 최종 선택, 구현과 모델 품질/성능/비용 측정은 하지 않았다.

## 사용자 심사 의견 반영 — 필요성과 대표 사례 교체 (2026-10-09)

사용자가 전달한 Senior SW Architect 지적을 반영했다. 기존 “지난번 고친 표현 방식”은 Agent가 이전 수정/결과 맥락을 재사용할 수 있어 VIA의 복잡한 관계 복원을 입증하기에 약하다. 같은 Agent 선택과 실제 실행 맥락 연결을 구별하고, Agent 장애/교체/기억 소실을 주된 반박으로 삼지 않았다. 기존 사례는 본문 §1.2의 단순 Task/실행 참조로 충분할 수 있는 반례로 남겼다. 아래 독립 리뷰는 당시 원본과 그림의 이력이며 이번 판본을 새로 독립 심사한 기록이 아니다.

새 대표 요청은 “지난번 네가 설명한 평가 기준에 맞춰, 앞서 조사한 제품 비교 결과와 받아둔 견적을 이번 제안서에 반영해줘.”다. VIA 직접 설명 E20의 실제 전달 P20, 서로 다른 업무 T21/D21@v2 제품 비교와 T22/D22@v1 견적을 현재 R23에 연결한다. 사례 ID와 대화 C20~23, 새 T23 및 후속 R24는 본문 §3.1에 정의했다. 결과 본문/참조, 생성/실제 전달, 원본/파생 관계와 현재 판단을 구별했다.

VIA의 역할은 허용된 발췌와 참조의 연결, 요청 이해 및 위임 Context 구성이다. 평가 기준 적용과 업무 분석, 제안서 작성은 Agent 책임이다. 단순 참조, A의 작은 index/cache 확장과 기존 실행 맥락 재사용으로 충분할 수 있으며 공통 파생 관계 계약의 필요성은 아직 검토 중이다. A/B 메커니즘과 갱신/삭제/철회/오류/coverage/원문 확인, 공통 자원 조건 및 미선정/미측정 상태는 유지한다. 새 사례가 B의 정확성/속도나 주요 DP 자격을 확정하지 않는다.

본문/배경/MAIN/수명 설명과 발표 비교, 지도 45 행을 생성 원본에서 함께 갱신했다. 수명 그림은 결과 수정/철회 ID만 바꿨으며 기존 사용 차단과 복구 순서를 유지했다. 배경의 실선 시간 흐름과 점선 과거 참조를 구별하고 Component/저장소를 답으로 제시하지 않았다. 표현 스타일을 전면 재설계하지 않았다.

검증과 게시 결과는 아래 최신 검수에 기록한다. 이번 변경만 commit/push하도록 승인받았으며 다른 세션의 04-50 및 `work/`는 제외한다.

### 이번 판본의 작성자 검수

- 45와 배경/비교 생성기의 `--check`, 활성 Markdown 151개 링크, 용어와 정식 Component 이름, QA catalog 및 `git diff --check`를 통과했다. SVG/draw.io는 현재 scene과 일치한다.
- 배경, 전체 지도, MAIN, 수명, 발표 비교를 실제 브라우저 PNG로 렌더하고 직접 확인했다. 문자 겹침, 선언 폭 초과와 canvas 밖 잘림은 0건이다. 발표 비교의 결과 참조 라벨 간격을 보완했다. MAIN의 기존 배선이 Context Manager 제목을 지나던 문제를 발견해 같은 송수신 관계를 유지하며 경로를 조정했고 실제 이름 관통 검사 0건을 확인했다.
- 세 PPTX의 10/5/5장과 native 도형/텍스트/경로, 평면 이미지 0개를 확인했다. 통합 9/10번, 배경 5번, 비교 5번과 해당 notes/관계만 갱신했다. package/layout 및 재가져오기 검증을 통과했고 변경 슬라이드 렌더를 확인했다. PowerPoint 앱에서 확인한 것은 아니다.
- 작업 시작본과 PPTX 파일 목록/다른 package 부분을 바이트 대조했다. 공유 draw.io 모음의 다른 DP 페이지, 41~44 원본/PNG와 참조 Architecture/QA/ADR, 초기 04-50 및 `work/` 해시도 보존했다. 기존 예시 수치와 점수는 그대로이며 새 사례의 검증값이 아니다.

검수는 표현/생성물 일치 확인이다. 모든 과거 지칭의 완전 지원, 실제 기기 성능이나 공통 파생 관계 계약의 필수성은 검증하지 않았다. 최종 DP/A/B 판단은 원래 리뷰 세션에서 계속한다. 이번 사용자 지적 반영 뒤 새로운 독립 리뷰를 수행했다고 기록하지 않는다. 게시 SHA와 원격 CI는 Git 이력 및 최종 응답에서 확인한다.

## 1. 설계 전 반론 검토

`structure_review`가 참조 기억 설계와 기존 비교를 읽고 편집 없이 검토했다. 기존 summary/index/cache와 배경 갱신을 B만의 기능으로 삼으면 안 된다는 지적을 반영했다.

- B의 정상 경로는 공통 관계 읽기이며 A의 원본별 요청 조합을 실제 대체하도록 설계했다. 별도 process나 DB 제품으로 차이를 만들지 않았다.
- source별 cursor/관계 종류/coverage와 원본 유효성, 현재 의미 확정의 경계를 유지했다.
- A의 작은 정정 링크 index로 같은 이익을 얻는 반론, 양방향 전환과 공통 query/cache를 본문 §7~8에 포함했다.
- 41/43/42/34를 동일 설정으로 고정하고 조합 시 표현/버전/수명 충돌을 본문 §2에 남겼다.

## 2. 본문 1차 독립 리뷰와 보완

리뷰어: `structure_review`. 범위: 본문 전체, 기존 책임/정책 대조. 그림은 작성 중이어서 이번 회차 대상에서 제외했다. 결과: P1 없음, P2 세 건.

| 지적 | 보완 |
| --- | --- |
| epoch 검사와 실제 enqueue/게시 사이 철회 race가 닫히지 않음 | §4.3: 사용 게이트에서 epoch 확인과 사용 승인/queue 등록의 직렬화, 철회 시 이전 job/output fence, B 게시의 expected epoch/revision 조건부 commit 명시. 외부 실행의 회수 한계와 전역 snapshot 아님을 구별 |
| 원본 revision 변화 없는 summary/관계 오류를 누가 고치는지 누락 | §4.2: `EvidenceDispute`를 Request Interpreter→Request Controller→Context Manager로 전달. ID와 게시 revision 조건부 무효화, 원문 재검증/재생산. A cache에도 같은 오류 보고 허용 |
| 미생산 범위와 표현 불가능한 관계가 혼합됨 | §3.3/4.2/5/6: `NOT_COVERED`와 `UNSUPPORTED_RELATION` 구분. backfill은 지원 종류의 미생산만 해결하며 새 관계는 설계 변경 또는 B+ 필요 |

비차단 의견도 유지했다. 같은 스타일의 반복 조회는 A warm cache도 해결하므로, 그것만으로 B의 주요 정상 사용상 이익을 입증하지 않는다. 여러 source/소비자의 공통 관계 계약이 요청 조합을 실제 대체하는 범위에서만 독립 선택 이유가 남는다.

## 3. 재검토와 그림 검토

`structure_review`의 본문 2차 재검토에서 위 P2 세 건 모두 해결, 추가 P1/P2 없음으로 보고됐다. 확인 범위는 소유권, 계약, 동일 조건, 사례, 품질 인과, 스타일 구별과 작은 확장 반론의 정적 문서 검토다. 44 초안의 읽기 전용 경계 확인과 가이드의 네 완료 질문 정렬도 확인했다. 구현 동시성이나 품질 우위 검증은 아니다.

별도 `independent_review`는 수정 본문에서 추가 P1/P2를 찾지 못했다. 원문 확인과 원본 검색/교차 결합 우회(B+)를 그림에서도 구별해야 한다고 지적했다. 실제 PNG 선행 검토에서는 다음 문제를 발견했고 그림 작성자가 보완했다.

| 그림 지적 | 요구한 보완 |
| --- | --- |
| 메인에서 공통 User Memory가 보이지 않아 발표 대본의 지시를 따라갈 수 없음 | 명시 저장/삭제되는 공통 기억과 그 읽기를 메인에 표시하고 과거 파생 관계와 구별 |
| A 수명 그림이 cache 무효화 뒤 사용 차단으로 읽힘 | 양안 모두 첫 사건에서 유효성/권한 원장과 사용 승인 fence, 이후 파생물/KV 정리로 수정 |
| B의 재게시가 삭제/철회 뒤 기억 부활로 읽힐 수 있음 | 허용된 수정 범위만 재생산/재게시하며 삭제/철회 대상은 읽기 재개하지 않는 조건 표시 |

그림 2차 리뷰에서 위 세 항목의 해결을 확인했고, 메인만 보는 기준으로 P2 두 건을 추가 발견했다. B의 원문 재확인이 Task Manager에만 연결되어 R7 정정/실제 P4 제시의 원본 소유자까지 추적되지 않는 점, 원본 변경 없는 파생 오류 보고 경로가 보충 그림에만 있는 점이다. 원본 세 소유자로 재확인 경로를 확장하고, 양안의 오류 신고 및 B 재검증 경로를 메인에도 표시했다. User Memory를 Context Manager 소유 경계 안에 표시하라는 표현 권고도 반영했다.

**최종 재검토:** `independent_review`가 수정 메인 PNG를 원래 2560×1440 해상도로 다시 열어 추가 P2 두 건과 User Memory 소유 표기의 해결을 확인했다. 본문과 두 PNG 검토에서 현재 미해결 P1/P2 없음으로 보고했다. 작성자도 최종 메인과 수명 렌더를 직접 확인했다. 색이나 이름뿐 아니라 A의 요청별 원본 fan-out/임시 근거와 B의 지속 producer→repository→reader 경로가 다르다는 판단이며, 강한 A보다 기능 품질이 높다는 실증은 아니다. 실제 발표 화면/거리에서의 가독성과 심사위원의 수용성은 확인 범위 밖이다.

## 4. 재현과 검증 범위

생성 원본은 [generate_memory_context_diagrams.py](../../../../scripts/architecture/generate_memory_context_diagrams.py)이며 기존 41/42/43 생성기와 공통 그림 helper는 수정하지 않는다. 메인과 수명 그림의 SVG/draw.io는 같은 scene을 사용한다. PNG는 실제 렌더를 확인하는 발표용 preview다.

재현 명령:

```sh
.venv/bin/python scripts/architecture/generate_memory_context_diagrams.py
.venv/bin/python scripts/architecture/generate_memory_context_diagrams.py --check
.venv/bin/python scripts/architecture/check_active_markdown_links.py
.venv/bin/python scripts/architecture/check_active_terminology.py
.venv/bin/python scripts/architecture/check_qa_catalog.py
git diff --check
```

PNG는 설치된 Google Chrome의 headless renderer로 SVG를 직접 열어 만들었다. `--headless --no-sandbox --disable-gpu --hide-scrollbars --screenshot=<PNG 절대 경로> --window-size=2560,1440 file://<SVG 절대 경로>`를 사용하며, 보충 수명 그림은 `2560,1920`이다. 출력 PNG를 작성자와 독립 리뷰어가 직접 열어 확인한다. 웹 페이지나 외부 콘텐츠를 실행하는 렌더가 아니다.

검사 결과:

- 45 생성기 `--check`: 두 SVG/draw.io 쌍의 source 일치, XML/ID/참조, 소유 포함, canvas 범위, 직교 경로와 무관 노드 관통 검사 통과.
- 기존 target/decision-package/stage4/mechanism/functional/six-comparison 및 41/42/43 생성기의 `--check`도 모두 통과. 기존 생성물은 수정하지 않았다.
- active terminology, QA catalog, staged diff 공백 검사 통과.
- 공유 작업공간의 전체 링크 검사에서는 병행 작성 중인 44 초안의 아직 없는 검수/그림 링크가 발견됐다. 그 파일을 수정하거나 제외 처리하는 대신, **45의 12개 staging 파일만 포함한 Git tree를 임시 디렉터리에 export해** 동일 링크 검사를 실행했고 active Markdown 149개 모두 통과했다. 해당 게시 snapshot에서 terminology/QA/45 생성기도 다시 통과했다.
- 메인 2560×1440, 보충 2560×1920 PNG를 실제 열어 텍스트/경로/소유/반환을 확인했다. SVG/draw.io와 PNG를 함께 게시한다.

어떤 검사도 실제 VIA 구현, 모델 실행, 기기 성능, 기능 정확도 수치 또는 심사위원의 발표 수용성을 검증하지 않는다. CI의 게시 결과는 Git 이력과 최종 응답에서 확인한다.

## 5. 보존과 게시

이번 게시 범위는 45 본문/검수/전용 생성기와 그림, 읽기 안내 및 45 생성물 일치 CI 검사다. 다른 세션의 변경은 staging하지 않는다. 31/32/34/35/36/41/42/43/50 본문/도식, 참조 Architecture와 QA/ADR를 보존한다. 게시 commit과 CI 결과는 Git 이력과 최종 응답으로 연결한다.

## 6. 사용자 후속: 좌우 독립 블록 (2026-10-06)

사용자가 A는 왼쪽 블록만, B는 오른쪽 블록만 사용하도록 요청했다. 메인의 공동 하단 처리부를 제거하고 Request Controller, Request Interpreter, User Memory를 소유하는 Context Manager, Model Access/Omni를 양쪽 블록 안에 각각 표시했다. 좌우는 대안이지 동시 배치가 아니며 안마다 Omni 한 벌이라는 조건은 유지한다.

- 원본 조회/반환, 오류 신고/재검증, B의 생산/게시와 원문 재확인, 사용 차단 경로는 유지했다. 하단 현재 의미 판단과 최종 채택도 각 안에서 끝난다.
- 생성기에 모든 노드와 처리 화살표가 왼쪽 또는 오른쪽 하나에만 속하는 assertion을 추가했다. SVG/draw.io 동기화와 기존 소유/경로 검사를 함께 실행한다.
- 2560×1440 PNG를 다시 렌더하고 작성자가 직접 열어 좌우 경계, 글자와 선의 겹침, 모델 호출/반환을 확인했다. 보충 수명 그림은 이미 좌우 흐름이 분리되어 있어 변경하지 않았다.
- 이 후속 변경은 배치/표현 수정이며 앞선 독립 리뷰를 새 그림의 추가 제3자 리뷰라고 재표시하지 않는다. 변경 파일만 commit/push하고 원격 CI 결과를 최종 응답으로 연결한다.


## 6. 사용자 지침의 구조 표현 보완과 실제 검수 (2026-10-09)

30623a71의 새 대표 사례와 781a326c의 공식 Reference/§10을 유지하고, 사용자 지침을 생성 원본과 실제 자료에 반영했다. 새로운 독립 리뷰를 수행한 기록이 아니다.

- MAIN/발표 비교는 [memory_context_scene.py](../../../../scripts/architecture/memory_context_scene.py)를 공유한다. 각 안에서 같은 정식 여섯 Component를 한 번씩 표시하고 Context Manager 안에 Module, Source View Cache, Request Evidence Set, Episodic Memory Repository와 User Memory를 해당 안의 소속으로 배치했다. 흰색/검정 공통, 살구색/짙은 테두리 차이, 둥근 Module/원통 상태/육각 외부 모델 및 Legend를 맞췄다. 요청/재사용/지속 관리 수명은 밖 주석이다.
- A는 Task Manager를 포함한 직접 owner 읽기와 반환, 유효 cache 재사용, 요청 근거 기록 및 Context Composer의 직접 Bundle/receipt 반환이다. 데이터 상태를 실행 주체처럼 연결하던 발표 경로를 수정했다.
- B는 변경 알림과 본문 읽기, 조건부 모델 생산, 출처/권한/지원 종류 검사, 관계+coverage 게시, Reader 조회/반환을 분리했다. 생산 revision/상태 반환 후 실제 Repository 재조회, 원문 재확인, 즉시 fence와 EvidenceDispute를 연결했다. 미래 R23의 정답이나 T23 binding은 게시하지 않는다.
- 45 수치/원형 점수는 다섯 조건의 정성 표로 교체하고 첫 요청/반복/정정 직후/표현 밖 관계를 구별했다. §9 공식 Reference, 대응과 한계를 각주/발표 노트에 넣었다. A의 관계 cache와 B의 전체 생산/갱신/미사용 비용, 미선정/미측정 상태는 유지한다.

**실제 확인:** MAIN/비교 1920×1080 및 수명 2560×1920 SVG를 Chrome에서 렌더하고 PNG를 열었다. 브라우저 글자 bbox의 잘림/겹침 및 주석 mask 가림 검사는 0건이다. scene 검사로 Module/상태의 부모, 여섯 정식 이름, A 직접 Task read/Module 반환, 현재 판단/모델 연결, 비관련 도형 관통 없는 직교 경로와 좌우 독립성을 확인했다. 수명 그림의 갱신/철회/원본 변화 없는 오류 및 부모 표기도 직접 확인했다. SVG/draw.io 생성 원본 일치, 비교 manifest의 SVG/PNG 해시, active Markdown 링크/용어/QA 및 git diff --check가 통과했다. CI에 있는 모든 그림 생성기의 --check도 로컬에서 통과했다.

PPTX는 `--dp45-only --comparison-only`로 생성하고 finalizer의 package/layout/font 및 Artifact Tool 재불러오기를 통과했다. 최종 통합 10번과 비교 5번 렌더를 각각 열어 확인했다. 슬라이드 수 10/5, 변경 슬라이드 각각 331개 native 도형과 0개 이미지, 노트의 공식 링크 다섯 개를 확인했다. 시작본과 ZIP entry를 비교하여 해당 slide/slide rel/notes/notes rel 네 파일씩만 바뀌고 다른 슬라이드, 배경과 공통 package가 byte 단위로 보존됐음을 확인했다. 합본 draw.io의 앞 네 페이지와 JSON의 41~44 레코드도 같다. 보호 대상 113개 tracked 파일과 초기 04-50/`work/` 파일이 보존됐다.

**한계:** Chrome과 Artifact Tool의 실제 렌더/재불러오기 범위이며 Microsoft PowerPoint 앱이나 발표 거리에서 확인한 결과는 아니다. 모델 실행/성능 측정, 새로운 독립 심사와 최종 A/B/DP 판단은 하지 않았다. 공통 파생 관계 계약이 강한 A의 작은 확장보다 필요한지는 원래 리뷰 세션에서 계속 판단한다. 사용자 승인에 따라 이번 변경 파일만 main에 정상 commit/push한다.

## 8. 2026-10-10 실행 계약과 집중 그림 재구체화

§3.4~3.9에 정식 Component/Module 소속, 고정 Evidence Contract v1, 실제 원문/전달 source span과 코드·모델 생산의 경계, 요청별/변경별 두 trigger와 재조회, cloud 사용 비용을 추가했다. JSON의 Memory1은 MODEL_DERIVED이며 EXPLICIT인 결과 참조와 구별한다. source[27,52)의 예외 한 문구와 gap[0,27)은 PARTIAL 예시이며 전체 기준의 완전 추출이 아니다. 현재 Request/Task binding을 미리 기억에 넣지 않는다.

전체 MAIN은 전용 memory_context_scene을 사용하고, 집중 발표 source는 dp45_comparison_scene으로 분리했다. 원본에는 집중 그림을 **추가**했으며 기존 MAIN/수명도는 유지했다. 집중 그림에 Query key, 원본 발췌/파생 관계, state owner, 원본 변경 생산과 현재 query 읽기, NOT_COVERED 생산·게시·재조회 loop 및 Reader의 원문/현재 revision 확인을 표시했다. 큰 owner 경계는 흰색, 차이 작은 Module·상태는 살구색이며 41/44와 같은 Legend를 사용한다. 전체 MAIN의 RI 직접 Task read와 Model Access의 클라우드 역할도 명시했다.

전용 생성기/고정 예시/집중 생성물·render manifest/활성 링크·용어·diff 검사를 통과했다. 전체 MAIN/수명과 집중 그림은 Chrome에서 글자 canvas/폭/겹침 0건으로 렌더했고 PNG를 확인했다. native PPTX package/layout/font와 Artifact Tool 재import를 통과했으며 실제 45 렌더를 확인했다. 통합 11장/비교 5장의 기존 slide 순서를 유지하고 45의 slide XML/notes XML만 바뀌었다. 비선택 parts는 byte 단위로 보존했다.

이 기록은 문서/자료의 검수이며 producer 구현, 모델 의미 성공률, 품질/비용 측정 또는 최종 대안 선정을 뜻하지 않는다. 독립 검토의 후속 발견/보완은 아래에 별도 기록한다.

독립 agent가 최신 본문 §3.9/JSON, 전체 MAIN/집중 scene 및 PNG와 native 45 렌더를 읽기 전용 재검토했다. 원본/실제 전달/source span·PARTIAL/gap, A cache/부분 갱신, B 생산/게시/재조회와 조건부 모델 호출 및 현재 의미/채택 경계에서 남은 P1/P2를 찾지 못했다. 이는 이 자료 범위의 검토 결과이며 의미 정답·런타임·실측 검증이 아니다. 전체 조합의 40 미결 기록 갱신은 별도 통합 단위에 반영한다.

## 2026-10-11 v3 집중 비교

현재 Query와 독립된 원본 변경 시작점, 요청별 구성 상태/공통 게시 상태, Bundle/MemoryRecord 예시 및 별도 순서도를 표시했다. 표현 밖 관계, 미생산의 게시 후 Reader 재조회, B의 실제 원문/개정 확인, A의 유효 cache/부분 갱신과 같은 현재 의미 판단을 유지한다. Task1/Task2는 본문에 기존 JSON ID 대응을 기록한 표시용 별칭이다. 고정 예외 문구/부분 범위를 유지하고 새로운 성능 수치나 승자를 만들지 않았다.

Chrome 글자 폭/겹침/화면 범위 검사 및 직교 선/무관 박스 관통/owner 안 상태/실제 결과 반환 경로 검사 통과. native PPTX는 두 v3의 45 slide/notes만 바꾼다. 기존 전체 MAIN/사건도와 구판 PPTX/승인된 41은 보존한다. 검수는 그림/문서 일치이며 모델 의미 판단이나 런타임 검증은 아니다.

## 2026-10-13 방안별 구현 수준의 설계도

사용자가 DP-44 §3.7보다 자세한 A/B별 SW Architecture와 주요 흐름, 본문 삽입 및 commit/push를 요청했다. [45 본문 §3.10](./04-45-memory-and-context.md#310-구현-수준의-sw-architecture-두-안을-각각-따라가기)에 별도 3840×2160 구현 설계도 두 장과 번호별 호출/완료 표, 상태 수명과 복원, 같은 사건 여덟 가지의 경로 및 발표 설명을 추가했다. 현행 그림은 아홉 장이며 모두 본문에서 확인한다.

- A는 Context Composer의 접수/계획/원본 I/O/근거 조합, 요청별 RAM 상태와 재사용 cache를 구별했다. 유효 cache, 병렬 원본 읽기와 부분 재조회는 유지한다.
- B는 생산 접수/claim/원본 읽기/조건부 의미 추출/임시 후보/검증/조건부 게시, 영속 Journal과 Repository, 현재 read의 대기 상태를 구별했다. 미생산은 권한/예산 접수 후 생산하고 게시 revision을 재조회한다. B+ 우회를 정상 B로 표현하지 않았다.
- 두 안의 공통 Component는 같은 위치다. Request Controller, Response Manager, Task Manager의 read/return, 현재 의미와 Model Access, source와 epoch의 사용 검사를 표시했다. 저장소는 실행 주체가 아니며 B의 worker가 Journal을 claim한다.
- relations/Coverage/cursor와 job 게시 결과/outbox의 함께 확정, wait 등록 후 revision 재검사, 중복/유실 통지, 취소와 늦은 완료, 원본 변경 없는 추출 오류와 재시작 계약을 본문에 명시했다. Context Manager 내부 transaction을 다른 원본 owner 전체의 원자성이나 철회 보장으로 확대하지 않았다.

전용 scene은 `dp45_implementation_scene.py`, 생성기는 `generate_dp45_implementation_diagrams.py`, Chrome 렌더 검사는 `render_dp45_implementation_diagrams.cjs`다. SVG와 draw.io는 같은 scene에서 생성한다. draw.io는 native 도형의 label, 부모와 연결 endpoint를 갖는 편집 원본이다. PNG와 SVG의 SHA-256 및 글자 검수 결과는 `diagrams/dp45-implementation-render.json`에 기록한다. CI에는 생성물과 본문 삽입/렌더 해시 검사도 추가했다.

작성자 검수에서 제목과 경로의 겹침을 수정한 뒤 두 장 모두 Chrome 글자 폭/도형 경계/상호 겹침/선의 글자 관통 0건으로 렌더했다. scene의 소유 포함, 직교 경로와 무관 도형 관통, XML ID/부모/endpoint 검사도 통과했다. PNG를 직접 열어 공통 위치와 A의 원본 반환, B의 두 시작점과 게시 후 재조회, 원본 검증과 실제 전달 범위를 확인했다.

이 기록은 작성자에 의한 문서/그림 검수다. 새로운 독립 agent 리뷰, draw.io 앱 자체의 렌더 검사, VIA 제품 구현/실행, API나 모델 검증, QA 측정 또는 대안 선정을 뜻하지 않는다. 실제 게시 commit과 CI 결과는 Git 이력과 최종 응답에서 확인한다. 병행 세션의 DP-42/47, AGENTS와 workplan 변경은 이 작업에서 게시하지 않는다.

첫 게시의 CI에서는 Python 3.12와 로컬 3.14의 부동소수 합산 차이로 draw.io 주석 폭의 마지막 자릿수가 달라져 생성물 일치 검사가 실패했다. 예를 들어 `377.18000000000006`과 `377.17999999999995`의 차이였다. 폭 계산을 정수 font unit 기반으로 바꾸고 draw.io 좌표를 소수 셋째 자리로 정규화했다. Python 3.9, 3.12, 3.14에서 같은 생성물 검사를 통과했다. SVG/PNG는 byte 단위로 동일하여 기존 렌더 검수와 해시가 유지된다. 그림 내용이나 설계 계약 변경은 없다.

## 2026-10-15 구현 구조 한 페이지 비교 추가

사용자가 기존 비교 페이지는 너무 높은 수준이라 내부 구조를 보기 어렵다며 상세 A/B 구현 설계를 바탕으로 새 비교 한 장을 요청했다. 이후 기존 그림 보존과 commit/push를 명시했다. §3.10.7에 별도 한 페이지 SVG/draw.io/PNG와 편집 가능한 PPTX를 추가했다. 기존 §3.9 비교와 §3.10.2/3의 상세 그림, 통합 v13은 보존한다.

A의 요청별 읽기 계획/원본 I/O/완료 반영/근거 조합과 재사용 cache, B의 생산 접수/job claim/관계 생산/후보/검증/게시/현재 read를 내부 코드로 펼쳤다. 원통에는 현재 read RAM, 지속 job과 공통 게시 상태를 명시했다. 미생산 backfill은 Index / Coverage Resolver 코드에서 Change Admission으로 보내며 저장소가 실행하는 것으로 표현하지 않았다. 대기 등록과 게시 알림, Reader의 재조회도 구별한다. 공통 Component 좌표와 원본/권한 확인을 유지하고 Source Read Worker와 Origin Verifier의 입력 이력 읽기는 S port 참조로 표시한다.

SVG의 소유/직교 경로/무관 노드 관통 검사와 Chrome 글자 폭/잘림/겹침/선 관통 검사를 통과했다. PPTX는 같은 scene에서 native 도형으로 작성하고 실제 도형의 text body에 이름을 넣었다. Calibri/정수 pt/ko-KR 정책, package/layout/font 검증과 Artifact Tool 재불러오기를 통과했고 최종 렌더를 직접 확인했다. native 도형 211개와 이미지 0개를 확인했다. 게시 대상 Git tree의 활성 Markdown 447개 링크 검사도 통과했다. 이 작업은 작성자 검수이며 독립 리뷰나 모델/제품 실행 및 품질 측정이 아니다. Windows Open XML SDK와 게시 CI 결과는 해당 commit의 실행 기록으로 확인하며 PowerPoint 앱 자체 렌더와 구별한다.
