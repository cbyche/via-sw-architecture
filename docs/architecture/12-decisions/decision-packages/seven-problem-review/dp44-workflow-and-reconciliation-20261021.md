# DP44 구현 대안: 응답별 전달 절차와 대화 출력 상태의 대조

> **후속 사용자 정정:** 이 pair는 구현 구조의 차이는 있으나 요구한 QA 교환을 확보하지 못했다. 주요 A/B로 권고하지 않고 설계 탐색 이력으로 보존한다. 현재의 구체적인 비용 비교는 [음성 선행 준비 대 발화 시점 준비](dp44-audio-preparation-tradeoff-20261021.md)의 QA03 대 QA05다. QA01은 그 비교에서도 동점이다.

> 2026-10-21 / 실제 구현할 수 있는 기본안 A/B의 설계 제안 / 미구현·미측정·사용자 미선정
>
> [문제 경계 재정립](dp44-conversation-delivery-reset-20261021.md)을 이어 실제 상태와 동작을 구성했다. A는 응답별 전달 절차, B는 원하는 출력 상태와 실제 상태의 대조를 실행 원본으로 삼는다. 두 안의 현실적인 선택 이유는 있으나 현재 QA 3~4축의 큰 교환이나 QA01 실측 우위를 확정한 문서는 아니다. 기존 Dispatcher/Stage A/B의 이름·결과를 계승하지 않는다.

## 1. 비교 질문과 설계 판단

**준비된 응답을 전달하는 절차를 응답마다 실행할 것인가, 대화에 유지할 출력 상태를 기록하고 실제 출력이 그 상태를 따르도록 만들 것인가?**

- **A 응답별 전달 절차:** 응답마다 짧은 상태 전이와 비동기 작업으로 구성된 Delivery Workflow를 둔다. 이 실행체가 표시·음성 준비·중단·교체 명령을 직접 결정한다.
- **B 대화 출력 상태의 대조:** 전달할 내용과 의무를 Desired Delivery Manifest에 기록한다. Reconciler가 실제 표시·준비·재생 상태 및 실행 중 작업과 대조하여 필요한 명령만 발행한다.

일반적인 비동기 작업을 조립하는 개발자는 A를 선택할 수 있다. 수정과 철회가 잦은 출력 상태를 여러 채널에 유지하는 개발자는 B를 선택할 수 있다. 어느 쪽도 모델에게 응답을 다시 작성시키거나 사용자 의미를 다시 해석하지 않는다.

**당시 권고는 이 pair를 구현 설계 비교의 기본안으로 검토하는 것이었으나, 사용자 재검토 뒤 해당 권고를 철회했다.** 진행 상태의 원본, 명령의 생산 방식과 복구 계약이 달라도 실제 QA의 교환을 설명하지 못하면 이번 DP 요구를 충족하지 않는다.

## 2. 양안의 공통 입력과 필수 동작

```text
DeliveryItem {
  response_id, conversation_id, request_id, task_id?, question_id?,
  content_ref, content_revision, source_refs,
  detail_text_ref, speech_content_ref?, required_channels
}

AdoptedControl {
  control_id, affected_response_refs, adopted_revision,
  kind: hold | release | replace | withdraw | resume,
  affected_scope
}
```

이는 설계용 자료형이다. 새 Agent wire 계약이나 실제 구현된 API라는 뜻은 아니다. `speech_content_ref`는 공통 음성 경로에 전달할 내용이며 A/B별로 다른 요약량을 주지 않는다. S2S의 고정 내용 발화·취소·전달 위치 연동 capability는 실제 선택 모델에서 확인해야 한다. 이 비교를 위해 제3의 TTS 모델이나 응답 재작성 LLM을 추가하지 않는다.

- Agent Gateway·Task Manager에 해당하는 공통 연동이 외부 Task 사실을 확인한다. 요청 의미·정정 범위는 해당 의미 판단 owner가 채택한다. 이 권한을 전달 엔진이 가져오지 않는다.
- 응답 본문이 준비됐다는 사실만 입력으로 고정한다. 대화 연결과 게시 조건을 적용하는 내부 작업까지 이미 완료됐다고 가정하지 않는다.
- 첫 상세 표시와 음성 준비는 독립적으로 시작할 수 있다. 음성을 만들거나 앞 응답을 다 말할 때까지 첫 UI 표시를 기다리지 않는다.
- 모델·네트워크 작업은 비동기다. 같은 모델 서비스의 다중 접속, 같은 전체 수용량, 같은 내용·음성 품질과 같은 원본 저장 경계를 사용한다.
- 이미 전달한 범위, 음성만 미전달인 범위, 정정으로 무효가 된 미전달 범위를 구별한다. 한 질문의 사용자 답변 대기가 다른 응답의 처리를 막지 않는다.
- 출력 직전 현재 판본·권한·출력 epoch를 검사한다. 사용자 발화 시작 때 장치의 음성 중단은 양쪽 공통으로 즉시 처리한다.
- 모델의 취소 요청, 실제 취소 완료, 늦은 결과는 다르다. 중단 전 실제 전달 내용은 지우지 않는다.

