# DP51 구현 설계와 코드 스케치

[진입점](README.md) / [같은 사건 추적](scenarios.md) / [조사 근거](references.md) / [검토](review.md)

## 1. 설계가 출발하는 실행 흐름

발화가 시작될 때 화면을 처음 찍으면 사전 선택이 이미 사라질 수 있다. 전사에서 “이것”을 찾은 뒤 찍으면 포인팅 당시 화면을 놓칠 수 있다. 따라서 허용된 interaction 수집이 활성인 동안 실제 확보한 화면 및 입력 이력을 유한하게 남기고, 전사 도착 후에는 그 이력을 읽는 흐름에서 시작한다.

이력 보관이 의미 해석 결과를 미리 확정하는 것은 아니다. 수집은 음성 샘플, 픽셀, pointer path, selection 및 창 관측을 남긴다. 의미 모델은 이 자료의 명시적 입출력 계약을 가진 의존성이다. 외부 Agent의 업무 실행은 여기서 시작하지 않는다.

다음 코드는 Rust 형태의 **설계 의사코드**다. API wrapper, allocator, async runtime과 오류 타입의 완전한 구현이 아니며 컴파일하거나 실행하지 않았다. GPU 관련 await는 전용 소유 스레드의 완료 관찰을 추상화한다. 일반 async executor가 COM/D3D 객체를 임의 스레드로 옮겨도 된다는 뜻이 아니다.

## 2. 시간과 식별자

~~~rust
struct TimeRange {
    earliest_ns: u64,
    latest_ns: u64,
}

struct Stamp {
    source_time: Option<TimeRange>,
    received_ns: u64,
    clock_epoch: u64,
}

struct WindowKey {
    process_instance: u64,
    native_handle: usize,
    incarnation: u64,
}

struct FrameMeta {
    id: FrameId,
    capture_epoch: u64,
    capture_seq: u64,
    image_time: TimeRange,
    received_ns: u64,
    copy_completed_ns: u64,
    display: DisplayId,
    geometry: GeometryId,
    window_observation: Option<WindowObservationId>,
    permission_epoch: u64,
}

struct PointerSample {
    stamp: Stamp,
    desktop_position: Point,
    geometry: GeometryId,
    buttons: Buttons,
}

struct SelectionObservation {
    id: SelectionId,
    trigger: EventId,
    window: WindowKey,
    read_interval: TimeRange,
    value: SelectionValue,
}

struct TranscriptRevision {
    utterance: UtteranceId,
    revision: u64,
    audio_epoch: u64,
    words: Vec<WordWithSampleRange>,
    received_ns: u64,
}
~~~

### 2.1 시각 값과 함께 관측 계약을 보존한다

음성과 화면의 시각을 혼동하지 않도록 다음 규칙을 적용한다.

1. WASAPI 패킷의 첫 sample 위치와 QPC 기반 100ns 값을 음성 어댑터가 정규화한다. raw QPC tick과 혼용하지 않는다. 장치 불연속과 timestamp 오류를 별도 기록한다.
2. ASR가 반환한 sample/offset을 실제 원음 구간에 연결한다. 리샘플링, 무음 제거, chunk 경계와 재연결 매핑은 음성 어댑터가 소유한다. 단어 시각 미지원이면 발화 구간 수준으로 불확실성을 넓힌다.
3. Windows Graphics Capture의 source 시각은 compositor 렌더 시각이다. callback 수신, copy 완료 및 실제 광학 출력과 구별한다.
4. 포인터 메시지 시각은 그 clock domain의 매핑과 오차를 남긴다. callback의 현재 시각을 정밀한 발생 시각으로 대체하지 않는다. 32-bit tick의 wrap과 오래 지연된 사건의 매핑 불확실성을 처리해야 한다.
5. UI Automation 조회는 read_interval 안에 관측한 값이다. 선택 이벤트의 시각으로 소급하지 않는다. provider가 원자 snapshot이나 문서 revision을 주지 않으면 그 보장을 만들어 내지 않는다.
6. 모든 source에 공통의 단일 전역 순서가 있다고 가정하지 않는다. producer sequence는 해당 생산자의 순서와 누락을 찾는 용도다.

