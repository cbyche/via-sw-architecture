# DP01: 선택 조건과 추가 실험

## 무엇을 선택할 수 있는가

기존 BASIC 12 episode/안의 QA01은 A 6.609초, B 6.267초, C 11.995초다. A/B는 첫 호출 위치만 같고 유지 view·최근 응답 등 실제 입력이 다르므로 **전체 비교의 구조적 동점이 아니라 우열 미확정**이다. C는 게시 상태의 입력·완성 답변 동시 생성이라는 영향 경로가 있으므로 느린 값을 무관한 서비스 변동으로 지우지 않는다. C의 요청 이전 관계 생산 33호출·$0.0729348은 QA05에 포함하며 요청 QA01에 소급 합산하지 않는다.

A는 QA02 지칭 36/36, 호스트 종료 36/36, 기본안 내 실제 근거 재사용과 비용을 근거로 한 조건부 선택이다. 상세 본문은 A/B 각각 30/36으로, 호스트 종료를 필수 내용 완전 충족이라고 부를 수 없다. QA01의 명확한 승자를 확인했다는 결론은 아니다. C는 QA02 34/36과 높은 사전 생산 비용을 별도로 부담했다.

## E3 공통 첫 반환 정책의 민감도

실제 E3 ABC × 두 정책 × 3회, 총 18 episode다. 각 행의 고정 분모는 첫 안내·호스트 종료·지칭 각각 6개다. ORIGINAL은 기존 select-or-complete, ADOPTION_ONLY는 실제 의미 모델이 source/intent를 먼저 채택해 안내한 뒤 기존 경로를 수행한다. 모델 호출은 모두 실제 Qwen이며 새 응답 정책을 추가한 실험이다.

| 안·정책 | QA01 ms | QA03 ms | 호출 | 전체 모델 $ | 호스트 종료 | 본문 저자 충족 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| A ORIGINAL | 7,092.283 | 21,508.024 | 12 | 0.0146552 | 6/6 | 3/6 |
| A ADOPTION_ONLY | 1,905.169 | 23,117.772 | 18 | 0.0157364 | 6/6 | 3/6 |
| B ORIGINAL | 6,724.036 | 22,171.558 | 12 | 0.0150744 | 6/6 | 3/6 |
| B ADOPTION_ONLY | 1,828.871 | 23,421.999 | 18 | 0.0159652 | 6/6 | 3/6 |
| C ORIGINAL | 17,730.378 | 17,731.196 | 15 | 0.0348104 | 6/6 | 6/6 |
| C ADOPTION_ONLY | 2,322.050 | 20,349.887 | 21 | 0.0426752 | 6/6 | 6/6 |

각 안에서 episode당 실제 채택 호출 2개가 추가됐다. 빠른 안내와 상세 종료/비용 사이의 교환이다. E3의 A/B 첫 API 요청 값은 12쌍 모두 같았다. 이는 이 사례에서 작은 A/B 시간 차이를 구조의 우열로 해석하지 않을 근거이며, 다른 사례의 유지 view 효과까지 같다는 증거는 아니다. E3의 원본 정정은 A의 기존 view를 비우므로 A의 전체 후속질문 재사용 이익을 재측정하지 않았다. C의 게시물은 각 정책에서 새로 생산되어 길이도 달랐으므로 정책만의 고립 인과 추정도 아니다.

QA01은 비스트리밍 JSON 반환 뒤 실제 표시되는 안내까지다. ORIGINAL이 충분한 근거에서 완성 답을 함께 만들면 호출 수는 줄어도 첫 안내는 그 생성이 끝날 때까지 기다린다. ADOPTION_ONLY는 이 반환 정책을 공통으로 바꾼 민감도이며, 기존 기본 구조의 순수 효과나 BASIC 전체 승자를 확정하지 않는다.

## 실제 안내의 수준

