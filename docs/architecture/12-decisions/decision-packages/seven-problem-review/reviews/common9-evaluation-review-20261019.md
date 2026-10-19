# 공통9 연결 평가의 사전 검토와 실행 후 보완 기록

2026-10-19 / 진행 중 / 두 정식 세대의 원시 보존, 종료, 취소 계약 결함 보완 중

대상: [19 공통 계약](../19-common9-evaluation-contract.md), 새 공통 runtime 및 fixture와 측정기. 이전 독립 PoC나 [조건 진단 검토](prediction-validation-review-20261019.md)의 통과를 새 연결 실행의 통과로 상속하지 않는다.

## 1. 현재 확인한 범위

- 공통 원장의 S02/S08/S11/S12/S15/S18/S19/S20/S26을 직접 대조했다. 한 묶음은 사용자28입력과 Agent15사건, 응답43개, 목표23개, 지칭15판정이다.
- 고유7구성에서 동일9사례를5회 실행하는 계획이다. 모든 구성의 QA01/QA03/QA02 예정 분모는215/115/75다. 대조AAAA를 네DP의 A행에서 공유하며 독립 표본으로 중복 계산하지 않는다.
- 동일 C01~C06의 QA04 실제 독립 변경을 계획했다. baseline 공동 코드가 있다는 이유만으로 자동 동점을 부여하지 않는다. 실제 필요한 운영 변경과 수용 결과를 확인해야 한다.
- source registry, 근거 공급과 인계, 화면 지칭을 공통 Context에 연결하는 코드를 읽었다. capture와 응답/완료는 공통 runtime이 소유한다. 독립 `run()` 연결 방식이 아니다.
- 전용 parser는 detector, EasyOCR, Florence caption의 실제 초기화와 parse에 성공했다. 앞선 두 호환성 실패 및 성공 probe를 새 결과 루트에 보존했다. 한국어와 영어 OCR로 실제 2880×1800 화면의 parse, 같은 원본 cache 재사용과 helper PID 관측도 수행했다. probe는 정식 QA 결과가 아니다.
- 최초 연결 오프라인 검사 `offline-v4`의 A/A/A/A 9사례를 평가기로 대조했다. 실제 Rust와 결정적 모델 대역에서 callback43/43, 최종 목표23/23, 지칭15/15 및 host 실패0을 확인했다. 이는 모델의 의미 정확도나 실제 지연 실측이 아니다. 다른 구성과 권한 보완 이후의 최종 회귀는 별도로 필요하다.
- 인과 시간과 endpoint의 합성 검사를 43개로 보강해 통과했다. 순차 및 병행 외부 대기, 사용자 대기 중 VIA 실행, 실행 경계 전환, 실제 호출 슬롯 경합, 이미 해제된 과거 슬롯 사용자의 제외, 정정의 원래 목표 시작과 새 목표 시작을 구별했다. 미관측 분모, 접수/입력 반복 제외, 구체적인 오답의 시각 보존, 사용량 미확정 예약과 실제 종료 경계의 RSS 처리도 확인했다. 원시는 [합성 검사](../../../../../../results/architecture-evaluation/current/common9-poc-20261019/evaluator-offline/synthetic-checks.json)와 [실행 및 hash 원장](../../../../../../results/architecture-evaluation/current/common9-poc-20261019/evaluator-offline/pre-formal-manifest.json)에 남긴다.
- `offline-v7`에서 7구성×9사례의 63개 회귀가 모두 통과했다. 내용 callback301/301, 최종 목표161/161 및 지칭105/105, 인과 경로 미확정0을 확인했다. 메모리 관측은 54episode 정상과 9episode 불완전으로 구분했다. 시작 또는 종료 순간의 누락 표본을 정상 0으로 바꾸지 않았으며 process lifecycle 계측을 보완한 최종 회귀가 필요하다. 이는 결정적 모델 대역 검사이며 정식 의미 정확도나 성능 결과가 아니다.
- `offline-v8`도 같은 63개 회귀와 301/161/105 관측을 모두 통과했다. 실제 process 종료가 해당 조회 중 확인된 13개 표본은 원시 자료를 보존하고 최고값 계산에서 제외했다. 그 외 완전 표본에 근거한 episode 관측은 63개이며 살아 있는 process 누락을 예외로 허용한 것이 아니다. 정책은 정식 유료 실행 전에 19 계약에 명시했다. 이후 독립 검토의 제어 계약 수정 및 QA04를 마친 상태가 최종 baseline이 된다.

## 2. 정식 실행 전에 해결하거나 확인할 사항

