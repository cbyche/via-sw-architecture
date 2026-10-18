# DP05 인계: 구조와 실제 관측의 연결

## 현재 범위

최종 `basic-dp05-reuse-union-corrected-v2` 36회를 완료했다. 안별4사례×3회이며 예정 인계 본문은18개씩 총54개, 첫 처리안내는21개씩 총63개다. 54본문·63최초 안내의 고정분모를 유지했고, 번호표기9회는 별도 진단으로 분리한다. 합성 사례로 구성한 최소 Rust/실제 Qwen PoC이며 실사용자 연구가 아니다.

발표용 직관: **A는 지금 목적에 필요한 설명을 고르고, B는 활동 중 만들어 둔 설명에서 골라 넘기고, C는 수신자가 원문을 고르게 한다. 미리 만든 설명이 실제로 후속 일을 대체했는지, VIA의 절약이 수신자의 비용으로 옮겨간 것인지 확인해야 한다.**

후보는 evaluator-only 정답값·라벨을 받지 않았다. 허용 원문에 적힌 가격·날짜·조건은 정상 모델 입력으로 제공됐다. 사전 동결 rubric의 source values 표현을 실제 원문 사실 미제공이나 evaluator 정답 누설로 읽지 않는다.

## QA별 구조 → 예상 조건 → 실제 확인 경로

|QA|구조상 예상과 성립 조건|실제 관측·실패 해석|최종값|
|---|---|---|---|
|QA01 첫 응답|ABC 모두 raw 사용자 입력을 같은 의미 prefix로 해석한다. 인계 상세구성 이전의 안내여서 차이가 작을 수 있다. 이미 보유한 실제 raw가 prefix 입력 크기를 늘릴 수도 있다.|capture→실제 목적/수신자/완료조건 채택→처리안내 DISPLAY_PROXY. 실제63개 안내를 원 입력·대상·현재 판본과 읽어 올바름/미검증을 구분한다. 상세본문 실패를 첫 안내 오답으로 자동 전이하지 않는다.|A4,306.803ms(20/21), B4,421.473(21/21), C4,320.514(21/21). 실제 안내: A20PASS+1미표시, B/C21PASS.|
|QA02 지칭 대상 정확도|인계 목적/조건 계약은 referent-set QA02와 다른 문제다.|QA02 N/A. 별도 N-H18개/안의 구조·권한·판본 계약, 목적표현 진단, 실제54문안의 의미판정을 구분한다. N-H를 본문 정확성이라고 부르지 않는다.|N-H A17/18, B18/18, C18/18. 별도 본문 full-PASS A8/18, B15/18, C12/18.|
|QA03 VIA 처리시간|A의 기확보 원문/유효 참조나 B의 충분한 record는 새 compose를 생략한다. C는 세부 설명의 추론을 외부 수신자로 옮긴다.|입력→실제 final 완료에서 해당목표에 연결된 RECEIVER Qwen HTTP send_ns→terminal_ns 구간의 합집합만 공제한다. 전송 전 queue/slot/직렬화, VIA source serving·receipt는 처음부터 포함한다. logical shim ACCEPTED→RETURN 전체를 공제하지 않는다. full elapsed와 미완료 분모도 같이 보고한다. 짧은 VIA 시간이 사용자 전체 대기 감소를 뜻하지 않는다.|A11,018.836ms(20/21), B12,438.588(19/21), C4,329.101(19/21). 전체 경과시간33,025.170/31,106.113/24,073.139ms. 관측 완료에 한한 평균.|
|QA04 변경 용이성|실제 상태 소유권은 section update와 부분범위 철회 패치에서 드러난다.|최종 baseline에 C01/C03 및 C04/C06을 독립 재적용했다. controls1/2, C04 A/B/C1/1/1, C06 2/3/2. 기능 수용검사에서 철회된 원문·ancillary 문구의 후속 payload 재주입도 검사했다.|4과제 평균 A1.50/B1.75/C1.50모듈. 기능2개 평균1.5/2/1.5. tie 보존.|
|QA05 전체 모델 비용|A의 목적별 선택·필요한 compose, B의 목적 미상 지속생산·추출, C의 prefix·수신자 읽기/작성 비용이 다르다.|실제 stage별 calls/input-output tokens/USD를 VIA와 RECEIVER로 나누고 합산한다. 교체/미사용 준비, E1 무인계 생산도 제외하지 않는다. C에도 실제 prefix 비용이 있다.|VIA+수신자 합계 A$0.1018408, B$0.2886828, C$0.0757928. 실제 호출80/116/59회.|
|QA06 실제 RAM|A common 원문, B record와 supporting raw, C manifest/수신자 read의 수명이 다르다. bridge의 실제 요청·응답·trace 보관이 차이를 압도할 수 있다.|같은 시각 Python+Rust RSS 합계와 각 역할 기여, 별도 source/common/record/prefix/capsule 논리 bytes를 구분한다. 논리 byte를 RSS로 부르지 않는다.|동시 관측 합계 peak A49.119/B57.229/C48.153MB. Python45.351/52.527/44.483 + Rust3.768/4.702/3.670MB.|

