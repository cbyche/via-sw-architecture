# V5 QA-02/03 정규화와 QA-06 집계 독립 검토

상태: `STATIC_AND_PREFLIGHT_REVIEW`, 공식 A1/B1 완료 trace 및 최종 judge 연결 확인은 대기 중이다. 이 문서는 QA 통과, 점수 또는 A/B 우열을 판정하지 않는다. 검토자는 코드, fixture, oracle, frozen source를 수정하지 않았고 API 호출, 테스트, 후보 실행, 재채점 또는 memory raw 전체 읽기를 하지 않았다. 작은 JSON과 소스의 읽기 전용 대조만 수행했다.

## 고정 근거

- 공식 활동: [V5 manifest](../dp41-p21-mac-text-v5-20261012/manifest.json), source digest `2143b39761bd0b74e8d9d973d8b59c1e8ba722ccc24ebafcc0d7e03d85cda4b0`.
- 요구사항: `ASR-QA-v6`, [현재 QA 정의](../../../../docs/architecture/12-decisions/decision-packages/03-02-quality-attribute-definitions.md) §4.1~4.4, §7.1~7.4 및 canonical workload의 I/T 목록. frozen QA 문서 SHA-256 `ab76bfc813e1f5c8bb72894a705cfcc9495f7f92e021d566dfcbf8224d8d7d65`.
- [실제 V5 preflight 요약](v5-query-label-preflight/execution-summary.json): A/B 각 S2-02와 S2-05가 실행되었고 실패는 보존됐다. `included_in_final_population=false`다. preflight의 관측값을 공식 모집단에 합하지 않는다.

| 검토 코드 | frozen SHA-256 |
| --- | --- |
| `normalize.py` | `e172fcfafea6700cfd4e9e774b5f0289de51dc3df28965e6f8f1346d214fdb83` |
| `metrics.py` | `fe7aadac13ab74e36222eae951c9d0199c9e9b61a13229bf9a2cb4893a11ccc1` |
| `judge.py` | `e96fee7a06176fdbc53fef1c7e9a3cfbfe8cb4485382432e5e6d152f89902119` |
| `evaluate.py` | `6985590abae400b8a251762143d863f67875849f61fab39061a5c64489fa3bb1` |
| `finalize.py` | `7d2420e0142bb199cf5b56cd5f61bba12584ae198a7937a0e227fa3b04273296` |
| `worker.py` | `f144078d212a159e3a3827266bc74aa54fcfef2b2fd126e635efec9ea0ad4dfa` |
| `shared/timing.py` | `6f5275313103926dbbc701bef691f5fc5fb649c48dca2e98aa41b97371b59356` |
| `shared/memory.py` | `9092a854b516a60c89e315f1244ad59603fbd9e86254db3cb1e4574121d0f4b0` |
| `shared/runtime.py` | `4bd5a5b3d3c802cbc0249fe195ebadef34fcccfeb9c389859fe2c4f2a48d35ec` |
| `shared/evidence.py` | `fd04c635d1e4279b636fd5175f5a250708ad8380c84cd0f4fa85da5d6239dba7` |

위 10개 파일은 검토 시점의 live source, V5 `frozen-source`, V5 `evaluator-tools`, manifest 값이 모두 같았다. 이 확인은 소스 동일성 검토이며 실행 성공 검사가 아니다.

## QA-02

`normalize.py:14~29`는 실제 INPUT_STARTED, 실제 speech-end receipt 또는 event ID별 실제 source availability를 시작점으로 사용한다. 예정 `start_at_seconds`를 결과 시각으로 대체하지 않는다. S5-03은 최초 사용자 입력이 없으므로 재연결 발화가 `observed_inputs[0]`이며 FOLLOWUP_SPEECH_END가 그 실제 종료를 참조한다. 현 P21에 대해 이 첫 input 선택은 대응표와 맞는다. 일반 다중 후속 입력의 index 선택을 구현했다는 주장은 하지 않는다.

`normalize.py:38~52`는 Voice ON publication에서 judge가 선택한 정확한 핵심 절이 최종 실제 `voice_text`에 존재해야 하며, 그 시작 문자에 기록된 `voice_char_events`의 실제 onset을 사용한다. Text가 먼저 표시되고 Voice가 시작된 실제 순서에서는 두 시각 중 늦은 경계가 적용된다. 인사 또는 음성의 첫 문자만으로 핵심 사실 전달을 만들어내지 않는다. Voice OFF는 실제 Text 표시의 causal node를 사용한다. local stop는 `LOCAL_STOP` receipt의 실제 경계로 자동 처리한다. 이는 합의된 모사 출력 sink의 관측이며 실제 오디오 device sample을 측정한 값은 아니다.