| 항목 | 검토에서 발견한 문제 | 필요한 처리 / 상태 |
| --- | --- | --- |
| 원본 화면 크기 | 초안 asset1000×900이 원장2880×1800과 불일치 | 2880×1800으로 재생성 보고. 새 asset과 manifest를 최종 동결에 포함 |
| 발화 및 trigger | 전사 submit으로 원문 발화 길이를 모두 압축하면 BG와 정정의 겹침 조건 변경 | 시작+지정 발화 길이, BG종료±100ms 및 S08첫표시+200ms시작을 보존하도록 구현 중 |
| 과거 판본 권한 | current 판본에만 철회하면 old 판본 passage나 record로 우회 가능 | 모든 해당 판본의 권한과 공통 게시 guard를 연결하는 수정 보고. 역사 판본의 정당한 조회는 보존. 수용 검사 필요 |
| 파생 출처 권한 | 직접 요약이 새source가 되며 원본의 외부 전달 제한을 잃을 가능성 | provenance 권한 상속과 신규 사용/대기 출력 재검사 필요 |
| Agent 사건 연결 | 초안fixture가 새업무 결과의Task를TA로 고정 | 실제 ingress가 만든Task와 실행에 연결하도록 수정 중. 평가자 정답Task 주입 금지 |
| 외부 결과 진실성 | 잘못된control이라도 고정된TA승인/TB거부 결과를 제공하면 오류를 숨김 | 실제 control 및 실행조건으로 외부shim의사실을 생성하고, 필요한 ingress없이는 성공 결과 생성 금지 |
| 늦은 결과 게시 | receiver 사용 span 없이 최종 Agent 결과를 게시하면 철회 검사 불충분 | 실제 supplied span와 파생 출처를 최종 원자적 publication에 연결하는 수정 보고. 수용 검사 필요 |
| revision 순서 | 의미 판단 중 새 입력과 기존 결과가 교차하면 현재 revision 숫자만으로 부족할 수 있음 | pending 입력, 정정 및 원래 workflow 재개 순서의 수용 검사 필요 |
| 근거 생산 범위 | 임시 12source/48line 제한이 원본과 무관한 누락을 만들 가능성 | 고정 원본 순서 및 명시적 64KiB 입력 budget과 coverage gap으로 교체. 초기 원문은 사례별 약16.8~17.3KiB로 전체 포함 가능함을 읽기 대조. 후속 생성 이력도 확인 필요 |
| 합성 이력 | 반복 배경 200턴을 실사용 이력으로 오해할 가능성 | 합성 작성 배경이며 단순성 한계를 19 계약에 명시 |
| parser 언어 | English-only OCR로 한국어 원문 인식이 깨지는 준비 관측 | 지원되는 ko+en 가중치로 설정 보완 및 실제 parse 완료. 실패 및 영어 probe 보존 |
| QA03 | 겹치는 외부 시간을 wall에서 단순 차감하면 인과 경로 오류 | 순차, 병행, 사용자 대기 중 VIA 실행, 정정 및 slot 경합의 합성 원장 단위 검사 필요 |
| 입력 수신 정지 | capture loop가 Agent publication에서 후속 사용자 의미 판단을 기다리면 transcript를 수신하지 못할 가능성 | Agent publication을 별도 worker로 옮기는 수정 보고. user begin → Agent result → transcript 순서 회귀 필요 |
| 실패한 선행 사건 | ingress가 없어서 Agent 결과가 미제공되면 그 결과를 기다리는 후속 입력도 영원히 발생하지 않을 가능성 | 미제공 anchor의 30초 실패 fallback 필요. Agent 성공을 위조하지 않고 후속 입력만 이어가는 검사 필요 |
| 최초 source 시각 | EPISODE_READY 수신에 5초를 더하면 원본 제공 이후 초기화 지연이 숨겨질 가능성 | 실제 initial source 제공 시각에 5초를 더하도록 실행기 수정 필요 |
| QA01 endpoint | nonempty callback만으로 단순 접수나 미해결 입력 반복까지 관측할 가능성 | 계측 필터, 실제 채택 문구와 제외 사유를 보존하는 검사 필요. 본문 전체 의미 정답과 구별 |
| 철회된 대기 draft | 게시를 막아도 requests.adoption의 answer/claim이 다음 의미 입력으로 다시 들어갈 가능성 | 전체권한 baseline 경로와 구간권한 C06 확장 구별. 기술적 Task 연결은 보존하고 source 유래 본문은 현재 권한으로 검사하는 보완 및 전체 prompt 회귀 필요 |
| 같은 Task 최신 상태 대체 | INPUT_CAPTURED에 실제 task_id가 없으면 최신 완료로 진행 응답을 충족한 연결을 평가자가 확인할 수 없음 | 실제 입력 metadata를 기록. oracle Task 주입 없이 같은 Task/시간 조건으로만 대체 인정 |
| episode 사이 메모리 | Python 실행기를 재사용하면 앞선 trace/heap이 다음 구성의 RSS에 섞일 가능성 | 정식 실행은 episode마다 새 Python 및 Rust process. 외부 orchestrator와 측정 process 구별 |
| process 시작과 종료 RSS | 시작 또는 종료 순간 Rust PID 누락/RSS0 표본을 정상 메모리로 셀 가능성 | v7의 불완전9건 보존. 실제 start/active/confirmed-exit 상태를 기록하고 active 누락은 계속 실패로 처리하는 보완 중 |
| 과거 슬롯 소유자 | 마지막 슬롯 사용자를 무조건 인과 부모로 연결하면 경합이 없던 과거 작업까지 목표에 포함 | 현재 wait 이전에 이미 해제된 소유자는 제외. 실제 경합과 release 이후 dispatch 지연은 유지하는 합성 검사 통과 |
| 불완전 집계 표시 | Markdown에는 숫자만 보여 비용 미확정이나 RSS 하한이 정상값처럼 보일 가능성 | 표의 각 행에 비용 상태와 미확정 예약, RSS 상태 및 관측 하한을 직접 표시하도록 분석기 보완 |
| 중단 후 재실행 | STARTED 뒤 TERMINATED가 없는 identity를 자동 재시도하면 실패 및 비용 중복 가능 | orphan 원장을 보존하고 자동 재실행을 중지하는 guard 추가. 명시적 복구 판단 전 같은 identity 재실행 금지 |
| 실행 binary와 helper | source hash만 같아도 다른 binary나 파서 가중치를 사용할 가능성 | frozen binary를 직접 실행하고 실행 SHA 기록. 외부 dependency inventory 및 가중치 검증을 동결에 연결 |
| C04 수용 범위 | 대기 상태 필드 보존을 실제 음성 부분 교체처럼 설명할 가능성 | 실제 Agent 결과 section, Task 결과와 표시 이력 및 후속 참조를 검사. 대기 음성은 Voice OFF 범위 밖으로 명시 |
| Task와 질문의 결합 검증 | CONTROL의 task_id와 question_id를 각각 존재 검사하면 다른 Task의 질문을 결합해 승인 가능 | 독립 검토 P2. 실제 해당 Task 소유 질문인지 원자적으로 검사하는 공통 guard 및 음성/모델 대역 부정 사례 필요. 의미 정답을 자동 교정하지 않고 잘못된 제어를 거절 |

