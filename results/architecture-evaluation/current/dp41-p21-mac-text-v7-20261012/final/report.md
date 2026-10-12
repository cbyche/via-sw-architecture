# DP41 A/B PoC 측정 결과

**상태: 측정·판정·집계 완료.** 같은 P21을 방안별 5회 실행했고, 독립 변경 실험 12개를 완료했다. 목표값 달성을 위해 후보 의미나 측정 결과를 보정하지 않았다.

A는 41A+44A+45A+42A, B는 41B+44A+45A+42A다. 실제 Qwen 의미 호출, transcript 입력과 공통 음성 출력 모형, Mac 실행을 사용했다. 아래 0–6점은 ASR-QA-v6 경계를 이 PoC의 해당 원지표에 적용한 값이며 보호 조건은 별도로 표시한다. 비용은 실제 의미 API usage×고정 가격, 메모리는 Mac 원지표다.

| QA | A | B | A / B 점수 |
| --- | ---: | ---: | ---: |
| 01 Request 처리 정확성 | 63.492 % | 56.984 % | 0 / 0 |
| 02 상호작용 응답시간 · 실패 cap | 6.420 초 | 6.144 초 | 0 / 0 |
| 03 VIA Request 처리시간 · 실패 cap | 23.534 초 | 25.639 초 | 0 / 0 |
| 04 VIA 의미 모델 사용 비용 | 0.132040 USD/P21 | 0.143066 USD/P21 | 6 / 6 |
| 05 변경 용이성 | 1.500 책임/과제 | 1.500 책임/과제 | 4 / 4 |
| 06 로컬 VIA phys_footprint peak | 1014.408 MiB | 648.532 MiB | 6 / 6 |

## 정확성 및 보호 조건

| 관측 | A | B |
| --- | ---: | ---: |
| 통과 조건 / 630 | 400 | 359 |
| 여섯 조건 모두 통과한 사례 / 105 | 34 | 29 |
| 필수 위반 관측 | 7 | 0 |
| qa01 보호/자격 충족 | 실패 | 실패 |
| qa02 보호/자격 충족 | 실패 | 실패 |
| qa03 보호/자격 충족 | 실패 | 실패 |
| qa04 보호/자격 충족 | 실패 | 실패 |
| qa05 보호/자격 충족 | 통과 | 통과 |
| qa06_mac_footprint_reference_budget 보호/자격 충족 | 통과 | 통과 |

| 정확성 보호 세부 | A | B |
| --- | --- | --- |
| target_98_percent | 실패 | 실패 |
| scenarios_95_percent | 실패 | 실패 |
| every_condition_95_percent | 실패 | 실패 |
| every_group_95_percent | 실패 | 실패 |
| mandatory_violation_zero | 실패 | 통과 |

낮은 비용이나 짧은 실패 응답만으로 우수 방안을 선정하지 않는다. 정확성·응답·완료의 보호 조건과 각 원지표를 함께 읽으며, 가중 합산 점수는 만들지 않는다.

## QA02의 세 평균

| 방법 | A 평균 · 관측 수 | B 평균 · 관측 수 |
| --- | ---: | ---: |
| 오답/미전달을 5초로 반영 | 6.420초 · 135 | 6.144초 · 135 |
| 정답 첫 반응만 | 7.282초 · 84 | 6.907초 · 81 |
| 오답 포함 실제 첫 핵심 반응 | 5.663초 · 128 | 5.034초 · 133 |

최초 오답 A/B=43/52, 이후 유효 정정=1/4, 유효 반응 누락=50/50. 실제 시각이 없는 반응에 실측시간을 만들어 넣지 않았다. 유효한 5초 초과 반응은 실제 시간을 유지한다.

| 반응시간 보호 관측 | A | B |
| --- | ---: | ---: |
| 일반 2초 이내 / 전체 · 95% 필요 | 11 / 105 | 14 / 105 |
| 집중 도착 5초 이내 / 전체 · 모두 필요 | 12 / 30 | 14 / 30 |
| 로컬 중단 0.1초 이내 / 전체 · 모두 필요 | 10 / 10 | 10 / 10 |
| 잘못된 첫 반응 또는 유효 반응 누락 | 51 | 54 |

## QA03의 두 평균

