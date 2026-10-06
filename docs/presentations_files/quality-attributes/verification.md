# 제작 검증 기록

작성일: 2026-10-06. 검증 대상은 발표 문서와 슬라이드이며 VIA의 구현·품질 측정 결과가 아니다.

| 검사 | 결과 |
| --- | --- |
| 03-00의 V-01~13 이름 및 공동 순위 | 일치, 13개 누락 없음 |
| 설명 한 문장, 목표·측정·책임 경계 | 모든 항목 존재 |
| Markdown·JSON·PPTX 표·speaker notes | 문안·목표·방법 및 상세 경계 일치 |
| PPTX 구조·관계·레이아웃·글꼴 정책 | 오류 0, 경고 0 |
| 편집 가능한 native table | 1페이지 8개, 2페이지 5개 행 확인 |
| Artifact Tool의 최종 PPTX 재가져오기 | 성공 |
| 최종 파일 렌더링 | 두 페이지 각각 확인, 글자 잘림과 주석 겹침 수정 후 재확인 |
| 발표 문서·README·갤러리의 로컬 링크 | 모두 존재 |
| 기존 active 문서의 링크·용어·QA 카탈로그 | 검사 통과 |

일치 검사는 [check_quality_attributes.py](../../../scripts/presentations/check_quality_attributes.py)로 재실행할 수 있다. 생성은 [generate_quality_attributes.mjs](../../../scripts/presentations/generate_quality_attributes.mjs)를 사용했다.

최종 PPTX SHA-256: `fb4db8f2a729721904de853a19e1f7b6cd9e8c4fc9e3a147f2a58f710154508c`.

V의 목표 숫자와 정확한 측정 계약은 미확정이다. 기존 금지 조건의 0건을 전체 정확성·복구율 목표로 확대하지 않았다. PowerPoint/Google Slides 앱의 직접 편집·저장·재열기는 확인하지 않았다. 현재 architecture-ci의 경로 필터에는 이번 발표 파일과 scripts/presentations가 포함되어 있지 않아 이 변경만으로 원격 CI가 실행되지는 않는다.