위 API 사실의 출처는 [조사 근거](references.md)에 있다. 이 설계의 정규화 타입과 오차 정책은 VIA 적용 제안이다.

### 2.2 좌표와 판본

GeometryId는 display 원점, frame 크기, 회전, 픽셀 형식 및 desktop/window/frame 좌표 변환에 연결한다. crop은 parent FrameId와 crop offset/scale을 유지한다. 화면 크기와 좌표 변환이 바뀌면 새 geometry를 만들고 옛 화면의 변환을 바꾸지 않는다.

Capture sequence는 문서 revision이 아니다. 화면 hash가 같아도 창이나 문서의 정체가 같다는 증거가 아니다. native window handle은 재사용될 수 있으므로 process instance와 incarnation을 함께 쓴다.

현재 전경 창 또는 WindowFromPoint 조회 결과를 옛 pointer sample의 확정 창으로 붙이지 않는다. 창 관측은 별도 시각을 가진다. 시각 범위가 창 전환 경계에 걸리면 복수 가능성과 unknown을 남긴다.

현재 설명은 허용된 display를 미리 수집하는 binding을 기준으로 한다. 창별 capture만 활성화한 구조는 더 작은 입력을 얻을 수 있지만 새 창을 구독하기 전의 화면 공백이 생긴다. 이 선택은 아래 보관 구조와 별도 축이며 이번에 묶어서 비교하지 않는다.

## 3. 모듈, 의존 방향과 실행 배치

참조 구조의 Interaction Manager 책임을 구체화하기 위한 내부 모듈 이름이다. 새로운 참조 Component 명칭이나 승인된 배치 변경이 아니다.

| 내부 모듈 | 소유 상태 | 실행 및 호출 |
| --- | --- | --- |
| AudioIngress | 장치 epoch, 원음 block, sample 매핑 | 음성 캡처 스레드가 생성하고 ASR 및 음성 경로에 제한된 참조 전달 |
| EventIngress | pointer/창 사건 sequence, 손실 표시 | OS hook 및 message loop에서 작은 record만 저장 |
| CaptureOwner | capture session, frame pool, D3D context, 복사 중 작업 | 전용 스레드 및 유한한 in-flight 작업 |
| NativeObservationWorker | UIA client와 COM 객체 | 별도 MTA 스레드에서 선택 및 속성 조회 |
| EvidenceHistory | 사건 색인, image metadata, 보관 상태, hold 및 lease 원장 | 한 상태 소유 task가 짧은 명령을 직렬 처리 |
| EvidenceReader | image 접근 및 변경분 복원 작업 | EvidenceHistory가 허가한 사용권과 scratch 예산만 사용 |
| InterpretationClient | 입력/evidence 판본이 붙은 분석 작업 | 근거를 읽은 뒤 의미 모델 호출. 저장소 내부 mutable 상태에 접근하지 않음 |

OS adapter는 사건과 frame을 EvidenceHistory에 제공한다. EvidenceHistory는 의미 모델을 호출하지 않는다. InterpretationClient는 EvidenceHistory의 read port만 사용하며 OS callback이나 UIA 객체를 직접 호출하지 않는다.

이력 관리 task를 하나로 둔 이유는 frame 공개, 의존성 등록, 예산 및 사용권 발급의 경합을 짧은 상태 전이로 처리하기 위해서다. GPU 복사나 이미지 복원을 이 task에서 직렬 실행하라는 뜻은 아니다. 전체 이력 lock을 보유한 채 외부 호출을 await하지 않는다.

UIA native 호출의 강제 종료가 필요하면 NativeObservationWorker를 별도 프로세스로 격리할 수 있다. 프로세스 분리와 이력 저장 형식은 조합 가능하다. worker의 IPC에는 복사한 DTO와 조회 구간을 싣고 COM pointer를 장기 근거로 넘기지 않는다. 프로세스 격리 자체가 의미 정확도 개선은 아니다.

## 4. callback에서 보관 완료까지

### 4.1 작은 사건은 callback에서 복사한다