| 방법 | A 평균 · 관측 수 | B 평균 · 관측 수 |
| --- | ---: | ---: |
| 30초 내 미완료를 30초로 반영 | 23.534초 · 95 | 25.639초 · 95 |
| 실제 완료한 사이클만 · 늦은 완료 포함 | 18.328초 · 46 | 20.535초 · 35 |

완료 A/B=46/35, 30초 초과 완료=10/12, 관측 종료까지 미완료=49/60.

15초 이내 완료 보호 관측: A 23/95, B 16/95. 목표 평균 10초, 95%가 15초 이내, 늦은 완료·누락 0을 각각 검사했다.

핵심 구절은 judge가 실제 표시·모사 음성에 있는 정확한 구절로 판정하고, 그 첫 문자의 기록된 출력 시점/DAG node를 사용한다. 인사·filler 시작을 핵심 onset으로 대신하지 않는다. VIA 시간은 동일 관측 DAG에서 외부 Agent 작업·사용자 대기·사용자 발화 기간만 0으로 재계산하며, 병렬 join/queue/모델/출력 시간은 남긴다. 원 wall-clock과 계산 경로는 CSV/JSONL에 함께 보존했다.

## QA04 호출과 비용

| 실제 VIA 원장 · 다섯 활동 합 | A | B |
| --- | ---: | ---: |
| 실제 API attempt 수 | 423 | 401 |
| CINTERPRET | 226 | 220 |
| CRESPONSE | 197 | 181 |
| 입력 token | 1385047 | 1353670 |
| 출력 token | 62580 | 92313 |
| 재시도 attempt | 0 | 0 |
| 폐기된 반환 | 1 | 1 |

