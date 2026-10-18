# 공통 계측과 DP02 후속 검토

새 API 호출, 후보 수정 또는 재실행 없이 기존 원시 자료에서 산출했다. 기존 QA 원값과 최종 산출물은 덮어쓰지 않았다.

## 1. 먼저 바로잡을 설명

사용자가 말한 구체적 수행 안내도 정식 QA01의 첫 응답이다. 정식 문서의 예시는 “매출표와 비용 그래프를 비교하겠습니다”이며, 위임 이전에 표시할 수 있다. 오답도 실제 내용 표시 시간으로 포함하지만 미해결 지칭의 되풀이와 빈 안내는 종료점이 아니다. [QA01 정의](/Users/changbae/Workspace/via-sw-architecture/docs/architecture/12-decisions/decision-packages/qa-candidate-package/qa01-first-response-latency.md:29); [미관측 처리](/Users/changbae/Workspace/via-sw-architecture/docs/architecture/12-decisions/decision-packages/qa-candidate-package/qa01-first-response-latency.md:72).

현재 01·02·04는 반환 내용을 처음 표시한 직후 같은 payload로 GOAL_COMPLETE를 기록한다. 앞선 구체적 수행 안내나 문장별 streaming 경로가 없다. 따라서 두 값이 거의 같은 것은 두 품질이 본질적으로 같아서가 아니라 이번 출력 경로가 붙어 있기 때문이다.

- [DP01 표시와 종료](/Users/changbae/Workspace/via-sw-architecture/prototypes/candidates/four_dp_poc/src/dp01.rs:96)
- [DP02 공통 display](/Users/changbae/Workspace/via-sw-architecture/prototypes/candidates/four_dp_poc/src/dp02.rs:10)
- [DP04 표시와 종료](/Users/changbae/Workspace/via-sw-architecture/prototypes/candidates/four_dp_poc/src/dp04.rs:76)
- DP05만 [구체적 purpose의 즉시 안내](/Users/changbae/Workspace/via-sw-architecture/prototypes/candidates/four_dp_poc/variants/dp05-receiver-v2.rs:85)와 [수신자 결과 뒤 종료](/Users/changbae/Workspace/via-sw-architecture/prototypes/candidates/four_dp_poc/variants/dp05-receiver-v2.rs:125)를 분리한다.

계산기는 [비어 있지 않은 문자열만 확인](/Users/changbae/Workspace/via-sw-architecture/benchmark/architecture/four_dp_poc/analyze.py:67)한다. 그 문자열이 실제로 채택된 대상·범위를 구체적으로 알리는지까지 검사하지 않는다. DP01 E3의 echo처럼 내용 단위 자격이 의심되는 응답은 별도 검토가 필요하다. 표시 콜백의 관측 시각은 남아 있지만, 존재하지 않았던 앞선 진행 안내의 시간을 사후에 만들 수는 없다.

정식 QA03은 사용자 목표의 VIA 상태·관계, 필요한 외부 사실, 필수 상세 텍스트와 음성 준비까지 포함한다. 오답 종료 시간은 그대로 보존하고 기능 오류를 별도로 기록한다. 이번 값은 Voice OFF, 실제 UI 프레임이 아닌 callback proxy이며, 완전한 제품 QA03 측정은 아니다. [QA03 완료 조건](/Users/changbae/Workspace/via-sw-architecture/docs/architecture/12-decisions/decision-packages/qa-candidate-package/qa03-via-processing-time.md:29); [필수 상세 내용과 음성](/Users/changbae/Workspace/via-sw-architecture/docs/architecture/12-decisions/decision-packages/qa-candidate-package/qa03-via-processing-time.md:42).

또한 정식 QA03은 Agent 알림 자체를 별도 사용자 목표로 세지 않는데, 현재 DP02는 Q-MAIL-1 표시를 STIMULUS와 GOAL_COMPLETE로 센다. 질문 표시 약 0.004ms인 세 건이 A/B 평균에 들어 있다. [정식 목표 구분](/Users/changbae/Workspace/via-sw-architecture/docs/architecture/12-decisions/decision-packages/qa-candidate-package/qa03-via-processing-time.md:38); [현재 질문 표시](/Users/changbae/Workspace/via-sw-architecture/prototypes/candidates/four_dp_poc/src/dp02.rs:29). 이 세 건만 제외한 사후 값은 A 27관측 13,102.269577ms / B 21관측 10,723.761129ms다. 이 값도 원래 목표 전체의 인과 그래프를 복원한 정식 QA03은 아니다.

