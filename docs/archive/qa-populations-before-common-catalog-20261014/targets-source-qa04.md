# QA04 목표의 내부 Rust 변경 근거

> 2026-10-14 / 기존 코드와 Git 이력의 읽기 전용 검토 / 새 후보 구현이나 PoC 아님

[QA04 변경 범위](./qa04-change-scope.md)의 목표는 변경 과제당 평균 3개 운영 Rust 모듈 이하다. 이 문서는 숫자의 참조점인 기존 VIA Rust 수정 두 건을 추적한다. 두 건의 평균은 2개이며 3개 목표를 그 평균에서 계산한 것은 아니다. 협력 모듈을 함께 수정한 선례의 상단을 향후 대표 변경의 평균 상한으로 채택하는 개발 정책 제안이다.

## 1. 원본과 확인 범위

- 원본 로컬 checkout: `/Users/changbae/Workspace/via`
- 확인한 원본 HEAD: `b6032a4c472e18ec216b3e8592346ab269495342`
- 기존 reference 기록: [Internal Rust Voice Agent Reference](../../../../references/internal-rust-voice-agent/README.md)
- reference snapshot repository: `cbyche/via-internal-rust-reference`
- snapshot commit: `37b69d8`
- 코드의 위치와 역할: [Source Index](../../../../references/internal-rust-voice-agent/source-index.md)

HEAD는 reference 문서가 지정한 원본과 같다. snapshot은 최종 코드의 존재와 모듈 선언을 추적하고 원본 Git 이력은 변경 전후 diff를 추적한다. snapshot의 최초 import commit을 아래 원본 commit과 혼동하지 않는다. 원본 commit의 테스트 통과 문구를 이번 검토의 실행 결과로 옮기지 않는다.

## 2. Agent 실행 포트 반영 수정: 3개

**원본 commit:** `06bfa51bc6373d206f46c4f0ab01bcf5d6b05ab6`

**변경 내용:** 실행할 Agent의 포트를 재할당했지만 미리 만든 자식 프로세스 실행 인자에 이전 포트가 남던 문제를 고쳤다. backend 실행 명세에는 포트 placeholder를 넣고 실제 spawn 시점의 환경값으로 해석하도록 했다.

| 파일 | Rust 고유 모듈 | 운영 변경 | 집계 |
| --- | --- | --- | ---: |
| `crates/via-backends/src/openclaw.rs` | `via-backends / lib / crate::openclaw` | OpenClaw 실행 인자의 포트 placeholder 생성 | 1 |
| `crates/via-backends/src/runtime.rs` | `via-backends / lib / crate::runtime` | backend 실행 명세의 포트 placeholder 생성 | 1 |
| `crates/via-process/src/spawn.rs` | `via-process / lib / crate::spawn` | 실제 프로세스 실행 전 인자 placeholder를 환경값으로 해석 | 1 |
| `apps/via/src/commands/gateway.rs` | 테스트 모듈 변경 | `#[cfg(test)]` 아래 assertion 수정만 존재 | 0 |
| `crates/via-backends/tests/contracts.rs` | integration test | 실제 spawn 경로의 계약 확인 추가 | 0 |
| `crates/via-process/tests/path_environment.rs` | integration test | placeholder와 fallback 등 검증 추가 | 0 |
| `docs/deviations/README.md` | 문서 | 동작 차이 기록 정정 | 0 |
| **합계** | **고유 운영 모듈** | | **3** |

`crates/via-backends/src/lib.rs`의 `pub mod openclaw;`와 `pub mod runtime;`, `crates/via-process/src/lib.rs`의 `pub mod spawn;` 선언을 확인했다. 해당 crate root는 이 commit에서 수정되지 않았다. 세 운영 파일 안의 테스트 수정은 별도 운영 모듈로 더하지 않는다.

## 3. 첫 음성 입력에서 엔진 연결: 1개

**원본 commit:** `6d984b635d755ec03e8d00a0b970ebeff61044aa`

