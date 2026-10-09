# 04-46 재설계 검수와 재개 기록

> 상태: **작성자 문서/도식 대조 완료 / 사용자 검토 준비 / A/B 미선정 / 구현과 품질 측정 없음** / 2026-10-10
> 대상: [46 본문](./04-46-input-and-context-evidence.md), [문제](./diagrams/dp46-background.svg), [MAIN](./diagrams/choice46-structure.svg), [정정 사건도](./diagrams/choice46-timeline.svg), [상태/사용 경계](./diagrams/choice46-lifecycle.svg), [확대 보기](./diagrams/choice46-review.html). 독립 agent 또는 외부 리뷰가 아니다.

## 1. 사용자 지적과 완료 기준

초기 46은 공통 Interaction Manager가 시간 대응을 끝낸 다음 45의 A/B를 적용했다. 사용자는 이것으로 시간 대응까지 포함한 구조 비교가 되지 않는다고 지적했다. [초기 세대](../../../archive/46-observation-contract-only-2026-10-09/README.md)는 논의 이력으로 보존하고 현재 46을 다시 작성했다.

이번에는 A의 Temporal Evidence Resolver가 요청별 원본 결합을 책임지고 B의 Temporal Evidence Publisher/Repository/Reader가 공통 시간 관계의 생산/게시/소비를 책임진다. Context Manager의 A 원본 owner 조합과 B 과거 생산/조회도 유지한다. 두 안의 정상 준비 조건, 수정 대상과 비용이 양쪽 owner에서 달라진다. 같은 시간 자료를 Context Manager에 재게시하는 단계는 제거했다.

41/42/44/45의 본문, MAIN과 관련 검수 기록을 읽고 아래 완료 기준을 적용했다. 이 표는 기존 문서를 평가하거나 새로운 DP 선정 기준을 확정한 것이 아니다.

| 참고 문서 | 46에 적용한 기준 | 실제 반영 위치 |
| --- | --- | --- |
| 40 | 로컬 VIA, 로컬 VAD, cloud voice/transcription/semantic, owner와 실제 전달 구별 | §3~4, §8, §10. 공통 Input + Evidence Record와 역할별 Model Access 계약. 최신 MAIN은 read/게시 차이를 확대하고 공통 호출을 하단에 구별 |
| 41 | 같은 입력과 미확정 Task, 실제 교환 자료/조건부 read, 코드/모델 판단과 잠정 상태/채택 구별 | §1~3, §5~8. MAIN의 Q1/TEv1 예시, 공통 현재 해석/채택. 사건도의 늦은 j1 거절 |
| 42 | 상태마다 다른 논리적 수명과 owner, local 확정과 외부 사실, 관련 commit/복구 경계 | §3/8/14. 시간 view와 과거 view가 전역 transaction이 아님. 원본 소실 시 rebuild 한계 |
| 44 | 준비와 실제 사용/게시 조건, 새 입력 우선 제어, 정상/정정/실패와 늦은 결과 | §7~8. input/local stop와 view 생산 분리, read 성공을 admission으로 취급하지 않음 |
| 45 | 강한 A, 생산/조회가 실제 작업을 대체하는 B, 비용과 반대 선택 조건, cache/혼합 반론 | §5~6, §9~13. 과거와 시간의 QA 효과를 별도 비교, 모델 usage와 code 자원 구별 |

## 2. 설계와 도식의 대조