### 발표에서 사용할 수 있는 것과 다시 측정할 것

|항목|현재 사용할 수 있는 사실|새 실험이 필요한 주장|
|---|---|---|
|QA01|현재 코드가 실제로 첫 callback을 낸 시간, nonempty/empty 건수. 내용 단위 자격은 별도 확인|해석 중 채택된 구체적 진행 안내를 먼저 보이는 제품 경로의 지연과 A/B 우열|
|QA03|현재 종료 선언까지의 관측 경로와 오류, Voice OFF proxy|사용자 목표를 고정한 인과 그래프, 필수 상세 내용과 음성 준비, 실제 UI 표시까지의 정식 제품 수치|
|실패|null 반환과 종료 시각, 누락률|없었던 성공 응답 시간, timeout으로 대체한 평균|
|QA04|아래 두 호환성 과제의 실제 Rust diff 수|여러 제품 기능 변경에서의 일반적인 유지보수 우열|
|메모리|기록된 두 PID의 동시 RSS와 PID별 값|확정된 Python/Rust 역할별 메모리 및 buffer/cache별 내역, 제품 전체 메모리|

## 2. DP02 네 시나리오의 관측값

시간은 비어 있지 않은 내용 callback의 조건부 평균이다. 네트워크 합은 모든 호출의 전송부터 반환까지의 합이므로 병렬인 경우 사용자 경과시간과 다르다. 토큰과 비용은 세 반복의 전체 실제 사용량이다.

|사례|안|내용 관측/예정|내용 callback 평균 ms|모델 호출|입력/출력 tokens|비용 USD|queue 합 ms|모델·전송 구간 합 ms|
|---|---|---:|---:|---:|---:|---:|---:|---:|
|02-E1|A|6/6|5888.989|6|19321/1673|0.0104052|10.971|35292.602|
|02-E1|B|6/6|8804.752|6|20372/2784|0.0126032|10.901|52786.902|
|02-E2|A|6/6|11738.600|6|19345/1695|0.0104500|13.699|35366.392|
|02-E2|B|6/6|6069.715|6|17847/1855|0.0101068|10.446|36376.717|
|02-E3|A|6/6|14168.683|6|19834/2186|0.0114312|11.145|42656.012|
|02-E3|B|6/6|20344.879|6|17847/3164|0.0122012|47863.487|61205.816|
|02-E4|A|12/12|13581.768|12|38925/3107|0.0205412|23.587|64698.787|
|02-E4|B|6/12|2313.290|12|36102/2597|0.0185960|25.323|59385.713|

