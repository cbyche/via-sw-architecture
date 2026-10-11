# VIA 통합 발표자료와 버전별 편집

현재 발표와 후속 수정의 기준은 [VIA-architecture-presentation_v3.pptx](../VIA-architecture-presentation_v3.pptx)다. 2026-10-12에 v2를 기준으로 네 DP 배경의 시각적 설명을 주제별로 구분했다. **본문 24장, appendix 안내 1장, 최초 원본 보존 32장, v2 배경 보존 4장으로 총 61장**이다.

[최초 통합본](../VIA-architecture-presentation.pptx)과 [v2](../VIA-architecture-presentation_v2.pptx)는 변경하지 않았다. v2에서 종합한 [독립 리뷰](./review-2026-10-11.md)와 [사용자 구성 지시](./v2-composition.md)의 발표 순서를 유지한다. 본문에서 대체하거나 이동한 내용도 원본 슬라이드와 발표자 노트, 이미지 및 두 시연 GIF 그대로 appendix에 보존했다. 처음 통합에 사용한 일곱 개별 PPTX도 이전 구성의 참고본이며 후속 업데이트의 입력이 아니다.

## 현재 발표 순서

| 실제 슬라이드 위치 | 구역 | 내용 |
| --- | --- | --- |
| 1~6 | 배경과 소개 | 관련 동향 → 필요성 → VIA 역할 → 도메인과 전체 흐름 → 대표 시나리오 → 업무 위임 책임 |
| 7~11 | 요구사항과 QA | 요구의 근거 → 기능 6개와 제약 2개 → 고정된 여섯 QA → 두 시간 지표의 경계 |
| 12~14 | 설계 도입 | 요구/품질/설계 질문 연결 → 설계 Overview → 겹쳐 도착하는 사건 |
| 15~16 | DP41 | 배경과 선택 질문 → A/B 구조와 여섯 QA 평가표 |
| 17~18 | DP44 | 배경과 선택 질문 → A/B 구조와 여섯 QA 평가표 |
| 19~20 | DP45 | 배경과 선택 질문 → A/B 구조와 여섯 QA 평가표 |
| 21~22 | DP42 | 배경과 선택 질문 → A/B 구조와 여섯 QA 평가표 |
| 23~24 | 검토 요약 | 기능 적용 범위와 공통 계약 → 선택 전 확인할 사항 |
| 25 | Appendix 안내 | 본문과 원본 상세 자료의 위치 안내 |
| 26~35 | Appendix / QA 상세 | 원본 8~17장: P21, 21개 사례 대응, QA-01~06 목표 근거와 등급 |
| 36~42 | Appendix / 원본 보존 | 원본 1~7장: 배경, 역할, 필요성, 시나리오, 요구사항과 QA |
| 43~57 | Appendix / 원본 보존 | 원본 18~32장: Overview, 책임과 겹침 장면, 네 DP의 배경/비교/QA 상세 |
| 58~61 | Appendix / v2 배경 보존 | v2의 15, 17, 19, 21장: DP41, DP44, DP45, DP42의 기존 배경 |

원본 22, 25, 28, 31장에 해당하는 본문 비교는 **16, 18, 20, 22장**이다. 각 페이지에 설계 비교와 여섯 QA를 함께 배치했다. 원래 비교 페이지는 각각 47, 50, 53, 56장에 보존했고, DP별 상세 QA와 지역 계산은 48, 51, 54, 57장에 있다. 보존한 슬라이드 안의 번호, 버전과 제목은 당시 표기이며 현재 위치는 위 표와 [manifest-v3.json](./manifest-v3.json)을 따른다.

### v3에서 구분한 네 배경

| 본문 위치 | DP | 첫눈에 보이는 주제와 표현 |
| --- | --- | --- |
| 15 | DP41 | 이번 지시의 대상과 적용 범위: 표현, 확인할 근거, 완성할 요청 의미를 대응하는 표 |
| 17 | DP44 | 겹쳐 도착하는 사건과 응답 시점: 사용자 발화, 음성 중단, 업무 질문, 늦은 응답 후보의 시간 흐름 |
| 19 | DP45 | 흩어진 과거 근거의 연결과 재사용: 서로 다른 원본의 기록에서 이번 요청의 Context로 이어지는 근거 |
| 21 | DP42 | 서로 다른 요청과 실행을 잇는 업무 상태: 여러 Request와 Execution을 같은 Task에 연결하는 관계 |

각 배경의 마지막 질문은 바로 다음 A/B 비교로 이어진다. 새 도식은 양안이 해결해야 할 공통 문제를 설명하며 특정 방안을 선택한 구조가 아니다. v2 배경 네 장은 다시 사용할 수 있도록 appendix 마지막 58~61장에 슬라이드, 노트와 자원을 그대로 옮겼다. 비교/QA 페이지를 포함한 나머지 v2 슬라이드도 그대로 유지한다.