위 상태는 구현 도중의 검토 기록이다. 담당자의 수정 보고만으로 수용 검사 완료라고 표시하지 않는다. freeze 전 source 대조, 실행 검사, 비용 projection과 미해결 항목을 다시 확인한다.

### 2.1 QA04 기준 소스 동결 시점의 재확인

위 표는 발견 당시의 항목과 필요한 처리를 보존한다. 최종 `offline-v11`에서 같은7구성×9사례가 모두 통과했고, callback301/301, 현재 revision의 목표161/161, 지칭105/105 및 미해결 인과 경로0을 확인했다. 합성 검사는48개로 늘어났다. 추가 항목은 process 시작 시각과 첫 양의 RSS 관측, 실제 종료 경계와 활성 process 누락을 구별하며, 누락을 정상0으로 바꾸지 않는 조건을 포함한다. [실행 및 hash 원장](../../../../../../results/architecture-evaluation/current/common9-poc-20261019/evaluator-offline/pre-formal-manifest.json)을 기준으로 읽는다.

최종 오프라인63episode의 메모리 관측은 모두 유효했다. 조회 중 실제 종료를 확인한13개 경계 표본은 최고값에서 제외하고 원시에 남겼다. 잘못된 표본0, 시작 직후 RSS0 예외0이며, 50ms 표본으로 연속시간의 실제 최고값을 증명한 것은 아니다.

[독립 runtime 검토](common9-independent-runtime-review-20261019.md)의 두P2인 Task/질문 소유권 결합과 외부 결과 source의 파생 출처 상속을 수정했다. 잘못된 Task/질문 조합의 실제 거절 trace, actual adapter의 공급 span 전달, 재귀 권한 철회의 Rust 검사와 최종63회 회귀를 대조했다. adapter 검사와 registry 검사를 하나의 전체 제품 종단간 시험으로 표현하지 않는다. 해당 독립 검토 범위에서 미해결P1/P2는 없다.

QA04의 기준은 `qa04-baseline-snapshot-v2/freeze.json`이며 SHA256은 `38b92206e6ecfc10eedb839c97f18ddec4c898e850f5c0db5f55d721178501b1`이다. 운영 소스와 평가기, 입력 및 계약을 포함한60개 항목을 고정했다. 앞선v1은 검사 준비 이력으로 보존하며 최종 QA04 결과로 쓰지 않는다. 같은6과제를7구성에 적용하는42개 실제 수용 검사와 두 독립 원장 검토를 완료했다. 정식 유료 캠페인의 실행 허가를 이 준비용 snapshot이 대신하지 않는다.

