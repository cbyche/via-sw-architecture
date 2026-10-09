# VIA DP 설계 비교 발표 틀 — 41~45

[전체 미리보기](./index.html) / [5페이지 draw.io 편집 원본](./VIA-DP-comparison-41-45.drawio)

[수정 가능한 설계 비교 5장 PPTX](./VIA-DP-comparison-41-45.pptx) / [본 발표 9장과 부록 2장 PPTX](../VIA-DP-background-and-comparison-41-45.pptx) / [PPTX 구성과 편집 방법](../DP-PPTX.md)

이 디렉터리의 비교 전용 PPTX·draw.io·웹은 41~45 번호순 **참조 자료**다. 실제 본 발표는 통합 PPTX의 도입 → 41 → 44 → 45 → 42 순서이며 43 두 장은 부록으로 보존한다. 설명 순서 변경은 개별 설계 내용이나 후보 자격의 변경이 아니다.

사용자가 제공한 DP01~DP04 방안 PNG의 **좌우 설계안, 구조, 장점, 단점, QA Trade-off** 행 구성을 따른 1920×1080 발표 자료다. 기존 배경과 제목 및 서체를 맞추고 설계안 행은 레퍼런스의 연녹색을 사용했다. 2026-10-06 보강판은 기존 40번대 MAIN 그림을 대조하여 판단 생산자, 상태 소유, 조회와 재판단, 확정 및 전달 경로를 복원했다. 43은 기존 A/C 표기를 유지한다. 모든 도형, 이름, 선, 수치와 원형 점수는 draw.io에서 편집 가능하다.

| DP | 비교 | QA 세 항목 | 편집 원본 | SVG | PNG |
| --- | --- | --- | --- | --- | --- |
| 41 | 모델 중심 ReAct / 모델 틀과 코드 완성 | V-03 기능 완전성, V-05 요청 완료 신속성, V-08 변경 용이성 | [draw.io](./dp41-comparison.drawio) | [SVG](./dp41-comparison.svg) | [PNG](./dp41-comparison.png) |
| 42 | 통합 Core / 독립 대화 및 업무 서비스 | V-05 요청 완료 신속성, V-08 변경 용이성, V-06 메모리 효율성 | [draw.io](./dp42-comparison.drawio) | [SVG](./dp42-comparison.svg) | [PNG](./dp42-comparison.png) |
| 43 | 통합 의미 생산 / 기능별 의미 생산과 코드 조정 | V-01 기능 정확성, V-05 요청 완료 신속성, V-08 변경 용이성 | [draw.io](./dp43-comparison.drawio) | [SVG](./dp43-comparison.svg) | [PNG](./dp43-comparison.png) |
| 44 | 중앙 비동기 조정 / 반응형 Dataflow | V-04 응답 신속성, V-08 변경 용이성, V-06 메모리 효율성 | [draw.io](./dp44-comparison.drawio) | [SVG](./dp44-comparison.svg) | [PNG](./dp44-comparison.png) |
| 45 | 원본 서비스 조합 / 공통 파생 기억 생산과 조회 | 조건별 정확성, 반복/첫 조회, 전체 모델 비용, 변경 용이성 | [draw.io](./dp45-comparison.drawio) | [SVG](./dp45-comparison.svg) | [PNG](./dp45-comparison.png) |

## 구조 보강 내용과 원문

