# Target Architecture diagrams

이 디렉터리는 [VIA 목표 Architecture](../architecture.md)의 검토용 그림을 관리한다.

각 그림은 두 파일을 한 쌍으로 둔다.

- `.drawio`: diagrams.net 또는 draw.io Desktop에서 여는 편집 원본
- `.svg`: GitHub Markdown과 발표자료에서 바로 확인하는 vector preview

현재 baseline은 아래 명령으로 같은 구조 정의에서 두 형식을 생성했다.

```bash
.venv/bin/python scripts/architecture/generate_target_architecture_diagrams.py
```

`.drawio`를 직접 편집했다면 같은 이름의 `.svg`를 다시 export하여 같은 commit에 포함한다. 생성 script를 다시 실행하면 현재 script 정의로 두 파일을 덮어쓰므로, 수동 편집을 보존해야 할 때는 먼저 script 정의를 갱신하거나 실행하지 않는다.

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
| 빨강 | failure, uncertainty 또는 crash window |

박스와 화살표에 같은 색 체계를 사용한다. 화살표 색은 그 선이 전달하는 주된 책임·계약 영역이며 수신 Component의 상태 소유권을 자동으로 뜻하지 않는다. `→`는 호출·데이터·제어의 주 방향이고 `↔`는 request/response, command/event, stream/control처럼 양쪽 메시지가 있는 protocol이다. 양방향 표시는 공동 상태 소유를 뜻하지 않는다.

## 전체 구조 그림의 읽기 규칙

`01-system-overview`는 VIA 내부 Component와 외부 책임을 구분하는 논리 구조다. Interaction Manager도 다른 Component와 같은 수준의 한 박스로 두며 내부 모듈은 본문 §4에 설명한다. State Store·Model Access는 VIA 내부 공유 서비스다. 모델이 외부 책임 영역에 있다는 것은 원격 실행을 뜻하지 않는다.

점선 `S`·`M1~M3` 박스는 위에 표시한 Component의 공통 서비스 접근 port를 다시 표기한 것이다. 추가 Component·모델 복제본·메시지 bus가 아니다. 실선은 요청·결과·출력 계약, 점선 연결은 공통 서비스 접근 관계다. 번호는 연결 계약 ID이며 고정 실행 순서가 아니다. 선 두께에는 별도 의미가 없다.

전체 그림의 Agent command와 event, 응답 명령과 receipt는 서로 다른 선으로 표시한다. 생성기는 이 그림의 직교 경로가 Component 박스 내부를 통과하거나 서로 교차·접촉하면 실패한다. draw.io에도 같은 endpoint와 waypoint를 저장해 자동 routing에 따른 경로 변경을 줄인다. 이 검사는 글자·label·화살촉의 간섭을 판정하지 않으므로 SVG 렌더링을 별도로 육안 확인해야 한다.
