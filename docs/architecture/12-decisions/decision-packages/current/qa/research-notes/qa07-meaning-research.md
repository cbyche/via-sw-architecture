# QA07 업무 순서 및 연결 정확도의 벤치마크 근거

2026-10-23 / `QA07-ORDER-LINK-research-v4` / 공식 논문/리더보드 확인 / 작성자 목표 제안 / 미측정

[정의와 점수](../qa07-request-meaning-accuracy.md) / [평가 계획](../qa07-evaluation-plan.md) / [선정 입력](../qa07-evaluation-cases.md) / [이전 범위의 조사](../history/qa07-before-order-connection-20261023/qa07-meaning-research.md.txt)

## 1. 판단과 목표

**TaskBench는 업무 간 선후/의존 관계를 구조로 평가하는 방법, SMCalFlow는 이전 요청을 참조하거나 수정하는 연결을 구조로 평가하는 방법의 참고로 적합하다.** 다만 사용자 지정 업무 순서와 신규/기존 업무 ID만 묶어 평가하는 VIA와 완전히 같은 공개 지표는 이번 조사에서 확인하지 못했다.

대표값은 두 항목의 **전체 일치 정확도**로 정한다. 일부 관계만 맞힌 F1이나 최종 응답 만족도와 구별한다. 목표는 **85% 이상**으로 제안한다. 숫자 선택은 아래 공개 성능과 별도의 설계 판단을 연결하며, 논문이 VIA의 합격선이나 달성 가능성을 검증했다고 표현하지 않는다.

## 2. TaskBench: 선후 관계의 수치 참조

