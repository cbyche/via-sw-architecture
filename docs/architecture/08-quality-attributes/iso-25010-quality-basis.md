# ISO/IEC 25010 기반 VIA 품질 비교 근거

> 2026-10-01 · ISO/IEC 25010:2023 제품 품질 모델에 대한 VIA 적용 설명
> 기존 QA ID·대표 metric·ASR 지위·우선순위·target을 변경하지 않는다. 표준 적합성 인증이나 측정 완료 주장이 아니다.

## 기준 판본과 출처

이번 비교는 **ISO/IEC 25010:2023, 제2판의 제품 품질 모델**을 기준으로 한다. 기존 [변경 국소화 근거](./evidence/change-locality-rationale.md)도 이 판본을 참조한다. 판본·적용 범위는 [ISO 공식 소개](https://www.iso.org/standard/78176.html), 개정 사항과 공개된 정의는 [ISO 원문 preview](https://cdn.standards.iteh.ai/samples/78176/13ff8ea97048443f99318920757df124/ISO-IEC-25010-2023.pdf)를 확인했다. 2023판은 9개 특성을 사용하며 Interaction capability·Flexibility 명칭과 Safety 추가를 반영한다. 2011판의 분류를 섞지 않는다.

공개 preview는 모든 부특성 정의를 포함하지 않는다. 분석·시험·설치 관련 용어는 [인도 TEC의 공개 기술 초안](https://www.tec.gov.in/pdf/consultations/Combined%20Standard%20on%20AI%20Robustness%20dated%2006052025-57070.pdf)의 품질 모델 적용 표(PDF 12~13쪽)로 교차 확인했다. 이 자료는 ISO 발행본의 대체 규범이 아니다. 아래 한국어와 VIA 적용은 **우리 과제의 해석**이며 표준 문구의 공인 번역이 아니다. 표준 전문이나 정의를 복제하지 않는다.

## 기존 QA와 표준 품질속성의 대응

품질속성, 제품 요구, 측정 지표를 구분한다. 예를 들어 **성능 효율성 / 자원 활용성 → PC에서 감당할 메모리 → QA-41 최대 committed memory** 순이다. “CPU”, “삭제 부담”, “로그”는 각각 자원·비용·수단이며 그 자체를 새로운 ISO 품질속성으로 부르지 않는다.

| VIA의 현재 항목 | ISO 특성 → 부특성 (정확한 영문 명칭) | VIA에서의 적용·한계 |
| --- | --- | --- |
| QA-09; QA-01~05 | Performance efficiency → Time behaviour (성능 효율성 → 시간 특성) | 의미 있는 응답·제어까지의 시간. QA-04는 별도 회귀이며 QA-09에 합산하지 않음 |
| QA-19; QA-11~15 | Functional suitability → Functional correctness (기능 적합성 → 기능 정확성) | 목표·대상·관계 및 처리 결과의 정확성. 인식률·검색 recall만으로 대체하지 않음 |
| QA-29; QA-21~23 | Maintainability → Modifiability / Modularity (유지보수성 → 변경 용이성 / 모듈성) | 동일 변화의 실제 Architecture Element 변경 범위. 부특성 전체를 측정하는 값은 아니며 확인 작업·코드 줄 수를 변경 요소 수로 세지 않음 |
| QA-39; QA-31/32 | Reliability → Fault tolerance / Recoverability (신뢰성 → 결함 허용성 / 복구성) | 장애 영향 제한과 올바른 상태 회복. QA-39는 VIA의 복합 판정이지 ISO가 정한 계산식이 아님. 장애 중 기능 지속과 장애 뒤 복원을 구별 |
| QA-41 메모리 | Performance efficiency → Resource utilization (성능 효율성 → 자원 활용성) | 메모리는 자원 종류 중 하나. CPU·에너지·디스크도 같은 부특성의 다른 관찰 대상. QA-41은 진단 지위 유지 |
| QA-51 보호정보 노출 | Security → Confidentiality (보안성 → 기밀성) | 허용 범위 밖의 정보 노출을 다룸. 삭제 비용·복제본 수만으로 노출량을 추정하지 않음 |
| QA-61 실행 trace 완전성 | Maintainability → Analysability (유지보수성 → 분석 용이성)를 **지원** | 기록이 빠짐없이 연결되는지는 원인 분석의 기반이다. 완전성 비율과 사람이 원인을 찾는 정확도·수고는 서로 다른 관찰 |
| QA-62 평가 근거 재현 | Maintainability → Testability (유지보수성 → 시험 용이성)를 **지원**하는 평가 qualification | 이전 평가 결과의 재계산 가능성을 요구한다. 제품 시험 용이성 전체나 운영 상태 replay의 품질을 대신하지 않음 |

표준이 VIA의 QA 번호·ASR 선정·수치 target·계산식·대안 기호를 정해주는 것은 아니다. 현재 의미와 집계는 [Core ASR 계약](./core-asr-contract.md), QA-61/62는 [Observability 계약](./observability.md)이 기준이다. 기존 catalog의 Responsiveness·Observability 등은 **VIA의 관리용 category**이며 ISO 상위 특성의 이름으로 제시하지 않는다.

## 추가 비교에 사용하는 부특성

| 특성 → 부특성 | 네 DP에서 묻는 구체 질문 | 구별해야 할 것 |
| --- | --- | --- |
| Performance efficiency → Resource utilization | 같은 일을 수행할 때 색인·인식·실행 원장·journal이 CPU/에너지/디스크를 얼마나 더 요구하는가? | 메모리와 디스크 구별. 여러 자원을 별개 품질 특성으로 부풀리거나 합산하지 않음 |
| Maintainability → Analysability | 잘못된 검색·멈춘 요청·발화 연결·상태 전이의 원인을 정확히 찾기 쉬운가? | 로그가 많다는 사실과 분석이 쉽다는 결론은 다름 |
| Maintainability → Testability | 삭제·늦은 완료·timeout 등의 순서를 통제하고 기대 결과와 대조하기 쉬운가? | 시험 코드 재사용량만으로 판정하지 않음. 양안에 같은 요구와 oracle을 적용 |
| Flexibility → Installability (유연성 → 설치 용이성) | 같은 PC에 음성 runtime과 weights를 설치·제거할 때 필요한 의존성·실패 정리 절차는 무엇인가? | 다운로드 byte 감소 자체는 자원 지표. 실제 설치/제거 과정에 미치는 영향을 설명해야 함 |

추가 행은 이 부특성에서 도출한 **VIA 시나리오 제안**이다. 새 QA ID·독립 ASR·측정 계약 확정이 아니며 기호는 행에 적힌 조건에서의 정성 가설이다. 관찰 항목·모집단·상한·시험 절차의 freeze와 실행은 후속 작업이다.

## 분류에서 특히 주의할 경계

- Compatibility의 Co-existence / Interoperability는 제품 간 공존·상호운용이다. 옛 workflow definition을 지원하는 개발 부담을 막연히 “업데이트 호환성”으로 분류하지 않는다. 실제 변경은 QA-29, 실패 뒤 재개는 QA-39에서 다룬다.
- “설명 가능성”이라는 임의 부특성을 만들지 않는다. 이 DP에서 내부 상태 전이의 원인을 조사하는 문제는 Analysability로 다룬다. 모델 내부의 설명 가능성을 구현한다는 뜻이 아니다.
- 현재 저장소에서 부르는 safety qualification은 평가의 필수 통과 규칙이다. 이것만으로 ISO의 Safety 특성을 평가했다고 말하지 않는다. 보호정보 접근은 기밀성, 잘못된 요청 처리는 기능 정확성 등 해당 품질 경로를 명시한다.
- 9개 특성을 모두 비교표에 억지로 넣지 않는다. 예를 들어 UI 동작·상호운용 계약이 공통이면 해당 DP의 구조적 우열 축으로 주장하지 않는다. 이 문서는 VIA 전체 표준 coverage 심사가 아니라 네 설계안의 인과가 있는 품질 비교다.
- 동일한 필수 요구를 제시했다고 결과가 동등한 것은 아니다. 구조가 같은 한정 경로만 `(0)`으로 보고, 정보 노출·전체 정확도 등 상대 결과가 미정이면 기호를 유보한다.