공통 저수준 작업은 `ShowText`, `PatchText`, `PrepareAudio`, `CancelAudio`, `PlayAudio`, `StopAudio`와 실제 출력 receipt다. 두 안은 같은 작업을 어떤 상태에서 누가 결정하는지가 다르다.

## 3. A: 응답마다 전달 절차를 실행한다

### 실행과 상태

응답 R1, 질문 Q1, 결과 R2에 각각 Delivery Workflow가 있다. 같은 구현 코드를 사용하는 인스턴스이며 응답마다 새 Module·thread·모델 가중치를 만드는 것이 아니다.

```text
R1 사건 → Delivery Workflow R1 ─┐
Q1 사건 → Delivery Workflow Q1 ─┼→ 공통 출력 Guard → UI / 음성 출력
R2 사건 → Delivery Workflow R2 ─┘
          ↑ 실제 완료·전달 receipt
```

각 Workflow의 지속 상태는 다음과 같다.

```text
DeliveryProgress {
  response_ref, revision,
  text_status, audio_status, hold_reason,
  active_operations, prepared_audio_ref,
  remaining_delivery_scope, last_confirmed_receipt
}
```

상태의 의미는 **이 응답을 전달하기 위해 어떤 작업까지 요청하고 확인했는가**다. 원본 본문과 음성 payload는 참조하며 복제하지 않는다. 완료된 Workflow는 정리하고 전달 기록은 공통 원장에 남긴다. 오랫동안 대기하는 Workflow의 snapshot 보존·실행체 해제도 가능하며, 계속 살아 있는 task가 A의 필수 조건은 아니다.

### Q1 하나가 도착했을 때

1. Router가 Q1의 Workflow를 생성하거나 찾아 사건을 전달한다.
2. Workflow가 업무·대화 연결과 게시 조건을 적용한다.
3. 직접 `ShowText(Q1)`을 발행하고, 별도 child task로 `PrepareAudio(Q1)`을 시작한다.
4. Workflow는 API 전체가 끝날 때까지 mailbox를 막지 않는다. 제어 사건과 준비 완료 사건을 계속 받는다.
5. 실제 UI receipt를 기록한다. 음성이 준비되면 현재 판본을 확인하고 Speech Floor에 재생을 요청한다.
6. Speech Floor는 실제 음성 출력권만 부여한다. Q1의 진행 절차를 대신 결정하지 않는다.

의사 코드:

```text
on Arrived(item):
  bind_and_check(item)
  issue(ShowText(item))
  start_child(PrepareAudio(item))

on AudioPrepared(job, artifact):
  if matches_current_revision(job):
    remember(artifact)
    request_speech_turn(artifact)

on Replace(adopted_change):
  update_revision_and_delivery_scope(adopted_change)
  cancel_affected_operations()
  issue_required_text_changes()
  start_only_required_audio_work()
```

정정의 구체적인 명령 생산과 진행 상태 변경은 Workflow 로직에 있다. 공통 helper, handler 등록과 동일한 자료형으로 중복 코드를 줄일 수 있다. A를 거대한 switch나 응답별 중복 구현으로 제한하지 않는다.

### 실제 구현자가 A를 고를 이유

- 도착한 응답에서 실행할 동작까지의 경로가 직접적이다. Q1 전달 문제를 Q1의 진행 기록과 작업으로 추적할 수 있다.
- 기본 처리가 표시·음성 준비·재생·중단으로 비교적 안정적이면 범용 출력 대조 엔진을 먼저 만들 필요가 없다.
- 독립 응답의 후처리를 기존 Workflow handler나 child task로 확장할 수 있다.

비용은 여러 응답·여러 진행 단계에 영향을 주는 변경이다. 예를 들어 원본 구간 철회는 영향받는 Workflow들, 준비 작업과 대기 출력에 제어를 전달해야 한다. 후속 입력이 사용할 실제 전달 뷰도 개별 진행 기록과 함께 일관되게 유지해야 한다.

