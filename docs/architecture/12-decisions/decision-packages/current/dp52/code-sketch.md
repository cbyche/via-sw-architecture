# DP52 타입과 실행 코드 스케치

2026-10-23 / Rust 형태의 설계용 의사코드 / 컴파일 및 실행하지 않음

정의가 생략된 ID, 저장 API와 오류 타입은 제안 인터페이스다. Tokio나 SQLite를 현재 제품에 설치 또는 채택했다는 뜻이 아니다. 실제 제품 구현 파일을 만들지 않는다. [설계](design.md)의 두 저장 경계를 같은 입력 및 모델 계약으로 비교한다.

## 1. 입력, 업무, 실행과 전달

```rust
struct InputRecord {
    id: InputId,
    conversation: ConversationId,
    turn_seq: u64,
    revision: u64,
    text: Arc<str>,
    evidence: EvidenceRef,
    exposure_cut: ExposureCut,
    resolution: ResolutionState,
}

enum ResolutionState {
    Unassigned,
    Resolving { attempt: AttemptId },
    WaitingForBinding { earlier_input: InputId },
    WaitingForClarification { interaction: InteractionId },
    Adopted { plan: PlanId },
    Deferred { reason: FailureRef },
}

struct WorkSession {
    task: TaskId,
    revision: u64,                 // 모든 저장 전이의 충돌 검사
    desired_revision: u64,
    link_revision: u64,
    progress_revision: u64,
    dialogue: Vec<DialogueEntryRef>, // 실제 구현에서는 페이지 단위로 읽는 참조 목록
    runs: HashMap<ExecutionId, RunState>,
    questions: HashMap<QuestionKey, QuestionState>,
}

struct ExecutionBinding {
    task: TaskId,
    execution: ExecutionId,         // VIA가 먼저 할당하는 ID
    agent: AgentId,
    account: AgentAccountId,
    remote_session: Option<String>,
    remote_run: Option<String>,
}

struct QuestionKey {
    execution: ExecutionId,
    question: QuestionId,
    revision: u64,
}

struct QuestionState {
    key: QuestionKey,
    status: QuestionStatus,         // Open / AnswerCommitted / Revoked / Expired
    presentations: Vec<PresentationId>,
    answer: Option<ApplyReceiptId>,
}

struct Presentation {
    id: PresentationId,
    question: QuestionKey,
    output_revision: u64,
    text_delivery: TextDelivery,
    audio_delivery: AudioDelivery,  // NotStarted / Partial / Complete / Unknown
    source_seq: u64,
}
```

업무 생성 시 Task Manager가 WorkSession을 만든다. 입력 수신 시 Request Controller가 InputRecord를 만들며, Task가 없어도 입력이 살아 있다. Presentation은 출력 관측 측이 생산하고 질문은 이를 참조한다. Task의 UI 상태가 WAITING_USER여도 다른 입력과 해당 Task 상태 조회는 처리할 수 있다. 질문 대기와 업무 전체 처리 금지 상태를 같은 enum 하나로 합치지 않는다.

## 2. snapshot과 판단 의존성

```rust
struct SnapshotView {
    input: Arc<InputRecord>,
    candidates: Vec<TaskSummary>,
    pending_inputs: Vec<PendingInputRef>,
    presentations: Vec<Presentation>,
}

struct ResolutionTicket {
    input: InputId,
    input_revision: u64,
    attempt: AttemptId,
    read_set: ReadSet, // 행 판본 + 후보 조회 범위의 membership 판본
    exposure_proof: ExposureProof,
}

struct PreparedSnapshot {
    view: Arc<SnapshotView>,
    ticket: ResolutionTicket,
}

enum Target {
    NewTask,
    Existing(TaskId),
    Reply { task: TaskId, question: QuestionKey },
    NoTrackedTask,
    EarlierInput(InputId),
}

struct AssociationProposal {
    operations: Vec<ProposedOperation>,
}

trait AssociationModel {
    async fn associate(
        &self,
        view: Arc<SnapshotView>,
    ) -> Result<AssociationProposal, ModelError>;
}
```