| 확인한 질문 | 현재 설계와 한계 |
| --- | --- |
| 두 Interaction Manager가 실제 다른가? | A query/Result/임시 dependency와 B producer/조건부 게시/coverage/Reader의 다른 정상 계약. raw owner와 process 수를 옮긴 비교가 아님 |
| 두 Context Manager도 다른가? | A Composer가 temporal 결과와 과거 owner read를 구성. B Memory Publisher가 과거 자료만 생산하며 Evidence Reader가 두 owner의 게시 자료를 소비 |
| 기본 시간 mapping이 비교를 선결정하지 않는가? | 공통 sample/clock/input 연결과 교차 원본 관계 구성 구별. 원문/refs의 Input Record가 모든 relation을 미리 완성한다는 가정 없음 |
| B가 지칭이나 Task 정답을 미리 게시하는가? | 게시 값은 시간 후보와 과거 관계. 현재 referent/Task/조건/수정은 동일 Request Interpreter의 잠정 제안과 Request Controller 채택 |
| 실제 사례와 교환 자료가 보이는가? | W46/a/b/S1/P1/S2/P2, Q1(r1)/TEv1(r1), P20→E20의 실제 범위, T21→D21@v2. 그림의 문서형 예시는 원본 schema 확정이 아님 |
| 시간과 과거 생산을 중복하는가? | temporal owner IM, historical owner CM. Bundle의 refs/발췌는 별도 temporal repository 재게시가 아님 |
| input 확정이 B 게시를 기다리는가? | 아니오. 공통 input/stop와 필요한 EvidenceRead 준비 상태는 별도. 첫/미준비 read에는 production 대기를 비용으로 포함 |
| 같은 전사 정정과 새 사용자 정정을 혼동하는가? | r1→r2와 u47을 구별. 전체 E1~E12와 확대 D1~D14 번호를 구별 |
| 유효성 알림 지연이나 파생 오류는 어떻게 되는가? | owner의 use gate/epoch/vector 재검사와 EvidenceDispute. 원본 revision이 그대로여도 오류 버전 차단/재검사 |
| 원본이 사라진 뒤 복구를 과장하는가? | raw/index/locator/view의 차이 명시. 사라진 화면/경로를 게시 record나 현재 화면으로 복원하지 않음 |
| code 시간 재사용을 모델 요금 절감으로 세는가? | 아니오. 시간 join은 코드. C-CONTEXT와 C-INTERPRET의 실제 billed usage, 미사용/폐기/정정 비용을 별도로 기록 |
| A를 약하게 만들거나 B의 고유 이익을 확정하는가? | A의 index/cache/관계 cache/부분 갱신/prefetch, B의 선택적 생산/부분 갱신 포함. temporal 작업이 작고 반복이 적으면 B 이익이 작아지는 반례 공개 |
| 미지원 자료를 재생산하면 해결되는가? | NOT_COVERED/UNSUPPORTED_RELATION/UNKNOWN_TIMING/SOURCE_GAP 구별. raw 결합 우회는 B+ 혼합으로 비용/계약 공개 |
| 다른 비교의 미결을 해결했다고 하는가? | 42 §4.2에 hold/ACK/CAS/reconciliation 설계 계약 문서화, 실제 선후·복구 실행 검증은 미수행. provider span timing, OS coverage와 budget/가격 미검증 |

현재의 다섯 QA 묶음은 03-02 공동 검토를 따른다. 45의 과거 처리 충돌과 46의 시간 처리 추가 충돌을 §11의 같은 표에서 구별하고 첫/반복/정정 조건을 따로 둔다. V-01~13은 전체 coverage 관점으로 남긴다. 지표/분모/순위/목표, 주요 DP 자격이나 승자를 새로 확정하지 않았다.

## 3. 첫 재설계에서 실제 발견하고 수정한 사항

| 검수에서 확인한 문제 | 수정 |
| --- | --- |
| 초기 세대에서 IM 시간 대응이 동일하고 차이는 45 뒤에만 존재 | IM query 구성과 producer/read model 소비로 정상 의존부터 다시 설계 |
| temporal과 historical을 모두 CM에서 게시하면 owner와 pipeline이 겹침 | IM에 temporal publication/Reader 유지, CM에는 historical 생산과 두 read 모델 소비 |
| MAIN에서 현재 Request Controller로 가는 공통 input 경로가 빠짐 | Input + Evidence Record와 r1/r2 refs/gap 경로 추가 |
| 공통 의미 모델만 보이면 CM의 조건부 과거 의미 가공 비용을 숨길 수 있음 | 한 Model Access 안에 voice/semantic client, 두 cloud 경로와 조건부 C-CONTEXT 표시. ID/시간 결합은 코드라고 명시 |
| 확대 사건도 E 번호가 전체 사건 표의 번호와 다른 사건을 가리킴 | 확대 메시지는 D1~D14로 바꾸고 전체 E3~E6 범위 확대임을 본문에 명시 |
| B 상태 그림의 repository 결과가 CM으로 바로 내려가 Reader를 생략 | Temporal Evidence Reader를 추가하고 유효 read 경로로 연결 |
| MAIN의 좁은 Temporal Evidence Reader 이름이 실제 glyph 폭 초과 | 두 줄 이름을 조정하고 최종 네 그림의 폭/겹침 검사를 다시 수행 |
| SVG 사건 참여자와 draw.io lifeline의 제목 배치가 달랐음 | 같은 폰트/상단 제목/명시 포트로 맞춤 |