| DP | 보강한 설명 경로 | 원래 설계 |
| --- | --- | --- |
| 41 | 모델의 읽기 선택과 재판단 대 코드의 조회 일정과 관계 결합. 임시 해석 상태, 동일 정보 공급, 현재 권한 및 버전 검사와 State Store 채택 | [41 본문](../../architecture/12-decisions/decision-packages/04-41-request-resolution-control.md) / [MAIN](../../architecture/12-decisions/decision-packages/diagrams/choice41-structure.svg) |
| 42 | Conversation, 세 Request, 두 Task와 실행 및 질문의 서로 다른 수명. 동일 Core의 공동 로컬 확정 대 독립 서비스의 명령 접수 및 대화 반영. 내부 접수와 외부 Agent 접수의 구별 | [42 본문](../../architecture/12-decisions/decision-packages/04-42-lifecycle-ownership.md) / [MAIN](../../architecture/12-decisions/decision-packages/diagrams/choice42-structure.svg) |
| 43 | F1~F6의 같은 의미 생산 의무. 통합 생산 대 여섯 기능별 생산자와 코드 조정. 제안 및 원생산자 재판단, 후보 정보 읽기, 생산자별 공유 모델 호출, 공통 현재 검사와 채택 | [43 본문](../../architecture/12-decisions/decision-packages/04-43-request-interpretation.md) / [MAIN](../../architecture/12-decisions/decision-packages/diagrams/choice43-structure.svg) |
| 44 | 중앙 반환 회수 및 재배정 대 typed 사건의 단계별 활성화와 window 및 credit/cancel. 양안 Direct 후보 통과, 후보 준비와 게시 Control, 실제 표시 및 후행 receipt | [44 본문](../../architecture/12-decisions/decision-packages/04-44-continuous-interaction.md) / [MAIN](../../architecture/12-decisions/decision-packages/diagrams/choice44-structure.svg) |
| 45 | 원본별 읽기와 요청 임시 조합 대 공통 과거 관계의 생산 및 지속 게시와 조회. 원본 owner, cache, 출처 및 coverage, 현재 버전과 권한 재확인, 미게시 범위 및 오류 재검증, 별도의 명시 User Memory | [45 본문](../../architecture/12-decisions/decision-packages/04-45-memory-and-context.md) / [MAIN](../../architecture/12-decisions/decision-packages/diagrams/choice45-structure.svg) |

각 칸은 완결된 별도 대안이다. 반복된 Component 이름은 같은 역할의 확대 표시이며 별도 인스턴스나 모델 복제를 뜻하지 않는다. 일반 실선은 전달 및 요청, 점선은 반환 또는 재판단이며, 43의 회색 bus는 생산자의 공유 모델 호출과 반환이다. 45는 공통 검정 실선 요청/전달과 점선 반환, 흰색 공통 및 살구색 차이 표기를 사용한다. Component와 내부 Module/상태를 구별하며 Model Access는 VIA Component, Shared Omni는 외부 모델이다. 그림의 숫자는 주요 설명 단계이며 전체 호출 순서가 아니다. 42의 배경은 보고서와 메일의 병행 사례, 구조 비교는 본문 E4의 보고서 질문 답변 확대 사례다. 후자의 R1/R2/R3는 최초 위임/질문 답변/결과 수정이며, 각 그림에 적은 사례 범위의 예시 ID다. 전체 실패 계약과 선택 조건은 링크한 본문을 따른다.

**41의 2026-10-08 수정:** 얇은 검정 선과 흰 바탕은 공통 구조, 살구색 바탕은 비교하는 설계 차이를 나타낸다. 왼쪽 Legend에서 직각 사각형은 논리적 책임 경계인 Component, 둥근 사각형은 그 내부 구현인 Module, 원통은 저장 상태로 구분한다. 포함 관계는 별도 프로세스나 서비스 배포를 뜻하지 않는다. B의 Request Interpreter와 Request Resolution Engine은 각각 틀의 의미 생산과 코드의 해석 진행 및 결합을 맡는 별도 논리 Component다. 양안의 Request Controller는 각 한 번만 표시한다.

