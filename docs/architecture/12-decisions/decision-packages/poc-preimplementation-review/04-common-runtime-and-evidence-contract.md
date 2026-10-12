# 공통 실행 경계와 원시 기록 계약 초안

> **PREIMPLEMENTATION_REVIEW_DRAFT / NOT_IMPLEMENTED / NOT_RUN — 2026-10-11.**
> 아래는 구현할 책임, 자료 형식과 검증 방법이다. 실행 코드, 실측 trace와 측정 freeze가 아니다.

[요약과 검토 항목](./README.md), [세 사례](./01-cases-and-oracles.md),
[DP41 실행 계약](./02-dp41-executable-contract.md),
[QA와 변경 과제](./03-qa-readiness-and-change-tasks.md)를 함께 읽는다.
현재 Architecture 계약은 [04-40](../04-40-common-execution-contract.md)과
[조합 검토](../composition-review.md), 품질 의미는 [현재 QA](../03-02-quality-attribute-definitions.md)가 원본이다.
새 필드와 수치는 사용자 검토용 제안이며 원본을 자동 개정하지 않는다.

## 1. 같은 구현 기반과 다른 실행 책임

초기 구현은 현재 저장소 `.venv`의 Python, 표준 `asyncio`, `sqlite3`와 JSON을 기반으로
제안한다. 설정 가능한 공통 비동기 job 실행기를 사용하고 blocking I/O는 제한된
worker로 넘긴다. 처음부터 UI 앱과 전체 Agent runtime을 만들지 않는다.

2026-10-11 준비 확인에서 Python **3.14.7**, SQLite **3.53.4**, `arm64`,
macOS **26.5.1 / 25F80**, `hw.model=Mac17,4`, RAM **24GiB**, logical CPU **10**을
확인했다. 이는 환경 정보 확인이며 QA 메모리나 성능 측정이 아니다.
현재 `.venv`에는 `jsonschema`, `httpx`, `websockets`, `numpy`가 없다.
이 단계에서 설치하지 않았다. 다음 단계의 실제 필요 dependency와 lockfile은
구현 범위에 따라 정하고, 표준 라이브러리로 가능한 구조 검증부터 시작한다.

| 경계 | 공통 제공 기능 | 후보가 유지할 책임 |
| --- | --- | --- |
| Input port | 확정 input ID/revision, 원문, 음성 구간, evidence refs와 gap 공급 | Request Controller의 입력 확정/hold. 의미를 runner가 미리 채우지 않음 |
| Bounded read port | 권한에 따른 owner 원본 read, version/coverage/error와 읽은 범위 반환 | 41A의 모델 또는 41B의 Engine이 다음 read를 선택. 45의 조합/생산/조회는 후보 내부 |
| Model Access port | context 직렬화, 실제 provider adapter 또는 지정 반환 mock, job/usage 대응 | 41 해석, 필요한 45 생산/응답 준비의 호출 조건은 해당 owner. 공통 코드가 반복 수를 정답에 맞추지 않음 |
| Meaning proposal port | 공통 의미 형식 검사, ID/source/권한/currentness 검사 | A의 모델 또는 B의 Engine이 의미 값을 생산. 공통 검증이 잘못된 뜻을 정답으로 수정하지 않음 |
| State/command port | 논리 ID, durable 변경, command idempotency, currentness와 전송 gate | 42A 공동 확정 또는 42B 독립 확정/접수 반영. 공통 repository가 몰래 전역 transaction을 제공하지 않음 |
| Completion/event port | 실제 source 사건과 job 완료의 typed envelope, 제한된 전달 기반 | 44A Dispatcher 또는 44B Stage/Join이 후속 실행, 대기, 취소와 수용량을 결정 |
| Publication/device port | 실제 Text 표시, audio sample/재생/stop와 receipt | Response Manager의 준비/release와 44의 게시 연결. sink가 늦은 후보를 알아서 버리지 않음 |
| Trace/memory port | 사건/호출/실제 효과/메모리의 보존과 읽기 전용 관측 | recorder와 evaluator는 후보 업무 상태를 고치거나 다음 처리를 선택하지 않음 |

공통 권한 검사에는 현재 source/행위의 허용과 version을 넣지만 evaluator-only
정답 Task, 대상, 관계를 넣지 않는다. 후보가 허용된 다른 문서를 잘못 골랐으면
그대로 read/인계/응답에 남고 oracle가 실패를 판정한다. 금지된 행위는 정책으로
차단할 수 있지만 payload를 올바른 행위로 바꾸어 성공 receipt를 반환하지 않는다.

