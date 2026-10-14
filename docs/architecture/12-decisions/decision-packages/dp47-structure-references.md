# DP-47 구조 조사 — 시간 근거, 공동 해석과 후보 교환

> 2026-10-14 보강 / 공개 코드, 공식 문서와 원 논문을 읽은 설계 조사 / 해당 SW 설치, 실행 및 VIA 품질 검증 없음
> [DP-47 설계](./04-47-screen-voice-grounding.md) / [사고실험과 후보 탐색](./dp47-design-experiments.md)

## 1. 조사 질문과 사용 방법

조사 질문은 세 가지다. 사라지는 화면과 동작을 어떤 자료로 전달하는가, 말과 동작의 상호 의존 판단을 어떻게 협력시키는가, 새 근거와 정정이 오면 어떤 결과를 다시 생산하는가. 모델 학습이나 지칭 매칭 알고리즘을 비교하려는 조사가 아니다.

아래 사실은 링크의 본문, 설명, 예제와 공개 코드에서 확인했다. **VIA 적용 열은 작성자의 설계 추론**이다. 공개 코드, 논문이 설명하는 실제 시스템, 구조 표준과 모델 입력 예제는 서로 다른 근거다. 공개 구현의 존재는 VIA의 대표 장면을 해결했다는 증거가 아니다. 버전 고정이나 설치 가능성 시험도 수행하지 않았다. 블로그나 검색 요약을 구현 사실의 원본으로 사용하지 않았다.

## 2. 조사한 구조와 구현

