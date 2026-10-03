# Historical Architecture Archive

> **Not a source of current requirements, measurement contracts, results, or recommendations.**

이 디렉터리는 현재 Architecture baseline에서 퇴역한 세대를 provenance와 재검토를 위해 보존한다.

| Generation | Contents | Git preservation point |
| --- | --- | --- |
| v1.1 | [requirements-v1.1](./requirements-v1.1/requirements-v1.1.md) | `archive/pre-rebaseline-main-20260923` |
| vNext / DP-00 | [vnext-dp00](./vnext-dp00/) | `archive/vnext-dp00-executable-v3`, `archive/vnext-interim-report`, `archive/qa03-v1/*` |
| Rebaseline W12-G1 | [w12-g1](./w12-g1/README.md) | repository history before the W12-G2 definition change |
| QA catalog draft v1 | [qa-catalog-draft-v1](./qa-catalog-draft-v1/README.md) | repository history before the 2026-09-23 QA catalog review |
| DP pre-inventory summaries and candidates | [2026-09-25 provenance](./dp-review-pre-inventory-2026-09-25/PROVENANCE.md) | 전수 VIA-DP-01~18 정리 직전 사본; 사용자 로컬 수정 포함 |
| DP document consolidation | [2026-09-25 consolidation](./dp-document-consolidation-2026-09-25/README.md) | 2eb83ff6의 중간 문서·검토 기록·이전 경로 안내를 통합 전 보존 |
| Core DP selection summary pre-cleanup | [2026-09-28 snapshot](./core-dp-selection-pre-cleanup-2026-09-28/README.md) | Core DP 6개 결론과 이전 DP 중심 설명·상세 지도가 혼재했던 active summary 교체 전 사본 |
| Pre-Core-ASR evaluation cleanup | [2026-09-28 cleanup snapshot](./pre-core-asr-evaluation-cleanup-2026-09-28/README.md) | 이전 runner·candidate·result와 과거 실행 원장을 active 경로에서 분리하기 전 provenance |
| Model premise before shared Omni | [2026-09-29 snapshot](./model-premise-before-shared-omni-2026-09-29/README.md) | `3b5a3081`의 분리 모델 전제와 관련 baseline; 공유 Omni·명시적 입력 ASR 방향으로 변경 전 보존 |
| Target Architecture review history | [2026-09-29 review history](./target-architecture-review-history-2026-09-29/README.md) | `94ef60a3`의 누적 검토 기록·이전 독립 검토를 현재 설계 문서와 분리하여 보존 |
| Withdrawn Component-boundary candidates | [2026-09-30 candidate review](./decision-reconstruction-component-boundaries-2026-09-30/README.md) | `7094db96`의 일곱 후보·그림·선발 원칙을 사용자 리뷰에 따라 추천에서 철회하고 원본·탈락 사유 보존 |
| Superseded uniform-layout decision candidates | [2026-10-01 structural rework](./decision-reconstruction-uniform-layouts-2026-10-01/README.md) | `0e2d15f`의 여덟 후보·그림·검토·생성기 48파일과 SHA-256 manifest; 구조 차이 리뷰 후 보관 |
| Superseded decision selection principles | [2026-10-02 principles](./decision-selection-principles-2026-10-02/README.md) | `9d2e7248`의 원칙 원문과 SHA-256. 현재 구조 계열/전환 기준으로 대체, S-01~06은 활성 위치의 재평가 자료로 유지 |

The current source of truth is [docs/architecture](../architecture/README.md).

## Archive rules

- Historical terminology and conclusions retain their original meaning; do not silently reinterpret them using current definitions.
- Relative links and commands inside a snapshot may reflect its original directory layout and may no longer run from the current tree.
- Use an `archive/*` Git tag when exact original topology or byte-for-byte state is needed.
- If an old idea becomes relevant again, cite its provenance but restate and approve it in the active baseline before implementation.
- Do not patch archived evidence merely to make it look current. Add an archive notice or active cross-reference instead.

- [04-30 이전 여섯 비교 문서](./decision-comparisons-before-0430-2026-10-03/README.md): 04-31~36 재작성 전 자료, 현재 후보 자격의 근거 아님.
