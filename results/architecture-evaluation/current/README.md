# Current Architecture Evaluation Results

> **Status: empty for the Core-ASR generation — all Core DP results are `NOT_RUN`**

현재 Core ASR은 QA-09·QA-19·QA-29·QA-39이고, Core DP는
VIA-DP-03·05·06·07·15·17이다. 이 계약으로 승인·실행된 A/B campaign은 아직 없다.

2026-09-26~27의 audio qualification과 VIA-DP-02/05/06/09/11/12/13 reference campaign은
[pre-Core-ASR evidence archive](../archive/pre-core-asr-reference-20260927/README.md)로
이동했다. 그 결과는 구현 가능성과 측정기 개발의 provenance일 뿐 현재 QA 값이나
Architecture winner가 아니다.

## 새 결과의 입장 조건

이 디렉터리에는 다음 조건을 모두 충족한 evidence만 추가한다.

1. 해당 DP의 A/B와 Core-ASR applicability가 active 문서에 확정됨
2. machine-readable contract, fixture, oracle, target, 반복, timeout, 집계와 실패 처리가
   결과 전에 동결됨
3. source revision과 모든 input digest를 가진 Measurement Freeze가 존재함
4. 정상 trace와 sentinel qualification이 통과함
5. raw sample, failure/timeout, environment, summary와 replay receipt가 함께 보존됨
6. evidence label이 실제 실행 범위와 일치함

새 campaign은 `dpNN-<campaign>-vN-YYYYMMDD/` 형태의 immutable directory를 사용한다.
동일 sample을 독립 실행처럼 복제하지 않으며 재사용 시 `source_execution_key`를 기록한다.