## 실제 시간 경계

`DOWNSTREAM_SHIM_ACCEPTED`/`RETURN`은 같은-process 논리 수신자의 접수·반환을 기록하는 lifecycle 사건이다. QA03의 수치 공제 경계와 다르다. 실제 동결 `timing.py`는 role=RECEIVER이고 causal goal_ids에 해당 목표가 연결된 호출의 HTTP `send_ns`→`terminal_ns` 구간을 수집해, 입력부터 final 완료 사이와 교차한 합집합 길이만 뺀다.

따라서 전송 이전 local queue/slot 대기·payload 직렬화, 원문 serving, assistant content JSON 파싱/검증/receipt/표시는 VIA 시간에 남는다. bridge.py의 send_ns는 urllib Request 구성 직전, terminal_ns는 응답 외피 json.load 이후의 client timestamp다. 작은 Request 구성·응답 외피 파싱도 이 API proxy 구간에 포함되므로 물리 패킷 송수신이나 순수 cloud 연산시간이라고 부르지 않는다. serving을 먼저 뺐다가 별도 복원하는 계산이 아니다. HTTP 내부의 네트워크와 provider 연산은 추정 분리하지 않는다. 원시 필드명 `receiver_shim_internal_excluded_ms`가 남아 있어도 실제 값은 이 HTTP 합집합이다. [실행 동결 analyzer](../basic-dp05-reuse-union-corrected-v2/frozen/source/benchmark/architecture/four_dp_poc_v2/timing.py).

## 실제 재사용과 근거 소유권

- A/B의 VIA_COMPOSER 실제 성공 원문 읽기는 읽기 직후 common 원장에 연결한다. compose/capsule/수신자 실패가 뒤에서 생겨도 읽은 원문은 남는다. 현재 권한·판본·범위 검사를 매 사용 전에 적용한다.
- B 생산자가 실제 읽어 host가 붙인 supporting_passages는 기존 work_record 소유로 유지한다. 공통 prefix에는 그 현재 raw만 임시 canonical view로 제공하며 설명/조건 자유문구는 섞지 않는다. 중복 영구 cache를 만들지 않는다.
- A/B의 build는 유효 원문/참조/record가 충분하면 DIRECT로 인계하고 부족하면 실제 bounded read 후 compose한다. 경로가 있다는 설명과 실제로 탄 횟수를 최종 trace로 따로 제시한다.
- C의 수신자가 선택해 읽은 원문은 외부 receiver evidence/receipt이며 VIA common으로 자동 승격하지 않는다.
- 모든 안의 같은 수신자는 충분한 실제 근거가 있으면 즉시 작성하고, 부족하면 필요한 범위를 직접 선택해 읽은 뒤 작성한다. manifest/제목/근거 없는 capsule 요약은 사실의 대용물이 아니다.

이번 공통조건은 **원문 읽기를 지원하는 같은 수신자**다. A/B도 최초 인계자료가 부족하면 수신자가 권한 있는 원문 범위를 추가 선택·읽을 수 있다. 따라서 이번 비교는 이 조건에서 최초 인계자료 준비의 효과를 본다. 원설계의 A/B 최초 가공·선택과 C의 최초부터 수신자 상세선택 차이는 유지하지만, 읽기 없는 Agent에 대한 A/B의 범용 연동 장점은 이번에 측정하지 않았다. 이를 모든 Agent profile에 대한 우열로 일반화하지 않는다.

`SOURCE_READ`는 실제 파일 읽기 호출이다. 내부 read_to_string은 전체파일을 읽을 수 있고 `HANDOFF_SOURCE_READ`는 그중 실제 공급한 범위다. 물리 디스크 I/O 또는 OS cache miss를 측정했다고 표현하지 않는다.