## 4. B: 원하는 출력 상태와 실제 상태의 차이를 실행한다

### 실행과 상태

응답을 받으면 실행 절차를 새로 시작하는 대신, 대화에 남아 있어야 할 내용과 전달 의무를 갱신한다. 작업이 끝나면 다음 절차를 이어가는 대신 실제 상태를 갱신하고 다시 대조한다.

```text
응답·채택된 정정 → Manifest Updater → Desired Delivery Manifest
                                             ↓
                         Reconciler(desired, actual, in_flight)
                                             ↓
                                     비동기 Effect Workers
                                             ↓
                                 공통 출력 Guard → UI / 음성
                                             ↓
                              실제 receipt → Actual Delivery Ledger
```

```text
Desired Delivery Manifest {
  response별 content nodes와 revision,
  현재 UI에 유지할 내용,
  유효한 음성 전달 의무와 보류 상태,
  source dependencies
}

Actual Delivery Ledger {
  실제 표시된 내용과 revision,
  준비된 음성 참조,
  실제 재생·중단 범위,
  확인되지 않은 전달 상태
}

InFlightEffects {
  operation_id, target_revision, node_ref, worker_handle
}
```

`content node`는 내용 참조·판본·채널 의무를 붙인 출력 단위다. 원본 보고서를 이해하는 새 의미 구조가 아니다. Agent가 제공하지 않은 문단 ID나 source 구간을 B만 공짜로 안다고 가정하지 않는다.

### Q1 하나가 도착했을 때

1. Manifest Updater가 같은 업무·대화 연결과 게시 조건을 적용한다.
2. Q1을 `화면 표시 필요`, `음성 전달 필요` 상태로 등록한다.
3. Reconciler가 Q1의 desired·actual·실행 중 작업을 대조한다.
4. 아직 표시되지 않았고 같은 표시 작업도 없다면 `ShowText(Q1)`을 발행한다. 음성 준비도 독립적으로 발행한다.
5. Effect Workers가 UI·모델 작업을 병렬 실행한다. Worker가 다음 전달 절차를 결정하지 않는다.
6. receipt와 준비 완료가 actual 상태를 바꾼다. 변화한 Q1만 다시 대조해 다음 필요한 효과를 결정한다.

의사 코드:

```text
on Arrived(item):
  update_manifest(bind_and_check(item))
  mark_dirty(item.response_id)

on AudioPrepared(result):
  adopt_artifact_if_current_else_dispose(result)
  mark_dirty(result.response_id)

on ActualDeliveryReceipt(receipt):
  append_historical_delivery_fact(receipt)
  update_observed_output_state_if_applicable(receipt)
  mark_dirty(receipt.response_id)

reconcile(response_id):
  desired = manifest[response_id]
  actual = ledger[response_id]
  running = in_flight[response_id]
  effects = difference(desired, actual, running)
  register_and_start(effects)
```

주기적 polling, 매번 전체 대화 scan, 전체 상태 복사, 별도 추가 DB transaction은 필수로 넣지 않는다. 변경된 ID를 dirty set에 넣어 즉시 처리한다. 모델·디스크 I/O를 reducer나 대조 루프 안에서 기다리지 않는다. 같은 operation이 완료되기 전에 중복 발행하지 않도록 in-flight 상태도 대조한다.

### 정정은 어떻게 다른가

Q1이 철회되면 `Stop Q1 → cancel job → remove queue entry → correct view`라는 절차를 외부 사건 handler가 직접 나열하는 대신, Q1의 유효한 미전달 의무를 제거한다. Reconciler가 현재 준비·대기·출력 상태와 대조해서 필요한 취소·제거 명령을 만든다.

이미 표시했거나 실제로 말한 과거 사실은 actual 원장에 남긴다. desired에서 제거됐다는 이유로 역사적 전달 기록까지 삭제하지 않는다. 구판 음성의 준비 완료는 폐기할 수 있지만, 실제 표시·재생 receipt는 늦게 도착해도 기록한다. 늦은 receipt가 현재 desired를 구판으로 되돌리거나 이미 확인한 최신 화면 상태를 덮어쓰게 하지 않는다. A도 같은 실제 전달 사실 보존 규칙을 따른다. 화면의 현재 판본 갱신과 과거 전달 기록 보존을 분리한다.

### 실제 구현자가 B를 고를 이유

