# 04-45 독립 리뷰와 수정 기록

> 대상: [45 본문](./04-45-memory-and-context.md), 전용 생성기, 메인/수명 SVG와 draw.io 및 PNG / 2026-10-06
> 독립 리뷰는 작성 작업을 맡지 않은 별도 에이전트의 문서/설계 검토다. 실제 사용자 심사, 실행 시험, 품질 우위나 최종 DP 선정이 아니다.

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
