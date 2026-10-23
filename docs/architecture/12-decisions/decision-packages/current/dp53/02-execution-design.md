# DP53 실행, 상태와 자원 설계

[진입점](README.md) / [사건 원장](01-scenarios.md) / [구조 선택](03-structural-choices.md) / [근거](04-evidence-and-review.md)

코드는 Rust 형태의 의사코드다. import, 오류형, 트랜잭션 구현과 직렬화는 생략했다. 실행하거나 컴파일한 제품 코드가 아니며, `.await` 위치와 소유권/확정 경계를 검토하기 위한 자료다.

## 1. 판본과 실제 실행을 구분한다

```rust
struct RequestVersion { request: RequestId, revision: u64 }
struct RunKey {
    request: RequestVersion,
    attempt: AttemptId,       // 같은 판본의 재시도도 다르다.
    incarnation: ProcessId,   // 재시작 전 callback과 구분한다.
}
struct Command {
    id: CommandId,
    based_on: RequestVersion,
    target: Option<AgentExecutionId>,
    purpose: CommandPurpose,
    payload: Arc<CommandPayload>, // 생성 후 불변
}
enum CommandPurpose { FulfilCurrentRequest, ReconcileOldExecution }
enum Transmission {
    Pending, Withdrawn,
    Dispatching { attempt: SendAttemptId },
    Acknowledged { execution: AgentExecutionId },
    Rejected, Unknown,
}
```

Request revision, 실제 run attempt, Agent execution, command, 음성 playback attempt는 서로 다른 identity다. r3 채택 후에도 r2 외부 실행의 조회/취소는 필요하므로 ReconcileOldExecution을 구판이라는 이유로 지우지 않는다. 과거 외부 실행의 실제 상태와 현재 사용자의 요구를 한 status 필드로 합치지 않는다.

## 2. 모듈과 의존 방향

| 소유 Component와 내부 모듈 | 소유 상태, 호출과 권한 |
| --- | --- |
| Request Controller의 RequestAuthority | 현재 요청, 기준 입력, attempt, hold와 채택. worker는 제안만 반환하고 이 모듈이 검증 |
| Request Controller의 RunSupervisor | run별 token, JoinHandle과 종료 수거. 입력 처리 함수는 이들의 종료를 await하지 않음 |
| Context Manager의 읽기 어댑터 | 읽기 작업, source 판본, 원본 버퍼와 자료 lease |
| Model Access의 호출 관리자 | 실제 호출/슬롯, provider ID, callback, 사용량과 종료. 요청별 모델 가중치 복제 없음 |
| Task Manager | Task와 Agent Execution 사실, 정정/취소 의미, command 내용 |
| Agent Gateway | 명령 전송 상태, 어댑터 연결, inbox/outbox, 외부 확인 |
| Response Manager | 응답 내용 판본, 게시 대기와 채널별 전달 상태 |
| Interaction Manager | 장치 출력과 입력, output epoch와 재생 receipt |

내부 모듈 이름은 이번 탐색용이며 새로운 Reference Architecture Component로 채택하지 않는다. worker는 `PreparationPorts`를 받아 읽기/모델만 호출하며 Agent 송신 객체나 현재 상태의 가변 참조를 받지 않는다. Task/Response 변경은 각 owner가 만든 검증된 patch를 공통 transaction에 참여시키며 RequestAuthority가 타 owner schema를 직접 쓰지 않는다. 아래 `tx.*`는 그 Unit of Work의 축약 표기다.

## 3. 한 실행의 생성과 반환

감독자는 run과 token을 먼저 등록하고 자원 한도 안에서 worker를 spawn한다. 등록과 정정이 경합하면 등록된 run이 취소되거나 구판 등록이 거절된다. spawn 직전 또는 Model Access 큐에서 대기 중 정정되어도 token/현재 허가를 시작 시 재확인한다. 구판 read-only 연산이 경합 중 시작될 가능성과 비용은 남지만 결과 확정은 허용하지 않는다.

```rust
async fn prepare(run: RunKey, input: Arc<RequestSnapshot>,
                 stop: CancellationToken, ports: PreparationPorts) -> RunExit {
    let document = ports.context
        .read(input.document.clone(), stop.child_token()).await?;

    let mut call = ports.model
        .start(run, input.clone(), document, stop.child_token()).await?;

    tokio::select! {
        _ = stop.cancelled() => {
            call.withdraw_interest(); // 소비 철회와 실제 취소 요청
            RunExit::Superseded
        }
        result = call.result() => RunExit::Proposal(result?)
    }
}
```