Qwen: `qwen3.6-35b-a3b`, Singapore/International, 입력 $0.375/M · 출력 $2.25/M. [고정한 Alibaba 가격 근거](https://www.alibabacloud.com/help/en/model-studio/model-pricing). 모델을 한 번만 부르도록 B를 제한하지 않았으며 실제 재조회·부분 판단·확인 질문·폐기를 비용에 포함했다.

OpenAI judge: `gpt-6-luna`, high reasoning. 판정 호출은 VIA 비용에서 분리했다. [모델·가격 근거](https://developers.openai.com/api/docs/models/gpt-6-luna). 본 캠페인 원장 사용량 환산 합계는 Qwen $1.375528, OpenAI $3.407008다. 무료 한도/충전 잔액과 구분되는 사용량 환산액이며 음성 API 청구는 없다.

## QA05 독립 변경 실험

| 변경 과제 | A 책임 수 | B 책임 수 | A / B 변경 책임 |
| --- | ---: | ---: | --- |
| M01 의미 provider wire 교체 | 1 | 1 | U15 / U15 |
| M02 Agent 명령/receipt wire 교체 | 1 | 1 | U16 / U16 |
| M03 Calendar raw 필드/시간대 교체 | 1 | 1 | U09 / U09 |
| M04 실제 producer 실패 뒤 draft 분기 | 4 | 4 | U01, U03, U07, U08 / U01, U03, U07, U08 |
| M05 Task owner 답변 scope token | 2 | 2 | U04, U07 / U04, U07 |
| M06 동의 철회·파생·늦은 반환 use fence | 0 | 0 | 변경 없음 / 변경 없음 |

과제별 최대 변경 책임 수: A 4, B 4. 평균 3 이하와 모든 과제 5 이하의 보호 조건을 함께 검사했다.

모든 과제는 같은 고정 원본을 독립 복사해 실행했다. 파일 수 대신 U01~U18의 동일 기능 책임을 세었고, 실제 diff/상위 port 보존/새 기능의 실행 probe/P21의 기록된 의미 재생/두 독립 코드 리뷰로 확정했다. 기존 의미 실패는 회귀 재생에서 그대로 보존한다. M06의 0은 전체 요구를 원본이 충족하는 실행 증거가 있을 때만 인정한다. 새 변경 기능의 실제 Qwen 검증은 별도 원장에 남기며 P21 비용/정확성에 섞지 않는다.

## QA06 Mac 메모리

| 원지표 | A | B |
| --- | ---: | ---: |
| phys_footprint · 다섯 활동 전체 동시합 최대 | 1014.408MiB | 648.532MiB |
| resident size/RSS · 다섯 활동 전체 동시합 최대 | 1028.219MiB | 665.359MiB |
| phys_footprint · 활동별 peak 평균 | 737.776MiB | 640.123MiB |

| Mac 추가 진단 · 주 footprint 점수와 별도 | A | B |
| --- | --- | --- |
| rss_peak_within_6gib_reference | 통과 | 통과 |
| idle_footprint_increase_all_within_16mib_reference | 통과 | 통과 |
| OS 측정 누락 sample | 0 | 0 |

50ms task_info로 실제 격리 worker를 관측했다. 한 프로세스의 후보·공통 owner/Agent stub·raw recorder를 포함하며 controller와 judge는 제외한다. 20MiB 원문/2,000turn/200 완료 Task/30 source 슬롯과 서로 다른 실제 RGBA 버퍼 15개(311,040,000byte)를 할당했다. 공유 객체는 한 번, 실제 복사본은 OS 사용량 그대로 계산한다. phys_footprint는 프로세스에 귀속되는 물리 메모리 회계, RSS는 현재 resident 페이지다. 공유 페이지를 포함하는 RSS 합을 유일한 물리 RAM으로 해석하지 않는다. 실제 환경/두 지표/기존 MiB 비교 점수 모두 Mac 기준이다.

## 다섯 쌍의 차이

| QA | B−A 평균 차이 | 쌍별 최소 / 최대 | 계약의 실용 차이와 일관성 |
| --- | ---: | ---: | --- |
| qa01 | -6.508 %p | -19.048 / 3.968 | 미충족 |
| qa02 | -0.276 초 | -0.838 / 0.224 | 미충족 |
| qa03 | 2.105 초 | -0.423 / 5.610 | 미충족 |
| qa04 | 0.011027 USD/P21 | -0.011 / 0.044 | 미충족 |
| qa06_phys_footprint_mib | -97.653 MiB | -387.516 / -0.047 | 미충족 |
| qa06_resident_size_rss_mib | -98.456 MiB | -385.328 / -0.062 | 미충족 |
| qa05 | 0.000 책임/과제 | 독립 변경 6과제 | 미충족 |

차이가 작거나 다섯 쌍에서 방향이 달라지는 축은 동점/변동으로 읽는다. 이 값은 다섯 번 관측과 여섯 변경 과제의 결과이며 통계적 유의성이나 전체 VIA의 A/B 우월성을 뜻하지 않는다.

## 실제 해석·해결 경로

| 다섯 활동 합 | A | B |
| --- | ---: | ---: |
| owner 조회 반환 | 134 | 184 |
| 조회 item의 cache hit | 29 | 137 |
| 확인 질문 요청 | 13 | 46 |
| 출력 계약 거절 | 58 | 43 |
| 해결 실패 사건 | 3 | 30 |
| 늦은 의미 채택 폐기 | 5 | 5 |
| 의미 반환의 policy 폐기 | 0 | 0 |

A의 논리 모델 호출 목적: `{"A_NEXT_ACTION": 226}`. A의 실제 선택: `{"READ": 68, "PROPOSE": 144, "CLARIFY": 13}`.
B의 논리 모델 호출 목적: `{"B_FRAME": 183, "B_PARTIAL_SELECT": 37}`. 항목별 상태: `{"UNRESOLVED": 256, "READ_PENDING": 200, "BOUND": 145, "PARTIAL_PENDING": 42, "AMBIGUOUS": 34, "NO_MATCH": 46, "DENIED": 6, "NO_VALID_CANDIDATE": 4}`.

논리 호출 요청 수와 API attempt 수는 재시도·예산 거절에 따라 다를 수 있다. provider의 취소 폐기, 반환 후 의미 채택 폐기는 별도로 기록했다. B의 부분 모델 판단과 갱신 Frame도 실제 경로에 포함한다. [Request별 조회·선택·해결·거절 경로](./request-paths.jsonl)와 [경로 합계](./mechanism-summary.json)를 호출 원장에 연결했다.

## 사례별 결과

| 사례 | A 통과 조건 / 30 | B 통과 조건 / 30 |
| --- | ---: | ---: |
| S1-01 | 24 | 30 |
| S1-02 | 28 | 28 |
| S2-01 | 18 | 0 |
| S2-02 | 13 | 4 |
| S2-03 | 15 | 14 |
| S2-04 | 20 | 12 |
| S2-05 | 6 | 0 |
| S3-01 | 30 | 30 |
| S3-02 | 28 | 29 |
| S3-03 | 17 | 22 |
| S4-01 | 6 | 27 |
| S4-02 | 10 | 5 |
| S4-03 | 15 | 25 |
| S5-01 | 25 | 25 |
| S5-02 | 30 | 30 |
| S5-03 | 27 | 23 |
| S5-04 | 22 | 26 |
| S6-01 | 20 | 5 |
| S6-02 | 2 | 2 |
| S6-03 | 30 | 10 |
| S6-04 | 14 | 12 |

오류 원인 검토는 실제 모델 선택/Frame 해결 상태/거절 사유/수신 payload/최종 전달을 연결해서 한다. 특정 구현의 validator 또는 모델 오류를 구조의 불가능성으로 확대하지 않는다. 유한 계약의 미지원 때문에 필요한 의미를 표현할 수 없는 경우와, 표현 가능한 의미를 모델/코드가 잘못 만든 경우를 구분한다. 상세 조건 판정은 conditions.csv, 후보의 실제 실패/조회/호출 경로는 원장에 보존했다.

## 고정 계약과 원시 데이터

- 계약: `DP41-P21-MAC-TEXT-v7`, `ASR-QA-v6`, source SHA-256 `44f725057e7589c35aa59cb23537a9a8c59f48a40944b70f03e3906e931f08ce`.
- 순서: A/B, B/A, A/B, B/A, A/B. 각 활동의 owner/cache/queue를 독립 초기화하고 timing worker를 순차 실행했다.
- 판정: 보정 사례 16개 → 블라인드 두 prompt pass → 불일치 시 세 번째 trace audit. 같은 judge 모델의 반복을 독립 사람 판정으로 표현하지 않는다.
- [전체 수치와 보호 조건](./qa-results.json), [완료 검수](./completion.json), [조건별 판정](./conditions.csv).
- [반응 raw/세 평균](./reactions.csv), [사이클 raw/두 평균](./cycles.csv), [활동별 결과](./repeat-results.csv).
- [호출 원장 CSV](./model-calls.csv), [변경 책임 CSV](./changes.csv), [인과 시간 재계산](./causal-attribution.jsonl).
- `../workers/`: 원문 gzip, 입력/원천 사건, Context/응답, token/cache/retry/discard, payload/effect, 실제 표시/모사 문자 출력/전달, OS 메모리 JSONL.
- `../changes/`: 동일 baseline/각 변경 source/diff/실행·회귀/두 책임 리뷰. `../evaluation/`: 보정·각 판정·최종 adjudication 원장.
- 준비·중단 기록은 별도 폴더에 보존하고 이 본 측정 모집단/점수에 포함하지 않았다.

## 실행 기록 교차 검수와 오류 해석

원래 OpenAI 판정은 그대로 보존했다. 별도 코드·실행 기록 검수에서 226개 제안의 명시적 disposition을 기록했고, 213개 제안을 120개 사례에 반영했다. 검수자는 LLM sub-agent와 root이며 사람 평가로 표현하지 않는다. 근거 ID와 실제 출력의 정확한 구절을 같은 validator로 다시 검사했다. [root 결정](../reviews/root-adjudication-decisions.json), [검수 완료](../reviews/root-adjudication-completion.json), [별도 최종 override](../evaluation/trace-adjudication-overrides.jsonl).

원래 평가기의 publication 조회가 중단 영수증의 없는 publication_id를 직접 읽어 KeyError를 내는 결함을 실제 S4-02/S5-04에서 재현했다. 후보 53개 frozen 파일과 원래 평가기는 보존하고, 평가기 사본의 조회 두 곳만 `.get`으로 고친 뒤 누락 판정을 이어서 생성했다. 프롬프트·정답·경계·모델은 동일하며 8개 API 없는 회귀 검사는 정확한 발화 구절의 검증과 가짜 ID/미발화 구절/중단 publication의 거부를 확인했다. 이 오류와 판정의 재시도 비용은 후보 품질 실패에 넣지 않았다. [평가기 수정 manifest](../evaluation-tool-manifest-v2.json), [회귀 검사](../evaluation-control/validator-repair-tests.json), [진행 기록](../evaluation-control/stages.jsonl).

이후 schema 재시도에서 source/Task/입력/인과 node의 실제 값 ID를 사건 evidence ID로 반복 인용하는 문제를 확인했다. V3는 첫 판정 프롬프트와 입력, 의미 판정 기준을 유지하고 재시도 안내에 유효하지 않은 인용과 그것을 포함하는 실제 제공 사건 ID를 명시했다. 코드가 의미의 PASS/FAIL이나 시간 endpoint를 정답으로 바꾸지는 않는다. 원래 출력과 모든 재시도 비용은 보존했다. [V3 manifest](../evaluation-tool-manifest-v3.json), [API 없는 참조 검증](../evaluation-control/evidence-feedback-repair-tests.json), [판정 원장별 provenance](../evaluation/evaluation-origin-manifest.json).

남은 publication UUID/구절 오류에는 V4가 잘못된 필드의 위치, 실제 publication ID와 해당 실제 발화 문자열을 재시도 안내에 추가했다. 최초 SYSTEM/입력·기준·validator는 V3와 같고 코드가 의미 판정 또는 endpoint를 자동 바꾸지 않았다. 후보 재실행은 없다. [V4 manifest](../evaluation-tool-manifest-v4.json), [실제 구절 참조 회귀 검수](../evaluation-control/exact-emission-feedback-repair-tests.json).

실패 여부는 실제 채택된 의미와 수행·전달에서 판정한다. 거절된 올바른 제안만으로 C1을 통과시키지 않고, 전달 문장의 오류를 발생하지 않은 외부 효과로 만들지 않는다. 기존 완료 Task의 이력을 현재의 신규 명령으로 세지 않는다. C2의 근거 오류와 C5의 실제 효과, C6의 checkpoint와 cycle의 최종 전달은 각각 해당 필수 사실을 확인한다. 정상적인 접수 뒤 완료는 모순이 아니다.

QA02의 `wrong_first`는 잘못된 목표·payload·상태, 불필요한 확인 질문, 또는 같은 목표를 종결하는 답변의 필수 사실 누락을 뜻한다. 실제 올바른 목표의 정상 단계별 진행·접수는 다른 단계가 남았다는 이유만으로 오답으로 표시하지 않는다. 이때도 지정된 유효 핵심 반응이 없으면 `valid=null`, cap=5초이며 정답 첫반응 평균에서는 제외한다. 오답 뒤 정정은 최초 오답의 실측시간과 후기 정정시간을 별도로 보존한다.

| 실제 오류 경로 | 관측과 해석 |
| --- | --- |
| A의 의미 출력 계약 | 선언하지 않은 meaning version, goal kind/operation, 잘못된 binding ID와 관계로 거절·수리가 발생했다. 모델이 선택·재해석하는 실제 반복 비용이며 A 구조가 그 목표를 표현할 수 없다는 증거는 아니다. |
| B의 Frame 계약·해결 | input_ref 불일치, 미선언 preservation/question reference, 잘못된 goal/reference 필드와 예산 소진이 발생했다. 유한 코드 해결의 표현·검증 비용과 모델의 계약 준수 문제를 구분한다. |
| 공통 처리·응답 경계 | 실제 접수만 말하고 필요한 Task/범위/보존 사실을 생략한 안내, 유효 질문의 잘못된 target으로 인한 거절은 실제 품질 실패다. 양안에 공통인 처리·응답 구현의 한계를 DP41 고유 구조의 한계로 확대하지 않는다. |
| 내부 거절과 실제 금지 효과 | 내부 잘못된 후보가 차단된 것과 실제 잘못된 새 Task/Execution에 명령이 접수된 것을 구분한다. 필수 위반에는 실제 효과·연결의 증거만 사용한다. |

[실패 사건 분류와 예시 근거](./failure-events.csv)는 사건 횟수다. 하나의 실패가 여러 거절/예산 사건을 만들 수 있으므로 이 합을 정확성 실패 조건 수로 사용하지 않는다. 정확성의 실제 실패 수는 conditions.csv에서 산출했다.

## QA05의 코드 변경과 실제 새 문장 처리

QA05의 책임 수는 같은 기능 계약을 수용한 실제 코드 diff, 지정 출력으로 수행한 새 분기·receipt·복원 검사, P21의 실제 기록된 의미 재생과 두 책임 검수에서 산출했다. 재생은 기존 성공·실패를 유지하는 회귀 검사이며 실제 모델의 새 의미 정확성을 검증한 것으로 합치지 않는다.

M04는 코드가 선행의 실제 실패 뒤 초안을 한 번 실행하고 발송을 막는지를 검증해 양안 모두 4개 책임을 셌다. 추가로 새 자연어 입력을 실제 Qwen으로 실행한 결과는 **양안 모두 FAIL**이다. A는 producer의 실제 실행 상태를 잘못 참조하고 draft를 보존 대상으로 묶어 초안을 실행하지 못했다. B는 관계 방향/producer 참조를 잘못 생산해 Frame 거절 뒤 의미 해결을 끝내지 못했다. 이 실패를 반복해 성공 결과만 고르거나 코드 점수에 가려서 제시하지 않았다.

| M04 증거 수준 | A | B |
| --- | --- | --- |
| 코드·실행 계약 probe와 책임 수 | PASS · 4개 | PASS · 4개 |
| 실제 새 문장의 Qwen 실행 | FAIL · $0.002810 | FAIL · $0.008040 |
| 실제 원문 변경/메일 발송 | 0 / 0 | 0 / 0 |

따라서 QA05의 수치 비교는 실제 코드 변경 전파 범위의 동점이다. 새 기능을 자연어부터 안정적으로 사용할 수 있다는 판정은 양안 모두 실패다. 낮은 변경 책임 수를 제품 준비 완료나 의미 품질의 통과로 읽지 않는다. [A 실제 신규 실행](../changes-semantic/M04-A-attempt1/change-semantic-validation.jsonl), [B 실제 신규 실행](../changes-semantic/M04-B-attempt1/change-semantic-validation.jsonl), [12개 선택 원장](../qa05-selection-manifest.json).

## 메모리 차이의 원인과 선택 해석

A의 전체 최대값은 A5/S2-02 부근에 나타났다. 이 시점의 30건 RECENT_TASK 조회와 20MiB 원문을 순위 계산 전에 문자열화하는 공통 조회 경로가 겹친다. 임시 문자열 할당이 peak에 기여했을 가능성은 실행 기록에 근거한 추론이며 object별 allocation을 추적해 확정한 원인은 아니다. 최대값과 활동별 peak 원값을 함께 유지했고 낮은 반복값으로 대체하지 않았다. 이 한 peak의 차이를 A 구조가 본질적으로 그만큼 RAM을 더 필요로 한다는 주장으로 이전하지 않는다.

실제 선택은 높은 비용/메모리 점수로 정확성·유효 반응·완료 실패를 상쇄하지 않는다. 원값, 다섯 paired 차이, 보호 조건과 실제 실패의 종류를 함께 비교한다. 같은 코드 변경 범위 또는 실용 차이 기준 미충족도 측정 결론으로 인정한다. 현재 두 구현·고정 P21·실제 Qwen의 결과를 전체 VIA/모든 모델/모든 16개 조합의 우열로 일반화하지 않는다.

## 전체 호출 비용과 재가공 안내

본 P21 호출 비용과 준비·제외 실행·변경·judge 비용은 구분했다. [모든 실제 API 호출 CSV](./all-actual-api-calls.csv)와 [전 세대 비용 inventory](./spend-inventory.json)에 실제 usage, 알려진 비용과 usage 미확인 호출, 예산 예약액을 각각 보존한다. 무료 token 한도와 실제 invoice를 사용량 환산액과 혼동하지 않는다.

원시 증거 검증은 53개 frozen source, 10개 활동, 1958개 본 실행·판정 호출 및 3916개 Context/출력 artifact를 검사해 PASS했다. [무결성 검수](./evidence-integrity.json), [raw 필드·경로별 재사용 안내](../raw-audits/raw-data-guide.md), [QA04/06 독립 검수](../raw-audits/qa04-qa06-independent-review.json), [QA05 선택 독립 검수](../raw-audits/qa05-selection-independent-review.json).

V1~6은 본 수치에 섞지 않은 준비/제외 실행이다. V3의 큰 원시 JSONL 세 개는 원본을 로컬에 그대로 보존하고 Git에는 bytes가 동일하게 복원되는 gzip과 원본/gzip SHA-256 manifest를 게시했다. 다른 세대의 실패·비용 기록도 보존하며 준비에 든 비용을 지우지 않는다. [V3 대형 raw 보존 manifest](../../dp41-p21-mac-text-v3-20261012/git-large-raw-preservation.json).

최종 독립 재계산은 174,750개 검사에서 실패 0으로 여섯 지표, 보호 조건, paired 차이와 raw 연결을 확인했다. [독립 전체 검수](../raw-audits/final-independent-review.json). 게시 대상 13,758개 파일의 평문 credential 검사는 발견 0이었다. [게시 검사](./publication-secret-check.json).
