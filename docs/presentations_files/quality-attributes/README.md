# VIA 품질 요구사항 발표 페이지

원문과 측정 경계는 [발표 문안](../quality-attributes.md)에 있다. V-01~13을 빠짐없이 유지하고 1~5순위 8개, 공동 6순위 5개를 두 장으로 나눴다. 각 행의 설명은 한 문장이며 목표와 측정 초안은 별도 줄이다.

| 파일 | 용도 |
| --- | --- |
| [VIA-quality-attributes.pptx](VIA-quality-attributes.pptx) | 16:9, 편집 가능한 표 2개 |
| [quality-attributes-01.png](quality-attributes-01.png) | 최종 PPTX 1페이지 렌더링 |
| [quality-attributes-02.png](quality-attributes-02.png) | 최종 PPTX 2페이지 렌더링 |
| [index.html](index.html) | 두 페이지 미리보기 |
| [quality-attributes.json](quality-attributes.json) | ID·순위·문안·목표·측정·경계의 공통 데이터 |
| [verification.md](verification.md) | 내용·구조·렌더링 검사 결과와 확인 범위 |
| [생성 스크립트](../../../scripts/presentations/generate_quality_attributes.mjs) | Markdown·native table·notes·PNG 생성 |
| [일치 검사](../../../scripts/presentations/check_quality_attributes.py) | V 원문과 MD/PPT/notes/링크 대조 |

목표 숫자는 아직 확정되지 않았다. 표의 미정은 목표 누락을 숨기지 않기 위한 표시이며 0건은 기존 금지 행위 조건이다. 측정 절차도 발표용 초안이므로 실제 측정 전에 모든 DP에 공통인 계약으로 확정해야 한다.

## 재생성

Codex의 `load_workspace_dependencies`로 bundled Node, Python, node_modules 경로를 확인하고 Presentations 스킬의 생성 및 검증 절차를 따른다. 별도의 새 임시 build 디렉터리에 스크립트를 복사하고 해당 디렉터리의 `node_modules`를 bundled node_modules에 연결한다. 기존 final 파일이 있는 build 디렉터리는 재사용하지 않는다.

필요 환경 변수는 `RUNTIME_NODE_MODULES`, `VIA_PRESENTATION_SKILL_DIR`, `VIA_RUNTIME_PYTHON`이며 모두 절대 경로다. bundled Node로 복사한 `.mjs`를 실행하고 첫 번째 인자로 저장소 절대 경로, 두 번째 인자로 새 build 디렉터리 절대 경로를 전달한다. 생성 후 최종 두 PNG를 각각 확인하고 다음 검사를 실행한다.

```sh
python3 scripts/presentations/check_quality_attributes.py
```

글꼴은 이 제작 환경에서 확인한 `Apple SD Gothic Neo`다. 다른 OS에서 재생성할 때는 사용 가능한 한국어 글꼴을 확인하고 생성 스크립트와 font policy를 함께 수정한다. PowerPoint의 실제 앱 편집·저장·재열기나 Google Slides import 호환성은 확인하지 않았다.