`read/start`는 취소 중인 큐 대기에서 빠져나올 수 있는 계약이다. 실제 연산 시작 후 lease가 drop되어도 호출 관리자는 underlying 작업과 비용/종료 관측을 잃지 않는다. 정상 반환과 취소가 동시에 준비되면 어느 select 분기가 선택되어도 최종 채택 검사가 구판 확정을 막는다. token 검사를 correctness의 유일한 근거로 삼지 않는다.

## 4. 정정과 결과 채택의 원자적 경계

입력 시작 hold와 아래 변경/채택, 송신 claim은 같은 authority 순서를 따른다. transaction 안에는 짧은 검증과 상태 patch만 있고 네트워크, 모델, 파일 read, join은 없다. DB 처리가 느려도 capture/audio 스레드는 기다리지 않는다. 아직 미해소 입력이 있다는 로컬 admission fence를 먼저 닫고, 입력 원장과 hold가 확정되고 관계를 해소한 뒤에만 연다. wake 유실만으로 hold를 해제하지 않는다.

```rust
async fn apply_change(change: AdoptedChange, store: &Store) -> Result<()> {
    let effects = store.transaction(|tx| {
        tx.require_input_and_base_version(&change)?;
        let old = tx.current_request(change.request)?;
        let new = old.next_version(change.normalized_request);

        tx.replace_current(&new);
        tx.revoke_runs_for(old.version);
        tx.withdraw_pending_fulfilment_commands(old.version);
        tx.invalidate_unpresented_responses(old.version);
        tx.record_required_external_reconciliation(&new)?;
        tx.mark_current_execution_needed(new.version);

        Ok(PostCommitEffects {
            cancel_runs: tx.runs_for(old.version),
            wake_request: new.version,
        })
    }).await?;
    supervisor.signal_cancel(effects.cancel_runs);
    supervisor.wake(effects.wake_request);
    Ok(())
}

async fn accept_run_exit(run: RunKey, exit: RunExit, store: &Store) {
    store.transaction(|tx| {
        tx.record_run_terminal(run, &exit);
        match tx.run_eligibility(run) {
            Eligibility::Obsolete => {
                tx.record_discard(run);
                return Ok(());
            }
            Eligibility::Held => {
                tx.defer_bounded_exit_for_recheck(run, exit)?;
                return Ok(());
            }
            Eligibility::Current => {}
        }
        match exit {
            RunExit::Proposal(p) => {
                tx.validate_proposal_dependencies(&p)?;
                tx.adopt_proposal(run, &p)?;
                tx.create_pending_effects(run, p)?;
            }
            RunExit::Failed(e) => tx.fail_current_attempt(run, e)?,
            RunExit::Superseded => {}
        }
        Ok(())
    }).await
}
```

`require_input_and_base_version`는 입력 ID/revision, 해석이 본 기준 요청, 이미 처리한 변경 ID를 검사한다. 중복 입력은 동일 채택을 재사용하고, 기준이 오래된 변경 제안은 조용히 최신 값에 적용하지 않고 재해석/재결합한다. 여러 후속 입력은 원문과 순서를 보존한다. 실행 예약을 최신 하나로 합칠 때도 누락된 변경 의미를 건너뛰지 않는다.

`run_eligibility`는 revision뿐 아니라 attempt, incarnation, 미해소 hold와 의존 상태를 검사한다. hold 때문에 아직 판단할 수 없는 유효 제안은 bounded 보류 상태로 남겨 해소 후 재검사한다. 보류 결과는 실행당 하나이며 byte 예산 초과/저장 실패 때 현재 실행 필요 상태와 원인을 남긴다. 이를 영구 stale로 버리고 최신 실행도 시작하지 않는 상태를 만들지 않는다. terminal 기록은 idempotent하고 보류 해소 시 재검사에서도 중복 효과를 만들지 않는다.

결과 검증 뒤 lock을 풀고 현재 상태를 쓰는 구조를 피한다. transaction이 pending 응답을 만들었다고 실제 UI/음성 전달이 완료된 것은 아니다. 각 전달 입구가 내용 판본과 허가를 재검사한다.

정정 commit 직후 wake 또는 취소 통지가 유실되어도 저장된 실행 필요 상태와 현재 판본을 감독자가 다시 조회한다. 프로세스 재시작 시 handle을 복원하려 하지 않고 새 incarnation에서 필요한 로컬 실행만 다시 만든다.

## 5. Agent 송신과 정정

송신 큐 항목은 CommandId다. 전송 자원 대기 중에는 Pending을 유지하고, sender가 아래 transaction을 통과한 뒤 전달 관측을 책임진다.

