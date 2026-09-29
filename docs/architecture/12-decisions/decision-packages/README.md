# Target-derived Architecture Decision Packages

> 상태: **목표 Architecture 합의 대기 / 새 package 없음**

이 디렉터리는 [목표 VIA Architecture](../target-architecture/README.md)가 충분히 합의된 뒤, 그 구조에서 역으로 추출한 Architecture Decision Package를 둔다.

## 도출 순서

```text
합의된 목표 Architecture
  → semantic accuracy 또는 responsiveness에 큰 인과 효과가 있는 구조적 선택
  → 같은 문제를 해결하는 가장 강한 현실적 대안
  → 책임·상태·계약·호출·배치·fault boundary 차이
  → 장점·비용·대안이 유리한 조건
  → 반증 조건과 결과 전 측정 freeze
```

새 package는 기존 VIA-DP-01~18 번호를 이어받거나 그 inventory에 다시 매핑하지 않는다. 모든 Component를 package로 만들지 않고, 성능·정확성·장애 특성을 실질적으로 바꾸는 구조적 선택만 남긴다.

각 package는 최소한 다음을 포함한다.

- 선택한 목표 구조와 해결하는 구체적 문제
- steelman한 현실적 대안
- Component, 상태 소유권, 계약, call graph, deployment 또는 fault boundary 차이
- accuracy와 responsiveness에 영향을 주는 인과 경로
- 선택 구조의 비용과 약점
- 대안이 더 유리해지는 조건
- 검증 방법과 선택 구조를 기각·재검토할 반증 조건

현재는 목표 Architecture가 제안 상태이므로 package 파일을 만들지 않는다. 기존 `via-dp-*.md`는 이전 decision-first inventory의 reference이며 이 디렉터리의 초안으로 간주하지 않는다.

## 비교 원칙

- `accuracy 1순위, responsiveness 2순위`는 목표 Architecture를 선택·설계할 때의 우선순위다.
- 선택안과 steelman은 동일하게 동결한 모집단·실패 처리·측정 경계로 accuracy와 responsiveness를 각각 측정한다.
- 한 축의 결과로 후보를 먼저 제외한 뒤 다른 축을 측정하지 않는 방식은 사용하지 않는다.
- 선택안이 더 정확하지만 느리거나 steelman이 더 빠르지만 덜 정확한 결과를 정상적인 trade-off로 보고한다.
- 두 축을 하나의 가중 점수로 합치지 않는다. 최종 rationale에서 어떤 손해를 감수하고 왜 선택했는지 명시한다.
- Qualification 위반이 최종 선택을 막더라도 해당 후보의 측정값과 실패 evidence는 보존한다.
