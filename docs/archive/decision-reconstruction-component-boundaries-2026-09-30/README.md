# 철회한 Component 경계 중심의 Decision Reconstruction 후보

> **역사 기록 / 현재 후보·요구·설계 근거가 아님**
> 보관일: 2026-09-30 · 원본 기준 commit: `7094db96d5e747cfefe5b6a5132edebf00b515b3`

사용자 리뷰에서 기존 일곱 후보가 Component를 묶거나 나누는 차이에 치우쳤으며, 실제 기능 구현 방식·품질 영향·합리적인 대안의 설명이 부족하다는 지적을 받았다. 이에 기존 추천을 철회하고 target의 주요 기능 구현 방식에서 설계 선택을 다시 복원하기로 했다. 승인된 target Architecture를 폐기하거나 성능 실험으로 후보를 탈락시킨 기록이 아니다.

| 이전 후보 | 철회·후순위 사유 |
| --- | --- |
| 요청 해석/확정 경계 | 동일 호출·검증·상태를 둔 채 Component 소속 차이에 집중 |
| Context 준비 주체 | 조회·자료 확보 방식보다 구성 책임의 위치 차이에 집중 |
| 대화/업무 상태 관리자 | 독립 lifecycle을 양쪽에 유지하면서 실질 동작 차이를 충분히 입증하지 못함 |
| 응답 전달 책임 | 같은 전달 기능을 종류별로 묶는 차이의 중요성이 불충분 |
| Agent 연동 책임 | adapter와 lifecycle 소속 변경을 넘는 대가·효과 설명이 부족 |
| 음성 입력 근거 | dependency 차이는 있으나 통합 대안의 필수 입력 기능이 미확인이고 후보 설득력 부족 |
| process 경계 | 장애 격리 차이는 있으나 과제 핵심 처리의 선택으로 우선할 근거가 부족 |

## 보존한 파일

- [이전 문서 묶음](./decision-packages/README.md): README·선발 원칙·검토 기록·일곱 개별 문서와 diagram README, 일곱 `.drawio`/`.svg` 쌍. 합계 25개 파일을 내용 변경 없이 이동했다.
- [이전 그림 생성기](../../../scripts/archive/decision-reconstruction-component-boundaries-2026-09-30/generate_decision_candidate_diagrams.py): script archive에 원본 그대로 보존했다. 현재 후보 생성·검사 도구가 아니다.
- [manifest.json](./manifest.json): 원래 경로·보관 경로·SHA-256·원본 commit을 기록했다. 이동 뒤 원본과의 byte 일치를 확인한다.

Snapshot 안의 “우선 검토”와 날짜·링크·실행 명령은 당시 상태 그대로다. 외부 상대 링크와 generator의 경로 계산은 원래 배치를 전제하므로 현재 경로에서 실행하지 않는다. 정확한 원래 배치는 위 commit에서 확인한다. 내부 문구를 새 결론에 맞게 고치지 않았다.

현재 목록은 [활성 Decision Reconstruction](../../architecture/12-decisions/decision-packages/README.md)에 있다. 이 archive는 사용자 리뷰와 작업 이력의 provenance만 제공한다.
