# VIA 설계 문제 지도와 DP 배경 — 04-41~45

각 문서의 공통 사용자 상황, 설계 고려 사항과 설계 과제를 담은 **1920×1080 / 16:9 / 문서별 1페이지**다. `presentation_example1`과 `presentation_example2`의 배경 설명, 상황 도식과 핵심 설계 과제 구성을 참고했다.

[지도 포함 6페이지 draw.io](./VIA-DP-background-overview-41-45.drawio) / [배경만 5페이지](./VIA-DP-background-41-45.drawio) / [전체 미리보기](./index.html)

[전체 지도 PNG](./dp-background-overview.png) / [지도 draw.io](../../architecture/12-decisions/decision-packages/diagrams/dp-background-overview.drawio) / [지도 SVG](../../architecture/12-decisions/decision-packages/diagrams/dp-background-overview.svg)

대표이사 보고 관점의 리뷰를 반영하여 **핵심 위험 → 구체적 사례 → SW 구조의 결정** 순서로 여섯 장을 재구성했다. 지도에서 다섯 결정의 범위를 구분한 뒤 개별 배경과 후속 대안 비교로 이어간다. 노란색 문구는 예방할 설계 위험이며 관측된 결함이나 측정된 품질 차이가 아니다.

| 문서 | 배경 주제 | 개별 draw.io | SVG | PNG |
| --- | --- | --- | --- | --- |
| [04-41](../../architecture/12-decisions/decision-packages/04-41-request-resolution-control.md) | 사용자 화면과 Request 해석에 필요한 Context | [원본](../../architecture/12-decisions/decision-packages/diagrams/dp41-background.drawio) | [미리보기](../../architecture/12-decisions/decision-packages/diagrams/dp41-background.svg) | [삽입용](./dp41-background.png) |
| [04-42](../../architecture/12-decisions/decision-packages/04-42-lifecycle-ownership.md) | Conversation, Request, Task와 Agent Execution의 수명 및 관리 책임 | [원본](../../architecture/12-decisions/decision-packages/diagrams/dp42-background.drawio) | [미리보기](../../architecture/12-decisions/decision-packages/diagrams/dp42-background.svg) | [삽입용](./dp42-background.png) |
| [04-43](../../architecture/12-decisions/decision-packages/04-43-request-interpretation.md) | Request 정정에 함께 영향을 받는 의미 판단과 책임 | [원본](../../architecture/12-decisions/decision-packages/diagrams/dp43-background.drawio) | [미리보기](../../architecture/12-decisions/decision-packages/diagrams/dp43-background.svg) | [삽입용](./dp43-background.png) |
| [04-44](../../architecture/12-decisions/decision-packages/04-44-continuous-interaction.md) | 새 발화와 별도 Task 질문의 병행 처리 및 전달 | [원본](../../architecture/12-decisions/decision-packages/diagrams/dp44-background.drawio) | [미리보기](../../architecture/12-decisions/decision-packages/diagrams/dp44-background.svg) | [삽입용](./dp44-background.png) |
| [04-45](../../architecture/12-decisions/decision-packages/04-45-memory-and-context.md) | 과거 정정, 결과와 실제 전달의 연결 및 Context 제공 | [원본](../../architecture/12-decisions/decision-packages/diagrams/dp45-background.drawio) | [미리보기](../../architecture/12-decisions/decision-packages/diagrams/dp45-background.svg) | [삽입용](./dp45-background.png) |

draw.io에서 **File → Open from → Device**로 원본을 열면 텍스트, 도형, 연결선을 개별 편집할 수 있다. 지도 포함 파일은 여섯 페이지이고 배경 전용 파일은 다섯 페이지이며, 개별 파일과 SVG는 각 Architecture 문서에 연결했다. PNG는 발표자료 삽입용이다.

이 슬라이드는 기존 비교의 대안 정의와 미선정 및 미측정 상태를 따른다. 배경 도식은 공통 문제의 설명이며 최종 구현의 호출 그래프나 대안 선택 결과가 아니다. 특히 42의 다른 논리 수명은 두 안의 공통 요구이고, 43은 A/C 표기를 유지한다.

## 발표 용어와 배경 페이지 작성 기준

2026-10-06 사용자 리뷰에서 합의한 기준이다. 41과 42에서 정리한 가이드를 43, 44와 45에도 적용했다. 42의 잠정 완료본을 포함한 다섯 배경을 후속 사용자 지시에 따라 다시 보강했다. 이번 여섯 장의 독립 리뷰와 검증 범위는 [검수 기록](./REVIEW.md)을 따른다. 아래 표는 발표의 표기 기준이며 Architecture의 기존 계약이나 QA 정의를 변경하지 않는다.

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