B의 생산은 동결 합성 event 순서에서 다음 사용자 입력 이전에 수행된다. 생산 중 사용자 입력 도착과의 경쟁·공유 스케줄링·실제품 overlap을 검증한 것은 아니다. 준비비용과 후속 실제 작업 대체를 함께 해석한다.

## 본문·안내 검토와 계약 한계

[동결 의미 rubric](../basic-dp05-reuse-union-corrected-v2/frozen/evaluator/dp05-semantic-review-rubric.md)으로 실제 GOAL_COMPLETE.text만 읽는다. lexical/equivalence/manual 의미판정을 분리하고 미표시도54개 고정분모에 남긴다. 동일 case/원문 조건과 실제본문 SHA가 같을 때만 이전 근거를 연결한다. 독립 검토는 사전 선택27개이며 저자54개와 구분한다.

host의 현재 원문 span·정확 quote 검사는 근거의 실제 공급 위치를 확인하지만 모든 본문 사실/조건 논리를 증명하지 않는다. 현재 교정은 집계 source_spans에서 실제 공급한 동일원본·판본·단위의 빈틈없는 인접/겹침 범위를 허용한다. fact별 quote/currentness 검사는 그대로다. 의미오답과 표시 계약 실패를 별도 분류한다.

## 이전 결과 보존

1. 원래 BASIC36/54: [교정 전 경로·비용 설명](dp05-pre-correction-explanation.md), [54본문 원시 대조](dp05-basic-content-review.json). A의 기확보 원문 연결 누락을 포함하므로 강한 기본안 A의 최종 비교로 사용하지 않는다.
2. retention 교정 중간11회/25미실행: [중단 목록](../basic-dp05-reuse-corrected-v1/PHASE-STOPPED.json), [중간 수동본문 검토](dp05-corrected-content-review.json), [인접 span 거절 감사](dp05-corrected-union-span-audit.md). 모든 비용/실패/판정은 보존한다.
3. 새 최종36회: [freeze](../basic-dp05-reuse-union-corrected-v2/freeze.json), [54본문 검토](dp05-reuse-union-corrected-v2-content-review.json), [63안내 검토](dp05-reuse-union-corrected-v2-prefix-review.json), [실제 raw/retention/분기 연결](dp05-reuse-union-corrected-v2-mechanism-review.json).

최종 source/binary·QA04 수용근거는 [준비 요약](../correction-preparation/dp05-span-union/readiness-summary.json)에 있다. 원격 모델 대기·출력길이·복구 변동을 구조의 보편적 인과로 단정하지 않는다. 이 PoC는 실제 상용 Agent/harness, UI 물리 표시, 오디오, Windows 또는 on-device Omni 검증이 아니다.


## 최종 고정분모와 내용 진단

|안|QA01 관측/예정|host 완료/예정(CLOSE 포함)|N-H 인계 계약|literal / 단위·날짜동치 / 사람 full-PASS(각18)|본문 나머지|
|---|---:|---:|---:|---|---|
|A|20/21|20/21|17/18|7 / 7 / 8|FAIL5, PARTIAL3, UNRESOLVED1, 미표시1|
|B|21/21|19/21|18/18|9 / 11 / 15|FAIL1, 미표시2|
|C|21/21|19/21|18/18|8 / 8 / 12|FAIL2, UNRESOLVED2, 미표시2|

목표는 안별21개(종료3+인계18)이며 본문 의미는 인계18개만 센다. N-H는 전달한 목적·조건·권한·판본·행동 상태의 계약 정합성이다. 계약 통과는 실제 문안의 조건 논리나 필수 내용 충족과 다르다. literal은 제한된 marker, 동치는 사전 고정 단위·영어 날짜 표기 진단이며 사람 판정으로 소급 교체하지 않았다. 실제 표시49/54를 읽었고 나머지5도 고정분모에서 미표시로 유지했다.

최초 안내는 원 입력의 목적·완료조건, recipient/대상/현재 판본, handling의 추가 사실주장을63개 예정 항목에 대해 확인했다. A E3 r2 메일은 recipient에 허용 adapter 대신 인간 수신자 '구매 담당자'를 두 번 내어 첫 안내·인계·본문이 모두 없었다. 나머지62개 실제 안내에서 알려진 잘못된 대상/현재 사실은 발견하지 않았다. 이 확인은 최종본문 의미까지 옳았다는 뜻이 아니다. QA01은 실제 UI 물리 표시가 아닌 비스트리밍 callback proxy다.

