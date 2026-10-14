# 변경 모듈 목표와 RAM 최고량 독립 재검토

> 2026-10-14 / 사용자 추가 지적에 따른 문서 검토 / Windows 실행 없음

## 검토 요청과 결론

사용자가 QA04의 1개 및 5개 모듈 가정과 QA06의 1GiB 확보량 목표를 지적했다. 통합 작성자와 별개인 `review_preload_memory` 에이전트가 공식 자료와 목표 논리를 다시 검토했다.

**독립 검토 결론:** 사용자 지적이 타당하다. 이전 검토는 산술과 문서 일치에 치우쳐 목표 자체의 정당성을 충분히 확인하지 못했다. 계산이 맞고 임의 제안이라고 표시했다고 해서 요구된 수준의 목표 근거가 완성되는 것은 아니다.

| 검토 결과 | 반영 |
| --- | --- |
| QA04의 1개와 5개는 실제 수정 경로를 확인한 값이 아님 | 평균 3개 목표와 CHG-v1 점수를 철회. 모듈 수 지표와 변경 과제는 유지 |
| 공식 삼성 및 Microsoft 자료에서 기본 탑재 앱별 공통 RAM 한도는 확인되지 않음 | 공식 제한과 작성자 제품 정책 제안을 구별 |
| 512MiB가 256MiB보다 적합하다는 외부 근거는 없음 | 512MiB를 검증 전 제품 상한 제안으로 한정. RAM 대비 비율을 도출 근거로 사용하지 않음 |
| 삼성에는 8GB 구성도 있음 | 16GB를 전체 최저 사양으로 쓰지 않고 제안하는 기준 장비로 명시 |
| private working set은 공유 RAM을 제외하고 전체 working set 합은 공유를 중복 계산 | 전체 RAM과 전용 RAM을 구별하고 공유 귀속 확인 전 채점 금지 |
| WPR Resident Set은 trace 종료 스냅샷이며 공유 페이지가 Unknown으로 분류될 수 있음 | WPR PID 필터로 연속 최고량을 얻는다고 주장하지 않음 |
| GPU 공유 시스템 RAM과 전용 VRAM은 다름 | 시스템 RAM 버퍼는 귀속과 중복을 확인하고 전용 VRAM은 별도 보고 |

## 독립 검토가 확인한 공식 자료

- [Microsoft 앱별 성능 목표](https://learn.microsoft.com/en-us/windows/apps/develop/performance/planning-measuring-performance)
- [Microsoft OEM 메모리 영향 확인](https://learn.microsoft.com/en-us/windows-hardware/test/assessments/results-for-the-memory-footprint-assessment)
- [삼성의 8GB와 16GB 구성](https://news.samsung.com/us/samsung-expands-galaxy-book6-lineup-making-galaxy-book-more-accessible/)
- [WPR Resident Set의 계측 한계](https://learn.microsoft.com/en-us/windows-hardware/test/wpt/memory-footprint-optimization-exercise-1)
- [GPU 메모리의 공유와 중복](https://devblogs.microsoft.com/directx/gpus-in-the-task-manager/)

## 남은 확인

QA04 목표를 다시 설정하려면 실제 모듈 목록과 수정 전파 경로 및 협업 허용 범위가 필요하다. QA06의 512MiB는 정책 제안이며 실제 제품에서 전체 RAM을 계측하고 다른 업무 앱과 공존하는지 확인해야 한다. 두 QA의 0~6점 구간은 보류했다. 문서의 부족한 근거를 철회한 것이며 목표 설정과 계측이 완료된 것은 아니다.

## 수정본 재검토와 보완

재검토에서 P1은 없고 P2 두 건이 발견됐다.

| 발견 | 보완 |
| --- | --- |
| 중심 문서와 README에 모든 목표 제안을 완성했다는 표현이 남음 | 초안 작성 후 목표와 계측 보완 중으로 수정하고 점수 보류를 공통 규칙에도 표시 |
| QA04의 평균 0.5개 차이 인정 기준도 근거 없이 남음 | 이 기준도 철회하고 실제 수정 경로와 협업 영향 확인 후 다시 정하도록 수정 |

512MiB를 공식 입증값으로 포장하거나 WPR 연속 계측을 과장한 부분은 발견하지 못했다. 수정 후 검토자가 두 P2의 반영과 공통 점수 규칙의 보류 표시를 재확인했다. 이번 문서 정정 범위에서 추가 P1/P2는 발견하지 못했다. QA04 목표와 QA06의 512MiB 타당성 및 전체 RAM 계측과 두 QA의 점수는 여전히 미완료다.
