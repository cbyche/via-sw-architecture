# Grounding Oracle 근거 — identity, geometry, timing 판정

> 작성일: 2026-09-21
> 상태: 11-B Grounding Test Case의 판정 근거. Architecture 후보 결과나 11-C score band가 아니다.

## 1. 원칙

Interaction Grounding은 가능한 경우 픽셀 근사보다 **UI/object identity와 event-time hit-test**를 우선한다.

웹/GUI pointer event 모델은 pointer coordinate와 event target을 제공하고, target은 해당 좌표에서의 hit testing으로 정해진다. Windows pointer/mouse 입력도 화면 좌표와 timestamp를 관측할 수 있다. 따라서 fixture가 object identity를 제공할 수 있는 경우 임의의 ±pixel tolerance보다 "해당 시각에 어떤 target을 가리켰는가"가 더 직접적인 oracle이다.

근거:
- W3C Pointer Events: https://www.w3.org/TR/pointerevents3/
- Microsoft PointerPoint.Timestamp: https://learn.microsoft.com/en-us/uwp/api/windows.ui.input.pointerpoint.timestamp
- Microsoft PointerPoint.Position: https://learn.microsoft.com/en-us/uwp/api/windows.ui.input.pointerpoint.position

## 2. 고정 판정 순서

### G-1. Object identity가 있는 경우

- expected target object ID 또는 selected object set과 실제 resolved referent를 비교한다.
- 동일 object를 맞혔다면 별도 pixel 허용오차를 적용하지 않는다.
- 복수 선택에서는 누락·추가 object가 있으면 FAIL이다.

### G-2. Pointer coordinate만 있고 target object가 있는 경우

- ground-truth event timestamp의 cursor hot spot을 사용한다.
- hot spot이 target bounding region 내부인지 확인한다.
- 겹치는 UI가 있으면 topmost hit-test target identity를 정답으로 사용한다.

### G-3. Object identity가 없는 raw region fallback

- ground-truth region과 predicted region의 IoU를 계산한다.
- **IoU >= 0.50**을 최소 overlap 조건으로 사용한다.
- 잘못된 추가 target을 포함하면 FAIL이다.
- IoU 0.50은 computer-vision evaluation에서 널리 쓰이는 최소 localization overlap 기준을 빌린 fallback이다. VIA의 UI grounding 품질 목표가 0.50이라는 뜻이 아니며 identity/hit-test보다 우선하지 않는다.

참고:
- COCO object detection evaluation은 IoU 0.50, 0.75 및 0.50:0.95 범위를 사용한다.
- https://cocodataset.org/#detection-eval

## 3. Timing — 작은 동시성 window를 강제하지 않는다

Speech와 pointing/gesture는 정확히 동시에 발생하지 않는다. Multimodal HCI 연구에서는 gesture/pen 입력이 speech보다 먼저 발생하는 sequential pattern이 흔하다.

Oviatt et al., CHI 1997의 multimodal temporal study에서는 sequential pen/speech input의 lag가 평균 약 1.4초였고, 70%가 2초 이내, 88%가 3초 이내, 관측된 100%가 4초 이내였다.

따라서 VIA 시험에서는 임의의 ±100ms/±500ms를 정답 기준으로 두지 않는다.

고정 규칙:

```text
candidate evidence window
= [User Turn 시작 - 4초, User Turn 종료]

oracle
= fixture의 실제 source event timestamp/order
  + deictic expression의 source timestamp
  + target identity
```

즉 4초는 "4초 안이면 모두 같은 지칭"이라는 matching tolerance가 아니라, **필요한 interaction evidence를 버리지 않기 위한 최소 history retention window**이다. 최종 target 판정은 event ordering과 identity로 한다.

근거:
- Oviatt et al., CHI 1997, Integration and Synchronization of Input Modes during Multimodal Human-Computer Interaction
- https://dl.acm.org/doi/10.1145/258549.258821
- 최근 gesture/speech timing 연구도 gesture onset/stroke가 lexical affiliate보다 선행하는 경향을 보고한다.

## 4. VIA Test Case에 적용

- TC-03.x: pre-selection / pointer / focus의 source timestamp와 identity를 사용
- TC-04.1~04.7: speech 중 multiple pointing, correction, drag, window transition의 event ordering을 사용
- TC-04.2: 첫 "여기"→chart-A, 둘째 "여기"→chart-B를 transcript arrival가 아니라 source timeline으로 판정
- TC-04.6: 철회된 para-A는 최종 referent에서 제외
- TC-04.7: 같은 좌표라도 display/window/document identity가 다르면 서로 다른 target

## 5. 한계

- 4초 값은 자연스러운 모든 사용자 행동의 상한을 보장하는 제품 SLA가 아니다.
- 실제 Windows 사용자 연구가 확보되면 history length sensitivity를 재검증할 수 있다.
- IoU 0.50은 object identity를 얻을 수 없는 synthetic fallback의 최소 criterion이다. 실제 UI Accessibility tree/object handle이 있으면 identity가 우선한다.