[저자54개 근거](dp05-reuse-union-corrected-v2-content-review.md), [63개 안내 근거](dp05-reuse-union-corrected-v2-prefix-review.md), [독립 사전선정27개](dp05-final-independent-content-review.md)를 분리했다. 독립27은 PASS14/FAIL10/미표시3이며 추가 경계2는 UNRESOLVED로 별도다. A E4 정정3개는 필수 예산누락에 동의하지만 저자 PARTIAL/독립 FAIL 라벨을 유지한다. full-PASS에는 영향이 없다. 이전 동일본문 노출1건·반복출력 친숙도와 대안 정체성 인지를 공개하며 블라인드 사용자 연구라고 부르지 않는다.

## 실제 호출·토큰·비용: 준비를 포함한 전체

|안/단계|호출|입력 토큰|출력 토큰|USD|
|---|---:|---:|---:|---:|
|A VIA 요청·복구|51|46,542|9,548|0.0338936|
|A 외부 수신자|29|71,228|24,660|0.0679472|
|B 사전 지속생산|51|189,001|46,072|0.1493156|
|B VIA 요청·추출|39|98,338|12,820|0.0598472|
|B 외부 수신자|26|89,248|27,388|0.0795200|
|C VIA 사용자 해석|21|13,722|3,690|0.0113928|
|C 외부 수신자|38|62,456|24,636|0.0644000|

A는 사용자 해석21+해석복구1+인계 선택17+compose12=VIA51회다. B는 사용자 해석21+record 추출18=요청39회 외에 생산51회가 있다. C의 VIA 비용은0이 아니며 공통 사용자 해석21회를 포함한다. 수신자 비용도 모두 합산했다. 수신자 초기 선택은 A17/B18/C18회, 실제 추가 작성은7/4/18회이고 나머지는 복구 호출이다. 모든 HTTP가 COMPLETED여도 본문/목표 완료가 된 것은 아니다. C 비용의 약85.0%가 수신자에 있고 A는66.7%, B는27.5%다. 다만 실제 수신자 절대비용은 C$0.0644000으로 A$0.0679472/B$0.0795200보다 낮았다. C의 수신자 호출수가 많다는 사실을 곧바로 수신자 비용도 더 크다는 주장으로 바꾸지 않는다. 입력 토큰·수정/복구·실제 출력 길이가 함께 다르다.

B 생산51창의 실제 합계814,739.014ms는 준비 구간 전체의 합이다. 다음 사용자 입력 이전에 완료하도록 구성한 합성 순서이므로 이 합을 입력 이후 latency에 더하거나 빼지 않는다. 무인계 E1에서만 생산9회/$0.0172844를 쓰고 실제 인계에는 전혀 사용하지 않았다. E1의 B 총비용$0.0189184는 종료 입력 해석도 포함한다. '미리 만들어서 요청 시 빠르다'만 보고 준비를 무료로 처리하지 않았다.

## 실제 재사용 분기와 조건 보존 사례

|안|VIA 실제 원문 파일 읽기|인계 build 실제 분기|수신자 즉시 작성 / 추가 원문 읽기|수신자 원문 파일 읽기|
|---|---:|---|---|---:|
|A|COMPOSER24|DIRECT5 / READ+COMPOSE12 (해석실패1은 build 없음)|10 / 7|13|
|B|RECORD_PRODUCER99|현재 record DIRECT18 / 새 compose0|14 / 4|11|
|C|0|계약+manifest 전달, 별도 상세 build 없음|0 / 18|34|

A DIRECT5는 E3의 실제 첫 목적에서 읽은 공개 비교/견적을 이후 다른 목적에 재사용한 경로다. 첫 목적3회는 READ+COMPOSE이고 후속6개 중 메일1개가 prefix에서 실패해 나머지5개에서 compose를 생략했다. [E3 r1 실제 후속 재사용](dp05-final-retention-case-e3-A-r1.md)에 첫 원문 read→retention→다음 prefix/build의 동일 raw→DIRECT를 연결했다. 따라서 이전 원본 누락 세대의 DIRECT0을 유지기능이 불필요하다는 결과로 사용하지 않는다.

B는 활동별 실제 생산 record에서 뽑아18개 인계를 모두 DIRECT로 구성했다. 하지만 추출 모델18회는 남고, 수신자가14개에서만 즉시 작성했으며4개는 원문을 추가 읽었다. 사전생산이 새 compose를 대체했으나 '후속 모델 호출 전부를 없앴다'는 뜻이 아니다. B99번의 생산자 원문 읽기는 현재 registry를 재생산할 때 실제 파일을 읽은 비용이다.

