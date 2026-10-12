# PoC 1단계: 구현 전 검토 묶음

> **READY_FOR_USER_REVIEW / DRAFT / NOT_IMPLEMENTED / NOT_RUN — 2026-10-11.**
> 세 사례, 실행 계약, 기록 형식과 QA 준비 항목을 구체화했다. 후보 코드, 모델 API 호출,
> 유료 실행, 메모리 sampling과 성능 측정은 시작하지 않았다. 이 문서는 측정 freeze가 아니다.

목적은 **같은 입력과 source에서 DP의 실제 처리 구조가 만드는 QA tradeoff를 비교**하는 것이다.
목표값 달성, 발표 예시 재현이나 특정 A/B 승리를 구현 목표로 삼지 않는다. 작은 차이와 동점도 인정한다.
이번 작업은 [수행 계획의 1단계](../poc-implementation-plan.md#1단계-구현-전-검토-묶음)이며,
아래 검토 ① 이후 공통 mock 환경과 DP41 구현으로 넘어간다.
[분업, 교차 검토와 정적 검증 기록](./review-notes.md)을 함께 남겼다.

**후속 수행 지시:** 사용자는 필요한 확인 시점에서만 멈추고, mock/stub이 비교 기제를
지우지 않는 작은 PoC를 요구했다. [수행 계획의 후속 원칙](../poc-implementation-plan.md#2026-10-11-후속-지시-비교-기제-보존과-필요한-확인만-수행)이
아래 검토 ①의 일괄 승인 대기를 대체한다. 구현 세부사항은 진행하며 미정인 최종 채점
의미는 본 측정 계약 확정 전 확인한다. 실제 API의 비용 상한은 실행 전에 정한다.

**2026-10-12 축소 제안:** [전사 fixture와 음성 경계 모사](./05-speech-boundary-simulation.md)로
반복 음성 API를 줄일 수 있다. 아래 실제 음성 반복/pilot은 선택 검증으로 재검토하며
음성 포함 추정과 실제 후보/모델 관측을 구별한다. 이 제안은 QA 원문 개정이나 실행 결과가 아니다.

## 1. 첫 구현에서 끝까지 연결할 세 사례

현재 ASR-QA-v6/P21의 입력과 C1~C6를 유지한다. 후보당 세 사례를 한 번 실행한다면
18개 정확성 조건, 지정 반응 3개와 사이클 3개가 대상이다. 아직 실행 수치는 없다.

| 사례 | 실제 처리할 경로 | 반응과 최종 완료 | 반드시 남길 실패 근거 |
| --- | --- | --- | --- |
| S1-01 한정 조회 | D1@v1 원문 read → 마감일 해석 → Text와 Voice 전달 | 정확한 날짜 전달이 반응과 최종 완료. 직접 조회 또는 한정 Agent 위임 허용 | 다른 날짜/자료, 불필요한 외부 변경, 표시만 하고 음성 미전달 |
| S3-02 결과 의존 요청 | 현재 D1 요약 생산 → 그 실제 산출물을 민수 메일에 연결 → SEND | 반응은 실제 메일 외부 접수 안내를 제안. 최종 완료는 접수+8초 SENT의 실제 전달 | K옛을 현재 요약으로 사용, 의존 누락, 중복 SEND, 접수를 발송 완료로 설명 |
| S4-01 부분 정정 | 기존 보고서 FORMAT만 TABLE로 변경 → 메일 기존 지시 유지 → UPDATE | 반응은 변경 외부 접수 안내. 최종 완료는 접수+8초 ReportRevised@v2의 실제 전달 | 기간/자료/메일까지 변경, 메일 재위임, 늦은 R옛의 역변경, 허위 완료 |

접수 receipt는 실제 접수+0.2초이며 final tail과 다르다. S3-02/S4-01의 C5/C6는
접수 checkpoint와 최종 tail 모두 맞아야 통과한다. 늦은 반환은 실제 이전 후보 job을
만들어 보류하고 반환한다. 합법적 경로에 그 job이 없으면 미발생으로 기록하며 강제 sentinel과
구별한다. 준비 호출과 할당도 비용/메모리 원장에 포함한다.

추천 route는 S1-01 Core 직접 조회, S3-02 VIA의 두 목표 연결이다. 직접 요약/위임 요약과
단일 Agent의 compound 위임은 원본이 허용한 경로다. 양안에 동일 capability를 제공하고,
아직 구현하지 않은 합법적 경로는 pilot 범위의 누락으로 공개한다. 이를 모델 오답으로 바꾸지 않는다.
세 사례의 원천, 정답, 음성 경계와 stub 규칙은 [사례 부록](./01-cases-and-oracles.md)에 있다.

## 2. 실제 구현할 차이와 공통 기반

초기 구현 언어는 저장소 Python의 `asyncio`, `sqlite3`, JSON을 제안한다. 외부 UI 제품과
Agent 도메인 업무 전체를 만들지 않고 실제 로컬 제어, 상태, 조회, 인계와 전달을 연결한다.
VIA의 상태 권위는 Mac에 두며 VAD는 Interaction Manager의 Turn-Taking Control에 둔다.
클라우드 모델과 Agent의 내부 실행은 local transaction 밖이다.

| 부분 | A의 실제 기능 | B의 실제 기능 | 첫 구현/후속 범위 |
| --- | --- | --- | --- |
| DP41 | 모델이 READ/CLARIFY/PROPOSE를 선택하고 read 결과가 다음 모델 판단에 돌아감. 구조화 출력/부분 patch/병렬 read 허용 | 모델의 frame에서 코드가 항목 상태, 다음 read/부분 호출/질문과 전체 binding을 구성 | 검토 ① 다음 양안 구현. B도 여러 호출 허용. 모델의 최종 전체 재구성은 B_HYBRID로 분리 |
| DP44 | 중앙 Dispatcher가 비동기/병렬 완료를 회수하고 후속 지시 | Stage가 입력/대기 조건과 후속 처리를 소유하고 Publication Join으로 연결 | 첫 세 사례는 강한 A. 후속 B를 runner의 중앙 제어로 대신하지 않음 |
| DP45 | 요청에 맞춰 owner 원본을 read/조합. 유효 관계 cache와 부분 갱신 허용 | 실제 원본에서 공통 파생 관계를 생산/검증/게시하고 coverage와 권한을 확인하여 조회 | 첫 세 사례는 A. 후속에 초기/반복/새 source/미사용 갱신/철회 사례 추가 |
| DP42 | 모듈형 Core에서 관련 로컬 상태를 공동 확정 | 독립 대화/업무 process와 저장소가 command/receipt/event로 연결 | 첫 세 사례는 A. 후속 B는 실제 IPC/개별 확정/ACK/CAS/reconciliation 구현 |

공통 기능은 ID/revision, bounded read, 실제 모델 adapter, 의미 형식과 권한 검사,
Task 인계, publication/device, telemetry port다. 공통 runner가 의미, 다음 조회나
continuation을 후보 대신 정하면 구조 비교가 사라진다. [공통 실행 부록](./04-common-runtime-and-evidence-contract.md)에
owner 경계와 실제/모사 범위를 기록했다.

처음 전체 비교는 `C0=41A+44A+45A+42A`와 각 DP 하나만 B로 바꾼 C41/C44/C45/C42다.
C0는 실험 기준이다. 16개 조합의 결과를 미리 일반화하지 않고, 단일 교체 결과와 조합 계약에서
상호작용 근거가 나오면 필요한 조합과 반증 사례를 결과 전에 추가한다.

새 구현 경로는 `prototypes/candidates/dp41_45_poc/`, harness/fixture/oracle 경로는
`benchmark/architecture/dp41_45_poc/`다. 이 경로의 실행 코드는 아직 만들지 않았다.
[이전 실험](../../../../../scripts/archive/pre-dp41-45-experiment-tools-20261011/README.md)은 archive에 보존한다.

## 3. RequestFrame에서 닫아야 할 실제 빈틈

기존 RequestFrame v1은 설계 예시다. 그대로는 첫 세 사례의 **실제 행위, 수신자,
현재 producer 결과, 속성 변경과 무관 Task 보존**을 코드가 충분히 구별하지 못한다.
원본을 보존하고 별도 버전 **`dp41-pilot-frame-v1-draft`**를 제안한다.

- 명시 문서/연락처 source lookup과 bounded read를 위한 `EXPLICIT_SOURCE`.
- goal별 `READ_FIELD(DEADLINE)`, `SUMMARIZE`, `SEND_MAIL`, `SET_PROPERTY(FORMAT,TABLE)`와 유한 인자.
- 실제 생성 전에도 표현 가능한 현재 request/goal/revision의 symbolic `GOAL_RESULT`와 `USE_RESULT`.
- 메일의 `NO_NEW_COMMAND` 보존과 보고서 baseline에 FORMAT만 적용하는 patch.
- host가 제공하는 원문 span/기존 채택 조건/read receipt와 항목별 상태.
- 제공된 후보 하나를 선택하는 부분 반환. 임의 식이나 전체 의미를 부분 반환에 숨기지 않음.

첫 제안 한도는 goal4개, reference8개, read limit10개, logical Request의 read16회와
의미 호출8회다. 형식 재생성/부분 호출/재시도도 같은 예산을 소비한다. 이는 runaway 방지용
검토 제안이며 기대 호출 수나 성능 목표가 아니다. A에도 같은 자원 조건과 같은 최종 의미 표현을 제공한다.
후속 P21의 selector/WHEN/질문 답변/취소 확장은 [DP41 실행 계약](./02-dp41-executable-contract.md)에
미지원 목록으로 남겼다. 세 사례 지원을 P21 전체 지원으로 표시하지 않는다.

## 4. 여섯 QA: 이번 준비와 아직 필요한 실행

| QA | 이번에 정리한 관측/판정 | 실제 수치를 내려면 필요한 것 |
| --- | --- | --- |
| 01 정확성 | C1~C6를 의미/인계/효과/전달에 연결. 구조 자동 판정과 자유문 필수 사실/금지 주장 rubric | mock은 상태 구조만 검증. 실제 의미 모델, 고정 judge와 소수 사람 검수 후 P21 5회 |
| 02 응답시간 | cap 적용/정답만/오답 포함 실측의 세 평균, raw 시각과 각 분모. 선행 오답 뒤 정정 규칙은 아래 검토안 | 실제 Text/핵심 음성 onset 또는 local stop. 음성 packet/queue 시각으로 대체 불가 |
| 03 처리시간 | 미완료 30초 cap 평균과 완료-only 평균. causal DAG에서 외부 구간 제외, 전체 wall-clock 보존 | 실제 입력부터 명세된 final 전달까지의 trace. cap 이후도 공통 관측 종료까지 보존 |
| 04 모델 비용 | context/role/attempt/token/재시도/폐기/배경 생산/늦은 청구와 billing 대응 | 실제 provider usage와 청구. 미제공은 UNKNOWN. judge/Agent 비용은 별도지만 지출 상한에는 포함 |
| 05 변경 용이성 | U01~U18 기능 책임 registry와 다섯 구성×M01~06의 30개 독립 job | 실제 patch/완료/회귀와 복수 검토자의 변경 책임 집합. 파일 수나 예상 변경 수로 대체 불가 |
| 06 로컬 메모리 | Mac phys_footprint와 resident size/RSS의 같은 sampling 회차 process 합, 공유/copy 원장 | 실제 후보 process의 OS 수집 검증, P21 전체 준비/배경/철회/drain/idle. Windows commit 점수 자동 환산 불가 |

QA05의 여섯 변경은 **의미 provider wire/usage 교체, 메일 Agent wire/receipt 교체,
일정 source/timezone 교체, 실패 뒤 초안 실행 관계, 질문 답변 범위 token,
문서 동의 철회의 파생 정보/Context/처리 receipt 범위**다. 30개 job ID와 전후 계약,
회귀와 산출물은 [QA 준비 부록](./03-qa-readiness-and-change-tasks.md)에 모두 있다.
M04/M06은 기존 필수 기능과 겹칠 수 있다. baseline을 약화하지 않고 실제 신규 delta를 audit한다.
이미 모두 지원하면 정당한0변경과 근거를 남긴다. baseline 버그 수정은 신규 기능 변경으로 세지 않는다.

DP45의 cache는 보완 tactic이다. 비교의 본체는 요청별 원본 조합과 공통 파생 관계의
생산/검증/게시/조회다. 강한 A의 cache를 금지하지 않으며 B의 준비 생산, 미사용 갱신,
coverage 부족, 원본 확인과 철회도 모두 센다. 첫 세 사례는 이 tradeoff의 충분한 모집단이 아니다.

## 5. raw 기록과 사용자 준비

원시 기록에는 입력/source/event, 모델의 실제 요청 context/출력/usage, 실제 효과,
전달 receipt, causal DAG, byte 단위 OS 메모리, allocation/share/copy와 판정을 분리한다.
집계마다 포함/제외 observation ID와 raw digest를 남겨 재가공한다. 후보와 stub은
oracle에 접근하지 않고, 잘못 받은 payload가 실제 잘못된 효과 또는 protocol 오류로 남는다.

[기록 형식 예시](./evidence-record-example.json)는 **손으로 작성한 NOT_RUN 문서 예시**다.
10월15일을16일로 답한 응답을 0.2초에 전달했다고 가정할 때, cap 적용 값5초,
정답-only 제외, 오답 포함 실측0.2초로 구별하는 필드를 보여준다. 실제 실행 결과가 아니다.
본 실행 raw는 새 campaign의 영속 경로에 보존하고 큰 WAV 등의 위치/hash도 검증한다.

**이번 사용자 준비는 아래 검토 ①뿐이다.** API key와 비용 상한은 아직 필요하지 않다.
다음 mock 단계는 네트워크와 유료 호출 없이 진행할 수 있다. 실제 모델 pilot 직전에는
Qwen/GPT Realtime 계정과 billing, 총 비용 상한, judge와 음성 입력/출력 수집 정책을 확인한다.
API secret은 사용자가 설정하고 Git/trace에 저장하지 않는다.

에이전트는 승인한 단계 안에서 후보/harness 구현, fixture와 hidden oracle 작성,
recording/집계, 자동 검증과 실패 분석을 수행한다. 사용자에게 630조건 수동 채점을 요구하지 않는다.
자유문은 identity를 숨긴 judge와 기준 응답 검수로 시작하고 불확실/불일치 판정을 사용자 검토로 올린다.

## 6. 검토 ①: 코드 작성 전에 확인할 제안

| 항목 | 추천안 | 확인이 필요한 이유 |
| --- | --- | --- |
| 첫 범위와 route | 위 세 사례부터 시작. S1 직접 조회와 S3 VIA의 두 목표 연결을 중심으로 같은 capability 제공 | 허용 route를 오답으로 만들거나 처음부터 전체 제품으로 확대하지 않기 위함 |
| frame와 실행 예산 | 별도 runtime version과 위 지원 연산/예산 사용. 미지원 P21은 명시 | 기존 v1 schema 통과만으로 실행 지원을 주장할 수 없음 |
| S3-02의 핵심 반응 | 실제 외부 메일 접수 안내를 endpoint로 사용 | 원문 “현재 요약/의존 메일의 유효 처리 사실”에 요약-only 안내 포함 여부가 닫혀 있지 않음 |
| QA02의 선행 오답 뒤 정정 | 최초 핵심 오답이면 주 지표5초, 정답-only 제외, 오답 포함 평균은 최초 실제 시간. 후속 유효 정정은 별도 보존 | 단순 오답보다 오답 후 정정의 membership이 원문에서 충분히 닫혀 있지 않음. 유효 첫 정답6초는6초로 유지하며 clipping하지 않음 |
| QA05와 Mac의 해석 | M04/M06 실제 delta audit, 기존 지원은0변경 허용. Mac 두 원값 비교부터 보고 | 변경 수를 만들려고 baseline 기능을 빼거나 Windows 점수를 전용하면 비교 근거가 깨짐 |

QA02의 정정 처리 추천은 기존 보고 안내의 “유효 반응 시간” 문구보다 엄격한 membership 제안이다.
자동 채택하지 않으며 검토 후 공통 계약과 집계를 함께 맞춘다. 지정 반응과 무관한 QA01 실패를
해당 반응의 시간 실패로 일괄 전파하지 않는다.

검토 ①을 마치면 **2단계 공통 환경과 DP41 mock 실행**으로 진행한다. 완료 기준은 세 사례의
입력부터 실제 payload/효과/전달/판정/집계까지 raw로 재계산할 수 있고, A의 모델 재판단과 B의
코드 항목 해결이 실제 경로에 드러나는 것이다. 오답/stale/중복/미완료의 sentinel이 실패를
드러내는지도 확인한다. 후보가 모두 성공하거나 품질 차이가 발생해야 완료되는 단계가 아니다.

다음 사용자 검토는 그 첫 A/B trace와 집계가 나온 뒤 **실제 API 호출 전**이다.
그때 호출 목록, 음성 경계 수집, judge 검수 예시와 준비/재시도 포함 비용 상한을 제시한다.
전체 P21과 30개 변경 실험 이전에는 여섯 QA의 최종 점수나 제품 성능을 주장하지 않는다.