1. **배경의 목적:** 사용자 상황, 필요한 기능, 설계 고려 사항과 설계 과제로 연결. 다음 장의 구조 비교가 필요한 이유까지 설명.
2. **구체적인 출발점:** 최초 발화나 화면에서 시작. 이미 Delegation 중인 추상 상태만으로 시작하지 않음.
3. **예시의 완결:** 예시에 등장한 모든 목표를 설명. 두 목표 중 하나를 누락하거나 후속 발화의 연결을 독자의 추측에 맡기지 않음.
4. **시간과 정보 관계의 구분:** 최초, 잠시 뒤, 완료 후와 지난번 등의 표식 사용. 필요한 정보의 나열을 처리 순서 번호로 표현하지 않음.
5. **용어 통일:** 위 영어 표기와 쉬운 한국어 설명 사용. 사용자 인용은 자연스러운 한국어 유지. 어색한 “근거”는 구체적인 “정보”로 표현.
6. **문구 형식:** 제목은 “(품질속성)을 위한 (VIA 기능) 설계”. 사용자 인용 외에는 명사형. 가운데 점과 어려운 명사의 나열 지양.
7. **설계 고려 사항:** 오른쪽 제목을 고정하고 예시에서 필요한 구분, 유지와 연결을 설명. 일반적인 복잡성이나 확장성 문구로 채우지 않음.
8. **대안 중립성:** 공통 기능과 개념만 표시. Component 배치, 모델 또는 코드의 생산 권한과 A/B 또는 A/C 결론은 후속 비교 페이지에 배치.
9. **핵심 하나:** 노란색 헤드라인에 사용자에게 발생할 핵심 위험, 그림에 사례, 하단에 구체적 구조 결정 배치. 반복 부제와 경쟁하는 강조를 줄이고 필요한 관계만 선택.
10. **정의보다 관계:** 용어가 하나의 사례에서 어떻게 연결되는지 그림으로 표현. Conversation은 입력을 담는 기록이며 발화가 변환되는 단계가 아님. Task와 Agent Execution은 목표 추적과 개별 실행으로 구별.
11. **공통 동작 우선:** 정상 사용자 상황으로 문제를 설명하고 필요한 경우에만 장애 사례 사용. 배경에서 별도 서비스 필요 등의 해결책을 선언하지 않음.
12. **이미지 자체의 설명력:** PNG만 보고 최초 상황, 핵심 문제, 모든 목표, 시간 및 정보 관계와 다음 구조 비교의 이유를 읽을 수 있는지 독립 리뷰. A/B 중 하나를 정답으로 암시하지 않는지와 잘림 및 겹침 확인.

## 다섯 구조 결정의 구분

| 문서 | 배경에서 드러낼 결정 |
| --- | --- |
| 41 | 필요한 정보 조회와 의미 완성의 진행 책임 |
| 42 | Conversation과 Task 상태의 관리 책임 및 실행 경계 |
| 43 | 연관된 의미 판단의 생산 책임과 조정 방식 |
| 44 | 겹친 사건의 후속 실행과 실제 전달을 이어가는 실행 구조 |
| 45 | 교차 기록 Context의 생산, 조회와 갱신 책임 |

41/43은 진행 제어와 의미 생산, 42/44는 상태 관리 경계와 사건 후속 실행 구조로 구별한다. 특정 구조를 선택하거나 장점을 입증하는 배경이 아니다.

## 각 페이지의 예시와 해석 범위

- **41:** 표 수치와 이전 Request는 설명용 가상 화면. 두 보고서는 같은 수준의 Task 후보이며 수정할 Task를 미리 정답으로 지정하지 않음.
- **42:** 한 Conversation의 처음 발화, 잠시 뒤 진행 확인, 보고서 완료 후 수정. 두 Task의 생성과 Delegation, 메일 결과 완료 및 보관, 보고서의 기존 실행 종료와 새 수정 실행을 같은 시간 열에서 표시. 확인된 상태 조회에는 새 실행이 없고 외부 Agent 접수 확인 후 Request 처리 완료와 Task 및 현재 실행의 지속 추적을 구별. 도형은 공통 데이터와 수명의 개념이며 서비스나 process 배치가 아님.
- **43:** 조건 지정과 잠시 뒤 메일 정정. 보고서의 PDF 유지, 메일 내용의 결론에서 표로의 교체와 결과 참조를 구분. 그림은 Referent, Task Association과 정정 범위를 중심으로 표시. 의도, 관계와 처리 방향은 사례와 고려 사항에서 유지. 전체 여섯 의미 항목은 Module, 모델 호출 또는 순차 단계가 아님.
- **44:** 최초 발화의 보고서 설명과 메일 초안 목표. 설명 도중 새 발화와 메일 Task 질문 도착. 즉시 음성 중단과 새 발화 해석, 실제 질문 전달 및 답변까지의 실행 대기를 구분. 표 설명 후 메일 질문 전달 순서는 예시이며 모든 사건의 고정 우선순위는 아님.
- **45:** 지난번 정정과 수정 전후 결과 및 실제 전달 기록을 이번 Request의 Context로 참조. 과거 기록 화살표는 사용자 발화가 아닌 Context에 연결. 문서 실루엣은 기록 종류의 개념 표현이며 실제 업무 결과가 아님. 현재 적용 의미와 장기 User Memory 등록을 자동 결정하지 않음.

## 재생성과 검증

원본 장면은 [generate_dp_background_slides.py](../../../scripts/architecture/generate_dp_background_slides.py)에 있다. 저장소 루트에서 다음 명령으로 개별 SVG/draw.io, 통합 draw.io 및 HTML을 재생성하거나 일치를 확인한다.

```sh
python3 scripts/architecture/generate_dp_background_slides.py
python3 scripts/architecture/generate_dp_background_slides.py --check
```

PNG는 같은 SVG를 1920×1080으로 렌더링한 결과다. 수정 후 해당 PNG도 함께 갱신한다.
