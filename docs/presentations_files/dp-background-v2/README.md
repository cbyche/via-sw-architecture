# DP 배경 발표 v2 — 페이지별 검토 작업

편집 대상은 [VIA-DP-background-and-comparison-41-45_v2.pptx](../VIA-DP-background-and-comparison-41-45_v2.pptx)다. 최초 생성 시 기존 원본과 byte-identical인 별도 복제본으로 시작했다. 기존 원본, 원본 generation scripts와 docs/architecture 도식은 이번 시안 작업에서 변경하지 않는다.

사용자는 한꺼번에 전체를 바꾸는 대신, 도입 → 41 배경 → 44 → 45 → 42를 검토하는 방식을 요청했다. 2026-10-10 후속 리뷰에서 도입을 사용자 경험 두 장으로 나누도록 제안하고 그림 제작을 요청했다. 이번 작업은 이 두 장의 검토용 초안을 v2에 반영한다. 43/설계 비교의 내용·배치 변경은 별도 지시 없이 하지 않는다. 작업 단위별로 commit/push하여 검토·되돌리기 지점을 남긴다. 파일 게시를 시각 디자인 승인으로 해석하지 않는다.

## 사용자 경험 1 — VIA의 책임

헤드라인은 **“사용자의 요청을 이해해 업무를 위임하고, 입력부터 응답까지 사용자와의 대화를 관리하는 VIA”**다. 결과 전달만 강조하지 않고, 입력 수신과 요청 이해·위임, 업무 중 질문·답변, 추가 조건, 결과 전달의 왕복 관계를 보여준다.

- 요청: “이 표와 지난달 매출보고서를 바탕으로 이번 달 매출보고서를 만들어줘.”
- Agent 질문: “강조할 항목은 무엇인가요?” 사용자 답변: “국내 매출을 강조해줘.”
- VIA는 목표·자료·조건을 이해해 위임하고, 질문과 답변을 해당 업무에 연결한다. 완성된 보고서는 VIA를 통해 화면과 음성으로 사용자에게 돌아온다. 실제 보고서 작성과 도메인 분석은 Downstream Agent 책임이다.
- 사용자 혜택: VIA에서 대화를 이어가며 업무를 맡기고 확인한다.
- [슬라이드 미리보기](./via-user-experience-1.png), [삽화](./assets/via-responsibilities-illustration.png).

## 사용자 경험 2 — 계속되는 입력과 업무 결과

헤드라인은 **“업무를 맡긴 뒤에도, 새 발화와 업무 결과가 같은 대화로 들어옵니다”**다.

- 복합 요청: “이 표와 지난달 보고서로 이번 달 매출보고서를 만들고, 내일 오후 3시 팀 회의 안내 메일 초안도 준비해줘.”
- 보고서와 회의 안내 메일은 독립적인 두 업무다. 보고서 결과에 의존하는 메일이나 메일 발송 요청으로 그리지 않는다.
- 업무 처리 중 정정: “잠깐, 보고서엔 국내 매출만 넣어줘.”
- 같은 시기에 이전에 맡긴 공급업체 견적 결과가 들어온다. 정정은 보고서에 연결하고, 견적은 관련 업무 결과로 유지한다. 사용자가 말을 마친 뒤 견적 도착을 안내한다.
- 새 입력과 비동기 업무 결과가 VIA로 모이는 순간을 두 방향 화살표로 표현한다. 접수와 실제 Agent 적용, 결과 도착과 사용자 전달을 구별한다. 이 장면으로 구조·스케줄링 우선순위·병렬 실행 순서·성능 우위를 선택하지 않는다.
- 사용자 혜택: Agent별 창을 오가거나 진행 중인 업무를 다시 설명할 필요가 없다.
- [슬라이드 미리보기](./via-user-experience-2.png), [삽화](./assets/via-overlapping-events-illustration.png).

## 표현과 보존 범위

- 삽화는 builtin imagegen으로 만든 설명용 이미지이며 실제 제품 화면이 아니다. [두 이미지의 생성 프롬프트](./assets/illustration-prompts.json)를 보관한다. 제목·문구·참여자·메시지 경로는 native editable PPTX objects다.
- 화면·표 선택·포인터가 입력 근거다. 손을 든 동작은 설명용 끼어들기 표현이며 카메라 인식을 전제하지 않는다.
- 이 페이지의 사용자/VIA/Agent는 사용자 경험의 참여자다. 세로 점선은 참여자의 역할이 이어짐을 뜻하며 Component·Module·process 경계가 아니다. 여러 업무를 서로 다른 Agent가 반드시 수행한다고 가정하지 않는다.
- 도입에서는 ID/version·세부 Component·모델 배치·위험 목록·QA 계약·DP 번호를 펼치지 않는다. 다음 배경/비교 페이지에서 필요한 내용을 설명한다.
- v2는 기존 도입 한 장을 위 두 장으로 바꾼 **12장**이다. 본 발표 10장과 43 부록 2장이다. 기존 DP 배경/비교 10장의 slide·notes·관련 기존 package parts는 그대로 보존한다. 원본 PPTX와 Architecture 문서·SVG/draw.io/PNG는 수정하지 않는다.
- 초기 [구도 시안](./assets/intro-composition-study_v2.png)은 이전 검토용으로 보존한다. 그 시안의 약한 헤드라인과 목적이 불명확한 메일 예시는 현재 도입에서 사용하지 않는다.

## 재생성

전용 [native 슬라이드 생성기](../../../scripts/presentations/generate_dp_v2_intro.mjs)와 [범위 제한 package 조립기](../../../scripts/presentations/dp_v2_intro_package.py)를 쓴다. 기존 원본 generation scripts는 변경하지 않았다. 현재 v2에서 도입 두 장만 갱신하므로 앞으로 다른 배경 페이지를 승인받아 수정해도 이 생성기가 덮어쓰지 않는다.

생성기를 private build directory에 복사하고 bundled node_modules를 연결한다. Presentations skill의 작업 시작 marker를 한 번 실행한 후, `VIA_PRESENTATION_SKILL_DIR`, bundled `VIA_RUNTIME_PYTHON`, `RUNTIME_NODE_MODULES`를 설정해 bundled Node로 `generate_dp_v2_intro.mjs REPO BUILD`를 실행한다. build마다 새 디렉터리를 사용한다. 최종 파일을 재import한 `intro-1.png`, `intro-2.png`로 시각 검토하고 이 디렉터리의 미리보기를 갱신한다.

이번 결과는 사용자 피드백을 기다리는 검토용 초안이다. 실제 제품 구현·성능 측정이나 최종 디자인 승인을 뜻하지 않는다. 다음 작업은 두 도입 페이지 리뷰 후 **41 배경 한 장**이다.
