# DP3 기존 B4 실측의 활용

2026-10-19 사용자 정정 반영. 검토본은 `VIA-DP1-DP4-reference-style-17slides-v4-windows.pptx`다.

## 정정

기존 pixel B4는 화면·발화 생산을 같은 공통 호출 한도 안에서 독립 병렬 실행한 실제 구성이다. 이를 현재 병렬 기본 B의 pixel 조건 실측으로 활용할 수 있다. v3의 11쪽에서 B를 모두 미측정으로 표시한 것은 지나치게 넓은 표현이었다. 미측정인 것은 정규화 출력 계약의 병렬 B다.

11쪽 A도 같은 pixel 계약의 기존 값으로 맞췄다. 정규화 A와 pixel B4를 섞지 않는다. 새 실행이나 원시 variant 이름의 변경 없이 표시 목적을 정정했다.

| 지표 | pixel A | pixel 병렬 B (원래 실행 ID B4) |
| --- | --- | --- |
| QA01 초 (관측/15) | 9.268 (12/15) | 13.881 (13/15) |
| QA02 정답/24 | 19/24 | 20/24 |
| QA03 초 (완료/12) | 12.559 (10/12) | 17.604 (11/12) |
| QA04 | 미측정 | 미측정 |
| QA05 확정 USD | 0.0339156 + 사용량 미확정 1회 | 0.0459164 |
| QA06 최고 동시 로컬 RSS MB | 41.452 | 42.762 |

A의 미확정 1회 예약 상한 $0.0729404는 실제 비용이 아니며 별도로 보존한다. 미확정 사용량을 0으로 두거나 전체 비용 증감을 확정하지 않는다.

양쪽 모두 첫 응답·완료 누락이 있다. 관측 평균만으로 전체 속도 승자를 만들지 않는다. QA02는 원래 최초 채택 집합과 revision의 고정 분모로 유지한다. 공통 endpoint의 관측과 사용자 목표의 완전한 의미 충족도 구별한다.

## 기존 조건과 한계

근거는 `results/architecture-evaluation/current/four-dp-presentation-20261019/dp04/comparison.md`, `comparison.json`, `plan.json`이다. 네 사례와 같은 pixel 계약·고정 모델·공통 slot cap·평가 정의를 사용한 48회 중 A와 B4 각각12episode를 표시했다. 당시 A는 BASIC, B4는 TACTIC phase였고 두 phase는 순서대로 실행했다. 따라서 같은 사례의 관측으로 활용하되, 새 동시 무작위 A/B 실행이나 순수 병렬화 단독 인과로 설명하지 않는다. 모델 반환 변동, 실패, 원본과 판본을 보존한다.

12쪽에서도 pixel 표를 A/B4로 맞추고, 정규화 A/당시 순차 B 비교는 별도 이력으로 남겼다. 9~10쪽 배경과 13쪽의 별도 pixel A2+A5 복구 설명은 유지한다. 정규화 병렬 B 또는 정규화 A+A2+A5의 새 실측은 없다.

## 산출과 확인

- 생성 소스: `scripts/presentations/revise_dp3_pixel_parallel_v4.mjs`.
- 패키징: `scripts/presentations/package_dp3_parallel_v3.py`를 재사용한다.
- `previews-v4/`에 최종 17장 렌더를 제공한다. 바뀐 11~12쪽을 실제 열람해 확인했다. 나머지 DP3는 v3와 같은 설명이며 DP1/DP2/DP4의 본문과 notes 보존 범위는 `dp3-v4-preservation.json`에 기록했다.
- `dp3-v4-validation.json`의 package/layout 및 재가져오기 검사가 통과했다. 실제 Windows PowerPoint 렌더와 새 CI, 모델 호출·benchmark 실행·QA 변경 실험은 하지 않았다.
