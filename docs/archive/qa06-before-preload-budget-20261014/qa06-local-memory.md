# 후보 QA-06 — 최대 로컬 메모리

> 작성일: 2026-10-14 / 사용자 검토안 / Windows 측정 없음
> 후보 순서는 기존 `ASR-QA-06`의 계약 개정을 확정하지 않는다. [중심 문서](../03-03-qa-dp-evaluation-plan.md)와 기존 세대 결과를 구별한다.

## 1. 발표 한 장에 넣을 내용

| 항목 | 내용 |
| --- | --- |
| 이름 | **최대 로컬 메모리** |
| 정의 | **같은 사용자 활동을 처리하는 동안 VIA가 로컬 PC에 동시에 확보한 commit 메모리의 최대량** |
| 지표 | **VIA 전체 동시 최대 commit** — MiB, 낮을수록 좋음 |
| 목표 제안 | **4,096MiB 이하** |
| 근거 | 32GiB급 PC 계획에서 VIA에 4GiB의 commit 예산을 배정하는 작성자 제품 예산 |
| 예시 | 같은 시점의 UI 300 + Core 420 + helper 80 + 고유 공유 100 = **900MiB** |
| 주요 비교 후보 | DP-47과 DP-42. 추가 구조의 작은 차이는 동점일 수 있음 |

**발표 설명:** “VIA의 모든 프로세스가 같은 시점에 확보한 메모리를 합쳐 가장 큰 값을 비교합니다.”

## 2. 정확히 어떤 메모리를 뜻하는가