~~~rust
fn on_pointer(raw: OsPointerEvent, ingress: &Ingress) {
    let event = copy_small_record(raw, ingress.clock.now());
    if let Err(rejected) = ingress.events.try_send(event) {
        ingress.loss.note(rejected.stamp, LossKind::PointerQueueFull);
    }
}
~~~

callback에는 blocking send, 화면 조회, UIA 조회 및 모델 호출을 넣지 않는다. LossLatch는 가득 찬 같은 queue로 Gap을 재전송하는 구조가 아니다. 고정 크기의 별도 손실 상태이며, 상세 구간을 보관할 공간이 없으면 해당 source의 손실 구간을 넓게 합친다. 성공적으로 기록한 producer sequence와 손실 latch를 함께 읽어 공백을 표시한다.

포인터를 마지막 위치 하나로 합치면 원형 경로와 순차 지시를 잃는다. 비손실 batch는 가능하지만 경로 축약은 별도 손실 정책이다. 포인터 경로 수집과 새 픽셀 생성도 분리한다. 화면이 바뀌지 않았다면 여러 pointer sample이 같은 image를 참조할 수 있다.

### 4.2 원음은 OS buffer의 수명 안에서 복사한다

AudioIngress는 GetBuffer로 받은 패킷의 상태와 timestamp를 읽고 사전 할당한 PCM block으로 복사한 뒤 ReleaseBuffer한다. ASR와 음성 모델이 OS buffer를 장시간 보유하지 않게 한다.

SILENT 패킷은 유효한 음성 데이터 pointer가 있다고 가정하지 않고 sample 수에 맞는 무음으로 표현한다. Timestamp 오류를 받은 패킷은 수신 시각만으로 정확한 sample 시각을 채우지 않는다. 공간 부족이나 후속 전달 실패에도 OS 패킷의 반납 경로는 실행한다.

ASR 및 음성 경로의 queue와 read cursor는 분리한다. 한 소비자가 지연된다고 캡처 스레드가 그 소비자를 기다리지 않는다. 소비 중 block은 재사용하지 않고, 소비자별 lag와 전체 block 예산을 제한한다. 부족하면 명시적 오디오 gap을 기록하고 필요한 source 구간을 정상 전사한 것처럼 처리하지 않는다.

원음의 epoch, sample mapping과 발화 식별자는 화면 capture session의 수명과 독립적이다. 마이크 재연결만으로 과거 화면 identity가 바뀌지는 않는다.

### 4.3 화면은 OS frame과 VIA 소유 image를 구분한다

Windows Graphics Capture binding에서는 frame 수명이 OS frame pool과 연결된다. 분석기에 OS frame을 그대로 넘겨 장시간 보유시키지 않는다. 다음 스케치는 queue slot, in-flight 제한 및 예상 bytes를 확보한 뒤 복사하는 구조다.

~~~rust
fn on_frame(frame: OsFrame, ingress: &CaptureIngress) {
    let meta = read_source_metadata(&frame);
    let Some(admission) = ingress.try_admit(&meta) else {
        ingress.loss.note(meta.image_time, LossKind::CaptureBusy);
        return; // frame 반납
    };
    ingress.enqueue_reserved(CopyJob { frame, meta, admission });
}

// 전용 GPU owner의 local 작업 루프에서 수행하는 의사코드
async fn copy_job(job: CopyJob, gpu: &mut GpuOwner, store: &StorePort) {
    let reservation = match store.reserve(job.storage_cost()).await {
        Ok(r) => r,
        Err(reason) => {
            store.report_gap(&job.meta, reason);
            return; // GPU 제출 전이므로 job과 OS frame 반납 가능
        }
    };

    let pending = gpu.submit_copy(&job.frame, reservation);
    // pending은 source, destination과 예산을 실제 완료까지 소유한다.
    let completed = pending.complete_without_early_reuse().await;
    drop(job.frame);

    match completed {
        Ok(image) => store.publish_reserved(job.meta, image).await,
        Err(error) => store.report_gap(&job.meta, error),
    }
}
~~~

이 스케치의 helper 계약은 다음과 같다.