첫 재설계 당시 네 PNG를 직접 열어 MAIN의 양쪽 내부 구조, 실제 사례 자료, 모델/공통 input 경로, 정정 메시지와 상태/owner 연결을 대조했다. 아래 사용자 지적에 따라 MAIN 표현을 다시 갱신했다. 작성자 검수 범위에서 남아 있는 문서/그림 불일치는 확인하지 못했다. 독립 전문가 검증, 모델 의미 성능이나 시간 대응 구현의 보증을 뜻하지 않는다.

### 3.1 사용자 후속 지적: 이름 차이로 보이는 MAIN을 실행 자료 흐름으로 수정

사용자는 실제 동작이 다른데 그림에서는 네모박스 이름만 다르다고 지적했다. 앞선 [MAIN 표현](../../../archive/46-name-and-connection-view-2026-10-10/README.md)을 보존하고 배치/자료/화살표를 다시 구성했다. 실제 A/B 설계, §3~14의 기능/QA 계약, 다른 DP와 기존 사건/상태/문제 그림은 바꾸지 않는다.

| 새 도식에서 확인한 항목 | 반영 |
| --- | --- |
| 서로 다른 그래프 | A의 Request Interpreter→Query→Composer→TemporalQuery→Resolver→요청 결과 반환 loop. B의 시간/과거 producer 두 경로와 게시 자료의 소비 경로 |
| 실제 raw/query/view/Bundle 자료 | W46/r1/a,b, raw S1/P1/S2/P2, Q1 Result, TEv1 relation/coverage/vector, HMv7 실제 전달/확인 결과, 반환 Bundle를 문서형 예시로 표시 |
| 발생 입력과 정상 준비 조건 | A query가 필요한 raw read/결합을 실행. B source revision 또는 미게시 생산 요구가 publication을 실행하며 Reader는 유효 자료를 소비 |
| 유효 cache와 미게시 경로 | 당시에는 A cache와 B 미게시 요구를 점선으로 구별했다. 통합 검수에서는 공통 Legend와 충돌해 아래 §6대로 조건 문구의 실선 경로로 수정 |
| 생산/소비와 코드/모델 경계 | 시간 join은 코드. 과거 가공은 조건부 C-CONTEXT. 최종 C-INTERPRET/채택은 공통. B가 정답을 미리 만드는 그림이 아님 |
| scheduling 오해 방지 | 숫자/위아래 배치는 기능 식별과 책임 구분. A의 parallel/prefetch, B의 on-demand/별도 생산을 허용. first-ready 가정을 무료로 주지 않음 |
| 선이 지워져 자료 경로가 끊기는 문제 | MAIN 주석만 배경 없는 text로 만들고 선과 글자 위치 조정. 다른 scene의 출력은 그대로 보존 |

새 MAIN의 SVG/draw.io/PNG를 동기화하고 직접 렌더를 열어 위 경로를 추적했다. renderer의 glyph 겹침 한 건은 과거 read 주석과 cache 설명 간격을 벌려 해결했다. generator의 owner/port/다른 node 통과 검사와 PNG/HTML 검증을 다시 실행한다. 작성자 대조이며 독립 Architecture 검증이나 실제 QA 우위 측정이 아니다.

### 3.2 사용자 재지적: 동일 배치에서 읽기/쓰기를 비교

