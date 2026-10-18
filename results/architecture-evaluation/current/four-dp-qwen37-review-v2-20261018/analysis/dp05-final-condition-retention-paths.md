# DP05 실제 조건 원문 보존 경로

동일 E3 r3의 원문/사용자 목적/권한 조건에서 실제 API payload와 최종 표시본문을 대조한 사례다. 모델을 새로 호출하거나 결과/후보를 바꾸지 않았다. 전체 차이의 독립 인과 실험은 아니다.

|단계|B 15540f|A 0028b1|
|---|---|---|
|기확보 상태|활동별 생산 record에 guidance_criteria_and_omission block + 실제 delivered-guidance v1 1–3행 raw 보존|첫 목적에서 실제 읽은 comparison/quote raw2개 보존|
|목적별 선택|extract_record DIRECT가 guidance block/raw까지 선택|select_handoff_details DIRECT가 comparison/quote만 선택|
|capsule|조건 raw “가격이 같으면 유지보수 기간이 긴 제품을 우선” 실제 공급|동률 원문은 없음; selection.summary가 이미 “A preferred despite higher price”라고 잘못 축약|
|수신자 실제메일|“가격이 동일할 경우 ... 원칙”을 명시하며 현재 가격차/유지보수 사실 병기. PASS|“가격 차이가 있을 경우 ... Aster 추천”으로 조건 반전. FAIL|

B summary 자체가 모든 조건을 정확히 반복한 것은 아니다. 실제 supporting raw를 함께 넘겨 수신자가 원래 전제를 복원한 경로이며, 지속 설명생산이 본문 정확성을 보장한다고 주장하지 않는다.

E2/E3 초기 receiver payload 전수: B12인계 중11개에 실제 guidance raw가 있었고, A11개/C12개 초기 인계에는 없었다(A1개는 prefix 실패). B E3는9/9 raw공급·9/9본문PASS였다. 그러나 B E2r3는 raw가 있어도 조건FAIL이고 B E2r1은 초기 raw가 없어도 PASS였다. 따라서 정보 유지 구조의 작동 경로는 관측됐지만 raw 존재/대안명만으로 의미정확성이 결정되는 것은 아니다. C의 최초 자료는 manifest이며 공통 수신자의 후속 원문읽기 가능성과 구분한다.

[실제 요청 파일·SHA·raw·선택·응답 및 전수 공급목록](dp05-final-condition-retention-paths.json), [최종54개 판정](dp05-reuse-union-corrected-v2-content-review.json).

이 조건에서 반복인계·원문 조건 보존을 우선하고 사전생산/비용/RAM 증가를 감수한다면 B의 조건부 권고를 지지한다. 동시에 B의 실제 미표시2개·조건FAIL1개, 수신자의 추가읽기4개, 비동률 조건실패를 보존하므로 보편적 보장이나 제품 적격 판정은 아니다.