`judge.py` SYSTEM은 접수와 완료, 업무 주체, checkpoint와 final tail, Text 표시 전 snapshot과 최종 spoken snapshot을 구분한다. `validate_judgement`는 존재하는 evidence/publication ID와 실제 발화 substring을 요구한다. 이 검사는 문장이 해당 endpoint의 모든 사실을 충족했는지 또는 earliest publication인지 자체적으로 증명하지 않는다. 그 의미 판단은 judge와 최종 trace 대조에 남는다. `evaluate.py`는 두 판정의 publication ID, 핵심 절 및 wrong_first 차이를 비교하고 불일치에 세 번째 판정을 사용한다.

`metrics.py:84~109,425~450`은 다음을 분리한다.

| 결과 | 산식과 분모 |
| --- | --- |
| cap 평균 | 잘못된 최초 핵심 반응 또는 미전달은 5초. 나머지는 유효 귀속 시간. 예정 135개를 유지한다. 올바른 6초 반응을 5초로 잘라내지 않는다. |
| 오답 제외 평균 | 유효하며 wrong_first가 아닌 반응만 사용하고 `n`을 공개한다. 후기 수정된 첫 오답도 여기에는 넣지 않는다. |
| 실제 오답 포함 평균 | 실제 최초 핵심 전달의 시각이 있는 관측만 사용한다. 미전달에 가짜 실제 시각을 넣지 않는다. `n`을 공개한다. |
| 후기 수정 진단 | wrong_first, eventually_valid, correction_seconds 및 wall-clock을 보존한다. 첫 오답의 cap을 수정 성공으로 지우지 않는다. |

canonical reaction ID를 21개 scene마다 검증하며 정상 105개, 집중 S5-04 30개 및 local stop 10개 보호 조건을 별도로 계산한다. strict finalization은 누락 또는 중복 ID를 허용하지 않는다. 정확성 630조건을 시간의 분모로 복사하지 않는다.

## QA-03

`normalize.py:77~87`는 해당 사용자 목표의 실제 발화 종료부터 judge가 선택한 필수 완료 publication의 핵심 절까지 연결한다. 보고서 접수 안내를 보고서 final result 전달로 자동 승격하지 않는다. S5-01/02에는 cycle이 없으며 0초 관측으로 만들지 않는다. 현 P21은 19개의 cycle ID를 가진다.

`metrics.py:112~127,451~461`은 귀속 30초 이내의 유효 완료 시간을 원값으로 사용하고, 미완료 또는 30초 이후 완료는 cap 평균에서 30초로 둔다. 95개 분모를 유지한다. 완료 subset 평균에는 실제 늦은 완료 시간도 보존하며 `late_completed_count`와 `incomplete_count`를 구분한다. 이 subset의 분모가 95라는 주장을 하지 않는다. 15초 이내 91개와 미전달 0건 조건은 cap 점수와 별도로 남는다.

`shared/timing.py:EXCLUDED_KINDS,attribute`는 동일 관측 DAG의 EXTERNAL_AGENT_WORK, EXTERNAL_USER_WAIT, EXTERNAL_USER_SPEECH duration만 0으로 놓는다. 실제 parents와 queue 경합 관계를 유지하고 longest path를 다시 구한다. 실제 wall-clock 재구성 일치를 먼저 검사하며 미설명 scheduling gap은 VIA 시간으로 남긴다. 겹친 Agent 구간을 단순 합산하여 빼지 않는다. `finalize.py`는 causal-attribution JSONL과 reaction/cycle CSV에 근거를 저장한다.

## QA-06

V5 manifest의 범위는 `ONE_ISOLATED_WORKER_VIA_COMMON_STUB_AND_RAW_RECORDER`다. 동일 worker process 안에 VIA, 모사 Agent, 기록기가 존재한다. cloud 모델 weights/KV, 별도 evaluator 및 campaign supervisor의 메모리는 VIA worker OS 합에 포함하지 않는다. 이는 전체 제품 UI나 실제 Agent process 배치를 측정한 범위가 아니다.

`shared/memory.py`는 Darwin의 TASK_VM_INFO resident_size와 phys_footprint를 읽고 50ms 간격으로 PID, generation, sample_id, phase 및 실제 수집 시작/끝 시각을 저장한다. 샘플러는 observation buffer와 20MiB 초기 원본을 만들기 전에 시작한다. worker는 PREPARATION, ACTIVITY, DRAIN 30초, IDLE 10초까지 샘플러를 유지한다. 실제 touched RGBA 15개 buffer와 실제 직렬화 copy의 allocation 원장은 설명용이며 OS 메모리에 bytes를 다시 더하지 않는다. shared Python 원본은 OS 값에 한 번 반영되며 실제 copy는 실제 process allocation으로 반영된다.

`metrics.py:130~215`는 repeat와 sample_id가 같은 process 관측을 합산한 뒤 순간별 합의 최대값을 구한다. 서로 다른 process peak를 합치지 않는다. 중복 PID/generation은 거절하고 명시된 expected PID 누락과 수집 실패는 0으로 바꾸지 않는다. 다섯 활동의 peak 평균과 전체 최대값을 함께 제공하며 주 값은 전체 최대 phys_footprint다. RSS도 별도 peak를 제공한다. idle 종료-시작이 음수이면 증가 0을 사용하며 idle peak도 전체 peak에 포함된다.

