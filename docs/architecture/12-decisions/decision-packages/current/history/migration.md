# 2026-10-22 현재 패키지 이관

사용자는 DP2, DP3, DP5, DP44의 최신 설계 파일, 36쪽 발표 v10과 공통 QA v3를 확인했고, QA 후보 패키지의 목표와 점수 원전까지 함께 편집할 수 있도록 한곳으로 모으라고 요청했다. PPTX와 제작 지침도 같은 패키지에서 관리한다.

## 이관 원칙

- 새 현재 경로는 `docs/architecture/12-decisions/decision-packages/current/`다.
- 설계와 QA 문서는 링크를 새 위치에 맞춰 이관하고 현재 안내와 과거 설명의 적용 범위를 구별한다. 후속 설계 보정은 DP별 읽기 문서에서 연결한다.
- 원본 위치의 본문과 이전 판본은 출처 및 기존 생성기/평가의 재현을 위해 보존한다. 주요 이전 진입점에서 새 현재 경로를 안내한다. 후속 편집 정본은 새 패키지다.
- PPTX, 계산 원장, Excel, HTML과 기존 검증 자료는 해당 판본의 bytes를 보존한다. 문서 이관과 상대 링크 보정은 새 성능 평가나 PPTX 재작성으로 보지 않는다.
- 평가의 부모 자료와 과거 명령은 해당 판본의 재현 자료다. 현재 편집본의 변경을 기존 원장으로 자동 반영하지 않는다.
- 새 QA07과 이전 QA07, COMMON-28과 v3 사례군, 조건부 권고와 사용자 채택을 구별한다.

## 원본 경로와 검증

[package-manifest.json](../package-manifest.json)은 이관 당시 원본 commit, 원본 경로와 SHA256, 새 경로, byte 보존 여부를 기록한다. `byte_preserved`는 이관 당시 동일성이다. 후속 수정 가능 여부는 별도 `artifact_policy`로 구분한다. `EDITABLE_SOURCE`인 설계, QA와 현재 그림은 이곳에서 수정한다. `IMMUTABLE_SNAPSHOT`인 기존 PPTX와 평가 산출물 및 재현 자료는 보존하고 후속 변경을 새 판본으로 만든다. 분야별 세부 이관 목록은 [설계](../design-migration.json), [QA](../qa-migration.json), [발표](../presentation-migration.json)에 있다.

원본이 `history`에 있다는 이유만으로 현재 요구나 채택 상태가 되지 않는다. 반대로 오래된 날짜라는 이유만으로 여전히 유효한 사용자 지시를 폐기하지 않았다. 저장소 전체의 INTENT와 AGENTS, accepted ADR 및 실측 원시는 기존 정본을 유지한다.

Windows 검증 보고서 2개는 원본 작업 폴더의 CRLF bytes를 보존했다. Git 원본은 줄바꿈이 LF로 정규화되어 있어 원장에 `source_git_sha256`와 `source_encoding_note`를 함께 남겼다. 새 고정 snapshot은 `.gitattributes`로 줄바꿈 변환을 막는다.

검사 범위와 결과는 [이관 검토](review.md)에 기록한다. 후속 작업은 [현재 README](../README.md)에서 시작한다.
