# 문제 3 조사 근거: 발화 중 화면 지칭

2026-10-16 / 공개 구현과 원 논문 조사 / 설치, API 검증과 모델 실행 없음

[설계 비교](../../architecture/12-decisions/decision-packages/qa-first-exploration/five-problem-study/03-screen-speech-grounding.md) / [공동 조사 기준](../../architecture/12-decisions/decision-packages/qa-first-exploration/five-problem-study/00-research-contract.md)

## 1. 조사 범위와 판정

이번 조사는 현재 요청의 말과 화면 행동을 어떤 자료로 보존하고, 어디서 함께 판단하며, 부분 정정 때 어떤 상태를 다시 생산하는지 확인했다. 기존 [DP-47 구조 조사](../../architecture/12-decisions/decision-packages/dp47-structure-references.md)를 읽은 뒤 아래 코드와 원 논문을 직접 다시 읽었다. 이전 문서의 구현 수준 세부사항을 외부 reference의 구현 사실로 옮기지 않는다.

**확인한 주요 사실은 공동 원본 입력, 별도 후보 생산과 결합, 증분 가설 갱신, 원시 시각에 따른 stream 결합이라는 네 메커니즘이다.** 앞의 두 개는 지칭 의미를 생산하는 구조의 대안이고, 뒤의 두 개는 양쪽에서 활용할 수 있는 실행 및 관측 계약이다. 새로운 독립 C를 만들 만큼 다른 의미 생산 구조는 이번 범위에서 확보하지 못했다.

| 범위 | 이번 확인과 적용 경계 |
| --- | --- |
| Magentic-One, OpenHands, OpenClaw | 고정 commit에서 관측 입력, 이미지와 native 정보, snapshot reference 유효성 경로를 직접 확인했다. 사용자의 발화 중 pointer 경로, 원형 집합과 부분 지칭 정정의 완성 구현으로 소개하지 않는다 |
| Deep Agents, CrewAI | 공동 조사의 [기존 harness 원장](../agent-harness-20261015.md)을 활용했다. 이 두 프로젝트의 기억, 도구와 subagent 메커니즘은 문제 1/4에 직접 관련된다. 문제 3에서 새 temporal grounding 구현 근거를 확보했다고 주장하지 않는다. 저장소 전체의 기능 부재 판정도 아니다 |
| Google Live Console | 음성과 화면 frame을 같은 live client로 전달하는 공개 샘플을 고정했다. temporal binding이나 pointer 의미의 정확성 근거는 아니다 |
| QuickSet, OmniParser | 실제 시스템 원 논문의 부분 의미 결합과 고정 코드의 화면 요소 생산 interface를 각각 확인했다. 서로 다른 시스템을 하나의 완성 구현처럼 합치지 않는다 |
| Psi, InproTKS, ReTiCo | 시간 결합과 증분 가설 수명에 필요한 실제 코드 / 원 논문을 추가 조사했다. 로봇이나 센서의 정확한 시각, 지칭 성공률을 PC UI에 이전하지 않는다 |

## 2. 고정 소스와 확인한 실행

### R3-01. Magentic-One WebSurfer: native 목록과 이미지의 공동 공급

