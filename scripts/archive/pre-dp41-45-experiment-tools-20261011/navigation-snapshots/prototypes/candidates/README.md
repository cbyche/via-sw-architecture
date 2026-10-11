# Current Architecture Candidates

> **Status: no active candidate implementation — awaiting VIA-DP-03 Measurement Freeze**

이 디렉터리는 현재 Core DP의 A/B candidate 구현 위치다. 이전 IR/TASK/AGENT/EXEC 및
VIA-DP-02/05/06/09/11/12/13 reference workspace는 active baseline과 분리해 archive했다.

현재는 Cargo workspace나 executable candidate가 없다. 먼저 VIA-DP-03에서 S2S native
timestamp·revision capability를 확인하고, 동등 기능의 A/B 계약과 QA-09/19/29/39
Measurement Freeze를 승인해야 한다. 그 뒤 해당 DP만 바뀌는 최소 candidate를 만든다.

새 candidate는 다음 원칙을 지킨다.

- S2S Model 1개와 semantic LLM 1개를 모든 대안에 동일하게 사용한다.
- 다른 DP의 authority, fixture, Agent capability와 dependency profile을 고정한다.
- 후보 입력과 evaluator-only oracle을 분리한다.
- unit/smoke PASS를 Architecture measurement나 winner로 부르지 않는다.
- 새 Cargo workspace가 생기면 그 source와 lockfile에 맞는 fmt·clippy·test 명령을 이
  문서와 CI에 함께 추가한다.

이전 candidate source와 미완성 VIA-DP-11 v5 source snapshot은
[pre-Core-ASR prototype archive](../archive/pre-core-asr-reference-20260927/README.md)에
보존했다. Archive code는 현재 candidate가 아니며 그 자리에서 실행해 현재 evidence를
만들지 않는다.
