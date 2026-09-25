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

The current source of truth is [docs/architecture](../architecture/README.md).

## Archive rules

- Historical terminology and conclusions retain their original meaning; do not silently reinterpret them using current definitions.
- Relative links and commands inside a snapshot may reflect its original directory layout and may no longer run from the current tree.
- Use an `archive/*` Git tag when exact original topology or byte-for-byte state is needed.
- If an old idea becomes relevant again, cite its provenance but restate and approve it in the active baseline before implementation.
- Do not patch archived evidence merely to make it look current. Add an archive notice or active cross-reference instead.