## 2. 첫 구현과 후속 DP의 실제 배치

첫 세 사례에서는 41A/B를 교체하고 44A/45A/42A를 공통 기준으로 유지한다.
44A는 여러 job을 비동기로 실행하고 완료를 중앙에서 회수한다. 45A는 필요한 owner
원본을 읽어 조합하며 유효 cache를 사용할 수 있다. 42A는 모듈을 나누되 관련
대화/업무 로컬 변경을 한 Core의 transaction으로 확정한다. 클라우드 모델과
외부 Agent 호출을 SQLite transaction 안에 넣지 않는다.

41B의 항목별 잠정 상태는 Request Interpreter 내부다. 채택된 Request/질문은
Request Controller, Task/Execution/command는 Task Manager, 전달 원장은
Response Manager가 소유한다. frame 해석 완료는 summary result 생성 완료와 다르다.
현재 goal의 미래 산출물은 symbolic result reference와 의존 관계로 전달하고 실제
산출물이 도착하면 Task 실행이 version을 연결한다. 평가자가 K현재 값을 선주입하지 않는다.

후속 42B에서는 대화 서비스와 업무 서비스를 별도 process 및 별도 저장소로
실행한다. command/receipt/event와 hold/ACK/CAS/reconciliation port를 실제로
사용하고 각각의 process 수명, IPC와 copy를 기록한다. A에 불필요한 IPC를 추가해
균형을 맞추거나 B의 IPC를 시간 상수로 모사하지 않는다.

후속 44B에서는 각 Stage/window와 Publication Join이 해당 후속 처리를 소유한다.
같은 async job 실행 기반을 써도 중앙 runner가 모든 continuation을 결정하면
구조 교체로 인정하지 않는다. 후속 45B에서는 원본의 실제 준비 활동에서 producer가
자료를 만들고 reader가 coverage/source/권한을 확인한다. 완성 관계를 fixture에 넣지 않는다.

## 3. 실제로 쓰는 것과 모사하는 것

| 대상 | 두 번째 단계의 구조 검증 | 실제 모델 pilot/본 측정 | 주장 범위 |
| --- | --- | --- | --- |
| VIA 제어/상태/조회/인계/게시 | 실제 후보 코드 | 같은 후보 코드 | 구조와 상태 처리가 실제로 수행됨 |
| 원본 문서/Task/대화/source 변경 | 합성 owner 원본과 version, 실제 read/변경 | 같은 합성 source | 정의한 원천 범위의 처리. 임의 앱/실사용 coverage 일반화 불가 |
| Downstream Agent | 실제 payload를 소비하는 stub과 고정 외부 시간 | 같은 stub | VIA 위임/결과 연결. Agent 도메인 품질과 업무 SLA 제외 |
| 의미 모델 | 지정 반환 mock. A tool 선택/결과 재입력과 B frame/부분 반환을 각각 제공 | 같은 고정 provider/model/revision의 실제 Qwen | mock은 의미 정확성 근거가 아님. 실제 관측만 모델 오류 분석 가능 |
| capture/VAD/전사 | 발화 종료/전사 완료를 별도 모사 사건으로 제공 | 재현 가능한 WAV 입력, 실제 로컬 VAD와 cloud 전사 | mock 시간은 음성 endpoint 실측이 아님. 전사 별도 의존성과 청구 명시 |
| 음성 출력 | 출력 준비/release/receipt의 모사 | GPT Realtime 생성과 실제 재생, 가능하면 loopback 수집 | packet 시각으로 실제 핵심 음성 onset을 대체하지 않음 |
| 자유문 판정 | 검토용 정답/오답 문장으로 판정 흐름 검증 | 고정 judge와 소수 사람 검수 | judge 오류/불확실을 후보 성공으로 자동 처리하지 않음 |
| 로컬 메모리 | 실제 후보 process의 두 OS 값 수집 준비/검증 | 실제 활동의 동일 범위 관측 | 첫 세 사례의 peak를 전체 P21 QA-06으로 표시하지 않음 |