가장 분명한 조건 보존 사례는 동일 E3 r3 메일이다. B의 생산 record는 delivered-guidance v1 1–3행을 supporting_passages로 유지했다. 추출이 그 record block과 원문을 선택해 capsule로 전달했고, 수신자는 '가격이 동일할 경우'를 본문에 보존했다. A는 같은 가격·기간·납기 원문을 이미 보유하여 DIRECT했지만 guidance를 선택하지 않았고, 선택 summary에서 이미 '더 비싸도 A 선호'로 잘못 바꾼 뒤 실제 메일에 '가격 차이가 있을 경우'라고 조건을 반전했다. [실제 record→capsule→본문 증거](dp05-final-condition-retention-paths.json).

이는 구조가 어떤 정보를 다음 추론에 남겼는지 보여주는 실제 경로다. B의 일부 summary 자체는 동률 전제를 축약했으며 정확한 원문이 함께 공급돼 수신자가 복원했다. B E2 r3도 같은 조건을 잘못 적용해 FAIL이었다. 따라서 지속 record가 의미 정확성을 보장하거나 전체15/18 차이의 단독 인과라고 단정하지 않는다. 원문 선택과 모델 출력 변동이 함께 작용한다.

## 사례별 관측값과 실패

|사례|A QA01 / VIA QA03 ms|B QA01 / VIA QA03 ms|C QA01 / VIA QA03 ms|실제 의미·분모 해석|
|---|---|---|---|---|
|E1 인계 없이 종료|2,787.773 / 2,787.899|3,008.567 / 3,008.680|3,127.094 / 3,127.189|모두 종료3회. B의 생산은 미사용 비용으로 남음.|
|E2 단일 인계|4,452.186 / 12,884.525|4,067.204 / 14,836.424|4,436.075 / 4,470.086|본문 full-PASS A1/3, B2/3, C3/3. 조건 논리 오류와4문장/수치 통과를 구분.|
|E3 세 목적|4,357.408 / 11,465.413|4,442.120 / 14,729.983|4,254.016 / 4,302.044|본문 full-PASS A4/9, B9/9, C6/9. A 메일 해석실패1, 조건오류/모호성 별도.|
|E4 큰 문서·정정·철회|4,926.154 / 13,606.023|5,274.090 / 12,557.003|4,959.192 / 5,185.675|본문 full-PASS A3/6, B4/6, C3/6. B/C 각각2개의 실제 quote 행 불일치로 미표시. A 정정3개는 예산조건 누락.|

각 시간은 실제 관측한 callback/host 완료 평균이며 미완료는 성공 평균으로 대체하지 않았다. [실패 callback 정본](../basic-dp05-reuse-union-corrected-v2/all-failure-callbacks.csv): A 해석실패8,572.417ms, B 인용 실패63,337.799/82,751.110ms, C 인용 실패38,173.409/39,971.395ms. B/C의 인용문은 실제 입력에 있었지만 지정한 행이 다른 문장을 가리켰다. 최종 집계 span의 연속 합집합 거절 문제와 구별한다. [개별 현재 quote·실제 공급·행 위치](dp05-final-provenance-failures.json).

E4 정정 후에는 v1 raw와 철회된 비공개 자료를 현재 근거로 재사용하지 않도록 걸렀고 현재 v2만 전달했다. 세부 source 범위 철회는 이 BASIC event가 아니라 QA04 C06 독립 수용시험에서 검증했다. 그 시험은 block뿐 아니라 ancillary 문자열과 실제 후속 payload의 철회 marker 재주입 부재도 확인했다. 이 제한된 host 검사가 모든 실제품 기억/권한 경로를 검증한 것은 아니다.

## RAM과 선택 조건

메모리 최대 동시 표본은 A49.119/B57.229/C48.153MB이며 같은 표본 Python/Rust 기여는 위 QA06행과 같다. 각각 fresh-run 초기 관측 합계 대비 증가는14.287/22.610/13.353MB다. 이는 steady baseline이나 기능별 순수 메모리 차이가 아니다. B는 실제 record/supporting raw와 더 큰 요청·응답이 존재하지만 trace/call 보관과 Python heap도 함께 늘었다. 같은 표본의 논리 직렬화 bytes는 RSS 분할이 아니며 단순 합산해 원인분해하지 않는다. 클라우드 가중치·제품 UI·외부 orchestrator와 공유페이지 중복 제거는 포함되지 않는다. [동시표본·소유자 근거](dp05-final-numerical-summary.json).