- **E1 겹침 없음:** 같은 코드 분기를 탄다. 첫 R1 HTTP 요청 JSON은 A/B 세 쌍 모두 같다. 그 다음 U2는 각 실행에서 받은 R1 답변을 문맥에 넣으므로 실제 payload는 달랐다. 최초 호출 출력과 서비스 변동이 후속 문맥에도 전달된 관측 차이다. 구조적 우위로 설명할 수 없다. [같은 직렬 분기](/Users/changbae/Workspace/via-sw-architecture/prototypes/candidates/four_dp_poc/src/dp02.rs:16).
- **E2 겹침, K=2:** A는 R1 반환 뒤 U2를 판단하고, B는 두 판단을 겹친다. B의 실제 전송 구간 겹침은 2,529.548 / 3,701.185 / 4,841.585ms였고 A는 모두 0ms다. A는 R1이 준비되어도 U2의 관계·답변 호출이 끝날 때까지 약 2.8초 더 기다렸다. 반환문을 한 호출에서 함께 생성하기 때문에 관계만 먼저 얻어 게시할 수 없다. [A step 경계와 문맥 선택](/Users/changbae/Workspace/via-sw-architecture/prototypes/candidates/four_dp_poc/src/dp02.rs:35); [통합 관계/답변 호출](/Users/changbae/Workspace/via-sw-architecture/prototypes/candidates/four_dp_poc/src/dp02.rs:6).
- **E3 같은 겹침, K=1:** B가 먼저 enqueue한 U2는 실제 슬롯을 얻기 위해 기다렸다. B의 세 반복 queue 합은 47,863.487ms, A는 11.145ms다. 그러나 A는 enqueue 전에 step join으로 기다리므로 이 queue 차액을 B의 추가 구조 비용으로 해석하면 안 된다. B의 R1은 모델에서 준비된 뒤에도 U2 관계 해소까지 4,341.093 / 4,027.961 / 4,613.014ms 더 기다렸다. 다만 B의 실제 출력 토큰도 3,164로 A의 2,186보다 많다. 차이 전체를 queue나 구조 하나에 귀속할 수 없다. [공통 호출 한도](/Users/changbae/Workspace/via-sw-architecture/benchmark/architecture/four_dp_poc/run.py:38); [stale 게시 방지 관문](/Users/changbae/Workspace/via-sw-architecture/prototypes/candidates/four_dp_poc/src/dp02.rs:27).
- **E4 정정·Agent 질문/답변:** B의 빠른 2,313.290ms는 예정 12건 중 내용이 있는 6건만의 평균이다. 이 중 세 건은 거의 즉시 표시한 Agent 질문이다. U2와 수정된 R1의 여섯 null은 이 평균에서 응답으로 세지 않았다. 따라서 전체 완료가 빠르다는 뜻이 아니다.

A/B는 모두 30회 모델 호출이다. 전체 비용은 A $0.0528276 / B $0.0535072다. 같은 호출 수가 같은 토큰 수나 같은 경과시간을 뜻하지 않는다.

## 3. B의 null 여섯 건은 언제 돌아왔나

아래는 성공 응답 시간이 아니라 **유효한 내용이 없었던 반환 시도**의 실제 종료 시각이다. R1 rev2의 첫 입력 기준에는 오래 수행한 원래 R1의 대기도 들어 있다.

|반복|반환|해당 모델 attempt enqueue→terminal ms|최초 해당 입력→null callback ms|U2 정정 입력→null callback ms|
|---|---|---:|---:|---:|
|1|U2 rev1|2067.284708|2112.408541|2112.408541|
|1|R1 rev2|2144.530875|17866.732083|17740.416458|
|2|U2 rev1|2688.383792|2736.241625|2736.241625|
|2|R1 rev2|2072.349750|19512.589167|19382.481625|
|3|U2 rev1|1741.559500|1787.733750|1787.733750|
|3|R1 rev2|2029.468417|8217.279958|8087.252375|

여섯 반환의 HTTP/JSON 처리는 끝났고 실제 `answer:null`, `relation:CORRECTION`이 기록됐다. 모델 attempt 자체의 평균은 2,123.929507ms다. U2 입력부터 null까지 평균 2,212.127972ms, R1 최초 입력부터 수정된 null까지 평균 15,198.867069ms다. 원시 응답 경로와 시각은 `dp02-failed-attempts.csv`에 남겼다.

|집계|A|B|
|---|---:|---:|
|내용 있는 반환/예정|30/30|24/30|
|내용 있는 반환의 조건부 평균 ms|11791.961321|9383.158887|
|null 실패 반환 평균 ms|해당 없음|8705.497521|
|내용 유무를 묻지 않은 첫 callback의 운영 보조 평균 ms|11791.961321|9247.626614|

마지막 행은 빈 반환도 반환 시도로 세는 별도 운영 지표다. **QA01, QA03 또는 성공 완료시간으로 대체해서는 안 된다.** 현재 규칙은 표시가 없으면 NOT_OBSERVED이며 timeout 벌점이나 가짜 응답시간을 만들지 않는다. 실패 후 실행이 종료됐고 독립적인 고정 추적 기간이 없으므로, 여섯 건을 정상적인 우측 검열로 간주한 생존분석·RMST 역시 현재 자료만으로 정당화하기 어렵다. 응답률과 조건부 시간, 실패 반환시간을 나란히 보고하는 것이 가장 직접적이다.

### B를 보완하려면

