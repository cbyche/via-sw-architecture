# Grounding Oracle 근거 — identity, geometry, timing 판정

> 작성일: 2026-09-21
> 상태: Grounding Test Case의 판정 근거. Architecture 후보 결과나 확정 score band가 아니다.

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

## 3. Timing — 고정 시간 tolerance를 두지 않는다

Speech와 pointing/gesture는 관련되어 있지만 정확히 동시에 발생한다고 가정할 수 없다.

1997년 Oviatt 연구는 multimodal HCI의 중요한 초기 근거이지만 pen+speech 및 당시 click-to-speak interface의 관찰값이므로, 그 연구에서 나온 최대 lag를 **현재 VIA의 4초 requirement로 직접 사용하는 것은 부적절하다.**

최근 연구도 핵심 방향은 재확인한다.

- 2023 JSLHR 연구는 referential/deictic gesture를 포함한 gesture stroke가 speech의 의미·prosodic prominence와 긴밀하게 정렬되는 경향을 보고한다.
- 2024 Frontiers 실험은 gesture stroke가 관련 speech와 **동시 또는 선행**하는 패턴이 일반적이며, 원래 위치보다 500ms 선행시킨 조건은 synchronized 조건과 유의한 차이가 없었던 반면 500ms 지연 조건은 처리에 불리할 수 있음을 보였다.

근거:
- Florit-Pons et al., 2023: https://doi.org/10.1044/2022_JSLHR-22-00451
- Nirme et al., 2024: https://doi.org/10.3389/fpsyg.2024.1345906
- Oviatt et al., 1997은 historical background로만 유지: https://dl.acm.org/doi/10.1145/258549.258821

하지만 이 최근 연구들도 desktop pointer/selection + speech 시스템에 적용할 **보편적인 ±N ms 또는 N초 threshold**를 제시하지 않는다.

따라서 VIA scoring에서는 시간 허용오차를 만들지 않는다.

    oracle
    = fixture의 source event timestamp/order
    + deictic expression의 source timestamp/order
    + target identity / hit-test result

Candidate는 제공된 timestamped interaction timeline에서 올바른 target을 찾아야 한다. transcript가 늦게 도착했다는 이유로 transcript-arrival 시각의 pointer snapshot을 정답으로 사용하면 FAIL이다.

### Interaction history retention

Retention horizon은 correctness threshold와 분리한다.

- 현재 canonical fixture의 pre-turn interaction은 최대 약 600ms 전에 발생한다.
- Test Case는 필요한 source event를 모두 candidate input timeline에 제공한다.
- 실제 제품의 ring-buffer retention 길이는 Architecture/implementation parameter이며 **이번 QA score의 고정 4초 requirement가 아니다.**
- 이후 더 긴 select-then-speak 간격을 robustness variant로 추가하려면 candidate 결과를 보기 전에 별도 rebaseline으로 고정한다.

즉 이번 평가가 검증하는 것은 “4초를 저장하는가”가 아니라 **필요한 timestamped evidence를 잃지 않고 올바른 speech/deictic event와 연계하는가**이다.

## 4. VIA Test Case에 적용

- TC-03.x: pre-selection / pointer / focus의 source timestamp와 identity를 사용
- TC-04.1~04.7: speech 중 multiple pointing, correction, drag, window transition의 event ordering을 사용
- TC-04.2: 첫 "여기"→chart-A, 둘째 "여기"→chart-B를 transcript arrival가 아니라 source timeline으로 판정
- TC-04.6: 철회된 para-A는 최종 referent에서 제외
- TC-04.7: 같은 좌표라도 display/window/document identity가 다르면 서로 다른 target

## 5. 한계

- 현재 연구 문헌에서 VIA에 그대로 적용할 수 있는 보편적인 temporal threshold는 확인하지 못했다.
- retention horizon과 referent matching correctness를 같은 숫자로 취급하지 않는다.
- IoU 0.50은 object identity를 얻을 수 없는 synthetic fallback의 최소 criterion이다. 실제 UI Accessibility tree/object handle이 있으면 identity가 우선한다.
