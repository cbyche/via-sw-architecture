# VIA 통합 발표자료와 부분 편집

현재 발표와 수정의 기준은 [VIA-architecture-presentation.pptx](../VIA-architecture-presentation.pptx) 하나다. 2026-10-11에 기존 일곱 자료의 32장을 발표 순서대로 통합했다. 이후 파일명은 유지하고 이력은 Git에서 관리한다. 통합에 사용한 개별 PPTX는 이전 구성의 참고본으로 보존하며, 통합본을 다시 만드는 입력으로 사용하지 않는다.

[2026-10-11 독립 발표자료 리뷰](./review-2026-10-11.md)는 도메인 초심자 심사위원의 이해, storytelling과 고정 QA 연결, DP 구조 대비와 가독성의 보완안을 담는다. 리뷰 기준의 현재 32장에 대한 기록이며, 제안은 아직 PPTX나 QA 정의에 적용하지 않았다.

## 발표 순서

| PowerPoint 슬라이드 위치 | 구역 | 내용 |
| --- | --- | --- |
| 1~3 | 배경과 필요성 | 관련 동향, VIA의 역할과 범위, 과제 필요성 |
| 4 | 대표 시나리오 | 표 설명부터 두 업무 위임, 부분 정정, 질문 답변과 Text 상태 확인 |
| 5 | 기능 요구사항 | 기능 6개와 제약사항 2개 |
| 6~11 | 품질 요구사항 | 여섯 QA, P21 공통 활동과 21개 사례 대응 |
| 12~17 | QA 목표 근거 | QA-01~06의 목표 근거, 계산 및 7단계 등급 |
| 18 | 설계 Overview | 최신 품질 목표, 모델 배치와 네 DP 매핑 |
| 19~20 | 사용자 경험 | 업무 위임과 겹쳐 도착하는 사건 |
| 21~23 | DP41 | 배경, 구조 비교, QA 상세 |
| 24~26 | DP44 | 배경, 구조 비교, QA 상세 |
| 27~29 | DP45 | 배경, 구조 비교, QA 상세 |
| 30~32 | DP42 | 배경, 구조 비교, QA 상세 |

위 구역은 PowerPoint의 실제 Section으로도 저장했다. 여기서 페이지는 통합본의 슬라이드 위치다. 품질 두 구역의 `1 / 6` 표시는 각각의 구역 내 순서이며 전체 페이지 번호가 아니다. 대표 시나리오의 표시 번호는 통합 위치에 맞춰 4로 바꿨다. 43과 46은 포함하지 않았다.

전체 크기는 16:9, 1920×1080에 해당하는 20×11.25in이다. 처음 네 장의 1280×720 개체는 좌표, 크기와 서식을 1.5배로 맞췄다. 텍스트, 표와 도형은 기존 편집 가능한 개체로 유지했다. 원래 이미지 개체와 두 시연 GIF, 32장의 발표자 노트도 보존했다. 그림 내부의 래스터 요소는 이미지로 유지한다.

## 앞으로 수정하는 방법

PowerPoint에서 통합본을 열어 해당 슬라이드를 직접 수정하고 같은 파일에 저장한다. Codex에 요청할 때도 “통합본 18장 설계 Overview 수정”, “DP41 비교와 QA 상세만 수정”처럼 대상과 변경 내용을 지정하면 된다. 항상 **현재 통합본의 사본에서 시작**하고 비대상 슬라이드와 노트를 보존한다. 일곱 개별 파일을 먼저 고친 뒤 전체를 재병합하면 통합 후 편집 내용이 사라질 수 있으므로 그 방식은 사용하지 않는다.

Architecture 내용의 근거는 [활성 문서](../../architecture/README.md)다. 품질 정의를 바꾸면 해당 Markdown/JSON과 통합본의 관련 장을 함께 갱신한다. Architecture 그림이 바뀌면 대응하는 draw.io와 SVG 원본도 함께 유지한다. 발표의 문구와 배치만 편집할 때는 Architecture의 계약을 자동 개정하지 않는다. 개별 폴더의 PNG와 PPTX는 통합 시점의 참고본이며 통합본을 편집해도 자동 갱신되지 않는다.

### 자동화로 특정 장만 교체

[manifest.json](./manifest.json)에 최초 위치, 의미별 slide key와 원본 파일/hash를 남겼다. key는 각 슬라이드의 `p:cSld/@name`에도 저장되어 페이지가 이동해도 대상을 식별할 수 있다. 슬라이드를 추가·삭제·재배치하면 이 안내의 위치 표와 manifest의 위치/구역 정보를 함께 갱신한다. `initial_*` 필드와 원본 hash는 최초 통합 이력으로 유지한다.

[부분 교체 도구](../../../scripts/presentations/full_presentation_package.py)는 현재 통합본, 수정한 PPTX와 명시한 key만 받아 새 candidate를 만든다. 수정한 PPTX의 페이지 번호는 1부터 시작한다. 예를 들어 그 파일의 1장을 통합본의 기능 요구사항으로 교체하려면 매핑 JSON을 다음과 같이 만든다.

```json
{"functional-requirements": 1}
```

```text
python3 scripts/presentations/full_presentation_package.py replace CURRENT.pptx EDITED.pptx MAPPING.json CURRENT_SHA256 NEW_CANDIDATE.pptx REPORT.json
```

`CURRENT_SHA256`는 편집 시작 시 현재 통합본 bytes의 SHA-256이다. 도구는 hash가 달라지면 중단하여 다른 편집을 덮지 않으며, 기존 candidate도 덮어쓰지 않는다. 선택한 장의 개체와 노트와 필요한 의존 자원만 가져온다. 비선택 슬라이드/노트와 기존 공용 자원 및 media가 bytes로 유지되는지 검사하고 슬라이드 ID, 순서와 Section은 보존한다. 수정 장을 재가져와 렌더링하고 package/layout/font 검사를 마친 후 candidate를 현재 통합본에 반영한다. 검수 산출물은 private build에 둔다.

주요 key는 `design-overview`, `dp41-comparison`, `dp41-qa`, `dp44-comparison`, `dp45-comparison`, `dp42-comparison`이다. 여섯 목표 근거는 `qa01-metric`~`qa06-metric`, 품질 요약 두 장은 `quality-01-03`과 `quality-04-06`이다. 전체 32개는 manifest를 따른다.

[최초 통합 스크립트](../../../scripts/presentations/assemble_full_presentation.mjs)는 최초 이관의 재현용이다. 현재 통합본이 존재하면 실행을 거부한다. 이후 업데이트에는 현재 파일을 직접 편집하거나 위 부분 교체 도구를 사용한다.