### 2.2 QA04 수용과 독립 검토 완료

모든42개 복사본에서 실제 Rust 검사/build, 유한 추가 기능 검사와 공통9 회귀를 통과했다. 각 구성의 C01~C06 변경 수는1/1/3/2/1/2, 평균10/6이다. parent의 실제diff 읽기와 두 담당의 독립 원장 검토가 같은 운영 모듈 집합에 동의했다. 보존된378개 trace를 고정 평가기로 별도 재평가한 결과도 callback1,806개, 목표966개, 지칭630개로 일치했다. [QA04 결과](../../../../../../results/architecture-evaluation/current/common9-poc-20261019/qa04/QA04-RESULTS.md), [실행 및 도구 검토](../../../../../../results/architecture-evaluation/current/common9-poc-20261019/preflight/qa04-independent-review.md), [독립 원장 재평가](common9-qa04-independent-ledger-review-20261019.md)를 근거로 한다.

이전PASS가 이후 실패를 숨길 수 있던 도구 상태 전이와 C05의 실제 새 출력 검사 부족은 최종 수용 전에 수정했다. 최초v2 AAAA/C01의 `ps` 접근 차단 실패는 보존했으며, 같은 후보가 권한 확인 후 새 verification에서 통과했다. 이 변경 측정은 공통adapter와 게시/권한 과제에 대한 동점이다. 방안 고유 producer와 실행정책의 일반적인 변경 용이성 동등성을 입증하지 않는다. 코드 검사 중 실제 provider 호출은0이다.

### 2.3 사전 예측의 검증 범위

[24개 예측 범위 검토](common9-prediction-coverage-review-20261019.md)는 공통9의 공정성과 가설 성립 조건의 포괄성을 구별한다. DP2의 독립 새 의미목표 병행과 DP3의 작은 밀집 요소 조건은 이번 입력에 없다. B의 실제 screen producer에도 원본PNG를 주므로 완전한 text-only producer 비용 가설을 검증하는 배치도 아니다. 이를 정식 모델 결과 전에17문서의 해당 셀과§11.1에 반영했다. 이 조건들의 우열이 관측되지 않았다는 이유로 가설을 반증하거나 다른 경로의 차이를 그 기제의 지지로 표현하지 않는다.

### 2.4 controller 잠금과 정식 실행 승인

같은 결과 루트에 두 controller가 접근할 때 검증부터 dispatch와 종료 기록까지 하나의 OS 잠금을 유지하도록 보완했다. 실제4개 process를 사용한 guard는 두 번째 controller가 body 진입 전에 거절되는지, 정상 종료와 주입한 실패 후 잠금을 다시 얻을 수 있는지 확인했다. 전체 campaign guard6개가 통과했으며 원시 실패와 잠금 사건은 보존한다. SIGKILL과 실제 유료 요청 중 충돌을 시험한 것으로 확대하지 않는다.

최종 준비 snapshot `preformal-snapshot-v3/freeze.json`의 SHA256은 `85ab51db40529af34417c43fc512afb4e737c25185dcedca73709012106a92c2`다. v2와의 차이는 `campaign.py` 및 `check_campaign.py` 두 파일이며 나머지58개 소스, 후보 binary와 입력, 정답 및 계약은 동일하다. [독립 검토 원장](../../../../../../results/architecture-evaluation/current/common9-poc-20261019/preflight/controller-lock-v3-independent-review.json)은 이 대조와 실제 잠금 증거를 확인했고 미해결P1/P2가 없다. 큰 외부 가중치는 이 가벼운 재검토에서 다시 읽지 않았다.

사용자는 이후 **$40 상한으로315회 실행**을 명시적으로 승인했다. 최초$4 상한과 승인 대기 상태는 과거 기록으로 보존하고 새 승인 원장 및 정식 동결에 연결한다. 비용 승인으로 시험 입력이나 QA 의미를 변경하지 않는다. 정식 원격 모델 결과는 실행을 마친 뒤 별도로 검토한다.

### 2.5 첫 모델 실행에서 관측한 공통 제안 오류

정식 `formal-v1`의 AAAA/S02와 BAAA/S02는 실제 API 각12회가 정상 반환됐지만 각6개 사용자 workflow가 모두 선언되지 않은 권한 구간 이름으로 거절됐다. 첫 응답과 목표 완료는 각각0/6이며 그럴듯한 모델 답변을 실제 게시 결과로 바꾸지 않는다. 두 독립 담당이 원시 응답과 source 읽기 및 거절을 대조했고, 권한 guard의 거절은 올바른 동작임을 확인했다. 모델이 일반 조회에도 권한 변경을 제안하고 행 번호 표현을 구간 이름으로 사용한 실제 오류다. API나 endpoint 계측 실패로 재분류하지 않는다.

