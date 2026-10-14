# QA01 첫 응답 지연시간 조사 부록

> 조사 확인일 2026-10-14 / 상세 기록 보강 / [현재 QA01 명세](../qa01-first-response-latency.md)

## 1. 조사 질문과 채택한 논리

질문은 “사용자가 다음 생각과 행동을 이어가게 하려면 구체적인 첫 내용이 언제 보여야 하는가”였다. 첫 토큰이나 인사 및 Agent 업무 완료 시각을 섞지 않고 **화면에 첫 완결된 정답 내용이 보이는 지연**에 적합한 자료를 찾았다.

Nielsen의 사고 흐름 유지 약 1초와 Microsoft의 Windows 응답성 유지 구간 0.5~1초를 참조하여 대표 시험 최대 1초를 제안했다. 사용자와 CTO/담당임원의 빠른 대화 요구를 UI의 첫 내용 표시로 구체화한 제품 판단이다.

## 2. 출처별 확인 내용

| 출처와 확인 위치 | 확인한 내용 | 채택한 용도와 한계 |
| --- | --- | --- |
| [Jakob Nielsen, Response Times: The 3 Important Limits, 1993](https://www.nngroup.com/articles/response-times-3-important-limits/) / Usability Engineering 제5장 발췌 | 약 1초를 지연을 느끼면서도 사고 흐름을 유지할 수 있는 UI 응답 지침으로 설명 | 1초 목표의 주요 출발점. VIA 음성 대화에서 실험한 성공률이나 만족 보장으로 해석하지 않음 |
| [Microsoft, Improve the responsiveness of your Windows app](https://learn.microsoft.com/en-us/windows/apps/develop/performance/responsive) / 응답시간 분류 표 / 2026-03-18 갱신 | Pause 0.5~1초를 지연은 있지만 응답성이 유지되는 구간으로 구분. Fast는 100~200ms이고 Interactive는 300~500ms | VIA가 실행되는 Windows UI의 적용 근거. 최종 업무 전체를 1초에 끝내라는 기준이 아님 |
| [Ben Shneiderman, Response Time and Display Rate in Human Performance with Computers, ACM Computing Surveys 16(3), 1984, pp.265–285](https://www.cs.umd.edu/~ben/papers/Shneiderman1984Response.pdf) / p.266 | 입력 실행과 결과가 나타나기 시작하는 때를 연결하여 응답시간을 설명. 접수 신호와 실제 결과 표시를 구별 | 첫 내용 표시를 종료점으로 삼는 이유. VIA의 정확한 자연어 판정 규칙은 별도로 정의해야 함 |
| [Robert B. Miller, Response time in man-computer conversational transactions, 1968](https://doi.org/10.1145/1476589.1476628) | 응답 대상과 목적에 따라 적절한 시간이 달라지는 문제를 다룸 | 응답의 종류와 경계를 먼저 고정할 배경. 문헌의 추정 지침을 보편적인 실험 임계값이라고 인용하지 않음 |
| [Stivers 외, Universals and cultural variation in turn-taking in conversation, 2009](https://pubmed.ncbi.nlm.nih.gov/19553212/) | 10개 언어의 자연 대화에서 발화 겹침과 턴 사이 침묵을 줄이는 경향을 관찰 | 음성 턴 조절이 중요한 이유. 사람의 턴 간격을 화면 표시 지연의 합격선으로 직접 전용하지 않음 |
| [Maslych 외, Mitigating Response Delays in Free-Form Conversations with LLM-powered Intelligent Virtual Agents, CUI 2025](https://arxiv.org/html/2507.22352v1) | VR 음성 Agent에서 1.5초 / 4초 / 6.5초의 응답 시작 조건을 비교 | 음성 Agent 지연과 사용자 경험을 연결하는 보조 자료. PC 화면의 1초를 직접 실험한 결과는 아님 |

### 원문에서 놓치면 안 되는 조건

[Miller 원문 p.271](https://concreta.com.uy/wp-content/uploads/Miller1968.pdf)은 제시한 시간값을 저자의 계산된 추정으로 설명하고 실제 과제 환경의 검증이 필요하다고 명시한다. 따라서 Miller와 이를 참조한 UI 지침을 VIA 대상의 통제 실험 결과로 소개하지 않는다. Shneiderman의 결과 표시 시작 개념도 VIA에서 인정할 완결된 자연어 내용의 판정까지 정의해 주지는 않는다. 이 부분은 QA01의 정답 판정표가 담당한다.

Maslych 연구는 **참가자 54명의 VR 대화**이며 1초 조건이 없다. 자연스러운 지연 안내가 4초와 6.5초 조건에서 체감 응답시간 평가를 개선했지만 참여와 인상 및 재이용 의향 등 더 넓은 경험 개선은 유의하지 않았다. 안내의 효과와 실제 요청 내용의 전달은 구분해야 한다. 따라서 이 연구를 근거로 “네”나 기다림 표시를 QA01의 유효 응답에 포함하지 않는다. [연구 §5.1~5.3](https://arxiv.org/html/2507.22352v1)

### 다른 방향의 결과도 조사 기록에 남긴다

[The Impact of Response Latency and Task Type on Human-LLM Interaction and Perception, 2026 사전 공개본](https://arxiv.org/html/2604.06183v1)은 240명 규모의 생성과 조언 과제에서 2초 / 9초 / 20초 조건을 비교한다. 9초 응답이 2초보다 유용성 평가가 높았고 긴 대기가 더 신중한 응답으로 평가되는 결과도 보고한다. 이는 모든 LLM 과제의 모든 사용자 경험이 지연과 함께 단조롭게 나빠진다는 주장을 지지하지 않는다.

VIA의 현재 결론은 유지한다. **QA01은 요청을 이해했다는 구체적인 첫 내용 표시를 1초 안에 제공하는 제품 목표**이며 긴 최종 답변의 품질이나 전체 과제 만족도를 하나의 지연값으로 대표하지 않는다. 위 연구를 VIA의 9초 목표 근거로 옮기거나 기존 1초 제안을 실험으로 반증했다고 쓰지 않는다. 과제와 표시 방식 및 평가 대상이 다르기 때문이다.

## 3. 평균에서 최대값으로 바꾼 이유

외부 지침은 사용자가 경험하는 한 번의 응답 대기를 설명한다. 평균만 사용하면 일부 긴 대기가 빠른 사건으로 상쇄된다. 가상으로 29건이 0.2초이고 한 건이 20초이면 평균은 0.86초지만 사용자는 긴 대기를 한 번 경험했다. 따라서 고정 시험 90건의 최대값을 목표에 대조한다.

이 변경은 모든 미래 요청의 수학적인 최악 시간 보장을 뜻하지 않는다. 같은 사례와 반복 수를 유지하고 평균 및 분포를 진단으로 남긴다. 최대값으로 바뀌었으므로 평균용 bootstrap이나 이전 점수를 그대로 사용하지 않는다. 전체 비교 규칙은 현재 명세 §10을 따른다.

## 4. 발표 그림으로 보존할 예시

```text
0.0초  사용자의 실제 발화 종료
0.1초  “네” 표시: 종료점으로 인정하지 않음
0.8초  “매출표와 비용 그래프를 비교하겠습니다.” 표시
1.3초  Agent로 요청 전송
2.5초  앞선 발화가 끝나 음성 요약 재생
```

대상과 범위 및 수행 내용이 실제 채택된 상태라면 위임 전 안내도 유효하다. 이 사건은 0.8초이며 전송 1.3초나 음성 재생 2.5초가 종료점이 아니다. 가상 시험 네 건이 0.8 / 0.6 / 1.0 / 0.8초라면 최대 1.0초로 1점이다. 실제 측정값은 아니다.

## 5. 심사 답변과 피해야 할 표현

- **왜 1초인가:** “Nielsen의 사고 흐름 유지 지침과 Windows 응답성 유지 구간을 화면의 첫 내용 표시 목표로 적용했습니다.”
- **음성 Agent에 왜 UI 지침인가:** “이 지표의 종료점이 화면 표시이기 때문입니다. 실제 음성 전달은 QA09에서 따로 확인합니다.”
- **인사만 먼저 하면 되는가:** “요청을 반영한 첫 완결 내용이 맞아야 합니다. 준비 신호는 제외합니다.”
- 피해야 할 표현: “사람은 정확히 1초까지 대화가 자연스럽다고 느낀다”, “평균이 1초면 모든 요청을 만족한다”, “모델의 첫 토큰이 첫 응답이다”.

## 6. 발표와 후속 검증에 넘길 자료

슬라이드에는 정의와 1초 목표 및 위 타임라인을 크게 두고 Nielsen과 Microsoft를 각주로 단다. Miller와 Stivers 및 VR 연구는 질문이 나왔을 때 설명하는 보조 자료로 둔다. 0~6점 구간은 [현재 명세 §7](../qa01-first-response-latency.md#7-06점--qa-c01-score-v2)에서 가져온다.

첫 내용의 정답 판정표, 실제 UI 프레임과 response revision의 연결, source 시계의 오차, 실제 음성 전달 누락 여부는 PoC 준비에서 확인한다. DP45의 반복 조회 평균이 빨라도 최대값을 만드는 갱신 사건에서 이익이 있는지는 별도로 확인해야 한다.