Request Controller가 ticket을 소유하고 모델은 view만 받는다. 입력 ID 및 attempt 대응은 호출 wrapper가 보존한다. 모델이 반환한 임의 ID를 신뢰하지 않고 view의 참조와 존재 여부를 검증한다. 이 인터페이스는 어떤 의미 판단 알고리즘을 쓰는지 고정하지 않는다. try_adopt는 적용 또는 예약뿐 아니라 의존 대기와 질문 생성도 ticket 현재성 검사와 함께 저장한 뒤 반환한다. 오래된 attempt의 실패 및 보류 기록도 현재 입력 상태를 덮어쓰지 않도록 조건부 저장한다.

```rust
async fn resolve_input(id: InputId, svc: Services) -> Result<(), Error> {
    loop {
        let prepared = svc.state.prepare_snapshot(id).await?;
        // prepare가 끝난 뒤 transaction 및 lock은 닫혀 있다.
        let proposal = match svc.model.associate(prepared.view).await {
            Ok(p) => p,
            Err(e) => {
                svc.state.record_attempt_failure(id, prepared.ticket, e).await?;
                return Ok(()); // 재시도는 한도 및 기한을 가진 별도 스케줄 기록
            }
        };

        match svc.state.try_adopt(prepared.ticket, proposal).await? {
            AdoptResult::Applied(_) | AdoptResult::Reserved(_) => return Ok(()),
            AdoptResult::WaitForInput(_) | AdoptResult::Clarify(_) => {
                return Ok(()); // 해당 대기와 질문은 try_adopt에서 이미 영속 기록
            }
            AdoptResult::Stale(retry) if retry.allowed => continue,
            AdoptResult::Stale(retry) => {
                svc.state.defer_if_current(id, retry.attempt, retry.reason).await?;
                return Ok(());
            }
        }
    }
}
```

await의 수명은 입력별 해석 job이다. 공통 이벤트 수신 루프가 이 함수를 직접 await하며 다음 입력을 막지 않는다. supervisor가 유한 in-flight 예산으로 실행하고 반환과 실패를 수거한다. 이전 attempt가 늦게 반환하면 ticket 검사에서 거절한다. 호출 취소를 요청해도 실제 종료가 확인될 때까지 해당 자원의 점유를 추적한다.

## 3. 한 transaction으로 적용하는 구현

선택한 plan은 불변이다. 한 입력에서 나눈 operation의 순서와 ID는 plan 채택 때 고정한다. 모델 재시도마다 반환 배열 index를 새로운 실행 key로 사용하지 않는다.

```rust
fn adopt_and_apply(
    tx: &mut Transaction,
    ticket: ResolutionTicket,
    proposal: AssociationProposal,
) -> Result<ApplyReceipt, ApplyError> {
    // 중복 호출은 현재 질문 검증보다 먼저 이전 적용 결과를 반환한다.
    if let Some(receipt) = tx.applied_receipt(ticket.input)? {
        return Ok(receipt);
    }
    tx.check_input_revision_attempt(&ticket)?;
    tx.check_read_set(&ticket.read_set)?;
    tx.check_exposure_proof(&ticket.exposure_proof)?;

    // 전체 plan의 충돌도 검증하고 새 TaskId와 operation key를 transaction 안에서 할당.
    let plan = validate_and_allocate_plan(tx, ticket, proposal)?;
    for op in &plan.operations {
        task_validate(tx, op)?;
    }
    for op in &plan.operations {
        let effects = task_reduce(tx, op)?; // Task Manager 소유의 전이 규칙
        tx.append_task_dialogue_ref(op)?;
        tx.insert_outbox(effects.commands)?;
    }
    let receipt = tx.insert_apply_receipt(&plan)?;
    tx.adopt_plan_and_mark_applied(&plan, &receipt)?;
    Ok(receipt)
}
// caller가 transaction을 commit한 뒤에만 성공을 반환한다.
// 어떠한 오류에서도 전체 rollback하며 외부 I/O는 수행하지 않는다.
```