현재 Python bridge는 JSON object인지 확인하고, Rust는 `value["answer"]`를 그대로 표시한다. 유효한 JSON이어도 null 응답을 막는 형식 검사가 없다. B 구조가 정정을 처리할 수 없다는 결과가 아니라 이번 제어/응답 계약의 실패다.

후속 후보는 다음과 같다. 이번에는 구현하거나 재실행하지 않았다.

1. 관계·정정 상태와 사용자에게 보일 응답의 반환 계약을 분리한다. CORRECTION이면 해석한 수정 범위와 대상 revision을 확정하고, 사용자에게 표시할 내용 또는 명시적 보류 사유를 반드시 갖게 한다.
2. null을 완료로 게시하지 않는 공통 검증을 둔다. 빠른 거짓 완료 대신 요청을 대기 상태로 남기고, 필요한 경우 제한된 복구 호출을 한다. 복구 비용과 시간을 모두 기록한다.
3. 수정된 R1 실행에 원래 조건, 실제 수정 범위와 현재 원본을 구조화해 전달한다. Task/Request 연관과 revision을 검증한 뒤 새 결과를 채택한다.
4. 구체적 진행 안내와 상세 결과를 별도 event로 표시한다. A/B 모두 같은 사용자 기능을 제공한 뒤 다시 QA01/03을 비교한다.
5. “계속하고, 별도로 질문”처럼 복수 의도가 있는 입력은 단일 relation label의 정확도 시험과 분리하거나 사전에 복수 해석 계약을 고정한다. 기존 정답을 결과에 맞춰 바꾸지는 않는다.

## 4. A1의 관측 범위

E2 K=2의 같은 세 반복이다. 모델 호출은 6→6, 입력 토큰 19,345→10,134, 출력 토큰 1,695→2,396, 비용 $0.0104500→$0.0078872다.

|입력|기본 A 평균 / 최대 ms|A1 평균 / 최대 ms|
|---|---:|---:|
|R1|11803.801528 / 17071.999417|11025.404431 / 14513.167667|
|U2|11673.397777 / 16941.494375|4081.305875 / 4670.532042|

R1 반복3은 8,739.828→14,513.168ms로 늦어졌다. U2의 짧고 도구 없는 독립 질문은 먼저 전달됐지만, 각 반복의 서비스·출력 변동과 원래 요청의 손익을 함께 봐야 한다. frozen N-R 3/6→6/6은 혼합 발화의 단일 label 진단이며 일반적인 정확도 이익은 아니다.

## 5. QA04의 1개·2개와 3개의 관계

|과제|실제로 바꾼 계약|운영 Rust 모듈|수|
|---|---|---|---:|
|C01|후보 Rust와 Python 실행기 사이 모델 호출 IPC: payload/result/usage를 request.context/provider_response.json/renamed usage로 변경|crate::model_wire|1|
|C03|입력 JSON의 version 객체, coordinate dimensions, captured_at microseconds를 기존 canonical 값으로 정규화|crate root(main), crate::source_contract|2|

평균은 (1+2)/2=1.5다. 두 과제에서 수정한 고유 모듈의 합집합은 세 개지만, 이것은 세 모듈짜리 세 번째 변경 과제가 아니다. 정식 규칙은 crate root의 연결 코드도 수정하면 한 모듈로 센다. [집계 규칙](/Users/changbae/Workspace/via-sw-architecture/docs/architecture/12-decisions/decision-packages/qa-candidate-package/qa04-change-scope.md:50).

C01은 실제 Qwen HTTP API 규격을 바꾼 실험이 아니다. 공통 bridge는 이미 두 IPC 형식을 받아 준다. [C01 실제 diff](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-20261018/qa04/C01.patch:5); [bridge 양 형식 처리](/Users/changbae/Workspace/via-sw-architecture/benchmark/architecture/four_dp_poc/run.py:42). C03도 실제 PC 원본 수집기 구현 변경이 아니라 작성 입력 JSON을 공통 admission에서 복구하는 호환성 과제다. [C03 실제 diff](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-20261018/qa04/C03.patch:7).

수는 실제 diff와 계약 실행으로 확인했지만, 모두 공통 adapter에 모이는 두 작은 과제다. 모든 안이 1.5로 동점이다. 정식 C01~C06 전체나 기능 추가의 유지보수 우위를 검증한 수치로 확대할 수 없다. 모델 의미 정확도를 재검증한 것도 아니다.

