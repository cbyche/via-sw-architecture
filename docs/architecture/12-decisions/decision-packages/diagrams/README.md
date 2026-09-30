# 설계 후보 발표 그림 — 배경·비교 16페이지

[후보 문서·품질 비교표 목록](../README.md) · [16페이지 연속 보기](./review.html)

2560 × 1440, 16:9 벡터 그림이다. 각 SVG와 draw.io는 같은 scene에서 생성한다. SVG는 GitHub에서 바로 확인하고, `.drawio`는 diagrams.net 또는 draw.io 편집기에 열어 box·text·connector를 개별 수정할 수 있다. Raster 이미지를 draw.io 안에 넣은 파일이 아니다.

| 후보 | 배경 SVG / 편집 원본 | 설계 비교 SVG / 편집 원본 |
| --- | --- | --- |
| [요청 의미 해석](../request-interpretation-topology.md) | [SVG](./request-interpretation-topology-background.svg) · [draw.io](./request-interpretation-topology-background.drawio) | [SVG](./request-interpretation-topology-comparison.svg) · [draw.io](./request-interpretation-topology-comparison.drawio) |
| [Context 획득](../context-acquisition-strategy.md) | [SVG](./context-acquisition-strategy-background.svg) · [draw.io](./context-acquisition-strategy-background.drawio) | [SVG](./context-acquisition-strategy-comparison.svg) · [draw.io](./context-acquisition-strategy-comparison.drawio) |
| [Agent 상태 관측](../agent-state-observation.md) | [SVG](./agent-state-observation-background.svg) · [draw.io](./agent-state-observation-background.drawio) | [SVG](./agent-state-observation-comparison.svg) · [draw.io](./agent-state-observation-comparison.drawio) |
| [직접 응답 생성](../direct-response-generation.md) | [SVG](./direct-response-generation-background.svg) · [draw.io](./direct-response-generation-background.drawio) | [SVG](./direct-response-generation-comparison.svg) · [draw.io](./direct-response-generation-comparison.drawio) |
| [음성 입력 근거](../speech-evidence-source.md) | [SVG](./speech-evidence-source-background.svg) · [draw.io](./speech-evidence-source-background.drawio) | [SVG](./speech-evidence-source-comparison.svg) · [draw.io](./speech-evidence-source-comparison.drawio) |
| [대화 Context 유지](../conversation-context-maintenance.md) | [SVG](./conversation-context-maintenance-background.svg) · [draw.io](./conversation-context-maintenance-background.drawio) | [SVG](./conversation-context-maintenance-comparison.svg) · [draw.io](./conversation-context-maintenance-comparison.drawio) |
| [자료 표현·소비](../context-representation-pipeline.md) | [SVG](./context-representation-pipeline-background.svg) · [draw.io](./context-representation-pipeline-background.drawio) | [SVG](./context-representation-pipeline-comparison.svg) · [draw.io](./context-representation-pipeline-comparison.drawio) |
| [복구 상태 원본](../recovery-state-source.md) | [SVG](./recovery-state-source-background.svg) · [draw.io](./recovery-state-source-background.drawio) | [SVG](./recovery-state-source-comparison.svg) · [draw.io](./recovery-state-source-comparison.drawio) |

## 도식 읽는 법

- **검정:** 양안 공통 책임·모듈·정보·경로. **파랑:** 양안에서 각각 달라지는 모듈·상태·계약·의존 경로. 파랑은 우수함이나 최종 채택 표시가 아니다.
- **큰 실선 테두리:** 논리 책임 묶음. Component 내부 모듈·data record는 그 안에 표시한다. 모든 모듈 상자를 별도 process로 배치한다는 뜻이 아니다.
- **명시적으로 이름 붙인 점선 runtime 테두리:** 음성 근거 그림의 Speech Input Worker / Shared Inference Service 실행 경계. Model Access는 이를 사용하는 논리 Component다.
- **실선 화살표:** 주 처리·데이터 전달. **점선 화살표:** 보완·무효화·read/service 호출. 정확한 의미는 선의 label과 각 문서의 실행 계약을 함께 따른다. 실선/점선 자체를 sync/async 구분으로 해석하지 않는다.
- 연결선의 `read / receipt`, `query / snapshot` 같은 표기는 요청·반환 계약을 한 선으로 묶어 표현한다. 방향은 호출 시작점이며 응답도 같은 계약으로 되돌아온다. Callback·새 event 경로가 다르면 별도 선을 둔다.
- 같은 Component가 상단 I/O와 하단 소비 등에 반복되면 해당 책임을 확대해서 보여준 것이다. Instance·process·Omni weights가 복제된 것이 아니다. 내부 설명 이름은 후보 구체화를 위한 것이며 target 최상위 Component 목록 변경이 아니다.
- 하단 파란 실행 순서와 선택 근거·비용·반증은 구조 차이의 핵심이다. `∥`는 가능한 중첩이며 hardware 동시 실행이나 측정된 이득을 보장하지 않는다.
- 배경의 사건 흐름은 설명용 시나리오이며 실제 실행 trace·제품 screenshot·측정값이 아니다.

## 편집·검사

원본 scene은 [그림 생성기](../../../../../scripts/architecture/generate_decision_package_diagrams.py)에 있다. 저장소에 반영할 수정은 scene에 적용한 후 쌍을 재생성한다. 외부 편집기에서 수정한 draw.io만 덮어쓰면 SVG와 생성 검사가 어긋나므로 변경을 scene에도 반영해야 한다.

```bash
.venv/bin/python scripts/architecture/generate_decision_package_diagrams.py
.venv/bin/python scripts/architecture/generate_decision_package_diagrams.py --check
```

검사는 XML·ID·canvas 경계·수평/수직 routing·무관한 node 관통·생성물 일치를 확인한다. 글자 폭·label 겹침·화살촉과 가독성은 SVG를 실제 브라우저에서 렌더링하여 별도 확인한다. 2026-10-01 작성 때 16페이지를 로컬 Chrome으로 렌더링하고 실제 text bounding box와 시각 배치를 검토했다. 이것은 Architecture 구현·성능 측정이 아니다.

[review.html](./review.html)은 같은 폴더에서 로컬 브라우저로 열면 SVG를 순서대로 보여준다. GitHub는 HTML 파일을 실행하지 않으므로 GitHub에서는 위 SVG 링크나 각 상세 문서의 삽입 그림을 이용한다.