단일 controller는 현재 episode 종료 뒤 다음 dispatch를 보류하도록 했고 정상 종료했다. 이 세대는2/315실행과313미실행,24API 및$0.0420392로 보존한다. 이전 조건 진단을 포함한 사용분은$0.1436528이며 미확정 예약은0이다. [종료 원장](../../../../../../results/architecture-evaluation/current/common9-poc-20261019/formal-v1/METHOD-HOLD-CLOSURE.json)과 [첫 원인 감사](../../../../../../results/architecture-evaluation/current/common9-poc-20261019/independent-formal-audit/AAAA-S02-r1-acd5e6-cause-review.json)를 남긴다.

공통 모델 출력 형식과 전체 제안의 변경 전 검증, 최대3회 단일 의미 재판단 경로를 보완한 새 기준을 준비한다. 이 보완은 모든7구성에 동일하게 적용하고 테스트 입력, 정답, QA 경계는 유지한다. 실패 제안을 자동 수정하거나 삭제하지 않으며, 새 성공의 시간과 비용에는 앞선 재판단도 포함한다. 새로운 기준의315회와 QA04 변경 검사는 별도로 확인한다. 보완 전후 결과는 합산하지 않는다.

### 2.6 보완 후 기준의 독립 검토

새 `qa04-protocol-v2-baseline-snapshot/freeze.json`의 SHA256은 `91f6b309e6af675c7ce8e8b7b5eb14345ef2cc325bc9d5619325f1830c8f5aa5`다. [독립 동결 검토](../../../../../../results/architecture-evaluation/current/common9-poc-20261019/preflight/protocol-v2-independent-freeze-review.json)는61개 frozen/live 소스, 후보 binary와 변경7경로를 대조했다. 후보 변경은 `runtime`과 `sources`이며 입력, 정답, 화면, 모집단, 계약, 315개 작업 순서와 QA 계산식은 그대로다.

운영 Rust23검사, 실제 Rust 프로토콜98검사, runtime guard7개, controller guard6개와 인과 측정 합성48검사를 통과했다. 같은7구성×9사례의63개 결정적 모델 대역 회귀를 독립 재평가해 callback301개, 완료161개, 정확 지칭105개 및 인과 경로를 확인했다. 메모리63episode는 모두 관측 가능했고 잘못된 표본0, 시작RSS0 예외0, 실제 종료 확인으로 제외한 경계 표본8개는 원시에 보존했다. [독립 코드 검토](../../../../../../results/architecture-evaluation/current/common9-poc-20261019/preflight/protocol-v2-independent-core-review.json)와 [원시 회귀 재평가](../../../../../../results/architecture-evaluation/current/common9-poc-20261019/preflight/protocol-v2-independent-stable-acceptance.json)가 근거다. 유료 v2 호출 전의 검사다.

공급된 원본의 현재 권한을 의미 호출 전과 제안 검증에서 확인하고, 실제 권한 변경 직전 같은 gate 안에서 다시 검증한다. DOC 근거 철회 후 OTHER 원본에 대한 유효한 권한 변경도 부분 적용 없이 거절되는 부정 검사를 확인했다. 실제 catalog와 입력 지원 여부의 검사이며 평가자의 의미 정답을 제안에 주입하지 않는다. 한 번에 첫 오류만 알려주는3회 한도는 복수 오류를 모두 고치지 못할 수 있고, 형식상 유효한 의미 오답도 남을 수 있다.

새 QA04는 같은6과제의42개 복사본과 실제 패치를 준비했고, [모듈 집합 사전 대조](../../../../../../results/architecture-evaluation/current/common9-poc-20261019/preflight/qa04-protocol-v2-independent-count-precheck.json)를 통과했다. 실제 수용 검사는 진행 중이므로 아직 새 기준의 QA04 완료값으로 쓰지 않는다. 최초 AAAA/C01과C02의 process 조회 차단 실패는 원장에 남기고 같은 후보에서 읽기 전용 process 접근을 확인한 별도 verification으로 재검사한다. 이전42개 수용 결과를 새 기준에 상속하지 않는다.

### 2.7 새 QA04 수용 완료 및 formal-v2 시작