Shen 외, *TaskBench: Benchmarking Large Language Models for Task Automation*, NeurIPS 2024 Datasets and Benchmarks Track. 도구를 노드, 의존 관계를 간선으로 표현하며 전체 구조 일치를 평가한다. Table 10의 GPT-4 보고값은 **전체 graph accuracy 67.25%**, **2개 도구 84.53%**, **3개 도구 60.39%**다. 같은 표의 전체 node set accuracy 73.52%와 edge set accuracy 67.55%를 graph accuracy로 혼동하지 않는다. [공식 논문 Table 10, PDF 19쪽](https://proceedings.neurips.cc/paper_files/paper/2024/file/085185ea97db31ae6dcac7497616fd3e-Paper-Datasets_and_Benchmarks_Track.pdf).

공식 저장소는 입력과 도구 목록에서 그래프를 생성하는 prompt와 평가 코드를 공개한다. 논문의 Table 3 및 저장소 leaderboard가 강조하는 n-F1/e-F1은 일부 일치 점수이므로 QA07의 전체 일치 정확도 목표를 직접 계산하는 수치로 사용하지 않는다. [Microsoft 공식 TaskBench 코드와 평가 설명](https://github.com/microsoft/JARVIS/tree/main/taskbench).

TaskBench가 고르는 도구는 VIA가 Downstream Agent 내부에서 고를 도구와 다르다. VIA에는 **사용자가 요구한 업무 간 관계를 정답 구조와 비교하는 방법**을 참고한다. GPT-4 수치는 논문 당시의 보고값이며 지금의 최신 최고 성능이나 동일 API snapshot의 재현 결과가 아니다.

## 3. SMCalFlow: 기존 업무 연결의 참고

SMCalFlow는 이전 대화를 참조하는 `refer`와 이전 계산을 수정하는 `revise`를 포함한 대화 의미 구조를 다룬다. 공식 리더보드의 Value-Agnostic Conversational Semantic Parsing 단일 모델은 test accuracy **73.751%**, Seq2Seq 단일 모델은 **66.783%**다. 참조 판단을 제외한 별도 열의 75.269%를 사용하지 않는다. 이 모델들은 당시 대화 의미 파서이며 GPT-4 또는 최신 범용 LLM 결과로 소개하지 않는다. [공식 데이터셋과 리더보드](https://microsoft.github.io/task_oriented_dialogue_as_dataflow_synthesis/).

정확도는 값과 API를 포함한 전체 의미 프로그램의 평가이며 신규/기존 업무 연결만의 정확도는 아니다. 따라서 73.751%를 VIA의 연결 정확도로 환산하거나 TaskBench의 84.53%와 평균/곱하여 목표를 계산하지 않는다. 기존 요청과 새 요청을 명시적으로 연결하여 정답과 비교하는 설계 및 대화 구조 전체 일치의 성능 수준을 참고한다. [데이터셋 원 논문, TACL 2020](https://aclanthology.org/2020.tacl-1.36/), [모델 논문, ACL 2021](https://aclanthology.org/2021.acl-long.284/).

## 4. 최근 LLM의 신규 주제/후속 요청 구별 연구

*Beyond Continuity: Challenges of Context Switching in Multi-Turn Dialogue with LLMs*는 PIVOT/REFINE과 관련 과거 문맥 선택을 평가한다. Table 1의 TopiOCQA V1에서 GPT-4o의 PIVOT F1은 0.695, Claude 3.7 Sonnet은 0.941이다. 기존 문맥과 새 요청을 구분하는 문제가 별도로 평가된다는 점을 확인할 수 있다. [저자 공개 논문, 2026, Table 1](https://arxiv.org/html/2605.09268v1).

이는 F1이며 여러 기존 Task 중 정확한 Task ID 연결과 업무 선후 관계를 함께 평가하지 않는다. 따라서 94.1%를 QA07 전체 일치율로 바꾸지 않는다. 이 연구를 널리 확립된 VIA benchmark나 지금의 최고 성능 표로 소개하지 않는다.

## 5. 85%를 선택한 이유와 주장 범위

1. 선후 관계를 평가할 수 있는 최소 복합 구조는 두 업무다. TaskBench의 2개 도구 구조 전체 일치 84.53%를 이해하기 쉬운 공개 성능 참조로 선택한다.
2. `ceil(84.53) = 85`로 정수 목표를 제안한다. VIA에서는 이 수치를 2업무 사례에만 적용하지 않고 사전 등록한 업무 연결/순서 평가 입력 전체에 적용한다.
3. TaskBench 전체 67.25%와 3개 도구 60.39%도 함께 공개한다. 쉬운 구간의 수치만 보여주어 일반적인 LLM 성능이 85%라고 설명하지 않는다. 더 복잡한 입력에도 85%를 요구하는 부분은 VIA의 개발 목표 선택이다.
4. SMCalFlow 수치는 기존 요청 연결을 포함한 별도 대화 구조 평가의 참조다. 두 벤치마크의 평균이나 F1을 VIA 전체 일치 정확도로 환산하지 않는다.
5. **같은 숫자를 참조한 목표와 같은 시험에서 이긴 결과는 다르다.** VIA 85% 달성만으로 GPT-4 대비 0.47%p 개선이라고 주장하지 않는다. 실제 개선 주장에는 동일 입력/정답/채점기의 기준 모델 또는 harness 비교가 필요하다.

85%는 사용자가 새 정의와 함께 최종 승인한 수치가 아니라 이번 조사에 근거한 구체 제안이다. 이전 QA07의 90%는 넓은 대화/응답 범위의 제안으로 보존하고 자동 유지하지 않는다. 이번에는 Sensors의 대화 성공률이나 TreeDST의 의미 파싱 수치를 목표 근거로 사용하지 않는다.

## 6. 측정 조건의 출처

| 항목 | 이번 결정 | 근거 |
| --- | --- | --- |
| 대표 채점 | 입력별 업무 순서와 신규/기존 연결 전체 일치 | 사용자가 확정한 두 항목. TaskBench의 구조 일치와 SMCalFlow의 대화 의미 구조 평가를 참고 |
| 입력 | COMMON-28의 업무 생성/후속 요청 28개 | 기존 공통 원전의 84입력 중 선정. 포함/제외 원장 제공. 32개 별도 응답 사례는 사용하지 않음 |
| 문맥 보완 | S17-U03의 참조 검색은 진행 중이고 결과 미도착 | 모호한 원문을 임의 해석하지 않기 위한 작성자 fixture 보완. 다른 QA에도 같은 파생 조건 적용 필요 |
| 반복 | 입력당 5회, 총 140회 | 출력 변동 확인을 위한 작성자 시험 규칙. 5회가 논문에서 도출된 필수 표본 수는 아님 |
| 관측 경계 | 입력 제공부터 최초 관계 채택까지. 무판정 한도 60초 | 응답 품질/Agent 업무 완료를 제외하는 새 범위와 유한한 누락 판정 |
| 목표 성공 수 | 119/140 이상 | 140 × 0.85 = 119 |
| 점수 | 85%~100%를 3%p 균등 분할 | 기존 QA 점수 설계 원칙. 연구의 실증 임계값이 아님 |

선정 집합에는 업무 연결 사례가 많고 양의 선후 관계 사례는 2개다. 유형별 건수와 결과를 부록에서 반드시 드러낸다. 목표 수치의 문헌 근거가 시험 집합의 통계적 대표성까지 입증하는 것은 아니다. 긴 업무 그래프와 실제 사용 빈도 분포의 품질을 검증했다고 확대하지 않는다.
