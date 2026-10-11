# DP 비교 발표자료 v4

v3의 구조 비교 그림 아래에 **원형 점수, 가정값, 짧은 이유**를 함께 표시하고, 각 비교 장 다음에 **여섯 ASR-QA의 상세 비교 장**을 추가한 발표자료다. 참고 이미지의 `●●◐ 5점 99.63% [오류 2개 가정]` 형식을 따른다. 원형 기호는 PPT의 편집 가능한 도형이고 값과 이유도 편집 가능한 텍스트다.

## 파일과 순서

| 파일 | 구성 |
| --- | --- |
| [배경과 비교 통합본](../../VIA-DP-background-and-comparison-41-45_v4.pptx) | 16장. 기존 배경 유지, 41 → 44 → 45 → 42의 비교 장 뒤에 QA 상세 장 1장씩 추가, 기존 43 부록 유지 |
| [비교 전용본](../VIA-DP-comparison-41-45_v4.pptx) | 9장. 41 → 44 → 45 → 42의 구조 비교와 QA 상세 비교, 기존 43 부록 유지 |

두 v3 파일은 그대로 보존한다. 구조 그림, 입력 데이터 예시, Module 연결, 실행 순서도와 공통 조건도 유지한다. 이번 변경은 네 비교 장의 QA 영역과 추가 상세 장에 한정한다.

## 수치의 상태

**사고 실험 가정값이며 실제 측정 결과나 대안 선정 결과가 아니다.** QA 조정 작업과 독립적으로 발표자료의 표시 형식을 먼저 완성한 버전이다. 현재 원지표, 목표, 점수 경계는 조정 전 **ASR-QA-v5** 기준이다.

- [QA 기준 원본의 고정 버전](https://github.com/cbyche/via-sw-architecture/blob/4af3e1973c599ca1904de2df8c13bc9b4481bcaf/docs/architecture/12-decisions/decision-packages/03-02-quality-attribute-definitions.md)
- [가정값과 QA 스냅샷](qa-thought-experiment.json): 원값, 목표, 보호 조건, 점수 경계, 차이 판단 기준, A/B 이유와 비용 가정

정확성의 오류 2개와 8개, 처리시간, 호출 단가, 변경 단위 수와 메모리는 모두 가정이다. 예를 들어 비용의 공통 부분은 `$0.3787 + 200 × $0.0024`이고, 41A의 추가 재판단 비용과 45A의 cache miss 비용을 더한 가정은 `$1.1867`이다. 실제 모델 가격이나 구현에서 얻은 결과로 해석하지 않는다.

44와 42는 동일한 의미 판단을 사용한다는 가정이므로 정확성을 같은 값으로 표시한다. 같은 점수는 그대로 유지한다. 원값 차이와 등급 차이를 구분하며, QA별 점수를 합산하지 않는다. 상세 장의 오른쪽에는 원값 차이와 v5의 차이 판단 기준을 함께 표시한다. 보호 조건은 별도 확인 대상이다.

## 표시 형식

| 위치 | 내용 |
| --- | --- |
| 구조 비교 장 하단 | 여섯 QA 이름, A/B 원형 점수와 숫자 점수, 단위를 포함한 가정값, 짧은 이유 |
| 바로 다음 QA 상세 장 | 여섯 QA의 목표, A/B 점수와 가정값, 차이를 만든 이유, 원값 차이와 판단 기준 |
| 상세 장 하단 | 차이가 발생하는 조건과 차이가 축소되거나 반전되는 조건 |
| 발표자 노트 | 가정의 범위, 원값, 점수와 이유, 기준 자료 경로 |

세 개의 원으로 0–6점을 표현한다. 빈 원은 0점, 반원은 1점, 채운 원은 2점이다. 점수는 반올림 전 원값으로 계산한다.

| 점수 | 표시 |
| --- | --- |
| 0 | ○○○ |
| 1 | ◐○○ |
| 2 | ●○○ |
| 3 | ●◐○ |
| 4 | ●●○ |
| 5 | ●●◐ |
| 6 | ●●● |

## QA 조정 후 교체

1. 확정된 QA의 정의, 측정 모집단, 지표, 목표, 보호 조건, 점수 경계와 차이 기준을 확인한다.
2. `qa-thought-experiment.json`의 `qa_snapshot`, `difference_thresholds`, 가정과 각 DP의 여섯 행을 함께 갱신한다. `display_target`은 상세 장의 짧은 목표 표기다. 원본 경로, Git commit과 해시도 해당 버전으로 변경한다.
3. 같은 테스트 모집단으로 사고 실험을 다시 계산한다. 지표나 모집단이 달라졌으면 기존 수치에 새 점수 경계만 적용하지 않는다. 비용에는 배경 생산, 재시도와 폐기 결과도 포함한다.
4. 변경된 수치가 A/B 이유와 선택 조건에도 맞는지 확인한다. 측정 전에는 `THOUGHT_EXPERIMENT_LAYOUT_DRAFT_NOT_MEASURED` 상태를 유지한다.
5. 새 임시 빌드 폴더에서 두 PPTX를 다시 생성하고 미리보기를 갱신한다. v3는 생성 입력으로만 읽는다.

공식 QA 문서와 Architecture 원본은 이 생성 과정에서 수정하지 않는다.

## 생성

`load_workspace_dependencies`에서 받은 Node, Python과 Node 모듈 경로를 사용한다. `VIA_PRESENTATION_SKILL_DIR`은 Presentations skill의 절대 경로다. `finalizePresentation`은 기존 최종 파일을 덮어쓰지 않으므로 실행마다 새 임시 빌드 폴더를 사용한다.

```sh
VIA_QA_BUILD=$(mktemp -d /private/tmp/via-dp-v4-qa.XXXXXX)
ln -s "$RUNTIME_NODE_MODULES" "$VIA_QA_BUILD/node_modules"
cp scripts/presentations/generate_dp_v4_qa.mjs "$VIA_QA_BUILD/author.mjs"
"$VIA_RUNTIME_NODE" "$VIA_QA_BUILD/author.mjs" "$PWD" "$VIA_QA_BUILD"
```

실행 환경에는 `RUNTIME_NODE_MODULES`, `VIA_PRESENTATION_SKILL_DIR`, `VIA_RUNTIME_PYTHON`, `VIA_RUNTIME_NODE`의 절대 경로를 지정한다.

생성 구성:

- `scripts/presentations/build_dp_v4_qa_scenes.py`: JSON 원값을 점수로 환산하고 요약과 상세 장의 배치 생성
- `scripts/presentations/generate_dp_v4_qa.mjs`: native PPT 도형과 텍스트 생성, 검증, 렌더링과 두 v4 파일 저장
- `scripts/presentations/dp_v4_qa_package.py`: v3의 QA 영역만 교체하고 상세 장 삽입. 나머지 기존 객체와 package part의 보존 확인

## 미리보기

| DP | 구조 비교 | QA 상세 비교 |
| --- | --- | --- |
| 41 | [구조와 요약](dp41-comparison_v4.png) | [여섯 QA 상세](dp41-qa-detail_v4.png) |
| 44 | [구조와 요약](dp44-comparison_v4.png) | [여섯 QA 상세](dp44-qa-detail_v4.png) |
| 45 | [구조와 요약](dp45-comparison_v4.png) | [여섯 QA 상세](dp45-qa-detail_v4.png) |
| 42 | [구조와 요약](dp42-comparison_v4.png) | [여섯 QA 상세](dp42-qa-detail_v4.png) |
