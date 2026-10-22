# 현재 패키지 이관 검토

2026-10-22. 사용자가 확인한 네 DP 설계, 발표 v10, 공통 QA v3와 QA 목표 및 점수 원전을 같은 작업 경로로 이관했다. 수치, 점수, 대안 선택과 기존 PPTX의 내용은 바꾸지 않았다. 판본과 출처는 [이관 원장](../package-manifest.json)에 기록한다.

## 독립 검토와 수정

설계, QA와 발표를 각각 이관한 세 검토자가 자신이 담당하지 않은 문서와 통합 규칙을 확인했다. 중복을 합친 발견과 조치는 다음과 같다.

| 발견 | 조치 |
| --- | --- |
| 그림 지침이 비교안 고유 Component에도 참조 Architecture 이름을 강제함 | 해당 DP의 현재 계약 이름을 사용하고, 참조 Component를 실제 재사용할 때 참조 이름을 적용하도록 수정 |
| 일부 현재 문서의 QA01이 정답이 나온 시각을 기다리는 표현을 사용함 | 현재 적용 경계는 첫 구체적 채택 내용의 표시이며, 오답의 최초 시각과 오류도 보존함을 명시. 역사적 평가 수치는 유지 |
| 이관 해시 검사가 수정할 설계 그림까지 영구 고정함 | 이관 당시 byte 동일성과 후속 보존 정책을 분리. 현재 설계와 그림은 편집 가능, 기존 발표와 평가 snapshot은 고정 |
| QA의 과거 대응표 앵커가 새 개요로 연결됨 | 과거 패키지 개요의 해당 절로 연결해 출처 보존 |

## 검증 범위

최종 검사는 다음 범위에서 통과했다. 의미상 최신 평가를 발표에 반영했는지는 별도 상태로 남긴다.

| 검사 | 결과와 한계 |
| --- | --- |
| 이관 출처 | 294개 경로를 원본 commit과 대조. 원본 줄바꿈이 CRLF인 Windows 보고서 2개는 Git의 LF 정규화 hash와 원본 bytes hash를 각각 기록 |
| 편집과 보존 정책 | 현재 편집 원본 167개, 고정 snapshot 127개. 작업 중 자동 생성된 Python cache는 이관 대상에서 제외 |
| 현재 패키지 | `check_current_package.py` PASS_WITH_RECORDED_CONTENT_SYNC_GAP. Markdown 링크와 앵커, QA 점수 237개 probe, COMMON-28의 28사례, PPTX 36쪽, SVG/draw.io 35쌍 확인 |
| 저장소 링크 | `check_active_markdown_links.py` PASS. active Markdown 1,041개 검사 |
| 용어 | `check_active_terminology.py` PASS |
| 기존 QA 원전 | `check_qa_candidate_package.py` PASS. 기존 9개 정의와 점수표 및 생성 투영 유지 |
| 변경 형식 | `git diff --check` PASS |

Windows 보고서와 평가 산출물 및 발표 생성 snapshot의 줄바꿈은 `.gitattributes`에서도 보존한다. 원본 전체 hash 대조는 이관 시점의 검증이고, 현재 설계와 QA를 이후 수정하지 못하게 고정하는 조건이 아니다. 별도 스크립트를 추가한 CI는 같은 패키지 검사를 후속 변경에도 수행한다.

발표 v10은 36쪽과 기존 미리보기 및 검증 보고서를 그대로 가져왔다. 새 PPTX 렌더나 Windows 실행은 하지 않았다. 공통 QA v3는 가상 작성 평가이고, 새 모델 호출 또는 성능 측정은 하지 않았다. [남아 있는 내용 동기화](../guidelines/change-and-sync.md#현재-미반영-상태)는 후속 발표 개정 대상이다.
