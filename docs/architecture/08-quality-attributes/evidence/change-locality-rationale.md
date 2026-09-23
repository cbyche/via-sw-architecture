# QA-21~QA-23 Change Locality Contract Rationale

> 작성일: 2026-09-23
> 상태: **USER REVIEW DRAFT** — target, score band, change pack 모두 결과 전 승인 대상이다.

## 1. 왜 이 두 QA를 유지하는가

Responsiveness와 correctness를 높이는 구조는 semantic responsibility, state, adapter와 runtime boundary를 더 복잡하게 만들 수 있다. QA-21~23은 이 trade-off를 **변화 한 건을 수용할 때 실제로 손대야 하는 Architecture Element 수**로 드러낸다.

ISO/IEC 25010의 modularity/modifiability와 SEI의 modifiability tactics는 변경 국소화와 ripple-effect 제한의 중요성을 뒷받침하지만, VIA용 숫자 target을 제공하지는 않는다. 따라서 `2개`와 `3개`는 표준값이 아니라 아래 고정 change pack과 VIA 책임 경계에서 도출한 초안 budget이다.

- ISO/IEC 25010:2023 product quality model: https://www.iso.org/standard/78176.html
- SEI, Modifiability Tactics: https://www.sei.cmu.edu/library/modifiability-tactics/

## 2. Architecture Element의 의미

Architecture Element는 파일·클래스·함수·diagram box가 아니다. 다음 네 유형 중 하나이며, 소유자·경계·독립성·변경 판정·설계 근거를 모두 설명할 수 있어야 한다.

| 유형 | 한 개로 세는 단위 |
| --- | --- |
| C — Component | 구분되는 입력·출력과 책임을 가진 처리 책임 |
| I — Interface | 제공자와 소비자가 함께 지키는 독립된 의미·오류·수명 계약 |
| S — State | 독립 소유자·참조자·수명·갱신/이행 규칙이 있는 상태 모델 |
| D — Runtime | 시작·종료·복구·통신 결합을 독립 관리하는 배치 명세 |

후보별 전수 element ledger를 결과 전에 동결한다. 같은 의미를 파일 여러 개로 나눠도 하나이고, 부모 요약과 자식을 중복 계산하지 않는다. 설정값·rename·rebuild만 바뀌면 0개다. 상세 등록·중복 금지·수정/추가/제거 규칙은 [Architecture Element Definition](../../10-element-definition.md)이 유일한 기준이다.

## 3. QA-21 Agent change pack

QA-21은 다음 **9개를 모두** 같은 후보 baseline에 독립 적용한다.

| ID | 변화 |
| --- | --- |
| A-01 | Agent 추가 |
| A-02 | 동일 업무 Agent 교체 |
| A-03 | 다른 Agent protocol 지원 |
| A-04 | 상태 전달 방식 변경 |
| A-05 | 실행 식별·후속 요청 계약 변경 |
| A-06 | capability 계약 재구성 |
| A-07 | Agent 인증 계약 변경 |
| A-08 | 질문·승인 응답 계약 변경 |
| A-09 | 결과물 전달 계약 변경 |

```text
QA-21 = mean N(A-01...A-09)
draft target = 평균 2.0개 이하
```

0은 기존 계약 안의 configuration/registration, 1은 한 adapter/binding에 국소화, 2는 adapter와 실제로 필요한 공통 contract/registry 한 요소의 변화로 해석할 수 있다. 3개 이상이 반복되면 Agent 변화가 integration seam을 넘어 Core responsibility/state로 퍼지는지 검토한다.

## 4. QA-22 Model·Context·state change pack

QA-22는 다음 **15개를 모두** 같은 후보 baseline에 독립 적용한다.

| ID | 변화 | ID | 변화 |
| --- | --- | --- | --- |
| M-01 | S2S 제공자 교체 | M-02 | 의미 판단 모델 교체 |
| M-03 | 모델 실행 프로필 변경 | M-04 | 외부 Cloud → Private Cloud |
| M-05 | Remote → 사용자 PC | M-06 | 사용자 PC → Remote |
| M-07 | S2S 이벤트 정보 변경 | M-08 | 의미 판단 응답 방식 변경 |
| M-09 | 모델 대화 이력 전달 계약 변경 | C-01 | 정보 Source 제공자 교체 |
| C-02 | 문서 형식 추가 | C-03 | 화면 연동 계약 변경 |
| C-04 | 기억 기록 형식 확장 | C-05 | 같은 종류의 정보원 추가 |
| C-06 | 대화·업무 기록 형식 변경 |  |  |