권한 검사가 필요한 operation은 task_validate의 현재 권한 조건도 만족해야 한다. 질문 답변은 정확한 Task/Execution/Question/revision을 검사하고 질문 소비와 답변 명령 생성을 함께 저장한다. 서로 다른 두 입력이 같은 질문에 답해도 첫 답변만 열린 질문을 소비한다. 두 번째가 명시적 답변 수정이면 별도 수정 계약으로 처리하며 첫 적용을 삭제하지 않는다.

Task 생성, 대화 연결, 질문 소비, 명령과 입력 적용 상태를 같은 저장 경계에서 보므로 재시작 때 이들 사이를 별도 추측 복구하지 않는다. 여러 업무 operation의 내부 적용은 함께 확정할 수 있으나 외부 실행들의 완료는 각각 별도다.

## 4. 업무 소유자에게 전달하는 구현

```rust
struct ApplyTicket {
    link: LinkId,
    plan: PlanId,
    operation: OperationId,
    input: InputRef,
    target: TaskId,
    payload: ImmutableOperation,
    expected_link_revision: u64,
    exposure_proof: Option<ExposureProofRef>,
}

enum ApplyOutcome {
    Applied(ApplyReceipt),
    Rejected(RejectReceipt), // 미적용을 영속 확인한 결과
}

enum OperationDelivery {
    Reserved { link: LinkId, task: TaskId },
    Applied { receipt: ApplyReceiptId },
    Rejected { receipt: RejectReceiptId },
}
```

Request Controller가 입력 revision, attempt와 directory ReadSet을 검사한 transaction에서 plan과 모든 operation의 대상 예약 및 영속 ticket을 저장한다. 새 Task도 안정적인 TaskId와 초기 저장 레코드 및 wake 등록을 함께 만든다. 예약 후에는 같은 ticket의 대상이나 payload를 바꾸지 않는다. 입력 원문의 이후 수정은 별도 correction 관계로 남긴다.

Task별 async 실행체는 단일 writer지만 crash 후 재생성되면 이전 실행체가 늦게 쓰지 않도록 owner epoch가 필요하다. 아래 저장 함수는 owner epoch, 읽은 상태 revision, inbox 중복 키를 transaction에서 검증한다.

```rust
async fn task_loop(task: TaskId, store: TaskStore, wakes: WakeReceiver)
    -> Result<(), Error>
{
    let lease = store.acquire_owner_epoch(task).await?;
    let mut state = store.load(task, lease.epoch).await?;

    loop {
        // channel은 wake 힌트다. 실제 미처리 항목의 정본은 durable inbox.
        let Some(item) = store.next_pending(task, lease.epoch).await? else {
            await_wake_or_recovery_scan(&wakes).await;
            continue;
        };

        if let Some(outcome) = store.saved_outcome(item.key()).await? {
            store.finish_duplicate_and_redeliver(item.key(), outcome).await?;
            continue;
        }

        // 예상 가능한 stale 질문/잘못된 실행은 loop 종료가 아닌 거절 전이.
        let transition = match validate_and_reduce(&state, &item) {
            Ok(next) => Transition::apply(next, &item),
            Err(RejectReason::StaleQuestion) => Transition::reject(&item),
            Err(reason) => Transition::reject_with(&item, reason),
        };

        // 업무 상태 + routing summary + outcome + inbox 완료 + outbox를 원자 저장.
        let saved = store.commit(lease.epoch, state.revision, transition).await?;
        state = saved.state;
        // 외부 Agent 호출이나 입력측 반영을 여기서 await하지 않는다.
    }
}
```

