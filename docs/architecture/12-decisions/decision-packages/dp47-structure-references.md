# DP-47 구조 조사 — 시간 근거, 공동 해석과 후보 교환

> 2026-10-13 / 공식 문서, 원 논문을 읽은 설계 조사 / 해당 SW 설치, 실행 및 VIA 품질 검증 없음
> [DP-47 설계](./04-47-screen-voice-grounding.md) / [사고실험과 후보 탐색](./dp47-design-experiments.md)

## 1. 조사 질문과 사용 방법

조사 질문은 세 가지다. 사라지는 화면과 동작을 어떤 자료로 전달하는가, 말과 동작의 상호 의존 판단을 어떻게 협력시키는가, 새 근거와 정정이 오면 어떤 결과를 다시 생산하는가. 모델 학습이나 지칭 매칭 알고리즘을 비교하려는 조사가 아니다.

아래 사실은 링크의 본문, 설명, 예제에서 확인했다. **VIA 적용 열은 작성자의 설계 추론**이다. 공개 구현의 존재는 VIA의 대표 장면을 해결했다는 증거가 아니다. 버전 고정이나 설치 가능성 시험도 수행하지 않았다. 블로그나 검색 요약을 구현 사실의 원본으로 사용하지 않았다.

## 2. 조사한 구조와 구현

| 자료 | 확인한 구조 또는 구현 | VIA에 적용한 설계 판단 | 그대로 가져오지 않는 것 |
| --- | --- | --- | --- |
| [Microsoft Platform for Situated Intelligence: Brief Introduction](https://github.com/microsoft/psi/wiki/Brief-Introduction) | 센서의 발생 시각을 가진 stream과 비동기 처리, 여러 stream의 시간 결합을 설명한다 | 원래 시각과 처리/도착 시각을 분리한 관측 계약은 양안의 공통 기반이다 | 이 framework를 설치하거나 각 Windows 관측이 정확히 같은 시각을 가진다고 가정하지 않는다 |
| [Psi Stream Fusion and Merging §2.3–3.1](https://github.com/microsoft/psi/wiki/Stream-Fusion-and-Merging) | 현재 도착한 값만 사용하는 결합과, 추가 자료를 기다려 발생 시각 기준 결과를 재현하는 결합을 구별한다 | 완료 범위, 늦은 자료와 제한된 기다림을 명시해야 한다. 빨리 도착한 화면을 정답으로 채택하는 경로를 금지한다 | stream별 순서 보장과 정확한 timestamp를 VIA의 모든 provider에 자동 적용하지 않는다. 시간 결합의 정확성이 사용자 의도의 정확성을 보장하지 않는다 |
| [GStreamer Synchronisation](https://gstreamer.freedesktop.org/documentation/additional/design/synchronisation.html), [Latency](https://gstreamer.freedesktop.org/documentation/additional/design/latency.html) | clock, buffer timestamp, segment에서 running time을 계산하며 live pipeline의 지연을 다룬다 | 음성과 영상 운반과 시계 대응은 의미 판단 이전의 책임이다. sample/frame의 기반과 변환 오차를 남긴다 | 재생용 동기화 정책이나 고정 latency를 화면 지칭의 의미 결합 정책으로 사용하지 않는다 |
| [Qwen3-Omni 공식 README: QuickStart](https://github.com/QwenLM/Qwen3-Omni#quickstart) | 한 message에 image/audio/text를 함께 넣는 예제와 audio/video 입력 예제를 제공한다 | A의 공동 원본 입력에는 실제 dependency 형태가 있다. 동일 dependency를 B의 생산자와 결합 판단에도 사용할 수 있다 | 실제 선택할 클라우드 API의 mixed-input, 시간 보존, 출력 schema 지원은 별도 확인 대상이다. 이 자료는 VIA의 pointing/정정 성능 증거가 아니다 |
| [QuickSet: Unification-based Multimodal Integration, Johnston 등, ACL-EACL 1997, §2–3](https://aclanthology.org/P97-1036.pdf) | 음성, gesture의 복수 부분 해석을 typed feature structure로 표현하고 별도 multimodal integration이 결합한다. gesture의 하나의 해석만 먼저 확정하지 않는다 | B는 모달리티 생산자, 불완전, 경쟁 후보와 결합 책임을 실제 계약으로 만든다. 단일 화면 후보의 조기 확정을 피한다 | pen ink와 지도 command 영역, 논문의 시간 window와 확률 곱, Prolog/unification 알고리즘을 PC 전체, click 없는 pointing으로 확대하지 않는다 |
| [W3C Multimodal Architecture and Interfaces, 2012, §4.3, 5, 6](https://www.w3.org/TR/mmi-arch/) | modality 구성과 조정 책임을 구별하고 lifecycle event 및 결과 전달 계약을 정의한다. EMMA를 통한 인식/결합 분리를 설명한다 | 모델 내부를 black box로 두어도 시작, 결과, 추가 자료 요구, 취소와 context 정리는 SW가 설계할 수 있다 | W3C의 Interaction Manager를 VIA의 동명 Component와 동일시하지 않는다. XML transport나 distributed deployment를 요구하지 않는다 |
| [W3C EMMA, 2009, §3.3, 4.1.2, 4.2.10](https://www.w3.org/TR/emma/) | 경쟁 해석, 집합, 순서, 원본과 단계별 파생 관계, 시간 기준을 표현한다. 서로 다른 인식기의 confidence가 같은 척도라고 보장하지 않는다 | B의 후보에는 경쟁/집합 구별, 원본 참조, 생산 revision과 시간 불확실성이 필요하다. 단일 숫자 confidence로 의미 채택을 대체하지 않는다 | EMMA 준수나 XML schema를 VIA 요건으로 추가하지 않는다. 정확한 단어 시각을 없는 source에서 만들어내지 않는다 |
| [Microsoft OmniParser 공식 구현](https://github.com/microsoft/OmniParser), [Microsoft UFO AppAgent strategy: control collection](https://github.com/microsoft/UFO/blob/main/documents/docs/ufo2/app_agent/strategy.md) | screenshot의 구조화된 요소 생산 및 UIA/visual control 수집과 병합 뒤 LLM에 넘기는 단계가 공개되어 있다 | 이미지와 native 정보를 가진 대상 후보 생산은 구체적 구현 형태가 있다. source 종류와 원본 영역을 보존해야 한다 | GUI control 후보가 PDF의 모든 표, 그래프, 셀 범위를 포함한다는 주장은 하지 않는다. overlap만으로 같은 객체라고 확정하지 않는다. OmniParser의 helper 모델이나 UFO의 업무 실행을 VIA 안에 추가하지 않는다 |

Qwen의 공동 입력 예제는 A의 구현 가능성 참고, QuickSet/W3C와 화면 후보 생산 구현은 B의 구조 참고다. 어느 쪽도 필수 package로 선정하지 않았다. **B의 양방향 재판단과 version 계약은 위 자료를 그대로 복제한 것이 아니라, VIA의 페이지 이동, 정정 장면에 맞추어 추가한 설계**다.

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
