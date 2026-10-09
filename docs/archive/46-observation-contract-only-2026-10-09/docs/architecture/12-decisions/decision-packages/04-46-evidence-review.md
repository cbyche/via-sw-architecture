# 04-46 관측과 Context 근거 공급 확장안 검수

> 상태: **작성자 문서/그림 검수 / 사용자 검토 전 / A/B 미선정 / 구현과 측정 없음** / 2026-10-09
> 대상: [46 본문](./04-46-input-and-context-evidence.md), [MAIN](./diagrams/choice46-structure.svg), [사건도](./diagrams/choice46-timeline.svg). 독립 agent 또는 외부 리뷰가 아니다.

## 1. 승인 범위와 설계의 요점

사용자는 45를 보존하고, 음성과 화면, 마우스 관측부터 Context 공급까지 넓힌 별도 46을 요청했다. 04-40의 로컬 VIA/로컬 VAD와 cloud voice/transcription/semantic 모델을 적용했다. 45의 과거 설명 실제 전달과 두 Task/result 참조 사례를 남기고, 두 화면을 말하며 가리킨 u46과 후속 정정 u47을 추가했다.

원본 취득/기본 시간 대응은 공통 Interaction Manager에 있다. Context Manager의 A는 원본 owner read로 요청별 EvidenceBundle을 구성하고, B는 Evidence Publisher가 temporal/historical record를 생산/검사/게시한 뒤 Evidence Reader가 소비한다. B의 입력 확정은 게시를 기다리지 않지만 정상 Context 소비는 유효 게시 근거가 필요하다. 현재 지칭 의미와 Task 연결은 Request Interpreter가 판단하고 Request Controller가 채택한다.

중앙 raw 기록 대 생산자별 기록 조회를 새 A/B로 제안하지 않는다. 45의 사용자 검토 원칙대로 기본 구조의 손익과 cache/index/부분 갱신 보완을 구별한다. A도 유효 근거를 재사용할 수 있으며, 화면 사례 추가만으로 B의 우위나 주요 DP 자격이 강화됐다고 주장하지 않는다.

## 2. 본문/그림 대조와 보완

| 확인한 항목 | 반영과 한계 |
| --- | --- |
| 같은 입력과 미확정 Task | 양안의 u46@r1/W46, S1/P1→S2/P2, C20/E20/P20와 T21/D21, T22/D22. R46/T46은 현재 판단 결과 |
| clock/acoustic/arrival 구별 | audio sample과 provider item 대응, capture interval/epoch/오차/gap. 수신 시각이나 VAD로 단어 timing을 만들지 않음 |
| 실제 데이터 생산/소비 | IM 공통 관측 window, A Composer/set, B Publisher/repository/Reader. publication 의존과 NOT_COVERED backfill, UNSUPPORTED_RELATION 및 B+ 원본 우회 구별 |
| source owner와 상태 | 과거 입력/질문 read는 Request Controller, 원문/실제 전달은 Response Manager, 확인된 결과는 Task Manager. User Memory는 Context Manager 소유 |
| 코드와 cloud 모델 | temporal/ID/revision join은 코드, 현재 C-INTERPRET와 선택적 C-CONTEXT 구별. 두 Model Access 박스는 같은 Component의 client 확대 보기 |
| 정상/정정/실패/취소/만료 | E1~E11, 늦은 revision과 admission, dependency 무효화, 원본 소실, 철회 gate와 재시도 cleanup. 사라진 원본을 파생 view로 복원하지 않음 |
| 선택 조건과 반증 | V-01~13, cloud 금액 비용, 첫 화면 요청/재사용/과거만 사용/미지원 분리. A의 cache와 B의 미사용 생산/수정/backfill 비용 포함 |
| 다른 DP와의 연결 | 40/41/42/43/44와 책임 구별. 42B+44의 hold/전송 미결을 이번 작업으로 해결했다고 하지 않음 |

첫 렌더에서 하단 호출 설명과 공통 주석, 사건도의 두 설명 줄이 겹쳐 위치를 보완했다. 상태 원통의 ellipse와 이름, optional cache와 Request Evidence Set의 연결, B의 독립 현재 해석 경로를 보완했다. Response/Task 결과뿐 아니라 Request Controller의 과거 입력/질문 read를 메인에 추가했다. 변경한 두 최종 PNG를 직접 열어 대조했다.

