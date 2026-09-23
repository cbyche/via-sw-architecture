# Quality Model, QA Catalog, and ASR Selection

> 상태: **QC·QA taxonomy current / QA-01~QA-12 정의 중 / ASR 미선정**
>
> 근거: [01 시스템 정의](../01-system-mission-and-boundary.md) · [05 대표 UC](../05-representative-use-cases.md) · [06 비교 조건](../06-fixed-assumptions.md) · [07 변경 집합](../07-intentional-variables.md)

## 1. 목적

VIA에는 많은 품질 관심사가 존재한다. 이 문서는 ISO/IEC 25010을 품질 분류의 기준으로 삼되, 상위 관심사와 실제 측정 대상과 Architecture 중요성 판정을 서로 다른 단계로 관리한다.

```text
ISO/IEC 25010 quality model
        ↓
QC — 제품 수준의 상위 품질 관심사
        ↓
QA — 측정 가능한 VIA 품질 속성
        ↓
DP별 구조 인과·trade-off 평가
        ↓
ASR classification — Architecture에 중대한 QA만 분류
```

ISO/IEC 25010과의 연결은 분류 근거이며 인증 또는 표준 적합성 주장 자체가 아니다.

## 2. ID와 의미

| 구분 | ID | 의미 |
| --- | --- | --- |
| Quality Concern | `QC-01`~`QC-10` | ISO/IEC 25010 관점과 VIA 제품 문맥에서 도출한 상위 품질 관심사 |
| Quality Attribute | `QA-01`~`QA-12` | stimulus, response, measure와 evidence를 정의하여 후보 Architecture를 비교할 품질 속성 |
| Architecture Significant Requirement | 별도 ID 없음 | Architecture에 중대한 영향을 준다고 판정된 QA에 부여하는 분류 |

ASR은 별도의 요구사항 목록이나 번호 체계가 아니다. 예를 들어 QA-07이 Architecture에 중대한 것으로 확정되면 `QA-07 (ASR)`로 표시하며 별도의 numbered ASR ID를 새로 만들지 않는다.

현재 확정된 ASR은 없다. 모든 QA의 측정 계약과 DP별 구조 인과를 검토한 뒤 상태를 기록한다.

## 3. Quality Concern catalog

| ID | Quality concern | ISO/IEC 25010 기반 관점 | 현재 처리 |
| --- | --- | --- | --- |
| QC-01 | User-Experienced Responsiveness | Performance efficiency, interaction capability | QA-01~QA-04로 구체화 |
| QC-02 | Task Completion Effectiveness | Functional suitability | QA-05로 구체화 |
| QC-03 | Interaction & Task Continuity | Reliability, interaction capability | QA-06으로 구체화 |
| QC-04 | Agent Ecosystem Interoperability & Substitutability | Compatibility, flexibility | QA-07로 구체화 |
| QC-05 | Evolvability & Maintainability | Maintainability, flexibility | QA-08로 구체화 |
| QC-06 | Cross-Device Portability & Adaptability | Flexibility/portability | 현재 제품 범위 밖; 재평가 조건 유지 |
| QC-07 | Compute & Energy Efficiency | Performance efficiency | 독립 QA 미정; 필요한 경우 resource budget과 함께 정의 |
| QC-08 | Reliability & Recoverability | Reliability | QA-09·QA-10으로 구체화 |
| QC-09 | Privacy, Security & Action Safety | Security, safety-related risk control | QA-11·QA-12로 구체화 |
| QC-10 | Diagnosability & Operability | Maintainability, interaction capability | 공통 evidence 요구로 유지; 독립 QA 미정 |

QC는 coverage와 traceability를 위한 상위 분류다. QC 자체의 점수나 A/B 승자를 만들지 않는다.

## 4. Current QA catalog

| ID | Quality Attribute | Current state |
| --- | --- | --- |
| QA-01 | Delegated Task Result Responsiveness | Voice definition current; harness/target/result pending |
| QA-02 | VIA Direct Voice Response Responsiveness | Voice definition current; harness/target/result pending |
| QA-03 | Agent Progress Voice Feedback Responsiveness | Voice definition current; harness/target/result pending |
| QA-04 | Concurrent Task Performance Isolation | contract review required after QA-01 change |
| QA-05 | Task Completion Effectiveness | definition current; execution evidence pending |
| QA-06 | Interaction & Task Continuity | definition current; execution evidence pending |
| QA-07 | Agent Ecosystem Interoperability & Substitutability | definition current |
| QA-08 | Evolvability & Maintainability | definition current |
| QA-09 | Recovery Timeliness & Recoverability | definition current |
| QA-10 | Dependency Failure Containment & Graceful Degradation | definition current |
| QA-11 | Privacy Exposure Minimization | definition current |
| QA-12 | Action & Access Safety | definition current |

각 QA는 하나의 품질 이름만으로 끝나지 않는다. 다음 항목을 결과 전에 고정해야 비교 가능한 QA가 된다.

- 사용자 또는 외부 source의 stimulus와 시작 event
- 성공 response와 종료 event
- 적용 가능한 DP와 실제 software path
- fixture, oracle, 반복 수와 집계 방법
- target, score boundary와 evidence level
- 실패·timeout·N/A 처리 규칙

상세 계약은 [Measurement Guide](../11-measurement/README.md)에서 관리한다.

## 5. ASR 판정 계약

QA를 ASR로 분류하려면 다음 근거를 함께 남긴다.

1. **Product relevance** — 핵심 UC, 중요한 변화 또는 신뢰 경계와 직접 연결된다.
2. **Architectural impact** — 책임 배치, interface, state authority, process/deployment boundary 또는 persistent model을 바꿀 수 있다.
3. **Structural causality** — 합리적인 DP 대안의 차이가 해당 QA에 영향을 주는 경로를 설명할 수 있다.
4. **Material consequence** — 잘못된 선택의 비용·위험·변경 난이도 또는 QA trade-off가 중요하다.
5. **Traceable evidence** — scenario, measurement 또는 명시적인 설계 분석으로 판정을 추적할 수 있다.

측정값 차이만으로 ASR을 정하지 않는다. 안전·규제·회복성처럼 반드시 지켜야 하는 구조 제약은 후보가 동일 점수를 내더라도 ASR일 수 있다. 반대로 제품적으로 중요한 QA라도 구조 대안과 무관하면 해당 DP의 primary driver는 아니다.

QA별 ASR 상태는 다음 중 하나로 관리한다.

| 상태 | 의미 |
| --- | --- |
| `UNASSESSED` | 측정 계약 또는 구조 영향 검토 전 |
| `CANDIDATE` | Architecture 영향 가설은 있으나 근거 검토 미완료 |
| `CONFIRMED_ASR` | 위 판정 기준과 근거가 승인됨 |
| `NOT_ASR` | 현재 범위에서 Architecture 중요성이 충분하지 않음 |

현재 QA-01~QA-12는 모두 `UNASSESSED`다. Candidate A/B 결과를 보기 전에 판정 기준을 유지하고, 결과가 잘 갈리는 QA만 사후 선택하지 않는다.

## 6. 과거 ID와의 관계

과거 metric ID, parent QA ID와 numbered ASR ID의 상세 mapping은 [historical archive](../../archive/w12-g1/README.md)에만 보존한다. active 문서와 새 결과에서는 현재 QC·QA ID와 ASR 분류만 사용한다.
