# Current Architecture Evaluation Results

> **Status: no current DP41/44/45/42 measurement results — `NOT_RUN`.**

현재 논의하는 QA는 ASR-QA-01~06이다. 정의, 목표, 보호 조건과 점수 경계는
[현재 QA 문서](../../../docs/architecture/12-decisions/decision-packages/03-02-quality-attribute-definitions.md)를 따른다.
현재 후보 실행이나 모델 측정 결과는 없다. 문서 검산과 발표의 가정값을 이 디렉터리의
실측 evidence로 옮기지 않는다.

이전 실험의 결과와 코드 위치는
[전체 보관 안내](../../../scripts/archive/pre-dp41-45-experiment-tools-20261011/README.md)에 정리했다.
기존 archive 결과는 이전 계약의 provenance이며 현재 QA 값이나 A/B 승자의 근거가 아니다.

새 실험이 시작되면 결과 전에 정한 DP/QA 계약, fixture/oracle과 source digest, 실제 실행
환경, 모든 실패/timeout, raw trace와 집계 근거를 새 campaign directory에 함께 보존한다.
mock 실행, 실제 모델 실행과 계산 추정은 구별하고 이전 결과를 덮어쓰지 않는다.

2026-10-11 사용자 지정 보고 방식은
[현재 harness의 보고 요구사항](../../../benchmark/architecture/README.md#poc-결과-보고-요구사항--2026-10-11-사용자-지정)을 따른다.
QA-02는 cap 적용/정답만/오답 포함 실측의 세 평균, QA-03는 cap 적용/완료만의 두
평균을 표본 수와 실패·누락 수와 함께 저장한다. cap으로 raw 시간을 덮어쓰지 않는다.
QA-06은 실제 Mac 환경의 `phys_footprint`와 resident size/RSS를 모두 보존한다.
Mac 관측을 Windows 실측이나 기존 commit 점수로 바꾸어 표시하지 않는다.
이 요구사항은 아직 구현·실행되지 않았으며 결과 상태는 `NOT_RUN`이다.