사용자는 §3.1의 세로/가로 배치가 설계 차이를 표현하지 못하고 눈속임처럼 보인다고 지적했다. 해당 [배치 세대](../../../archive/46-axis-swapped-main-2026-10-10/README.md)는 반려 이력으로 보존했다. 공통 요소를 다른 위치에 놓아 차이를 강조하는 방식은 현재 검수 기준으로 사용하지 않는다.

- 공통 14개 Component/원본/요청/반환 요소의 상대 x/y/width/height를 한 정의로 생성하고 양안 일치를 assert한다. owner의 경계도 동일하다.
- 같은 원본 위치에서 A Resolver의 bounded read와 B Publisher의 source-change 입력을 추적한다. Source View Cache의 유효 재사용도 허용한다.
- 같은 자료 위치에서 A가 query 결과/Set을 구성하고 기록하는 흐름과 B가 관계/coverage를 게시한 뒤 Reader가 소비하는 흐름을 대조한다. A에 공통 게시를 필수화하거나 B의 정상 Reader가 raw 관계를 재구성하게 하지 않는다.
- Query/Bundle/현재 채택의 위치와 포트는 동일하다. 임시 상태와 공통 게시 상태의 이름뿐 아니라 구성/쓰기/읽기 방향과 생성 주체를 구별한다.
- 원본 변화와 query, 미게시 생산 요구는 서로 다른 입력이다. 동시 snapshot/고정 scheduling이나 첫 상태 무료 제공의 주장이 아니다. 모델 역할과 기존 QA/선택 조건은 유지한다.

Timeline & Buffer 제목/공통 설명과 A의 query/결과 구성 주석 간 글자 겹침을 보완했다. 최종 MAIN을 직접 열어 공통 좌표와 위 경로를 대조하고 생성/렌더/링크 검사를 다시 수행했다. 이 검수는 표현과 계약 일치에 관한 작성자 확인이며 실제 QA 효과를 증명하지 않는다.

## 4. 검증과 재현

검증 대상은 문서와 렌더다. VIA 구현, 모델 실행, timing 정확도나 QA 우위를 측정하지 않았다.

```sh
.venv/bin/python scripts/architecture/generate_evidence_context_diagrams.py --check
.venv/bin/python scripts/architecture/check_active_markdown_links.py
.venv/bin/python scripts/architecture/check_active_terminology.py
.venv/bin/python scripts/architecture/check_qa_catalog.py
```

[전용 생성기](../../../../scripts/architecture/generate_evidence_context_diagrams.py)와 [serializer](../../../../scripts/architecture/evidence_context_diagram_scene.py)는 다른 DP/발표 generator에 의존하지 않는다. 같은 scene에서 네 SVG와 editable draw.io를 생성하며 owner containment, node/edge ID, 명시 끝점, 직각 선과 관련 없는 node 내부 통과를 검사한다. 사건도의 lifeline 교차는 의도된 메시지 경로로 구별한다. 확대 HTML도 같은 생성기의 출력이다.

[전용 renderer](../../../../scripts/architecture/render_evidence_context_diagrams.cjs)는 설치된 Playwright와 Chrome을 사용한다. 네 PNG의 실제 glyph 폭, canvas 경계와 글자 겹침을 검사하고 각 `*-render.json`에 SVG/PNG SHA-256을 기록한다. Chrome 경로와 Playwright module은 환경변수로 지정할 수 있으며 설치/외부 전송은 필요하지 않다.

| 실행한 검증 | 결과와 범위 |
| --- | --- |
| 전용 생성기 `--check` | 네 SVG/draw.io와 확대 HTML, geometry/owner/port/경로 검사 PASS |
| PNG 렌더와 직접 대조 | 네 장 모두 2560×1440, glyph 폭/canvas/글자 겹침 0건, 현재 SVG/PNG hash 일치 |
| 활성 Markdown 상대 링크 | 최신 158개 활성 파일 PASS(첫 재설계 당시 157개). 46의 Markdown anchor 2개도 별도 확인 |
| 확대 HTML | 네 figure 전환/로컬 SVG 로딩/소스 링크/원본 크기 토글 PASS, 외부 HTTP 요청 없음 |
| 활성 용어 / QA catalog | PASS. 기존 23 QA ID와 4 confirmed ASR 및 migration/retirement 상태 유지 |
| 45 보존 | 첫 재설계 완료 당시 16개 시작 SHA-256 일치. 최신 표현 갱신 중에는 병행 세션의 45 변경을 별도 관찰했으며 본 작업의 write 대상이 아님 |
| write scope / whitespace / 가운데점 | 전용 46 파일과 shared 안내의 46 블록, 이전 세대 archive만 이번 갱신. 최종 whitespace와 신규 산출물 표기 확인 |

