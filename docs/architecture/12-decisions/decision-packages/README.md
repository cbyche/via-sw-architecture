# Target-derived Architecture Decision Packages

> 상태: **목표 Architecture 검토 기준선 확정 / 다음 작업은 후보 목록 공동 작성 / 정식 package·비교 결과 없음**

현재 검토 자료는 [구조적 Decision Point 후보](./candidates/README.md)에 있다. [선발 원칙](./candidates/selection-principles.md), 후보별 두 방안·비교 그림과 [자체 검토 기록](./candidates/review-notes.md)을 작성했다. 사용자 요청에 따라 Component 변화가 분명한 구조를 먼저 검토하며 ASR 논의는 후속으로 둔다. 후보는 아직 사용자 선정 전이다.

이 디렉터리는 [검토 완료 목표 VIA Architecture](../target-architecture/README.md)에서 역으로 추출할 구조적 선택·steelman·관련 ASR 후보 목록과, 검토를 거쳐 발전시킬 Architecture Decision Package를 둔다.

## 다음 작업: 후보 목록부터 함께 작성한다

먼저 간결한 후보 표를 작성하고 사용자와 범위·묶음·대안의 강도를 논의한다. 정식 package 번호나 개수, ASR 목록을 미리 고정하지 않는다.

| 후보 표의 필드 | 작성 기준 |
| --- | --- |
| 선택한 구조·해결 문제 | target 본문 절·계약에 연결하며 Component 이름만으로 항목을 만들지 않음 |
| 가장 강한 현실적 대안 초안 | 동일한 사용자 목표·제품 경계를 충족하며 현재의 adapter·검증 순서·저장 방식을 그대로 강제하지 않음 |
| 실제 구조 차이 | 상태 소유권·계약·호출·배치·영속성·fault boundary에서 달라지는 점 |
| 관련 ASR과 인과관계 | 직접 효과·조건부 효과·차이 없음·미해결을 구분; 현재 네 ASR 밖의 품질도 검토 |
| 논의할 비용·결합 | 다른 후보와 중복/결합되는 부분, 대안이 유리할 조건, 남은 질문 |

사용자가 관심을 표시한 Model Access scheduling, Agent 선택, 응답 제어, 프로세스 배치는 발견의 출발점이며 반드시 각각 package가 되는 것은 아니다. 프로세스 수만으로 accuracy 우위를 주장하지 않으며 단일 process 대안에도 충분한 thread·비동기·자원 제어를 허용한다.

메모리 사용량은 중요 ASR 후보로 검토한다. 공유 weights는 중복 집계하지 않고 역할별 KV·작업 공간·capture/pin·Context cache·IPC·queue·helper 비용과 동시 부하를 함께 고려한다. 후보별 최대 사용량·증가 상한·예산 부족 시 기능과 다른 ASR의 trade-off를 정의할지 논의한다. QA-41의 현재 diagnostic 정의·분류를 이 안내만으로 변경하거나 새 metric/target을 확정하지 않는다.

후보 표를 검토한 뒤 중요한 선택만 정식 package로 확장한다. 다음 세션도 AGENTS.md와 target README의 확정 범위·리뷰 반영 사항을 먼저 읽는다. 기준선을 처음부터 재설계하거나 기존 DP에 매핑하지 않으며, 후보 논의를 구현·모델 실행·성능 측정의 승인으로 해석하지 않는다.

## 도출 순서

```text
합의된 목표 Architecture
  → 중요한 품질 차이를 만드는 구조적 선택과 가장 강한 현실적 대안
  → 후보 구체화와 함께 ASR 추가·변경·비교 기준 정의
  → 책임·상태·계약·호출·배치·fault boundary 차이
  → 장점·비용·대안이 유리한 조건
  → 반증 조건과 결과 전 측정 freeze
```

새 package는 기존 VIA-DP-01~18 번호를 이어받거나 그 inventory에 다시 매핑하지 않는다. 모든 Component를 package로 만들지 않고, 그 과정에서 정의한 ASR 중 하나 이상을 실질적으로 바꾸는 구조적 선택만 남긴다. 현재 네 ASR은 출발점이며 목록·개수·정의가 고정된 것은 아니다. Decision Point와 steelman 후보를 구체화하면서 ASR의 의미·우선순위·적용 범위·평가 기준을 함께 정한다.

각 package는 최소한 다음을 포함한다.

- 선택한 목표 구조와 해결하는 구체적 문제
- steelman한 현실적 대안
- Component, 상태 소유권, 계약, call graph, deployment 또는 fault boundary 차이
- 비교에 사용할 ASR 정의와 선정·변경 근거, 각 ASR에 영향을 주는 인과 경로와 applicability
- 선택 구조의 비용과 약점
- 대안이 더 유리해지는 조건
- 검증 방법과 선택 구조를 기각·재검토할 반증 조건

현재는 검토 완료 기준선을 확보했으며 후보 목록 공동 작성이 다음 작업이다. 정식 package와 비교 결과는 아직 없다. 기존 `via-dp-*.md`는 이전 decision-first inventory의 reference이며 이 디렉터리의 초안으로 간주하지 않는다.

## 비교 원칙

- `accuracy → responsiveness → modifiability → reliability/recoverability`는 목표 Architecture를 선택·설계할 때의 우선순위다.
- 모든 package는 후보 구체화 과정에서 정한 ASR 전체를 `PRIMARY`, `REGRESSION_ONLY`, `NOT_APPLICABLE`, `UNRESOLVED` 중 하나로 명시한다.
- 선택안과 steelman은 동일하게 동결한 모집단·실패 처리·측정 경계로 모든 applicable ASR을 각각 측정한다.
- 한 축의 결과로 후보를 먼저 제외한 뒤 다른 축을 측정하지 않는 방식은 사용하지 않는다.
- 선택안이 더 정확하지만 느리거나, 변경 범위가 작지만 복구율이 낮은 결과도 정상적인 trade-off로 보고한다.
- 비교 축을 하나의 가중 점수로 합치지 않는다. 최종 rationale에서 어떤 손해를 감수하고 왜 선택했는지 명시한다.
- Qualification 위반이 최종 선택을 막더라도 해당 후보의 측정값과 실패 evidence는 보존한다.
