# QA04 / QA06 원시 원장 독립 검수

생성 UTC: 2026-10-12T00:02:37.989082+00:00

상태: **PASS_WITH_DECLARED_LIMITATIONS**. API 0회, 후보/동결/측정 원시 자료 수정 0개. 모든 호출의 token × 기록된 가격을 Decimal로 다시 계산하고, 순서가 있는 OS 표본을 직접 계산한 뒤 frozen metrics와 대조했다.

| 방안 | 실제 의미 호출 | 5회 총 usage × 가격 USD | P21당 평균 USD | 전체 5회 footprint peak MiB | 전체 5회 RSS peak MiB |
| --- | ---: | ---: | ---: | ---: | ---: |
| A | 423 | 0.660197625 | 0.132039525 | 1014.407692 | 1028.218750 |
| B | 401 | 0.715330500 | 0.143066100 | 648.532280 | 665.359375 |

QA04 값은 provider가 반환한 실제 의미 모델 token을 동결 단가로 계산한 값이며 청구서 확인 금액은 아니다. 음성 API 호출은 없고 음성 비용 추정은 이 주지표에 더하지 않았다. 청구용 billing.jsonl은 model-calls.jsonl의 같은 call_id 기록을 복제하므로 한 번만 셌다. 실패/폐기/취소된 호출을 제외하지 않았다.

| Worker | 호출 | 실제 USD | footprint peak MiB | RSS peak MiB | idle footprint 증가 bytes | 메모리 표본 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| A1 | 83 | 0.125378625 | 689.563576 | 709.609375 | 16384 | 13350 |
| A2 | 81 | 0.126990000 | 647.907280 | 665.281250 | 0 | 14638 |
| A3 | 84 | 0.124161000 | 648.579155 | 665.421875 | 0 | 15072 |
| A4 | 87 | 0.138943875 | 688.422974 | 704.796875 | 0 | 13302 |
| A5 | 88 | 0.144724125 | 1014.407692 | 1028.218750 | 0 | 13066 |
| B1 | 84 | 0.169054500 | 648.454155 | 665.000000 | 0 | 15304 |
| B2 | 81 | 0.139406625 | 628.532303 | 642.703125 | 0 | 17769 |
| B3 | 83 | 0.143351625 | 648.532280 | 665.359375 | 0 | 16412 |
| B4 | 77 | 0.129726750 | 648.204155 | 665.093750 | 0 | 17098 |
| B5 | 76 | 0.133791000 | 626.891678 | 642.890625 | 0 | 18035 |

자동 대조 11260개 중 실패 0개. call 시작/종료/billing의 ID와 내용, Context/output의 canonical SHA, 실제 usage와 cache 항목, summary 합계, 전체 21사례 및 10 worker를 확인했다. 재시도는 실제 attempt 기록으로 집계했다. OS 실패 표본, phase 누락, 순서/중복/process identity를 확인했고 포함된 각 호출은 메모리 관측 구간 안에서 끝났다.

메모리 주지표는 다섯 회 전체에서 같은 관측의 process 합 최대이며 반복 peak의 평균은 보조값이다. 현재 실제 구성은 worker마다 PID 하나여서 합은 그 PID의 kernel 값과 같다. footprint 최대 시점의 RSS와 RSS 자체 최대 시점은 JSON에 따로 저장했다. 10초 idle 구간 첫/마지막의 같은 process 값을 비교하며 감소는 보호조건용 증가값 0으로 둔다. Windows commit은 None이며 Windows 점수/환산을 주장하지 않는다.

준비/평가/추가 변경 원장 스냅샷:

| 범위 | 호출 | snapshot known usage × price USD | pending |
| --- | ---: | ---: | ---: |
| V7_READINESS_PREFLIGHT_NOT_P21 / v7-final-source-preflight-A | 6 | 0.010386750 | 0 |
| V7_READINESS_PREFLIGHT_NOT_P21 / v7-final-source-preflight-B | 3 | 0.006365625 | 0 |
| JUDGE_NOT_VIA_QA04 / evaluation | 112 | 0.333266435 | 4 |
| NEW_FEATURE_SEMANTIC_VALIDATION_NOT_FORMAL_P21 / M04-B-attempt1 | 5 | 0.008040000 | 0 |
| NEW_FEATURE_SEMANTIC_VALIDATION_NOT_FORMAL_P21 / M04-A-attempt1 | 4 | 0.002810250 | 0 |

범위와 검증 한계:

- L01: Actual semantic HTTP usage at frozen standard prices is measured. Account invoices, physical voice model, physical microphone/ASR, and external Agent internals are not measured. Simulated voice estimates must remain separate.
- L02: Mac task_info samples include one isolated worker PID per activity, its HTTP and sampling threads, VIA, local Agent/speech stubs, source buffers, and RAM evidence recorder. They exclude campaign controller, separate post-timing judge/change processes, server RAM and OS desktop. This is a PoC worker scope, not a whole-product process census.
- L03: Observed peaks are maxima of sampled TASK_VM_INFO values, not a kernel-maintained all-time peak. The sampler starts after module imports and stops after idle. Between-sample allocations and earlier import-time peaks cannot be recovered. Actual interval/read durations are preserved for every worker.
- L04: For the current one-PID runtime, simultaneous process sum equals that PID's observation. No per-process peaks from different times/repeats are added. RSS is mapped resident size and phys_footprint is kernel footprint; neither is Windows commit. For future multi-PID/shared mappings, this sum would not establish unique physical RAM or automatically deduplicate shared pages.
- L05: Allocations trace 15 real touched RGBA buffers, the actual serialization copy and 21 materialized canonical original payloads. It is explanatory rather than a complete Python object/alias census. Common buffers/client are shared within each worker; source/owner/context/event snapshots and serialization copies are accounted in OS samples when present. Declared allocation byte counts are never added to OS metrics.
- L06: All attempted HTTP costs remain included regardless of semantic adoption or local cancellation. MEANING_DISCARDED has Request/scene provenance, but not a terminal call_id; request-associated calls are disclosed without claiming a unique per-call semantic discard mapping. Frozen synthetic OUTPUT_DISCARDED events do not imply a newly billed voice/model call.
- L07: Preparation, judge and change-feature validation ledgers are separate snapshots and excluded from formal VIA mean. They can continue appending after this audit. The complete account spending ledger, including invalidated earlier campaigns/preparation and invoices, is not reconstructed by this QA04/QA06 audit.
- L08: The formal terminal ledgers label every actual call purpose=FOREGROUND, including the late-return preparation calls before a scene onset. These formal preparation calls remain in the measured cost and are separately reconstructed from enqueue_ns versus the recorded scene origin. The separate excluded preparation ledgers are readiness preflight, not these formal P21 preparation calls.

증거: qa04-qa06-independent-review.json의 worker별 peak 표본, 역할별 token/비용, 폐기 사건, phase 경계, 배분 기록, 원본 SHA를 참조한다. 모든 개별 대조는 qa04-qa06-independent-checks.json에 보존했다. 이 검수는 QA01~QA03/QA05 수치나 QA 우열을 선주입하지 않는다.