41의 읽기 도구 실행기와 미해결 항목 처리기는 Context Manager와 Task Manager에 각각 직접 조회하고 응답을 받는다. B의 확인 후보와 정보 및 버전은 미해결 항목 처리기에서 관계 결합기로 전달하며, 미해결 및 충돌은 추가 조회로 되돌린다. 모델 호출 및 결과 반환, B의 부분 재해석, 임시 해석 상태의 보존과 갱신, 공통 Controller의 현재 정보 및 Task/질문 연결 재확인, 권한 검사와 채택 저장의 결과를 각각 표시한다. 오른쪽 외곽의 공통 검사선은 두 소유자에 분기하는 요청/응답 bus이며 해석 중 조회선과 구분한다. 입력 접수와 해석 요청, 의미 제안과 채택은 같은 Controller의 책임이다.

**42의 2026-10-08 수정:** 포괄적인 대화/업무 처리기 이름을 정식 Request Controller·Task Manager·Agent Gateway로 정리하고, Request Interpreter·Response Manager·Interaction Manager와 Model Access의 공통 협력을 표시한다. 별도 대화 처리 Component를 추가하지 않는다. Request Controller는 `R2/u2→Q1→T1` 관계와 실제 제시 P1 참조를, Response Manager는 P1/P2 publication·내구 전달 원장을, Task Manager/Agent Gateway는 Q1 변경·K1 전달·외부 확인을 소유한다. A의 관련 변경 공동 확정과 B의 접수 대기/업무 접수/대화 반영을 데이터 예시로 구분한다. `release(P2,epoch)`와 실제 전달 receipt는 별도 경계이며 최초 답변 transaction에 물리 재생을 포함하지 않는다. Voice Runtime은 Interaction Manager의 음성 모듈과 Model Access client가 실행되는 영역이며 새 의미 판단 Component가 아니다. 공통/차이 색과 얇은 선 및 Legend는 41과 같은 표기를 사용한다. 주요 Architecture MAIN과 비교 페이지는 같은 구조 scene을 재사용한다.

45의 2026-10-09 구조 보완: MAIN과 같은 [공유 scene](../../../scripts/architecture/memory_context_scene.py)에서 생성한다. A는 owner 읽기 조합과 직접 Bundle 반환, B는 변경 알림/본문 조회/생산/검증/게시/조회와 미게시 재조회 경로다. 45의 예상 수치와 원형 점수는 다섯 조건의 정성 비교로 교체했고 공식 Reference와 한계는 각주/발표 노트에 넣었다. 41~44의 표와 수치는 보존한다. 이번에는 45 비교 슬라이드만 갱신하며 `--dp45-only --comparison-only`는 기존 넓은 옵션을 유지하면서 배경을 제외한다.

## 수치와 원형 점수

사용자 지시에 따라 예상 수치와 점수를 채웠다. **형식을 검토하기 위한 작성자 가정이며 실측, 통계 추정 또는 선정 결과가 아니다.** 각 장에 예상 예시와 미측정을 표시했다. 2026-10-07 사용자 확인에 따라 V-04와 V-05는 품질속성 페이지와 같은 산술평균 기준으로 통일했다. 기존 시간 수치는 평균 시간의 형식 예시로 유지하며 p95 측정값을 평균으로 변환한 결과가 아니다. 장점은 해당 예상값에서 이기는 항목, 단점은 지는 항목을 원인 및 수치와 함께 한 줄로 설명하고 마지막에 QA 이름과 품질의 ↑/↓를 붙였다.

