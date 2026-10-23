# DP53 구현 흐름에서 도출한 구조 선택

[진입점](README.md) / [공통 구현](02-execution-design.md) / [사건 원장](01-scenarios.md)

## 1. 코드에서 갈림길이 나타나는 지점

먼저 한 요청의 read → interpretation → proposal → admission → dispatch/response 흐름을 구현한다고 가정했다. 정정으로 현재 판본이 바뀌어도 read와 모델 호출은 즉시 끝나지 않는다. 공통 확정 경계와 종료 수거를 마련한 뒤에도 **그 진행 중 연산을 이전 요청 실행의 일부로 끝낼지, 새 요청 실행이 인수할지**가 남는다.

이 문서는 편의상 두 구조를 R(판본별 실행 교체)과 O(진행 중 연산 인수)로 부른다. 최초 컨셉 분류나 기존 DP의 A/B를 이어받은 번호가 아니다. actor와 mutex, 중앙/분산 배치는 이 축과 독립적이다. 둘 다 올바른 현재 판본 검사, 입력 보존, 전송/출력 경계와 외부 사실 추적을 포함한다.

## 2. R: 판본별 실행 교체

RunSupervisor가 요청별 실행을 생성하고 그 worker가 read와 모델 결과를 await한다. r2 채택 시 r1의 채택 권한을 회수하고 token을 취소한 뒤 r2 worker를 만든다. r1 JoinHandle과 underlying 연산은 각각 감독자와 어댑터가 종료까지 수거한다.

```rust
async fn reconcile_current(request: RequestId) {
    let current = authority.execution_needed(request).await?;
    if supervisor.has_authorized_run(current) { return; }

    let registered = authority.register_attempt_if_current(current).await?;
    // 등록과 정정이 교차하면 token이 이미 취소되었거나 등록이 거절된다.
    supervisor.spawn(registered, prepare);
}
```

완료된 불변 문서 자료는 Context Manager를 통해 재사용할 수 있다. R에서 원본 복사를 강제하거나 모든 정정마다 파일 재독출을 강제하지 않는다. 기본 R은 진행 중 연산의 소비 권한을 새 run으로 옮기지 않는다. 이전 read가 아직 완료되지 않았다면 취소/폐기 경로를 밟고 새 run이 다시 요청할 수 있다.

이 구조의 가변 의미 상태는 한 run의 지역 객체다. 결과 채택이 차단된 run은 그 상태를 새 run에 넘기지 않는다. 처리 함수, 오류 반환과 종료 수거가 직접 연결되어 새 단계를 추가할 때 의존성 registry까지 반드시 수정할 필요가 없다.

정정이 모든 모델 입력을 바꾸는 경우 재사용할 연산이 적으므로 단순한 격리가 유리할 수 있다. 반대로 큰 읽기의 완료 직전 형식만 바뀌면 이미 진행한 연산의 잔여 시간을 활용하지 못할 수 있다. 취소 불가능 호출에는 구판과 신판이 동시에 자원을 쓰는 구간이 생긴다.

## 3. O: 진행 중 연산을 유지하고 소비자를 교체

진행 중 연산의 소유권을 요청 worker 밖의 OperationRegistry로 옮긴다. worker는 연산에 소비자로 연결하고 결과 lease를 기다린다. r1에서 r2로 변경할 때 실제 입력이 같은 연산은 r2 소비자를 먼저 붙인 후 r1을 떼어 계속 수행할 수 있다.

```rust
struct Operation {
    id: OperationId,
    key: OperationKey,
    consumers: Set<RunKey>,
    state: OperationState,
    resources: ResourceLease,
}
struct OperationKey {
    kind: OperationKind,
    exact_inputs: InputDigest,
    source_versions: SourceVersionSet,
    implementation: ContractVersion,
    access_scope: AccessScope,
}

fn replace_run(old: RunKey, new: RunKey, needed: Vec<OperationKey>) {
    let actions = registry.with_lock(|ops| {
        for key in needed {
            // 취소가 이미 시작된 항목에는 붙이지 않고 새 연산을 예약한다.
            ops.attach_or_schedule_active_or_complete(key, new);
        }
        ops.detach_consumer(old);
        ops.mark_unneeded_operations_for_cancel()
    });
    // 실제 모델/OS API와 callback은 registry lock 밖에서 실행한다.
    adapter_controls.submit(actions);
}
```

`needed`는 모델이 임의로 제시한 "비슷한 작업" 목록이 아니다. 구현된 처리 단계의 명시적 입력과 의존 계약에서 만든다. 전체 문서 추출은 D@17과 추출기 version/옵션에 의존할 수 있지만, 결론 범위 해석은 요청 의미에도 의존한다. 판본만 바꾸어 구판 결과를 최신으로 표시하지 않는다.

