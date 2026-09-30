# 독립 검토와 보완 기록

> 2026-10-01 · 문서·구조·렌더 검토. VIA 구현·성능 검증이 아니다.

## 이번 검토의 기준

직전 세대의 검토는 lifecycle 계약을 상세히 보는 데 비해, 사용자가 요구한 **실제 Component·실행체·저장 체계의 차이와 그림의 전달력**을 충분히 검증하지 못했다. 그 세대의 완성 평가는 철회하고 원본을 archive했다. 이번에는 선발 gate부터 다시 적용했다.

작성자와 별도의 검토 에이전트 세 역할로 읽기 전용 검토를 수행했다. `architecture_review`는 target 충실성·실제 책임 이동·steelman 기능 완결성, `quality_review`는 ASR/QA 인과·공정한 비교·과장, `diagram_review`는 8개 실제 렌더와 문서·그림 topology의 일치를 검토했다. 사람 Senior Architect의 승인이나 실제 제품 검증을 대신하지 않는다.

## 발견사항과 수정

| 검토 단계 | 발견사항 | 수정 내용 |
| --- | --- | --- |
| 후보 선발 | conversation projection도 가능하지만 기존 cache와 달리 독립 서브시스템을 운영할 근거가 약함 | 5개 탐색에서 4개 주력으로 좁히고 재검토 조건을 discovery에 기록 |
| 구조 1차 | workflow의 고정 activity 목록에서 UC-17 기억 변경 경로 누락 | ApplyUserMemoryChange 및 Context Manager owner·change ID·revision·epoch·atomic invalidation 계약 추가 |
| 구조 1차 | retrieval의 현재 정책·삭제 권위를 Context Manager 하나로 혼동 | Policy Manager Use Envelope와 Context Manager data/deletion epoch를 분리, 검색 서비스는 강제만 수행 |
| 구조·품질 1차 | top-1 원문 최신성 확인만으로 대상 유일성·coverage가 확보되는 것처럼 읽힘 | exact identity·bounded owner 보완·사용자 선택 전 외부 Action 보류 |
| 품질 1차 | WAIT_USER 자원 해제와 durable wait를 workflow 고유 이점으로 읽을 여지 | 양안 공통임을 명시. 대안 이점 가설을 ready signal/timer registry와 재개 대상 탐색으로 한정 |
| 품질 1차 | 인식 지연이 반드시 잘못된 화면 연결로 이어진다는 과장 | 보존된 당시 timeline이면 늦게 정렬 가능. 대기·유한 buffer와 현재 화면 대체 오류를 구분 |
| 구조·품질 1차 | 복구 비교에서 동일 저장량 가정, 일반 데이터 손상 복구로 과장 | 동일 논리 상태·이력·DB 내구성만 고정, 실제 저장량은 비용. 정상 commit 이후 projection 손상과 의미 오류·권위 원본 손상 구분 |
| 그림 1차 | retrieval A에 Context Manager가 두 instance처럼 보임 | 하나의 경계로 묶고 내부 Evidence Package Assembly로 표시 |
| 그림 1차 | retrieval B에 query embedding 경로와 Context Manager의 검색 위임 경로 누락 | Request Controller→Context Manager→검색 서비스→Context Manager, 공유 embedding 요청·vector 반환 연결 추가 |
| 그림 1차 | 검색·소비 gate의 사용 차단과 비동기 purge 혼동 | 현재 policy·data epoch의 gate 입력 및 purge 경로 분리 |
| 그림 1차 | 양안 공통 cache가 1안에만 파란색으로 표시됨 | 1안 cache 검정, 2안 Context Manager의 기억·조립·cache 유지 문구 검정 |
| 그림 1차 | Interaction Manager capture 선이 semantic job 호출까지 소유하는 것처럼 보임 | audio 선으로 정정; semantic owner를 변경하지 않음 |
| 최종 재검토 | 장애 시 동일 기능 유지라는 선발 문구가 native 인식 중단과 충돌 | 정상 기능/완료 조건과 장애 시 공통 실패·보류 요구를 구분, 손실 범위를 공개하도록 수정 |
| 최종 재검토 | R을 단순히 개선을 주장하지 않는 축으로 정의 | 직접 악화 인과도 P이며 R은 구조가 직접 바꾸지 않는 공통 경로임을 명시 |
| 최종 재검토 | 복구 반례가 양안 current row를 모두 파생물로 표현 | 1안 권위 current record / 2안 파생 projection의 손상임을 구분 |
| 최종 그림 재검토 | 검색 A 외부 박스가 CM 경계에 접촉, B purge 라벨이 후보 반환선에 붙음 | 외부 박스 간격과 purge 라벨·별도 경로 수정 |

