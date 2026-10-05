# 04-45 독립 리뷰와 수정 기록

> 대상: [45 본문](./04-45-memory-and-context.md), 전용 생성기, 메인/수명 SVG와 draw.io 및 PNG / 2026-10-06
> 독립 리뷰는 작성 작업을 맡지 않은 별도 에이전트의 문서/설계 검토다. 실제 사용자 심사, 실행 시험, 품질 우위나 최종 DP 선정이 아니다.

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
