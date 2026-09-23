# Current Architecture Evaluation Results

> **Status: EMPTY — current W-01~W-03 contract is not implemented and no campaign has run.**

이 디렉터리에는 다음 조건을 모두 충족한 새 evidence만 추가한다.

1. metric definition과 DP applicability가 active 문서에 확정됨
2. machine-readable contract, fixture, target, score, repetition, aggregation이 결과 전에 동결됨
3. source revision과 모든 input digest를 가진 Measurement Freeze가 생성됨
4. 비교 실행 승인이 기록됨
5. raw trace validator와 관련 tests가 통과함
6. raw sample, failure/timeout, environment, summary가 같은 freeze directory에 보존됨
7. evidence label이 실제 실행 범위와 일치함

권장 campaign directory는 서로 덮어쓰지 않는 immutable name을 사용한다. 같은 sample을 non-applicable configuration에 복제해 independent run 수를 늘리지 말고 mapping ledger에서 `source_execution_key`로 참조한다.

Current evidence가 생기기 전에는 `NOT_RUN`을 유지한다. 이전 결과는 [historical archive](../../gate2/archive/README.md)에만 있다.