- 정정·철회·채널 변경을 출력 상태의 변경으로 표현할 수 있다. 실행이 어느 단계까지 갔는지에 따른 후속 명령은 공통 대조 엔진이 도출한다.
- 대기 중인 응답은 데이터다. 실제 작업이 필요한 동안만 Effect Worker가 동작한다.
- 재연결·복구 때 원하는 상태와 확인된 실제 상태를 다시 대조할 수 있다. 특정 callback의 실행 위치를 복원할 필요가 적다.

대가로 manifest, actual 대응, in-flight registry와 diff 규칙을 기본 설계부터 구현·유지한다. 독립적인 응답 하나에도 이 경로가 참여한다. 잘못 설계한 diff가 반복 효과를 만들 수 있으므로 연산 identity, 판본, 실패와 UNKNOWN 처리가 필요하다.

## 5. 같은 사건을 끝까지 비교한다

R1의 상세 텍스트는 표시됐고 음성은 일부 전달됐다. 다른 메일 Task의 질문 Q1이 도착한다. 사용자가 끼어들고, 의미 판단 owner가 R1 정정 관계를 채택한다.

| 사건 | A 응답별 전달 절차 | B 출력 상태 대조 |
| --- | --- | --- |
| Q1 도착 | Q1 Workflow가 표시와 음성 준비를 직접 시작 | Manifest에 Q1을 추가하고 해당 항목 대조로 효과 발행 |
| Q1의 음성 대기 | Q1 진행 상태와 준비된 참조를 유지 | Q1의 미전달 의무·준비 상태를 유지 |
| 사용자 발화 시작 | 공통 로컬 stop. R1 Workflow에 실제 중단 범위 반영 | 같은 stop. Actual Ledger에 범위 반영하고 speech 보류 상태 갱신 |
| R1 정정 채택 | R1 Workflow가 영향 범위와 진행 단계에 맞는 교체·취소 명령 결정 | R1의 desired 내용·미전달 의무 변경. actual·in-flight 차이로 교체·취소 도출 |
| R1 구판 준비 결과 도착 | Workflow의 현재 revision과 비교해 폐기 | 현재 desired와 대응하지 않는 완료를 폐기 |
| 관련 없는 Q1 | Workflow와 질문 원본 유지 | Manifest의 Q1과 실제 전달 상태 유지 |
| 후속 “응” 입력 | 공통 실제 전달 뷰에서 입력 시점까지 제시된 질문 근거 제공 | 같은 실제 전달 사실의 뷰 제공 |

후속 의미 판단에는 화면 표시와 실제 음성 범위를 구분해 준다. 화면 표시를 독해나 승인으로 간주하지 않는다. 질문이 여러 개라면 어느 질문의 답인지 DP2가 판단하거나 확인한다.

### 실패·복구 공통 조건

- A는 확정한 진행 snapshot과 operation 상태에서 재개하고, B는 desired·actual·in-flight를 다시 대조한다.
- 양쪽 모두 같은 내구 기록·현재성 검사를 사용한다. 이것은 DP42의 저장소·서비스 경계 변경이 아니다.
- 장치에 전달됐으나 receipt가 없는 경우는 UNKNOWN이다. `actual 없음`을 `아직 재생 안 됨`으로 해석해 음성을 무조건 재생하지 않는다. 장치 상태 확인 또는 공통 복구 정책을 적용한다.
- 재시작 후 immutable 내용·준비된 유효 artifact를 재사용할 수 있다. A의 재사용을 제거해서 B를 유리하게 만들지 않는다.

## 6. 구조 차이가 유지되는 기준

| 질문 | A | B |
| --- | --- | --- |
| 진행을 설명하는 원본 | 응답별 수행 단계와 발행·확인한 operation | 원하는 출력 의무와 확인된 실제 상태 |
| 새 명령을 만드는 주체 | 사건을 처리하는 Workflow의 로직 | desired·actual·in-flight를 대조하는 Reconciler |
| 정정의 표현 | 관련 실행 상태를 바꾸고 필요한 명령 발행 | 원하는 상태를 바꾸고 필요한 명령을 대조로 도출 |
| 복구의 출발점 | 확인된 실행 진행과 미완료 operation | 현재 desired와 확인된 actual 사이의 차이 |

A에 공통 조회 view나 helper를 추가해도 개별 Workflow가 절차를 결정하면 A다. B의 Worker가 비동기 task라는 이유만으로 A가 되지도 않는다. 반대로 A의 모든 다음 동작을 desired·actual의 차이로 결정하게 바꾸면, 함수 하나를 추가한 것이 아니라 상태 모델과 진행 결정권을 B로 옮긴 것이다.

