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

## (+)·(-)·(0) 표기와 추가 QA의 읽는 법

| 표기 | 뜻 | 잘못 읽으면 안 되는 것 |
| --- | --- | --- |
| (+) / (-) | **그 행에 적힌 조건**에서 상대적으로 유리/불리할 설계 예상 | 실측 점수, 무조건적인 QA 우승, 다른 조건까지의 일반화 |
| (0) | 명시한 범위의 구조·기능이 공통이라 동등하게 보는 설계 예상 | 아직 모르거나 구현하지 않았다는 뜻, 실제 결과가 반드시 같다는 보장 |

같은 QA라도 반복 검색/첫 검색, 단순 요청/복잡한 대기, 정상 동작/특정 장애를 나누어 표시한다. 기호는 개수로 합산하거나 가중 점수로 변환하지 않는다. 한 행에서 다룬 조건 밖의 유불리가 미정이면 그 한계를 이유에 적는다. 예를 들어 workflow의 자연어 해석 (0)은 질문·재개 association 전체가 동등하다는 뜻이 아니다. 기존 PRIMARY/REGRESSION_ONLY 적용 제안은 유지한다. QA-29 기호는 코드 수정량·재사용량이 아니라 같은 기능을 수용할 때 바뀌는 Component·Interface·State·Runtime 범위에 붙인다. 한쪽이 필요한 자료를 갖지 못해 같은 기능을 만들 수 없는 문제는 변경 요소 수와 구분한다.

네 문서는 **공통 5가지(현재 ASR 4개 + 메모리)** → **DP별 추가 QA** → **상세 인과·검증 조건** 순서로 읽는다. 추가 항목에는 기존 QA의 구체적 진단 관점과 아직 정식 ID가 없는 품질 후보가 함께 있다. 추가 QA 후보는 단순 운영 기능 목록이 아니라 구조에서 실제 비용·능력 차이가 생기는 항목만 고른다.

| DP | 추가로 보는 품질 | 기존 QA와의 관계 |
| --- | --- | --- |
| 자료 검색 | 요청 없는 동안 CPU·전력, 디스크 사용, 파생 자료 삭제 부담, 권한 철회 뒤 정보 노출 | CPU/전력·디스크는 추가 자원 효율 후보. 삭제 부담은 QA-51의 운영 관점이며 실제 노출 metric과 구분 |
| 요청 workflow | 멈춘 이유 찾기, 업데이트 중 대기 상태 호환, 반복 예외 시험의 재사용, 늦은 승인 실행 차단 | 조사 수고는 QA-61과 다른 진단. 호환/시험은 QA-29/39 설명 관점. Action/access 요구는 공통 |
| 음성 근거 | 계산·전력, 설치/배포 용량, 말 끊기, 입력 근거 진단의 관리 부담 | 전력·배포 용량은 추가 후보. QA-04 regression 유지. 진단 수고와 QA-61 전체 trace 완전성은 구분 |
| 복구 저장 | 디스크/쓰기, 과거 상태 변경 순서 설명, 삭제 정보의 부활 방지, 평가 결과 재현 | 삭제·추적은 QA-51/61 관련 진단. QA-62는 운영 replay와 별개 |

보안·근거 qualification은 비용이 크더라도 동일하게 충족해야 한다. 지울 사본이 많다는 이유로 실제 QA-51 노출이 더 많다고 단정하지 않고, 운영 이력이 풍부하다는 이유로 QA-61/62를 자동 통과했다고 하지 않는다. 기존 ASR과 같은 효과를 설명하는 추가 관점은 별도 점수로 중복 계산하지 않는다. **QA catalog·ASR 지위·metric·우선순위·측정 계약은 이번 편집으로 변경하지 않는다.**

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
