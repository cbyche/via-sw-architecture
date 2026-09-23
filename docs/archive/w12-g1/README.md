# W12-G1 Archive

이 디렉터리는 W-01~W-03 Voice 재정의 이전의 측정 계약, DP mapping, freeze, readiness와 결과 문서를 보존한다.

`08-reserve-driver-candidates.md`와 `08-reserve-formal-reassessment.md`는 현재 Working-12가 정리되기 전의 QA 후보 선정 기록이다. 현재 QA/measurement source of truth로 사용하지 않는다.

`08-premature-asr-baseline.md`는 상위 품질 관심사 7개를 ASR로 조기 확정했던 이전 active baseline이다. 현재 taxonomy에서는 당시 W-series가 QA catalog로 승계되었고 ASR은 아직 선정하지 않았다.

ID migration은 다음과 같다.

- 과거 `W-01`~`W-12` → 현재 같은 번호의 `QA-01`~`QA-12`
- 과거 parent `QA-01`~`QA-10` → 현재 같은 번호의 `QC-01`~`QC-10`
- 과거 `ASR-01`~`ASR-05` → 당시 QC-01~QC-05에 내린 조기 ASR 판정
- 과거 `ASR-06` → 당시 QC-08에 내린 조기 ASR 판정
- 과거 `ASR-07` → 당시 QC-09에 내린 조기 ASR 판정

- 이 자료는 provenance와 과거 판단 추적에만 사용한다.
- 현재 측정 정의, target, score 또는 DP별 A/B 근거로 사용하지 않는다.
- 현재 QA-01~QA-03 source of truth는 [`../../architecture/08-quality-attributes/voice-responsiveness.md`](../../architecture/08-quality-attributes/voice-responsiveness.md)다.
- 현재 active 문서에서 archive의 수치나 명칭을 복사해 새 결과로 제시하지 않는다.

Git history와 이 archive가 원문을 보존하므로 active 경로에는 redirect를 두지 않는다.
