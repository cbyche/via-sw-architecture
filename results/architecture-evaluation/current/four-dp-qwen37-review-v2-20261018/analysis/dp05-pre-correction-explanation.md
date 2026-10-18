# DP05 교정 전 실행 경로 진단

이 자료는 교정 전 BASIC36의 실제 비용과 경로를 설명한다. A의 VIA_COMPOSER 원문을 후속 공통 근거로 연결하지 않은 누락이 확인되어, 이 결과를 강한 기본안 A의 최종 비교로 사용하지 않는다. 원시 결과와 모든 실패는 보존한다.

## 실제 분기와 비용

|안|VIA/수신자 비용 USD|모든 호출|VIA build 분기|수신자 직접/읽기|E1 미사용 생산|
|---|---:|---:|---|---:|---|
|A|0.0359644 / 0.0982780|88|{'READ_AND_PURPOSE_COMPOSE': 18}|{'COMPLETE_FROM_SUPPLIED_EVIDENCE': 13, 'READ_ORIGINAL_SOURCES': 5}|{'calls': 0, 'cost_usd': 0}|
|B|0.2240744 / 0.0781776|118|{'DIRECT_VALID_REFERENCES_OR_AVAILABLE_EVIDENCE_OR_RECORD': 18}|{'COMPLETE_FROM_SUPPLIED_EVIDENCE': 11, 'READ_ORIGINAL_SOURCES': 7}|{'calls': 9, 'cost_usd': 0.0178332}|
|C|0.0115320 / 0.0626724|59|{}|{'READ_ORIGINAL_SOURCES': 18}|{'calls': 0, 'cost_usd': 0}|

A는18회 모두 원문 읽기+목적별 compose를 탔고 직접 경로는0이었다. B는18회 모두 유효 기록에서 직접 인계해 compose를 생략했지만, 수신자가7회 추가 읽기를 선택했다. C는18회 모두 실제 수신자가 원문 읽기를 선택했다. 코드상 경로 존재와 실제 경로 사용을 구분한다.

B의 생산자는 이미 실제 원문을 읽어 work_record.supporting_passages로 보유한다. 새 교정은 이를 중복 영구 cache로 옮기지 않으며, prefix에 현재 유효한 raw만 임시 제공한다. A/B VIA_COMPOSER의 실제 원문 읽기는 읽기 직후 공통 원장으로 연결한다. C 수신자의 외부 읽기를 VIA의 기확보 의미 상태로 승격하지 않는다.

## 단계별 실제 호출/토큰/비용

|안|stage|호출|입력 token|출력 token|USD|
|---|---|---:|---:|---:|---:|
|A|dp05_compose_capsule|18|21882|3161|0.0138104|
|A|dp05_interpret_user_request|21|13722|3729|0.0114552|
|A|dp05_receiver_recover_read_selection|4|14927|807|0.0072620|
|A|dp05_receiver_repair_result|4|32662|4228|0.0198296|
|A|dp05_receiver_result|5|31373|5248|0.0209460|
|A|dp05_receiver_select_or_complete|18|37317|22071|0.0502404|
|A|dp05_select_handoff_details|18|13875|3218|0.0106988|
|B|dp05_extract_record|18|75208|9704|0.0456096|
|B|dp05_interpret_user_request|21|13722|3801|0.0115704|
|B|dp05_produce_work_record|51|230100|46784|0.1668944|
|B|dp05_receiver_recover_read_selection|3|10318|571|0.0050408|
|B|dp05_receiver_result|7|21838|10581|0.0256648|
|B|dp05_receiver_select_or_complete|18|43288|18848|0.0474720|
|C|dp05_interpret_user_request|21|13722|3777|0.0115320|
|C|dp05_receiver_repair_result|2|6383|1734|0.0053276|
|C|dp05_receiver_result|18|30816|20624|0.0453248|
|C|dp05_receiver_select_or_complete|18|19434|2654|0.0120200|

## 시간과 내용의 해석

|안|QA01 관측/예정|QA01 ms|QA03 완료/예정|QA03 조건부 ms|전체 end-to-end 조건부 ms|평균 동시 RSS peak MB|
|---|---:|---:|---:|---:|---:|---:|
|A|21/21|4298.005|17/21|11150.302|30521.615|46.282|
|B|21/21|4426.724|21/21|13183.894|38002.079|50.657|
|C|21/21|4459.561|19/21|4426.841|24611.637|44.779|

QA01은 실제 사용자 입력 이후 공통 목적/수신자/완료조건 해석을 거친 실제 처리안내 DISPLAY_PROXY다. 원래63개 안내의 모든 의미를 사람이 검증했다는 주장은 하지 않는다. 교정 후63개가 최종 prefix 검토 대상이다. QA03은 실제 input 시각부터 final 완료까지에서 해당 목표의 RECEIVER Qwen HTTP send_ns→terminal_ns 합집합만 공제하는 동결 API-boundary proxy다. 전송 전 local queue/slot/직렬화와 VIA serving/receipt는 처음부터 포함하며 logical shim ACCEPTED→RETURN 전체를 공제하지 않는다. 위 성공 평균에서 미완료를 성공으로 대체하지 않았다.

N-H는 A/B/C 모두18/18이지만 실제 문안 PASS는6/15/9이며 서로 다른 평가다. 더 긴 유지보수의 우선 규칙을 가격 동률 없이 적용한 오류나 납기 누락을 구조계약 통과가 입증하지 않는다. B 문안 품질 이점은 관측되었지만 한 모델/작은 합성 사례의 출력 변동과 구조적 생산 효과를 분리할 추가 근거가 필요하다.

QA04 교정 baseline 재적용은 별도 수용검사 후 집계하며, QA06은 Python bridge+Rust 동시 RSS 관측치로 cloud 모델 가중치/UI/실제품 메모리를 포함하지 않는다.

[단계/읽기범위/실패 JSON](dp05-pre-correction-breakdown.json), [54본문 의미검토](dp05-basic-content-review.md), [행번호 입력 표현 진단](dp05-e4-line-coordinate-audit.json).