- try_admit은 queue 항목과 OS frame 보유 수를 제한한다. frame pool 크기에 맞게 in-flight를 제한하며 실제 상수는 검증 전 미정이다.
- reserve는 새 allocation 전에 부여한다. copy 중 destination, readback 및 후속 작업 공간을 이미 할당한 뒤 검사하지 않는다.
- GPU immediate context의 명령 제출과 완료 관찰은 owner 스레드가 직렬화한다. 여러 await 작업이 동시에 context를 사용하지 않는다.
- pending 작업은 의미 요청의 취소와 독립적으로 정리 완료까지 유지된다. task future를 버렸다는 이유만으로 GPU가 읽는 메모리의 quota를 반환하지 않는다.
- publish_reserved는 metadata 공개 공간까지 admission된 작업을 전달한다. 연결 종료로 공개할 수 없으면 저장을 취소하고 gap 및 cleanup을 수행한다. 무제한 완료 queue를 추가하지 않는다.
- OS source frame 해제 시점은 backend의 resource/command 완료 계약을 따른다. 이 스케치는 보수적으로 GPU 사용 종료까지 보유한다. 더 이른 반환은 backend에서 보장 여부를 확인한 뒤 적용할 별도 최적화다.

CPU readback과 PNG/기타 인코딩은 모든 capture callback에 강제하지 않는다. 어떤 메모리 domain에 보관하고 소비 시 어떻게 변환할지는 실제 비용에 포함한다. GPU 메모리를 사용했다고 QA06의 시스템 RAM 영향이 자동으로 사라지는 것은 아니다.

Desktop Duplication binding은 callback 대신 전용 owner의 유한 timeout AcquireNextFrame loop가 될 수 있다. 획득/반납 계약이 다르므로 WGC frame wrapper를 그대로 재사용하지 않는다. 두 binding 모두 공통 FrameMeta 및 저장소 계약으로 정규화할 수 있다.

## 5. 선택, 발화 hold와 evidence view

### 5.1 선택 관측

선택 사건을 받은 EventIngress는 bounded UIA 조회를 예약한다. worker는 실제 text/range/rect 값을 복사하고 조회 시작/종료 및 실패를 돌려준다. Unsupported, Empty, Timeout과 ReadFailed는 다르다.

같은 source에 조회가 몰리면 유한한 작업 수와 대기열을 적용한다. 새 조회로 이전 조회를 대체하면 이전 선택 상태를 읽지 못했음을 남긴다. 최신 상태 조회 성공을 모든 중간 선택의 확보로 해석하지 않는다.

발화 전 선택 anchor에는 선택 값과 확보한 당시 frame/crop을 연결할 수 있다. anchor도 별도 byte 및 개수 예산에 포함한다. 선택이 유지되는 동안 최근 raw ring보다 오래 남길 수 있지만 무기한 보장은 아니다. 수집 시작 전에 존재한 선택을 처음 읽었으면 최초 관측 시각만 알 수 있다.

### 5.2 발화 시작

음성 시작 감지는 늦게 전달될 수 있다. 감지 callback의 현재 시각이 아니라 원음 sample 시각으로 직전 범위를 찾는다. pre-roll을 무한히 늘리지 않고 보관 중인 범위와 공백을 반환한다.

~~~rust
enum HistoryCommand {
    BeginUtterance {
        id: UtteranceId,
        audio_start: SamplePosition,
        reply: Reply<Result<UtteranceHold, HoldError>>,
    },
    OpenView {
        request: EvidenceRead,
        reply: Reply<Result<EvidenceLease, EvidenceError>>,
    },
    Release { lease: LeaseId },
    EndUtteranceHold { hold: HoldId },
}

struct EvidenceView {
    version: EvidenceVersion,
    frames: Vec<FrameId>,
    events: Vec<EventId>,
    selections: Vec<SelectionId>,
    coverage: Coverage,
    permission_epoch: u64,
}
~~~

EvidenceHistory가 BeginUtterance 처리 시 기존 자료 선택과 pin을 한 상태 전이로 수행한다. 이 명령 이전에 이미 회수된 자료는 복구되지 않는다. 발화 중 들어오는 새 자료에도 hold 예산을 적용한다. pin된 자료가 가득 차면 capture를 무한 대기시키지 않고 신규 보관 실패를 알린다.

