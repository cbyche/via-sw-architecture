# 화면 지칭 통합 분석의 공식 근거

2026-10-18 / 공식 코드와 원 논문 재확인 / 공개 자료 조사 / 설치, 모델 호출과 성능 실행 없음

[통합 설계](../../architecture/12-decisions/decision-packages/seven-problem-review/04-screen-speech-grounding.md) / [분석 계약](../../architecture/12-decisions/decision-packages/seven-problem-review/00-analysis-contract.md)

## 1. 조사 결과와 판정 변경

이번 질문은 현재 발화의 지칭을 원본에서 직접 생산하는 A와, 원본에 근거한 부분 후보를 producer별로 생산하여 결합하는 B의 실제 선택 근거다. 이전 [DP-47 자료](../../architecture/12-decisions/decision-packages/dp47-structure-references.md), [문제 3 source note](../five-problem-study/screen-speech-grounding.md)와 [17일 추천 재확인](../five-problem-study/grounding-recommendation-check-20261017.md)을 통합하고 아래 원문을 다시 확인했다.

추가로 확인한 중요한 내용은 두 가지다.

1. QuickSet과 finite-state multimodal 시스템은 partial meaning을 계산 가능한 규칙으로 결합한다. B의 결합마다 LLM을 강제할 이유는 없다. 같은 native/code 경로를 A에도 허용한다.
2. OmniParser 원 논문에는 같은 최종 VLM에 화면 영역의 설명을 더하는 비교와 screen parsing ablation이 있다. 따라서 B의 후보 재사용이 총비용을 줄이는가만이 아니라, **별도 화면 생산의 정확도 가능성 대 추가 생산/모델 비용**으로 대안을 비교할 근거가 있다.

이 때문에 통합 문서는 주요 DP 포함을 권고한다. 단, OmniParser가 producer ownership이나 VIA의 temporal binding을 검증한 것은 아니다. 구체 B에 추가 parser 의존을 명시하고, A가 동일 parser를 보조 입력으로 쓰는 A4와 parser 없는 B를 교차 구성으로 남겼다. 기존 자료는 수정하지 않았다.

## 2. 새로 보강한 주요 근거

### S-G1. OmniParser 원 논문: 화면 생산의 정확도 경로