```rust
async fn send_one(id: CommandId, store: &Store, wire: &AgentWire) {
    let permit = wire.reserve_transport().await?;
    let command = store.transaction(|tx| {
        let c = tx.command(id)?;
        tx.require_pending(&c)?;
        tx.require_no_input_hold(&c)?;
        tx.require_current_authority_or_reconciliation(&c)?;
        tx.require_current_permission_and_capability(&c)?;
        tx.mark_dispatching(id, SendAttemptId::new())?;
        Ok(c.clone())
    }).await?;

    let receipt = wire.send_and_observe(command, permit).await;
    record_receipt_or_unknown(id, receipt, store).await;
}
```

`require_no_input_hold`는 일반 fulfilment 명령에 적용한다. 이미 필요한 이전 실행의 취소/조회는 현재 제어 권한과 대상 일치로 검증하며 새 발화 hold 뒤에 무조건 묶지 않는다. 신규 동작을 허용하는 수정/승인은 현재 입력과 revision을 검사한다.

정정이 먼저 Pending을 철회하면 송신이 없다. Dispatching이 먼저면 이미 외부에 전달되었을 가능성을 추적한다. claim 이후 실제 write 전 중단할 수 있으면 중단하되 확실한 미전송 증명이 없으면 Unknown을 남긴다. DB와 원격 Agent 접수를 하나의 원자 연산으로 간주하지 않는다. 실제 접수, 최초 write와 로컬 claim을 구별해 기록한다.

sender를 요청 worker의 token으로 함께 abort하지 않는다. 응답 유실 시 동일 command key의 status 조회/중복 방지가 확인된 경우만 계약에 맞게 재시도한다. 새 command ID 또는 다른 Agent로 자동 생성 재시도하지 않는다. 재시작 시 Dispatching/Unknown부터 조회한다. 송신 worker 실패와 timeout도 반드시 명령 상태에 남는다.

## 6. 취소, 종료 대기와 자원 반환

```rust
struct Running {
    key: RunKey,
    stop: CancellationToken,
    join: JoinHandle<RunExit>,
}
fn supersede(run: &Running) { run.stop.cancel(); }

async fn reap(mut run: Running) -> CompletedRun {
    // 별도 감독 경로. drop-safe 로컬 async worker만 유예 후 abort 가능.
    let result = wait_then_abort_local_worker(&mut run.join).await;
    CompletedRun { key: run.key, result }
}
```

입력 처리 함수에서 join을 기다리지 않는다. 감독자는 종료한 handle을 수거하며 panic, cancellation과 실패도 terminal 통지로 변환한다. 종료 작업 하나의 무한 대기로 다른 종료 수거가 막히지 않도록 여러 handle을 비동기로 관찰한다. 취소 유예/연산 deadline의 값은 adapter와 장비 검증 후 정한다.

| 대상 | 취소 의미 | 해제 시점 |
| --- | --- | --- |
| 로컬 drop-safe async worker | token 관찰/필요 시 abort | JoinHandle 종료 관측, 소유 지역 객체 destructor 완료 |
| 이미 실행 중인 blocking I/O/FFI | API 취소 요청 또는 소비 철회 | underlying 완료와 callback 수거 후 buffer/handle 해제 |
| 모델 연산 | provider별 취소 요청, 호출 ID 유지 | 실제 자원 반환 확인. 원격 상태 불명은 제한된 unknown 기록 유지 |
| Agent 업무 | 외부 제어 명령 | 접수/적용/종료의 source 확인. 로컬 worker 종료와 무관 |
| 음성 재생 | 로컬 장치 중단/flush | 소비 중 버퍼와 실제 전달 범위 확인 후 해제 |

모델 waiter를 drop해도 Model Access가 호출을 관측한다. 추론 세션과 KV를 다른 실행에 재사용할 때는 해당 런타임의 reset/종료 계약을 확인한다. 모델 가중치는 공유하고 한 요청 정정 때문에 공유 런타임 전체를 죽이지 않는다.

## 7. 큐, 메모리와 과부하

| 저장/큐 | 항목과 제한 | 포화 시 동작 |
| --- | --- | --- |
| 입력 원장/근거 buffer | 원문, 입력 판본, 근거 참조. capture 예산/byte 제한 | gap을 표시하고 미전송 허가 유지 보류. 무한 RAM 저장이나 조용한 입력 삭제 금지 |
| 실행 필요 상태 | Request별 최신 채택 판본 하나 | 중간 실행 예약은 대체하되 입력/변경 이력 보존 |
| 실행/수거 registry | RunKey와 handle. 실행 및 retired 수 제한 | 추가 연산 시작 지연. 입력, local stop, 기존 외부 상태 수신 유지 |
| 모델/읽기 호출 큐 | immutable spec, token, byte 예산, 실제 슬롯 lease | 미시작 stale 항목 제거. 실행 중은 종료 확인 전 슬롯 반환 금지 |
| 완료 통지 | OperationId, terminal 상태, bounded 결과 참조 | 실행 admission 시 final 통지 공간 예약. panic도 해당 경로로 수거 |
| 명령 outbox/wake | durable CommandId. 메모리 wake 제한 | wake 병합 가능, 내구 명령은 조회로 재발견. 무음 삭제 금지 |
| 오디오 큐 | 응답/판본/재생 ID와 PCM 참조. byte와 길이 제한 | 구판 제거, 생산 backpressure. 입력 처리와 장치 stop은 대기하지 않음 |