| 자료 | 확인한 구조 또는 구현 | VIA에 적용한 설계 판단 | 그대로 가져오지 않는 것 |
| --- | --- | --- | --- |
| [Microsoft Platform for Situated Intelligence: Brief Introduction](https://github.com/microsoft/psi/wiki/Brief-Introduction) | 센서의 발생 시각을 가진 stream과 비동기 처리, 여러 stream의 시간 결합을 설명한다 | 원래 시각과 처리/도착 시각을 분리한 관측 계약은 양안의 공통 기반이다 | 이 framework를 설치하거나 각 Windows 관측이 정확히 같은 시각을 가진다고 가정하지 않는다 |
| [Psi Stream Fusion and Merging §2.3–3.1](https://github.com/microsoft/psi/wiki/Stream-Fusion-and-Merging) | 현재 도착한 값만 사용하는 결합과, 추가 자료를 기다려 발생 시각 기준 결과를 재현하는 결합을 구별한다 | 완료 범위, 늦은 자료와 제한된 기다림을 명시해야 한다. 빨리 도착한 화면을 정답으로 채택하는 경로를 금지한다 | stream별 순서 보장과 정확한 timestamp를 VIA의 모든 provider에 자동 적용하지 않는다. 시간 결합의 정확성이 사용자 의도의 정확성을 보장하지 않는다 |
| [GStreamer Synchronisation](https://gstreamer.freedesktop.org/documentation/additional/design/synchronisation.html), [Latency](https://gstreamer.freedesktop.org/documentation/additional/design/latency.html) | clock, buffer timestamp, segment에서 running time을 계산하며 live pipeline의 지연을 다룬다 | 음성과 영상 운반과 시계 대응은 의미 판단 이전의 책임이다. sample/frame의 기반과 변환 오차를 남긴다 | 재생용 동기화 정책이나 고정 latency를 화면 지칭의 의미 결합 정책으로 사용하지 않는다 |
| [Qwen3-Omni 공식 README: QuickStart](https://github.com/QwenLM/Qwen3-Omni#quickstart) | 한 message에 image/audio/text를 함께 넣는 모델 사용 예제와 audio/video 입력 예제를 제공한다 | 공동 입력 dependency의 참고다. A와 B 양쪽에서 사용할 수 있으므로 이것만으로 A의 SW 구조를 뒷받침하지 않는다 | 실제 선택할 클라우드 API의 mixed-input, 시간 보존, 출력 schema 지원은 별도 확인 대상이다. 이 자료는 VIA의 pointing/정정 성능 증거가 아니다 |
| [Google Gemini Live API Web Console: ControlTray.tsx](https://github.com/google-gemini/live-api-web-console/blob/main/src/components/control-tray/ControlTray.tsx) | 공개 React 구현에서 audio recorder와 화면 또는 webcam frame을 같은 live client의 `sendRealtimeInput`으로 보낸다 | 별도 semantic target/mention 생산을 필수로 두지 않는 공동 입력 경로는 A의 선례다. 단일 호출이 아니라 같은 세션에 비동기로 들어가는 입력이다 | sample의 frame 축소/주기, 모델 및 browser 환경을 VIA 요건으로 가져오지 않는다. pointer 경로, native 선택, 과거 원본 read와 binding revision의 구현 근거로 사용하지 않는다 |
| [QuickSet: Unification-based Multimodal Integration, Johnston 등, ACL-EACL 1997, §2–3](https://aclanthology.org/P97-1036.pdf) | 실제 구현된 pen/voice 시스템의 설명이다. 음성, gesture의 복수 부분 해석을 typed feature structure로 표현하고 별도 multimodal integration이 결합한다. gesture의 하나의 해석만 먼저 확정하지 않는다 | B는 모달리티 생산자, 불완전, 경쟁 후보와 결합 책임을 실제 계약으로 만든다. 단일 화면 후보의 조기 확정을 피한다 | 공개 실행 소스는 이 조사에서 확인하지 않았다. pen ink와 지도 command 영역, 논문의 시간 window와 확률 곱, Prolog/unification 알고리즘을 PC 전체, click 없는 pointing으로 확대하지 않는다 |
| [W3C Multimodal Architecture and Interfaces, 2012, §4.3, 5, 6](https://www.w3.org/TR/mmi-arch/) | 구조와 interface 표준이다. modality 구성과 조정 책임을 구별하고 lifecycle event 및 결과 전달 계약을 정의한다. Interaction Manager의 구현 형태로 SCXML, statechart와 ECMAScript를 설명한다 | 모델 내부를 black box로 두어도 시작, 결과 전달, 취소와 context 정리는 코드 조정 책임으로 설계할 수 있다 | 실행 가능한 구현으로 소개하지 않는다. 후보의 의미상 충분 여부가 코드로 결정된다는 근거가 아니다. W3C의 Interaction Manager를 VIA의 동명 Component와 동일시하지 않는다. XML transport나 distributed deployment를 요구하지 않는다 |
| [W3C EMMA, 2009, §3.3, 4.1.2, 4.2.10](https://www.w3.org/TR/emma/) | 경쟁 해석, 집합, 순서, 원본과 단계별 파생 관계, 시간 기준을 표현한다. 서로 다른 인식기의 confidence가 같은 척도라고 보장하지 않는다 | B의 후보에는 경쟁/집합 구별, 원본 참조, 생산 revision과 시간 불확실성이 필요하다. 단일 숫자 confidence로 의미 채택을 대체하지 않는다 | EMMA 준수나 XML schema를 VIA 요건으로 추가하지 않는다. 정확한 단어 시각을 없는 source에서 만들어내지 않는다 |
| [Microsoft OmniParser 공식 구현](https://github.com/microsoft/OmniParser), [Microsoft UFO AppAgent strategy: control collection](https://github.com/microsoft/UFO/blob/main/documents/docs/ufo2/app_agent/strategy.md) | screenshot의 구조화된 요소 생산 및 UIA/visual control 수집과 병합 뒤 LLM에 넘기는 단계가 공개되어 있다 | 이미지와 native 정보를 가진 대상 후보 생산은 구체적 구현 형태가 있다. source 종류와 원본 영역을 보존해야 한다 | GUI control 후보가 PDF의 모든 표, 그래프, 셀 범위를 포함한다는 주장은 하지 않는다. overlap만으로 같은 객체라고 확정하지 않는다. OmniParser의 helper 모델이나 UFO의 업무 실행을 VIA 안에 추가하지 않는다 |

Google 샘플은 A의 공동 입력 경로, QuickSet은 B의 부분 의미 생산과 결합, OmniParser/UFO는 화면 대상 생산 단계, W3C는 실행 조정 interface의 참고다. Psi와 GStreamer는 양안의 시간 대응 참고다. 어느 쪽도 필수 package로 선정하지 않았다. **B의 양방향 재판단과 version 계약은 위 자료를 그대로 복제한 것이 아니라, VIA의 페이지 이동, 정정 장면에 맞추어 추가한 설계**다.

### 2.1 실제로 읽을 코드와 구조

**A의 입력 경로 — Google Web Console:** [ControlTray.tsx](https://github.com/google-gemini/live-api-web-console/blob/main/src/components/control-tray/ControlTray.tsx)의 audio recorder `data` handler는 PCM을 `client.sendRealtimeInput`으로 보낸다. 같은 파일의 `sendVideoFrame`은 video를 canvas에 그려 JPEG로 만든 뒤 같은 client로 보낸다. 두 수집 경로는 분리되어 있지만 의미 후보를 별도 생산한 뒤 결합하는 단계는 이 코드의 필수 경로가 아니다. 이 코드를 A 전체의 reference 구현으로 부르지는 않는다. VIA의 Timeline & Buffer 조회와 부분 binding 수정은 별도 설계다.

**B의 전체 협력 구조 — QuickSet:** 원 논문 §3의 **Figure 6과 바로 아래 설명**을 먼저 읽는다. speech recognizer의 문자열에서 natural language agent가 부분 의미를 만들고, gesture recognizer는 복수 해석을 만든다. multimodal integration agent가 호환되는 부분 의미를 결합해 bridge에 완성 명령을 전달한다. gesture의 route/area 같은 모호함을 음성과 함께 판단한다. 이것은 B의 생산자와 결합 책임을 설명하는 선례이며, 해당 unification 알고리즘을 VIA에 채택하자는 제안은 아니다.

**B의 화면 생산 interface — OmniParser:** [omniparserserver.py](https://github.com/microsoft/OmniParser/blob/master/omnitool/omniparserserver/omniparserserver.py)의 `POST /parse/`는 `base64_image`를 받고 `omniparser.parse`를 호출한다. 반환은 `som_image_base64`, `parsed_content_list`와 latency다. 원본 screenshot과 구조화된 요소 목록 사이에 실제 interface가 있다는 근거다. [UFO 공식 AppAgent strategy 설명](https://github.com/microsoft/UFO/blob/main/documents/docs/ufo2/app_agent/strategy.md)의 `AppControlInfoStrategy`는 UIA/OmniParser control 수집과 병합, `AppLLMInteractionStrategy`는 다음 LLM 처리 단계를 설명한다. 후자는 공식 실행 구조 설명으로 읽었으며 이 조사에서 해당 runtime을 실행하지 않았다.

**공통 시간 대응 — Psi:** 공식 문서의 `Available.Nearest`와 `Reproducible.Nearest`, `Fuse`와 `Join`의 차이를 읽는다. 도착 지연이 달라질 때 이미 온 자료만 사용하는 결과와 발생 시각에 대응하는 자료를 기다리는 결과가 어떻게 달라지는지 설명한다. VIA의 “전사가 도착했을 때 현재 화면만 읽는다”는 접근의 한계를 구체화하는 참고이며, 사용자 지칭의 의미 결합 구현은 아니다.

### 2.2 세 설계 포인트를 어느 수준까지 뒷받침하는가

| 사용자와 합의한 질문 | 참고 자료가 뒷받침하는 것 | VIA에서 추가한 것과 남은 검증 |
| --- | --- | --- |
| 지칭 근거가 어떤 형태로 남는가 | Google의 공동 media 입력, QuickSet의 불완전/경쟁 의미, OmniParser의 구조화된 화면 요소라는 서로 다른 전달 형태가 있다 | 사라진 page의 원본 참조, pointer 경로, native 선택과 source gap의 보존. 어느 표현이 VIA 장면의 필요한 정보를 유지하는지는 미검증 |
| 말과 화면 판단이 어디서 만나는가 | 공동 모델 입력 경로와 별도 부분 의미의 multimodal integration이라는 실제 선례가 있다. W3C는 처리 실행을 조정하는 interface를 제공한다 | B의 의미 결합은 Multimodal Binding Resolver가, 실행/version 조정은 Grounding Coordinator가 맡는다. 원 생산자로 돌아가는 후보 확장과 양방향 재판단은 VIA 설계 |
| 정정 시 무엇을 유지하고 다시 판단하는가 | 후보를 명시적으로 주고받는 선례는 있다 | “아래 graph만, 처음 매출 범위는 그대로”의 binding patch, producer revision과 dependency 전파는 위 reference에서 그대로 확인한 기능이 아니다. 재사용 이익과 재판단 비용을 A의 부분 patch와 비교해야 한다 |

이 비교가 전통적인 SW 설계 문제가 되는 지점은 학습 방법이 아니라 **정상 실행의 입력 자료, 중간 결과 소유와 수정 계약, 후속 처리의 준비 조건과 재실행 경로**가 달라진다는 것이다. A는 원본에서 새 target을 직접 제안할 수 있고, B는 해당 producer가 후보를 생산하거나 수정해야 결합이 진행된다. reference는 이 두 구조가 존재할 수 있다는 근거를 주며, VIA에서의 우위는 사고실험과 후속 검증으로 따로 판단한다.

### 2.3 구현 요소와 architectural pattern으로 구체화한 범위 / 2026-10-14

[본문 §8.5](./04-47-screen-voice-grounding.md#85-구현을-가정한-상세-구조와-실행-계약)는 A의 공동 입력과 B의 부분 의미 생산/결합을 코드 실행 구조로 펼쳤다. Google, QuickSet나 OmniParser가 아래 queue, lock과 데이터 모델을 그대로 사용한다고 주장하지 않는다. 이 부분은 VIA의 정정과 bounded 실행 요구를 위한 구현 가정이다.

- A는 layered application 안의 비동기 workflow, Ports & Adapters와 versioned binding aggregate로 표현했다. B는 Coordinator가 제어하는 dataflow, 생산자 경로의 Fork–Join과 재판단 feedback, producer별 versioned repository로 표현했다.
- 공통 전달 수단은 local Producer–Consumer queue다. [Microsoft System.Threading.Channels 공식 문서](https://learn.microsoft.com/en-us/dotnet/core/extensions/channels)는 typed FIFO, bounded capacity와 full-mode, 비동기 write/read를 제공한다. `Channel<T>`는 구현 대응 예시이며 VIA의 언어/SDK 최종 선택은 아니다.
- [Enterprise Integration Patterns의 Point-to-Point Channel](https://www.enterpriseintegrationpatterns.com/patterns/messaging/PointToPointChannel.html)은 한 메시지를 한 수신자가 소비하는 전달 관계를 설명한다. VIA의 역할별 job과 한 Coordinator로 돌아오는 완료 통지는 이 관계에 맞춘 설계다. subscriber 방송이나 외부 message bus를 추가할 근거로 쓰지 않는다.
- RevisionGate의 짧은 lock, immutable 후보 snapshot, 요청별 RAM Map과 영속 채택의 분리는 VIA에서 추가한 게시 계약이다. queue의 존재만으로 원자성, 취소, crash 복구나 의미 정확성이 확보된다고 설명하지 않는다.

## 3. reference를 조사한 뒤 배제한 단순 결론

- timestamp가 있으면 “이 표”의 정답을 자동으로 알 수 있다는 결론을 배제했다. 지칭은 발화보다 먼저 시작하거나 화면 변화 뒤에도 과거 대상을 가리킬 수 있다.
- 복수 후보를 남기면 B의 정확성이 높다는 결론을 배제했다. 원래 대상이 후보에 없으면 모든 후속 판단이 같은 오류를 공유한다. 원본 재조회와 후보 확장이 필요하다.
- structured screen parser가 있으면 semantic 모델을 쓰지 않는다는 결론을 배제했다. UIA로 읽을 수 없는 graph의 시각적 범위, navigation과 집합 지칭, 정정 의미는 여전히 의미 판단이 필요하다.
- multimodal 모델이 있으면 SW는 자료를 보내기만 하면 된다는 결론을 배제했다. source lifetime, missing evidence, revision, 권한과 후속 사용은 모델 API 밖의 책임이다.
- recognition/fusion 분리나 추가 service의 존재만으로 major DP를 주장하지 않는다. 후보 소유, 재생산과 정상 사용의 자료 흐름 차이가 남아야 한다.

## 4. 공식 자료로도 확인되지 않은 조건

| 미확인 | 설계에 남기는 제한 |
| --- | --- |
| 실제 cloud semantic API가 동일 호출에서 원음, 다중 이미지, 관측 시간을 어떻게 받는가 | 양안 같은 capability profile을 사용한다. 원음을 지원하지 않으면 양쪽 모두 전사와 알려진 시간 구간만 사용하며, 필요한 지칭 시점을 정할 수 없으면 확인한다 |
| spreadsheet/PDF viewer의 native selection, document identity가 실제로 무엇을 노출하는가 | 지원된 native 정보와 visual snapshot을 구별한다. 노출되지 않은 파일 주소, cell 주소나 stable object ID를 생성하지 않는다 |
| click 없는 circle의 표현력이 현재 모델에서 충분한가 | 취득 경로와 당시 화면을 전달하는 계약만 정의했다. 인식 성공은 미검증이다 |
| 요청 중 화면의 변화를 어느 범위까지 예산 안에 남길 수 있는가 | 동일 acquisition profile과 한도로 비교한다. 삭제, sampling, crop으로 잃은 근거를 표시한다 |
| B의 후보 생산과 교환이 A의 targeted 공동 해석보다 이득이 있는가 | [사고실험](./dp47-design-experiments.md)의 반증 조건으로 남긴다. 공개 구현의 다른 목적, benchmark 결과로 대신하지 않는다 |

조사 결과는 두 설계의 구조적 실현 경로를 뒷받침한다. 품질 우위, 모든 앱 지원과 DP 최종 자격은 아직 입증하지 않는다.