## 최종 재검토와 검사 기록

세 검토자가 마지막 수정까지 재확인했으며 담당 검토 범위에 미해결 P1/P2가 없음을 확인했다. 최종 재검토에서 발견한 표현·경계·라벨 문제도 위 표에 남겼다. 구현·측정에 따른 미래 결함이 없다는 뜻은 아니다.

| 확인 | 결과 |
| --- | --- |
| active Markdown local links | PASS · 102개 문서 |
| active terminology·정식 target Component 이름 | PASS |
| QA catalog | PASS · 23 active IDs, 현재 4 ASR 정의 변경 없음 |
| target 그림 원본 일치 | PASS · 기존 13쌍 변경 없음 |
| 새 후보 그림 원본·XML ID·연결·경계·route | PASS · 8쌍 |
| 브라우저 실제 SVG 렌더 | 8장 확인, node glyph overflow 0. 작성자·독립 그림 검토자가 실제 이미지와 의미 대조 |
| 보관 무결성 | 48파일 모두 원본 커밋 byte 및 manifest SHA-256 일치 |
| 보호 범위 | target·QA·ADR·benchmark·prototype·results 변경 없음 |
| 변경 검토 | git diff 및 diff --check 확인. active archive 링크는 보관 안내 용도이며 현재 규범은 target/QA로 연결 |

Geometry PASS만으로 설계가 정확하다고 판정하지 않았다. SVG의 실제 문구·경계·호출 주체는 독립 검토가 별도로 확인했다. Git 전달 SHA와 CI 상태는 이 자료를 포함한 커밋의 GitHub Actions 기록을 따른다.

## 검증의 한계

- 승인된 target·QA·ADR·기존 VIA-DP 및 이전 결과는 변경하지 않았다.
- Native evidence capability·embedding 모델·workflow 구현·replay 성능을 실행하거나 확인하지 않았다.
- SVG를 브라우저로 렌더링했고 draw.io XML·parent·source/target·동일 scene 생성을 검사한다. draw.io 편집기 자체의 화면 호환을 실사용 검증한 것은 아니다.
- 최종 DP 선정·메모리 ASR 승격·모집단/기준 freeze는 별도 논의 대상이다. 정성적 장단점 표를 실측 우열로 읽지 않는다.

## 후속 검토 — 기호 비교와 DP별 추가 QA (2026-10-01)

사용자 요청으로 각 상세 문서에 공통 5항목의 `(+)·(-)·(0)` 표와 추가 QA 4항목을 넣었다. 판단 조건, 구조에서 생기는 차이, 사용자에게 중요한 이유를 쉬운 말로 설명하고 기존 상세 적용·검증 표를 유지했다. 추가 QA 후보는 별도 ID·ASR로 확정하지 않았다.

| 품질 검토 지적 | 보완 |
| --- | --- |
| 같은 trace 요구를 두었다고 실제 QA-61을 (0)로 볼 수 없음 | 음성의 해당 행을 동일 native 관측성 확보 조건의 **진단 부담** 비교로 변경. 두 producer 연결·불일치 추적의 수고와 단일 producer를 비교하며 전체 trace 완전성 판정과 분리 |
| 같은 보안 요구가 실제 노출·오실행 위험의 동등함을 뜻하지 않음 | (0)을 변경하지 않는 최종 Policy Manager/Request Controller 검사 범위로 한정. 새 검색 gate·workflow 연결의 위험은 별도 비교 |
| 코드 재사용을 QA-29 변경 Element 감소와 혼동 | Workflow의 유리 조건을 실제 진행 State·재개 Interface 변경 대 기존 엔진/활동 고정·정의 변경으로 좁힘. Target도 graph 데이터만 바뀌면 차이 없음을 명시 |
| 과거 자료 부족을 QA-29 변경 용이성의 불리함으로 혼동 | 복구의 새 view 행은 양안 동일 근거·동일 변경 범위일 때 (0). 기능 실현 가능성과 변경 Element 수를 분리 |

이 검토는 문서의 품질 인과와 표기 의미에 대한 정적 검토다. 구현·성능 측정·target 재설계·그림 변경은 수행하지 않았다. 공통 5항목 존재, DP별 추가 4행과 양안 기호, Markdown 표 열 수를 확인했다. Repository 링크·용어·QA catalog·기존 그림 원본 검사와 git diff 검토를 통과했다. 독립 품질 검토자가 지적한 수정 사항을 재확인했으며 이번 회귀 검토 범위에 미해결 P1/P2가 없다.