큰 결과는 완료 채널에 복사하지 않고 결과 저장소의 제한된 참조를 전달한다. 독립 작업들이 한 요청의 버려진 작업 때문에 논리적으로 취소되지 않도록 token은 요청별 root와 자식으로 둔다. 다만 자원 경합으로 생기는 지연은 남는다. 모델 슬롯이 전부 취소 불가능 연산에 잡혀 있으면 새 의미 추론 즉시 시작은 보장할 수 없다. 입력 수신 지속과 구분한다.

읽기 완료 D@17은 불변 Arc/자료 참조로 공유한다. 각 worker의 의미 상태는 별도다. Windows I/O의 OVERLAPPED와 buffer는 어댑터가 완료까지 소유하고 callback에는 가변 Request 포인터 대신 OperationId를 전달한다. 자료 반환은 source version, 범위/누락과 권한을 포함한다. 권한 철회는 참조가 살아 있다는 이유로 무시하지 않는다.

## 8. 늦은 외부 사실과 제어 응답

```rust
struct ControlReceipt {
    command: CommandId,
    execution: AgentExecutionId,
    requested_version: RequestVersion,
    outcome: ControlOutcome,
}
enum ControlOutcome {
    Accepted, Applied, AlreadyCompleted, Unsupported, Rejected, Unknown
}
```

구판 모델 초안은 현재 채택하지 않고 버릴 수 있지만 구판 Agent의 실제 artifact 생성 사실은 버리지 않는다. command/execution/event correlation과 source 순서를 확인하여 해당 실행 원장에 반영한 뒤 현재 요구와 대조한다. 순서나 신뢰 가능한 조회가 없으면 확인 불명으로 표시한다. `Accepted`를 `Applied`로 바꾸지 않는다.

Agent가 제어 순서를 보장하지 않으면 같은 execution의 제어 명령을 순서대로 전달한다. 대기 중 r2 수정은 r3 취소로 철회 가능하나 이미 전송된 수정은 취소와 역순 적용되지 않는지 확인해야 한다. 긴 수정 응답 때문에 취소까지 막히는 경우 제어 선점 capability가 필요한 조건을 명시한다. 미지원 환경에서 즉시 취소를 보장하지 않는다.

수정 불가 Agent의 cancel-then-start는 취소 종료와 기존 결과를 확인한 뒤 새로운 현재 명령을 만들어야 한다. 취소 불명/실패에서는 중복 생성하지 않고 상황을 안내한다. 안전하게 독립 수행할 수 있는 읽기/설명 경로와 충돌하는 외부 생성 재시작은 구별한다.

## 9. 표시와 음성 재생의 마지막 경계

```rust
struct AudioPacket {
    response: ResponseId,
    content_revision: u64,
    playback_attempt: PlaybackAttemptId,
    output_epoch: u64,
    pcm: Arc<PcmBlock>,
}
```

Response Manager는 내용 판본과 전달 의무를, Interaction Manager의 재생 스레드는 output epoch와 장치 제출을 소유한다. 다른 스레드는 Halt/Invalidate/Play를 요청한다. 빠른 입력 중단 신호는 새 블록 제출 전에 검사한다. DB lock, 모델 응답이나 Agent 취소 receipt를 기다리며 장치를 멈추지 않는다.

정정 시 새 음성 허가를 닫고 Invalidate를 전달한다. 렌더러는 이전 epoch packet을 버리며 장치 stop/flush acknowledgement 전에는 새 재생을 허용하지 않는다. 제어 메시지가 지연되는 동안 이미 제출한 블록까지 회수했다고 하지 않는다. 이를 `device_stop_requested`, `device_stop_ack`, 실제/추정 audible 범위로 구분한다. 하나의 atomic load만으로 검사와 장치 제출 전체가 원자적이라고 주장하지 않는다.

UI도 publication ID와 내용 판본으로 upsert/철회한다. 이미 표시된 구판은 과거 이력과 정정 표시를 남기며 사용자가 못 봤다고 간주하지 않는다. PCM 생성 완료는 전달 완료가 아니다. 실제 audible 관측이 없으면 proxy 또는 전달 불명으로 표시한다.

알림 선택/묶음은 DP54에 맡긴다. 현재 출력 중단은 다른 응답의 내용 판본이나 Agent 업무 취소를 자동 변경하지 않는다.