```text
QA-22 = mean N(M-01...M-09, C-01...C-06)
draft target = 평균 3.0개 이하
```

일반 provider 변화는 0~2개에 국소화할 수 있지만 persistent-state evolution은 schema, repository behavior와 migration responsibility를 정당하게 함께 바꿀 수 있다. 따라서 QA-22의 draft budget이 QA-21보다 한 요소 넓다.

각 change의 정확한 before/after, 동일 조건과 완료 확인은 [Intentional Variables](../../07-intentional-variables.md)가 기준이다. 제목만 보고 fixture를 축약하지 않는다.

## 5. QA-23 Experiment·Logging change pack

QA-23은 연구 조직이 새로운 기능을 시험하고 로그를 개선할 때 반복되는 다음 **5개 변화**를 같은 후보 baseline에 독립 적용한다. 이 묶음은 07의 제품·외부 계약 변화 24개와 별도이며, 새로운 사용자 기능을 추가하지 않는다.

| ID | 변화 | 유지 조건 |
| --- | --- | --- |
| E-01 | 기존 사용자 경로에 새 timing span 추가 | 기존 동작·metric 결과·privacy 유지 |
| E-02 | Conversation/Request/Task/Agent run을 잇는 correlation dimension 추가 | 기존 identity와 trace reader 유지 |
| E-03 | trace event schema를 새 version으로 확장 | 기존 evidence를 계속 읽고 검증 가능 |
| E-04 | evidence export/storage 방식을 local file과 분석 backend 사이에서 변경 | 같은 raw meaning, 누락·중복 금지, privacy 유지 |
| E-05 | 새 A/B experiment assignment와 실제 exposure configuration 기록 추가 | 비할당 사용자 동작과 기존 실험 재현성 유지 |

```text
QA-23 = mean N(E-01...E-05)
draft target = PENDING
```

사람이 구현하는 데 걸린 일수나 코드 line 수는 세지 않는다. 새 실험·로그 요구가 몇 개의 Architecture 책임·계약·상태·배치로 퍼지는지만 센다.

## 6. DP별 applicability 판정

QA-21~23을 모든 DP에 억지로 적용하지 않는다. 후보 명세가 완성된 뒤 각 `DP × change`에 대해 아래 중 하나를 **결과 전에** 기록한다.

| 값 | 의미 |
| --- | --- |
| `APPLICABLE` | 그 DP의 A/B 구조 차이가 해당 변화의 element count를 바꿀 인과가 있음 |
| `REGRESSION_ONLY` | 변화 수용은 확인하지만 그 DP의 구조 차이와 직접 인과가 없음 |
| `NOT_APPLICABLE` | 변화 대상 자체가 후보에 없으며 이유를 설명할 수 있음. 0으로 평균에 넣지 않음 |
| `UNRESOLVED` | 필수 기능을 유지할 변경 설계를 만들지 못함. 0으로 처리하지 않음 |

Primary expectation은 Agent boundary를 바꾸는 DP에 QA-21, semantic/model/context/state boundary를 바꾸는 DP에 QA-22, instrumentation/evidence boundary를 바꾸는 DP에 QA-23이지만, 이름만으로 결정하지 않고 실제 responsibility·contract·state·deployment 차이를 확인한다.

## 7. 결과가 나오기 전에 잠글 것

- 후보별 전체 Architecture Element ledger와 granularity review
- 각 change의 `DP × change` applicability
- changed ID, 변경 종류와 이유를 기록할 ledger schema
- 기능 유지 regression과 `UNRESOLVED` 처리
- 평균, raw maximum, C/I/S/D breakdown과 0~5 band

개발 공수나 M/M은 팀 숙련도와 prototype 완성도에 크게 좌우되므로 대표 metric으로 환산하지 않는다. 필요하면 secondary planning evidence로만 병기한다. 후보 결과를 본 뒤 element를 쪼개거나 합치고 change pack 또는 threshold를 바꾸지 않는다.