새 기준의42개 실제 변경을 모두 수용했다. 각 구성의 모듈 수는1/1/3/2/1/2, 평균10/6이며 기준과 도구 및 실제 후보, binary hash를 확인했다. [QA04 새 결과](../../../../../../results/architecture-evaluation/current/common9-poc-20261019/qa04-protocol-v2/QA04-RESULTS.md), [독립 원시 재평가](../../../../../../results/architecture-evaluation/current/common9-poc-20261019/preflight/qa04-protocol-v2-independent-final-review.json)와 [별도 실행 감사](../../../../../../results/architecture-evaluation/current/common9-poc-20261019/preflight/qa04-protocol-v2-independent-evidence-audit.json)를 남겼다. 두 담당 모두 해당 범위의 미해결P1/P2가 없음을 확인했고 parent도6종 실제diff와 집합을 읽었다. 새378개 회귀의1,806callback, 966목표, 630지칭이 일치했다. 이전42개 자료는0개 상속했으며 실제 API 호출도0이다. 최초2개 process 조회 실패는 같은 후보의 별도검사 통과와 함께 보존했다.

정식 `formal-v2` 동결 `072044b7a46050f4a20d839cda6ecf66cceebfb641748d8ce6b6d8e5c609aa28`은 새 QA04의61개 소스 및 binary, 계약, 315개 작업과 동일하다. [정식 승인 독립 검토](../../../../../../results/architecture-evaluation/current/common9-poc-20261019/preflight/formal-v2-independent-authorization-review.json)는 사용자의$40 승인, 기존 사용분$0.1436528 및 미확정 예약0, 모든315개 허용 작업과 정확한 명령을 대조했다. parent GO 이후 **2026-10-19 06:11 UTC** 단일 controller가 시작됐다. [시작 기록](../../../../../../results/architecture-evaluation/current/common9-poc-20261019/formal-v2/LAUNCH-SESSION.json)을 근거로 하며, 실행 준비 통과를 정식 결과의 의미 정확성이나315회 완료로 표현하지 않는다.

측정 창에서는 팀의 build, 시험과 무거운 분석을 중지했다. 다른 사용자 세션이나 OS cache까지 통제한 전용 host라고 주장하지 않는다. 실제 모델의 오답과 제한된 재판단 실패는 그대로 남기며 결과에 맞춘 prompt 변경이나 재추첨을 하지 않는다. 최종 QA 집계와 조건부 예측의 일치 판단은 완결한 실행 원장을 검토한 뒤 추가한다.

### 2.8 종료, 취소 계약 결함과 두 번째 세대 보류

