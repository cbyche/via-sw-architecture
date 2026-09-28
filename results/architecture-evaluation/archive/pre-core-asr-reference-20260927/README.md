# Pre-Core-ASR Evaluation Evidence — 2026-09-26~27

> **Historical reference evidence only — not current Core-ASR results.**

이 디렉터리는 QA-09·QA-19·QA-29·QA-39가 확정되기 전
`results/architecture-evaluation/current/`에 있던 qualification과 reference campaign을
`evidence/` 아래 보존한다.

포함된 campaign은 다음과 같다.

- audio-loopback qualification v1
- VIA-DP-02 v2
- VIA-DP-05 v1
- VIA-DP-06 v4, v5, v6
- VIA-DP-09 v1
- VIA-DP-11 v4
- VIA-DP-12 v1
- VIA-DP-13 v1
- 2026-09-27 preliminary executive report

이 결과는 predecessor detailed QA, p95 중심 집계와 당시 fixture를 사용했다. 새 Core
ASR의 모집단·단순평균으로 소급 집계하거나 현재 A/B winner 근거로 사용하지 않는다.
실행하지 못한 DP-11 v5 결과는 이 archive에도 존재하지 않는다.

Git이 추적하던 JSON/JSONL/Markdown과 audio-loopback qualification WAV는 그대로
보존한다. DP-06의 대용량 generated/captured WAV 중 원래 Git ignore 대상이던 파일은
originating checkout에서만 local archive로 남으며 GitHub의 재현 가능한 evidence로
주장하지 않는다. Tracked report와 digest가 해당 payload의 존재를 대신해 현재 결과를
성립시키지도 않는다.

현재 결과 위치는 [current results](../../current/README.md)이며 현재 상태는 `NOT_RUN`이다.