**변경 내용:** 클라이언트가 명시적인 unmute 없이 음성을 보내면 초기 chunk를 버리던 문제를 고쳤다. connection이 음성을 보관하고 엔진 연결을 시작하며 연결 후 재전달한다. 실패한 연결을 매 chunk마다 재시도하지 않게 제어하고 sleep 시 대기 음성도 정리한다.

| 파일 | Rust 고유 모듈 | 운영 변경 | 집계 |
| --- | --- | --- | ---: |
| `crates/via-app/src/realtime/connection.rs` | `via-app / lib / crate::realtime::connection` | 음성 보관, 엔진 시작, 재전달, 실패 및 sleep 처리 | 1 |
| `apps/via/tests/wire_payloads.rs` | integration test | 실제 첫 입력 경로를 확인하도록 시험 변경 | 0 |
| **합계** | **고유 운영 모듈** | | **1** |

`crates/via-app/src/lib.rs`의 `pub mod realtime;`와 `crates/via-app/src/realtime/mod.rs`의 `pub mod connection;`을 확인했다. 두 상위 모듈은 해당 commit에서 수정되지 않았다. connection 안의 테스트 모듈은 집계에 더하지 않는다.

## 4. 재현 절차

다음은 원본 checkout에서 수행한 읽기 전용 검토를 재현하는 명령이다. 코드를 실행하거나 수정하지 않는다.

```sh
git rev-parse HEAD
git show --stat --oneline 06bfa51bc6373d206f46c4f0ab01bcf5d6b05ab6
git show --format= --no-ext-diff 06bfa51bc6373d206f46c4f0ab01bcf5d6b05ab6 -- '*.rs'
git show 06bfa51bc6373d206f46c4f0ab01bcf5d6b05ab6:crates/via-backends/src/lib.rs
git show 06bfa51bc6373d206f46c4f0ab01bcf5d6b05ab6:crates/via-process/src/lib.rs
git show --stat --oneline 6d984b635d755ec03e8d00a0b970ebeff61044aa
git show --format= --no-ext-diff 6d984b635d755ec03e8d00a0b970ebeff61044aa -- '*.rs'
git show 6d984b635d755ec03e8d00a0b970ebeff61044aa:crates/via-app/src/lib.rs
git show 6d984b635d755ec03e8d00a0b970ebeff61044aa:crates/via-app/src/realtime/mod.rs
```

운영 함수와 자료형이 실제로 바뀐 hunk를 확인한 후 `mod` 선언으로 고유 경로를 연결한다. tests 디렉터리, `#[cfg(test)]` 모듈, 주석만 바뀐 hunk와 설명 문서는 운영 변경에서 제외한다. 빌드 설정이나 운영 자산이 바뀌면 그 소유 모듈을 추가해야 하지만 두 사례에는 해당 추가가 없었다.

## 5. 이 선례로 주장할 수 있는 범위

**주장:** 기존 VIA Rust의 독립적인 동작 수정이 운영 모듈 1개와 3개로 처리된 실제 사례가 있으므로, 새 구조에서 대표 변경을 평균 3개 이하로 유지한다는 목표에 내부 참조점을 제공한다.

**적용 한계:** 두 수정은 현재 C01~C06을 수행한 결과가 아니다. 일반 변경의 분포나 성공 확률을 추정할 표본으로 사용하지 않는다. 모든 기능 확장이 3개로 가능하다는 증명도 아니다. 현재 후보의 A/B 승패와 무관하게 같은 목표를 적용한다.

여러 기능이 묶인 원본 commit `23d7429b689badcf5090c0d52f780863d6eab0e9`와 초기 전체 구축 commit을 독립 변경 한 건으로 섞지 않았다. 해당 commit은 음성 사건, sleep/wake, provider 등록과 결과 저장 등 다수의 완료 조건을 함께 포함한다. 위 두 수정은 완료 조건이 독립적으로 설명되는 국소 변경이라는 이유로 참조했다. 이 선정 기준 자체가 더 넓은 기능 확장의 평균을 통계적으로 보장하지는 않는다.

코드 이력의 실제 수정 개수는 `REFERENCE_DIFF_REVIEW` 증거다. 향후 현재 A/B의 실제 변경 결과인 `IMPLEMENTED_CHANGE_REVIEW`와 구별한다.