## 6. 메모리를 어디까지 나눌 수 있나

collector는 자기 Python PID와 생성한 Rust PID 두 개의 RSS를 요청했다. 그런데 raw에는 PID별 값만 기록했고 역할 이름과 PPID를 기록하지 않았다. PID 순서나 크기로 Python/Rust 역할을 사후 확정하지 않는다. [collector 대상과 기록](/Users/changbae/Workspace/via-sw-architecture/benchmark/architecture/four_dp_poc/run.py:17).

|안|최대 동시 RSS MB|그 peak에서 기록된 두 PID별 RSS MB|두 PID 동시 관측이 없는 실행 수|
|---|---:|---|---:|
|01A|39.239680|70189: 36.667392, 70193: 2.572288|0|
|01B|39.174144|64035: 36.634624, 64039: 2.539520|0|
|01C|39.976960|69047: 37.126144, 69051: 2.850816|0|
|02A|39.092224|77370: 36.601856, 77374: 2.490368|0|
|02B|39.190528|77656: 36.716544, 77662: 2.473984|0|
|04A|38.928384|82013: 36.454400, 82017: 2.473984|0|
|04B|39.632896|80424: 37.060608, 80428: 2.572288|0|
|05A|39.501824|93412: 36.585472, 93415: 2.916352|2|
|05B|40.878080|94169: 37.699584, 94172: 3.178496|0|
|05C|39.092224|98293: 36.339712, 98296: 2.752512|3|

이 값으로는 PID별 RSS와 두 프로세스의 합만 복원할 수 있다. Python 인터프리터, 실제 Rust 상태, JSON/문자열, 원본 읽기 buffer, 이미지 base64, 후보 목록, 유지 cache별 사용량은 기록하지 않았다. Python이 수집한 event/call 목록과 HTTP 요청 buffer도 합계 안에 섞여 있다. 외부 실행 controller와 브라우저, cloud model weights/KV는 측정 대상이 아니다. 공유 page 중복을 제거한 실제 물리 사용량이나 Windows commit 값도 아니다.

향후 역할별·buffer별 내역이 필요하면 PID 역할을 명시하고 RSS/private/USS 같은 OS 수치를 구분해야 한다. Python/Rust allocator나 주요 buffer의 용량도 별도로 기록해야 한다. 현재 자료에 없는 내역을 역산해서 확정할 수는 없다.

## 산출물

- `runtime-review.json`: 시나리오, 호출별 queue/모델 구간, 실패 반환, 원시 PID 내역
- `dp02-failed-attempts.csv`: 실패 여섯 건의 정확한 시각과 원시 경로
- `derive_runtime_review.py`: API 없이 위 원시 자료를 읽는 재현 코드


## 7. A1 · B1 · B2가 실제로 뜻하는 것

|보완|실제 목적과 동작|이번 검증 상태|
|---|---|---|
|A1 제한된 별도 질문|원래 대화의 긴 판단은 유지하고, 짧고 도구가 필요 없는 독립 질문에 한 번의 별도 모델 호출을 허용한다. 모델이 부적합하다고 하거나 계약 검증이 실패하면 원래 step 경계로 돌아간다. 원래 요청의 checkpoint와 수명은 유지한다.|E2 K=2 세 번 실제 실행. 전체 호출은 6회로 기본과 같았다. 짧은 side 답변이 뒤의 기본 U2 호출을 대체했기 때문이다. 일반적인 긴 새 요청이나 원본 조회까지 확장한 실험은 아니다.|
|B1 불변 문맥 참조 공유|여러 요청 실행이 같은 원본 조각을 로컬에서 복사해 갖지 않고 공유 참조와 변경분을 유지한다. 판본·허용 철회·해제 관리가 필요하다.|설계 후보이며 구현·측정하지 않았다. 직렬화한 cloud 요청과 입력 token은 그대로일 수 있다. null 응답의 정확성 보완은 아니다.|
|B2 foreground 호출 여유 예약|같은 총 호출 한도 안에서 새 입력을 처리할 여유를 남기는 scheduling 정책이다. 원래 요청을 먼저 보내는 시점이나 완료를 늦출 수 있다.|설계 후보이며 실행하지 않았다. K=2에서 두 호출만 있는 현재 E2는 이미 두 번째 slot이 비어 있다. K=1에서는 시작한 비선점 호출을 예약만으로 밀어낼 수 없다. 시작을 유예하거나 실제 중단을 지원해야 하며, 무후속 입력 사례의 원래 요청 지연도 함께 측정해야 한다.|

