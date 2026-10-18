# 교정 A E3 r1: 다른 목적에서 실제 원문 재사용

전체36행 중 한 실제 기전 사례다. 전체 성능/품질 결론은 아니다.

1. 발표 개요 목적에서 VIA_COMPOSER가 public-comparison v1 1–4행과 public-quote v1 1–3행을 실제 읽었다.
2. 두 host 원문 반환이 compose 전에 COMMON_ORIGINAL_EVIDENCE_RETAINED에 기록됐다. 새 의미 요약/capsule cache로 저장한 것이 아니다.
3. 다음 메일·회의 메모의 실제 모델 요청에는 각2개 canonical raw span이 actual_common_evidence로 제공됐다.
4. 모델이 두 다른 목적의 build에서 DIRECT를 선택했고 host가 유효 원문 span을 capsule로 붙였다. 새 VIA source read/compose는 발생하지 않았다. 수신자의 개별 판단은 별도 호출로 남는다.

|stage|호출|입력token|출력token|USD|
|---|---:|---:|---:|---:|
|dp05_compose_capsule|1|684|233|0.0006464|
|dp05_interpret_user_request|3|2538|610|0.0019912|
|dp05_receiver_select_or_complete|3|4557|3207|0.0069540|
|dp05_select_handoff_details|3|2917|854|0.0025332|

이 사례의3개 실제 본문은 가격 동률 전제를 삭제한 현재 Aster 선호를 포함하여 author 의미검토 FAIL이다. 재사용 경로의 작동이 추천 조건의 의미 정확성을 보장하지 않는다. 첫 처리안내는 목적별 초안·기존조건·미발송 의도를 올바르게 표현하므로 후속 상세 오류와 구분했다.

HANDOFF_SOURCE_READ는 실제 공급 범위다. 내부 source()는 전체파일을 read_to_string하므로 이 수를 물리 디스크 I/O 또는 OS cache miss로 해석하지 않는다.

[원시 trace](../basic-dp05-reuse-corrected-v1/raw/runs/basic_corrected-dp05-e3-A-r1-f6a5a1/trace.json), [실제 요청/retention/분기 연결 JSON](dp05-corrected-mechanism-review.json), [고정54본문 검토](dp05-corrected-content-review.json).