`formal-v2`는 9/315회 실행 뒤 다음 작업을 보류하고 controller가 정상 종료했다. 원인은 공통 실행기와 runtime의 종료 처리다. 실행기는 실제 `RESPONSE_FINAL`과 30초 미응답 fallback을 같은 완료 집합에 넣어 마지막 source 뒤 약30초에 입력을 닫을 수 있었으며, runtime의 `end_input`은 worker join을 시작할 뿐 실제 취소를 수행하지 않았다. 평가기도 QA01/QA03의 마지막 실제 필수 source+60초 상한을 적용하지 않았다. [19 계약 §5](../19-common9-evaluation-contract.md#5-qa별-같은-측정과-집계)의 요구와 다르며 앞선 오프라인 검사의 통과로 이 누락이 해소되지 않는다.

[정적 검토와 원시 대조](../../../../../../results/architecture-evaluation/current/common9-poc-20261019/independent-formal-audit/formal-v2/final-source-deadline-static-review.json), [S02 source 시각 대조](../../../../../../results/architecture-evaluation/current/common9-poc-20261019/formal-v2/S02-SOURCE-TIMELINE-AND-DEADLINE-REVIEW.json), [종료 원장](../../../../../../results/architecture-evaluation/current/common9-poc-20261019/formal-v2/METHOD-HOLD-CLOSURE.json)을 보존한다. 첫 S02의 약38초 source 간격은 30초 fallback+후속 대기2초+발화6초로 설명되며, source 간격 자체를 잘못된 것으로 처리하지 않는다. 확인된 잘못은 마지막 실제 source 뒤 관측 창과 취소다. 이 S02에서 실제 늦은 성공이 발생했다고 주장하지 않으며, 잘못 닫힌 창 이후의 추가 비용, 메모리는 기존 자료 재계산으로 복원할 수 없다.

이 세대는 실행9개와 미실행306개, API189회, 비용$0.5198596으로 닫았다. 앞선 조건 진단과 첫 정식 세대를 포함한 사용분은$0.6635124, 미확정 예약0이다. 관측과 실패를 삭제하거나 성공 건만 추려 새315회에 합치지 않는다. 고정 입력, 정답, 구성, 반복 수와 사용자의 총$40 상한은 유지한다.

[보완 설계](../../../../../../results/architecture-evaluation/current/common9-poc-20261019/preflight/deadline-correction-design-proposal.md)에 따라 다음을 구현, 독립 검증한 뒤 새 기준을 동결한다. 이 목록은 수행 계획이며 아직 수용 완료가 아니다.

- 실제 응답과 fallback을 분리하고, 마지막 실제 제공 필수 source 뒤60초까지 공통 관측한다. 누락 사건을 가짜 source나 성공으로 만들지 않는다. 취소 전 늦게 실제 제공한 필수 Agent source는 그 실제 시각으로 창을 갱신하되 앞선 fallback 실패를 남긴다.
- 같은 host 단조시계와 공통 effect gate로 의미 채택, Task/권한, 출처 변경, Agent/control 진입 및 게시를 취소 시점에 멈춘다. 이미 전송한 모델 호출의 원시 응답, 사용량과 RSS는 보존하고, 새 호출이나 대기 호출의 전송은 막는다. 실제 네트워크 패킷 전송 시각을 측정했다고 표현하지 않는다.
- 실제 상태 변경 앞에 blocking audit 출력을 끼워 넣지 않는다. 필요한 잠금과 검증을 마친 뒤 마지막 시각 검사, 제한된 상태 변경, 사후 감사 기록 순서로 처리한다. 로컬 파서도 잠금 대기, 초기화 뒤 실제 기동 및 parse 요청 직전에 같은 admission 조건을 검사한다.
- 표시에는 모든 구성에 같은 기술 receipt를 둔다. 고유 publication ID에 대해 host가 실제 관측한 시각을 돌려주며, candidate는 대기 중 상태, 출처, effect, 출력 잠금을 보유하지 않는다. receipt 후 현재 기한, revision, 권한을 재검사한다. 관측된 과거 표시 사실과 현재 revision의 완료 자격은 구별하며, revision 변경만으로 실제 표시한 이력을 없애지 않는다. 그 이력의 이후 재사용에는 현재 권한을 적용한다. 취소 시점 이후 receipt는 원시 표시 사실로 남기되 새 상태 변경을 허용하지 않는다. QA01의 시작, 끝 정의는 유지하고 receipt 왕복의 실제 지연도 QA03의 VIA 경로에 남긴다.
- 관측 종료와120초 사용량 정산을 분리한다. 정산 중 늦은 callback/목표를 성공에 추가하지 않으며, 미확정 사용량을0으로 바꾸지 않는다.
- clock 또는 deadline ACK 실패 같은 기술적 중단에서는 원래 계약 기한, 제안 기한, 마지막 승인 기한을 보존한다. 오류를 감지한 시각과 실제 gate 취소 시각도 구별한다. 실제 gate 취소와 이미 적용된 더 이른 cutoff 중 이른 시각을 별도 `effective_admission_stop_ns`로 기록하고 그 시각+120초까지 정산한다. 오류 감지 시각으로 실제 중단을 소급하거나 늦은 ACK, 종료 처리로 정산 기한을 다시 시작하지 않는다. 이 경우를 정상60초 관측 완료로 취급하지 않으며 캠페인을 보류하고 자동 재시도하지 않는다.
- 공통 참조 형식 안내를 실제 catalog의 ID, 판본, 선언 범위에서 기계적으로 만든다. 출처 신원 정보는 원문 읽기나 외부 전달 권한을 부여하지 않는다. 정답, 미래 사건을 넣거나 기존 무효 출력을 자동 수정하지 않고 엄격한 검증과 최대3회 의미 판단 한도를 유지한다.
- 고정된 작은 실제 모델 호환성 점검을 정식 QA 모집단과 분리해 준비한다. 새315회와 새 기준의 QA04 42개 변경 검사는 이전 세대 결과를 상속하지 않는다.

별도로 발견한 최신 완료 대체 endpoint의 판정 누락은 [승인된 공통 보완 규칙](common9-newer-completion-endpoint-addendum-20261019.md)에 따라 새 평가기에서 검사한다. 실제 같은 Task의 더 최신 RESULT와 연결되는 경우만 대체할 수 있고, 과거 원시에 실제 제공 순서가 불명확한 경우 earliest boundary를 임의 확정하지 않는다. 이전 평가기와 보완 평가기의 차이는 별도 원장으로 남긴다.

기술적 시각, 취소 증거가 불완전하면 실제 출력 원시를 보존하면서 대표 집계의 계측 미확정 건으로 구별한다. QA01/QA03의 관측 평균은 기술적으로 확인된 경계만 사용하고 예정215/115를 유지한다. QA02는 확인된 정답/75와 의미 오답 및 기술적 판정 불가를 분리하며 불완전한 정확도를 완결값으로 표현하지 않는다. 실제 지출은 모두 비용 원장에 남고, 관측이 불완전한 RSS 최고값은 하한 진단이다. 같은 기술 수용 규칙을 모든 구성에 적용하며 정상 계측에서 발생한 모델 실패를 제거하지 않는다.

재개 시 사용자는 단순 구현, 테스트를 **GPT-6.1 Sol High**, 판단, 계획, 해석을 **GPT-6 Astra Ultra**에 맡기도록 명시했다. 기존 구현 및 평가기 검사 담당은 Sol High로 계속하고, 별도 Astra Ultra 담당에게 계약, 측정 경계와 재실행 계획의 독립 판단을 배정했다. 모델 역할 분담 자체를 코드나 측정의 검증 결과로 취급하지 않는다.

### 2.9 공통 보완126회 승인

사용자는 대표9에서 빠진 DP2 독립 요청 중첩과 DP3 작은 밀집 화면 조건을 보완하도록 **공통6조건×7구성×3회, 총126회**를 별도 승인했다. 기존315회와 QA 의미를 유지하고 두 모집단을 합산하지 않는다. 총$40 상한은 이전 사용분, 실패분과 개발 호환성 점검까지 포함한다. Astra Ultra가 입력, 메커니즘 발동 기준, 분모, 실행 순서를 사전 등록하고 Sol High가 구현과 검사를 수행한다. 승인만으로 보완 시험이 구현, 실행되거나 예측이 입증된 것으로 표시하지 않는다. 비용은 정확한 보완 fixture를 구현한 뒤 대표315회와 함께 다시 계산한다.

### 2.10 고정 개발 점검의 외부 전송 승인 대기

수정 코어와 평가기, 최종63개 오프라인 원장을 검토한 Astra Ultra는 고정2사례의 개발 점검만 수용했다. [불변 검토 기록](../../../../../../results/architecture-evaluation/current/common9-poc-20261019/preflight/astra-probe-r2-readiness-20261019.md)과 `compatibility-probe-v3-r2` 동결을 기준으로 root가 제한 실행을 승인했지만, 실제 명령은 자동 승인 검토에서 거절됐다. 이유는 외부 DashScope 대상과 전송 자료에 대한 명시적 사용자 동의가 부족하다는 판단이다. 프로세스, 유료 호출과 추가 비용은 발생하지 않았다. 자동 승인 거절을 다른 실행 경로로 우회하지 않는다.

Sol 담당은 동결한 S02/S18의 전송 자료가 작성된 합성 대화, Task, 문서, 사건과 실험 지시문임을 확인했다. 이 두 사례에는 실제 개인 파일이나 저장소 코드, 화면 frame이 없다. API key는 인증에만 사용하며 모델 본문에 넣지 않는다. [전송 자료 근거](../../../../../../results/architecture-evaluation/current/common9-poc-20261019/compatibility-probe-v3-r2/SYNTHETIC-PAYLOAD-PROVENANCE.json), [대상과 자료 분류](../../../../../../results/architecture-evaluation/current/common9-poc-20261019/compatibility-probe-v3-r2/DESTINATION-AND-DATA-CLASSES.json), [거절 기록](../../../../../../results/architecture-evaluation/current/common9-poc-20261019/compatibility-probe-v3-r2/AUTO-REVIEW-REJECTION-01.json)을 보존한다. root는 전체 승인된 시험의 합성 자료와 작성한 화면 fixture를 DashScope 싱가포르의 Qwen 모델로 전송하는 동의를 요청했다. 응답을 기다리는 동안 별도 보완 조건 설계를 계속한다.

사용자는 이어서 **“해당 시험 자료의 DashScope 전송 승인”**이라고 직접 답했다. 승인 범위는 합성 대화와 Task 기록, 작성한 시험 문서와 화면 fixture 및 실험 지시문이며, 고정 개발 점검2회와 대표315회, 보완126회에 적용한다. 실제 개인 파일, 저장소 코드와 비밀값은 모델 입력에 포함하지 않고 총$40 상한을 유지한다. 이 명시적 동의와 이전 거절을 모두 보존한 뒤 같은 실행 명령을 정상 승인 절차로 다시 요청한다. 외부 전송 동의는 아직 준비되지 않은 정식315/126의 기술 수용을 대신하지 않는다.

## 3. 정식 결과와 별도로 남길 한계

실제 텍스트 callback은 UI proxy이며 본문 전체의 의미 정확성 보증이 아니다. 사용자가 요청한 같은 시험 원칙은 모든 구성에서 같은 입력과 관측 분모를 유지함으로써 확인한다. 음성, Windows 전체 제품, 상용 Agent와 장기간 상태의 미구현 범위는 그대로 남긴다. 파서 배치 비용, 기본 구성AAAA를 바꾼 조건부 효과 및 사전 순위가 없는 QA는 결과에 맞춰 숨기거나 강제 순위로 바꾸지 않는다.