단순한 `reduce(state,event)->effects`만 있고 효과의 재대조·수렴·복구 계약이 없다면 B의 핵심을 구현한 것이 아니다. 이는 중앙 Dispatcher의 다른 표현에 그칠 수 있다.

## 7. QA 손익과 검증 가능한 예측

### 가장 유력한 교환

**A는 직접적인 정상 실행과 작은 범위의 진행 모델을 택한다. B는 출력 변경을 공통 상태 대조로 처리하는 일반성을 미리 구축한다.** 이 선행 비용이 현재 QA01/06에서 얼마나 남고 C04/C06의 QA04 변경 범위를 얼마나 줄이는지가 주 비교다.

| QA | 기본안의 구조적 예상 | 현재 근거 수준 |
| --- | --- | --- |
| QA01 첫 응답 | A는 연결·검사 뒤 직접 표시 명령. B는 manifest 반영·대조가 추가된다. 단순 응답에서는 A의 경로가 직접적 | 짧은 코드 비용이므로 큰 차이를 예고하지 않음. 실제 시계는 입력/Agent source부터 실제 UI 표시까지 유지 |
| QA02 지칭 정확도 | 같은 의미 결과와 실제 전달 근거 사용 | 기본 차이 주장 없음 |
| QA03 VIA 처리시간 | 같은 상세 표시와 음성 준비를 병행. B의 대조 비용과 변경 시 재사용 경로, A의 제어 전파를 비교 | 보편적 승자 미확정. 이전 음성 재생 차례를 B 비용으로 세거나 A의 동일 artifact 재사용을 금지하지 않음 |
| QA04 변경 모듈 수 | B의 keyed 상태 대조가 부분 변경을 기존 엔진으로 흡수하는 경우 유리. A는 직접 명령과 진행 처리의 확장으로 대응 | 가장 유력한 B 이익 가설. 아래 공통 추가 작업과 실제 Rust diff로 확인 필요 |
| QA05 전체 모델 비용 | 같은 준비 내용을 같은 음성 경로에 전달 | 기본 동점 가설. 실제 생성량 차이가 없는 한 B의 비용 절약 주장 없음 |
| QA06 최대 로컬 RAM | A 진행·작업 상태 대 B manifest·실제 대응·diff index. B의 파생 상태 유지가 추가 비용 후보 | A의 여러 실행체 비용도 포함. payload 참조 공유와 실행체 해제를 허용하며 큰 차이·고정 승자 미확정 |
| 새 QA07 요청 의미 정확도 | 의미 판단 자체는 동일. 전달 근거가 이후 입력에 미치는 경로는 공통 기능으로 보존 | 사후 전달 오류를 이미 채택된 의미의 오류로 재분류하지 않음 |

### QA04를 구체화하는 변경 영향

[현재 QA04](../qa-candidate-package/qa04-change-scope.md)의 C04는 전체 답변 교체에 `section_id` 지정 문단 교체를 추가하는 변경이다. C06은 문서 전체 사용 철회에 `source_id + extent` 지정 구간 철회를 추가한다. 두 변경은 같은 초기 기본안에서 각각 독립 수행한다.

| 변경 | A에서 확인할 변경 | B에서 확인할 변경 | 양쪽 공통으로 추가할 수 있는 작업 |
| --- | --- | --- | --- |
| C04 부분 결과 정정 | 지정 문단에 대응하는 진행·작업 참조와 부분 명령 생성, 관련 늦은 완료 처리 | 외부 section_id를 keyed node로 연결하고 해당 desired revision만 변경. 기존 diff가 해당 단위 교체를 지원하는지 확인 | 문단 ID 전달, 부분 UI patch, 해당 문단과 음성 구간의 대응 |
| C06 부분 사용 철회 | 영향받는 Workflow·준비·대기 작업을 찾아 해당 범위의 제어 명령 발행 | source extent와 겹친 node의 사용 조건 변경. 기존 diff의 취소·교체 재사용 가능성 확인 | 구간 근거 생산·전달, 겹침 판정, 최종 권한 검사 |

B의 일반 node와 diff 엔진은 초기 비용이다. 이미 부분 동작까지 지원한다면 그 구현을 기준선에 공개해야 한다. 외부 문단·구간 정보를 뒤늦게 추가해야 하는 일을 무료로 보지 않는다. A도 공통 helper와 기존 부분 대응을 재사용한다. 두 안이 같은 Module에서 변경을 흡수하면 QA04는 동점으로 남긴다. 설계 문단이나 파일을 여러 개로 쪼개서 A의 변경 수를 늘리지 않는다.

