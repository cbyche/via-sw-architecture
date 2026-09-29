# Target Architecture diagrams

이 디렉터리는 [VIA 목표 Architecture](../architecture.md)의 검토용 그림을 관리한다.

각 그림은 두 파일을 한 쌍으로 둔다.

- `.drawio`: diagrams.net 또는 draw.io Desktop에서 여는 편집 원본
- `.svg`: GitHub Markdown과 발표자료에서 바로 확인하는 vector preview

현재 baseline은 아래 명령으로 같은 구조 정의에서 두 형식을 생성한다. 모든 그림의 선 교차·박스 관통도 함께 검사한다.

```bash
.venv/bin/python scripts/architecture/generate_target_architecture_diagrams.py
.venv/bin/python scripts/architecture/generate_target_architecture_diagrams.py --check
```

첫 명령은 생성, `--check`는 읽기 전용 검사다. CI도 source와 `.drawio`·`.svg` 쌍의 일치 및 경로를 검사한다. `.drawio`를 직접 편집했다면 source 정의에도 변경을 반영하고 두 형식을 함께 재생성한다. Source를 갱신하지 않은 수동 편집은 다음 생성에서 덮어써지며 CI의 pair 검사에도 실패한다.

색상은 그림 사이에서 같은 책임을 뜻한다.

| 색상 | 의미 |
| --- | --- |
| 파랑 | 사용자 interaction, Voice/Text 출력과 latency-sensitive 경로 |
| 보라 | semantic proposal, host validation과 authority |
| 청록 | Context source, evidence와 bounded read |
| 초록 | Task projection, commit과 verified recovery |
| 주황 | Agent command, connector와 외부 실행 |
| 노랑 | policy, admission, fence와 사용자 결정 |
| 회색 | shared dependency, durable infrastructure와 관측 상태 |
| 빨강 | 출력 중단, failure, uncertainty 또는 crash window |

박스와 화살표에 같은 색 체계를 사용한다. 화살표 색은 그 선이 전달하는 주된 책임·계약 영역이며 수신 Component의 상태 소유권을 자동으로 뜻하지 않는다. `→`는 호출·데이터·제어의 주 방향이고 `↔`는 request/response, command/event, stream/control처럼 양쪽 메시지가 있는 protocol이다. 양방향 표시는 공동 상태 소유를 뜻하지 않는다.

## 전체 구조 그림의 읽기 규칙

`01-system-overview`는 VIA 내부 Component와 외부 책임을 구분하는 논리 구조다. Interaction Manager도 다른 Component와 같은 수준의 한 박스로 두며 내부 모듈은 본문 §4에 설명한다. State Store·Model Access는 VIA 내부 공유 서비스다. 모델이 외부 책임 영역에 있다는 것은 원격 실행을 뜻하지 않는다.

점선 `S`·`M1~M3` 박스는 위에 표시한 Component의 공통 서비스 접근 port를 다시 표기한 것이다. 추가 Component·모델 복제본·메시지 bus가 아니다. 실선은 요청·결과·출력 계약, 점선 연결은 공통 서비스 접근 관계다. 번호는 연결 계약 ID이며 고정 실행 순서가 아니다. 선 두께에는 별도 의미가 없다.

전체 그림의 Agent command와 event, 응답 명령과 receipt는 서로 다른 선으로 표시한다. 생성기는 모든 그림의 직교 경로가 Component 박스 내부를 통과하거나 서로 교차·접촉하면 실패한다. draw.io에도 같은 endpoint와 waypoint를 저장해 자동 routing에 따른 경로 변경을 줄인다. 이 검사는 글자·label·화살촉의 간섭을 판정하지 않으므로 SVG 렌더링을 별도로 육안 확인해야 한다.

## 그림 목록과 관점

그림마다 박스가 무엇을 나타내는지 구분한다. 01은 논리 Component, 02는 데이터 entity와 관계, 03~05·08~10은 runtime 단계·근거·내구 경계, 06은 process와 의존성 연결 port, 07은 품질 인과 경로다. 서로 다른 관점을 모두 Component 배치도로 읽지 않는다. 06의 점선 port는 process가 아니다.

| 그림 | 확인할 내용 | Preview / 편집 원본 |
| --- | --- | --- |
| 01 전체 구조 | VIA 내부·외부 책임, 요청과 결과, 공통 서비스 | [SVG](./01-system-overview.svg) · [draw.io](./01-system-overview.drawio) |
| 02 Identity와 수명 | Conversation·Turn·Request·Task·Execution·실제 응답 | [SVG](./02-lifecycle-and-ownership.svg) · [draw.io](./02-lifecycle-and-ownership.drawio) |
| 03 요청 확정 | 최소 근거 해석, bounded refinement, clarification·실패 | [SVG](./03-request-resolution.svg) · [draw.io](./03-request-resolution.drawio) |
| 04 지칭의 시간축 | 당시 근거, 정정, 화면 변경, 수집 공백 | [SVG](./04-interaction-evidence-timeline.svg) · [draw.io](./04-interaction-evidence-timeline.drawio) |
| 05 내구 경계 | command·event·publication과 crash 후 불명 상태 | [SVG](./05-dispatch-and-recovery.svg) · [draw.io](./05-dispatch-and-recovery.drawio) |
| 06 배치·장애 경계 | Voice/Core 분리, 모델 adapter 배치, UI·Store·worker | [SVG](./06-runtime-and-fault-boundaries.svg) · [draw.io](./06-runtime-and-fault-boundaries.drawio) |
| 07 네 품질 경로 | accuracy·responsiveness·modifiability·recoverability | [SVG](./07-four-asr-critical-paths.svg) · [draw.io](./07-four-asr-critical-paths.drawio) |
| 08 응답·중단 | S2S와 Core 응답 게시, actual delivery, local barge-in | [SVG](./08-response-and-interruption.svg) · [draw.io](./08-response-and-interruption.drawio) |
| 09 복합 요청 | 결과 의존, 독립 업무, 조건별 분기와 실행 전 확인 | [SVG](./09-compound-and-task-routing.svg) · [draw.io](./09-compound-and-task-routing.drawio) |
| 10 Context·권한·기억 | 읽기와 제공, 권한 철회, User Memory 변경·삭제 | [SVG](./10-context-policy-and-memory.svg) · [draw.io](./10-context-policy-and-memory.drawio) |

10개 그림은 [Architecture 본문](../architecture.md)의 관련 절에 모두 삽입했다. [설계 완결성 점검](../design-completeness.md)에서 UC별 경로와 미확정 항목을 확인할 수 있다.
