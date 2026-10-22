# VIA 통합 발표자료와 버전별 편집

> **2026-10-22 현재 편집 위치:** 사용자 확인 후 이 자료를 [네 DP 통합 패키지의 현재 기준본](../../architecture/12-decisions/decision-packages/current/presentation/README.md)으로 이관했다. 후속 수정은 새 위치에서 한다. 아래 내용과 기존 파일은 해당 판본의 출처 및 재현을 위해 보존하며 새 편집본과 동시에 갱신하지 않는다.

현재 발표와 후속 수정의 기준은 [VIA-architecture-presentation_v17.pptx](../VIA-architecture-presentation_v17.pptx)다. **본문 40장과 QA Appendix 9장으로 총 49장**이다. [전체 미리보기](./overview-v17.jpg), [manifest-v17.json](./manifest-v17.json), [검수 기록](./review-v17-2026-10-15.md)을 제공한다.

- **필요성 2~3장:** [과제 소개 v10](../VIA-project-introduction_v10.pptx)의 2장을 먼저 넣고, [v9](../VIA-project-introduction_v9.pptx)의 2장을 다음에 넣었다. 기존 필요성 5안은 4~8장에 그대로 남겼다.
- **DP 비교 21~24장:** 20장 요약 표 바로 뒤에 [레퍼런스 형식 비교 4장](../VIA-DP-comparisons-reference_v1.pptx)을 DP-44, DP-47, DP-45, DP-42 순서로 연속 배치했다.
- 기존 v16의 43장과 추가 원본 6장의 본문, 표, 그림과 노트를 유지했다. 전체 페이지 번호만 갱신했으며 기존 DP 상세 구간의 상대 순서도 유지했다. QA 수치는 원본의 구조 기반 예상이며 실측이나 최종 선정이 아니다.

이전 [v16 통합본](../VIA-architecture-presentation_v16.pptx)은 보존한다. 설계 Overview 17장 바로 뒤의 **18장에 DP-44 / DP-47 / DP-45 / DP-42 요약 표**를 추가했다. VIA 문제, 설계 주제, A/B 차이와 평가를 편집 가능한 표로 정리한다. 평가는 구조 기반 예상이며 실측이나 최종 선정이 아니다. 본문 34장과 QA Appendix 9장으로 **총 43장**이다. [요약 표 미리보기](./dp-summary-v16.png), [manifest-v16.json](./manifest-v16.json), [검수 기록](./review-v16-2026-10-15.md)을 제공한다.

기존 42장의 본문과 노트, 자원 및 순서를 유지하고 페이지 번호만 갱신했다. 공통 요청 장면은 19장, DP44 시작은 20장이다. 요약 표의 순서는 사용자 지정 44 / 47 / 45 / 42이며 기존 본문 재배치는 이번 수정에 포함하지 않는다.

이전 [v15 통합본](../VIA-architecture-presentation_v15.pptx)은 보존한다. DP44의 복잡한 종합 그림을 참고 자료로 보존하고, 그 앞에 상황별 비교 네 장을 넣었다. **본문 33장과 QA Appendix 9장으로 총 42장**이다. [v15 검수 기록](./review-v15-2026-10-15.md)과 [manifest-v15.json](./manifest-v15.json)을 제공한다.