Registry는 연산 실행 사실을 소유하고 RequestAuthority는 결과 사용과 후속 효과를 소유한다. registry에 new 소비자를 붙인 직후 r3로 바뀌더라도 new의 최종 채택 권한은 회수된다. 권한 없는 소비자는 다음 조정에서 제거한다. 이 일시적 자원 낭비를 숨기지 않으며 registry 갱신 자체를 현재 요청 확정으로 취급하지 않는다.

완료 callback은 OperationId를 조회하여 종료 상태와 결과 lease를 기록한다. 대기 중이던 소비자에는 불변 결과 참조가 전달된다. 각 소비자의 RequestVersion/RunKey와 자료 의존 조건을 확인한 뒤 결과를 사용한다. callback이 어떤 소비자가 최신인지 추측해 현재 상태를 직접 수정하지 않는다.

### 인수 가능한 것과 아닌 것

| 단계 | 인수 조건 | 인수하면 안 되는 경우 |
| --- | --- | --- |
| 원본 문서 읽기/추출 | 동일 source version, 실제 read spec와 변환 계약, 현재 접근 허용 | 파일명만 같음, 최신 source 판본 불명, 권한 철회 |
| 결론 범위 추출 | 전체 입력과 출력 계약이 그대로이며 새 요청도 그 범위를 필요로 함 | 전체/결론 변경으로 실제 입력/범위가 달라짐 |
| 모델 해석 | 직렬화 입력, 의존 상태, 모델/계약과 사용 범위가 동일하고 결과 재사용이 허용됨 | "비슷한 정정"이라는 의미 추측, 과거 Task/권한 snapshot 포함 |
| Agent command | 조회/중복 방지는 공통 command 계약으로만 처리 | 새로운 revision으로 재태깅하여 이전 생성 권한 재사용 |
| 음성 PCM | 같은 유효 내용과 구간 대응, 새 재생 허가 | 내용이 바뀐 구간, 생성 완료를 청취 완료로 간주 |

O의 기본 구조에는 진행 중 연산과 소비자 연결 관리가 들어간다. 모든 단계에 semantic cache를 의무화하거나 모든 모델 호출을 인수 가능하게 만들지는 않는다. 공유 범위가 넓어지면 입력 의존 계약, 참조 수명과 접근 철회 비용도 커진다.

## 4. 같은 사건 비교

| 사건 | R | O | 차이의 원인 |
| --- | --- | --- | --- |
| S01 정상 | 직접 await와 단일 run 상태 | 연산 등록/연결 후 await | O의 registry 처리와 메타데이터 |
| S02/S03 해석 중 내용 정정 | 구판 실행 교체, 완료 자료 재사용 | 영향받은 해석 교체, 동일 read 유지 가능 | 진행 중 read의 소유권 |
| S03 읽기 중 장수만 변경 | 완료 자료가 없으면 새 read 요청 가능 | 같은 read에 새 소비자 연결 | 잔여 읽기 시간과 중복 자원 |
| S04/S05 전송 대기/claim 경합 | 공통 명령 경계 | 공통 명령 경계 | 여기서 O만 안전하거나 빠르다고 하지 않음 |
| S06 Agent 접수 후 정정 | 외부 capability에 따라 제어 | 동일 | 로컬 연산 인수로 외부 rollback 불가 |
| S07 늦은 모델 완료 | 구판 결과 수거, 현재 채택 거부 | 유효 소비자 있으면 사용, 없으면 수거/폐기 | 입력 동일성과 소비 관계 |
| S08 준비된 음성 정정 | 동일 출력 판본 검사 | 동일 | 음성 준비 시점은 별도 선택 축 |
| S09 같은 revision 재시도 | attempt로 차단 | 소비 RunKey로 차단 | 공통 필수 조건 |
| S10 반복 정정 | 실행/retired 슬롯 부담, 중복 연산 가능 | 연산 유지 이익, 소비자/무효화 상태 부담 | 실제 살아 있는 연산 수와 메타데이터 |
| S11 재시작 | current 필요 상태에서 실행 재생성 | 동일. process 내 진행 연산은 복원 불가 | O에 내구 연산 checkpoint를 자동 포함하지 않음 |
| S12 권한 철회 | 관련 run/자료 차단 | 모든 관련 소비자와 자료 차단 | 공유 범위의 추가 관리 비용 |
| S13/S14 외부 역순/무관 업무 | 공통 실행 원장과 대상별 제어 | 동일 | 정확한 외부 사실 반영은 양안 의무 |

## 5. QA 관계와 현재 판단

