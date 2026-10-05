# 04-41 메인 그림 재배치 검수

> 2026-10-05 / 사용자 요청: 내용을 보존하면서 참고 그림처럼 가로 한 장으로 재배치, commit/push
> [설계 본문과 메인 그림](./04-41-request-resolution-control.md#3-공통-조건과-두-구조)

## 1. 범위와 결과

메인 그림을 2560×3340에서 **2560×1440(16:9)**으로 재배치했다. 위에 같은 입력, 가운데 A/B의 해석 구조, 아래에 공통 채택·위임·사용자 전달을 배치하고 마지막에 장점·대가·선택 조건을 대조했다. 공통 경로를 한 번만 펼쳐 세로 중복을 줄였다. A/B로 나뉘는 입력선은 대체 설계의 비교이며 두 해석 경로를 동시에 실행하는 구조가 아니다.

A는 모델이 다음 조회와 최종 의미를 제안하고 코드는 읽기 실행·검증을 담당한다. B는 모델이 틀/부분 해석을 생산하고 Request Resolution Engine이 조회 진행과 최종 관계 결합을 담당한다. 기존 설계, 기능 지원 한계, 상태 소유권과 조건부 선택 근거는 변경하지 않았다. 모델 호출 감소, 품질 우위나 승자를 새로 주장하지 않는다.

수정 범위는 41 메인 그림의 SVG/draw.io, 해당 생성 scene과 진입점, 발표용 PNG, 본문의 그림 안내와 이 검수 기록이다. A/B 사건도 두 쌍과 다른 비교의 본문·그림은 그대로다. 작업 시작 시 이미 변경되어 있던 35/36 및 공통 안내 파일은 이번 커밋 대상에서 제외한다.

## 2. 내용 보존 대조

이전 SVG의 Component/Module/저장소/의존성 이름 18종을 새 scene과 비교했으며 누락은 없다. 기존 설명에 있던 Policy Manager와 State Store는 그림의 공통 경로에 추가로 펼쳤다. 같은 이름이 반복된 요소는 같은 논리 인스턴스이며, Omni는 PC의 가중치 한 벌을 공유한다.

| 기존 내용 | 새 메인 그림의 위치·표현 |
| --- | --- |
| 원문, 시점, 입력 버전, 신규/보완 미결정, 같은 최근 대화·게시 질문 | 상단 Interaction Manager → Request Controller 및 공통 입력 설명. 정답 Task를 미리 주지 않음 |
| 모델이 고르는 제한 조회와 결과 재판단 | A의 ReAct 해석 제어기 → 읽기 도구 실행기 및 결과 반환 루프 |
| 임시 근거·조회 기록과 회수 | A의 임시 해석 상태에 기록하고 제어기로 회수 |
| 의미/질문 제안, 검증 오류 반환 | A의 의미 제안 검증기와 제한 재생성 경로 |
| 구조화 출력, 조건 보존, 부분 수정, cache | A 구조 아래에 명시 |
| 미결정 대상/Task/조건을 담는 모델 틀 | B의 Request Interpreter → 미해결 항목 처리기 |
| 코드 주도 조회, 후보·근거 버전, 관계 결합, 충돌 시 추가 조회/질문 | B의 Request Resolution Engine 내부 모듈과 요청 해석 상태 및 반복 경로 |
| 필요할 때 범위를 지정한 추가 모델 해석 | B의 미해결 항목 처리기 → Request Interpreter. 1회 호출을 전제하지 않음 |
| 표현 범위 밖 관계 | B의 미지원 보류 및 지원 한계 반환 |
| Context/Task 조회와 근거·사실/버전 반환 | 양쪽의 Context Manager/Task Manager 왕복. 권한·자료 계약은 동일 |
| Model Access와 Omni의 입력/결과 | 양쪽에 근접 배치. 모델 의존성과 PC 가중치 공유, 역할별 세션 분리를 명시 |
| 의미/질문 반환과 현재 검사 | 두 결과가 하단의 동일 Request Controller로 반환. 입력·자료·Task·질문 버전과 Policy Manager 검사 |
| 채택 상태 보관 및 B 채택 통지 | State Store 왕복과 B Engine 채택·게시 연결 통지. State Store는 저장 수단이며 의미 소유자를 교체하지 않음 |
| Task 생성/변경, command ID, 외부 인계, 접수/결과 반환 | Request Controller ↔ Task Manager ↔ Agent Gateway ↔ Downstream Agent. 접수를 완료로 바꾸지 않음 |
| 질문/직접 응답/확인 상태, Text/음성 전달 | 하단 Response Manager ↔ Interaction Manager. 응답 추론도 Model Access 경유 |
| 실제 전달·게시 기록과 사용자 답변의 구분 | 전달 기록 반환과 Response Manager → Request Controller의 게시 기록. 사용자 답변은 상단의 6번 새 입력으로 표시 |
| 음성 입력/인식 유지, 논리 책임과 외부 업무 경계 | 범례·Omni 의존성·Downstream Agent 외부 업무 책임 설명. 별도 process를 가정하지 않음 |
| 이익과 비용 | 마지막 A/B의 장점·대가·선택 조건 3행. 측정 수치나 별점 추가 없음 |

## 3. 렌더 검수와 수정

작성자 검수이며 독립 심사나 사용자 승인 기록이 아니다.

- 첫 가로 렌더에서 B의 조회 왕복이 Engine 제목 부근에 몰리는 것을 확인했다. 미해결 항목 처리기를 자료 제공자 가까이 옮겨 직접 조회하고 반환받게 했다.
- 텍스트 경계 검사에서 B의 부분 해석 라벨과 Engine 제목, 결합 결과 라벨과 공통 인계 라벨이 겹친 것을 수정했다. 마지막으로 부분 해석/모델 반환 라벨의 겹침과 범례의 화면 밖 잘림을 수정했다.
- A의 읽기 도구 실행기를 옆 아래에 두어 모델이 선택한 조회와 결과 재판단 경로를 구분했다. 유효 기록 회수 라벨을 저장소 밖으로 이동했다.
- 로컬 Chrome에서 2560×1440과 발표 표시 크기인 1920×1080을 렌더링하여 확인했다. 마지막 렌더의 SVG 텍스트 bounding box 대조에서 겹침과 canvas 밖 텍스트는 없었다. 자동 대조는 의미 검증을 대신하지 않으며, 루프·채택·인계·반환 경로는 그림과 본문을 수작업으로 대조했다.
- SVG와 draw.io는 같은 scene으로 생성하며 XML, ID/연결 참조, Module 소유·포함, canvas 범위와 직교 경로/무관한 노드 관통 검사를 수행한다. PNG는 최종 SVG의 브라우저 렌더다.

## 4. 재생성과 검증

```bash
.venv/bin/python scripts/architecture/generate_request_resolution_diagrams.py
.venv/bin/python scripts/architecture/generate_request_resolution_diagrams.py --check
.venv/bin/python scripts/architecture/check_active_markdown_links.py
.venv/bin/python scripts/architecture/check_active_terminology.py
.venv/bin/python scripts/architecture/check_qa_catalog.py
```

실행 결과: 41의 세 SVG/draw.io 쌍 검사, active Markdown 링크, 용어·Component 이름, QA catalog 검사와 `git diff --check`가 모두 통과했다. 기존 A/B 사건도 네 파일의 diff는 없다.

메인 scene은 `scripts/architecture/request_resolution_presentation.py`다. 기존 진입점은 메인과 변경하지 않은 A/B 사건도를 함께 검사한다. PNG를 다시 만들 때는 최종 SVG를 브라우저에서 2560×1440으로 렌더링한다. 커밋·게시 결과는 Git 이력과 사용자 보고로 연결한다.

문서·도식 검사는 구현, 모델 capability, 사용자 이해도나 성능의 측정이 아니다. Stage 5/6 진행, 최종 DP 선정, 참조 Architecture 또는 QA/ADR 변경을 포함하지 않는다.