DP44의 21~24장은 [일반 사용자 발화](../../architecture/12-decisions/decision-packages/diagrams/dp44-scenario-input.png), [Agent 질문](../../architecture/12-decisions/decision-packages/diagrams/dp44-scenario-agent.png), [새 발화 시작](../../architecture/12-decisions/decision-packages/diagrams/dp44-scenario-onset.png), [정정과 늦은 완료](../../architecture/12-decisions/decision-packages/diagrams/dp44-scenario-correction.png)를 각각 한 경로씩 비교한다. 모든 SVG와 draw.io 원본은 [04-44 §3.7.4](../../architecture/12-decisions/decision-packages/04-44-continuous-interaction.md#374-상황별-실행-경로)에 직접 표시했다. 기존 DP44 비교 20장과 종합 구현 그림 25장도 남아 있다. [상황별 4장 PPTX](../dp-comparison/VIA-DP44-scenario-comparisons_v1.pptx)는 별도로 편집할 수 있다.

[v14 통합본](../VIA-architecture-presentation_v14.pptx)은 이전 판본으로 보존한다. v14는 기존 DP44 설계 비교 20장을 보존하고 바로 뒤인 21장에 [구현 구조 종합 그림](../dp-comparison/dp44-implementation-comparison.png)을 추가한 판본이다. 본문 29장과 QA Appendix 9장으로 총 38장이었다. [v14 검수 기록](./review-v14-2026-10-15.md)과 [manifest-v14.json](./manifest-v14.json)을 유지한다.

새 21장은 A의 중앙 Event Mailbox와 Dialogue Dispatcher, B의 Stage별 Control, Input 또는 Task, Done Queue와 Publication Join을 같은 사건으로 비교한다. Queue와 상태 원통, 일반 입력과 제어 신호, 비동기 완료와 늦은 Job 폐기 위치를 구별한다. [편집 가능한 draw.io](../dp-comparison/dp44-implementation-comparison.drawio), [SVG](../dp-comparison/dp44-implementation-comparison.svg), [한 장 PPTX](../dp-comparison/VIA-DP44-implementation-comparison.pptx)를 별도로 제공한다. 구현, 품질 측정, A/B 선정은 아직 하지 않았다.

[v13 통합본](../VIA-architecture-presentation_v13.pptx)은 이전 판본으로 보존한다. v13은 사용자 요청에 따라 v11의 필요성 1장을 v12의 시각 대안 5장으로 교체한 2026-10-15 판본이며 본문 28장과 QA Appendix 9장으로 총 37장이다. [v13 검수 기록](./review-v13-2026-10-15.md), [manifest-v13.json](./manifest-v13.json) 제공.

**필요성 2~6장:** 3열 장면형, PC 중심 통합형, 가로 3단형, 확대 장면형, 정보 레이어형 순서. 다섯 안의 그림과 문구 및 노트를 보존하고 전체 페이지 번호와 안 이름을 표시. [5장 비교 PPTX v12](../VIA-necessity-options_v12.pptx)와 [확대 미리보기](../necessity-options-v12/index.html)도 유지.

v15에서도 v14의 38장은 페이지 번호를 제외한 본문과 노트를 보존했다. QA 정의와 목표, 표 및 기존 구조 그림의 내용 변경 없음. [현재 QA 원문](../../architecture/12-decisions/decision-packages/qa-candidate-package/README.md)과 [Architecture 문서](../../architecture/README.md) 참조. 기존 FR 분류와 화면/음성 구조 그림의 최신 Architecture 동기화는 별도 작업. DP 관계 문장은 가설이며 수치 예시는 실측 결과가 아님.

v1~v14의 통합본과 v12의 별도 비교본은 이력으로 보존.

[v13 Windows Open XML 검사](https://github.com/cbyche/via-sw-architecture/actions/runs/38607452044) 및 [통합본 commit의 Architecture CI](https://github.com/cbyche/via-sw-architecture/actions/runs/38607428134) 통과. Windows에서 37장 열기와 스키마 오류 0건 확인. PowerPoint 자체 렌더링과 GIF 재생은 미검증.

## 현재 발표 순서

| 실제 슬라이드 위치 | 구역 | 내용 |
| --- | --- | --- |
| 1 | 관련 동향 | 기존 관련 동향 |
| 2 | 과제 필요성 | v10의 도식형 필요성 |
| 3 | 과제 필요성 | v9의 일러스트형 필요성 |
| 4~8 | 과제 필요성 | 기존 그림 구성 5안 전체 |
| 9~12 | 과제 소개 | VIA 역할, 전체 흐름, 대표 시나리오, 업무 위임 책임 |
| 13~14 | 기능 요구사항 | 기존 요구의 근거와 기능 및 제약 표 |
| 15~16 | 품질 요구사항 | QA01~QA06 정의와 목표 및 조건 |
| 17 | 시간 지표 | 첫 UI 응답, VIA 처리 완료, 실제 음성 전달 |
| 18~19 | 설계 도입 | 품질 영향 경로와 설계 Overview |
| 20 | DP 요약 | 44 / 47 / 45 / 42의 문제와 설계 및 평가 |
| 21 | 레퍼런스 비교 | DP-44 |
| 22 | 레퍼런스 비교 | DP-47 |
| 23 | 레퍼런스 비교 | DP-45 |
| 24 | 레퍼런스 비교 | DP-42 |
| 25 | 공통 요청 장면 | 기존 공통 요청 배경 |
| 26~27 | DP44 | 정정과 업무 알림 배경, 기존 A/B 구조와 QA 표 |
| 28~31 | DP44 상황별 실행 | 일반 발화, Agent 질문, 새 발화 시작, 정정과 늦은 완료 |
| 32 | DP44 종합 그림 | 기존 구현 구조 비교 |
| 33~35 | DP45 | 과거 근거 배경, 기본 구조와 택틱, 기존 A/B 구조와 QA 표 |
| 36~37 | DP42 | 상태의 다른 수명, 기존 A/B 구조와 QA 표 |
| 38 | 화면과 음성 | 화면 전환 및 정정 문제 소개 |
| 39~40 | 검토 요약 | 기능 범위, 공통 조건, 평가 전 확인 사항 |
| 41~49 | QA Appendix | QA01~QA09 상세 설명 |

아래 버전별 설명의 페이지 번호는 해당 판본 기준이다.

## 이전 버전의 작성 이력

### v11의 QA 갱신

[v11 통합본](../VIA-architecture-presentation_v11.pptx)은 본문 24장과 QA Appendix 9장으로 총 33장. 기존 Appendix 25~63장 39개를 제거하고 QA01~QA09를 한 장씩 작성. 본문 9~11장과 12/13/16/19/21/23/24장의 QA 인용도 최신 패키지와 정렬. QA01과 QA03은 산술평균이며 QA06의 목표는 사용자 지정 512 MB.

[QA 미리보기와 편집 원본](../qa-metrics-v11/README.md), [v11 검수 기록](./review-v11-2026-10-15.md), [manifest-v11.json](./manifest-v11.json) 보존. [v11 Architecture CI](https://github.com/cbyche/via-sw-architecture/actions/runs/38602750338)와 [Windows Open XML SDK 검사](https://github.com/cbyche/via-sw-architecture/actions/runs/38602763042) 통과. PowerPoint 자체 렌더링은 별도.

### v10의 과제 소개

v9의 사용자 주석 10건을 반영해 1~2장을 수정했다. 관련 동향의 양방향 관계와 공식 발표일, 관련 상위 시장의 범위를 명확히 했고 필요성 그림을 중첩 시간축과 관계도 및 원인 관계도로 교체했다. [두 장 전용 PPTX](../VIA-project-introduction_v10.pptx), [미리보기와 출처](../project-introduction-v10/README.md), [검수 기록](./review-v10-2026-10-15.md), [manifest-v10.json](./manifest-v10.json)을 보존한다. v10은 총 63장이며 v11은 그 1~2장의 본문과 자원을 유지한다.

### v8의 문제 소개와 편집 원본

사용자 장면 → 잘못 처리했을 때의 피해 → 구체적인 SW 어려움 → 설계 질문 순서로 문장을 다듬었다. 45는 A의 요청별 원본 구성과 B의 가공 기억 유지·조회라는 기본 구조를 먼저 설명하고, 요약·색인·cache와 갱신 택틱을 별도로 둔다. 42의 논리 수명 차이로 B의 서비스 분리 우위를 주장하지 않는다. 새 화면·음성 페이지는 32/46의 대안을 계승하지 않으며 시간 겹침만으로 지칭 의도를 확정하지 않는다.

[현재 6장 모아 보기](../dp-background/index.html)와 [편집 가능한 draw.io](../dp-background/VIA-problem-intros-v8.drawio)를 제공한다. [공통 장면 정의](../../../scripts/architecture/via_problem_intro_scenes.py)에서 SVG, draw.io와 PPTX native 도형을 작성해 같은 문구와 배치를 사용한다. [v8 작성 스크립트](../../../scripts/presentations/update_problem_intros_v8.mjs), [패키지 보존 도구](../../../scripts/presentations/package_problem_intros_v8.py)와 [렌더링 도구](../../../scripts/presentations/verify_problem_intros_v8.mjs)가 재현 원본이다. bundled runtime의 환경 변수와 private build 사용법은 아래 기존 작성 기록 및 검수 기록을 따른다.

63장 전체를 렌더링하고 변경 페이지를 확인했다. 파일 구조, geometry, native 표, Calibri·정수 pt·한국어 정책을 검사했다. v7의 Windows cloud 검증은 v8의 검증으로 인용하지 않는다. **v8은 Windows Open XML SDK와 PowerPoint 자체 실행·글꼴 배치·GIF 재생을 미검증**이다. 모델 실행, 품질 측정이나 A/B 최종 선정은 하지 않았다. 후속 사용자 요청에 따라 발표자료와 관련 그림·생성 원본을 commit/push 대상으로 지정했다.

### v7의 Claude 브라우저 시연

첫 장 왼쪽을 [Claude Cowork 내장 브라우저 공식 발표](https://claude.com/resources/articles/cowork-built-in-browser)의 [공개 시연](https://www.youtube.com/watch?v=63GVebZvqok)으로 교체했다. “요금 조사 요청 → 웹사이트 열기 → 요금제 확인”을 설명한다. 원래 영상에서 Northwind의 요금제를 확인하는 장면을 사용하며, 비교 결과 작성이나 모든 데스크톱 앱 조작을 시연한 것으로 설명하지 않는다. 첫 장의 네 설명 문구와 왼쪽 GIF, 관련 출처 노트만 변경했다. 오른쪽 음성 수정 사례와 첫 장의 나머지 객체도 유지한다.

[무음 반복 GIF](./assets/claude-cowork-browser-20260826.gif)는 공식 기사에 삽입된 플레이어의 실제 브라우저 화면을 캡처한 것이다. 약 6.5~19.2초를 원래 재생 시점으로 발췌하고 첫 요금제 화면 1초와 마지막 화면 1.8초를 더해 약 15.4초로 구성했다. 68프레임, 552×310px이며 [자원 출처와 발췌 정보](./assets/claude-cowork-browser-20260826.json)를 함께 보관한다. 기존 2024 시연 GIF는 다른 media part로 남겨 appendix 36장에서 계속 사용한다.

[v7 작성 스크립트](../../../scripts/presentations/update_claude_first_slide_v7.mjs)는 v6를 Artifact Tool로 열어 첫 장의 텍스트와 이미지만 편집한다. 원본 package에 해당 문구, 이미지 관계와 노트를 반영하고 새 GIF의 content type을 등록한다. 변경 가능한 기존 part 네 개는 첫 slide XML, 해당 관계, 해당 notes XML과 media 등록용 `[Content_Types].xml`이다. 나머지 package part, 원래 25개 media part, 2~61장의 slide XML 및 노트는 bytes까지 동일하게 유지한다. [manifest-v7.json](./manifest-v7.json)에 이전 파일 hash, 변경 범위와 각 장의 hash를 기록한다. bundled `node_modules`를 연결한 private build에 작성 스크립트를 복사하고 v6 작성과 같은 환경 변수 및 저장소와 build 절대 경로를 전달하면 재현할 수 있다.

로컬 검증에서 package, geometry와 native 표 검사를 통과했고 Calibri, 정수 pt와 한국어 정책 위반은 0건이다. 61장을 모두 렌더링해 첫 장의 배치를 확인하고, 2~61장의 렌더링 결과가 v6와 픽셀까지 동일함을 확인했다. GIF의 68프레임 중 67개가 서로 다른 이미지이며 반복 설정과 15.38초 재생 시간을 확인했다. 이는 파일의 애니메이션 데이터 검사이며 PowerPoint 자체의 재생 검증은 아니다.

최종 v7는 [Windows cloud 검사](https://github.com/cbyche/via-sw-architecture/actions/runs/38231504590)를 통과했다. Calibri가 설치된 Windows Server 2022에서 Microsoft Open XML SDK 3.3.0으로 61장을 열었으며 Office2021 스키마 오류는 0건이다. SHA-256 `80f5762b962da0d732f973a11eda706b6d66f52e530bacdd05a8d98c3519e44e`는 로컬 최종 파일과 일치한다. Cloud에는 PowerPoint가 설치되어 있지 않아 프로그램 자체의 실행, native 글꼴 배치와 GIF 재생은 미검증이다. [v7 변경의 Architecture CI](https://github.com/cbyche/via-sw-architecture/actions/runs/38231484234)도 통과했다.

### v6의 참조 스타일과 도형 텍스트

참조 자료의 실제 저장 경로는 [reference_presentaion/presentation_kjh](../reference_presentaion/presentation_kjh/)다. 제공된 58개 PNG를 확인하고 공통 서식을 편집 가능한 객체로 다시 작성했다. 흰 배경, 검정색 제목과 본문, 회색 구분선, 회색 진행 단계와 청록색 현재 단계, 연한 초록색 표 머리글을 적용한다. 품질 요구사항 표의 본문은 연한 크림색이다. `설계 질문` 표지만 빨간색으로 강조하고 질문 문장은 검정색으로 유지한다. 상단 진행 표시의 흰 글자는 대비를 위한 탐색 서식이다.

기존 도형의 별도 라벨을 도형 자체의 `p:txBody`로 옮겼다. 모듈, 저장소, 실행 경계와 강조 상자는 자신의 텍스트를 가지며 도형과 라벨을 따로 움직일 필요가 없다. 부록의 비교표 머리글도 25개의 native 도형 셀로 작성했다. 동작이나 전달 데이터의 설명은 원래 화살표 근처에 유지한다. 배치가 다른 설명들을 하나의 도형 제목으로 합치지 않는다. 좁은 마름모의 글자는 가운데 정렬하고 내부 여백을 조정해 잘림을 방지했다.

본문 1, 3~6, 8, 12, 15~16장에서 화면 지칭의 강조를 줄였다. 시나리오의 표와 화면은 남기되, DP41의 예는 매출 자료, 이전 보고서와 조건 및 결과 의존 관계를 중심으로 설명한다. DP44의 사건 겹침, DP45의 과거 근거, DP42의 상태 수명은 유지한다. 여섯 QA의 정의, 목표값, 모집단과 등급 표의 문구는 변경하지 않았다. 슬라이드 순서와 ID, key, 노트 내용과 25개 media part의 원본 bytes도 보존한다.

[v6 작성 스크립트](../../../scripts/presentations/restyle_presentation_v6.mjs)는 Artifact Tool에서 기존 텍스트와 표를 편집하고 공통 머리글을 작성한다. [native 패키지 보존 도구](../../../scripts/presentations/restyle_presentation_v6.py)가 기존 도형의 텍스트를 합치고 원본 자원을 보존한다. `prepare SOURCE.pptx PLAN.json`으로 변경 계획을 만들고, v5 작성 스크립트와 같은 bundled 환경을 설정한 private build에서 v6 스크립트의 사본에 저장소와 build 절대 경로를 전달한다. 이 도구는 v5→v6 재현용이며 이후 수정은 최신 통합본을 입력으로 한다.

Windows cloud의 Microsoft Open XML SDK로 v5를 검사해 원래 통합본의 마스터 등록 오류 3건을 확인했다. v6에서는 `slideMasterId`를 표준 `sldMasterId`로 바꾸고, presentation의 notes master를 하나로 등록하며 각 notes slide를 그 master에 연결했다. 기존 노트와 다른 마스터 자원은 보존한다. [Windows 검사 workflow](../../../.github/workflows/presentation-windows-check.yml)는 글꼴과 한국어 정책, 61장의 package 열기와 Office2021 스키마를 검사하고 PowerPoint 설치 여부도 기록한다. SDK 검증과 PowerPoint의 직접 실행 및 렌더링 검증은 구별한다.

최종 v6는 [Windows cloud 검사](https://github.com/cbyche/via-sw-architecture/actions/runs/38207406108)를 통과했다. Calibri가 설치된 Windows Server 2022에서 Microsoft Open XML SDK 3.3.0으로 61장을 열었으며 Office2021 스키마 오류는 0건이다. 검사한 파일의 SHA-256은 `78eabff07ee0ba6c98e8abacf3e50bb055f1d526bf47b705a48c8d7f8ede2bf8`로 로컬 완성본과 일치한다. Calibri, 정수 pt와 한국어 정책도 통과했다. Cloud 환경에는 PowerPoint가 설치되어 있지 않아 직접 실행, native 글꼴 배치와 GIF 재생은 미검증이다. 로컬에서는 61장 모두를 렌더링해 확인했으며 package, geometry와 native 표 검사에서 오류나 경고는 없었다. [v6 변경의 Architecture CI](https://github.com/cbyche/via-sw-architecture/actions/runs/38207386475)도 통과했다.

### v5와 이후 PPTX의 작성 기준

사용자가 지정한 다음 기준은 [저장소 에이전트 지침](../../../AGENTS.md)의 PPTX 정책에도 기록했다.

- 사용 환경: Windows의 Microsoft PowerPoint.
- 글꼴: 모든 텍스트, 표, 노트, master/layout/theme의 영문, 동아시아와 복합 문자 글꼴을 `Calibri`로 통일한다. 제목의 Calibri Light와 Mac 전용 글꼴도 남기지 않는다.
- 글자 크기: PowerPoint pt 단위의 정수만 사용한다. CSS pixel 정수 여부와 구별하고, 자동 축소로 소수점 크기를 만들지 않는다.
- 언어: 문서와 전체 텍스트의 언어를 한국어 `ko-KR`로 설정한다. 새 텍스트 입력의 기본 서식과 문단 끝 서식도 같은 언어를 사용한다.
- 본문과 제목의 기본 글자 색은 검정색으로 하고 다른 색은 강조에만 사용한다.
- 도형의 글자는 도형 자체에 넣는다. 참조 자료의 공통 서식을 적용하며 화면 지칭보다 주요 QA와 DP의 문제를 강조한다.

v5는 61장의 문구, QA 정의와 수치, 도형 배치, 노트 내용과 페이지 순서를 v4와 같게 유지한다. 이미지, GIF와 관계 파일은 bytes까지 보존한다. 글꼴 변경 후 두 줄이 된 9, 10, 41, 42장의 QA ID 셀 12개만 19pt에서 17pt로 조정해 한 줄 표시를 유지했다. 문서 기본 텍스트 스타일의 기본 수준과 1~9 수준, 노트와 기존 run에도 한국어를 명시했다. 이것은 PPTX 문서의 언어 설정이며 PowerPoint 프로그램의 메뉴 언어를 바꾸는 설정은 아니다. 구현은 Microsoft의 [기본 텍스트 스타일 설명](https://learn.microsoft.com/en-us/office/open-xml/presentation/working-with-presentations)과 [문자 속성의 언어 정의](https://learn.microsoft.com/en-us/dotnet/api/documentformat.openxml.drawing.textcharacterpropertiestype.language?view=openxml-3.0.1)를 따른다.

[v5 작성 스크립트](../../../scripts/presentations/update_presentation_windows.mjs)는 Artifact Tool에서 Calibri 글꼴을 적용하고, [Windows 서식과 한국어 정책 도구](../../../scripts/presentations/presentation_windows_policy.py)가 기존 native package의 글꼴 및 언어만 정규화한다. bundled `node_modules`를 private build에 연결하고 `RUNTIME_NODE_MODULES`, `VIA_PRESENTATION_SKILL_DIR`, `VIA_RUNTIME_PYTHON`을 해당 bundled 경로로 설정한 뒤, 작성 스크립트의 사본에 저장소와 build 절대 경로를 전달한다. 스크립트는 v4를 입력으로 새 v5를 생성하는 재현 이력이다.

후속 새 PPTX도 다음 검사를 거쳐 글꼴, 정수 pt와 한국어 기본값을 확인하고 전체 변경 페이지를 렌더링한다.

```text
python3 scripts/presentations/presentation_windows_policy.py check NEW.pptx
```

v5의 package, native 표, geometry, 글꼴 및 언어 검사와 61장 렌더링을 확인했다. Windows의 PowerPoint에서 직접 열어 확인한 것은 아니다. 기존 이미지에 포함된 글자는 원본 픽셀을 보존하며 편집 가능한 PPTX 글꼴 설정의 대상과 구별한다.

### v4의 설계 질문과 글자 크기

본문 15, 17, 19, 21장의 질문은 각각 요청의 대상과 조건 확정, 겹친 사건의 유효한 응답 전달, 과거 근거 연결과 재사용, 수명이 다른 대화/업무 상태의 일관성을 묻는다. 모델/코드, 중앙/단계, 요청별/공통 생산, 공동 확정/독립 접수의 양자택일 문구는 질문에서 제외했다. 12장의 설계 도입 표와 appendix 58~61장의 질문도 같은 문제 중심으로 맞췄다. 본문 배경의 하단 보충 문장은 문제의 어려움을 설명하며, 이전 방안별 비용/조건 문장은 해당 페이지 발표자 노트에 보존했다.

글자 크기는 PowerPoint의 pt 단위로 가장 가까운 정수에 반올림한다(0.5는 올림). 전체 61장의 native 텍스트/표, 노트와 공통 master/layout/theme의 명시된 글자 크기 및 절대 bullet 크기를 포함한다. 예를 들어 9.75pt는 10pt, 14.25pt는 14pt다. 슬라이드 순서와 key, 그림 배치, QA 내용과 숫자, 이미지 및 GIF는 유지한다.

### v3에서 구분한 네 배경

| 본문 위치 | DP | 첫눈에 보이는 주제와 표현 |
| --- | --- | --- |
| 15 | DP41 | 이번 지시의 대상과 적용 범위: 표현, 확인할 근거, 완성할 요청 의미를 대응하는 표 |
| 17 | DP44 | 겹쳐 도착하는 사건과 응답 시점: 사용자 발화, 음성 중단, 업무 질문, 늦은 응답 후보의 시간 흐름 |
| 19 | DP45 | 흩어진 과거 근거의 연결과 재사용: 서로 다른 원본의 기록에서 이번 요청의 Context로 이어지는 근거 |
| 21 | DP42 | 서로 다른 요청과 실행을 잇는 업무 상태: 여러 Request와 Execution을 같은 Task에 연결하는 관계 |

각 배경의 마지막 질문은 바로 다음 A/B 비교로 이어진다. 새 도식은 양안이 해결해야 할 공통 문제를 설명하며 특정 방안을 선택한 구조가 아니다. v3에서 v2 배경 네 장을 appendix 마지막 58~61장에 슬라이드, 노트와 자원 그대로 옮겼다. v4에서도 그림과 시나리오 및 위치를 유지하며 질문과 글자 크기만 정리한다. 비교/QA 페이지의 내용도 유지한다.

도메인 용어는 소개에서 설명하고, 필요성에 모델 사용 비용과 PC 메모리의 원인을 추가했다. 요구사항에서 고정 QA를 거쳐 네 구조 질문으로 이어지는 연결과 DP 배경의 선택 질문을 보완했다. DP 제목은 `(주요 품질속성)을 위한 (기능) 설계` 형식을 따른다. 그림은 실제 판단, 생산, 상태 소유와 확정 방식의 차이를 보여주며 강한 A와 B의 비용을 함께 남긴다.

여섯 QA의 정의, 목표, 모집단, 보호 조건과 등급은 변경하지 않았다. 비교 표의 `평가 전`, 메모리 예산과 지역 가정 계산을 구별한다. A/B 선정, 구현 또는 실측 결과를 추가하지 않았다. 43과 46은 본문에 포함하지 않았다. 전체 크기는 기존과 같은 16:9, 20×11.25in이다. 신규 텍스트, 도형과 표는 PowerPoint에서 편집할 수 있다. [v2-validation.md](./v2-validation.md)는 v2 발행 당시의 검증 이력이다.

## 앞으로 수정하는 방법

항상 **최신 버전의 사본에서 시작해 새 버전으로 저장**한다. 다음 수정은 `_v17.pptx`를 입력으로 `_v18.pptx`를 만들고, 기존 버전은 유지한다. 최초 통합본이나 일곱 개별 파일을 수정한 뒤 다시 전체 병합하면 후속 편집이 사라질 수 있으므로 사용하지 않는다. 이후 버전에서도 Calibri, 정수 pt와 한국어, 검정색 본문 및 도형 자체의 텍스트 기준을 적용한다.

요청에는 “최신본의 DP44 비교와 QA만 수정”, “최신본의 19장 설계 Overview 수정”처럼 대상과 변경 내용을 지정하면 된다. 비대상 슬라이드, 노트와 자원은 보존한다. QA 개정은 별도 명시적 요청이 있을 때만 한다. 발표 문구와 배치 편집으로 Architecture 계약을 자동 개정하지 않는다. 구조의 근거는 [활성 Architecture 문서](../../architecture/README.md)이며, 원본 draw.io와 SVG의 수정은 해당 문서 작업으로 함께 관리한다.

### 자동화로 특정 장만 교체

[manifest-v17.json](./manifest-v17.json)에 현재 위치, Section, 의미별 slide key, 원본 대응과 hash, 서식 정책 및 v17의 변경 범위와 보존 검증을 남겼다. key는 각 슬라이드의 `p:cSld/@name`에도 저장되어 페이지가 이동해도 대상을 식별한다. 새 버전에서는 비대상 key를 유지하고 새 위치와 hash를 기록한다. [manifest-v6.json](./manifest-v6.json)은 서식, 도형 텍스트와 마스터 등록 수정 내역을 기록하며 [manifest-v5.json](./manifest-v5.json), [manifest-v4.json](./manifest-v4.json), [manifest-v3.json](./manifest-v3.json), [manifest-v2.json](./manifest-v2.json)과 [manifest.json](./manifest.json)은 이전 버전의 이력이다.

[부분 교체 도구](../../../scripts/presentations/full_presentation_package.py)는 현재 버전, 수정한 PPTX와 명시한 key만 받아 새 candidate를 만든다. 예를 들어 수정 PPTX의 1장으로 현재 DP44 비교를 교체하려면 매핑 JSON을 다음과 같이 만든다.

```json
{"dp44-comparison-v2": 1}
```

```text
python3 scripts/presentations/full_presentation_package.py replace CURRENT.pptx EDITED.pptx MAPPING.json CURRENT_SHA256 NEW_CANDIDATE.pptx REPORT.json
```

`CURRENT_SHA256`는 편집 시작 시 최신 파일의 SHA-256이다. 도구는 hash가 바뀌거나 candidate가 이미 있으면 중단한다. 선택한 장과 필요한 자원만 가져오고 비선택 슬라이드/노트 및 기존 media의 bytes 보존을 검사한다. 렌더링과 package/layout/font 검증 후 **새 버전 파일**로 발행한다.

본문의 주요 key는 `overview-v2`, `via-common-request-background-v8`, `dp44-comparison-v2`, `dp45-basic-v8`, `dp45-comparison-v2`, `dp42-comparison-v2`, `screen-grounding-background-v8`, `quality-01-03-v2`, `quality-04-06-v2`다. v11의 새 QA 상세 key는 `qa01-metric-v11`~`qa09-metric-v11`이다. 예전 Appendix의 `dp41-comparison` 등은 v11에서 삭제했다.

새 배경 key는 `dp41-background-v3`, `dp44-background-v3`, `dp45-background-v3`, `dp42-background-v3`다. 기존 `*-background-v2`는 appendix 58~61장으로 이동했다. [v3 배경 작성 스크립트](../../../scripts/presentations/update_dp_backgrounds.mjs)와 [배경 보존 패키저](../../../scripts/presentations/package_background_revision.py)는 v2→v3 변경을 재현하며 v2의 57장과 기존 자원의 보존을 검사한다.

v4도 기존 key를 유지한다. [질문과 서식 수정 스크립트](../../../scripts/presentations/update_presentation_typography.mjs)와 [정수 글자 크기 패키저](../../../scripts/presentations/package_typography_revision.py)는 v3의 기존 native 도형과 자원을 유지하며 질문 문구와 pt 크기만 변경한다. 이후 새 텍스트를 작성할 때도 CSS pixel의 정수 여부와 구별해 정수 `fontSizePt`를 지정한다.

[v2 구성 스크립트](../../../scripts/presentations/rebuild_presentation_v2.mjs)와 [원본 보존 패키저](../../../scripts/presentations/package_presentation_v2.py)는 32→57장 이관의 재현용이다. 최초 통합본의 hash를 확인하고 private build에 candidate를 만든다. 이후 부분 수정에는 최신본과 부분 교체 도구를 사용한다. [최초 통합 스크립트](../../../scripts/presentations/assemble_full_presentation.mjs)도 초기 이관 이력으로만 유지한다.

### v17 재현

[integrate_reference_comparisons_v17.mjs](../../../scripts/presentations/integrate_reference_comparisons_v17.mjs)가 원본 세 파일을 Artifact Tool로 열어 페이지 번호를 편집하고, [package_reference_comparisons_v17.py](../../../scripts/presentations/package_reference_comparisons_v17.py)가 원본 객체와 자원을 보존해 v16에 삽입한다. 새 private build에서 bundled Node modules를 연결하고 `VIA_PRESENTATION_SKILL_DIR`, `VIA_RUNTIME_PYTHON`, `RUNTIME_NODE_MODULES`를 설정한 뒤 mjs 사본에 저장소와 build의 절대 경로를 전달한다. 결과는 build의 `final/VIA-architecture-presentation_v17.pptx`이며 49장 전체 렌더와 manifest 및 검증 기록도 생성한다.