### 5.3 view 발급과 늦은 사건

OpenView는 source 범위와 현재 접근권한을 확인하고 image 및 event 참조를 고정한 뒤 불변 view를 반환한다. frame을 해석해 특정 지칭의 정답으로 골라 주는 API가 아니다. 시간 오차와 공백에 걸친 후보를 유지한다.

내부 queue의 fence는 “VIA가 그때까지 받은 사건을 처리했다”는 뜻이다. OS나 UIA가 더 늦은 과거 사건을 절대 보내지 않는다는 보장은 아니다. source가 그런 보장을 제공하지 않으면 임의 watermark로 COMPLETE를 만들지 않는다. bounded 대기 후 관측된 범위와 pending/unknown을 반환한다.

늦게 도착한 사건이 특정 view에 영향을 주면 새 EvidenceVersion을 발행하고 사용 중인 소비자에게 무효화 통지를 보낸다. 채택된 결과에 대한 재검토는 상위 계약으로 전달하며 수집 계층이 외부 업무를 스스로 재실행하지 않는다.

## 6. 실행을 따라 발견한 보관 구조

### 6.1 각 프레임을 독립 이미지로 보관

~~~rust
struct IndependentFrame {
    meta: FrameMeta,
    pixels: OwnedPixelBlock,
}

async fn read_frame(
    store: &StorePort,
    frame: FrameId,
    consumer: ConsumerId,
) -> Result<ImageLease, EvidenceError>;
~~~

CaptureOwner가 확보한 frame을 VIA 소유 불변 PixelBlock으로 복사한다. EvidenceHistory가 그 소유권과 metadata를 등록하고 ring에는 참조를 넣는다. 분석기는 ImageLease로 읽고 다른 프레임을 참조하지 않는다.

ring에서 참조를 제거해도 소비 중 픽셀을 덮어쓰지 않는다. 저장소 보관 참조와 소비자 사용권이 모두 사라지고 GPU 작업도 끝났을 때 실제 allocation을 해제한다. 참조 계수는 메모리 안전성의 한 수단일 뿐 byte 상한을 대신하지 않는다.

장점은 과거 임의 시점의 읽기가 독립적이라는 것이다. 한 프레임이 누락되어도 다른 frame의 픽셀을 읽을 수 있다. 단점은 화면 대부분이 같아도 새 프레임 전체의 복사와 보관이 필요할 수 있다는 것이다.

화면 내용이 갱신되지 않았음을 source가 제공하면 같은 image ID를 여러 사건에서 참조할 수 있다. 중복 방지를 제거해 이 구조를 불필요하게 약화하지 않는다. 다만 픽셀 비교/hash 자체의 비용과 화면 identity의 차이는 별도로 유지한다.

### 6.2 기준 이미지와 변경분을 보관

~~~rust
enum StoredImage {
    Full {
        meta: FrameMeta,
        pixels: OwnedPixelBlock,
    },
    Delta {
        meta: FrameMeta,
        base: FrameId,
        moves: Vec<MoveRect>,
        patches: Vec<OwnedPatch>,
    },
}

fn append(c: Capture, history: &mut DeltaHistory) -> AppendResult {
    let can_delta =
        c.base_capture_seq == history.last_committed_capture_seq()
        && c.epoch == history.epoch()
        && c.geometry == history.geometry()
        && c.damage_is_complete;

    if !can_delta || history.replay_limit_reached() {
        history.append_full(c.copy_full_with_reserved_budget()?)
    } else {
        history.append_delta(c.copy_patches_with_reserved_budget()?)
    }
}
~~~

DeltaHistory는 OS가 제공한 move/dirty 정보와 변경 영역 픽셀을 소유한다. 매 frame 전체를 독립 보관하는 대신 과거 frame을 읽을 때 필요한 기준과 변경분을 함께 읽는다.

