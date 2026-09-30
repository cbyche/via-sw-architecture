# 이전 후보·VIA-DP 아이디어 검토 기록

## 참고한 것과 이어받지 않은 것

기존 일곱 후보는 사용자 문제를 찾는 참고였으며 Component 분할·통합 제안 자체는 철회된 상태다. 요청 확정·Context 준비·대화/업무·응답 전달·Agent 연동·음성 근거·process 경계라는 문제 축을 재검토했다. 요청 진행과 자료 검색은 다른 실행체·생산체 관점으로 다시 발굴했고, 음성은 독립 recognizer 존폐로 구체화했다. 단순 publisher/adapter/manager 분리는 다시 주력으로 올리지 않았다.

[VIA-DP-01~18 inventory](../README.md#기존-via-dp-0118의-위치)의 직접 처리·상태 일치·음성 근거·게시·Context·의미 확정·복합 요청·복구·Agent 수명·모델 수명·process·영속 확정·자원 예약·Task·상태 관측·모델 이력·표현·권한 문제를 검토했다. 과거 자료의 유용한 질문은 다음과 같다.

| 참고 질문 | 이번 반영 또는 보류 |
| --- | --- |
| 입력이 실제로 어느 시각의 무엇을 근거로 하는가? | 독립 recognizer와 native producer의 전체 자원·실패 비용 비교 |
| 기다리던 진행과 외부 업무 상태는 누가 소유하는가? | workflow continuation과 Task owner를 분리하며 기존 Task 권한 유지 |
| 무엇이 운영 상태의 권위 원본인가? | current row와 event journal의 저장·reader·replay 체계 구체화 |
| Context를 언제, 누구를 위해 준비하며 무엇을 복제하는가? | 의미 검색 생산체 선발; conversation projection과 Fact View는 조건부 보류 |
| publication·Agent ACK·승인·삭제가 엇갈리면 어떻게 되는가? | 모든 대안의 공통 계약으로 유지. 편리한 대안을 위해 제거하지 않음 |
| 별도 process가 어떤 실제 실패를 차단하는가? | 음성 경계는 실제 helper 유무로 비교. 범용 adapter 분리는 근거 부족으로 보류 |

과거 번호와 새 후보의 1:1 매핑, 과거 A/B winner의 승계, 이전 결과의 새 ASR 소급 해석은 하지 않았다. ADR의 accepted/deferred 지위와 caveat도 변경하지 않았다.

## 보관 위치

| 세대 | 위치·보관 방법 |
| --- | --- |
| 처음 일곱 Component 경계 후보 | [2026-09-30 archive](../../../archive/decision-reconstruction-component-boundaries-2026-09-30/README.md), 기존 그대로 유지 |
| 직전 여덟 후보·동일 격자 구조도 | [2026-10-01 archive](../../../archive/decision-reconstruction-uniform-layouts-2026-10-01/README.md), 원본 커밋 `0e2d15f`의 문서·SVG/draw.io·검토 기록·생성기 48파일을 byte 단위 보존하고 manifest에 SHA-256 기록 |
| 당시 생성기 | [script archive](../../../../scripts/archive/decision-reconstruction-uniform-layouts-2026-10-01/README.md) |
| VIA-DP-01~18·기존 ADR·측정 초안 | 기존 active reference 위치 유지, 이동·개정 없음 |

Archive 안의 옛 상대 링크와 과거 “완료” 진술은 당시 기록이다. 현재 설계의 규범 근거가 아니다. 현재 선발 이유는 [discovery](./discovery-and-selection.md), 현재 검토 결과는 [review notes](./review-notes.md)를 따른다.
