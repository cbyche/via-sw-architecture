# ASR-04/05 Target Rationale — Change Locality

> 작성일: 2026-09-21
> 상태: **Target proposal의 근거**. 아직 0~5 score band나 Architecture 후보 결과를 보지 않고 작성한다.

## 1. 외부 근거가 말해 주는 것과 말해 주지 않는 것

ISO/IEC 25010의 Maintainability 계열과 SEI의 modifiability tactics는 변경을 국소화하고 ripple effect를 제한하는 것이 좋은 구조의 핵심임을 지지한다.

- ISO/IEC 25010의 modularity/modifiability 개념은 한 component 변경이 다른 component에 미치는 영향을 최소화하고 효과적으로 수정할 수 있어야 함을 다룬다.
- SEI modifiability tactics는 semantic coherence, information hiding, interface 사용, dependency restriction 등을 통해 변경이 가능한 한 적은 module에 영향을 주도록 하는 것을 목표로 한다.

근거:
- ISO/IEC 25010:2023 product quality model: https://www.iso.org/standard/78176.html
- SEI, Modifiability Tactics: https://www.sei.cmu.edu/library/modifiability-tactics/

그러나 **ISO나 SEI가 "변경 요소 2개 이하가 합격" 같은 VIA용 숫자를 규정하지 않는다.** 따라서 숫자 target은 외부 표준의 숫자를 가져오는 것이 아니라 VIA의 책임 경계와 intentional change catalog에서 도출한다.

## 2. ASR-04 — Agent Ecosystem Interoperability & Substitutability

고정 제품 방향은 VIA가 특정 Primary Agent Runtime에 종속되지 않는 Agent-neutral orchestration system이라는 것이다.

Agent change A-01~09에서 정상적인 구조의 기대는 다음과 같다.

1. provider/protocol-specific 차이는 integration boundary에 국소화한다.
2. VIA의 Conversation/Request/Task/Policy/Core orchestration semantics까지 반복적으로 수정하지 않는다.
3. 기능 유지가 전제다. 변경 요소가 적어도 기능을 잃으면 유효한 값이 아니다.

### Target proposal

```text
ASR-04 target:
Average changed architecture elements per Agent change <= 2.0
```

제품적 해석:

- **0**: 기존 계약 안의 configuration/registration만으로 대응
- **1**: Agent-specific adapter/binding 한 요소에서 국소화
- **2**: adapter/binding + 공통 registry/factory/contract 중 실제로 필요한 한 요소 수정
- **3 이상이 반복**: Agent 변화가 integration seam을 넘어 Core contract/state/responsibility로 전파되는 구조적 결합을 의심해야 함

따라서 평균 2.0은 "외부 표준의 정답 숫자"가 아니라 **Agent-neutral이라는 VIA의 제품 정체성을 만족시키는 locality budget**이다.

## 3. ASR-05 — Evolvability & Maintainability

ASR-05의 M-01~09 + C-01~06은 Agent 변화보다 이질적이다.

- Model provider/runtime change는 보통 binding/configuration 수준에 국소화 가능하다.
- Context provider/format 변화는 adapter/contract 수준 변화가 가능하다.
- persistent-state schema 변화는 정당하게 State schema + Repository behavior + Migration responsibility를 함께 바꿀 수 있다.

10 문서의 C-06 예시도 다음 세 요소를 정당한 변경으로 예시한다.

1. State/Data Schema 수정
2. Repository/loader-writer 책임 수정
3. Migration responsibility 추가

### Target proposal

```text
ASR-05 target:
Average changed architecture elements per non-Agent change <= 3.0
```

제품적 해석:

- 일반 Model/Context provider 변화는 이상적으로 0~2 요소에 국소화
- persistent-state evolution처럼 본질적으로 migration을 요구하는 변화는 3 요소까지 정상 범위
- 평균이 3을 지속적으로 초과하면 intentional variable이 subsystem boundary를 넘어 여러 Core responsibility로 ripple되는 구조를 의미할 가능성이 커짐

ASR-04보다 한 단계 넓은 budget을 두는 이유는 non-Agent catalog에 **실제 schema migration과 deployment/runtime relocation**이 포함되어 있기 때문이다.

## 4. 숫자를 후보 결과에서 역산하지 않는 규칙

위 2.0 / 3.0은 Architecture 후보의 변경량을 계산하기 전에 제안한다.

Measurement Contract Definition에서 target을 승인하면:

- A-01~09 모두 같은 ASR-04 target/score rule을 사용한다.
- M-01~09+C-01~06 모두 같은 ASR-05 target/score rule을 사용한다.
- 특정 DP나 후보에 유리하도록 threshold를 바꾸지 않는다.
- raw change별 C/I/S/D breakdown을 항상 보존한다.

평균 하나만으로 hotspot을 숨기지 않도록 per-change raw maximum과 change ledger도 secondary evidence로 표시하지만 대표 metric은 08에서 확정한 average changed elements를 유지한다.

## 5. 아직 Measurement Contract Definition에서 결정할 것

- 위 proposed target 2.0 / 3.0 승인 여부
- target을 기준으로 한 0~5 score boundary의 폭
- 처리 불가 / 미측정 change의 score 처리

score band는 Architecture 후보 결과를 보기 전에 고정해야 한다.