OS metadata의 기준은 마지막 OS 취득 상태일 수 있다. VIA의 마지막 저장 성공 상태와 동일한지 확인하지 않고 연결하면 잘못된 화면을 복원한다. 저장 실패, metadata 불완전, epoch 또는 geometry 변경 후에는 새 full이 필요하다.

source가 누적 변경 metadata를 제공해 최신 화면을 복원할 수 있더라도 누락된 중간 화면을 얻는 것은 아니다. 중간 지칭의 coverage gap은 남는다.

~~~rust
async fn materialize(
    store: &StorePort,
    target: FrameId,
) -> Result<ImageLease, EvidenceError> {
    // 의존성 pin과 작업 공간 예약을 한 상태 전이로 처리한다.
    let plan = store.pin_chain_and_reserve_scratch(target).await?;
    let mut image = copy_checkpoint(&plan.checkpoint).await?;

    for delta in &plan.deltas {
        image = apply_moves_from_previous_then_patches(
            image, delta, &plan.scratch
        ).await?;
    }
    Ok(plan.finish(image))
}
~~~

apply_moves_from_previous_then_patches는 이전 이미지의 불변 source를 기준으로 move를 적용한 뒤 dirty pixels를 덮는다. 겹치는 move를 같은 배열에서 순차 in-place 처리하면 source가 먼저 바뀔 수 있다. 적어도 필요한 원본 보존 또는 두 출력 면의 작업 공간을 예약해야 한다.

읽기 의존성과 회수 조건도 달라진다.

- frame 하나의 pin이 기준 이미지와 앞선 변경분을 유지시킬 수 있다.
- 기준 이미지만 회수하고 그 기준을 쓰는 delta를 남길 수 없다.
- 묶음 전체를 회수하거나 중간 frame을 새 기준으로 만든 뒤 앞부분을 회수한다.
- 새 기준 생성은 복사, scratch와 순간 최고 메모리 비용을 낸다.
- 최대 replay 단계와 byte 수를 제한한다. checkpoint 간격 숫자만 바꾸는 것은 동일 구조의 정책 조정이다.

두 안의 공정한 비교는 같은 source frame, geometry, 픽셀 형식과 보관 대상으로 한다. 동일 Desktop Duplication 입력을 한쪽은 전체 복사하고 다른 쪽은 변경분으로 저장할 수 있다. WGC 전용 dirty 기능의 지원 여부를 확인하지 않은 채 무료 metadata를 가정하지 않는다. metadata가 없으면 직접 차이 계산 비용 또는 전체 frame fallback이 필요하다.

### 6.3 사건 후 캡처만 수행하는 구조의 한계

~~~rust
async fn on_selection_changed(event: SelectionEvent) {
    let frame = capture_next_frame(event.window).await;
    save(event, frame);
}
~~~

이 경로는 정적 앱에서는 적은 이미지로 동작할 수 있다. 그러나 capture_next_frame은 과거 화면을 요청하는 API가 아니다. 선택 사건 전달 및 캡처 지연 중 이미 스크롤했다면 새 화면만 남는다. 클릭 없는 포인팅은 선택 사건도 만들지 않는다.

pointer 이동/체류, 스크롤과 창 전환을 트리거로 추가해도 캡처 전에 사라진 화면의 문제는 남는다. 직전 이력을 보완하면 6.1 또는 6.2의 이력 메커니즘이 다시 필요하다. 따라서 현재 UC 전체의 동등 대안으로 채택하지 않는다. 제한된 앱 지원안 또는 이력 위의 추가 캡처 tactic으로는 검토 가능하다.

### 6.4 구조 차이와 조정값

| 변경 | 분류와 이유 |
| --- | --- |
| 독립 이미지에서 기준/변경분으로 변경 | 읽기 dependency, 저장 권위의 내용, 복원기와 회수 단위가 바뀌는 구조 차이 |
| capture 주기, pointer 체류 시간, checkpoint 간격 | 같은 구조의 정책/상수. 성능 영향은 있지만 독립 대안으로 늘리지 않음 |
| 모델에 줄 이미지 개수/순위 | 의미 입력 정책. 사라진 근거를 복구하는 수집 구조의 대체가 아님 |
| UIA worker 프로세스 격리 | 별도 장애/배치 축. 두 저장 구조와 조합 가능 |
| window capture와 display capture | 별도 수집 범위/구독 축. 비용과 coverage를 함께 검토해야 함 |
| 최근 독립 이미지와 오래된 변경분 병용 | 변환 작업과 두 원장의 수명 규칙이 추가되는 혼합안. 기본안에 무료로 포함하지 않음 |

