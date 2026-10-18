# DP05 발표용 C 보완 확인

기본 QA01은 공통 목적 채택 함수를 쓰지만 A의 취득 원문·B의 record 원문·C의 catalog 입력이 달라 구조적 동점이 아니다. 작은 관측 시간차의 우열은 미확정이며 원값·누락 분모를 유지한다. C는 이 불확실성과 QA02 비적용을 공개하고, QA03의 VIA 가공 생략을 근거로 조건부 권고한다. C는 관측 QA04/05/06에서도 상대 열세를 보이지 않았다. 따라서 아래 실험은 상대 QA 열세 해결이 아니라 **조건 원문 누락의 기능 취약점과 반복 읽기의 절대 부담**을 보완한다. N-H·본문 충족을 QA02로 바꾸지 않는다.

## C1 선택 자료 prefetch

기존 E3에 이미 채택된 조건의 출처를 가리키는 `condition_source_refs`를 공통으로 추가한다. C/C1 모두 같은 source ID·revision·1–2행 참조를 받으며 평가용 정답이나 추천값은 받지 않는다. 허용 원문에 적힌 가격·날짜·조건은 정상 입력이다.

C1은 다음 위임 목적을 알기 전에 adopt 시점의 참조 원문을 실제 읽고 보존하며 현재 판본·권한을 재확인한 원문만 capsule에 붙인다. 새 의미 요약·domain 계획·수신자 private memory는 생성하지 않는다. 기존 C는 같은 참조를 받은 수신자가 필요한 원문을 선택해 읽을 수 있다. 양쪽은 같은 frozen Qwen·수신자·권한·본문 계약이다. 이 출처 metadata를 추가한 fresh C는 기존 BASIC 전체 C의 재실행이 아니다. 조건 출처 ref가 이미 제공되는 상황을 전제로 하며, 이를 새로 의미 해석·추적하여 생성하는 upstream 경로의 비용은 이번 추가 실험에서 측정하지 않았다. 따라서 C1을 모든 제품 흐름에 비용 없이 추가할 수 있다는 뜻이 아니다.

E3 × C/C1 × 3 = 6 episode의 VIA/receiver 전체 호출·token·비용·누락·원문과 최종 본문을 보존한다. 고정 본문 분모는 안마다 9개다. prefetch 준비는 QA05/RSS에 포함하며 현재 요청의 시간을 소급하지 않는다. E1 미사용 준비 및 정정·철회 경계는 별도 오프라인 기능 검사이며 실제 LLM 품질 성과로 쓰지 않는다.

## C2 읽기 결과 cache

C1과 독립이다. 기존 최종 C의 실제 모델 읽기 선택을 재생하고 source/revision/정확 range/access epoch/recipient 키로만 저장한다. 매 hit 현재 권한을 확인하며 정정·철회는 재사용을 막는다. 읽기 선택과 모델 입력은 같아야 한다. 12 episode × basic/C2 = 24회에서 디스크 read_to_string, hit, serving 시간, 보존량/RSS를 비교한다. 운영체제 물리 디스크 I/O나 Qwen token·호출 감소를 주장하지 않는다. 재생 응답은 새 LLM 정확도·지연 측정이 아니다.

공통 읽기 지원 수신자 조건이다. 읽기 없는 Agent에 대한 A/B의 범용 연동성은 측정하지 않는다. QA03은 목표에 연결된 RECEIVER HTTP send→return 합집합을 공제하며, queue/직렬화/VIA 원문 serving/결과 검사는 포함한다. 전체 wall time·수신자 비용도 별도로 보고한다.

## 완료 결과

- [발표용 해석과 실패·조건](presentation-report.md), [실측 집계](fresh-summary.json), [원문/단계/동일 완료8목표 분해](mechanism-breakdown.json).
- [실제 본문18 저자 검토](content-review.json), [prefix18 검토](prefix-review.json), [독립 mail6](../dp04/independent-dp05-prefetch-mail-review.json).
- C/C1 첫 요청9쌍은 같았으나 실제 출력이 달랐다. C의 첫 안내 조건 강화1건과 호스트 fact15>14 미표시1건을 보존했다. callback/호스트 종료·literal·본문 충족은 별개다.

## 기본 첫 응답 경계 재검토

[basic-prefix-boundary-audit.json](basic-prefix-boundary-audit.json)은 기존 최종 BASIC의 정상 첫 해석21호출/안 실제 요청을 읽기만 했다. 입력 합계는 A15,318/B41,002/C13,722token, 출력은3,720/3,828/3,690token이다. A/C 요청은15/21만 같고 B와 다른 안은0/21이다. B의 이미 취득한 record 원문은 임시 prefix view에 들어가며 A는 앞선 VIA 읽기를 후속 prefix에 제공한다. 단순 함수 위치 일치만으로 구조적 동점으로 넘어갈 수 없다. 이는 새 실행이나 이전 지연 수치의 재채점이 아니다.
