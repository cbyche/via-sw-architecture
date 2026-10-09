# VIA DP 41~45 수정 가능한 PowerPoint

2026-10-06의 배경과 보강된 설계 비교 페이지를 PowerPoint 기본 개체로 변환한 파일이다. 기존 draw.io, SVG와 PNG의 문구 및 배치를 유지한다.

| 파일 | 구성 |
| --- | --- |
| [통합 PPTX](./VIA-DP-background-and-comparison-41-45.pptx) | 발표용 11장. 본 발표 9장: 도입 → 41 배경·비교 → 44 배경·비교 → 45 배경·비교 → 42 배경·비교. 부록 2장: 43 배경·비교 |
| [배경 PPTX](./dp-background/VIA-DP-background-41-45.pptx) | 41~45 배경 5장. 번호순 참조 자료. 사용자 경험 도입은 미포함 |
| [설계 비교 PPTX](./dp-comparison/VIA-DP-comparison-41-45.pptx) | 번호순 참조 자료. 41~45 설계 비교, 장점, 단점과 QA Trade-off 5장 |

## 편집 방법

- 텍스트, Component와 Module 도형, 저장소, 표의 칸, 선 및 화살표는 개별 PowerPoint 개체다. 슬라이드 전체를 PNG로 삽입하지 않았다.
- 텍스트 상자를 선택하면 문구, 수치와 원형 점수 `●●○` 등을 수정할 수 있다. 기존 줄 배치를 유지하도록 여러 줄의 문구는 줄별 텍스트 상자로 나누었다.
- 선과 화살표는 원본 경로를 유지한 자유형 도형이다. 점 편집으로 경로를 바꿀 수 있으며, Component를 이동할 때 선도 함께 조정한다. 도형 이동에 자동으로 따라오는 연결선은 아니다.
- 슬라이드 크기는 16:9다. 서체는 원본과 같은 **Apple SD Gothic Neo**다. 이 서체가 없는 PC에서는 설치된 한글 서체로 교체한 뒤 줄 폭을 확인한다.
- 각 슬라이드의 발표자 노트에 관련 Architecture 문서와 SVG 원본 링크를 넣었다.

41~44 비교 페이지의 수치와 원형 점수는 사용자가 요청한 **형식 검토용 예상 예시**다. 실측 또는 대안 선정 결과가 아니다. 45는 2026-10-09에 조건별 정성 비교로 교체하고 공식 Reference와 한계를 발표 노트에 넣었다. 기존 A/B 및 43의 A/C 표기와 미선정 및 미측정 상태를 유지한다. [비교 자료 설명](./dp-comparison/README.md)의 조건과 해석 범위를 따른다.

2026-10-07 사용자 확인에 따라 비교 페이지와 발표자 노트의 V-04는 **평균 반응시간**, V-05는 **평균 VIA 처리시간**으로 품질속성 페이지와 통일했다. 기존 시간 수치는 평균 시간의 형식 예시로 유지하며 p95 실측값을 평균으로 환산한 결과가 아니다.

## 생성과 확인

생성기는 [generate_dp_editable_pptx.mjs](../../scripts/presentations/generate_dp_editable_pptx.mjs)다. [배경 생성기](../../scripts/architecture/generate_dp_background_slides.py)와 [비교 생성기](../../scripts/architecture/generate_dp_comparison_slides.py)의 장면을 읽고, 저장된 SVG와 일치하는지 확인한 뒤 PowerPoint 개체로 변환한다. PPTX를 직접 편집한 변경은 draw.io나 생성기에 자동 반영되지 않는다. 재생성 시에는 장면 원본에 수정 내용을 반영한다.

Codex의 번들 런타임과 Presentations 스킬이 있는 환경에서 다음 값을 실제 절대 경로로 지정한다.

```sh
export VIA_PRESENTATION_SKILL_DIR=/absolute/path/to/presentations/skills/presentations
export VIA_RUNTIME_PYTHON=/absolute/path/to/bundled/python3
export RUNTIME_NODE_MODULES=/absolute/path/to/bundled/node_modules
VIA_RUNTIME_NODE=/absolute/path/to/bundled/node
VIA_REPO_ROOT=/absolute/path/to/via-sw-architecture
VIA_PPTX_BUILD_DIR=$(mktemp -d /tmp/via-dp-pptx.XXXXXX)
ln -s "$RUNTIME_NODE_MODULES" "$VIA_PPTX_BUILD_DIR/node_modules"
cp "$VIA_REPO_ROOT/scripts/presentations/generate_dp_editable_pptx.mjs" "$VIA_PPTX_BUILD_DIR/"
"$VIA_RUNTIME_NODE" "$VIA_PPTX_BUILD_DIR/generate_dp_editable_pptx.mjs" "$VIA_REPO_ROOT" "$VIA_PPTX_BUILD_DIR"
```