Mac process OS 합은 `OS_PROCESS_SUM_NOT_UNIQUE_PHYSICAL_RAM`이고 Windows commit 또는 working set과 동일하지 않다. `windows_commit_score=null`, `windows_commit_equivalence=false` 및 Mac PoC 기존 MiB 경계 적용이라는 근거를 보존한다. 현재 한 PID 범위에는 다중 process 공유 페이지 중복 문제가 없지만 이 집계기를 독립 service 구조의 고유 shared physical RAM 측정기로 일반화하면 안 된다.

메모리 수집 실패가 있을 때 `incomplete_samples`는 공개되지만 `complete_repeats`는 그 반복에 유효 peak가 하나라도 있는지를 센다. 따라서 값이 산출됐다는 사실만으로 50ms 관측 누락이 없다고 주장할 수 없다. 공식 raw 종료 뒤 수집 실패와 sample 간격, phase coverage를 별도로 확인해야 한다. 지금은 메모리 raw 전체를 읽지 않아 그 실행 확인을 완료했다고 표시하지 않는다. 실제 누락이 확인되지 않은 이 사항을 후보 코드 P2로 판정하지 않는다.

## 실제 preflight 경계 확인

다음은 판정용 정답이 아닌 실제 실행 데이터의 shape와 clock 연결 확인이다. 각 raw line SHA는 마지막 newline을 포함한다.

| tuple | raw line SHA-256 | 실제 전달 특성 |
| --- | --- | --- |
| A/preflight/S2-02 | `7d36bd0f53a7f3b40c813b864e1f5e37d1b830ccc7299827ba8b668d4415e278` | 첫 질문은 whole Voice와 receipt가 존재한다. 뒤 ACCEPTED 응답은 Voice가 `CREATE_PROPOSAL 업무는 ACCEPTED(접수)`까지 관측되고 receipt=false다. |
| A/preflight/S2-05 | `c69d891eac74e4e43e7e81a8cd821f43686bf28af8a95959d270155d20cc9e7c` | CREATE_TABLE 접수와 실제 완료의 두 publication이 있다. 제안서 요청 endpoint와 다른 operation의 진실한 완료를 구분해야 한다. |
| B/preflight/S2-02 | `61dcfb377861606e2576f965db580b90678cc670880996f43df2c8ede626167f` | 후보 목록 질문의 Text와 Voice onset이 존재하지만 whole Voice 이전 timeout으로 receipt=false다. actual heard criteria가 label에 있고 background 후보도 남았다. |
| B/preflight/S2-05 | `a68518c1f28e41d187a7bc8a66577abcbe1b8cc8c764914c8f2f3ed6b3a6d78e` | background 후보 목록 Voice 일부가 실제 재생됐고 receipt=false다. onset이 있다는 사실만으로 올바른 제안서 접수 endpoint를 만들면 안 된다. |

네 row의 모든 기록된 Voice 문자 시각은 해당 causal node의 absolute 시각과 일치했고 Text는 Voice보다 먼저 표시됐다. 이것은 시간 연결의 실제 근거다. 어떤 문장이 정답이거나 유효 endpoint인지의 최종 판정은 수행하지 않았다. 특히 partial Voice의 필수 절이 실제 전부 발화됐는지, 정확한 operation과 source scope가 효과에 연결됐는지, 접수/완료를 구분했는지는 최종 judge 및 trace 대조가 필요하다.

## 현재 결론과 남은 확인

이 읽기 범위에서 확정된 P1/P2는 없다. 계약의 분모, 요청한 보조 평균, 실패 cap, 실제 core clause clock 및 Mac OS 지표 분리는 구현에 존재한다. 실제 의미 endpoint 선택은 아직 확정하지 않았다. 공식 A1/B1 완료 후 다음을 같은 원시 근거로 확인한다.

1. 실제 owner read, adopted meaning, command/Agent payload와 endpoint가 같은 목표 및 scope를 가리키는지 확인한다.
2. filler 또는 잘못된 빠른 반응을 first actual과 valid로 구분했는지 확인한다. 후기 수정으로 wrong_first를 삭제하지 않는다.
3. receipt-only checkpoint와 final tail, Voice OFF Text와 Voice ON 실제 발화, 동시 source 사건 및 늦은 반환을 별도로 확인한다.
4. 두 judge 결과가 일치하더라도 중요한 실제 payload와 필수 조건을 최종 trace로 대조한다. 판정 수정이 필요하면 원판정과 증거, 이유를 보존하며 후보 결과를 바꾸지 않는다.
5. timing worker 종료 뒤 memory 수집 실패, 실제 간격 및 전 phase coverage를 확인하고 실제 누락이 있으면 관측 한계를 공개한다.
