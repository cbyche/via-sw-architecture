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