생성 시 슬라이드 수, 16:9 크기, 텍스트의 원본 일치, 개체의 편집 가능 여부와 이미지 개체 미사용을 검사한다. PPTX를 다시 불러와 렌더링한 결과와 검사 기록은 임시 빌드 폴더에 저장한다. 현재 통합 산출물은 재가져오기 렌더 11장을 확인했다. 개별 DP 10장의 슬라이드 XML과 43의 노트는 도입 작업 시작본과 동일하다. Microsoft PowerPoint에서 직접 열어 확인한 결과는 아니다.

비교 페이지만 수정한 경우 생성기 마지막 인자에 `--comparison-only`를 추가하면 통합 11장과 비교 5장 PPTX만 갱신하고 배경 5장 PPTX는 유지한다.

## 사용자 경험 도입과 발표 흐름 — 2026-10-09

본 발표의 9장은 도입 → 41 → 44 → 45 → 42이며, 각 주제는 배경과 비교 두 장이다. 43 두 장은 부록에 그대로 보존한다. PowerPoint의 “본 발표”(9장)와 “부록 · 보충 자료”(2장) section으로도 구분한다. section은 [MS-PPTX의 표준 표현](https://learn.microsoft.com/en-us/openspecs/office_standards/ms-pptx/5bd8a237-8753-4d05-b757-c5e84e5d1ba6)을 사용한다. 본 발표에서 빠졌다는 이유로 43의 후보 자격이 취소된 것은 아니다. 설명 순서는 설계 선택 순서나 최종 DP 확정과 다르다.

도입은 기존 `dp-background-overview` 장면에서 생성한 편집 가능한 텍스트·선·구간으로 구성한다. PNG 한 장을 삽입한 슬라이드가 아니다. [도입과 대본](./dp-background/README.md)에 사례·책임·범례·설명 범위를 기록하고, 같은 45~60초 대본과 첫 주제로의 연결을 PPTX 첫 장 노트에 수록한다. 41→44, 44→45, 45→42 전환은 각각 비교 장의 기존 노트 뒤에 추가한다.

이번처럼 도입과 발표 흐름만 변경할 때는 생성기 마지막 인자 `--intro-flow-only`를 사용한다. 통합 PPTX만 갱신하며 기존 DP 슬라이드·노트·공통 package를 보존하고 overview, 세 전환 노트, 순서·section·추가 장의 관계와 slide count만 갱신한다. 배경/비교 전용 자료는 번호순 참조용으로 유지한다. 인자 없이 전체를 생성해도 통합 자료는 같은 11장 순서를 사용한다.

특정 DP의 갱신 대상은 **DP 번호 + 배경/비교 종류**로 선택한다. 실제 package의 scene 이름과 presentation relationship으로 찾아 바꿔 물리적인 `slideN.xml` 번호에 의존하지 않는다.

| 옵션 | 현재 통합 PPTX의 갱신 대상 | 번호순 참조 자료 |
| --- | --- | --- |
| `--intro-flow-only` | 1번 도입, 3·5·7번 전환 노트와 발표 순서 metadata | 갱신 없음 |
| `--dp41-42-only` | 3번 41 비교, 9번 42 비교 | 비교 1·2번 |
| `--dp44-only` | 5번 44 비교 | 비교 4번 |
| `--dp45-only` | 6번 45 배경, 7번 45 비교 | 배경/비교 각각 5번 |
| `--dp45-only --comparison-only` | 7번 45 비교 | 비교 5번 |
| `--comparison-only` | 3·5·7·9·11번 비교 | 비교 5장 |

`--list-selected`는 실제 파일을 바꾸지 않고 옵션별 scene과 표시 위치를 JSON으로 출력한다. `--list-selected --comparison-only`는 45 비교만 갱신하는 subset도 진단한다. 출력 파일을 아래 검사에 전달하면 물리적인 part 번호를 바꾼 후보를 사용하여 각 옵션이 지정한 DP의 도형·노트만 갱신하는지 확인한다.

```sh
python3 scripts/presentations/check_dp_pptx_flow.py --selection-json /tmp/selection-check.json
```

[package 보존 도구](../../scripts/presentations/dp_pptx_package.py)와 [순서·노트·범위 검사](../../scripts/presentations/check_dp_pptx_flow.py)는 생성기 경로를 보완한다. `--before /absolute/path/to/original.pptx`로 작업 시작본의 DP 슬라이드, 다른 노트와 공통 part 보존도 확인할 수 있다. 개별 41~45의 설계안·그림·비교 값, QA·참조 Architecture와 미선정·미측정 상태는 이번 도입 작업에서 변경하지 않는다.