### 반증 조건

1. 같은 실제 입력에서 QA01/06 차이가 작고 C04/C06의 실제 변경 범위도 같으면, 구조는 유효해도 과정에서 요구한 강한 QA 교환을 확보한 DP라는 주장은 기각한다.
2. A를 전체 응답 동기 실행으로 만들거나 B에 추가 polling·DB 왕복을 넣어야 시간 차이가 나오면 잘못된 비교다.
3. B의 node 모델이 C04/C06 때문에 엔진·Worker·원장까지 확장돼야 한다면 예상 변경 국소화 이익을 낮춘다.
4. B에만 자동 합치기·음성 준비 지연을 적용하거나 A에만 선행 음성 생성을 강제하면 독립적인 음성 준비 축을 섞은 것이다.

## 8. 이번에 주 비교로 택하지 않은 두 축

- **음성 선행 생산과 발화 시점 생산:** 준비 job·artifact 보존 대 현재 producer·bounded stream이라는 실제 선택이다. QA03/05/06의 별도 교환 후보지만 위 A/B 어느 쪽에도 붙일 수 있다. 이를 묶어서 새 A/B의 고유 비용으로 세지 않는다.
- **SpeechHandle에 text/audio를 함께 맡기기와 UI 게시 분리:** 실제 음성 SDK에서 존재한다. 다만 VIA는 이미 상세 Task UI와 실제 음성 범위를 구별해야 한다. A에만 준비된 text를 발화 차례 뒤로 묶어 큰 QA01 손해를 만드는 안은 이번 주 비교로 채택하지 않는다. 독립 UI 추가의 크기가 실제 구현에서 작은지 큰지 확인하지 않고 강한 대안이라고 부르지 않는다.

## 9. 구현 패턴의 근거와 적용 한계

| 공식 원자료 | 확인한 메커니즘 | 이번 VIA 설계에 적용한 부분 |
| --- | --- | --- |
| [XState Invoke](https://stately.ai/docs/invoke) | 상태와 수명이 연결된 비동기 actor, 부모와 사건 통신 | A의 응답별 비동기 진행 구현 가능성. XState 도입·성능 우위 주장은 아님 |
| [Redux Side Effects](https://redux.js.org/usage/side-effects-approaches) | 상태 전이와 외부 비동기 효과 실행 분리 | B의 상태 변경과 Worker 분리. reducer만으로 desired/actual 대조를 구현했다고 보지 않음 |
| [Kubernetes Controllers](https://kubernetes.io/docs/concepts/architecture/controller/) | 원하는 상태와 현재 상태를 대조해 차이를 줄이는 제어 | B의 명령 생산·재대조 패턴. VIA에 Kubernetes나 주기적 polling을 설치한다는 뜻이 아님 |
| [LiveKit Speech and audio](https://docs.livekit.io/agents/multimodality/audio/) | 주어진 발화의 전달과 SpeechHandle을 통한 상태·중단·재생 완료 관리 | 내용 생성과 실제 음성 전달의 구별. 임의 S2S의 동일 capability 보장은 아님 |
| [LiveKit AgentActivity 원본](https://raw.githubusercontent.com/livekit/agents/main/livekit-agents/livekit/agents/voice/agent_activity.py) | 공급된 text를 처리하는 경로에서 speech 승인 뒤 TTS/text forwarding, 완료·중단 기록 처리 | §8의 통합 speech lifecycle이 실제 구현에 있음을 확인. 이 TTS 경로를 VIA S2S 구현이나 성능 실측으로 바꾸지 않음 |

확인일 2026-10-21. 원자료는 패턴의 구현 가능성을 뒷받침하며 VIA의 QA 수치나 우위를 제공하지 않는다. 세 독립 검토에서 A/B의 상태·실행 원본 차이, QA04의 조건부 이익, QA01/06 효과 크기의 한계를 대조했다. 작성 후 검토에서 구판 준비 완료와 늦은 실제 전달 receipt를 구별하라는 지적을 반영했다. 준비 완료는 폐기할 수 있어도 실제 전달 사실은 보존한다. 로컬 링크와 Markdown 코드 fence를 확인했다. 제품 구현, 모델 호출, 성능 측정, QA 점수 변경, ADR 채택이나 commit/push는 이번 작업에 포함하지 않았다.
