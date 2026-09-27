# Current Architecture Evaluation Results

- [Audio loopback qualification v1](audio-loopback-qualification-v1-20260926/README.md): BlackHole 2ch known-waveform output→capture qualification, PASS (`MEASURED_REFERENCE_HARNESS`)

> **Status: 통합 Architecture campaign 재구축 중. 아래 결과는 예비/부분 증거이며 최종 완료 DP는 없음.**

## Current evidence

- [2026-09-27 실행 요약 보고서](./evaluation-executive-report-20260927.md)는 기존 VIA-DP-02/05/06/09/11/12/13 결과와 VIA-DP-01~18 readiness를 정리한 예비 보고서다. `COMPLETE` 표기는 현재 완료 기준으로 superseded되었다.
- [VIA-DP-02 v2](./via-dp-02-evaluation-v2-20260927/report.md), [VIA-DP-05 v1](./via-dp-05-evaluation-v1-20260927/report.md), [VIA-DP-09 v1](./via-dp-09-evaluation-v1-20260927/report.md), [VIA-DP-12 v1](./via-dp-12-evaluation-v1-20260927/report.md), [VIA-DP-13 v1](./via-dp-13-evaluation-v1-20260927/report.md)은 component/targeted 예비 증거다. 통합 A/B 측정값으로 사용하지 않는다.
- [VIA-DP-06 evaluation v4](./via-dp-06-evaluation-v4-20260927/report.md)는 24개 case × A/B/B′의 72 trial을 실행한 `MEASURED_REFERENCE_HARNESS`다. 증거 수준은 `INTEGRATED_REFERENCE_BREADTH_ONLY`다. active QA 19행을 모두 보고하며 유효 endpoint가 없는 행은 `N/A` 또는 `BLOCKED`다. case당 1회라 production p95나 최종 대안 선택 근거로 사용하지 않는다.
- [VIA-DP-11 evaluation v4](./via-dp-11-evaluation-v4-20260927/report.md)는 Process fault·recovery·memory·trace에 한정한 `TARGETED_PROCESS_EVIDENCE`다. QA-32는 A 0개, B 4개의 초과 중단 단위를 관측했고 QA-41은 B가 약 3.7 MiB 작았다. 전체 interaction path의 결론이 아니며 proxy endpoint와 partial predicate는 `BLOCKED`다.
- 실행 evidence는 허용된 evidence label을 실제 source별로 구분한다. Physical Voice product path가 아니므로 `PRODUCT_E2E`가 아니다.
- 이전 VIA-DP-11 pilot 및 complete-v1~v3는 [preliminary archive](../archive/dp11-preliminary-20260926/README.md)로 이동했다. current Architecture 주장에 사용하지 않는다.

이 디렉터리에는 다음 조건을 모두 충족한 새 evidence만 추가한다.

1. metric definition과 DP applicability가 active 문서에 확정됨
2. machine-readable contract, fixture, target, score, repetition, aggregation이 결과 전에 동결됨
3. source revision과 모든 input digest를 가진 Measurement Freeze가 생성됨
4. 비교 실행 승인이 기록됨
5. raw trace validator와 관련 tests가 통과함
6. raw sample, failure/timeout, environment, summary가 같은 freeze directory에 보존됨
7. evidence label이 실제 실행 범위와 일치함

권장 campaign directory는 서로 덮어쓰지 않는 immutable name을 사용한다. 같은 sample을 non-applicable configuration에 복제해 independent run 수를 늘리지 말고 mapping ledger에서 `source_execution_key`로 참조한다.

현재 naming convention은 `dpNN-<campaign>-vN-YYYYMMDD/`다. 각 directory에는 frozen 입력·digest, raw trace, environment/source manifest, 재생성 가능한 `summary.json`, `report.md`를 함께 둔다.

이전 generation 결과는 [historical archive](../../gate2/archive/README.md)에만 있다.
