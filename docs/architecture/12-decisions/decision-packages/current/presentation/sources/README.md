# 발표 생성 원본과 재현 범위

`v10-snapshot/`은 2026-10-22의 v9에서 v10 변환에 사용한 파일을 바이트 그대로 복사한 **고정 재현 스냅샷**이다. [이관 원장](../../presentation-migration.json)에 원본 경로와 양쪽 SHA가 있다. 이 스냅샷과 이전 저장소 경로를 두 개의 최신 편집 원본으로 유지하지 않는다.

## 들어 있는 자료

| 파일 | 역할 |
| --- | --- |
| [input-v9.pptx](v10-snapshot/input-v9.pptx) | 당시 36쪽 변환의 입력. 현재 발표 편집 시작점은 상위 폴더의 v10 |
| [build_reference_guide_v10.mjs](v10-snapshot/scripts/presentations/build_reference_guide_v10.mjs) | 11개 그림 영역, SVG, draw.io와 PNG 생성 및 PPTX export |
| [dp3_v8_scene.mjs](v10-snapshot/scripts/presentations/dp3_v8_scene.mjs) | 공통 scene과 검증 모델 |
| [legacy_dp_structure_scenes.mjs](v10-snapshot/scripts/presentations/legacy_dp_structure_scenes.mjs), [legacy_tactic_detail_fragments.mjs](v10-snapshot/scripts/presentations/legacy_tactic_detail_fragments.mjs), [dp5_v10_grounding_loops.mjs](v10-snapshot/scripts/presentations/dp5_v10_grounding_loops.mjs) | v10에서 수정한 구조와 보완 상세 영역 |
| [whole_deck_v10_scene_io.mjs](v10-snapshot/scripts/presentations/whole_deck_v10_scene_io.mjs) | native PPTX, SVG와 draw.io 표현 |
| [package_whole_deck_v10.py](v10-snapshot/scripts/presentations/package_whole_deck_v10.py) | 비대상 표, 개체, 노트와 package 보존 |
| [migrate_native_shape_labels.py](v10-snapshot/scripts/presentations/migrate_native_shape_labels.py), [whole_deck_v10_label_map.json](v10-snapshot/scripts/presentations/whole_deck_v10_label_map.json) | 확인한 라벨을 해당 도형의 text body로 이관 |
| [check_whole_deck_v10.py](v10-snapshot/scripts/presentations/check_whole_deck_v10.py) | 당시 v9/v10 변환의 독립 구조 검사 |
| [full_presentation_package.py](v10-snapshot/scripts/presentations/full_presentation_package.py), [presentation_windows_policy.py](v10-snapshot/scripts/presentations/presentation_windows_policy.py) | 위 스크립트가 직접 사용하는 package와 정책 helper |

v10 결과의 현재 편집 가능한 그림 및 scene JSON은 [design-review](../../design-review/README.md)에 한 벌로 관리한다. DP3와 DP44의 기존 슬라이드는 v9 입력 package에서 보존한 것이므로 이 11개 scene 생성기가 전체 36쪽의 모든 작성 코드를 대체하지 않는다. 해당 부분의 차기 변경은 각 DP 폴더의 설계와 그림 원본부터 확인한다.

## 역사적 v10 변환의 재현 준비

원본 스크립트의 당시 runtime 절대 경로는 스냅샷 안에서 보존한다. [prepare_v10_reproduction.py](prepare_v10_reproduction.py)는 별도의 빈 작업 폴더에 실행 복사본을 만들고, 전달받은 runtime과 skill 경로만 그 복사본에 적용한다. 저장소의 현재 발표와 원본 스냅샷은 수정하지 않는다. 기본 동작은 준비와 실행 명령 출력이며 모델을 호출하지 않는다. `--run`을 명시해야 artifact 도구의 재생성과 렌더가 시작된다.

```sh
.venv/bin/python docs/architecture/12-decisions/decision-packages/current/presentation/sources/prepare_v10_reproduction.py \
  --workdir /private/tmp/via-v10-reproduction \
  --runtime-root /absolute/path/to/bundled/dependencies \
  --skill-dir /absolute/path/to/presentations/skills/presentations
```

`--runtime-root` 아래에 `node/bin/node`, `node/node_modules`, `python/bin/python3`가 있어야 한다. `--skill-dir`에는 `container_tools`가 있어야 한다. 이미 존재하는 작업 폴더는 덮어쓰지 않는다. 도구 버전과 renderer가 달라지면 재생성한 ZIP 바이트가 원본과 같다고 보장할 수 없으며, 이번 이관의 바이트 동일성은 기존 파일의 복사에 대한 검사다. 이번 정리 작업에서는 재생성을 실행하지 않았다.

## 차기 편집 원본

새 발표를 편집할 때는 상위 폴더의 현재 v10 PPTX를 입력으로 지정하고 필요한 코드만 새 `sources/v11/`로 분기한다. 새 코드의 manifest는 사용하는 DP 본문과 그림, QA 계약 및 평가 파일을 같은 패키지 안의 정본으로 연결해야 한다. 기존 snapshot의 판본별 치환 규칙을 v10에 그대로 재실행하지 않는다.

내용과 표 또는 그림을 변경한 뒤에는 [PPTX-GUIDE](../PPTX-GUIDE.md)의 부분 편집, 보존 검사 및 최종 렌더를 수행한다. 현재 파일 포인터 갱신은 새 파일 검증 후에 한다.
