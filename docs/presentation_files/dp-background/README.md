# VIA DP 배경 슬라이드 — 04-41~45

각 문서의 공통 사용자 상황, 설계 난점, 구조적 선택 질문을 담은 **1920×1080 / 16:9 / 문서별 1페이지**다. `presentation_example1`과 `presentation_example2`의 배경 설명·상황 도식·핵심 설계 과제 구성을 참고했다.

[5페이지 draw.io 편집 원본](./VIA-DP-background-41-45.drawio) / [전체 미리보기](./index.html)

| 문서 | 배경 주제 | 개별 draw.io | SVG | PNG |
| --- | --- | --- | --- | --- |
| [04-41](../../architecture/12-decisions/decision-packages/04-41-request-resolution-control.md) | 사용자 화면과 Request 해석에 필요한 Context | [원본](../../architecture/12-decisions/decision-packages/diagrams/dp41-background.drawio) | [미리보기](../../architecture/12-decisions/decision-packages/diagrams/dp41-background.svg) | [삽입용](./dp41-background.png) |
| [04-42](../../architecture/12-decisions/decision-packages/04-42-lifecycle-ownership.md) | 대화·요청·질문·업무·외부 실행의 다른 수명 | [원본](../../architecture/12-decisions/decision-packages/diagrams/dp42-background.drawio) | [미리보기](../../architecture/12-decisions/decision-packages/diagrams/dp42-background.svg) | [삽입용](./dp42-background.png) |
| [04-43](../../architecture/12-decisions/decision-packages/04-43-request-interpretation.md) | F1~F6의 상호 의존과 의미 생산 책임 | [원본](../../architecture/12-decisions/decision-packages/diagrams/dp43-background.drawio) | [미리보기](../../architecture/12-decisions/decision-packages/diagrams/dp43-background.svg) | [삽입용](./dp43-background.png) |
| [04-44](../../architecture/12-decisions/decision-packages/04-44-continuous-interaction.md) | 지속 발화와 비동기 업무 사건의 실행 조직 | [원본](../../architecture/12-decisions/decision-packages/diagrams/dp44-background.drawio) | [미리보기](../../architecture/12-decisions/decision-packages/diagrams/dp44-background.svg) | [삽입용](./dp44-background.png) |
| [04-45](../../architecture/12-decisions/decision-packages/04-45-memory-and-context.md) | 흩어진 과거 근거의 조합과 공통 기억 계약 | [원본](../../architecture/12-decisions/decision-packages/diagrams/dp45-background.drawio) | [미리보기](../../architecture/12-decisions/decision-packages/diagrams/dp45-background.svg) | [삽입용](./dp45-background.png) |

draw.io에서 **File → Open from → Device**로 원본을 열면 텍스트, 도형, 연결선을 개별 편집할 수 있다. 통합 파일에는 다섯 페이지가 있으며, 개별 파일과 SVG는 각 Architecture 문서에 연결했다. PNG는 발표자료 삽입용이다.

이 슬라이드는 기존 비교의 대안 정의와 미선정·미측정 상태를 따른다. 배경 도식은 공통 문제의 설명이며 최종 구현의 호출 그래프나 대안 선택 결과가 아니다. 특히 42의 다른 논리 수명은 두 안의 공통 요구이고, 43은 A/C 표기를 유지한다.

## 발표 용어와 배경 페이지 작성 기준

2026-10-06 사용자 리뷰에서 합의한 기준이다. 이번 재작성은 41번에 적용하며, 42~45의 기존 페이지는 이후 개별 리뷰를 위해 유지한다. 아래 표는 발표의 표기 기준이며 Architecture의 기존 계약이나 QA 정의를 변경하지 않는다.

| 영어 고정 용어 | 의미 |
| --- | --- |
| Request | VIA가 해석하고 처리하는 사용자 요청. User Request와 VIA Request를 구분할 때는 정식 명칭 사용 |
| Task | 하나 이상의 Request에 걸쳐 상태를 지속적으로 추적하는 사용자 업무 목표 |
| Delegation | 실제 업무 수행을 Downstream Agent에 맡기는 행위 |
| Referent | “이 표” 또는 “아까 보고서” 같은 사용자 표현이 가리키는 것. 화면에 보이는 것으로만 제한하지 않음 |
| Conversation | 사용자와 VIA 사이의 이어지는 논리적 대화 기록 |
| User Turn | 사용자의 한 번의 발화 또는 메시지 입력 |
| Context | 현재 Request를 이해하고 처리하는 데 사용하는 관련 정보 |
| Response | VIA가 사용자에게 전달하는 답변, 질문, 진행 안내 등 |
| Downstream Agent | 실제 업무 수행을 담당하는 외부 Agent |
| Agent Execution | Downstream Agent의 개별 실행 단위 |
| User Memory | 사용자가 허용한 개인화 기억 |
| Direct Response | Downstream Agent의 새로운 업무 수행 없이 VIA가 직접 응답하는 경로 |
| Clarification | 불명확한 Request를 보완하는 확인 질문 |
| Task Association | 현재 Request와 관련 Task의 연결 판단 |
| Orchestration | VIA의 Request 처리와 Agent 실행 흐름을 연결하고 관리하는 책임 |
| Component / Module | 설계 구성요소와 그 내부 구현 단위. 구성요소 이름은 원문 정식 명칭 유지 |

- 시스템 용어는 위 영어 표기, 설명은 쉬운 한국어. 사용자 발화 인용은 자연스러운 한국어 유지.
- 일반 표현은 정보, 해석, 조회, 조건, 저장, 삭제, 정정, 취소로 통일. 배경 설명의 “근거”는 구체적인 “정보”로 표현.
- 제목은 “(품질속성)을 위한 (VIA 기능) 설계”. 품질속성은 정확성, 반응성, 변경 용이성 등 한국어로 표현.
- 사용자 발화 인용을 제외한 설명은 명사형. 가운데 점 사용 금지.
- 오른쪽 영역은 “설계 고려 사항”. 사용자 상황에서 필요한 기능과 고려할 조건을 설명.
- 왼쪽 그림은 화면과 정보의 개념 관계. Component 배치, 처리 순서, 모델 또는 코드의 생산 권한과 A/B 결론은 후속 설계 페이지에서 설명.
- 41의 표 수치와 이전 Request는 상황 이해를 위한 가상 화면 데이터. 두 보고서는 동일한 Task 후보이며 수정할 Task의 정답을 미리 지정하지 않음.

원본 장면은 [generate_dp_background_slides.py](../../../scripts/architecture/generate_dp_background_slides.py)에 있다. 저장소 루트에서 다음 명령으로 개별 SVG/draw.io, 통합 draw.io 및 HTML을 재생성하거나 일치를 확인한다.

```sh
python3 scripts/architecture/generate_dp_background_slides.py
python3 scripts/architecture/generate_dp_background_slides.py --check
```

PNG는 같은 SVG를 1920×1080으로 렌더링한 결과다. 수정 후 해당 PNG도 함께 갱신한다.