Windows의 `PROCESS_MEMORY_COUNTERS_EX.PrivateUsage`는 해당 프로세스의 private commit을 바이트로 제공한다. `WorkingSetSize`는 다른 값이며 현재 물리 메모리에 상주한 페이지를 나타내는 working set과 구분해야 한다. [Microsoft PROCESS_MEMORY_COUNTERS_EX](https://learn.microsoft.com/en-us/windows/win32/api/psapi/ns-psapi-process_memory_counters_ex), [Microsoft Memory Performance Information](https://learn.microsoft.com/en-us/windows/win32/memory/memory-performance-information)

이 QA는 **물리 RAM 점유율이나 VRAM을 합친 값이 아니다.** commit은 운영체제가 backing을 제공하기로 확보한 메모리다. 디스크 기반 파일 mapping의 파일 전체 크기를 commit에 더하지 않는다. RAM에 상주하는 파일 mapping과 shared code가 크면 PC 공존성에 영향을 줄 수 있으므로 working set과 GPU 메모리는 별도 진단으로 남긴다.

pagefile-backed 공유 mapping은 여러 프로세스가 접근해도 고유 allocation을 한 번만 센다. Windows의 `CreateFileMapping`은 backing 방식과 `SEC_COMMIT`/`SEC_RESERVE`에 따라 동작이 다르므로 가상 주소 예약 크기를 모두 commit이라고 세지 않는다. [Microsoft CreateFileMapping](https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-createfilemappinga)

## 3. 단일 지표와 관측 범위

```text
M_r(t) = Σ VIA process p의 PrivateUsage_p(t)
         + Σ 고유 VIA 소유 공유 allocation a의 실제 committed_bytes_a(t)

P_r = max_t M_r(t)
M = max(P_1, P_2, P_3, P_4, P_5) / 2^20   [MiB]
```

프로세스마다 다른 시점의 peak를 더하지 않는다. `PeakWorkingSetSize`도 이 산식에 쓰지 않는다. 활동 중 재시작한 프로세스와 종료된 helper의 앞선 사용량도 timeline에 보존한다. 공유 allocator가 private commit에 이미 포함한 영역이면 다시 더하지 않도록 allocation ledger와 API 의미를 먼저 교차 확인한다.

| 포함 | 제외 또는 별도 보고 |
| --- | --- |
| VIA UI, Core, 모든 후보 전용 service/helper의 private commit | 다른 Office/browser 앱과 외부 Agent 프로세스 |
| 화면·음성·전사 원본 buffer, 변환·전달용 실제 copy | 클라우드 모델의 weight와 KV cache |
| Request/Task/질문/응답 상태, queue, 진행 중 job | 가상 주소 예약만 한 미commit 영역 |
| 원문 cache와 파생 후보·관계·revision | 디스크 DB·로그·파일의 저장 크기 자체 |
| VIA 소유의 고유 pagefile-backed 공유 commit | GPU 전용 메모리. 별도 진단으로 표시 |
| 실제 제품 경로의 로컬 VAD·인식 helper가 있다면 해당 메모리 | 이번 후보에서 사용하지 않는 on-device 대형 모델을 가정한 가상 비용 |

동일 원본을 immutable reference로 공유하면 원본 payload는 한 번만 유지할 수 있다. IPC로 실제 copy했다면 각 copy의 commit은 포함한다. B에만 이미지 전체 복사를 강제해 차이를 만들지 않는다.

### 최대값을 관측하는 절차

1. 하나의 monotonic clock으로 전체 VIA process lifecycle과 공유 allocation ID/크기/수명을 기록한다.
2. 모든 프로세스의 private commit을 **10ms 주기**로 가까운 시점에 수집하고 각 수집 시작·끝을 남긴다.
3. 큰 capture·직렬화·공유 allocation의 생성/해제 시에도 snapshot을 추가한다. 예비 실행에서 샘플링을 5ms로 바꿔 peak 누락과 계측 부하를 점검한다.
4. 비원자적 다중 process 표본의 시간차와 누락 가능성을 보고한다. 모든 allocation을 추적하지 못한 표본 최대는 **관측 최대값**이며 수학적으로 완전한 최대를 증명했다고 하지 않는다.
5. trace나 공유 allocation 원장이 누락되면 0으로 가정하지 않고 `INCOMPLETE_MEMORY`로 표시한다.

OOM이나 필수 근거 삭제 및 중요 사건 유실로 메모리가 작아진 실행은 기능 미충족이다. 실패 전 peak도 기록한다. 별도 process로 이동해 사용량을 숨길 수 없으며 단순 process 개수로 메모리를 추정하지 않는다.

## 4. 대표 활동과 동시성

비용 QA의 [COST-20](./qa05-model-cost.md#4-대표-활동-cost-20-v1-제안)과 같은 활동을 사용하되 memory fixture에는 화면 payload와 동시 상태를 명시한다. 활동 초기화부터 원본 준비, 20요청, source 갱신, drain·idle까지 관측한다.

| 고정 조건 제안 | 값과 의미 |
| --- | --- |
| 이력 원본 | 200turn, 완료 Task 20개, 진행 Task 5개. 모든 payload bytes를 fixture에서 고정 |
| 활성 Task 상한 | 10개. 상한만큼 허구의 업무를 만들지 않음 |
| 동시 클라우드 호출 상한 | 4개. 양안 공통이며 무조건 4개 호출 강제 아님 |
| 화면 관측 buffer | 1920×1080 RGBA frame 최대 15개와 같은 pointer/selection event. frame당 약 7.91MiB |
| 입력 음성 | 동일 PCM 표본과 전사. 필요 보존 기간은 양안 동일 |
| 정정 | 4건. 이전 결과 fence와 필요한 원본 보존을 지키며 stale 작업 회수 |
| 반복 | 같은 원본 초기 상태에서 5회. 반복 내부 cache/이력 유지 |

15개 RGBA frame의 payload 합은 `1920 × 1080 × 4 × 15 / 2^20 = 118.65MiB`다. 이는 buffer 내용의 산술 예산이며 실제 heap/copy/압축 구현을 측정한 값이 아니다. 압축과 crop 재사용은 양안에 허용한다. 필요 근거를 B에서만 빨리 지워 작은 값으로 만들지 않는다.

짧은 PoC는 장기 누수 0을 증명하지 않는다. drain 뒤 남은 상태와 증가량은 진단으로 보존하되 주지표는 위 최대 commit 하나다.

## 5. 4,096MiB 목표의 도출

**제품 예산 제안:** 32GiB급 Windows PC에서 다른 업무와 함께 실행하는 VIA에 4GiB를 배정한다. 이전 문서의 장치 계획과 연속성을 유지하되 실측 RAM과 commit을 같은 수치라고 설명하지 않는다.

| 계획상 자원 배분 | GiB | 성격 |
| --- | --- | --- |
| Windows와 기본 상주 앱 | 8 | 계획 가정. Windows 최소 요구량이나 실측값 아님 |
| 사용자 Office/browser/업무 앱 | 16 | 동시 업무 예산 가정 |
| VIA | 4 | 제안하는 VIA commit 목표 |
| 변동 여유 | 4 | 공존 계획 여유 |
| 합계 | 32 | 제조사의 32GB를 정확한 32GiB 실측값으로 변환한 결과 아님 |

이 표는 **PC 메모리를 독점하지 않겠다는 제품 자원 배분 판단**이다. commit 목표를 만족하면 물리 RAM 공존성이 자동 보장된다는 증거가 아니다. 실제 장치의 물리 메모리·pagefile·GPU 공유량과 다른 앱 부하는 추후 공존 시험으로 확인한다.

4GiB의 내부 배분을 다음처럼 검토할 수 있다. 이 값도 모듈별 실측치가 아니다.

| VIA 예산 | MiB |
| --- | --- |
| UI/Core/runtime | 512 |
| capture·전달·변환 buffer | 512 |
| 원문·근거 cache | 256 |
| Task·요청·응답 상태 | 128 |
| DB·영속 읽기 buffer | 128 |
| 클라우드 호출 임시 상태 | 64 |
| queue·직렬화·burst | 128 |
| 작은 로컬 입력 helper | 64 |
| 기본 계획 합 | **1,792** |
| 후보별 추가 상태와 변동 여유 | **2,304** |
| 총 VIA 예산 | **4,096** |

16GiB 제품까지 동일 4GiB를 무조건 배정하지 않는다. 예를 들어 16GiB에서 OS4 + 업무8 + 여유2의 별도 계획을 채택하면 VIA 예산은 2GiB가 된다. 이는 다른 device profile이며 동일 비교 중간에 후보별로 목표를 바꾸는 근거로 쓰지 않는다. NC 팀이 실제 장치 예산을 제공하면 제품 승인값으로 교체한다.

## 6. 0~6점 구간 — MEM-v1 제안

| 점수 | 표시 | 최대 전체 commit M (MiB) |
| --- | --- | --- |
| 6 | ●●● | 0 ≤ M ≤ 1,536 |
| 5 | ●●◐ | 1,536 < M ≤ 2,048 |
| 4 | ●●○ | 2,048 < M ≤ 2,560 |
| 3 | ●◐○ | 2,560 < M ≤ 3,072 |
| 2 | ●○○ | 3,072 < M ≤ 3,584 |
| 1 | ◐○○ | 3,584 < M ≤ 4,096 |
| 0 | ○○○ | M > 4,096 |

목표 4GiB부터 512MiB 감소마다 한 등급을 주는 자원 예산 규칙이다. 512MiB는 목표의 1/8이며 capture/전달 buffer 한 묶음의 계획 크기다. 상위 경계와 모든 중간 숫자가 Windows 공식 권장량은 아니다. 미측정·trace 누락을 0점으로 표시하지 않는다.

**중요한 판정:** 두 구조 모두 1GiB 아래라면 6점 동점이다. 후보 metadata 수 MiB 차이를 부각하려고 사후에 전용 좁은 구간을 만들지 않는다. 원값 차이는 보여줄 수 있지만 사용자에게 중요한 자원 trade-off인지 별도로 판단한다.

## 7. DP별 예상 결론과 한 문장

| DP | 사전 예상 | 발표에 넣을 구조 설명 |
| --- | --- | --- |
| 47 | **A 우세 방향 예상, 실용 차이는 약한 가설** | **“A는 같은 원본 위에 별도의 후보 집합과 revision 저장소를 추가하지 않아 동시에 유지할 상태가 적다.”** |
| 42 | **A 우세 예상** | **“A는 한 Core에서 상태를 연결하므로 서비스별 heap과 전달용 상태 사본의 추가 메모리가 적다.”** |
| 44 | **A 우세 방향 예상, 보조 검토** | “A는 진행 상태를 한곳에 모으므로 단계별 window와 연결 buffer를 따로 유지하는 부담이 적다.” |
| 45 | **A 우세 방향 예상, 보조 검토** | “A는 실제 조회 범위의 cache를 유지하므로 공통 과거 관계를 지속해서 보관하는 추가 상태가 적다.” |

모든 예측은 같은 원본 보존과 기능, async/부분 갱신/cache를 갖춘 A/B의 대표 활동에 한정한다. B가 실제 payload를 참조로 공유하거나 디스크 기반 파생 조회를 사용하면 추가 RAM은 작을 수 있다. A도 cache와 진행 buffer가 필요하다. 따라서 **47의 후보 수가 많다는 도식만으로 128MiB 이상의 차이를 만들 수 있다고 주장하지 않는다.** 45의 파생 기록이 디스크에만 있다면 저장 용량은 이 QA의 차이가 아니다.

차이 인정 제안은 **128MiB 이상이며 계측 오차·활동별 변동보다 큰 경우**다. 128MiB는 목표의 1/32로 정한 계획상 최소 차이다. 5회 각각의 peak와 phase를 공개하고 최대를 만든 실제 allocation을 설명한다. 이 기준보다 작은 차이는 메모리 핵심 커버리지로 세지 않는 것이 타당하다. 현재 47/42의 두 연결은 확정 커버리지가 아니다.

## 8. 이해 예시 — 실제 측정 아님

| 시점 | UI | Core | helper | 고유 공유 | 같은 시점 합 |
| --- | --- | --- | --- | --- | --- |
| t1 | 300 | 420 | 80 | 100 | **900MiB** |
| t2 | 320 | 350 | 80 | 100 | **850MiB** |

이 활동에서 두 시점만 있었다면 최대값은 900MiB다. UI 최대 320과 Core 최대 420을 서로 다른 순간에서 골라 더한 920MiB를 쓰지 않는다. 100MiB 공유 영역을 UI/Core가 함께 읽어도 200MiB로 세지 않는다. 이 예시의 점수는 6점이며 전체 앱이 실제 900MiB를 쓴다는 뜻은 아니다.

## 9. 최소 PoC와 검증 범위

필요한 구조만 실제 Windows process/queue/repository와 payload로 구현한다. 화면 입력은 동일하게 녹화한 frames와 pointer event를 재생할 수 있고 외부 Agent는 고정 사건으로 모사할 수 있다. 클라우드 모델의 응답 내용은 고정 fixture로 모사해도 되지만 원본·요청·응답 payload의 byte 크기와 생존 기간 및 동시성은 보존한다.

차이를 만드는 후보 repository, 서비스 process, serialization/copy, pending 상태를 실제로 유지해야 한다. “B는 메모리 500MiB 추가”와 같은 가상 allocation을 넣지 않는다. 같은 shared payload는 B에서도 공유를 허용한다. 원본이 필요한 기간을 줄여 기능을 포기한 구조는 비교 대상이 아니다.

PoC가 UI/helper 등 공통 경로를 생략했다면 **비교 메커니즘의 메모리 원값과 차이**만 보고한다. 생략한 공통 부분에 임의 상수를 더해 VIA 전체 4GiB 목표를 달성했다고 하지 않는다. 전체 목표 채점은 공통 실행 경로를 함께 계측한 경우에만 가능하다. 제품 전체 구현은 불필요하지만 대표 활동에서 사용하는 프로세스와 buffer는 포함해야 한다.

관측 원장은 process ID/lifecycle, sample timestamp 구간, PrivateUsage, 공유 allocation identity/bytes/lifetime, 활동·요청·revision, queue depth, 원본 참조/copy 수다. Windows API 의미와 중복 제거를 검토하고 알려진 크기의 짧은 allocation으로 표본 누락을 확인하는 계측 검증을 먼저 한다.

## 10. 출처와 남은 확인

- 공식 Microsoft 문서는 commit과 working set 및 공유 mapping의 의미를 확인하는 근거다. 4GiB/128MiB/512MiB는 작성자 예산·등급 제안이다.
- 구조 근거: [DP-47 RAM repository](../04-47-screen-voice-grounding.md#85-구현을-가정한-상세-구조와-실행-계약), [DP-42 구현 단위](../04-42-lifecycle-ownership.md#37-구현-단위로-펼친-ab-구조와-전체-처리-흐름).
- 남은 확인은 NC 목표 PC의 실제 자원 예산과 구현 allocation이다. 현재는 Windows API 검증이나 실제 PoC peak 측정을 수행하지 않았다.
- 47·42 모두 실용 차이가 작거나 점수가 같다면 메모리는 중요한 제품 검증으로 유지하되 핵심 trade-off 축 선정에서는 낮춰야 한다. 숫자 차이만으로 핵심 커버리지를 채우지 않는다.