- 원문: Lu 외, 2024, [OmniParser for Pure Vision Based GUI Agent, arXiv v1](https://arxiv.org/html/2408.00203v1).
- 확인: §3의 detection/OCR/local semantics 구성, §4.1 Table 1의 같은 GPT-4V에서 local semantics 유무 비교, §4.2 Table 2의 parsing ablation, §5의 실패 분석.
- 범위: 정지 화면의 UI 대상 연결이다. 반복 요소, bbox granularity와 crop 맥락 손실이 남는다. 한국어 발화, click 없는 circle, page crossing, 정정의 referent-set 정확도로 외삽하지 않는다.
- 설계 적용: 후보를 명시적으로 조직해 시각 연결 부담을 줄일 가능성과 선행 생산 비용. 소유권 분리 자체의 우위로 해석하지 않는다.

추가로 공식 [OmniParser V2 설명](https://www.microsoft.com/en-us/research/articles/omniparser-v2-turning-any-llm-into-a-computer-use-agent/)도 확인했다. 고해상도 작은 UI 요소에서 parsing을 사용한 grounding 결과를 소개하지만 다른 버전/모델/benchmark다. 위 v1 ablation과 합쳐 VIA 결과나 예측 수치를 만들지 않는다.

### S-G2. OmniParser 고정 코드: 추가 모델과 실제 반환 경계

- snapshot: `microsoft/OmniParser` / `354021201345a96178360b28733573e27269f2de`.
- [`util/omniparser.py`](https://github.com/microsoft/OmniParser/blob/354021201345a96178360b28733573e27269f2de/util/omniparser.py): detector와 caption processor를 초기화하고, image를 decode한 뒤 OCR와 local semantics를 사용해 annotated image와 `parsed_content_list`를 반환한다.
- [`omniparserserver.py`](https://github.com/microsoft/OmniParser/blob/354021201345a96178360b28733573e27269f2de/omnitool/omniparserserver/omniparserserver.py): `/parse/`가 base64 image를 받고 parser 결과와 latency를 반환한다.
- 두 파일을 이번에 직접 열었다. 이 코드 snapshot이 2024년 논문의 동일 모델/실험 재현본이라고 주장하지 않는다.
- VIA에서 parser를 cloud service 뒤에 두는 것은 설계 가정이다. 공개 API의 상용 availability/가격이나 실제 PC RAM을 확인한 것이 아니다. 로컬 배치라면 전체 helper weights/workspace가 QA06에 들어간다. cloud 배치라면 추가 청구와 network 및 로컬 전달 사본을 남긴다.

### S-G3. QuickSet: typed 부분 의미와 결합

- 원문: Johnston 외, 1997, [Unification-based Multimodal Integration](https://aclanthology.org/P97-1036.pdf), §3, Figure 6.
- speech 문자열의 언어 해석과 gesture의 복수 해석을 typed feature structure로 표현하고, integration이 양립하는 부분 의미를 결합/순위화한다. 불완전한 의미를 표현하고 type 제약을 쓸 수 있다.
- B의 코드 결합 근거다. VIA에서는 필요한 관계가 실제 생산되고 경쟁/미커버 의미가 남지 않는 조건을 따로 정의해야 한다. candidate가 하나라는 이유만으로 정답이 되지는 않는다.
- 논문의 pen/지도 영역, 인식 모델과 temporal window를 일반 PC pointer에 복사하지 않는다. 원본 보존, producer revision과 currentness gate는 VIA 설계다.

### S-G4. Finite-state multimodal parsing: 다른 결합 계산과 범위 한계

- 원문: Johnston and Bangalore, 2000, [Finite-state Multimodal Parsing and Understanding](https://aclanthology.org/C00-1054.pdf), §3–4.
- speech/gesture/결합 의미를 finite-state 표현으로 처리하며 특정 object와 gesture 값은 event buffer 참조로 전달한다. gesture 해석으로 speech recognition search를 제약하는 구성도 설명한다.
- 모든 값 조합을 거대 원본 입력으로 다시 처리하지 않는 실구현 선례다. 그러나 domain grammar, 인식기 제어와 순서 단순화 가정이 있다.
- 현재 VIA의 고정 cloud 음성 API가 decoder lattice/search 조정을 허용한다는 근거가 없어 독립 C로 채택하지 않는다. B의 typed 결합을 검토하는 근거로만 쓴다.

### S-G5. ReALM: text로 표현한 화면 후보의 실용성과 손실

- 원문: Moniz 외, 2024, [ReALM v1](https://arxiv.org/html/2403.20329v1), §3, §5.3.2, §7. 공식 [Apple 연구 소개](https://machinelearning.apple.com/research/realm-reference)에서도 원 논문을 확인했다.
- upstream detector가 얻은 entity와 주변 text, 위치를 이용해 화면을 text로 구성한다. 출력은 entity 집합 또는 해당 없음이다. 섬세한 공간 관계의 정보 손실을 논문이 명시한다.
- VIA에서는 native가 이미 제공하는 정보인지, 별도 visual 생산이 필요한지 구별한다. A도 같은 native 정보를 사용할 수 있으므로 text-only라는 이유만으로 B 고유 이익은 아니다.
- fine-tuned 모델의 성과를 동일 cloud 모델의 VIA 성과로 이전하지 않는다. 과거 frame 보존과 pointer/정정 계약도 제공하지 않는다.

## 3. 공동 입력과 관측 현실의 재확인

### S-G6. Magentic-One WebSurfer

- snapshot: `microsoft/autogen` / `027ecf0a379bcc1d09956d46d12d44a3ad9cee14`.
- [`_multimodal_web_surfer.py`](https://github.com/microsoft/autogen/blob/027ecf0a379bcc1d09956d46d12d44a3ad9cee14/python/packages/autogen-ext/src/autogen_ext/agents/web_surfer/_multimodal_web_surfer.py), `_generate_reply`의 rect/viewport/screenshot, Set-of-Mark, visible targets와 model create 경로를 직접 읽었다.
- native 목록과 annotated pixels, 요청을 함께 최종 model에 줄 수 있는 근거다. native가 없어도 parser 없이 항상 성공한다는 증거는 아니다.
- 현재 browser action의 입력 경로이며 발화 시각, 과거 page, 사용자 pointer와 부분 정정의 완성 구현으로 소개하지 않는다. VIA의 domain 업무 Action도 이 코드에서 가져오지 않는다.

### S-G7. Google Live Console

- snapshot: `google-gemini/live-api-web-console` / `0a4542fe0e39d07956ea7af5de45d7c81fde8960`.
- [`ControlTray.tsx`](https://github.com/google-gemini/live-api-web-console/blob/0a4542fe0e39d07956ea7af5de45d7c81fde8960/src/components/control-tray/ControlTray.tsx), audio `onData`와 `sendVideoFrame`를 직접 확인했다. PCM과 canvas JPEG가 같은 client의 realtime input으로 전달된다.
- 공동 media 입력의 구현 선례다. sample cadence/downscaling, browser 권한, source timestamp와 binding 저장의 부재를 VIA의 capability 보장으로 바꾸지 않는다. A/B 기본은 완성 입력 이후 의미 판단이고 streaming은 A3/B3 보완이다.

### S-G8. Windows 관측 API

| 공식 원문과 이번 확인 | VIA가 추가로 정의할 것 |
| --- | --- |
| [WASAPI GetBuffer](https://learn.microsoft.com/en-us/windows/win32/api/audioclient/nf-audioclient-iaudiocaptureclient-getbuffer): audio packet, 첫 frame device/QPC position, discontinuity/timestamp flags, buffer 수명 | provider 전사의 원음 구간 대응. VAD에서 단어 시각을 만들지 않음 |
| [Windows screen capture](https://learn.microsoft.com/en-us/windows/apps/develop/media-authoring-processing/screen-capture): frame의 ContentSize/Surface/SystemRelativeTime, frame 반납 뒤 surface 참조 재사용 금지 | 필요한 bytes의 안전한 보존, decode/upload 사본과 encoded raw의 다른 수명 |
| [TextPattern GetSelection](https://learn.microsoft.com/en-us/windows/win32/api/uiautomationclient/nf-uiautomationclient-iuiautomationtextpattern-getselection): 현재 선택 range, 빈 caret range, selection 지원 조회 | 과거 선택은 실제 기록/현재 유효 상태가 있어야 함. 현재 조회로 지나간 selection을 복원하지 않음 |

이 API 문서의 존재를 VIA 구현이나 모든 앱/version의 UIA coverage 시험으로 취급하지 않는다. pointer 경로, DPI와 다중 창의 상세 관측 목록은 기존 [Windows 자료](../../architecture/12-decisions/decision-packages/dp47-windows-observation-sources.md)에 보존되어 있으며 이번 문서에서는 그 지원 범위를 확대하지 않았다.

### S-G9. 시간 대응과 증분 상태

- [Psi Stream Fusion and Merging](https://github.com/microsoft/psi/wiki/Stream-Fusion-and-Merging)를 이번에 다시 열었다. 도착 시각과 원래 시각을 구별하고 stream을 대응시키는 자료다. 시간 결합은 의도 판단이 아니다.
- [InproTKS 원 논문](https://aclanthology.org/W14-4312.pdf) §2–3을 다시 읽었다. 가설은 added/revoked/committed 상태와 grounded-in 관계를 가진다. sensor listener 자체가 모든 근거 링크와 revoke를 자동 제공하는 것은 아니다.
- A3/B3의 잠정 가설 계약에 적용한다. processing commit을 VIA 영속 채택이나 미래 사용자 정정 금지로 해석하지 않는다.

## 4. 다섯 harness의 직접 적용 한계와 재조회 실패

다섯 harness는 [기존 공식 코드 원장](../agent-harness-20261015.md)과 [문제 3 원장](../five-problem-study/screen-speech-grounding.md)을 함께 읽었다. 지금 표는 전체 저장소에 기능이 없다는 판정이 아니다.

| harness / source | 확인 범위와 이번 상태 | 이 분석의 역할 |
| --- | --- | --- |
| AutoGen Magentic-One | 위 S-G6 고정 코드를 이번에 직접 재조회 | 공동 native/image 입력의 직접 근거 |
| OpenHands SDK | `fd387b58dc7b707b73c159845c0396a17000be24`의 [`BrowserObservation`](https://github.com/OpenHands/software-agent-sdk/blob/fd387b58dc7b707b73c159845c0396a17000be24/openhands-tools/openhands/tools/browser_use/definition.py), 이전 직접 확인 기록 유지. 이번 raw/GitHub web 재조회는 cache miss | text/image와 저장본의 구별. 새 temporal grounding 근거로 세지 않음 |
| OpenClaw | `e7a155a8ae860d71b1ab5eed45fa4af338011db9`의 [`snapshot`](https://github.com/openclaw/openclaw/blob/e7a155a8ae860d71b1ab5eed45fa4af338011db9/extensions/browser/src/browser/pw-tools-core.snapshot.ts), 이전 직접 확인 기록 유지. 이번 raw/GitHub 재조회는 cache miss | snapshot identity와 사용 현재성의 공통 제약 |
| LangChain Deep Agents | 이전 고정 코드의 offload/요약/위임 원장 검토. 이번 temporal grounding 코드 재조사 완료라고 하지 않음 | 현재 문제의 직접 후보 생산/결합 근거로 쓰지 않음 |
| CrewAI | 이전 고정 코드의 Task context/위임/기억 원장 검토. 이번 temporal grounding 코드 재조사 완료라고 하지 않음 | 현재 문제의 직접 후보 생산/결합 근거로 쓰지 않음 |

Psi의 기존 고정 `Fuse.cs` raw 경로도 이번에는 cache miss였으며 공식 wiki 재확인과 이전 고정 코드 분석을 구별했다. WindowsAgentArena 논문 PDF도 재조회에 실패하여 새 정확도/시간 주장의 근거로 사용하지 않았다. 읽지 못한 본문이나 검색 snippet을 재현 검증으로 바꾸지 않는다.

## 5. 근거에서 설계로 옮긴 범위

| 주장 종류 | 이번에 말할 수 있는 것 | 아직 말할 수 없는 것 |
| --- | --- | --- |
| 실제 구현 | 공동 image/native 입력, screen parser의 별도 모델/반환, typed 부분 의미 결합이 실제 코드 또는 원 논문에 있음 | 모든 기능이 한 harness에 완성되어 있음 |
| 외부 비교 근거 | 동일 최종 VLM에서 screen representation을 바꾸어 target 연결을 개선한 보고가 있음 | VIA QA02의 수치나 모든 temporal 지칭 정확도 우위 |
| VIA 설계 추론 | 후보 production과 binding을 분리한 B에서 시각 혼동 감소 가능성 대 production/확장 비용을 비교할 수 있음 | producer ownership 또는 추가 module 수 자체가 정확도/QA04 이익을 만듦 |
| 검증 계획 | A/B/B4와 A4/parser 없는 B로 representation과 ownership의 효과를 분리하여 확인 | 현재 해당 비교를 실행했거나 dependency 가격/RAM을 확정함 |

같은 관측 원본과 native 정보, 완료 조건을 유지한다. B의 원본 재확장과 raw 보존은 기본이며 A의 한 번 다중 지칭 판단을 제한하지 않는다. 새 모델 학습, 제품/API 실행, 측정과 commit/push는 이 조사에서 수행하지 않았다.