[현재 QA](../qa/README.md), [시나리오 상태](../scenarios/README.md), [기존 평가의 근거 한계](../evaluation/README.md)를 그대로 따른다. 아래는 설계 인과이며 점수, 확정 applicability 또는 차별화 coverage가 아니다.

| QA | 가능한 경로 | 반대 비용과 확인 한계 |
| --- | --- | --- |
| QA01 첫 실질적 음성 | 현재 응답에 필요한 동일 연산을 인수하면 대기 감소 가능 | registry 비용, 재사용 없음, 모델 슬롯 포화. NORMAL/WAIT 공통 사전 분류 유지 |
| QA02 지칭 대상 | 같은 해석 입력과 결과 채택 조건이면 우위를 주장할 근거 없음 | stale 차단을 지칭 정확도로 바꾸지 않음 |
| QA03 전체 필수 처리의 VIA 시간 | O가 재독출/재연산을 줄일 때 경로 단축 가능 | 변경 영향 조정과 결합 비용. Agent 내부 업무 시간 제외 |
| QA04 실제 변경 모듈 | R은 실행 함수 중심, O는 단계 의존/registry 계약까지 영향 가능 | 기능별로 국소화 이익도 가능. 공통 8과제 실제 diff 없이 우열/개수 산출 금지 |
| QA05 모델 비용 | 동일한 모델 호출 인수가 실제 중복 호출을 피할 때 이익 | 파일 read 절감은 이 QA의 모델 비용 절감이 아님. 취소 후 usage와 재시도 포함 |
| QA06 전체 RAM | 중복 buffer/연산 축소 가능 | registry, 소비자 집합, 유지 결과 증가. 불변 자료 공유는 R에도 허용 |
| QA07 업무 순서/연결 | 이번 조건은 대상 업무 연결을 동일하게 고정 | stale 결과/명령 차단을 의미 판단 정확도 점수로 재명명하지 않음 |
| QA08 추가 모델 메모리 | 새 helper나 가중치 복제를 요구하지 않음 | 런타임 세션 자원은 실제 소유/공통 계약에 따라 관측. 추가 모델 감소를 발명하지 않음 |

정확한 연산 재사용이 드문 요청, 취소가 빠른 어댑터, 짧은 read에서는 R의 단순한 실행 격리가 유리할 가능성이 있다. 큰 원본을 읽는 동안 장수/표현 형식만 자주 바뀌고 해당 read를 계속 사용할 수 있으면 O의 인수 이익이 구체적이다. 어느 조건이 VIA 대표 사용에서 지배적인지는 미확인이다.

**현 단계 권고:** R을 한 흐름의 검토 기준으로 삼되 제품 대안으로 선정하지 않는다. O는 진행 중 연산 소유권이 달라지는 실질 대안으로 유지한다. 재사용이 문서 읽기 한 곳에 국한되면 그 부분의 진행 중 요청 병합을 R의 국소 보완으로 검토한다. 여러 단계의 결과 결합과 취소 소유권까지 바뀔 때 O의 독립 구조 분류가 더 타당하다.

## 6. 독립 선택과 반증 조건

| 항목 | 분류 |
| --- | --- |
| token, revision, attempt 검사, 전달 직전 재검사 | 양안 공통 correctness |
| mutex 대신 단일 owner event loop | 동일 원자성/소유권을 구현하는 별도 선택. 이것만으로 R/O 차이 아님 |
| 완료 자료 cache, 불변 자료 참조 | 양안 가능. R을 약화하려고 제거하지 않음 |
| 진행 중 read 병합 한 곳 추가 | 국소 보완일 수 있음. 전체 연산 registry로 자동 확대하지 않음 |
| 재시작 후 연산 checkpoint | 별도 내구화 축. O에만 기본 포함하지 않음 |
| 취소 안 되는 helper를 별도 프로세스로 격리 | 장애/자원 회수 축. 공유 Omni 전체 종료를 요청 취소 수단으로 사용하지 않음 |
| 음성 선행 생산/선택 후 생산 | DP54 등 전달 설계와 조합 가능한 축 |

O의 이익 가설은 동일 입력 연산의 재사용이 대부분 불가능하거나, 반복 정정에도 실제 임계 경로/전체 모델 비용/메모리 이익이 없으면 약해진다. R의 단순성 권고는 종료 수거가 누적되어 입력 외의 진행을 지속적으로 막거나 실제 공통 변경 과제에서 더 넓은 수정이 필요하면 재검토한다. 수치와 모집단을 결과에 맞춰 바꾸지 않는다.

실제 검증 전에는 두 구조의 자원 상한과 adapter capability, 동일 source 버전/정정 사건/외부 상태를 고정해야 한다. 이 문서는 검증 요구를 남기는 것이며 구현 또는 측정 캠페인을 시작하지 않는다.
