# 41 검토 기준을 이어가는 44·45·42·46 재구체화 작업

> 2026-10-10 사용자 위임: 44 → 45 → 42 → 다른 세션의 46까지 본문·그림·발표자료를 완성하고 검증된 작업 단위마다 main에 commit/push한다. 이 파일은 재개 시 00-workplan과 함께 읽는 이번 작업의 지속 기록이다.

## 범위와 승인

실제 프로그램에서 두 대안의 판단·실행·조회·상태·확정·복원 차이가 추적되게 구체화한다. 박스 이름만 다르게 하거나 차이를 과장하기 위해 기능을 바꾸지 않는다. 참조 Architecture 전체 변경, 최종 A/B 선정, 구현·모델 실행·품질 측정은 이번 문서 완성에 포함하지 않는다. 강한 대안과 조건부 이익·비용·반증 조건을 유지한다.

VIA와 상태 권위는 로컬이며 음성/전사와 의미 모델은 클라우드 의존성이다. 로컬 VAD는 Interaction Manager의 Turn-Taking Control에 있다. 모델의 영속 업무 권위는 없다. 기존 on-device 참조 구조와 accepted/deferred ADR 및 QA/evidence는 보존한다. 호출 수는 진단이며 모델 사용 비용에는 배경 생산·retry·폐기된 청구 작업도 포함한다.

## 41에서 확립한 작성·검수 기준

1. 구조를 그리기 전에 입력·출력·제어권·상태 owner·수명·확정·복원 계약을 쓴다. 정식 Component 명칭을 사용하고 내부 Module·데이터 상태·실행 라이브러리·배치 조건을 구별한다.
2. 같은 원본·사용자 목표·후보·정정/실패 조건을 양안에 준다. 해석 결과인 Task를 입력 정답으로 미리 제공하지 않는다. ID는 필요한 것만 쓰고 뜻을 그 자리에서 설명한다.
3. 실제 데이터 key/type/허용 연산·미해결·오류 예시를 제시한다. 코드가 자유문장 의미를 몰래 해석하지 않게 한다. Schema/예시 검사와 실제 의미 정확성 시험을 구별한다.
4. 구조도에는 이름만 든 Component/Module 박스와 데이터/조건 화살표를 유지한다. 반복은 반환선·다음 실행 주체로 드러내고 필요하면 별도 순서도로 보완한다. 순서도 동작/분기를 Module로 취급하지 않는다.
5. 큰 경계는 흰색/투명이며 차이는 테두리로 표시한다. 차이 작은 Module은 살구색, 공통은 흰색/검정, 선은 얇게 한다. 공통 Legend·owner 안의 상태를 유지한다.
6. 문서 MAIN은 전체 협력을 남기고 발표는 핵심을 확대한다. 집중 그림은 원본 문서에도 추가하며 기존 전체 그림·사건도를 임의로 대체하지 않는다.
7. A의 비동기·cache·부분 갱신과 B의 필요한 추가 모델 호출을 허용한다. 기본 구조 비교 후 선택한 안의 약한 QA를 tactic으로 보완하는 과정은 정상적이다. 측정 없이 고정 우열·숫자 점수를 만들지 않는다.
8. 출력 준비/실제 전달, 로컬 저장/내부 접수/외부 Agent 접수를 구별한다. 현재성 검사와 의미 정답 보증을 혼동하지 않는다.
9. 본문·SVG/draw.io·PNG·발표자료/notes를 동기화하고 실제 렌더를 검수한다. shared generator/통합 PPTX는 한 writer가 관리한다. 비선택 slides/parts를 보존하고 다른 세션 변경은 임의 게시하지 않는다.

## 완료 체크리스트와 순서

각 단계는 본문·고정 데이터 예시·구조 차이와 실행 반복의 그림·정상/정정/취소/실패/늦은 결과·코드/모델 호출·품질 손익/반증·독립 검토·렌더·관련 check·scoped commit/push로 마무리한다.

- [x] 41의 구체 출력과 반복, 원본 문서 추가 비교 그림 게시: 7e189129 / 5febf034.
- [x] 44: 중앙 continuation 회수/재지시와 Stage별 입력·대기·Join을 같은 네 사건으로 비교. 둘 다 비동기·제한 executor. 공통 cloud/local 계약 적용.
- [x] 45: 현재 요청별 원본 조합과 공통 파생 관계 생산/조회. 생산 trigger·미게시 대기·현재 사용 검증·조건부 모델 호출·전체 비용을 명시.
- [x] 42: 통합 Core 공동 확정과 독립 서비스 command/receipt. 실제 수명/상태 권위/복원 차이. 44의 공동 transaction 전제를 42B와 결합할 hold/ack·전송 권위 프로토콜 상세화.
- [x] 46: 다른 세션 작업 완료 여부를 먼저 확인. 동일 파일 동시 수정 금지. 시간 원본/관계 생산의 실제 차이와 45 범위 구분, Module/contract/cloud/자료·발표 일치 검수 후 이어받아 게시.
- [x] 조합 검토: 41의 READ/CLARIFY/PROPOSE와 44의 실행,45의 근거,42의 저장·전송,46의 시간 근거 공급을 추적. 호환 조건/혼합/제한을 명시.
- [x] 최종 게시/검증 보고: 실제 Git SHA·CI·렌더·검수 범위/남은 실험 과제를 기록.

