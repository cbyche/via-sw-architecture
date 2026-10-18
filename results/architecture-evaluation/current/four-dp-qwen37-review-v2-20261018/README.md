# 수정 검토 PoC evidence

보고 생성 시각: 2026-10-19T00:30:47.309739+09:00 (KST). 캠페인 이름의20261018은시작식별자로유지했다.

## 발표용 비교와 전체 실행

[수치 정본](final-measurement-summary.md) / [JSON](final-measurement-summary.json)이 최종 집계다. [effective120 선택](effective-main-selection.json)은 원래 BASIC의 DP01/02/04 84회와 최종 DP05 공통 원문 재사용·공급 범위 합집합 교정36회를 선택한다. 실패 trial을 개별 성공 trial로 치환하지 않았다. 예정 QA01/QA03 분모는261/243이고, callback proxy 관측은 255/261, host 완료 관측은 233/243이다. 평균은 관측된 endpoint에 조건부이며 내용·지칭·프로토콜 실패는 별도다.

전체 실제 229회 = 보정 26 + 원래 BASIC 120 + 보완/병행 진단 27 + 경계 중단 교정 11 + 최종 교정 36 + 행 번호 입력 진단 9다. 중간 교정은11/36 완료 경계에서 멈추고25개 job은 시작하지 않았다. [PHASE-STOPPED](basic-dp05-reuse-corrected-v1/PHASE-STOPPED.json)에 이유·완료/미실행 목록과 hash를 보존했다. 진행 중 API 취소나 원시 삭제는 없었다.

[전체 비용·실제 경과](campaign-totals.md) / [JSON](campaign-totals.json): 실제 usage 확인 $2.3201984, 진행 중 예약 $0.0000000, usage 미확인 보수 예약 $0.0000000, 관리 상한$4. 전체 비용에는 원래 실패·calibration·중간11·보완·진단을 모두 포함한다. 발표용 effective120만의 비교 비용은 $1.0864784이며 전체 지출과 다르다. 고정 비교 단가(입력 $0.40/출력 $1.60 per 1M tokens) 환산이며 청구서·현재 공급자 가격·개발 assistant inference 비용은 아니다.

## 읽을 파일

- [오프라인 기록 재생 demo.html](demo.html): 실제 첫 progress/detail, 정정, 실패/복구, model request/response와 입력PNG 내장. 새 모델 호출이나 물리 GUI 시간 측정이 아니다.
- [안별·사례별 CSV](effective-main-comparison/episode-metrics.csv), [case summary](effective-main-comparison/case-summary.csv), [같은 시각 메모리](analysis/memory-option-peaks.md), [최종 QA04 근거](qa04/final-effective-cross-review.md).
- [동일 사례 보완 비교](tactic-comparison.md), [원래36/최종교정36 비교](dp05-correction-comparison.md), [행번호 입력표현9 진단](input-representation-comparison.md).
- 원시 실패: [기존 BASIC 원인](basic-v1/failure-causes.json), [최종05 원인](basic-dp05-reuse-union-corrected-v2/failure-causes.json), [실제 실패 callback CSV](effective-main-comparison/all-failure-callbacks.csv). 각 phase의 raw/runs 및 raw/responses와 runs.jsonl에 실제 응답·인용·시각·usage가 있다. NOT_DISPLAYED나 미완료를 성공 시간으로 대체하지 않았다.
- [최종 파생계산기 pin](analysis/final-analysis-source-manifest.json), [artifact inventory](artifact-inventory.json). calibration-v1의 당시 분석 provenance UNKNOWN과 후속 명시적 CURRENT-REANALYSIS는 구분하여 보존했다.

## 재현 source와 한계

[재현 진입점](../../../../benchmark/architecture/four_dp_poc_v2/FINAL-REPRODUCTION.md)과 [영구 교정 crate](../../../../prototypes/candidates/four_dp_poc_v2_dp05_reuse_corrected/README.md)를 따른다. 원래 BASIC source/binary는 [basic-v1/freeze](basic-v1/freeze.json), 최종05는 [union-corrected/freeze](basic-dp05-reuse-union-corrected-v2/freeze.json), 각 tactic과 행번호 진단은 각 phase의 별도 freeze/plan에 연결된다. 기존 source와 초기 비용·실패를 덮지 않았다.

QA01은 실제 구체적 callback latency proxy이고 첫 의미 정답의 완전한 검증이 아니다. QA03는 동결 host 완료이고 본문 전체 충족과 별도다. DP05는 명시한 receiver-shim API구간 제외 proxy와 full wall을 함께 본다. 행번호9 진단은 선행 전체 기록 검증 때문에 초기 Python부터 약694MB였으므로 최고 약704MB를 기본 QA06나 formatter RAM 증가와 비교하지 않는다. 정확한 allocator 분해는 수행하지 않았다. RAM은 같은 표본의 Python+Rust RSS이며 논리 bytes/실험기록 보관과 물리 RSS를 혼합하지 않는다. 유한한 저작4사례×3반복이며 제품 성능·전체VIA 메모리·일반적 우월성을 입증하지 않는다.
