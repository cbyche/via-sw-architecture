# 대표 세 사례의 입력, 정답과 관측 부록

> **DRAFT_FOR_USER_REVIEW / NOT_IMPLEMENTED / NOT_RUN — 2026-10-11.**
> 구현 전 검토 문서다. fixture 파일, runner, 후보 코드, 녹음, 모델 호출과 실행 결과는 아직 없다.

## 1. 기준과 이 부록의 범위

[수행 계획 1단계](../poc-implementation-plan.md#1단계-구현-전-검토-묶음)의 S1-01, S3-02,
S4-01을 입력부터 실제 효과와 최종 전달까지 연결한다. 원본은
[ASR-QA-v6 정의](../03-02-quality-attribute-definitions.md),
[P21 공통 입력](../03-02-common-workload.json),
[관측 manifest](../03-02-workload-manifest.json)이다. 실행 책임은
[공통 계약](../04-40-common-execution-contract.md)과
[DP41](../04-41-request-resolution-control.md)을 따른다.
원문 C1~C6, 허용 경로, 접수 checkpoint와 최종 tail을 유지한다.

현재 Mac의 로컬 VIA가 Conversation, Request, Task와 전달 상태를 소유한다.
음성 모델과 의미 LLM은 클라우드 의존성이다. 이전 참조 구조의 on-device Omni 조건,
QA-09/19/29/39 모집단이나 archived runner를 이 시험의 요구사항으로 가져오지 않는다.

세 사례는 작은 구현의 동작과 평가 가능성을 확인하는 pilot이다. 1회 실행한다면 후보당
18개 C 조건, 지정 반응 3개, 사이클 3개가 관찰 대상이다. 이는 **계획한 관측 수**이며
현재 결과가 아니다. 이 부분집합에서 P21의 630조건, 135반응, 95사이클이나 최종 QA 점수를
산출하지 않는다. 세 사례만으로 DP45의 생산과 재사용 tradeoff, 집중 도착, 질문 답변,
전체 30개 변경 실험과 장시간 메모리를 검증했다고 주장할 수 없다.

## 2. 후보 입력과 평가 전용 정답의 분리

| 자료 | 후보 접근 | 내용과 주의점 |
| --- | --- | --- |
| 원천 fixture | 허용 port로 접근 | 발화, 실제 source 원문/version, 기존 Task/Execution과 전달 기록, 권한. 정답 Task 선택이나 완성 관계는 포함하지 않음 |
| 사건 계획 | 외부 replay만 읽음 | 독립 원천 사건의 offset, 실제 접수 뒤 반환 지연, event ID와 중복 규칙. 후보가 다음 정답 사건을 미리 읽지 않음 |
| Agent stub의 업무 세계 | Agent endpoint만 접근 | 실제 받은 payload, 현재 외부 실행 상태, protocol/capability/권한, 명령별 적용 기록. C1~C6와 정답 응답은 포함하지 않음 |
| 숨긴 oracle | evaluator와 judge만 접근 | 허용 목표/자료/관계, checkpoint/final 사실, 금지 효과, Text/Voice 필수 사실. 후보 process와 Model Access에는 mount하거나 제공하지 않음 |
| 관측 결과 | 후보는 자기 receipt만 받음 | 실제 전송/적용 payload, 원천 사건, 상태 전이, 표시와 재생 receipt. evaluator는 원본을 읽고 정답으로 덮지 않음 |

현재 `common-workload.json`은 실행 가능한 전체 fixture가 아니라 입력 blueprint다.
이후 후보 가시 입력, replay 계획과 evaluator-only oracle을 서로 다른 파일 및 실행 접근으로 만든다.
원본 fixture와 이 부록의 source hash는 구현 직전에 고정하며 현재 문서 존재를 measurement freeze로
취급하지 않는다. secret은 원천 fixture와 raw trace에 넣지 않는다.

합성 날짜는 원문과 같은 `2026-10-12 09:00 Asia/Seoul`이다. 사례의 상대시각 t=0은
해당 발화의 음성 onset이다. 의미 ID와 runtime ID는 별개다. `D1`, `T보고서`, `K현재`는
fixture/evaluator의 의미 ID이고, 실제 후보가 생성한 ID는 결과와 인과로 대응시킨다.
새 업무를 생성하기 전에 정답 runtime Task ID를 모델에 제공하지 않는다.
`T보호`는 원문에 있는 무관 업무 보호 대상이다. 보호 대상의 구체 원천 레코드는 이후 fixture에서
동일하게 확정하고 변경 전후 hash 및 command 기록을 검사한다. 이 부록은 보호 업무의 새 목표를
추가하거나 해당 업무를 현재 요청과 선결합하지 않는다.

후보 가시 입력의 예시는 다음 형태다. 이는 schema 확정이나 새 fixture 파일 생성이 아니다.

```json
{
  "fixture_version": "P21-pilot-draft-1",
  "case_id": "S1-01",
  "clock_origin": "scene_audio_onset",
  "input": {
    "input_id": "input-generated-at-ingress",
    "audio_ref": "candidate-visible/S1-01/input.wav",
    "speech_span_seconds": [0, 2],
    "transcript_final": "D1에 적힌 마감일 알려줘."
  },
  "source_records": [{
    "source_id": "D1",
    "revision": 1,
    "title": "하늘 프로젝트 안내",
    "text": "제출 마감일은 2026년 10월 15일입니다. 담당자는 민수입니다.",
    "read_allowed": true,
    "model_provide_allowed": true,
    "agent_provide_allowed": true
  }]
}
```

이 원천 레코드는 source owner에 넣는다. Request Interpreter의 첫 모델 context에 항상 원문 전체가
무료로 포함된다는 뜻은 아니다. 양안은 동일한 최초 context 범위와 동일한 bounded read port를 사용하고,
A는 모델이 다음 조회를 선택하며 B는 코드가 frame 항목 상태에서 조회를 선택한다.

## 3. 입력, 사건과 반환의 공통 재현 규칙

### 3.1 발화 종료와 전사 완료

원문 S1-01은 종료와 확정 전사를 모두 t=2초로 정의한다. S3-02의 발화는 0~3초,
S4-01의 발화는 0~2초다. **같은 시각이어도 서로 다른 사건**으로 남긴다.

| 사건 | 원천과 기록 | 후속 처리 |
| --- | --- | --- |
| `SpeechStarted` | audio sample 경계와 로컬 monotonic 대응 | Interaction Manager의 Turn-Taking Control이 InputStarted와 출력 세대/hold 처리를 연결 |
| `SpeechEnded` | 마지막 사용자 음성 sample. VAD 인식 시각도 별도 기록 | QA-02/03 시작 경계. 모델의 전사 completed 수신 시각으로 대체하지 않음 |
| `TranscriptFinalReceived` | 전사 provider의 실제 반환/수신 시각, input ID/revision | 확정 input과 근거 watermark/gap을 연결해 Request Controller가 채택 |
| `SourceEventAvailable` | source에서 사실이 생성된 시각 | replay 예정 시각, 실제 주입 시각, gateway 수신 시각을 각각 기록 |

첫 mock 실행에서는 S3-02/S4-01의 final 전사를 발화 종료와 같은 시각에 제공하는 구체안을
사용할 수 있다. 원본에 전사 지연값이 없는 두 사례에 대한 **검토할 실행 profile**이다.
실제 모델 pilot에서는 동일 WAV의 음성 종료를 고정하고 전사 completed 시각은 측정한다.
전사 지연 때문에 QA 시작을 뒤로 옮기지 않는다. 전사 지연/정정만의 별도 회귀 입력은
정상 사례와 분리하며 이 세 canonical 사례의 유리한 입력으로 교체하지 않는다.

### 3.2 동일 조건은 내부 실행 시각의 강제 일치를 뜻하지 않는다

- 고정 원천 사건은 두 후보 모두 같은 t offset에 예약한다. source availability와 실제 주입의 차이는
  raw에 남긴다. 내부 처리 완료를 기다려 한쪽 사건만 미루지 않는다.
- 업무 반환은 **stub이 실제 외부 접수를 확정한 시각** 뒤의 동일 지연이다. local enqueue나 outbox
  기록을 외부 접수 시각으로 쓰지 않는다. 후보가 늦게 보내면 반환도 늦어진다.
- model/read job의 늦은 반환은 실제 job의 원래 payload 및 revision을 유지한다. 해당 구조가 만들지
  않은 job을 억지로 만들지 않는다. 불발/해당 없음과 실제 폐기를 구별한다.
- duplicate는 동일 event ID, 동일 execution/result/version/payload를 재전달한다. 새로운 결과 ID로
  만들지 않는다. effect ledger의 고유 업무 효과와 gateway 수신 횟수는 따로 센다.
- oracle는 후보의 최종 상태만 보지 않고 명령, 실제 효과, 전달과 후기 사건 전체를 본다.
  잘못된 SEND/역변경을 후속 수정으로 지우지 않는다.

원문이 정하지 않은 duplicate 간격은 현재 요약 결과의 첫 실제 반환 이후 **+1ms**를 제안한다.
이를 두 후보의 같은 event trace에 적용하고 정상 결과와 분리된 event delivery index를 기록한다.
최종 확정은 구현 전 검토 대상이다. 정확한 동시성/다른 경합은 별도 regression variant로 보존하며
유리한 중복 간격의 실행만 대표 결과로 고르지 않는다.

## 4. S1-01 — 제한된 정보 조회

### 4.1 before-state, 입력과 진행

| 구분 | 실행에 제공할 사실 |
| --- | --- |
| source | D1@v1 제목 “하늘 프로젝트 안내”; 본문 “제출 마감일은 2026년 10월 15일입니다. 담당자는 민수입니다.”. D1@v2는 존재하지 않음 |
| 권한 | D1 read, 모델 제공, Agent 제공 허용 |
| before-state | 해당 조회의 기존 Task 없음. 무관 T보호 유지 |
| 사용자 입력 | “D1에 적힌 마감일 알려줘.”, Voice ON, 발화 0~2초 |
| 직접 경로 | 실제 bounded read로 D1@v1을 받고 날짜 답변. 이 응답을 위해 새 Task를 만들 필요 없음 |
| 위임 경로 | 실제 받은 문서/조회 범위로 Agent가 한정 조회를 외부 접수하고 **접수+0.2초**에 반환 |
| 후속 입력/경쟁 사건 | 없음. 불필요한 질문은 원문 공통 규칙의 비힌트 답변과 비용/시간을 적용 |
| 최종 완료 | D1의 정확한 마감일 Text 표시 및 Voice 핵심 사실의 실제 전달 |

첫 구현은 한정 조회의 Core 직접 경로를 중심으로 확인하되 oracle는 원문이 허용한 Agent 한정 조회도
통과시킨다. 위임을 선택했다는 이유만으로 실패시키거나, 위임 결과를 받기 전에 날짜를 맞춘 것으로
추정하지 않는다. 두 구조에 제공한 capability는 동일해야 한다.

### 4.2 평가 전용 C1~C6

| 조건 | 정답 및 근거 | 실패/금지 |
| --- | --- | --- |
| C1 | 목표=D1 마감일 조회, 처리=Core 직접 또는 한정 조회 위임. 채택 의미와 실제 처리의 일치 | 담당자 문의/새 조사로 대체, 영구 보류 또는 접수 인사로 끝냄 |
| C2 | 실제 사용 D1@v1, 마감일=2026-10-15. read receipt와 출력 사실 | D1@v2 발명, 다른 날짜/담당자를 마감일로 사용 |
| C3 | 직접은 No Tracked Task; 위임은 해당 조회의 New Task. 생성 및 binding trace | T보호에 연결/수정, 다른 업무의 결과 사용 |
| C4 | 조회 범위=D1의 마감일. 채택 의미와 인계 내용 | 문서 수정, 일정 생성, 범위 밖 조사 |
| C5 | 직접은 조회/응답만, 위임은 동일 목표의 접수/결과 반영만. 외부 변경 효과=0 | 실제 외부 변경, 위임 접수를 완료로 전이, 중복 변경 |
| C6 | “D1의 마감일은 10월 15일”과 같은 의미를 Text 및 실제 Voice에서 전달. 날짜/시점/상태 일치 | 다른 날짜/조회 실패를 성공으로 설명, 미재생을 청취 완료로 기록 |

이 사례는 반응과 사이클의 의미 종료가 같다. `I-S1-01-1`, `T-S1-01`의 시작은
t=2초의 실제 발화 종료이고, 종료는 같은 정확한 날짜 사실의 유효 전달이다.
위임의 내부 Agent 작업 구간은 QA-03 귀속 그래프에서 별도로 처리하며 full wall-clock은 보존한다.

## 5. S3-02 — 요약 결과에 의존하는 메일

### 5.1 before-state, 입력과 진행

| 구분 | 실행에 제공할 사실 |
| --- | --- |
| source | D1@v1 “하늘 프로젝트는 10월 15일까지 제출하며 담당자는 민수입니다.”, 민수=minsu@example.invalid |
| 권한 | 문서 조회/요약 및 이번 요약을 민수에게 보내는 요청의 실행/제공 허용 |
| before-state | 과거 완료 T옛/X옛/R옛은 D1@v0 요약. K옛@v0=“마감 10월 1일”. 현재 요약/메일의 Task는 아직 없음. T보호 유지 |
| 입력 | “D1을 요약한 다음, 그 요약을 민수에게 메일로 보내줘.”, Voice ON, 발화 0~3초 |
| 고정 사건 | t=3.1초에 R옛/X옛의 K옛@v0 event 도착. 과거 결과의 정체성을 유지하며 현재 job 결과로 바꾸지 않음 |
| 현재 요약 생산 | 위임 시 실제 접수+0.5초. Core 요약이면 후보가 실제 만든 산출물의 준비/등록이 source. 정답 요약을 후보의 Core 산출물로 주입하지 않음 |
| 중복 반환 | 현재 요약의 같은 event ID 재전달. +1ms는 §3.2의 구체안. Core도 실제 산출물 준비 event를 재전달하며 내용은 후보 출력 그대로 |
| SEND checkpoint | 실제 받은 수신자와 본문/산출물로 SEND를 외부 접수. 접수+0.2초에 **접수 receipt만** 반환 |
| final tail | 실제 SEND 접수+8초에 SENT 결과. 올바른 실제 payload에 대응할 의미 ID는 MailCurrent@v1. 접수 receipt와 final event는 별개 |

접수 원장과 실제 외부 효과 원장은 분리한다. 접수만 확인된 시각에는 `ACCEPTED/PENDING`을 기록하며
`SENT`라고 쓰지 않는다. mock Agent의 실제 SEND 효과 발생 시각과 SENT 반환 시각도 따로 남긴다.
첫 실행 구체안은 접수+8초의 final tail에서 한 번의 simulated SEND 효과를 생성하고 그 사실을 반환하는
것이다. 원문은 final 반환 지연을 정했지만 SEND의 내부 발생 시각을 모두 정한 것은 아니므로 이 선택은
stub profile 제안으로 보존한다. 어떤 profile에서도 잘못된 실제 payload를 올바른 효과로 만들지 않는다.

원문은 VIA가 요약과 메일을 연결하는 경로, 전체 관계를 하나의 Agent에 인계하는 경로를 모두 허용한다.
stub은 단계별 command와 명시된 compound 관계를 처리할 수 있도록 동일 capability를 제공한다.
compound의 하위 효과도 실제 입력과 관계를 기록한다. VIA가 선후/결과 사용 관계를 표현하지 않았는데
stub이 case ID나 정답 문장으로 관계를 복구하면 시험이 성립하지 않는다.

### 5.2 평가 전용 C1~C6와 tail

| 조건 | 접수 checkpoint의 정답 | 최종 관찰까지 필요한 정답 |
| --- | --- | --- |
| C1 | 현재 D1 요약과 그 요약의 민수 메일 전송 두 목표. 요약만/원문 메일로 바꾸지 않음 | 누락 목표 없이 최종 SEND 사실까지 연결 |
| C2 | D1@v1, 민수 주소, 이번 요청이 실제 생산한 K현재@v1. R옛/K옛은 현재 결과로 사용하지 않음 | 실제 보낸 내용/참조가 현재 요약이고 final result에도 같은 provenance |
| C3 | 현재 복합 업무 및 관련 하위 목표에 연결. 하나의 복합 Task 또는 연결 Task 허용 | final SENT도 해당 현재 업무/Execution/command에 연결. R옛/T보호 무변경 |
| C4 | 요약 선행과 실제 산출물 사용, 수신자 민수. 예정 결과 참조는 허용 | 예정 참조는 SEND 시 실제 현재 산출물로 해소. 원문/옛 요약으로 대체 금지 |
| C5 | K현재 준비 이후 실제 SEND 효과. 고유 SEND 접수=1회. 요약 중복으로 이중 SEND 없음 | 접수와 SENT를 구별하고 늦은/중복 사건에도 추가 SEND 없음 |
| C6 | “D1의 요약을 민수에게 보내는 요청이 접수됐다”를 사실에 맞게 Text/Voice 전달. 접수 단계에서 발송 완료 금지 | 실제 SENT 이후 현재 요약의 민수 메일 발송 확인을 Text/Voice 전달. 접수-only 안내로 최종 완료 대체 금지 |

C5/C6는 checkpoint와 tail **양쪽 모두** 맞아야 1이다. 과거 결과의 단순 수신/읽기는 오류가 아니며,
이를 현재 요청의 산출물로 채택하거나 SEND에 쓰는 것이 오류다. SENT 이후의 완료 안내는 원문 tail에
따라 허용한다. 반환이 온 뒤에도 새 목표로 다시 보내면 실패다.

`I-S3-02-1`은 t=3초 발화 종료부터 현재 요약/의존 메일의 유효 처리 사실 안내까지다.
이번 pilot의 반응 endpoint 구체안은 위 표 C6의 **실제 외부 메일 접수 안내**다.
원문 관측 표의 “현재 요약/의존 메일의 유효 처리 사실”이 요약-only 안내도 허용하는지는
검토에서 확인할 항목이며, 더 빠른 임의 접수 인사를 endpoint로 고르지 않는다.
`T-S3-02`는 같은 시작부터 현재 요약의 민수 메일 **발송 확인 최종 전달**까지다.
요약 준비/메일 접수와 final SENT의 시각을 모두 보존한다.

## 6. S4-01 — 부분 정정과 늦은 후보

### 6.1 before-state, 입력과 진행

| 구분 | 실행에 제공할 사실 |
| --- | --- |
| 보고서 | T보고서/X보고서는 D1@v1 “2026년 3분기 매출 150억원”을 2026년 3분기/문장 형식으로 작성 중. 원래 인계 외부 접수 완료 |
| 메일 | T메일/X메일의 원래 지시=안내문 D메일@v1 “회의는 10월 14일 10시”를 민수(minsu@example.invalid)에게 보내기. 아직 전송 전이며 원래 인계 외부 접수 완료 |
| 이전 후속 요청 | R옛=보고서 문장 유지/메일 지시 유지, 해석 준비 중. R옛의 신규 외부 command는 없음 |
| 입력 | “보고서는 표로 바꾸고 메일은 그대로.”, Voice ON, 발화 0~2초 |
| onset | t=0에 InputStarted 원천 사건. 로컬 hold와 출력 제어를 시작하며 보고서/메일 취소 의미는 아직 확정하지 않음 |
| old job | 실제 R옛 model/read job의 반환을 adapter가 종료+0.1초까지 보류해 실제 payload 그대로 전달. job 없는 구조에는 가짜 반환 없음 |
| 고정 반환 사건 | t=2.1초에 이미 준비된 이전 반환 release. provider가 이때도 미완료이면 실제 완료 시각을 기록하며 늦음/coverage 부족을 숨기지 않음 |
| 변경 checkpoint | 유효 보고서 변경의 실제 외부 접수+0.2초에 **접수 receipt만** 반환. 메일 재위임/취소는 발생하지 않음 |
| final tail | 같은 유효 변경의 실제 접수+8초에 ReportRevised@v2 반환. report 내용/형식은 실제 받은 변경 payload에서 생산 |

R옛의 old job을 만들기 위해 후보 앞에 완성된 old MeaningProposal을 선주입하지 않는다.
구현 fixture는 동일한 기존 요청 원문과 before-state를 양안에 제공해 각자의 해석 경로를 실제 시작한다.
provider adapter의 준비 barrier는 old job 입력과 실제 반환을 확보하되 채택을 보류하는 시험 준비다.
준비 단계의 호출/시간/할당도 원장에 남긴다. barrier는 새 입력을 후보 내부 완료 시각에 맞춰 미루지
않으며, 공통 scene 시작 전에 수행한다. 실제 모델 pilot에서 미리 생성한 old 반환은 해당 모델의 실제
호출로 얻고 준비 비용을 포함한다. 지정 mock 반환이면 의미 정확성 evidence로 사용하지 않는다.

**검토할 상세 계약:** 원문은 R옛의 해석 준비를 고정하지만 job 개수, old model/read 반환 body,
전사 완료 시각과 source-read 지연을 모두 고정하지 않는다. 이후 실제 fixture에서 raw source/version,
old call input, 보류할 job ID와 반환 status/body를 양안별로 남긴다. 서로 다른 A/B 출력 body를 정답으로
같게 덮지 않는다. old job이 없어 late-return 경로를 실행하지 못하면 canonical 실행과 별도로
`late_return_exercised=false`를 표시하고 그 경로가 검증됐다고 주장하지 않는다.
실제 old job을 만들지 않는 정상 선택을 이 사실만으로 불합격이나 0초 측정으로 만들지 않는다.
해당 폐기 branch의 강제 검증이 필요하면 별도 negative sentinel에서만 old 반환을 주입하고
`FORCED_STRUCTURAL_TEST`로 표시한다. 이것을 canonical 모델 실행이나 의미 정확성 표본으로 섞지 않는다.

### 6.2 평가 전용 C1~C6와 tail

| 조건 | 접수 checkpoint의 정답 | 최종 관찰까지 필요한 정답 |
| --- | --- | --- |
| C1 | 기존 보고서 형식 정정 및 메일 지시 유지. 새 보고서/새 조사로 바꾸지 않음 | 같은 기존 목표의 변경 결과로 끝남 |
| C2 | 보고서 D1@v1, 메일 D메일@v1/민수, source/version/수신자 보존 | report result가 2026년 3분기/150억원 근거 및 표 형식에 대응 |
| C3 | UPDATE는 기존 T보고서/X보고서에 연결. T메일/X메일 유지. 중복 Task/Execution 없음 | ReportRevised@v2도 해당 보고서 변경에 연결. 무관 T보호 무변경 |
| C4 | 변경=형식 표. 보존=3분기, 자료, 메일 대상/수신자/원래 지시 | old 문장 형식 재채택/역변경, 메일 취소/정정 때문에 재발송 금지 |
| C5 | 유효 UPDATE 고유 접수=1회, 실제 payload 형식=표. old 반환으로 역변경/중복 command 없음 | 변경 접수와 완료 구별. 유효 ReportRevised@v2 반영, 후기 old command/중복 적용 없음 |
| C6 | “보고서 형식 변경이 접수됐고 메일 지시는 유지된다”를 Text/Voice 전달. 접수-only 단계에서 변경 완료 금지 | 실제 report result 이후 올바른 표 보고서 참조를 Text/Voice 최종 전달. 메일 취소/재발송 주장 없음 |

메일 보존은 원래 허용된 실행을 영구 중단시킨다는 뜻이 아니다. 이 사례의 지정 사건에는 새 메일
완료 event가 없으므로 임의 메일 성공 tail을 만들지 않는다. 원래 실행의 계속 진행과 정정 때문에
발생한 재위임/취소를 effect ledger에서 구별한다.

`I-S4-01-1`은 t=2초 발화 종료부터 보고서 변경 **외부 접수와 메일 보존**의 유효 안내까지다.
로컬 hold ACK나 old job 폐기만으로 반응 완료를 대신하지 않는다.
`T-S4-01`은 같은 시작부터 ReportRevised@v2의 최종 유효 전달까지다.

## 7. Agent stub이 오답을 고치지 않는 규칙

1. stub의 분기는 case ID/정답 조건이 아니라 **실제로 받은 operation, 대상, revision, 관계,
   입력 산출물, 수신자 및 명령 ID**로 결정한다. fixture의 외부 실행 상태와 protocol validation만
   사용한다. C1~C6 oracle는 읽지 않는다.
2. 권한/지원 capability/protocol을 만족한 잘못된 수신자/내용/형식이 전달되면 해당 payload를 그대로
   적용하고 그 효과를 기록한다. 평가자가 실패를 찾아야 한다. 권한 위반/지원 불가/invalid schema는
   실제 protocol 오류로 반환하며 정답 payload로 재작성하지 않는다.
3. 위임 요약의 deterministic domain stub은 실제 받은 문서에서 마감/담당 사실을 추출한다.
   D1@v0를 받으면 v0 내용의 요약을 내고, 조회 실패/unsupported 내용이면 실패한다.
   “현재 case라서 10월15일”을 반환하지 않는다. Core 요약은 후보가 만든 text/ref를 그대로 사용한다.
4. SEND는 실제 body 또는 해소한 artifact의 text/hash를 effect ledger에 저장한다. 원문을 보내면
   원문 전송, K옛을 보내면 옛 요약 전송, 주소를 잘못 주면 해당 주소 전송으로 남는다.
   이 경우 결과를 `MailCurrent@v1`의 정답 효과로 대응시키지 않는다. 실제 invalid domain 주소를
   사용하며 외부 메일 서비스에는 접속하지 않는다.
5. report UPDATE는 실제 받은 format과 범위를 적용한다. 문장 형식 payload를 받으면 문장형 결과를
   만들거나 실제 capability 오류를 반환한다. 표 보고서라는 정답 artifact를 무료로 주지 않는다.
6. 동일 command ID의 재전송은 같은 접수/결과를 반환하고 고유 효과를 반복하지 않는다.
   서로 다른 command ID로 두 번 SEND/UPDATE를 보내면 두 효과로 기록한다. stub이 목표가 같다는
   이유로 oracle 기반 dedup을 하지 않는다. 후보가 스스로 업무 중복을 막아야 한다.
7. `Accepted`와 `Completed/SENT`는 서로 다른 event다. 반환 지연은 실제 접수 기준이다.
   잘못된 payload의 접수/실패도 raw에 남기며 성공 결과가 없다는 이유로 시험 행을 삭제하지 않는다.
8. bundled compound의 AFTER/USE_RESULT는 받은 구조적 관계대로 실행한다. 누락 관계를 원문 입력이나
   정답에서 복구하지 않는다. Agent 내부 도메인 계획과 VIA가 전달해야 하는 관계를 구별한다.

stub의 업무 품질은 고정 외부 조건이다. 이 시험에서 실제 Agent의 보고서 품질/발송 능력이나 운영
서비스 안정성을 평가하지 않는다. 후보가 도메인 지시만 보냈다는 이유로 숨겨진 정답 관계를 지원한
것으로 인정하지 않는다. stub protocol의 지원 제한은 모델이 쓸 공통 capability에도 공개한다.

## 8. checkpoint, 최종 관찰과 QA-02/03의 연결

| 사례 | 반응 ID와 유효 사실 | 사이클 ID와 유효 최종 사실 |
| --- | --- | --- |
| S1-01 | I-S1-01-1: 정확한 D1 마감일 | T-S1-01: 같은 날짜의 최종 전달 |
| S3-02 | I-S3-02-1: 현재 요약/의존 메일의 실제 처리 사실. pilot 구체안은 외부 메일 접수 | T-S3-02: 현재 요약의 민수 메일 SENT 확인 전달 |
| S4-01 | I-S4-01-1: 보고서 변경 외부 접수 및 메일 보존 | T-S4-01: ReportRevised@v2 최종 전달 |

Voice ON에서는 명세한 사실의 실제 Text 표시와 **첫 핵심 음성 onset 중 늦은 시각**이 유효 종료다.
API 첫 packet, audio 생성 완료, playback queue 등록, 일반 인사나 earcon은 종료가 아니다.
full audio completion/중단 범위와 sink receipt는 별도로 남긴다. mock delivery sink는 logical delivery
관측이며 물리 speaker의 실측 onset으로 표시하지 않는다. 실제 모델 pilot에선 동일 경계를 실제
표시/재생에서 확보해야 한다.

[사용자 지정 보고 계약](../../../../../benchmark/architecture/README.md#poc-결과-보고-요구사항--2026-10-11-사용자-지정)에 따라
QA-02는 5초 대체 전체 평균, 정답 반응만의 평균, 오답이어도 timestamp가 있는 실제 반응 평균을
함께 보존한다. QA-03는 30초 대체 전체 평균과 유효 완료만의 평균을 보존한다.
각각 분모, 제외/오답/미전달/미완료 수와 cap 이전 raw duration을 남긴다.
잘못된 빠른 응답을 주 지표의 유효 반응으로 세지 않는다.

후기 관찰은 원문대로 최종 처리 뒤 5초, 자동 처리 episode 60초, 전체 episode 10분의 같은 상한을
적용한다. 시간 지표의 5/30초 cap은 실행 종료 상한이 아니다. 뒤늦은 final/오답/중복/청구를
삭제하거나 cap을 raw timestamp에 덮어쓰지 않는다. 외부 Agent/사용자 대기/사용자 발화 구간은
raw causal DAG에서 유지하고, 귀속 계산 시 해당 node duration만 0으로 하여 같은 선후/경합 결과에서
끝 시각을 다시 계산한다. 병렬 구간의 Agent 시간을 단순 합산 차감하지 않는다.

## 9. 자동 판정, 자유문 판정과 구조 검증의 범위

| 판정 대상 | 방법과 근거 | 이번 단계에서의 한계 |
| --- | --- | --- |
| source/version, Task/Execution binding, payload, 관계, 고유 효과 | normalized ID mapping, 실제 read/command/effect/event trace 자동 비교 | 내부 frame 모양/모델 호출 수를 정답으로 강제하지 않음 |
| 접수/완료, 중복/old job, 후속 반환 | checkpoint 및 tail의 state/effect/receipt와 causal order 비교 | 올바른 final 값으로 먼저 발생한 금지 효과를 지우지 않음 |
| 자유 Text 및 Voice 핵심 사실 | 필수 사실/금지 주장 rubric의 LLM-as-a-judge. 후보 이름을 가리고 동일 rubric, 원문/전달 ref와 판정 이유 저장 | judge는 아직 호출하지 않음. 불명확/불일치는 두 번째 검토, 필요 시 사용자 확인. unresolved를 통과로 만들지 않음 |
| 실제 Voice 내용과 전달 | 생성 transcript와 실제 재생 구간의 대응, audible evidence 및 Text 비교 | 생성 transcript만으로 실제 듣기 성공을 주장하지 않음. 내용 대응의 불확실성도 기록 |
| 모델 mock의 지정 출력 | READ/PROPOSE 또는 frame/부분 반환을 주어 상태 전이, old/duplicate 처리와 평가자 오류 검출 | 실제 모델의 의미 정확성, latency/token/청구를 증명하지 않음 |

정답 oracle 예시는 evaluator-only 파일에서 다음처럼 한 조건 안의 필수 사실을 묶는다.
세부 fact는 진단에 사용하며 C 분모를 JSON 속성 수로 늘리지 않는다.

```json
{
  "case_id": "S3-02",
  "condition_id": "C5",
  "checkpoint_required": [
    "current_summary_ready_before_send_effect",
    "one_unique_send_acceptance",
    "actual_send_uses_current_summary"
  ],
  "final_required": ["sent_result_bound_to_same_current_send"],
  "forbidden_throughout": ["send_old_summary", "duplicate_send"],
  "evidence_required": ["artifact_record", "send_payload", "effect_ledger", "agent_event"]
}
```

정확성 오류는 원천 자료, 모델 output, adopted meaning, 코드 상태 전이, command payload,
실제 효과와 전달의 인과로 기록한다. 지원 계약을 코드가 어기면 구현 버그, frame/opcode가 필요한
관계를 표현하지 못하면 현재 지원 범위/구조의 한계, 모델 출력이 의미를 잘못 만들면 모델 출력 오류다.
근거가 부족하면 불확실로 둔다. 고친 source에서 재실행할 때 수정 전 실패를 보존한다.

## 10. 구현 전 확인할 구체안과 완료 기준

다음은 현재 QA 정의를 바꿀 결론이 아니라 실행 데이터에 남아 있는 빈칸이다.

| 확인 항목 | 현재 제안 |
| --- | --- |
| S3-02 반응의 구체 endpoint | 메일의 실제 외부 접수 안내로 고정. 원문 표현이 요약-only 안내를 허용하는지 검토 |
| S3-02 duplicate delivery | 현재 실제 산출물의 동일 event ID를 최초 반환+1ms에 재전달. 별도 경합 variant는 별도 evidence |
| S4-01 초기 old job | 기존 원문/before-state로 각 후보의 실제 job을 생성하고 adapter가 원래 반환을 보류. 준비 비용 포함, 없는 job은 없음으로 표시 |
| 전사 profile | mock에서 각 발화 종료와 같은 final 시각을 별도 event로 제공. 실제 API에서는 고정 audio 종료와 실제 final 수신 구별 |
| 보호 레코드/ID mapping | 원문 T보호의 구체 before-state를 후보 입력에 동일 제공하고 oracle만 현재 요청과 무관하다는 판정 사용 |
| Agent protocol | 단계별/compound 공통 capability, 실제 payload와 관계 기반 실행, wrong payload 보정 금지 |

첫 코드 작성 전 완료 기준은 세 사례 모두에 대해 (1) 후보 가시 raw source와 before-state,
(2) 숨긴 C1~C6 및 금지 효과, (3) 고정/접수 상대 사건과 반환,
(4) checkpoint/final endpoint와 관측 ID, (5) 실제 payload를 보존하는 stub 규칙,
(6) mock/실제 모델/실제 전달의 evidence 한계가 검토 가능한 것이다.
현재 이 부록은 그 구체안이며 구현과 실행의 완료 기준은 후속 단계에서 검증한다.