## 병행 작업과 재개 규칙

QA 정의(03-02)는 별도 세션의 검토 중 자료다. 최신 정의를 읽되 QA ID·집계·우선순위를 임의 확정하지 않는다. 46 작업은 다른 세션이 active하므로 먼저 읽기 검토하고 해당 writer가 끝난 뒤 수정한다. 공유 00-workplan/README와 04-50·work/의 기존 미커밋 변경을 보존한다. 각 commit은 explicit path 또는 해당 작업만의 staged patch로 구성한다. 필요한 공통 동기화와 조합 문서는 마지막 작업 단위에 별도로 게시한다.

재개할 때 git status/log, 이 체크리스트, 04-40, 해당 DP 및 최신 review를 읽는다. 남은 체크를 완료로 꾸미지 않으며 실제 검증 결과와 커밋을 단계별로 아래에 추가한다.

## 작업 이력

- 시작: 44 본문/scene 검토. 독립 agent가 45,42 조합,46 경계를 읽기 전용 검토한다. 구현이나 모델 실행은 하지 않는다.

- 44 게시 단위: 고정 네 사건/continuation/Window와 집중 그림, 전체 MAIN 직접 Task 조회, cloud/local 계약, 42A/B 확정 연결. 독립 재검토 미해결 P1/P2 없음; SVG/font와 native PPTX 실제 렌더 검수. 두 PPTX는 44 slide/notes만 바꾸고 비선택 parts를 보존. 전체 비용/현재성/확인 질문 예외와 미구현·미측정 상태를 유지. 커밋 SHA는 Git 이력에서 이 기록의 게시 commit으로 확인한다.

- 45 게시 단위: Evidence Contract v1/JSON의 실제 전달 원문 구간·PARTIAL/gap, code/model 생산 경계, 원본별/공통 생산·조회 graph와 NOT_COVERED loop, 원본에 추가 집중 그림, cloud 비용. 독립 읽기 검토 미해결 P1/P2 없음. Chrome 3종 실제 렌더와 두 native PPTX 검수/비선택 parts 보존 PASS. 전체 MAIN과 focused source를 분리했다.

- 42 게시 단위: 같은 질문/답변의 수명·owner·공동 확정/별도 접수, hold/ACK/CAS와 전송 권위·UNKNOWN/reconciliation 계약. 전체 MAIN/사건도와 추가 집중 그림, 실제 데이터/4개 조건부 QA 행, 두 native PPTX의 42 part만 동기화. 실제 SVG/native 렌더·정적 예시·생성물 일치 PASS, 독립 검토 미해결 P1/P2 없음. 구현/측정/외부 exactly-once 보증 없음.

- 46 인수/발표 게시 단위: 다른 세션 원본 dd1350cd8 이후 같은 14개 기준 좌표와 실제 원본/생산·조회 topology를 유지. 공통 Legend·42 계약 상태 정정, temporal 고정 JSON/정적 검사·조건별 QA 추가, 5장 native editable PPTX 및 source/hash/text 검사·실제 SVG/native 렌더. 45와 시간×과거 교차 조합을 구별하며 전체 효과를 중복 합산하지 않음. 독립 검토 미해결 P1/P2 없음(4개 구조 페이지); 추가 QA는 통합 writer와 독립 검토자가 시각·본문 대조.

- 공통 조합 완료: composition-review의 16개 조건부 연결과 46의 2×2, owner/port·모델 호출 지도·같은 경합을 문서 대조. 독립 P2 1건(ACK 미수신을 HOLD 적용과 혼동하는 문장)을 수정했다. 40은 42의 상세 계약과 연결하고 45의 46 링크를 복원. 게시 사본 전체 CI 검사 PASS; 원격 CI는 44 a544fd50b / 45 수정 dd9444041 / 42 469648469 / 46 원본 dd1350cd8·발표 fd282acc4까지 PASS. 최종 SHA/CI는 이 기록을 게시한 Git 이력과 GitHub Actions로 확인한다.

## 완료 후 다시 읽을 자료

[공통 40](./04-40-common-execution-contract.md) → [41](./04-41-request-resolution-control.md) → [44](./04-44-continuous-interaction.md) → [45](./04-45-memory-and-context.md) → [42](./04-42-lifecycle-ownership.md) → [46](./04-46-input-and-context-evidence.md) → [조합 검토](./composition-review.md). SVG/draw.io/PNG·native PPTX의 일치와 문서 예시 검사는 완료했고 각 review가 범위를 명시한다. 다음 단계는 사용자 리뷰/선택과 별도 승인하의 구현·QA 실험이다. 별도 QA 세션이나 기존 Architecture 전체 동기화는 이 완료 작업의 미처리 항목으로 재시작하지 않는다.
