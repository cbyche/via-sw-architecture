# DP44 음성 실행과 전달

2026-10-22 / 현재 편집 위치 / 기존 설계와 근거 상태 보존

응답별 준비 A와 현재 발화 준비 B의 실행 수명을 비교한다. 공통 QA01은 화면이고 첫 음성은 별도 지표다. 고정 정답 fixture의 정확도 동점을 일반 정확도 우위로 읽지 않는다.

| 파일 | 역할 |
| --- | --- |
| [design.md](design.md) | A/B 기본 구조, 입력, 정정과 재개, 전달 기록의 상세 계약 |
| [presentation-architecture-contract.md](presentation-architecture-contract.md) | 최신 책임과 호출, 7개 그림 및 공통 QA와 첫 음성의 분리 |

[공통 QA와 목표 및 점수](../qa/README.md) / [평가와 시나리오](../evaluation/README.md) / [현재 발표](../presentation/README.md) / [통합 안내](../README.md)

현재 편집은 이 폴더에서 진행한다. 이전 경로는 보존한 출처이며 새 수정본을 병행 작성하지 않는다. 기존 권고는 사용자 채택이나 accepted ADR 변경을 뜻하지 않는다. 출처 경로와 원본 SHA-256은 [설계 이관 원장](../design-migration.json)에 있다.
