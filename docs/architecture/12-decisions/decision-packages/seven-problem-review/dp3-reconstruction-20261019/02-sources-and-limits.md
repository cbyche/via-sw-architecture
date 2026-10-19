# 근거와 설계 추론의 경계

2026-10-19 / 기존 로컬 자료 재독과 공식 원문 조회 / 외부 결과의 VIA 점수 전용 없음

## 1. 제품 요구와 현재 QA

| 로컬 자료 | 이번에 사용하는 범위 |
| --- | --- |
| [System Mission](../../../../01-system-mission-and-boundary.md)·[고정 Scope](../../../../03-fixed-architecture-scope.md) | VIA의 연속 Voice/Text/화면 interaction과 Agent 실행 책임의 경계 |
| [대표 UC](../../../../05-representative-use-cases.md) | UC-03·04의 실제 지정 행동과 사전 선택·페이지 이동·전사 수정, UC-11·14의 입력·처리 겹침 |
| [Core ASR](../../../../08-quality-attributes/core-asr-contract.md) | 기존 QA-09/19/29/39의 정의와 상태 보존. 발표 QA01~06과 혼합하지 않음 |
| [QA 후보 패키지](../../qa-candidate-package/README.md) | 발표 여섯 QA의 단일 의미와 우선순위. 수치·목표·점수 수정 없음 |
| [공유 Omni](../../../target-architecture/shared-omni-runtime.md) | 동일 weights의 논리 역할/session 분리, 동시 입력 보장과 bounded 자원. 참조 구조를 새 비교의 강제 정답으로 두지 않음 |
| [기존 화면 지칭](../04-screen-speech-grounding.md) §2, §4~8, §11 | 기본 기능과 강한 B, A3/B3 증분 보완의 선행 제안, 같은 source와 비용 원칙 |
| [기존 공식 source 원장](../../../../../references/seven-problem-review/screen-grounding.md) | 종전 A/B의 provenance와 Windows/native/typed 결합의 지원 한계 |

현재 workplan, 단계1 문제 목록, 참조 Architecture의 소유권·제어·기억 계약과 현재 QA 논의도 재독했다. 새 재검토는 기존 기능의 선택을 다시 탐색하는 것이며 다른 채팅의 미커밋 변경과 측정 캠페인을 수정하지 않는다.

## 2. 이번에 직접 조회한 공식 근거

### 2.1 증분 처리의 실행과 상태

[Schlangen & Skantze, 2011, A General, Abstract Model of Incremental Dialogue Processing](https://aclanthology.org/2011.dnd-2.11.pdf)의 도입·동기와 increment 처리 틀을 확인했다. 입력이 완성되기 전 처리를 시작하는 구조, 처리 단위와 정보 흐름의 차이를 다룬다. 한 architecture가 항상 최상이라고 결론내리는 논문이 아니다.

**VIA 추론:** 입력 종료 후 남는 대기를 줄일 가능성을 탐색할 수 있다. VIA의 모델/API 지원, 최종 검증 비용, 정확도와 시간 수치를 논문이 입증한 것은 아니다. 이 설계 분석은 기존 A3/B3에서 이미 언급한 축을 명시적으로 재검토한다.

### 2.2 가설의 추가·철회와 의미 의존

[Kennington et al., 2014, InproTKs](https://aclanthology.org/W14-4312.pdf) §2를 확인했다. incremental unit의 연결과 added/revoked/committed 상태를 설명한다. 같은 논문은 당시 sensor listener가 raw 단위에 모든 grounded-in 관계와 철회를 자동으로 제공하지는 않는다고 명시한다.

**VIA 추론:** 지칭 결과가 어떤 발화·근거에 의존하는지와 무엇을 폐기할지 host에서 설계해야 한다. 논문의 commit을 Request Controller의 영속 채택이나 Agent 실행 권한과 동일시하지 않는다. 작은 모델의 QA02 개선 근거로 쓰지 않는다.

### 2.3 여러 입력 stream의 원래 시각과 기다림

[Microsoft Psi, Stream Fusion and Merging](https://github.com/microsoft/psi/wiki/Stream-Fusion-and-Merging) §2.3~2.4와 §4를 확인했다. 원래 시각과 도착 순서를 구분하고, 현재 도착한 자료를 쓰는 fusion과 충분한 자료를 기다리는 fusion의 latency/reproducibility 차이를 설명한다.

**VIA 추론:** 늦은 화면/선택/전사가 있을 때 buffer와 watermark/gap 경계를 명시해야 한다. 해당 source의 ‘correct’는 시간 정렬 규칙에 대한 것이며 사용자의 지칭 의도가 맞다는 뜻이 아니다. 불완전한 최신 화면을 빨리 채택하는 약한 대안을 만들지 않는다.

### 2.4 실제 Windows frame의 보존과 시간

[Microsoft Windows Screen capture](https://learn.microsoft.com/en-us/windows/apps/develop/media-authoring-processing/screen-capture)의 frame 획득·수명 항목을 확인했다. frame의 SystemRelativeTime과 ContentSize, pool 반납 뒤 surface 참조를 보존하지 말라는 제약이 있다.

**VIA 추론:** 과거 화면을 다시 판단하려면 필요한 데이터를 실제로 보존해야 한다. 화면이 사라져도 live handle을 나중에 읽으면 된다고 가정하지 않는다. Windows/API/app 실험을 이번에 실행했다는 뜻은 아니다.

### 2.5 native 조회의 묶음 읽기와 현재성

[Microsoft UI Automation caching](https://learn.microsoft.com/en-us/windows/win32/winauto/uiauto-cachingforclients)을 확인했다. 여러 속성의 bulk fetching과 snapshot 갱신 책임을 설명한다. 캐시는 UI 변화가 없을 때만 그대로 유효하며 새 snapshot은 기존 참조를 자동 갱신하지 않는다.

**VIA 추론:** 실제 native 자료의 획득 비용과 source revision을 비교에 포함한다. native 조회가 가능한 모든 장면을 B만의 장점으로 주지 않는다. 이 API는 모든 PDF 객체나 그래프 내부 범위의 노출을 보장하지 않는다.

## 3. 기존 측정에서 보존하는 해석

- [pixel 정본](../../../../../../results/architecture-evaluation/current/four-dp-presentation-20261019/dp04/comparison.md): A와 B4의 QA02는 19/24와 20/24이며 양쪽 응답·완료 누락을 유지한다. A 비용 미확정 1회도 보존한다.
- [정규화 정본](../../../../../../results/architecture-evaluation/current/four-dp-presentation-20261019/dp04/normalized-comparison.md): A와 당시 순차 B의 24/24와 18/24다. 초기 ID가 맞아도 공간 근거가 지시점을 포함하지 못한 B dense 실패를 유지한다.
- 순차 B와 병렬 B의 의미 경로가 같다는 조건에서 정확도를 같은 수준으로 예상할 수 있다. 하지만 실행하지 않은 정규화 B4의 관측값을 18/24로 작성하지 않는다.
- 종전 A/B의 결과는 새 증분 구조·질의 분해·선택 보존 구조의 측정치가 아니다.

이번 자료는 근거와 구조적 추론을 정리한 것이다. 구현, 모델 호출, 측정 freeze·재실행, 새 QA/점수와 ADR 채택 또는 외부 발행을 수행하지 않았다.