- 저장소 / commit: `microsoft/autogen` / `027ecf0a379bcc1d09956d46d12d44a3ad9cee14`.
- 읽은 코드: [`_multimodal_web_surfer.py` L490–620](https://github.com/microsoft/autogen/blob/027ecf0a379bcc1d09956d46d12d44a3ad9cee14/python/packages/autogen-ext/src/autogen_ext/agents/web_surfer/_multimodal_web_surfer.py#L490).
- 실행: interactive rectangle과 viewport, screenshot을 읽고 Set-of-Mark 이미지를 만든다. focus와 보이는 target 목록을 구성한 뒤 vision 모델에는 이미지와 text를 함께 준다. 모델 반환은 응답 또는 도구 처리로 이어진다.
- 상태 / 비용: 모델 history에 prompt와 사용자 요청이 추가된다. 이미지 scaling과 target 목록 구성 비용이 있고, screenshot 저장은 조건부다.
- VIA 적용: native 정보와 pixels를 공동 판단에 함께 제공하는 A의 실용적 입력 형태다. B도 이 자료로 screen 후보를 생산할 수 있다.
- 제한: 읽은 경로는 현재 page 상태다. 발화 시각의 과거 screenshot, 사용자 pointer 경로, 음성 정정 revision을 보존한 temporal grounding의 증거가 아니다. 브라우저 업무 Action은 VIA 경계로 가져오지 않는다.

### R3-02. OpenHands BrowserObservation: text와 이미지, 저장본의 구별

- 저장소 / commit: `OpenHands/software-agent-sdk` / `fd387b58dc7b707b73c159845c0396a17000be24`.
- 읽은 코드: [`impl.py` L568–596](https://github.com/OpenHands/software-agent-sdk/blob/fd387b58dc7b707b73c159845c0396a17000be24/openhands-tools/openhands/tools/browser_use/impl.py#L568), [`definition.py` L61–135](https://github.com/OpenHands/software-agent-sdk/blob/fd387b58dc7b707b73c159845c0396a17000be24/openhands-tools/openhands/tools/browser_use/definition.py#L61), 같은 파일 L299–330과 L782.
- 실행: `get_state(include_screenshot)`는 browser state JSON을 받고 image를 분리한다. `to_llm_content`는 text와 선택적 image를 함께 제공한다. 출력 디렉터리가 있으면 hash 이름으로 screenshot을 저장하고 경로도 text에 남긴다.
- 강한 보완: 현재 state는 screenshot 없이도 조회 가능하다. 큰 text의 잘림과 전체 저장 경로를 구분한다. 도구 설명은 browser state의 index가 Action 뒤 바뀔 수 있음을 경고한다.
- VIA 적용: 모든 관측마다 image 판단을 강제하지 않고 실제 노출 native 정보와 image를 함께 쓰는 공통 경로. 원본 보존과 모델 입력에 전부 넣는 것은 별개의 선택이다.
- 제한: hash 파일명은 시간 정렬이나 원본 사용권 철회 계약이 아니다. 현재 interactive index를 다음 화면의 stable ID로 재사용할 수 있다는 근거도 아니다. 사용자 발화 수집은 읽은 파일의 책임이 아니다.

### R3-03. OpenClaw snapshot reference: 문서 교체 중 게시 방지

- 저장소 / commit: `openclaw/openclaw` / `e7a155a8ae860d71b1ab5eed45fa4af338011db9`.
- 읽은 코드: [`pw-tools-core.snapshot.ts` L100–193](https://github.com/openclaw/openclaw/blob/e7a155a8ae860d71b1ab5eed45fa4af338011db9/extensions/browser/src/browser/pw-tools-core.snapshot.ts#L100).
- 실행: snapshot ref를 action cache에 게시할 때 frame guard 안에서 선택적 `expectedDocumentIdentity`와 현재 문서를 비교한다. 문서가 바뀌면 retry 오류를 내고, backend DOM 표시 뒤에도 현재성 검사를 한다. DOM binding 소실을 role/name 순서로 다른 대상에 다시 붙이지 않는 계약이 코드에 있다.
- VIA 적용: 당시 관측의 identity와 현재 앱 identity를 구별하는 양안 공통 요구의 근거다. 보관된 과거 대상과 현재 조작 가능한 selector는 다른 값이다.
- 제한: 옵션을 사용하는 이 게시 경로의 사실이다. 모든 OpenClaw 경로의 전역 보장이나 VIA의 사용자 의도 판단 증거로 확대하지 않는다. raw screenshot과 음성의 의미 결합은 구현 범위가 다르다.

### R3-04. Google Live Console: 같은 세션에 별도로 들어오는 media

- 저장소 / commit: `google-gemini/live-api-web-console` / `0a4542fe0e39d07956ea7af5de45d7c81fde8960`.
- 읽은 코드: [`ControlTray.tsx` L100–164](https://github.com/google-gemini/live-api-web-console/blob/0a4542fe0e39d07956ea7af5de45d7c81fde8960/src/components/control-tray/ControlTray.tsx#L100).
- 실행: audio recorder의 data handler가 PCM을 `sendRealtimeInput`으로 전송한다. screen / webcam video는 canvas에서 축소 JPEG로 만들고 같은 client로 전송한다. 연결 / mute / stream 상태와 cleanup이 수집 경로를 제어한다.
- VIA 적용: 모달리티별 최종 후보 계약을 필수로 두지 않고 원음을 포함한 media를 공동 의미 처리에 공급하는 A의 실행 선례다. 한 번의 거대 batch와 동일한 구조가 아니다.
- 제한: 샘플의 frame 간격, scaling과 browser 권한을 VIA 요건으로 복사하지 않는다. capture 시각 보존, pointer, native selection, local binding revision과 원본 재조회는 VIA에서 따로 설계해야 한다. 지금 사용할 모델 API의 capability 검증은 수행하지 않았다.

### R3-05. QuickSet: 불완전하고 경쟁하는 부분 의미를 결합

- 원 자료: Johnston 외, 1997, [Unification-based Multimodal Integration](https://aclanthology.org/P97-1036/), [원 논문 PDF §3, Figure 6](https://aclanthology.org/P97-1036.pdf). pp.284–286의 구조, 입력 표시와 결합 사례를 확인했다.
- 실제 시스템 설명: 음성 인식의 문자열에서 언어 agent가 부분 의미를 생산하고, gesture 인식도 복수 부분 해석을 생산한다. multimodal integration agent가 서로 맞는 후보를 결합해 bridge에 명령을 전달한다. 입력 하나를 먼저 정답으로 고정하지 않는다.
- VIA 적용: B의 producer별 부분 후보와 결합 책임을 뒷받침한다. 불명확 값, 경쟁 해석과 집합을 구별하는 중간 계약을 둘 수 있다.
- 제한: 논문이 설명한 pen / 지도 명령과 unification 구현이다. 현재 실행 소스를 확보하거나 재현하지 않았다. 원 논문의 시간 window와 확률 결합을 PC 전체의 click 없는 pointer에 적용하지 않는다. VIA의 후보 확장, producer revision과 부분 정정 계약은 별도 설계다.

### R3-06. OmniParser: screenshot에서 구조화된 요소를 반환하는 interface

- 저장소 / commit: `microsoft/OmniParser` / `354021201345a96178360b28733573e27269f2de`.
- 읽은 코드: [`omniparserserver.py` L34–44](https://github.com/microsoft/OmniParser/blob/354021201345a96178360b28733573e27269f2de/omnitool/omniparserserver/omniparserserver.py#L34).
- 실행: `/parse/`가 `base64_image`를 받고 parser를 호출해 annotated image, `parsed_content_list`와 latency를 반환한다. 서버 설정에 detector / caption model 경로와 device가 있다.
- VIA 적용: 원본을 읽는 screen 생산 단계와 다음 판단자가 소비하는 요소 목록 사이에 실제 interface를 둘 수 있다는 근거다.
- 제한: VIA에 이 helper 모델을 추가하지 않았다. parser의 존재는 모든 PDF 표 / graph / 셀 주소, 원형 집합과 navigation 의미를 지원한다는 뜻이 아니다. 단독으로 음성과의 결합, producer 재판단과 정확도 우위를 증명하지 않는다.

### R3-07. Psi: 도착 시각과 발생 시각, 충분한 자료를 기다리는 결합

- 저장소 / commit: `microsoft/psi` / `12e08469440704f0dfdb540ae7005b046a5d6509`.
- 읽은 코드: [`Fuse.cs` L126–191](https://github.com/microsoft/psi/blob/12e08469440704f0dfdb540ae7005b046a5d6509/Sources/Runtime/Microsoft.Psi/Components/Fuse.cs#L126). 공식 [Stream Fusion and Merging §2.3–3.1](https://github.com/microsoft/psi/wiki/Stream-Fusion-and-Merging)도 확인했다. wiki는 열람일 기준이며 commit 고정 코드와 구별한다.
- 실행: primary와 secondary를 별도 queue에 넣고 primary의 `OriginatingTime`에서 보간한다. 결과가 `InsufficientData`이면 기다리고, 자료가 만들어진 경우 원래 시각으로 게시한다. 더 이상 필요한 결과에 기여하지 않는 secondary 자료를 정리한다.
- VIA 적용: source 시각과 도착 시각을 보존하고, 실제 준비 범위를 표현해야 한다는 공통 기반이다. 이미 도착한 값만 쓰는 경로와 추가 자료를 기다리는 경로는 지연 / 자료 범위 선택이다.
- 제한: stream 순서와 시계 조건을 VIA의 모든 OS / cloud source가 보장한다고 가정하지 않는다. 시간 결합의 재현성은 사용자의 의도 정확성이 아니다. VIA의 유한 대기와 gap 처리 규칙은 추가 설계가 필요하다.

### R3-08. InproTKS와 ReTiCo: 잠정 가설의 추가, 철회와 확정

- 원 자료: Kennington 외, 2014, [InproTKS: A Toolkit for Incremental Situated Processing](https://aclanthology.org/W14-4312.pdf), §2–5. 원 논문은 센서 입력의 module network, IU의 same-level / grounded-in 관계, add / revoke / commit을 설명한다. Google Web Speech의 원격 지연, chunk 단위 partial과 거친 timestamp라는 제한도 직접 적는다. 이것은 VIA의 cloud 전사 시각을 단어 시각으로 간주하면 안 되는 근거다.
- 별도 구현: `retico-team/retico-core` / `ed4ba19f1b4d3f9f3f4b51c9addb0a49c67d600d`. [`abstract.py` L23–32](https://github.com/retico-team/retico-core/blob/ed4ba19f1b4d3f9f3f4b51c9addb0a49c67d600d/retico_core/abstract.py#L23), L83–145, L548–588, L762–815와 [`docs/basics.md`](https://github.com/retico-team/retico-core/blob/ed4ba19f1b4d3f9f3f4b51c9addb0a49c67d600d/docs/basics.md)를 읽었다.
- 코드: IU는 creator, ID, 이전 IU, `grounded_in`, committed / revoked와 생성 시각을 가진다. UpdateMessage로 변경을 전달하고 module의 입력 / 출력 가설에서 revoke 또는 commit한다. update 처리 시간과 queue overflow를 명시하며, 오래된 연결 깊이도 제한한다.
- VIA 적용: 잠정 binding이나 producer 후보를 일찍 만들 때 단순한 누적 문자열 대신 수정 가능한 가설과 실제 원본 dependency가 필요하다. A의 공동 binding과 B의 후보 양쪽에 적용할 수 있다.
- 제한: IU 생성 시각은 sensor 발생 시각이 아니다. `grounded_in` 한 연결과 참조 깊이 제한을 VIA의 모든 복수 원본 provenance로 복사하지 않는다. 프레임워크의 commit flag는 VIA의 영속 채택이나 외부 Action 허가가 아니다. revoke 호출이 모든 의미 의존 결과를 자동 올바르게 복구한다는 코드도 아니다. 적용 module이 갱신 규칙을 구현해야 한다.

## 3. 근거에서 설계로 옮길 때의 분리

| 확인 사실 | VIA가 추가해야 할 계약 | 미검증 품질 가설 |
| --- | --- | --- |
| 이미지와 native 목록을 공동 공급하는 코드가 있다 | 실제 발화 구간 원본, pointer, gap과 사용권의 보존 | A가 후보 손실 없이 더 적은 왕복으로 정확히 지칭할 수 있는가 |
| 부분 의미와 screen 요소를 별도 생산하는 실제 선례가 있다 | producer 소유 revision, 원본 재생산, 복수 후보와 미커버 범위 | B의 후보 재사용 이익이 생산과 왕복 비용보다 큰가 |
| 증분 가설의 identity와 철회 interface가 있다 | input revision과 채택 경계, 복수 원본 의존, 영향 불명 시 확대 판단 | 발화 중 처리가 실제 첫 응답을 줄이는가, 폐기 비용을 늘리는가 |
| 원시 시각에 따라 자료를 기다리는 stream 결합이 있다 | 시계 오차, bounded 대기, source별 누락과 시간 구간 | 추가 자료를 기다리는 이익이 어떤 장면에서 지연 비용을 정당화하는가 |

코드의 실행 가능성은 VIA 구현 완료가 아니며, 위 가설은 reference의 benchmark나 과거 DP 수치로 입증되지 않는다. 이번 조사에서는 두 주요 의미 생산 대안과 조합 가능한 실행 축을 구별할 근거를 확보했다. 지원 API의 실물 관측, 한국어 지칭 성공과 실제 메모리 / 지연 / 비용은 남은 검증이다.