최신 MAIN 표현 갱신에서는 55개 시작 snapshot 중 병행 세션의 42/44/45 문서/그림 변경을 관찰했다. 해당 변경은 이 작업에서 쓰거나 되돌리지 않았다. 46 문제/정정 사건/상태의 SVG/draw.io 여섯 원본은 시작 hash와 동일하다. renderer는 네 그림을 재렌더했으며 정정 사건 PNG와 해당 render hash는 재생성 차이가 있다. 모든 현재 SVG/PNG manifest가 일치하며 보충 설계/도식 원본의 변경은 아니다. 확대 HTML은 네 figure 선택과 SVG/draw.io 링크, 원본 크기/폭 맞춤을 제공한다. 통합 발표 PPTX를 갱신한 사실이 아니다.

## 5. 보존과 재개

45의 아래 16개 파일은 세션 시작 `/private/tmp/via46-preserved-45.json`과 대조했다.

- `04-45-memory-and-context.md`, `04-45-memory-and-context-review.md`
- `dp45-background.svg/.drawio`, `choice45-structure.svg/.drawio/.png`, `choice45-lifecycle.svg/.drawio/.png`
- `generate_memory_context_diagrams.py`, `memory_context_scene.py`
- 개별 발표 `dp45-comparison.svg/.drawio/.png`, 발표 배경 `dp45-background.png`

별도 45개 파일 snapshot에서는 병행 세션의 40/41 문서/그림 변경이 관찰됐다. 이를 본 작업의 수정이나 보존 실패로 처리해 되돌리지 않는다. 당시 42/44/45와 나머지 snapshot 파일은 동일했다. 03-02, 04-50/work/, 공통 계약/다른 DP와 통합 발표 자료는 이 작업에서 쓰지 않았다. `00-workplan.md`와 README는 현재 내용을 읽고 46 블록만 교체했다. 재설계 완료 당시에는 commit/push를 수행하지 않았다.

2026-10-10 사용자는 별도 세션 리뷰를 위해 46 관련 파일의 commit/push를 지시했다. 게시 단위는 46 문서/네 그림/전용 생성 및 렌더 스크립트/합성 시간 계약 예시/이전 세대 보존본과 공용 안내의 46 블록이다. 전용 PPTX 생성 소스도 포함하지만 생성된 deck이나 검증된 발표 산출물은 이번 단위에 없다. 병행 QA 문서는 별도 게시 대상이라 본문에는 문서 이름과 다섯 묶음을 기록한다. 다른 DP/QA/통합 발표/04-50/work/ 변경은 포함하지 않는다.

게시 대상만 합친 별도 사본에서 46 생성 일치, 155개 활성 Markdown 링크, 용어/QA catalog와 기존 44/45 계약 예시 검사를 통과했다. 네 SVG/PNG render manifest의 hash도 일치한다. 전용 PPTX 소스는 구문 검사만 수행했다. 병행 42의 미게시 예시는 기존 공통 계약 검사에서 SKIP이며 이번 게시 범위에 포함하지 않았다. 이 결과는 위 작업 공간 전체 검사와 구별한다.

재개 순서는 46 §2 MAIN → §3~6 책임/취득/두 실행 → §7~8 같은 사건/게시/사용 → §10~12 비용/QA/선택이다. 같은 window의 정상 재참조가 충분히 중요한지, 강한 A cache 이후에도 B의 반복 작업 대체와 대등한 QA 충돌이 남는지, 혼합 도입으로 충분한지는 사용자 검토 대상이다. 문서 완성을 주요 DP 또는 최종 A/B 선정으로 처리하지 않는다.
