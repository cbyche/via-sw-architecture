# 화면과 발화 지칭: 2026-10-17 설계 권고 근거 재확인

2026-10-17 / 공식 코드와 원 논문 web 재확인 / 설치, 제품 실행과 모델 실행 없음

[권고 본문 §13](../../architecture/12-decisions/decision-packages/qa-first-exploration/five-problem-study/03-screen-speech-grounding.md#13-2026-10-17-설계-권고) / [기존 상세 source 원장](./screen-speech-grounding.md)

## 1. 이번 확인의 목적과 결론

기본 A 원본 공동 해석과 기본 B 생산자 후보 결합 중 어떤 안을 현재 VIA에 추천할지 검토했다. 2026-10-16에 고정한 동일 commit과 원 논문을 2026-10-17에 web로 다시 열었다. 최신 HEAD를 새로 고정하거나 제품의 전체 기능을 검증한 것은 아니다.

**설계는 A를 추천하지만 현재 A/B는 조건부 DP 후보다.** 원본을 공동 공급하는 코드와 별도 후보를 결합하는 실제 선례는 확인된다. 그러나 같은 cloud 모델을 쓰는 VIA에서 B의 작은 결합 입력이 전체 모델 비용 감소나 정확도 우위로 이어진다는 근거는 이번 확인으로 확보되지 않았다. B 초기 producer의 순차 실행을 불리한 고정 조건으로 삼아 A를 추천하지도 않는다.

## 2. 재확인한 공식 자료

### GRC-01. WebSurfer: native 목록과 image를 같은 판단에 공급

- [고정 코드](https://github.com/microsoft/autogen/blob/027ecf0a379bcc1d09956d46d12d44a3ad9cee14/python/packages/autogen-ext/src/autogen_ext/agents/web_surfer/_multimodal_web_surfer.py), `microsoft/autogen`, commit `027ecf0a379bcc1d09956d46d12d44a3ad9cee14`.
- `_generate_reply`에서 interactive rectangles, viewport와 screenshot을 읽고 Set-of-Mark를 구성한다. vision 경로는 target 설명과 resize한 image를 같은 message에 넣고 사용자 요청과 함께 model client에 전달한다.
- 확인 범위는 공동 입력이라는 실행 구조다. DOM target과 image를 같이 읽는다는 사실은 VIA의 임의 visual extent 생산, 발화 시점 보존이나 부분 정정 성공을 입증하지 않는다. 이 코드의 tool target 제약을 VIA A의 지칭 표현 범위로 그대로 옮기지도 않는다.
- 추천에 쓰는 추론: A를 원본만 무제한 넣는 구조로 만들 필요가 없고 native 단서도 함께 쓸 수 있다. 한 발화의 표현 수마다 동일 image를 반드시 재전송해야 한다는 근거는 없다. 이는 비용 비교의 공정 조건이며 측정된 A 우위가 아니다.

### GRC-02. Google Live Console: audio와 image를 같은 client에 공급

- [고정 코드](https://github.com/google-gemini/live-api-web-console/blob/0a4542fe0e39d07956ea7af5de45d7c81fde8960/src/components/control-tray/ControlTray.tsx), commit `0a4542fe0e39d07956ea7af5de45d7c81fde8960`.
- `ControlTray`의 audio handler는 PCM을 보내고, `sendVideoFrame`은 canvas에서 JPEG를 만들어 같은 client의 `sendRealtimeInput`으로 보낸다.
- 별도 의미 후보를 소프트웨어 필수 교환 자료로 만들지 않고 media를 함께 공급하는 선례다. 샘플의 축소 비율이나 frame 간격을 VIA에 채택하지 않는다. pointer의 의미, source revision, 당시 근거 재조회와 채택 취소는 이 경로가 보장하지 않는다.
- 이번 기본 비교는 양안 모두 완성 입력 후 판단한다. 이 streaming 코드가 기본 A의 조기 판단이나 A3만의 이익을 입증한다고 쓰지 않는다.

### GRC-03. QuickSet: 부분 의미 후보와 제약 결합

- Johnston 외, [Unification-based Multimodal Integration](https://aclanthology.org/P97-1036.pdf), 1997, §3, Figure 6, pp.284–286 재확인.
- 언어 처리가 speech 문자열을 typed feature structures로 만들고, gesture 처리도 경쟁 해석을 같은 형식으로 생산한다. 결합자는 호환 해석을 찾고 순위를 정해 완성 명령을 bridge에 전달한다. 부분 값과 point / line 같은 유형 제약을 이용한다.
- 후보 생산과 최종 결합을 구별하는 B의 실용적 선례다. 논문은 제한된 지도 / pen 명령을 다루므로 VIA의 열린 desktop 대상, click 없는 pointer와 한국어 수정에서 동일 성능을 보장하지 않는다.
- 추천에 쓰는 추론: 충분한 후보와 관계 제약을 갖춘 경우 작은 표현을 조합하는 B의 이익은 타당한 가설이다. 다만 한 번의 공동 raw 입력보다 총 모델 비용이 작다는 실험은 아니다. 과거 구현의 시간 window와 확률 계산, 실제 처리 순서를 VIA의 강제 정책으로 복사하지 않는다.

### GRC-04. OmniParser: 별도 화면 생산 단계의 실제 API

- [고정 코드](https://github.com/microsoft/OmniParser/blob/354021201345a96178360b28733573e27269f2de/omnitool/omniparserserver/omniparserserver.py), commit `354021201345a96178360b28733573e27269f2de`.
- `/parse/`는 `base64_image`를 받고 parser를 호출하여 `som_image_base64`, `parsed_content_list`와 latency를 반환한다. 별도 화면 해석 단계와 다음 소비자가 읽을 결과 목록을 실제 interface로 둘 수 있다.
- 이 API가 음성 결합, 시간 지칭, producer 재확장이나 revision 검증까지 수행하는 것은 아니다. 실제 parser의 detector / caption 모델을 VIA에 추가한다는 제안도 아니다.
- 추천에 쓰는 추론: B의 목록 입력만 보고 비용을 계산할 수 없다. 목록을 만든 원본 처리와 출력도 전체 비용이다. API가 반환하는 latency는 VIA 전체 지연이나 QA03 결과가 아니다.

### GRC-05. InproTKS: 증분 판단은 별도 보완 축

- Kennington 외, [InproTKS: A Toolkit for Incremental Situated Processing](https://aclanthology.org/W14-4312.pdf), 2014, §2와 §4 재확인.
- IU는 same-level / grounded-in 연결을 가지며 가설을 add / revoke / commit할 수 있다. 논문은 원격 ASR의 지연, chunk 단위 부분 결과와 거친 시각의 제한도 설명한다.
- A3/B3의 잠정 상태와 의존 갱신을 설계할 근거다. 원본의 시각 정확성이나 VIA 영속 채택을 자동 제공하지 않는다. A2의 구체적인 binding patch와 영향 범위 판단은 VIA 추가 설계이며 이 논문의 완료 구현이라고 주장하지 않는다.

## 3. 기본 B의 재사용 이익을 과장하지 않는 비용 경계

한 발화에 `m`개 표현이 있어도 A는 raw 관측과 발화를 한 공동 호출에 넣을 수 있다. 따라서 `A image 비용 = m × raw`, `B image 비용 = raw`라는 비교는 사용하지 않는다.

B는 같은 candidate read-set을 여러 표현과 결합 판단에 사용할 수 있다. 이 기본 재사용으로 작은 후보에서 관계를 판단하는 비용을 줄일 가능성은 있지만 화면 / 발화 후보 생산과 결합, 누락 시 확장을 모두 합산해야 한다. A도 이미 얻은 binding으로 충분한 검토에 raw 재전송을 강제하지 않는다. 어느 쪽이 실제로 원본을 다시 처리했는지로 계산한다.

초기 생산을 병행한 B4까지 보았을 때도 생산된 후보 이후 결합과 필요한 producer 재확장은 남는다. 그 경로가 A보다 느린지, 압축된 입력으로 더 짧은지는 실제 모델과 입력에 달려 있다. candidate가 명시적이라는 사실만으로 QA02나 QA04 우위를 부여하지 않는다.

## 4. 추천과 검증의 분리

- **지금 추천:** 열린 visual 범위와 언어 / 동작의 상호 의존을 현재 요청에서 직접 해결하는 A. 선택 후 A1 입력 crop/cache와 A2 부분 binding patch를 우선 검토한다.
- **현재 DP 판정:** 기능은 핵심이며 구조 차이도 실재한다. 다만 기본 B의 반대편 QA 이익이 충분히 지지되지 않아 주요 DP 확정은 보류하고 조건부 후보로 유지한다.
- **B로 바꿀 조건:** 같은 입력의 기본 비교에서 B의 정확도, 전체 처리량이나 다른 적용 QA의 이익이 추가 부담을 정당화하면 기본 추천을 변경한다. 그 뒤 보완 구성은 별도로 비교한다. 기본 B가 A1/A2까지 이겨야 한다는 조건을 두지 않으며 B1/B2의 효과도 기본 B에 넣지 않는다.
- **후속 범위:** 권고 본문의 12개 입력 episode와 6개 구성에 대한 bounded PoC 제안. 같은 원본 / 모델 / budget을 적용하고 기본 비교와 보완 비교를 분리한다. subset 비용을 전체 COMMON-28 QA05로 부르지 않는다.
- **미수행:** Windows 앱/API 검증, 입력 fixture 생성, PoC 구현, 모델 호출, 제품 테스트, 메모리 / 지연 / 비용 측정, commit와 push.