이 읽기지원 수신자 profile에서는 C가 VIA 및 전체 관측 대기·총비용을 줄였고, A는 후속 목적의 실제 원문 재사용으로 compose를 생략했다. B는 가장 높은 비용/RAM과 느린 VIA 구간을 감수하면서 반복 인계 E3의 조건 원문 보존과 실제 문안 충족이 좋았다. **반복 인계에서 원문 조건 보존을 우선하고 준비비용을 감수할 상황에는 B를 조건부 권고할 근거가 있다.** 준비가 낭비되기 쉽고 수신자가 직접 읽을 수 있다면 C가 유리한 관측이다. A는 지속 설명생산 없이 지금 필요한 내용을 구성하되 같은 원문을 다른 목적에 재사용할 상황에 맞는다. 모든안이 일부 목표/내용 계약에서 실패했으므로 제품 배포 적격 또는 보편적 승자라는 결론은 아니다. 세번 반복·합성4사례·비스트리밍 Qwen 서비스 변동의 한계를 유지한다.

번호입력9회의18본문·가격조건/예산/행인용 진단은 별도 결과로 기록한다. 같은 binary·원문·검증기에서 전송 표현만 바꾸지만 실제 생성문안과 서비스 지연도 달라지므로 모든 개선을 행번호 포맷의 고립 인과로 단정하지 않는다. 원문 행번호 표현만 바꾼 성능을 이 기본36의 성공률로 소급 재계산하지 않는다.


## 별도 원문 행번호 입력 진단9회

실제 E4 원문을 전송할 때 원본 행번호를 붙인9회는 같은 실행 파일에서 별도로 수행했다. 최종 기본 E4의 host 완료는 A6/6·B4/6·C4/6, 번호표기 입력은 ABC각6/6이다. 기본 실패4건은 기존 결과에 남겼다.

|안|번호입력 QA01 ms|VIA QA03 ms|전체 경과 ms|전체 비용|호출|host 완료|저자 본문 full-PASS|
|---|---:|---:|---:|---:|---:|---:|---|
|A|4917.844|14363.581|42089.590|$0.0430084|26|6/6|6/6|
|B|5504.059|13255.524|30555.083|$0.1347904|36|6/6|6/6|
|C|5191.650|5243.135|27538.359|$0.0273428|18|6/6|5/6 + UNRESOLVED1|

[실제18문안·저자근거](dp05-numbered-lines-content-review.md)와 [독립18검토](dp05-numbered-lines-independent-content-review.md)를 분리한다. 저자17PASS+1UNRESOLVED, 독립18PASS이며 C r1초기 한 건은 동률 적용범위의 해석 차이다. 두 담당자가 합의한 성공18개라고 쓰지 않는다. 실제 무조건A추천이 없다는 사실에는 동의하나 full-PASS 여부에는 차이가 남았다. 원문 행번호 표현의 부담을 줄인 진단이며 출력문안/원격서비스 변동을 분리한 실험은 아니다. 기본값을 소급 교체하지 않는다.

번호 진단의 동시 RSS 약703MB는 기본36의49~57MB와 같은 출발조건의 후보 RAM 비교가 아니다. fresh Python worker가 실행 전 prerequisites에서 이전120/27/36의 대형 runs.jsonl을 전부 json.loads한 뒤 후보를 시작해, 초기 Python부터약694MB였다. A 최고 표본은 합계703.955MB(Python700.580+Rust3.375), 같은 실행 초기 합계695.861MB이고 formatter 논리 입력복사 약11.8KB다. 실행 전 대량 파싱은 [동결 worker38행→prerequisites12행](../diagnostic-dp05-numbered-lines/frozen/source/benchmark/architecture/four_dp_poc_v2/post_basic/line_number_diagnostic.py) 경로에서 확인했다. 정확한 allocator 기여를 profiling하지 않았으며 이 값을 포맷/후보 상태만의 RAM 증가로 귀속하지 않는다. 원시 수치와 역할/초기 표본은 보존하고 baseline36과의 동등 QA06 비교에서는 제외한다. 추가 수정/재실행으로 유리한 값을 만들지 않았다.

[진단 정본 수치](../diagnostic-dp05-numbered-lines/summary.json), [동시표본·초기값](../diagnostic-dp05-numbered-lines/memory-option-peaks.json), [실제 stage 호출](../diagnostic-dp05-numbered-lines/stage-calls.csv).