- QA 이름과 대표 지표는 2026-10-06의 [품질속성 발표자료](../quality-attributes.md)를 사용했다. V ID의 원래 범위는 [03-00 품질 시나리오](../../architecture/12-decisions/decision-packages/03-00-quality-scenarios.md)를 따른다. 발표 지표 및 목표를 새 Architecture 계약으로 채택한 것이 아니다.
- Trade-off의 ↑/↓는 **수치의 좋은 방향**이다. 장단점 괄호의 ↑/↓는 **품질의 상대적인 향상과 저하**다. 예를 들어 시간이 짧으면 요청 완료 신속성 ↑다.
- ●●●, ●●○, ●○○는 작성자의 상대적인 3단계 예시다. 같은 행에서 유리한 수치에 더 많은 점을 배정했다. 공식 score band, 목표 달성 판정, 가중 총점은 아니다.
- V-01은 정확 처리율, V-03은 기능 지원 달성률, V-04는 평균 반응시간, V-05는 평균 VIA 처리시간, V-08은 평균 변경 요소 수, V-06은 최대 전체 메모리다. V-05에서 외부 Agent 작업시간은 제외한다.
- 41의 지원율은 94개 변형 중 A 94개, B 82개라는 형식 가정이다. 실제 어떤 변형이 미지원인지 판정한 결과는 아니다. 시간 비교는 양안이 지원하는 동일 Request의 가정이며 전체 기능 모집단의 QA 통과를 뜻하지 않는다. 지원 손실은 별도 V-03 행에서 표시한다.
- 42의 변경 예시는 업무 저장 및 실행 수명 계약의 독립 변경이다. 일반 Agent adapter 변경도 B만 국소화할 수 있다고 주장하지 않는다.
- 43의 수치는 교차 의미 정정과 특정 지칭 기능 변경을 가정한다. C의 분해 자체가 정확성 개선을 보장하지 않는다.
- 44의 반응 지연은 집중 도착 사건의 유효 응답까지의 가정이다. 두 안의 로컬 음성 stop는 동일하며 B의 이익으로 계산하지 않는다. 같은 schema 안에서 단계 조합을 변경하는 경우를 가정한다.
- 45는 게시된 관계의 반복 재사용이 유효한 경우다. A의 index, cache 및 요약을 허용하고 B의 관계 추출 오류와 미게시 범위는 기존 본문에 남긴다. 항상 B가 정확하거나 빠르다는 뜻은 아니다. 2026-10-09 사례 교체의 검증값도 아니며 기존 형식 예시 값을 유지한다.
- 메모리 예상값은 동일한 공유 Omni 1벌, ASR, 역할별 KV와 공통 기능을 포함한 전체값이다. 추가 서비스나 관계 생산 및 buffer 비용만 조건에 따라 다르다. 실제 장치 메모리 계산이나 profiling 결과는 아니다.

## 41/42 공통 표기 — 2026-10-08

큰 Component 및 실행/서비스 경계는 흰색으로 유지한다. 42의 실행 및 확정 범위는 살구색 계열의 짙은 주황갈색 테두리와 A의 실선/B의 점선으로 구분한다. 공통 Component는 배치가 달라도 흰색과 검정 선을 쓰며 실제 책임이나 동작이 다른 작은 Component/Module은 살구색 채움과 같은 계열의 짙은 테두리를 쓰며, 경계선 두께도 공통 요소보다 높인다. 직각 Component, 둥근 내부 Module, 원통 저장 상태, 실선 요청/전달 및 점선 응답/반환의 Legend는 41/42가 같은 생성 함수를 사용한다. 41에는 별도 process 배치를 가정하지 않는다.

PNG 렌더러와 PPTX 생성기의 `--dp41-42-only` 옵션은 두 비교 슬라이드만 교체하고 나머지 슬라이드와 공통 package 부분을 보존한다. 범위 없는 재생성은 전체 자료를 만들 때 사용한다. 기존 예상 수치와 미선정/미측정 상태는 유지한다.

## 44 소속과 실행 연결 — 2026-10-08

A 중앙 조정 방식(중앙 비동기 Orchestration)과 B 이벤트 흐름을 연결하는 방식(반응형 Dataflow)은 같은 정식 10개 Component와 배치·원본·모델 조건을 사용한다. Request Controller 내부의 A Dispatcher/Progress State 또는 B Input Resolution Stage·Task Notice Stage/Window, Response Manager 내부의 공통 준비·게시·Outbox 및 B Publication Join/Window를 비교한다. 정식 Module·상태 이름과 포함 관계는 실제 그림에 표시한다. 실행 기반은 Component 박스가 아니다.

