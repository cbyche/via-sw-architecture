# 후보 비교 그림 작성 규칙

> 논의용 구조 비교 · [후보 목록](../README.md) · [선발 원칙](../selection-principles.md)

Senior SW Architect가 그림만 보고도 두 방안의 책임·상태·계약 차이와 비용의 원인을 검토할 수 있도록 작성한다. 각 SVG에는 사용자 상황, 양쪽 구조, 공통 조건, 이점·비용·검토 질문을 함께 넣는다.

## 색과 경계

| 표현 | 의미 |
| --- | --- |
| 검정 | 두 방안에서 유지되는 기능·구조·계약 |
| 파랑 `#0057B8` | 양쪽에서 달라지는 책임 경계·상태 owner·연결. 방안 1에도 같은 기준 적용 |
| 파란 외곽 + 검정 내부 | 기능은 유지되지만 그 기능을 소유하는 경계가 달라짐 |
| `«component»` 외곽 | 정식 논리 Component 또는 표시된 대안 전용 Component |
| 외곽 안의 작은 사각형 | 설명에 필요한 내부 책임·처리 모듈. 승인된 추가 Component가 아님 |
| `«process / runtime»` 점선 외곽 | 실행·process 장애 경계 |
| 이름이 있는 점선 dependency | 모델 또는 외부 Agent·service. VIA Component 수와 구분 |
| 원통 | State Store 안의 논리 기록. 여러 원통이 별도 DB를 뜻하지 않음 |
| `공개 조회 계약` 묶음 | 원본 owner의 read port를 모은 표현. 새 Component가 아님 |

방안 1은 검토 완료 기준선의 구조를 발췌하고, 방안 2는 같은 사용자 목표를 처리하는 논의용 대안으로 그린다. 색으로 승인 여부나 우열을 나타내지 않는다. 정식 Component 이름을 줄이거나 alias로 대체하지 않는다.

## 연결과 읽는 순서

번호는 핵심 시나리오를 설명하는 순서다. 실선 화살표는 호출·event·전달, 점선 화살표는 상태 조회·기록을 뜻한다. 양방향 화살표는 요청/반환 또는 release/receipt 같은 왕복 계약을 묶어 표시한다. 선이 교차하는 위치는 접속점이 아니다.

각 그림은 해당 선택의 핵심 경로를 보여준다. 모든 호출, thread scheduling, transaction 순서를 확정한 실행 명세는 아니다. 예를 들어 runtime 그림은 논리 호출 전체보다 process 경계와 주요 IPC를 보여준다. 생략한 공통 연결은 그림 주석과 후보 본문에서 밝힌다.

내부 블록을 늘릴 때는 책임·상태·계약 중 하나를 설명해야 한다. 입력·결과·원장·실제 I/O를 연결하고, 관련된 예외를 구체적으로 적는다. 예외의 예는 늦은 revision, 정정/hold, 접수 불명, 실제 전달 불명, 삭제/철회, process 종료다. 그림의 장단점은 그 경로에서 설명할 수 있어야 한다.

## 원본과 수정

각 후보는 같은 이름의 `.svg`와 `.drawio` 한 쌍을 가진다. Markdown에는 SVG를 삽입하고 draw.io 원본 링크를 함께 둔다. draw.io에는 Component별 실제 중첩과 편집 가능한 도형·연결선을 보존한다.

현재 쌍은 [그림 생성 스크립트](../../../../../../scripts/architecture/generate_decision_candidate_diagrams.py)에서 동일한 노드·연결 정의로 생성한다. 저장소 반영 시 스크립트와 두 결과물을 함께 갱신한다. draw.io에서 먼저 수정했다면 같은 변경을 생성 정의에 반영한 뒤 재생성한다.

```bash
.venv/bin/python scripts/architecture/generate_decision_candidate_diagrams.py
.venv/bin/python scripts/architecture/generate_decision_candidate_diagrams.py --check
```

`--check`는 파일을 쓰지 않고 XML ID, 도형 중첩, 글자 폭의 보수적 추정, 화살표 endpoint, 직교 연결, 관계없는 leaf 박스 관통과 쌍의 최신성을 검사한다. 글자·화살촉·제목·선의 겹침은 렌더링으로 별도 확인한다. 생성기는 문서 도구이며 VIA 후보 구현이나 성능 측정 도구가 아니다.

ASR·점수·지연·메모리 수치를 그림에 만들어 넣지 않는다. 현재의 기대 효과는 가설이며, 사용자와 후보를 검토한 뒤 ASR 논의를 진행한다.