첫 세 사례는 시각적 지칭을 포함하지 않는다. P21 확장 시 원천 observation의
표 내용/version/시각을 동일하게 text로 직렬화한다는 현재 QA 범위를 유지한다.
픽셀 인식/OCR/vision 모델의 품질이나 비용이 이 실험으로 측정된다고 주장하지 않는다.

실제 음성 pilot에서 VAD 임계값, 재생 device/sample clock, 전사 모델, 생성된
음성의 의미 보존과 출력 검증 정책을 정한다. 이때 같은 정책을 A/B에 적용한다.
초기 mock 검증은 VAD/ASR 정확도나 Realtime의 원문 낭독 능력 검증을 대신하지 않는다.

## 4. 입력과 사건 재현

candidate-visible fixture에는 사용자 입력, 허용 source와 before-state만 둔다.
미래 사건의 schedule/offset/seed, 보류한 job 반환 계획과 예정 payload는 replay-only
자료로 분리한다. 후보에는 실제 availability 이후 현재 사건의 envelope만 전달한다.
evaluator-only oracle에는 C1~C6, 허용/금지 효과, 필수 응답 사실과
관측 경계를 둔다. 후보 subprocess가 oracle 경로/내용을 읽지 못하도록 실행
파일 공급과 접근을 분리하고 runtime payload/context 원장으로 유출을 확인한다.

각 외부 사건에는 `source_event_id`, `source_sequence`, `clock_id`,
`scheduled_source_offset_ns`, 실제 availability/주입 시각과 payload digest를 둔다.
동일 seed와 초기 source를 사용하되 두 종류의 시각 조건을 구별한다.

- **고정 원천 offset:** 발화, source 변경, 옛 event와 동시 입력을 같은 monotonic
  offset에 주입한다. 실제 주입 지연도 기록하며 한쪽 내부 완료를 기다려 재배열하지 않는다.
- **실제 접수 상대 시간:** stub 반환은 실제 접수/유효 command 뒤의 명세된
  외부 지연을 사용한다. 잘못된 payload를 정답 command의 접수로 바꾸지 않는다.

이미 진행 중인 R옛의 늦은 job을 시험할 때는 동일 이전 원천 입력부터 실제 후보
job을 만든 뒤 반환을 보류한다. 그런 job을 만들지 않는 합법적 경로는 미발생
branch로 기록한다. 이 branch를 강제로 만드는 별도 sentinel은 본 사례 원값과
분리해 검증 증거로 표시한다. 실제 존재하지 않은 후보 job을 생성한 것처럼 기록하지 않는다.

source offset 자체가 다르게 주입되거나 수집기 결함으로 두 안의 공통 입력이
달라지면 해당 쌍을 `INVALID`로 보존하고 같은 조건으로 재실행한다. 후보의
미지원/timeout/필수 trace 누락은 `INVALID`로 바꾸지 않는다. 주입 오차 허용치와
clock drift 기준은 본 측정 전에 수집기 검증으로 정한다. 코드 전부터 달성하지
않은 정밀도를 숫자로 보증하지 않는다.

## 5. 보존할 원시 파일과 필드

아래 경로는 새 campaign 아래 만들 파일 형식 제안이다. [기록 형식 예시](./evidence-record-example.json)는
손으로 작성한 NOT_RUN 문서 데이터이며 실행 trace가 아니다. raw 구조는 JSON/JSONL,
표 집계는 JSON과 CSV로 내보낸다. raw byte 단위와 monotonic ns를 유지하고 표시할
때만 초/MiB로 변환한다. 각 데이터 집합에 schema/version과 evidence label을 둔다.