도메인 용어는 소개에서 설명하고, 필요성에 모델 사용 비용과 PC 메모리의 원인을 추가했다. 요구사항에서 고정 QA를 거쳐 네 구조 질문으로 이어지는 연결과 DP 배경의 선택 질문을 보완했다. DP 제목은 `(주요 품질속성)을 위한 (기능) 설계` 형식을 따른다. 그림은 실제 판단, 생산, 상태 소유와 확정 방식의 차이를 보여주며 강한 A와 B의 비용을 함께 남긴다.

여섯 QA의 정의, 목표, 모집단, 보호 조건과 등급은 변경하지 않았다. 비교 표의 `평가 전`, 메모리 예산과 지역 가정 계산을 구별한다. A/B 선정, 구현 또는 실측 결과를 추가하지 않았다. 43과 46은 본문에 포함하지 않았다. 전체 크기는 기존과 같은 16:9, 20×11.25in이다. 신규 텍스트, 도형과 표는 PowerPoint에서 편집할 수 있다. [v2-validation.md](./v2-validation.md)는 v2 발행 당시의 검증 이력이다.

## 앞으로 수정하는 방법

항상 **최신 버전의 사본에서 시작해 새 버전으로 저장**한다. 다음 수정은 `_v3.pptx`를 입력으로 `_v4.pptx`를 만들고, 기존 버전은 유지한다. 최초 통합본이나 일곱 개별 파일을 수정한 뒤 다시 전체 병합하면 후속 편집이 사라질 수 있으므로 사용하지 않는다.

요청에는 “최신본의 DP41 비교와 QA만 수정”, “최신본의 13장 설계 Overview 수정”처럼 대상과 변경 내용을 지정하면 된다. 비대상 슬라이드, 노트와 자원은 보존한다. QA 개정은 별도 명시적 요청이 있을 때만 한다. 발표 문구와 배치 편집으로 Architecture 계약을 자동 개정하지 않는다. 구조의 근거는 [활성 Architecture 문서](../../architecture/README.md)이며, 원본 draw.io와 SVG의 수정은 해당 문서 작업으로 함께 관리한다.

### 자동화로 특정 장만 교체

[manifest-v3.json](./manifest-v3.json)에 현재 위치, Section, 의미별 slide key, 원본 대응과 보존 hash를 남겼다. key는 각 슬라이드의 `p:cSld/@name`에도 저장되어 페이지가 이동해도 대상을 식별한다. 새 버전에서는 비대상 key를 유지하고 새 위치와 hash를 기록한다. [manifest-v2.json](./manifest-v2.json)과 [manifest.json](./manifest.json)은 이전 버전의 이력이다.

[부분 교체 도구](../../../scripts/presentations/full_presentation_package.py)는 현재 버전, 수정한 PPTX와 명시한 key만 받아 새 candidate를 만든다. 예를 들어 수정 PPTX의 1장으로 현재 DP41 비교를 교체하려면 매핑 JSON을 다음과 같이 만든다.

```json
{"dp41-comparison-v2": 1}
```

```text
python3 scripts/presentations/full_presentation_package.py replace CURRENT.pptx EDITED.pptx MAPPING.json CURRENT_SHA256 NEW_CANDIDATE.pptx REPORT.json
```

`CURRENT_SHA256`는 편집 시작 시 최신 파일의 SHA-256이다. 도구는 hash가 바뀌거나 candidate가 이미 있으면 중단한다. 선택한 장과 필요한 자원만 가져오고 비선택 슬라이드/노트 및 기존 media의 bytes 보존을 검사한다. 렌더링과 package/layout/font 검증 후 **새 버전 파일**로 발행한다.

본문의 주요 key는 `overview-v2`, `dp41-comparison-v2`, `dp44-comparison-v2`, `dp45-comparison-v2`, `dp42-comparison-v2`, `quality-01-03-v2`, `quality-04-06-v2`다. 원본 보존 장은 최초 key를 유지한다. 예를 들어 `dp41-comparison`은 appendix 47장으로 본문 key와 다르다.

새 배경 key는 `dp41-background-v3`, `dp44-background-v3`, `dp45-background-v3`, `dp42-background-v3`다. 기존 `*-background-v2`는 appendix 58~61장으로 이동했다. [v3 배경 작성 스크립트](../../../scripts/presentations/update_dp_backgrounds.mjs)와 [배경 보존 패키저](../../../scripts/presentations/package_background_revision.py)는 v2→v3 변경을 재현하며 v2의 57장과 기존 자원의 보존을 검사한다.

[v2 구성 스크립트](../../../scripts/presentations/rebuild_presentation_v2.mjs)와 [원본 보존 패키저](../../../scripts/presentations/package_presentation_v2.py)는 32→57장 이관의 재현용이다. 최초 통합본의 hash를 확인하고 private build에 candidate를 만든다. 이후 부분 수정에는 최신본과 부분 교체 도구를 사용한다. [최초 통합 스크립트](../../../scripts/presentations/assemble_full_presentation.mjs)도 초기 이관 이력으로만 유지한다.