## 7. 전사 정정과 늦은 결과

~~~rust
async fn handle_transcript(r: TranscriptRevision, app: &App) {
    let job = app.turns.replace_revision(&r).await;
    let evidence = match app.history.open_view(
        r.audio_epoch, sample_ranges(&r.words), job.utterance_hold
    ).await {
        Ok(v) => v,
        Err(gap) => {
            app.turns.record_missing_evidence(job.id, gap).await;
            return;
        }
    };

    let result = app.interpreter
        .interpret(r, &evidence, job.cancel.clone())
        .await;

    app.turns.publish_if_current(
        job.id, job.revision, evidence.version(),
        evidence.permission_epoch(), result
    ).await;
    // 실제 호출이 자료 사용을 끝낸 뒤 lease 반납
}
~~~

replace_revision은 최신 입력 판본 등록과 기존 작업의 게시 무효화를 한 상태 전이로 처리한다. 취소 신호가 늦거나 실패해도 publish_if_current의 입력/evidence/permission 검사를 통과해야 한다.

EvidenceHistory는 화면과 사건을 소유하고, 상위 요청 상태 소유자는 어떤 입력 판본의 결과를 채택할지 결정한다. 수집 자료의 새 판본을 알리는 것은 DP51 경계이며, 외부 업무 수정/취소의 전체 state machine은 이 문서에서 대체하지 않는다.

이미 만료된 자료는 Expired, 확보하지 못한 자료는 Gap/NotCaptured로 반환한다. 현재 화면으로 몰래 대체하지 않는다. ASR 정정 때문에 같은 원본 픽셀을 복제하거나 frame identity를 바꾸지 않는다.

## 8. bounded 자원과 실제 해제

| 자원 | 제한 단위 | 포화 처리 |
| --- | --- | --- |
| 원음 | PCM block 수와 bytes, 소비자 lag | gap, 느린 소비자 admission 제한 |
| 입력 사건 | record 수/bytes, source별 손실 latch | 경로 손실 구간 공개 |
| OS frame | 보유 개수와 진행 중 작업 수 | 신규 frame 반납, gap |
| image 저장 | 실제 allocation bytes | unpinned 회수, 부족 시 신규 보관 거절 |
| selection anchor | 수와 bytes, source 수 | 대체/만료를 명시 |
| 발화 hold | 발화별 및 전체 bytes/개수 | 무제한 확대 거절, coverage 악화 통지 |
| 소비자 lease | 소비자 수 및 동시에 잡는 고유 bytes | 신규 read 거절/유한 대기 |
| delta 복원 | 작업 수, 의존성 및 scratch bytes | 예약 실패 시 시작하지 않음 |
| 분석 | 동시 작업 수, 입력 복사본과 tensor | 취소/유한 대기. 수집과 별도 예산 |

한 allocation은 여러 lease가 있어도 전체 물리 bytes를 한 번 센다. 소비자별 부담과 전체 고유 bytes를 구분한다. GPU/host/shared memory domain도 기록하고 UMA의 공유 allocation을 임의로 두 번 더하거나 RAM에서 누락하지 않는다.

lease 발급 수 자체가 유한해야 하며 Release 통지도 일반 입력 queue 포화로 유실되어서는 안 된다. 한 구현은 lease별 고정 원장 slot에 released 상태를 원자적으로 기록하고 owner를 깨우는 것이다. 깨우기 통지가 합쳐져도 owner가 원장을 순회하여 회수할 수 있다. 다른 구현은 발급 시 반환용 control slot까지 예약하는 것이다. 어느 경우든 Release는 멱등이며 중복 통지로 예산을 두 번 반환하지 않는다. 프로세스가 죽었을 때는 해당 consumer epoch 및 실제 공유 자원 사용 종료를 확인해 원장을 정리한다.

