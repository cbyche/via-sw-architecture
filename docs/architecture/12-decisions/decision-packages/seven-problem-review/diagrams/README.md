# 비교 그림과 방안별 상세 설계 그림

2026-10-18 / [설계 분석 진입점](../README.md)

## 주요 DP: 비교 그림을 본 뒤 방안별 상세도를 읽는다

각 링크는 GitHub에서도 표시되는 SVG다. 같은 이름의 `.drawio`는 개별 상자와 연결선을 편집할 수 있는 원본이며 `.png`는 문서/발표에 삽입할 이미지다. 각 본문에 SVG를 직접 삽입하고 원본/PNG를 연결했다.

| 주요 DP | 전체 비교 | A 상세 | B 상세 | C 상세 | 설명 |
| --- | --- | --- | --- | --- | --- |
| [근거 공급](../01-evidence-supply.md) | [비교](problem01-evidence-supply.svg) | [A](problem01-A-detail.svg) | [B](problem01-B-detail.svg) | [C](problem01-C-detail.svg) | 입력, source 읽기, view/bundle/게시 상태, 원본 보완, 철회와 채택 |
| [의미 판단 실행](../02-interpretation-execution.md) | [비교](problem02-interpretation-execution.svg) | [A](problem02-A-detail.svg) | [B](problem02-B-detail.svg) | 해당 없음 | 대화 실행/공통 경로/요청별 실행, 호출 자원, 현재성, 정정과 늦은 결과 |
| [화면 지칭](../04-screen-speech-grounding.md) | [비교](dp47-seven-comparison.svg) | [A](dp47-seven-A-detail.svg) | [B](dp47-seven-B-detail.svg) | 해당 없음 | 관측/원본 보존, 조건부 모델, 생산자/결합, 후보 확장, 판본/원본 수명 |
| [업무 인계](../05-agent-handoff.md) | [비교](problem05-agent-handoff.svg) | [A](problem05-A-detail.svg) | [B](problem05-B-detail.svg) | [C](problem05-C-detail.svg) | 공개 사실, 계약/기록/manifest, 외부 전송, 실제 수신/읽기, 철회와 복구 |

**합계: 주요 DP 비교 4장 + 기본안별 상세 10장.** 같은 DP의 상세도는 공통 입력/종료 조건과 큰 배치를 맞추어 비교한다. 기본 흐름은 정수 번호로 표시하며 저장 상태, cloud 모델 또는 외부 Agent, 오류/정정 경로와 선택 후 tactic을 구별한다. 도식은 설계 명세이고 구현 trace가 아니다.

## 독립 상세 설계로 남긴 세 주제

| 주제 | 비교 SVG | 편집 원본 | PNG |
| --- | --- | --- | --- |
| [DP-44 처리 단계 연결](../03-execution-coordination.md) | [SVG](problem03-execution-coordination.svg) | [draw.io](problem03-execution-coordination.drawio) | [PNG](problem03-execution-coordination.png) |
| [Agent 선택](../06-agent-selection.md) | [SVG](problem06-agent-selection.svg) | [draw.io](problem06-agent-selection.drawio) | [PNG](problem06-agent-selection.png) |
| [DP-42 상태/서비스 경계](../07-lifecycle-boundaries.md) | [SVG](problem07-lifecycle-boundaries.svg) | [draw.io](problem07-lifecycle-boundaries.drawio) | [PNG](problem07-lifecycle-boundaries.png) |

## 읽을 때 구별할 것

- 화살표는 필요한 입력/결과/제어의 방향이다. 두 갈래를 그렸다고 두 호출이 항상 동시에 실행되는 것은 아니다. 해당 본문의 기본 scheduling과 병행 tactic을 따른다.
- 단계가 모델을 필요로 하는 조건을 표시한다. native/명시 ID/완전한 typed 관계는 가능한 코드 경로를 유지한다. every-event LLM 구조로 읽지 않는다.
- 원본 저장과 현재 실행 메모리, 실제 사용자 전달, 외부 Agent 수신/수행 사실은 다르다.
- 입력/요청 판본과 source/권한 판본을 검사하는 코드가 의미 판단의 정확성을 대신하지 않는다.
- 하단 A1/A2 등의 보완은 기본에 자동 포함되지 않는다. 추가 상태/비용을 본문의 QA 표와 같이 읽는다.

## 재생성

문서 자산 생성기만 실행하며 제품/모델 시험을 포함하지 않는다. Pillow와 한국어 시스템 폰트가 필요하다.

```text
scripts/architecture/generate_seven_problem_figures.py
scripts/architecture/generate_seven_evidence_handoff_details.py
scripts/architecture/generate_seven_execution_details.py
scripts/architecture/generate_seven_grounding_details.py
```

SVG, draw.io와 PNG는 같은 node/edge 데이터에서 나온다. 다시 생성한 경우 텍스트 잘림, 화살표의 인과 의미와 방안별 기본/tactic 일치를 실제 이미지와 본문에서 다시 확인한다.