B1/B2는 메모리·포화 대기 보완이다. B의 null 6건을 자동으로 고치는 정확성 보완이 아니다. 앞 절의 반환 계약 검증과 정정 상태/사용자 내용 분리가 별도로 필요하다. [기존 보완 정의](/Users/changbae/Workspace/via-sw-architecture/docs/architecture/12-decisions/decision-packages/seven-problem-review/02-interpretation-execution.md:175); [실제 A1의 모델 적합성 판단](/Users/changbae/Workspace/via-sw-architecture/prototypes/candidates/four_dp_poc/variants/dp02-A1-bounded-side-question.rs:19); [원래 실행으로의 복귀와 전달](/Users/changbae/Workspace/via-sw-architecture/prototypes/candidates/four_dp_poc/variants/dp02-A1-bounded-side-question.rs:51).

## 8. QA03 산식의 추가 한계

정식 문서는 외부 업무·사용자 대기 node만 제거한 실제 의존 그래프의 가장 긴 경로를 요구한다. 현재 analyzer는 각 STIMULUS→GOAL_COMPLETE 구간에서 receiver 전송→반환과 VIA가 점유하지 않은 사용자 대기 구간의 합집합을 뺀다. 일반적인 실행 DAG를 재구성하는 계산기는 아니다. 이번 DP02에는 외부 수신자 모델이 없고 30ms 사용자 대기는 VIA 호출과 겹쳐 차감하지 않았으므로, 원값은 현재 callback 경과시간으로 읽을 수 있다. 그러나 이를 모든 Agent/병렬 실행이 있는 제품 목표의 정식 QA03 알고리즘이라고 할 수는 없다. [현재 구간 차감](/Users/changbae/Workspace/via-sw-architecture/benchmark/architecture/four_dp_poc/analyze.py:78); [정식 인과 그래프](/Users/changbae/Workspace/via-sw-architecture/docs/architecture/12-decisions/decision-packages/qa-candidate-package/qa03-via-processing-time.md:80).


## 9. 대기 위치의 차이를 실제 추가 지연과 구별한다

아래는 U2 입력 기준 세 반복의 평균이다. enqueue 앞의 A step 대기와 enqueue 뒤의 B slot 대기를 같은 기준에서 보여 준다.

|사례|안|입력→enqueue ms|enqueue→send queue ms|입력→실제 send ms|send→모델 반환 ms|
|---|---|---:|---:|---:|---:|
|02-E1|A|0.777750|2.306041|3.083792|2852.372806|
|02-E1|B|0.853305|2.236222|3.089528|3172.758944|
|02-E2|A|8839.787347|3.093500|8842.880847|2826.497070|
|02-E2|B|0.433847|2.035805|2.469652|3690.772653|
|02-E3|A|11183.528888|2.291847|11185.820736|2914.300514|
|02-E3|B|0.453444|15953.125917|15953.579361|4321.417181|
|02-E4|A|11045.471000|2.626417|11048.097417|4255.789764|
|02-E4|B|40.531431|2.013375|42.544806|2163.729292|

E3 K=1에서는 양쪽 모두 첫 R1이 끝난 뒤 U2를 실행한다. 초기 R1의 HTTP 요청 JSON은 네 시나리오의 각 반복 A/B에서 모두 같다(12/12쌍). 따라서 E3에서 관측한 A/B의 대기 차이 약 4.768초는 같은 초기 요청의 모델 출력·서비스 변동이 크게 섞인 결과다. U2의 모델 구간도 서로 다른 후속 문맥·출력과 서비스 조건으로 약 1.407초 달랐다. 평균 반전 전체가 B 구조에 필연적으로 추가되는 순수 scheduling 비용이라고 말할 수 없다. K=2 E2에서만 실제 병행 호출이 있었고 A에는 없었다는 기전은 원시 전송 구간으로 확인된다.

