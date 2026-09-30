# 구조 비교안의 ASR·QA 해석 규칙

> 정성 설계 비교 계약 · 구현·measurement freeze·점수·winner 없음. 기존 QA 정의 변경 없음.

## 유지하는 현재 의미

규범 정의는 [Core ASR 계약](../../08-quality-attributes/core-asr-contract.md)이다. 설계 우선순위 QA-19 → QA-09 → QA-29 → QA-39를 유지하되 낮은 우선순위를 생략하지 않는다. 각 문서의 표는 구조에서 예상되는 장단점과 검증할 조건을 적은 것이며 측정 결과가 아니다.

| 축 | 현재 의미 | 이번 비교에서 금지할 대체 주장 |
| --- | --- | --- |
| QA-19 정확성 | 중복 없는 applicable QA-11/12 oracle field의 micro-average | 검색 recall·ASR 전사율·workflow 성공률을 전체 의미 정확도로 대체 |
| QA-09 응답성 | applicable QA-01/02/03/05 trial의 VIA 책임시간 산술평균 | Agent 작업시간을 VIA 지연으로 계산, Voice payload 생성으로 audible 끝점 대체, QA-04 합산 |
| QA-29 변경 용이성 | applicable QA-21~23 변화 집합에서 changed Architecture Element 수의 평균 | 박스 수·코드 줄 수·일반적인 “분리했으니 좋음”으로 점수 확정 |
| QA-39 신뢰성·복구성 | containment·올바른 복구·deadline·중복 방지·evidence 조건을 모두 충족한 fault trial 비율 | process 재기동이나 replay 완료만으로 복구 성공 선언 |

## 현재 네 축의 적용 제안

P=`PRIMARY`: 직접 구조 인과가 있어 차이 가설을 비교한다. R=`REGRESSION_ONLY`: 비교 구조가 직접 바꾸지 않는 공통 경로의 기능 유지 확인이다. 구조에서 직접 악화되는 인과도 P로 분류한다. `NOT_APPLICABLE`은 물리적 참여가 없는 경로, `UNRESOLVED`는 적용 계약이 아직 미정이라는 뜻이다. 한 후보의 전체 P가 모든 UC·모든 field의 참여를 의미하지 않는다.

| 상세 후보 | QA-19 | QA-09 | QA-29 | QA-39 | 범위 제한 |
| --- | --- | --- | --- | --- | --- |
| 의미 검색 서브시스템 | P | P | P | P | 검색을 거치는 referent·Context·handling 경로. 자료 조회 없는 일반 대화의 정확성 개선 주장 없음 |
| 내구 요청 workflow | P | P | P | P | 재개·질문 association·처리 결과·진행 경로. 의미 모델 자체의 해석 능력 향상 주장 없음 |
| 음성 근거 실행 구성 | P | P | P | P | Voice 입력과 해당 fault/change. Text-only 경로를 인식 개선 모집단으로 넣지 않음 |
| 권위 복구 저장 | R | P | P | P | 정상 의미 개선 없음. commit 비용과 실제 복구·migration 범위만 직접 비교 |

향후 실제 모집단·machine contract·targets·score bands·반복 횟수·dependency profile은 결과 전에 별도 승인된 작업으로 freeze한다. 동일 기능·허용 자료·사용자 목표·외부 capability·완료 조건을 유지한다. 한 축이 나빠도 다른 applicable 축의 결과를 숨기지 않고, 합성 가중 점수로 trade-off를 지우지 않는다.

## 메모리와 추가 QA

| 관점 | 이번 문서에서 검토할 실체 | 지위·남은 결정 |
| --- | --- | --- |
| QA-41 target-device memory | 공통 Omni weights/encoder/Talker·session KV, VIA Core, ASR/helper, worker heap, index resident pages, queue, replay peak. 저장 파일 크기와 RAM 구분 | 중요 ASR 후보로 계속 검토. 현재 diagnostic 지위 유지. target device budget·workload·상한과 점수/qualification 중 어느 역할인지 아직 미정 |
| QA-04 interruption | 모델 busy·helper 포화·재시작 중 local stop 유지 | Voice regression. QA-09 합산 금지 |
| QA-13~15 및 QA-31/32 | association·상태/근거·복구 상세 실패의 설명 | core input/회귀 진단의 기존 의미 유지. 같은 사실 중복 가중 금지 |
| QA-51 및 Action/access | stale 후보·폐기된 질문·삭제 payload·승인 철회 사용 차단 | qualification 유지. 성능 이득으로 위반을 상쇄할 수 없음 |
| QA-61/62 | source/build/revision·execution trace·재현 | 운영 journal/workflow trace가 평가 replay나 실제 음성 receipt를 대신하지 않음 |
| 저장 용량·write amplification·유휴 CPU/전력 | 색인·workflow 원장·event history가 만드는 지속 비용 | 추가 설계 진단으로 표기. 새 QA ID·ASR·점수를 발명하지 않음 |

메모리를 빠뜨릴 수 없는 이유가 이번 구조에서 더 분명해졌다. 검색안은 helper weights와 index를 추가하고 음성안은 별도 ASR을 제거하며 workflow·replay안은 새로운 상태와 peak를 만든다. 그러나 제거한 helper의 메모리만 빼고 공유 runtime의 추가 workspace·KV·queue를 무시해서는 안 된다. **ASR 승격 논의 근거를 확보했으며 승격 자체를 확정한 것은 아니다.**

## 대안별 흔한 오판 방지

- 색인이 있으면 최신·완전한 검색이 되는 것이 아니다. 원문 검증과 누락 처리 비용까지 포함한다.
- Workflow를 도입해야만 durable wait나 KV 해제가 생기는 것이 아니다. target에도 있다.
- Native speech가 별도 ASR보다 더 빠르거나 정확하다는 보장은 없다. 실제 timestamp/revision·동시 recognition 지원 조건이 남는다.
- Event sourcing은 외부 ACK와 실제 청취 여부를 알아내지 못하며 잘못된 의미 판단을 replay로 고치지 못한다.
- 이번 구조 검토·SVG 렌더링은 VIA 성능 측정이나 실제 제품 검증이 아니다.