저장 실패는 receipt 성공으로 바꾸지 않는다. supervisor는 새 owner epoch로 저장 상태와 미처리 inbox를 다시 읽는다. `saved_outcome` 재반환 시 inbox 완료와 receipt 재전달 상태도 일관되게 처리하여 같은 항목을 무한 조회하지 않는다. graceful shutdown 및 idle 해제 분기는 이 의사코드에서 생략했으며 [자원 계약](design.md#6-큐-대기와-자원-해제)을 따른다.

채널을 넣고 `send().await` 성공을 적용 성공으로 취급하지 않는다. 재전달과 wake 유실 뒤에도 durable inbox에서 복구한다. dispatcher는 한 업무 mailbox가 가득 차면 다른 업무를 계속 검사한다. 정기 scan도 유한 batch와 처리 예산을 사용한다.

### receipt와 복합 입력

입력 측 receipt consumer는 LinkId의 유일 제약으로 결과를 한 번 반영한다. 하나의 입력이 두 업무를 지시하면 operation별 상태를 유지한다. T1 Applied, T2 Reserved라면 전체 입력을 Applied라고 표시하지 않는다. T2 Rejected여도 이미 적용된 T1을 재실행하지 않는다.

Reserved timeout은 미적용 증거가 아니다. 대상 owner 조회 및 같은 LinkId 재전달만 허용한다. Rejected receipt를 확인한 operation은 새 plan version과 supersedes 연결로 재판단할 수 있다. 최초 채택된 연결은 이력에서 지우지 않는다. 특히 QA07 관측은 최초 실제 처리 기준으로 사용한 연결을 보며, 이 구조에서 대상 예약 및 전달이 이미 그 사용에 해당하면 receipt 성공까지 관측을 미루지 않는다.

## 5. 저장 및 외부 연동의 필수 조건

아래는 SQL schema 완성본이 아니라 두 구현에 공통으로 필요한 논리적 유일 조건이다.

| 키 | 제한할 중복 |
| --- | --- |
| InputId와 채택 세대 | 같은 입력의 경쟁 attempt가 별도 plan을 동시에 실행하는 것 |
| PlanId, OperationId | 한 plan operation의 여러 적용 |
| LinkId | 업무 소유자 방식의 ticket 재전달과 receipt 중복 |
| ExecutionId, QuestionId, question revision의 답변 slot | 서로 다른 입력이 같은 열린 질문을 이중 소비하는 것 |
| CommandId와 immutable payload digest | 외부 재전송에서 key와 내용이 달라지는 것 |
| provider 계정과 외부 실행 식별자 | 같은 Agent의 여러 실행 혼합 |
| owner epoch와 state revision | 재생성 전 업무 실행체의 늦은 저장 |

Agent Gateway는 outbox에서 명령을 읽고 전송 결과를 저장한다. 전송 직후 crash이면 실제 외부 접수 여부가 불명일 수 있다. Agent가 idempotency key 또는 상태 조회를 지원하면 같은 키로 확인한다. 지원하지 않으면 중복 방지 보장을 축소해 표시하고 무조건 새 실행을 만들지 않는다. outbox만으로 외부 exactly-once 실행을 보장하지 않는다.

## 6. 스케치에서 생략한 구현 binding

- DB driver와 transaction retry 및 busy 처리, migration, 저장 장치 오류 처리.
- 실제 Tokio runtime 배치, owner lease 만료, supervisor 및 공정 dispatcher 코드.
- Agent별 start/resume/reply/status API와 외부 idempotency 보장.
- 출력 측 ExposureCut과 watermark의 실제 플랫폼 관측 정확도.
- 개인정보 보관 및 삭제 시 복구용 식별 기록의 유지 범위.

이 항목들은 제안한 인터페이스의 실제 제품 적용 확인 사항이다. 생략된 구현을 작동한다고 주장하거나 코드 예시의 줄 수를 QA04 결과로 사용하지 않는다.
