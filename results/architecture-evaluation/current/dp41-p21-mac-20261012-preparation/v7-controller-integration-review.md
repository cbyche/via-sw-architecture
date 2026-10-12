# V7 공통 provider 장애 재실행 정책 독립 검수

검수 시점: 2026-10-12. 상태: READ_ONLY_INDEPENDENT_REVIEW. 후보 실행, 테스트 실행, API 호출은 하지 않았다. 구현자의 API0 회귀 보고와 저장된 V6 provider 원시 기록을 읽었다. 현재 범위에서 미해결 P1/P2는 확인하지 않았다.

## 적용 계약과 분류 경계

현재 [ASR-QA-v6 §3.5](../../../../docs/architecture/12-decisions/decision-packages/03-02-quality-attribute-definitions.md)는 수집기 결함과 공통 서비스 장애의 A/B 쌍을 INVALID로 보존하고 같은 조건에서 재실행한다. 후보 timeout, 미지원과 필수 trace 누락은 후보 실패로 보존한다. 이 검수는 재실행 정책의 구현 검수이며 후보 의미 정확성과 QA 점수 판정이 아니다.

[service_failure_calls](../../../../benchmark/architecture/dp41_45_poc/campaign.py)의 분류는 HTTP 429, 명시한 500/502/503/504, transport failure와 400 data_inspection_failed를 다룬다. 다른 400 또는 분류되지 않은 usage 누락은 감사 중단한다. HTTP200의 알려진 usage가 있는 모델 JSON/Frame 오류와 전송 전 취소의 알려진 0 usage는 INVALID 경로로 바꾸지 않는다. 임의 5xx를 무조건 재실행하는 구현은 아니며 명시 목록 밖 응답은 감사가 필요하다.

400 data_inspection_failed는 분류 후보일 뿐이다. verify_input_filter_transience가 그 개별 원본 Context를 재전송하여 원본 API body 값 전체가 같은 HTTP200 및 알려진 usage/cost를 확인한 후에만 쌍을 INVALID로 옮긴다. 지속적인 거절은 감사를 요구한다. 건강 확인의 출력은 후보 meaning에 채택되지 않으며 SERVICE_HEALTH 목적과 별도 원장으로 저장된다. ModelClient가 사용하는 model, temperature, thinking, response_format은 고정 adapter 값이고, 최종 승인에는 원본 body 전체의 동등성 검사를 다시 적용한다. 모델 의미나 출력의 정답 여부는 건강 확인 조건이 아니다.

## 실제 V6 연결 근거

[V6 원본 거절 Context](v6-question-followup-preflight-B/model-contexts/8815576d-ba00-4dc8-b055-bfbf87f5de16.json)와 [동일 입력 건강 재전송 Context](v6-provider-inspection-identical-recheck/raw/model-contexts/1719935c-3bae-4d90-80eb-e097de33234e.json)는 각각 19,070 byte이며, 파일 SHA-256도 모두 `3e7e7913679bc7ecb8bcc5fc37c29351a8e84edce1464f2f3f5743fcd368f6ac`이다. messages뿐 아니라 model=qwen3.6-35b-a3b, temperature=0, max_tokens=4096, enable_thinking=false, response_format=json_object도 동일하다.

원본 call `8815576d-ba00-4dc8-b055-bfbf87f5de16`, case S6-01, Request `R:b228da21-e46a-4060-a7b8-8c07d7a91c0a`는 HTTP400 data_inspection_failed, usage/cost unknown이다. 재전송 call `1719935c-3bae-4d90-80eb-e097de33234e`는 HTTP200, input 4668/output 446 token, $0.002754로 기록됐다. [원본 model-calls](v6-question-followup-preflight-B/model-calls.jsonl), [재전송 model-calls](v6-provider-inspection-identical-recheck/raw/model-calls.jsonl)와 [summary](v6-provider-inspection-identical-recheck/summary.json)는 함께 보존되어 있다. 이 증거는 서비스 응답의 일시성에 관한 증거이며 후보 B가 재전송의 의미를 채택했다는 증거가 아니다.

## 쌍 보존과 회귀 검토

run_workers는 각 반복의 양안 전체 활동을 완료한 뒤 분류한다. 장애가 확인되면 양안 worker 디렉터리 전체와 로그를 invalid-worker-attempts 아래 새 attempt로 이동하고 파일 hash와 원래 위치를 invalid-pair.json에 남긴다. 성공한 상대안도 같이 보존한 뒤 같은 frozen-source와 pair 순서로 두 안을 새로 실행한다. accepted 다섯 쌍만 workers에 남긴다. attempt 상한은 반복 쌍마다 세 번이며 기존 invalid 원장을 센다. 새 worker 실행 전에 frozen-source 전체 hash를 확인하고 부분 worker는 덮어쓰지 않고 감사를 요구한다.

[test_campaign.py](../../../../benchmark/architecture/dp41_45_poc/tests/test_campaign.py)의 7개 회귀를 읽었다. parameter 거절, 모델 schema 오류, queue 취소, 개별 Context 건강 확인, 지속 거절 및 전체 쌍 보존을 검사한다. 쌍 회귀는 첫 A1의 429와 정상 B1을 모두 보존한 뒤 두 안을 다시 실행하며, 최종 10 workers와 다섯 PAIR_ACCEPTED를 확인한다. 구현자의 PASS/API0 보고는 독립 재실행으로 바꾸어 표기하지 않는다.

## 한계와 원시 비용

자동 분류가 없는 provider 응답은 점수에 넣거나 자동 INVALID 처리하지 않고 감사해야 한다. 필수 trace 누락은 최종 평가의 qualification/coverage 실패로 다뤄야 하며, model-calls 파일 부재만으로 새 provider 장애를 만들어서는 안 된다. invalid 쌍과 건강 확인의 실제 호출/알려지지 않은 청구는 전체 준비 비용과 $100 budget 원장에 계속 남는다. accepted 모집단의 QA04와 별도로 공개해야 하며 unknown 비용을 0으로 단정하지 않는다. 이 보고서는 향후 서비스 장애가 없을 것이라는 보장이 아니다.

## 최종 읽기 대상 SHA-256

| 파일 | SHA-256 |
| --- | --- |
| `benchmark/architecture/dp41_45_poc/campaign.py` | `b1ec4108b7a52bb2372dc2b4c37ed427c70c473baba9b9f32c93df8d3f545db1` |
| `benchmark/architecture/dp41_45_poc/tests/test_campaign.py` | `623dfdf90c3812e4efc41bf16ce61fab2a837322c5adc45c556c60dc1d1a1531` |