[first-handling-examples.json](first-handling-examples.json)에 실제 표시와 채택 판본을 보존했다. A/B는 두 구체적 원문 이름 뒤 `summarize`를 표시했고 C는 `summarize_delivered_criteria_and_quote_excluding_installation_assumptions`를 표시했다. targets는 explained-criteria/vendor-quote, 각각 revision 1이다. 단순 빈 확인보다 구체적인 근거·처리 의도지만, 일반 동사와 내부 라벨이 노출된 제한된 UI다. 자연스러운 제품 안내나 실제 사실 답변으로 과장하지 않는다.

## 내용 판정은 별개다

| 구성 | literal marker | 기존 단위·날짜 동치 | 실제 본문 저자 대조 |
| --- | ---: | ---: | ---: |
| A ORIGINAL / ADOPTION_ONLY 각각 | 3/6 | 3/6 | 3/6 |
| B ORIGINAL / ADOPTION_ONLY 각각 | 3/6 | 3/6 | 3/6 |
| C ORIGINAL | 3/6 | 6/6 | 6/6 |
| C ADOPTION_ONLY | 2/6 | 6/6 | 6/6 |

A/B는 q1이 모두 충족됐으나 q2의 정정 납기 11월 1일이 실제 본문에서 모두 빠졌다. C의 영문 날짜/단위 표현은 기존 literal과 다르지만 원문과 뜻이 같았다. 앞선 안내를 빨리 했다는 사실로 상세 누락을 성공 처리하지 않는다. 미표시 초안은 성공 본문으로 세지 않았다. [content-review.json](content-review.json), [parallel-content-equivalence.json](parallel-content-equivalence.json)을 분리 보존한다.

## A3: 더 작게 보존하고 필요하면 다시 읽기

기존 14,000바이트 예산에서 마지막 단일 passage 초과를 허용했던 누락은 필수 hard-cap 수정이다. 그 수정을 tactic 성과로 세지 않는다. A3는 그 이후 선택적으로 8,192바이트 예산과 원본 locator를 쓰는 자원 교환이다. 직렬화 passage 합계 예산이며 전체 heap cap은 아니다.

- 자연 원문 48행의 native 12쌍: 보존 직렬화 8,555→146바이트, 후속 원문 읽기 0→12회. 반환 원문 100,776바이트, 실제 전체 파일 read_to_string 요청 174,828바이트가 추가됐다. native 최대 RSS는 1.966080→1.949696MB로 작은 차이다. 물리 디스크 I/O·VIA 전체 RAM·Qwen 품질 측정은 아니다.
- 기존 A 12 episode의 실제 응답 재생 12쌍: 작은 view에서 A3 미발동, read 39→39. 모든 모델 payload와 실제 완료 본문이 같았다. 최대 RSS 40.992768→40.976384MB, 평균 trial peak는 40.524459→40.559957MB로 소폭 악화했다. 새 LLM 지연/비용/정확도 개선을 주장하지 않는다.
- 큰 view에서 locator가 새 의미 호출을 유발할 수 있으나 이번 native 경계 검사는 그 모델 비용까지 재지 않았다.

따라서 A3 기본 적용은 보류한다. 상대 QA06 열세가 입증됐기 때문이 아니라 큰 보존 문맥의 절대 부담을 조절하는 선택지다. 기본안의 짧은 문맥에는 이득이 없었다. [replay-summary.json](replay-summary.json), [native/](native/)에 양방향 결과를 남겼다.

## 증거 경계

QA04의 구조 변경 용이성을 이번 정책/재생 실험에서 다시 측정하지 않았다. QA05는 실제 전체 모델 사용료 추정이며 청구서 금액이 아니다. RSS는 관측 프로세스 전체 동시 표본이며 작은 차이의 인과를 단정하지 않는다. 정확한 token·stage·실패·원문과 freeze는 [fresh-summary.json](fresh-summary.json), [first-call-policy-breakdown.json](first-call-policy-breakdown.json), [freeze.json](freeze.json), [README.md](README.md)에 연결한다. 기존 BASIC·A2의 고정 분모와 결과는 변경하지 않았다.
