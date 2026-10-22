# 현재 발표자료와 제작 안내

> **시간 QA 개정 미반영:** [현재 QA01/QA03 계약](../qa/timing-measurement-contract.md)은 2026-10-23 정의와 측정방법의 사용자 합의가 완료되었다. QA09는 독립 목록에서 제거했다. 이 폴더의 확정 발표와 수치를 새 정의의 결과로 사용하지 않는다.

2026-10-22 자료 정리. 이 패키지의 현재 PPTX는 **36쪽 reference-style v10의 원본 동일 사본**이다. 슬라이드 내용, 수치, 노트와 순서를 수정하지 않았다. 최신 공통 QA v3 전체를 반영한 발표라는 뜻은 아니다. 설계와 공통 평가의 현재 상태는 [패키지 안내](../README.md)를 함께 확인한다.

- [현재 36쪽 PPTX](VIA-DP1-DP5-DP44-reference-style-36slides-v10-guide-reviewed-windows.pptx)
- [제작과 수정 지침](PPTX-GUIDE.md)
- [발표 판본과 의존 자료 목록](current-deck.json)
- [전체 미리보기](preview-index.md)
- [이관 파일과 SHA-256 대조](../presentation-migration.json)
- [이관 검증](verification/migration-validation.json)과 [그림 및 native 표 대조](verification/migration-structure-validation.json)

## 네 DP를 찾는 위치

| 주제 | 발표 쪽 | 발표에 남은 근거 상태 | 현재 설계를 찾는 곳 |
| --- | --- | --- | --- |
| DP2 의미 해석 실행 | 5~8 | 기존 제한된 PoC와 별도 재생 결과 및 조건부 B 권고를 보존 | [DP2 설계](../dp02/README.md), [v10 설계 표현 보정](../design-review/README.md#4-dp2-논리-실행과-공통-모델-호출을-분리) |
| DP3 화면과 음성 지칭 | 9~15 | CPU 구성의 합성 평가 예시를 보존 | [DP3 설계](../dp03/README.md) |
| DP5 위임용 메모리 | 20~23 | 기본안 미선택과 보완 미평가라는 당시 발표 상태를 보존 | [DP5 설계](../dp05/README.md), [v10 설계 표현 보정](../design-review/README.md#6-dp5-조회-원본-보완과-관계-게시를-구별) |
| DP44 음성 실행과 전달 | 24~30 | 합성 비교와 조건부 권고를 보존. 정확도 100%는 고정 정답 제어 조건 | [DP44 설계](../dp44/README.md) |

DP1 1~4쪽, DP4 16~19쪽과 이전 DP3 부록 31~36쪽도 그대로 남겼다. 네 DP 정리라는 이유로 기존 발표 내용을 삭제하지 않았다. 발표 번호 DP5는 위임용 메모리이며 과거 분석 파일의 `05` Agent 인계와 구별한다.

## 최신 설계 및 평가와 맞출 때

이관 전 v10은 구조 그림 11쪽을 고치고 기존 QA 표를 보존한 판본이다. 공통 QA v3와의 전체 대조는 다음 발표 편집에서 필요한 작업으로 남긴다. 현재 설계의 변경 여부, QA 정의와 점수 판본, 시나리오의 모집단, 실제 관측과 합성 가정을 각각 대조한 뒤 반영한다. 새로운 공통 평가값을 과거 PoC 표에 이름만 맞춰 대입하지 않는다.

기존 발표의 화면 QA01과 새 음성 QA01을 구별하고, QA07은 요청 의미 해석 정확도로 설명한다. 서로 다른 층과 모집단의 수치를 합산하거나 같은 전후 효과로 연결하지 않는다. 발표의 최신성은 파일 번호 대신 [판본 목록](current-deck.json)의 반영 상태로 확인한다.

## 편집 원본과 검증 기록

현재 그림의 편집 정본은 [설계 표현 보정 문서](../design-review/README.md)와 각 DP의 설계 폴더에 있다. 발표 폴더에는 같은 그림을 또 편집용 정본으로 복제하지 않는다. SVG, draw.io, PNG와 해당 scene을 본문과 함께 수정하고 발표에 반영한다.

[sources](sources/README.md)는 v9에서 v10을 만들었던 생성기와 패키저, 검증기의 고정 스냅샷 및 재현 준비 도구를 제공한다. 기존 생성 경로는 당시 11쪽의 한정된 변환이다. 차기 발표 편집은 현재 v10 사본에서 새 판본으로 시작한다.

[당시 v10 검토 기록](verification/whole-deck-v10-review.txt)과 JSON은 역사적 경로와 시각을 그대로 보존했다. TXT 안의 상대 링크는 원문 위치를 기준으로 해석하며 현재 탐색에는 이 README의 링크를 사용한다. [Windows Open XML 결과](verification/whole-deck-v10-windows-openxml.json)와 [환경 기록](verification/whole-deck-v10-windows-environment.json)은 같은 SHA의 원본 파일에 대한 기존 검사다. PowerPoint 앱의 실행과 native 렌더링을 확인한 기록은 아니다.

이번 이관은 원본과 PPTX 전체 바이트 및 36개 PNG의 일치를 검사하고 Windows 서체 정책 검사를 다시 수행했다. 슬라이드 생성이나 렌더를 새로 하지 않았다. 동일한 파일을 복사한 작업이므로 기존 렌더를 함께 이관했으며, 파일을 편집하는 다음 작업에서는 변경 페이지를 다시 렌더한다.
