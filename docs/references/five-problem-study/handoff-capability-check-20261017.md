# 업무 인계의 원본 읽기와 native session 지원 추가 확인

2026-10-17 / 공식 문서와 고정 코드 읽기 / 제품 실행과 성능 측정 없음

[기존 근거 R06~R11](./evidence-and-handoff.md#3-agent가-바뀌는-업무의-문맥-전달)을 보완한다. 조사 목적은 [문제 4](../../architecture/12-decisions/decision-packages/qa-first-exploration/five-problem-study/04-agent-independent-handoff.md)의 기본 C와 보완 A2가 실제로 가능한 구조인지, 어떤 지원 조건이 필요한지 확인하는 것이다. 상용 음성 앱 전체의 기능 지원표가 아니다.

## 1. 고정한 확인 범위

| 공식 저장소 | 확인 commit | 범위 |
| --- | --- | --- |
| `openclaw/openclaw` | `e7a155a8ae860d71b1ab5eed45fa4af338011db9` | ACP session 소유 검사, runtime 복원과 실패 처리, acpx adapter, 의존 버전, ACP bridge 설명 |
| `openclaw/acpx` | `1f1cc452fd3c6e9dfb2b32ce12cdf728d022aa75` | 0.19.4의 실제 resume/load 선택, protocol 호출, 파일 읽기 |
| `langchain-ai/deepagents` | `7764f1f8d21a594f0cbd31d273a7cad36597d6ba` | child에 같은 backend 전달, 범위 읽기와 권한 검사, StoreBackend |
| `agentclientprotocol/agent-client-protocol` | `c8775197952214bf3d2cabdd83c66c3de358af57` | v1 파일 읽기 / session setup과 v2 migration 문서 |

검색 HEAD는 경로 발견에만 사용했다. acpx는 OpenClaw의 [의존 버전 0.19.4](https://github.com/openclaw/openclaw/blob/e7a155a8ae860d71b1ab5eed45fa4af338011db9/extensions/acpx/package.json#L10)와 일치하는 [release 준비 commit의 package.json](https://github.com/openclaw/acpx/blob/1f1cc452fd3c6e9dfb2b32ce12cdf728d022aa75/package.json#L3)을 다시 읽었다. 배포 npm 파일과 해당 commit의 바이너리 동일성은 검증하지 않았다. 검색 중 읽은 acpx 0.19.6 HEAD의 동작을 0.19.4의 증거로 대신하지 않았다.

## 2. 수신자가 원본을 골라 읽는 경로

### HC01. Deep Agents의 child에도 실제 파일 읽기 경로가 연결된다

**확인 사실:** [graph.py의 선언형 child 구성](https://github.com/langchain-ai/deepagents/blob/7764f1f8d21a594f0cbd31d273a7cad36597d6ba/libs/deepagents/deepagents/graph.py#L722)은 child의 FilesystemMiddleware에 같은 `backend`를 전달한다. [기본 general-purpose child](https://github.com/langchain-ai/deepagents/blob/7764f1f8d21a594f0cbd31d273a7cad36597d6ba/libs/deepagents/deepagents/graph.py#L827)에도 연결한다. [read_file](https://github.com/langchain-ai/deepagents/blob/7764f1f8d21a594f0cbd31d273a7cad36597d6ba/libs/deepagents/deepagents/middleware/filesystem.py#L2241)은 path 검사와 읽기 권한 검사 후 backend에 offset/limit를 전달한다. [StoreBackend](https://github.com/langchain-ai/deepagents/blob/7764f1f8d21a594f0cbd31d273a7cad36597d6ba/libs/deepagents/deepagents/backends/store.py#L90)는 호출자가 정한 namespace와 BaseStore를 사용해 thread를 넘는 자료 접근을 제공한다.

**VIA 해석:** 같은 허용 저장 범위를 구성한 child에게 자료 경로를 전달하고 필요한 세부를 읽게 하는 구조는 실재한다. 같은 backend 객체를 전달했다고 모든 namespace가 같아지는 것은 아니므로 namespace와 권한도 맞아야 한다. 이종 원격 Agent에 연결할 VIA 읽기 API, source revision과 유효기간 검사는 VIA의 추가 설계다.

### HC02. ACP v1은 선택적 읽기 기능이며 acpx에는 실행 코드가 있다

**확인 사실:** [ACP v1 파일 읽기 명세](https://github.com/agentclientprotocol/agent-client-protocol/blob/c8775197952214bf3d2cabdd83c66c3de358af57/docs/protocol/v1/file-system.mdx#L8)는 Client가 `fs.readTextFile` 지원을 알렸을 때만 Agent가 해당 메서드를 호출하도록 한다. 요청은 session ID, 절대 경로, 시작 줄과 최대 줄 수를 받는다. acpx 0.19.4의 [handler 등록](https://github.com/openclaw/acpx/blob/1f1cc452fd3c6e9dfb2b32ce12cdf728d022aa75/src/acp/client.ts#L1032)은 capability가 있을 때 읽기 요청을 연결한다. [실제 읽기](https://github.com/openclaw/acpx/blob/1f1cc452fd3c6e9dfb2b32ce12cdf728d022aa75/src/filesystem.ts#L91)는 경로, 권한과 호출 권위를 검사하고 반환 전 권위를 다시 확인한다.

해당 읽기 구현은 `workspace.readText`로 파일 내용을 얻은 뒤 줄 범위를 자른다. **작은 범위 응답만으로 작은 I/O나 낮은 최대 RAM을 입증하지 않는다.** 또한 session의 읽기 권한 검사는 VIA의 특정 결과 revision과 철회 계약을 자동 구현하지 않는다.

**VIA 해석:** C는 읽기를 지원하는 adapter에서 구현 가능한 기본안이다. Client가 제공하는 메서드, 수신 Agent의 도구 사용 능력, 허용 namespace와 파일 형식을 각각 확인해야 한다. 단순 URL이나 session ID를 전달하는 것만으로 C가 성립하지 않는다.

### HC03. 같은 ACP라는 이름으로 읽기 지원을 일반화할 수 없다

**확인 사실:** OpenClaw가 외부 harness를 띄우는 ACP Agents 경로와, IDE가 OpenClaw에 접속하는 `openclaw acp` bridge 경로는 다르다. [bridge 설명과 호환 표](https://github.com/openclaw/openclaw/blob/e7a155a8ae860d71b1ab5eed45fa4af338011db9/docs/cli/acp.md#L13)는 후자의 client filesystem 호출과 session별 MCP 설정을 미지원으로 명시한다. 반면 [외부 harness용 acpx adapter](https://github.com/openclaw/openclaw/blob/e7a155a8ae860d71b1ab5eed45fa4af338011db9/extensions/acpx/src/runtime.ts#L450)는 구성한 MCP 서버를 전달하는 별도 경로를 갖는다.

[ACP v2 migration](https://github.com/agentclientprotocol/agent-client-protocol/blob/c8775197952214bf3d2cabdd83c66c3de358af57/docs/protocol/v2/migration.mdx#L781)은 v1 client filesystem 메서드를 제거하고 MCP로 client 도구를 제공하는 방향을 명시한다. [같은 문서의 상태 설명](https://github.com/agentclientprotocol/agent-client-protocol/blob/c8775197952214bf3d2cabdd83c66c3de358af57/docs/protocol/v2/migration.mdx#L18)은 v2 전체를 아직 draft로 표시한다. 배포된 Agent가 v2를 지원한다고 추정하지 않는다.

**VIA 추가 설계:** `자료 읽기 지원`은 제품명 하나의 boolean으로 관리하기 어렵다. adapter와 협상한 protocol version, 전달 형식, 허용 자료 범위 및 실패 응답 계약을 기록해야 한다. 기본 C를 특정 ACP v1 메서드에 고정할 이유는 없으며 범위 제한 MCP 도구도 구현 경로가 될 수 있다.

## 3. 같은 Agent의 native session을 재개하는 경로

### HC04. session ID 전달을 넘어 실제 capability 검사와 protocol 호출을 확인했다

**확인 사실:** OpenClaw의 [resume 소유 검사](https://github.com/openclaw/openclaw/blob/e7a155a8ae860d71b1ab5eed45fa4af338011db9/src/acp/runtime/session-meta-resume-authorization.ts#L71)는 요청자 자신, 요청자가 생성한 session 또는 자식 연결인지 검사한다. [후보 조회](https://github.com/openclaw/openclaw/blob/e7a155a8ae860d71b1ab5eed45fa4af338011db9/src/acp/runtime/session-meta-resume.ts#L10)는 backend, harness와 현재 저장 기록을 대조한다. [adapter 초기화](https://github.com/openclaw/openclaw/blob/e7a155a8ae860d71b1ab5eed45fa4af338011db9/extensions/acpx/src/runtime.ts#L974)는 복원 인자를 실제 acpx runtime으로 전달한다.

acpx 0.19.4의 [createOrLoadRuntimeSession](https://github.com/openclaw/acpx/blob/1f1cc452fd3c6e9dfb2b32ce12cdf728d022aa75/src/runtime/engine/manager.ts#L272)은 resume capability를 우선 사용하고, 없으면 load capability를 확인하며, 둘 다 없으면 오류를 낸다. [client의 capability 검사](https://github.com/openclaw/acpx/blob/1f1cc452fd3c6e9dfb2b32ce12cdf728d022aa75/src/acp/client.ts#L503)와 [load/resume 실제 요청](https://github.com/openclaw/acpx/blob/1f1cc452fd3c6e9dfb2b32ce12cdf728d022aa75/src/acp/client.ts#L1230)도 확인했다. load에는 session ID, cwd와 MCP 서버가 전달된다. [v1 명세](https://github.com/agentclientprotocol/agent-client-protocol/blob/c8775197952214bf3d2cabdd83c66c3de358af57/docs/protocol/v1/session-setup.mdx#L83)도 지원 확인 후 기존 session을 불러오는 계약을 정의한다.

**한계:** 다른 Agent가 그 ID를 읽을 수 있다는 근거가 아니다. 대화 복원과 실행 중인 domain plan / 도구 상태 이동은 다르다. 특정 Claude, Codex, Gemini 배포에서의 응답, 조건 반영 정확성이나 재개 비용은 이번에 실행하지 않았다.

### HC05. 재개 요청의 성공과 문맥의 연속성을 구별해야 한다

**확인 사실:** OpenClaw [runtime handle 복원](https://github.com/openclaw/openclaw/blob/e7a155a8ae860d71b1ab5eed45fa4af338011db9/src/acp/control-plane/manager.runtime-handle-ensure.ts#L228)은 저장된 ID의 초기화가 특정 오류로 실패하면 그 ID 없이 다시 초기화한다. 이때 낡은 session 식별자를 새 handle에 다시 합치지 않는다. 모든 실패가 이 경로로 처리되는 것은 아니다.

**VIA 추가 설계:** A2는 실제 동일 session 재개와 자료의 유효성을 확인한 경우에만 적용한다. 새 session 생성, 재개 미지원, 확인 불가이면 A의 portable 묶음으로 돌아간다. 필수 조건과 허가 범위를 매 인계 revision에 명시하고, native session에 남았다고 추정해 생략하지 않는다. 외부 수행 여부가 불명이면 묶음을 복원할 수 있어도 동일 Action을 새 Agent에서 시작하지 않는다.

## 4. 이번 설계 권고에 주는 결론

| 판단 | 근거와 적용 한계 |
| --- | --- |
| 문제 4 기본 A를 초기 공통 구조로 권고 | 공개 원본과 현재 조건에서 위임 자료를 구성하므로 모든 Agent에 지속 memory / 공유 읽기 / native resume를 강제하지 않는다. 각 Agent의 실제 입력 형식과 예산은 여전히 맞춰야 함 |
| 기본 B는 반복 인계가 많은 경우의 대안 | Agent 교체 기능 자체가 매 사건의 의미 기록 생산을 요구하지 않음. 반복 사용으로 생산 / 갱신 비용을 회수할 수 있을 때 선택 근거가 생김 |
| 기본 C는 실제로 구현 가능한 대안 | Deep Agents의 backend와 ACP/acpx의 읽기 경로가 근거. 지원 adapter와 큰 자료에서의 세부 선택 이익을 따져 적용. 전체 Agent 공통 기본으로 권고할 근거는 부족 |
| A2는 같은 Agent의 조건부 보완 | 지원 확인과 실제 재개는 구현 근거가 있음. 필수 조건 검증, session 유실 fallback과 이종 Agent 인계는 VIA가 유지 |

위 마지막 표는 **작성자 권고**다. 외부 source가 VIA의 승자나 성능 우위를 입증한 것은 아니다. 실제 모델 호출, Agent 작업 실행, API 호환성 실험, 성능 측정과 숨겨진 추론 수집을 수행하지 않았다.