1920×1080 BGRA 한 장의 순수 픽셀은 8,294,400 bytes다. 30장은 248,832,000 bytes다. 이는 해상도와 4bytes/pixel로 계산한 예시이며 실제 RAM 관측값이 아니다. OS pool, GPU/CPU 중복, driver, readback, 복원 및 모델의 메모리는 추가로 확인해야 한다. 자체 allocator의 상한만으로 전체 제품 RAM 상한 준수를 입증하지 않는다.

회수는 다음 상태를 구별한다.

~~~text
Reserved -> Copying -> Ready -> Retired -> Freed
                |                  |
                +-> Failed --------+

Ready/Retired에서 소비 중 lease가 있으면 Freed로 가지 않는다.
Copying에서 취소되어도 GPU 사용 종료 전에는 Freed로 가지 않는다.
~~~

ring 참조 삭제는 Retired 전환일 뿐 Freed가 아니다. 취소 timeout도 자원 사용 종료 확인이 아니다. 격리 worker가 응답하지 않으면 그 worker 종료와 공유 자원 사용 종료를 확인한 뒤 회수한다. in-process에서 종료를 보장할 수 없으면 예산을 계속 점유시키고 신규 작업을 제한한다.

retention hold 만료와 실행 중 reader lease 만료도 구분한다. hold 만료는 이후 조회의 보관 보장을 끝낼 수 있지만, 이미 image를 읽는 작업의 메모리를 즉시 재사용하는 근거가 아니다.

발화가 끝나고 해석/정정 사용이 종료되면 필요한 대상 참조와 최소 근거를 별도 기억 계약으로 인계하고 hold를 해제한다. 장기 대화나 WAIT_USER 동안 raw 전체를 무기한 pin하지 않는다. 이 문서는 참조 구조의 디스크 보관/권한 정책을 자동 변경하거나 상시 화면 녹화를 도입하지 않는다.

## 9. 오류, 재시도와 종료

- **재시도:** UIA나 capture 재시도는 새 관측이다. 실패한 과거 시점의 snapshot을 복구했다고 표시하지 않는다.
- **capture 재생성:** 새 epoch, 초기 full 및 geometry 확인 후 재개한다. 옛 frame/crop의 시각과 변환은 유지한다.
- **source 종료:** 해당 stream에 Closed/Gap을 남기고 새 자료 admission을 막는다. 이미 확보한 허용 근거는 수명 계약에 따라 읽는다.
- **권한 철회:** permission epoch를 올려 신규 read, model 제공과 게시를 차단한다. 실행 중 작업 취소 및 실제 정리를 이어간다. logical 차단과 물리 해제 완료를 구분한다.
- **정상 종료:** 신규 admission 차단, hook/구독 해제, callback 진입 종료 확인, 진행 중 GPU 작업 정리, reader/lease 종료, OS session/pool 및 COM 객체 해제 순으로 닫는다.
- **프로세스 장애:** RAM 이력의 자동 복구를 약속하지 않는다. 별도 기억 계약에 이미 보존된 최소 근거만 재연결할 수 있으며 임시 frame ID를 새 프로세스 자료에 재사용하지 않는다.

## 10. 주변 주제에 전달할 최소 계약

| 상대 책임 | DP51이 제공할 것 | DP51이 소유하지 않는 것 |
| --- | --- | --- |
| 대화 및 요청 입력 | utterance/input revision, sample 범위와 evidence locator | 여러 업무의 의미 연결 |
| 진행 중 실행 정정 | evidence 판본 변경, gap/expiry, 취소 가능한 read | Agent 실행 취소와 외부 변경 복구 |
| 사용자 전달 | 당시 근거, 현재성 한계 및 실패의 구조화 결과 | 응답 순서와 음성 차례 |
| Agent 단계 실행 | 확보한 최소 근거와 source 식별/판본 | Agent 간 계획과 결과 의존 실행 |

타 주제와의 접합 이름과 세부 타입은 공동 검토 전 제안이다. 다른 DP 문서의 state owner나 API를 이 문서에서 확정하지 않는다.
