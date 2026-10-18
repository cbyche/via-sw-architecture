# 최종 A E3 r1: 다른 목적에서 실제 원문 재사용

최종36회의 한 실제 기전 사례이며 전체 성능/품질 결론은 아니다. 중단11회 사례와 분리한다.

1. 발표개요 목적에서 VIA_COMPOSER가 public-comparison v1 1–4행과 public-quote v1 1–3행을 실제 읽었다.
2. 두 host 원문을 compose 전에 공통 원장에 보존했다. 의미요약/목적별capsule cache를 만든 것은 아니다.
3. 다음 메일·회의메모의 실제 prefix와 build에 현재 유효한2개 rawspan이 각각 전달됐다.
4. 모델이 두 다른목적 build에서 DIRECT를 선택해 새 VIA read/compose를 생략했다. 수신자의 판단·원문추가읽기·작성은 별도다.

|stage|호출|입력token|출력token|USD|
|---|---:|---:|---:|---:|
|dp05_compose_capsule|1|680|222|0.0006272|
|dp05_interpret_user_request|3|2538|539|0.0018776|
|dp05_receiver_result|1|1574|1167|0.0024968|
|dp05_receiver_select_or_complete|3|4511|2974|0.0065628|
|dp05_select_handoff_details|3|2886|770|0.0023864|

실제본문은 author기준 deck PASS/mail FAIL/meeting PASS였다. mail은 가격차가미미하다며 유지보수기간만으로Aster선호를정당화해동률전제를삭제했다. 다른두본문은 현재가격차가있어규칙이적용되지않는다고보존했다. 재사용경로작동과 의미정확성은 서로대체하지않는다.

원문범위는 실제공급구간이며 물리디스크I/O가 아니다. source()는 파일전체 read_to_string을 수행할수있다. 이 사례의 시간차를 고립된구조인과로 단정하지 않는다.

[원시trace](../basic-dp05-reuse-union-corrected-v2/raw/runs/basic_corrected-dp05-e3-A-r1-4ba071/trace.json), [모델실제입력/host원장연결](dp05-reuse-union-corrected-v2-mechanism-review.json), [최종본문검토](dp05-reuse-union-corrected-v2-content-review.json).