[44 MAIN](../../architecture/12-decisions/decision-packages/diagrams/choice44-structure.svg)과 비교는 [공유 scene](../../../scripts/architecture/continuous_interaction_scene.py)을 사용한다. 41/42와 같은 기호 의미에 외부 의존성 육각형을 추가하고, 다른 실행 상태도 살구색과 짙은 테두리로 표시한다. 큰 Component는 흰색/검정이다. 동일 원본에서 Task Notice의 Request Controller 경유, A 완료 후보 회수와 B 직접 소비, 실제 receipt→Outbox→질문 focus를 읽는다.

PNG/PPTX의 `--dp44-only`는 44 비교만 교체한다. PPTX에서는 해당 notes와 관계도 갱신하며 다른 슬라이드와 공통 package 부분을 보존한다. MAIN의 Voice Runtime·executor/worker pool·scheduler 및 실행 지원 주석은 변경 슬라이드의 발표자 노트에도 남긴다. 기존 수치와 점수는 형식 예시/미측정 상태를 유지한다.

## 편집과 재생성

통합 draw.io에서 페이지를 골라 수정하거나 개별 파일을 복제해 같은 틀을 재사용한다. 보강판은 구조 영역을 494px에서 552px로 늘렸으며 하단에는 장점과 단점 각각 80px, QA 95px를 배정했다. 레퍼런스의 진행 단계 ribbon과 FR 번호는 VIA 자료에 해당 값이 없어 복사하지 않았다.

생성 원본은 [generate_dp_comparison_slides.py](../../../scripts/architecture/generate_dp_comparison_slides.py)와 [dp_comparison_structures.py](../../../scripts/architecture/dp_comparison_structures.py)다. 전자의 `DATA`와 `Comparison`은 공통 행 틀 및 수치를, 후자의 `graph41`~`graph45`는 구조를 정의한다. 설명 label은 native 흰 배경과 함께 출력하여 선이 글자를 관통하지 않도록 했다.

```sh
python3 scripts/architecture/generate_dp_comparison_slides.py
node scripts/architecture/render_dp_comparison_slides.cjs
python3 scripts/architecture/generate_dp_comparison_slides.py --check
```

렌더러는 Playwright와 Chromium을 사용한다. 필요한 경우 `VIA_PLAYWRIGHT_MODULE`에 Playwright 모듈 경로, `VIA_CHROMIUM_PATH`에 브라우저 실행 파일을 지정한다. PNG 수정 없이 SVG만 바꾸면 해시 검사가 실패한다. draw.io 직접 편집 후에는 같은 PNG/SVG도 내보내고 생성 scene에도 변경을 반영한다.

[검수 기록](./REVIEW.md) / [예상값 데이터](./comparison-data.json) / [렌더 해시](./render-manifest.json)

## 45의 여러 활동 참조 사례 — 2026-10-09

양안은 VIA 직접 설명 E20의 실제 전달 P20, 제품 비교 T21/D21@v2와 견적 T22/D22@v1에서 현재 제안서 요청 R23의 근거를 제공한다. VIA는 발췌/참조 연결을 담당하고, 기준 적용과 제안서 작성은 Downstream Agent가 수행한다. 현재 T23 연결은 Request Interpreter의 제안과 Request Controller의 채택 결과다. 기존 Agent 실행 맥락 재사용, A의 작은 관계 index/cache와 병렬 읽기로 충분할 수 있다. B는 같은 원본에서 검증/게시한 과거 후보만 읽으며 R23의 정답 관계를 미리 제공하지 않는다.

PNG/PPTX `--dp45-only`는 45만 갱신한다. PPTX는 현재 통합 6/7번(이전 10장 구성의 9/10번), 번호순 비교 5번, 배경 5번 및 필요한 notes/관계 외 부분을 보존한다. 과거 관계의 생산 비용, 미게시/표현 미지원, 오류와 갱신/삭제 및 원본 확인 계약은 유지한다. 이번은 필요성과 사례 보완이며 최종 판단은 원래 리뷰에서 계속한다.
