# 독립 검토 2차 — 리소스·변경 수정 확인

> 검토일: 2026-10-14 / 검토자: 1차와 같은 독립 resource/change/scoring reviewer
> 대상: [1차 발견 사항](./round1-resources.md)의 수정과 QA07 집계 명확화 및 신규 정적 검사

## 1. 결론

**1차 P2 두 건은 모두 해결됐다. 이번 재검토 범위에서 미해결 P1/P2는 없다.** QA07 집계 순서의 낮은 우선순위 보완도 반영됐다. 이는 문서의 정의·요약·점수 일관성 확인이며 실제 품질 달성이나 DP 우열 검증이 아니다.

| 항목 | 재검토 결과 |
| --- | --- |
| RES-01: README QA09 시작점 | **해결.** README가 `응답 내용 채택 후 재생 가능한 시간`으로 바뀌었다. 상세 QA09의 `t_adopted`와 일치하며 UI 표시 지연을 제외하는 잘못된 해석이 제거됐다. |
| RES-02: README QA08 × DP44 | **해결.** 표가 `=`로 바뀌어 상세안의 실용적 동점 예상과 일치한다. 이름도 `병행 수정 중복`으로 통일됐다. |
| QA07 집계 순서 | **반영.** 4개 사례×3개 겹침 시점의 12개 cell을 명시한다. 각 조건의 20회 평균 차이에 0 하한을 적용한 뒤 12개 cell을 같은 비중으로 평균낸다. 세 시점을 먼저 합치는 방법과 반복별 잘라내기를 명시적으로 배제했다. |

## 2. 정적 검사 재실행

검토한 스크립트: [check_qa_candidate_package.py](../../../../../../scripts/architecture/check_qa_candidate_package.py)

실행 명령:

```text
python3 scripts/architecture/check_qa_candidate_package.py
```

검토 시 출력:

```text
PASS: 9 QA specs, 237 score probes, 56 local file links, illustrative arithmetic.
Scope: score intervals/glyphs/monotonicity, local file existence and examples only.
Not checked: external URLs, heading anchors, empirical targets, DP superiority or PoC results.
```

### 검사 코드와 현재 표의 적합성

- 아홉 상세 명세의 0~6점 행을 읽으며 원형 표기를 점수와 대조한다. 각 QA마다 일곱 점수가 정확히 한 번 존재해야 한다.
- 현재 표의 단일 부등식·연쇄 부등식·등식과 소수·분수·단위 표기를 처리한다. `Fraction`을 사용해 1/6이나 소수 경계에서 이진 부동소수점 오차를 피한다.
- 각 경계와 바로 양옆 및 중간값에서 정확히 한 구간에 속하는지 검사한다. 현재 구간은 단순한 연속 interval이므로 해당 검사는 현재 표의 공백·겹침을 찾는 데 적합하다.
- 비용·시간·메모리·개수 평균은 비음수 영역에서 검사하며 정확도는 **0~100% 유효 영역**에서 검사한다. 100% 초과를 여덟 번째 점수로 요구하지 않는 처리가 맞다.
- 정확도는 값 증가에 따라 점수가 증가하고 다른 QA는 감소하는지 확인한다. 현재 표의 방향과 일치한다.
- 실제 표와 가상 예시의 값도 다시 읽었다. 이전 리뷰에서 확인한 QA04~09의 예시 채점과 충돌하는 변경은 없다.

### 정적 검사가 보장하지 않는 것

이 스크립트는 측정기나 범용 점수 API가 아니다. 음수·NaN·100% 초과 같은 잘못된 입력의 런타임 거부, `+∞` 실패 판정, 미완료 상태 및 기능 qualification은 실행하지 않는다. 현재 문서의 실패 규칙은 별도로 읽어 확인했다. 새로운 수식 문법을 허용하게 바꾸면 parser도 다시 검토해야 한다.

산술 assertion은 몇 가지 고정 가상 계산을 확인한다. 본문 모든 숫자를 자동 추출하여 예시 채점을 모두 재현하는 검사는 아니다. 링크 확인 역시 **로컬 파일의 존재**만 확인하며 heading anchor·외부 URL·인용 내용의 타당성은 확인하지 않는다. 이 한계가 스크립트 출력에 드러나므로 검사 결과를 과장하지 않는 한 현재 문서 검증용으로 적합하다.

## 3. 남아 있는 실행 전 확인

1차 보고서의 구현·제품 조건 확인은 그대로 남는다. 실제 Rust 모듈 registry와 최소 patch, provider usage·단가 snapshot, Windows 공유 allocation 및 audio loopback을 후속 PoC에서 확인해야 한다. 목표의 제품 승인과 DP별 실용 차이는 문서 정합성 검사로 대체할 수 없다.

그 항목은 이번 수정 미완료 결함이 아니라 명시된 후속 검증 범위다. 이번 재검토에서는 추가 구현·모델 호출·Windows 실측을 수행하지 않았다.
