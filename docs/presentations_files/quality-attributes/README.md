# VIA 품질 요구사항 발표 파일

사용자의 2026-10-06 리뷰에 따라 10개 품질 속성을 단일 지표와 수치 목표로 정리했다. V-11~13은 발표에서 제외하고 남은 ID와 원본의 공동 순위를 유지한다. 목표는 근거 기반 제안이며 구현 또는 실측 결과가 아니다.

| 파일 | 용도 |
| --- | --- |
| [발표 문서](../quality-attributes.md) | 표, 목표 근거, 상세 측정 설계 |
| [VIA-quality-attributes.pptx](VIA-quality-attributes.pptx) | 16:9 편집 가능한 표 두 페이지 |
| [quality-attributes-01.png](quality-attributes-01.png) | 기능과 시간 요구사항의 최종 렌더링 |
| [quality-attributes-02.png](quality-attributes-02.png) | 변경, 메모리, 복구와 분석 요구사항의 최종 렌더링 |
| [index.html](index.html) | 두 페이지 미리보기 |
| [quality-attributes.json](quality-attributes.json) | 단일 목표, 문안, 원본 명칭과 근거 ID |
| [functional-coverage.json](functional-coverage.json) | 기능 모집단 94개 ID와 1880회 시험 규모 제안 |
| [measurement-design.md](measurement-design.md) | 상세 측정 설계의 편집 원본 |
| [verification.md](verification.md) | 제작 검사와 확인 범위 |
| [생성 스크립트](../../../scripts/presentations/generate_quality_attributes.mjs) | 문서와 native table 및 notes 생성 |
| [일치 검사](../../../scripts/presentations/check_quality_attributes.py) | 내용과 모집단 및 파일 구조 검사 |

## 재생성

Codex의 load_workspace_dependencies로 bundled Node, Python과 node_modules 경로를 확인하고 Presentations 스킬을 따른다. 새로운 임시 build 디렉터리에 생성 스크립트를 복사하고 그 디렉터리의 node_modules를 bundled node_modules에 연결한다. 기존 final 파일이 있는 build 디렉터리는 재사용하지 않는다.

환경 변수 RUNTIME_NODE_MODULES, VIA_PRESENTATION_SKILL_DIR, VIA_RUNTIME_PYTHON을 절대 경로로 지정한다. bundled Node로 복사한 mjs를 실행하고 첫 번째 인자로 저장소, 두 번째 인자로 새 build 디렉터리의 절대 경로를 전달한다. 상세 측정 설계는 measurement-design.md에서 수정한다. 생성 스크립트가 그 내용을 발표 문서와 notes에 함께 반영한다.

```sh
python3 scripts/presentations/check_quality_attributes.py
```

글꼴은 이 환경에서 확인한 Apple SD Gothic Neo다. 재생성 환경에서 다른 글꼴을 사용하면 생성 코드와 font policy를 함께 변경한다. PowerPoint 또는 Google Slides 앱에서 직접 편집, 저장 및 재열기는 확인하지 않았다.
