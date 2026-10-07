# VIA DP 41~45 수정 가능한 PowerPoint

2026-10-06의 배경과 보강된 설계 비교 페이지를 PowerPoint 기본 개체로 변환한 파일이다. 기존 draw.io, SVG와 PNG의 문구 및 배치를 유지한다.

| 파일 | 구성 |
| --- | --- |
| [통합 PPTX](./VIA-DP-background-and-comparison-41-45.pptx) | 10장. 41 배경 → 41 비교 → 42 배경 → 42 비교 → 43 배경 → 43 비교 → 44 배경 → 44 비교 → 45 배경 → 45 비교 |
| [배경 PPTX](./dp-background/VIA-DP-background-41-45.pptx) | 41~45 배경 5장. 전체 지도는 미포함 |
| [설계 비교 PPTX](./dp-comparison/VIA-DP-comparison-41-45.pptx) | 41~45 설계 비교, 장점, 단점과 QA Trade-off 5장 |

## 편집 방법

- 텍스트, Component와 Module 도형, 저장소, 표의 칸, 선 및 화살표는 개별 PowerPoint 개체다. 슬라이드 전체를 PNG로 삽입하지 않았다.
- 텍스트 상자를 선택하면 문구, 수치와 원형 점수 `●●○` 등을 수정할 수 있다. 기존 줄 배치를 유지하도록 여러 줄의 문구는 줄별 텍스트 상자로 나누었다.
- 선과 화살표는 원본 경로를 유지한 자유형 도형이다. 점 편집으로 경로를 바꿀 수 있으며, Component를 이동할 때 선도 함께 조정한다. 도형 이동에 자동으로 따라오는 연결선은 아니다.
- 슬라이드 크기는 16:9다. 서체는 원본과 같은 **Apple SD Gothic Neo**다. 이 서체가 없는 PC에서는 설치된 한글 서체로 교체한 뒤 줄 폭을 확인한다.
- 각 슬라이드의 발표자 노트에 관련 Architecture 문서와 SVG 원본 링크를 넣었다.

비교 페이지의 수치와 원형 점수는 사용자가 요청한 **형식 검토용 예상 예시**다. 실측 또는 대안 선정 결과가 아니다. 기존 A/B 및 43의 A/C 표기와 미선정 및 미측정 상태를 유지한다. [비교 자료 설명](./dp-comparison/README.md)의 조건과 해석 범위를 따른다.

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

생성 시 슬라이드 수, 16:9 크기, 텍스트의 원본 일치, 개체의 편집 가능 여부와 이미지 개체 미사용을 검사한다. PPTX를 다시 불러와 렌더링한 결과와 검사 기록은 임시 빌드 폴더에 저장한다. 이번 산출물은 재가져오기 렌더 10장을 육안 확인했다. Microsoft PowerPoint에서 직접 열어 확인한 결과는 아니다.