| 파일 | 필요한 원시 필드 |
| --- | --- |
| `manifest.json` | campaign/config/repeat/case/observation IDs, source_execution_key, 계약/source/prompt/input/oracle digest, runtime/OS/hardware, mock/실제 의존성, seed, 실행/관측 종료 조건 |
| `inputs-and-source.jsonl` | input/turn/revision, 원문/span/audio refs, 발화 종료/전사 완료, source ID/revision/locator/권한/coverage, 예정/실제 availability |
| `events.jsonl` | event/job/parent IDs, process generation, owner, kind, mono 시각, source/Request/Task/Execution refs, 이전/이후 상태와 원장 ref |
| `model-calls.jsonl` | call/attempt/provider request/session IDs, 요구 owner와 C-role, foreground/background/initial/update/backfill/retry/discard, serialization/prompt/context digest와 artifact refs, source vector/permission epoch, enqueue/send/first/terminal/cancel/late 시각, status/usage/billing ref |
| `model-contexts/` | 실제 wire 요청과 의미 입력/조회 결과의 보존본. 원문/span, 실제 포함 context와 truncation/제외 이유, tool/응답 schema/version. secret header 제외 |
| `model-outputs/` | 실제 provider 응답, partial/tool/frame/의미 값, 구문 검증 오류, retry 관계와 폐기 이유. token usage와 출력 의미를 구별 |
| `effects.jsonl` | adoption proposal/receipt, target/operation/payload digest, authority/source/revision, 명령 생성/CAS/send/외부 접수/완료/UNKNOWN/취소/중복 사실과 관련 증거 |
| `deliveries.jsonl` | publication/revision/output generation, 실제 Text 표시, audio sample/핵심 onset/마지막 sample/중단/실패/미확인, 실제 전달 범위와 응답 내용 ref |
| `causal-dag.jsonl` | node/parents/owner/kind/duration/Request ref, 겹침/원인과 evidence. 외부 Agent 업무/사용자 대기/후속 발화 제외 구간 및 원래 wall-clock |
| `memory.jsonl` | sample 회차, process identity/generation, 실제 sample 시각/회차 내 시간차, phys_footprint_bytes/resident_size_bytes, 성공/누락/미지원, 단계/포함 범위 |
| `allocations.jsonl` | allocation/share/copy ID, byte 수, 생성/해제 시각, producer/보유 process, 실제 copy인지 shared view인지와 관측 가능 범위 |
| `billing.jsonl` | provider invoice/usage item, call/session 대응, input/output/audio/cached 항목, 실제 사용량/단가/price version/date/통화, late/취소 청구, 중복 조정, UNKNOWN 청구 |
| `judgements.jsonl` | C조건/반응/완료별 PASS/FAIL/UNRESOLVED, 실제 후보 출력/효과 ref, 자동 규칙/judge prompt/model/version/답변/판정 근거, 사람 재검토/분쟁과 최종 판정 |
| `derived/` | raw 입력 digest, 산식/도구 version, 포함/제외 관측 ID, 분자/분모, 실패 수, 반복별/전체/사례별/paired 차이 |

모델 Context와 token은 tokenizer로 추정한 값과 provider의 실제/billed usage를
별도 필드로 저장한다. audio와 session 이력의 재입력, cache, retry, stale/취소
후 생성과 배경 생산을 생략하지 않는다. usage 미제공이면 UNKNOWN이며 0원으로
바꾸지 않는다. trace에 access token, API key, Authorization header를 기록하지 않는다.

하나의 provider 청구 단위가 여러 call에 걸치면 `billing_item_id`로 연결하고
세부 사용량 분해가 불가능한 범위는 추정 또는 미분해로 남긴다. 합계에서 같은
청구를 두 번 세지 않는다. VIA 모델 비용, 외부 Agent 비용, 평가용 judge 비용은
분리하되 유료 실행의 전체 비용 상한에는 필요한 항목을 모두 포함한다.

실제 raw는 campaign의 지속 경로에 보존하며 임시 `/tmp`를 최종 저장소로 사용하지
않는다. Git에는 계약/구조화 trace/집계와 artifact digest를 보존하고 큰 WAV/frame은
같은 campaign의 영속 artifact 위치와 hash로 연결할 수 있다. Git에서 제외한 원본의
보존 위치/존재/크기/hash를 검증하고 원본이 없으면 전체 replay 가능으로 표시하지 않는다.
구체 보관 정책은 본 측정 전 확정하며 기존 result directory를 덮어쓰지 않는다.

## 6. 시간 판정과 계산의 경계

실제 반응의 최초 핵심 출력, 최초 유효 출력, 오답/정정 순서와 실제 완료를 각각
기록한다. generic 인사나 “처리 중” filler만의 음성 packet은 해당 핵심 반응이 아니다.
Voice ON이면 필수 내용의 Text 표시와 첫 핵심 음성 onset 중 늦은 시각을 사용한다.
Voice OFF와 local stop의 예외는 현재 QA를 따른다.

QA-02의 오답 이후 정정 membership은 [QA 부록](./03-qa-readiness-and-change-tasks.md)의
검토 제안으로 명시하며 원문에 없는 공통 5초 deadline이나 성공 시간의 clipping을
추가하지 않는다. QA-03는 귀속 30초 안의 유효 완료 부재에 cap을 적용하고 관측
종료까지의 늦은 완료를 별도 기록한다. 두 cap은 실행 watchdog 시각이 아니다.