## 3. 문서와 그림 검증

검증 대상은 문서/그림이다. 아래 검증은 VIA 실행, 모델 성능, timing 정확도나 품질 우위의 측정이 아니다.

```sh
.venv/bin/python scripts/architecture/generate_evidence_context_diagrams.py --check
.venv/bin/python scripts/architecture/check_active_markdown_links.py
.venv/bin/python scripts/architecture/check_active_terminology.py
.venv/bin/python scripts/architecture/check_qa_catalog.py
```

전용 renderer는 `scripts/architecture/render_evidence_context_diagrams.cjs`다. 설치된 Playwright/Chrome으로 2560×1440 PNG 두 장을 생성한다. SVG 텍스트의 실제 glyph 너비, canvas 경계와 서로 겹치는 글자를 검사하고 각 그림의 `*-render.json`에 SVG/PNG SHA-256을 기록한다. 전용 serializer는 owner containment, node/edge ID, 끝점/직각 경로와 관련 없는 node 내부 통과를 검사한다. SVG와 draw.io는 같은 scene에서 생성한다.

46 관련 상대 링크와 전체 활성 Markdown, 용어/QA 일관성, 새 문서/그림의 가운데점 제외, 변경 scope를 확인한다. 독립 리뷰나 전체 발표 PPTX 동기화는 하지 않았다. 기존 다른 DP/전체 Architecture의 cloud 전제 동기화는 04-40의 후속 과제로 남긴다.

| 실행한 검증 | 결과 |
| --- | --- |
| 전용 생성기 `--check` | MAIN/사건도 SVG와 draw.io 일치, node containment/port/직각 경로/관련 없는 node 통과 검사 PASS |
| 두 PNG 렌더와 직접 확인 | 2560×1440, glyph 너비/canvas/글자 겹침 0건, SVG/PNG hash 일치 |
| 활성 Markdown 링크 | 156개 활성 파일 PASS, 46 및 초기 초안의 후속 링크 포함 |
| 활성 용어와 QA catalog | 정식 Component 명칭 및 legacy 혼입 검사 PASS, 기존 23 QA ID/4 ASR 등 catalog 일관성 PASS |
| 45 원본 보존 | 16개 파일 시작 SHA-256 일치 |
| 변경 scope와 whitespace | 다른 DP diff를 이번 작업으로 취급하지 않음. 46 파일과 안내/후속 링크 범위 확인 |

## 4. 보존과 재개

45 본문/검수, 배경과 MAIN/수명 그림, 생성기와 개별 발표 자료의 **16개 파일을 작업 시작 SHA-256과 대조해 변경 없음**을 확인한다. 시작 manifest는 세션의 `/private/tmp/via46-preserved-45.json`이고, 파일 목록은 다음과 같다.

- `04-45-memory-and-context.md`, `04-45-memory-and-context-review.md`
- diagrams의 `dp45-background.svg/.drawio`, `choice45-structure.svg/.drawio/.png`, `choice45-lifecycle.svg/.drawio/.png`
- `scripts/architecture/generate_memory_context_diagrams.py`, `scripts/architecture/memory_context_scene.py`
- `docs/presentations_files/dp-comparison/dp45-comparison.svg/.drawio/.png`, `docs/presentations_files/dp-background/dp45-background.png`

46의 새 문서/그림/전용 생성기와 읽기 안내 및 작업 계획의 46 블록, 초기 관측 초안의 후속 상태/46 링크가 이번 변경 범위다. 초기 관측 A/B의 본문과 그림은 논의 이력으로 남으며 별도 주요 DP 추천이 아니다. 다른 세션의 41/40 및 통합 발표 변경, 기존 04-50과 `work/`를 보존한다. commit/push는 이번 요청에서 하지 않았다.

재개 시 [46 §2](./04-46-input-and-context-evidence.md#2-메인-비교-같은-입력-원본-다른-근거-공급-구조)에서 비교 그림을 보고 §4 공통 시간 계약, §5~7 공급/수정 계약, §8~10 비용/품질/선택 조건을 확인한다. provider 단어/span timing, 실제 capture budget/오차와 보존 수치, 모델 payload/capability와 요금, schema/helper 선택은 미검증/미결이다. 사용자 리뷰 전 45의 대체나 최종 A/B 선택으로 처리하지 않는다.