VIA 귀속 시간은 causal DAG에서 허용한 외부 구간의 duration을 0으로 두고 선후
관계/경합을 유지해 다시 계산한다. 병렬 span 합이나 wall-clock에서 Agent 시간
합을 빼는 방식은 사용하지 않는다. 반응/사이클 각각 해당 시작/종료 경계에서
그래프를 읽고 실제 전체 wall-clock과 제외 구간을 병기한다.

## 7. Mac 메모리 관측 계약

후보 process가 자신의 `phys_footprint`와 resident size를 같은 sampling 회차에
보고하도록 제안한다. 실제 macOS SDK의 `task_info` 사용 가능성, 권한/ABI와
실제 호출 overhead는 구현 시 검증한다. 값이 제공되지 않으면 RSS에서 footprint를
추정하지 않으며 해당 지표 수집 실패를 기록한다.

원시 sample에는 process ID 외에 시작 generation/시간을 둬 PID 재사용과 restart를
구별한다. 50ms 회차의 process membership, 각 실제 수집 시각과 오차/누락을 남기고
그 회차의 합을 계산한다. 모든 process를 원자적으로 읽었다고 주장하지 않는다.
각 지표의 회차별 합과 그 합의 관측 peak를 구하며 서로 다른 시각의 peak를 더하지 않는다.

candidate 내부 helper, cache, 생산 buffer, async queue와 telemetry buffer는 포함한다.
외부 stub, runner, judge와 기록 파일 writer가 별도 process이면 범위에서 구별한다.
프로파일러 자체가 후보 process에 할당한 부담도 기록하고 모든 구성에 같은 방식으로
적용한다. 작은 runtime의 고정 overhead가 구조별 차이보다 크면 그 한계를 공개한다.

RSS process 합은 공유 page를 중복할 수 있으므로 OS process 회계의 합으로 명명한다.
footprint 합도 검증 없이 고유 physical RAM이라고 부르지 않는다. allocation 원장은
후보가 아는 공유와 실제 copy를 설명하며 OS 값에 다시 더하지 않는다. 첫 사례부터
후속 독립 process에서 생길 serialization/copy의 ID와 크기를 기록할 수 있게 한다.
P21의 준비/갱신/철회/drain/idle와 다섯 반복을 모두 포함한 뒤에야 전체 활동의 peak를
보고한다. Mac 두 값은 기존 Windows commit 수치/점수와 자동 호환되지 않는다.

## 8. 두 번째 단계의 완료 검증

정상 세 사례와 아래 sentinel을 함께 검증한다. 이는 다음 단계에서 만들 실행 검사
목록이며 현재 통과한 검사나 신규 QA 모집단이 아니다.

| sentinel | 반드시 드러나야 하는 기록/판정 |
| --- | --- |
| 잘못된 D1 날짜/다른 Task payload | 실제 잘못된 read/인계/응답 보존, 관련 C조건 FAIL. stub/validator의 정답 보정 없음 |
| old result와 duplicate event | source/Request/Execution/revision 귀속과 중복 효과 수, 현재 산출물의 의존 관계 유지 |
| 늦은 정정 이전의 job 완료 | 실제 폐기/게시/효과 기록. sink가 대신 폐기하지 않음 |
| 첫 핵심 오답 후 정정 | 두 출력과 각 시각 보존, 세 평균 membership과 오답 증거 재계산 가능 |
| 유효 완료 누락/늦은 완료 | 30초 cap과 실제 관측 종료/늦은 반환 보존, 완료-only 분모 재계산 가능 |
| cancel 뒤 늦은 model usage | 폐기된 결과와 청구 포함, 재시도/청구 중복 분리 |
| process sample 누락/공유/copy | 누락을 0으로 합하지 않음, process 합과 allocation 설명 분리 |
| 기록 수집기 자체 결함 | paired INVALID 보존과 재실행. 후보 실패를 수집기 문제로 숨기지 않음 |

관측이 맞는지 검증하는 sentinel의 성공과 후보의 정확성 성공을 구별한다.
이 단계에서 구조 차이를 실행할 수 없으면 코드를 확대하기 전에 계약을 보완한다.
