# 통합 발표 v10 전체 가이드 검토

2026-10-22. DP44 세션의 작성과 commit/push를 확인하고, 추가 7쪽과 기존 29쪽을 가이드에 대조했다. DP44 최종 v9는 `30a5cd166`, Windows 검증 기록은 `62fad0b2c`로 main에 발행되어 있다. 그 파일을 기준으로 이전 내용을 보존하면서 기존 구조 그림과 라벨을 수정한 **36쪽 v10**을 만들었다.

- [최종 PPTX](VIA-DP1-DP5-DP44-reference-style-36slides-v10-guide-reviewed-windows.pptx)
- [전체 36쪽 실제 렌더](previews-v10-guide-reviewed/)
- [설계 설명과 SVG, draw.io, PNG 11개](../../architecture/12-decisions/decision-packages/seven-problem-review/presentation-guide-review-20261022/README.md)

## 적용 기준

[AGENTS §9](../../../AGENTS.md#9-pptx-authoring-policy), [INTENT](../../../INTENT.md), [설계 그림 가이드](../../architecture/12-decisions/decision-packages/diagram-design-guide.md), [참고 발표 형식 체크리스트](reference-checklist-and-review.md)를 사용했다. Component 안의 Module과 상태 소유, 도형 자체 텍스트, 실제 호출/반환 endpoint, 재허가와 보완 루프, 실행 수명, 색과 선의 범례, Windows 서체 정책을 대조했다. 이전 체크리스트의 16쪽 구성이나 과거 QA 의미를 현재 문서에 덮어쓰지 않았다.

## 발견한 문제와 수정

| 범위 | 수정 및 최종 확인 |
| --- | --- |
| DP44 25–27쪽 | 초안의 반환선을 점선으로 분리하고, Registry에서 채택한 정정 판본이 생성/turn/재생 제어에 도달하는 실제 경로를 추가하도록 작성 세션에 전달했다. self-loop와 문구 관통, chunk 반환 라벨 위치, 페이지별 범례도 수정됐다. 완성된 v9의 24–30쪽을 독립 재검토하여 D01–D05를 닫았다. 이 7쪽은 v10에서 그대로 보존한다. |
| DP1 2–4쪽 | 첫 해석부터 유효 작업 문맥을 공급하는 경로, Controller의 읽기 허가·의미 채택, 원본 부족 시 조회와 반환, A2의 변경 조각 읽기·전체 판본 검증·문맥 갱신을 실제 Module 연결로 표현했다. |
| DP2 6–8쪽 | 공통 Model Access의 호출 한도와 모델 요청/반환, 대화 실행과 R1/R2 논리 실행, 개별 가변 상태와 B1의 불변 입력 공유를 구분했다. A가 앞선 목표 전체 종료를 기다린다고 약화하지 않았다. |
| DP4 17–19쪽 | 구성 후보 반환, Controller 전송 허가, Task Manager 명령 확정, Agent Gateway 전송과 READ 왕복을 연결했다. B의 원본 직접 보완을 유지하며, C 접근 원장의 물리 저장 방식을 임의로 고정하지 않았다. C1의 조건 원문과 일반 상세 READ도 구분했다. |
| DP5 21–22쪽 | 조회 허가, 원본 결과, 추가 읽기·재허가·해석 재개를 닫힌 경로로 표현했다. B의 별도 관계 생산과 요청 조회, 미생산/부족 범위 원본 보완, A1/B1 추가 상태를 구별했다. |
| 기존 라벨 | 확인된 99쌍의 도형+overlay textbox를 도형 자체 text body로 옮겼다. 새 그림으로 교체한 영역의 200쌍은 이관 대상에서 제외했다. 일반 흐름 주석과 navigation chevron을 Component 라벨로 오인해 합치지 않았다. |
| 설명 정합성 | 범례 11개를 실제 선/색 의미에 맞추고 수정 11쪽의 발표자 노트에 도식 설명을 추가했다. 원래 notes paragraph와 스타일은 유지했다. SVG와 draw.io에도 독립적으로 읽을 수 있는 범례를 넣었다. |

수정한 구조 그림은 **2, 3, 4, 6, 7, 8, 17, 18, 19, 21, 22쪽**이다. 기존 배경, QA 표, 수치, 선택 조건과 보완의 평가 상태를 바꾸지 않았다. DP3 9–15쪽, DP44 24–30쪽, 기존 DP3 appendix 31–36쪽을 유지했다. 이전 v8/v9 파일도 남겼다.

## 독립 검토와 시각 확인

기존 29쪽과 DP44 초안 7쪽을 먼저 전부 열어 확인했다. 이후 DP44 최종 7쪽, v10에서 다시 그린 11쪽을 실제 최종 PPTX 렌더로 다시 검토했다. 메커니즘 교차 검토 과정에서 DP1 최초 문맥 공급, DP4 B 원본 보완 누락, C 접근 수명의 임의 RAM 고정을 수정했고, 선/라벨 겹침과 작은 원통의 줄 간격도 고쳤다. 최종 재검토에서 이 범위의 미해결 P1/P2를 발견하지 못했다.

새 SVG 11개도 로컬 브라우저에서 실제로 열었다. standalone DP5 보완 범례가 B1을 초록으로 설명하던 오류를 실제 파랑 A1/B1에 맞춰 수정한 뒤 다시 확인했다. PNG는 같은 SVG에서 파생한 그림이며, 전체 발표의 실제 모습은 PPTX 렌더를 기준으로 한다. SVG와 PPTX의 글꼴 렌더러가 달라 서로 픽셀 동일성을 주장하지 않는다.

- [초기 전체 검토](../../architecture/12-decisions/decision-packages/seven-problem-review/presentation-guide-review-20261022/reviews/initial-guide-review.md)
- [설계 교차 검토](../../architecture/12-decisions/decision-packages/seven-problem-review/presentation-guide-review-20261022/reviews/architecture-cross-review.md)
- [DP44 최종 확인](../../architecture/12-decisions/decision-packages/seven-problem-review/presentation-guide-review-20261022/reviews/dp44-final-review.md)
- [v10 수정 11쪽 최종 확인](../../architecture/12-decisions/decision-packages/seven-problem-review/presentation-guide-review-20261022/reviews/final-visual-review.md)

앞의 초기 검토 파일에는 당시 미완료 상태가 그대로 남는다. 최종 닫힘은 이 기록과 후속 두 재검토를 기준으로 한다.

## 보존 및 패키지 검증

| 검사 | 결과 |
| --- | --- |
| 전체 36쪽 | 순서, slide ID/part, footer 보존. 36쪽 전부 렌더 |
| QA native 표 | **30개 표, 698셀 전체 내용과 형식 보존** |
| 미재작성 25쪽 | v9와 실제 렌더의 RGB 채널 차이를 비교하여 **25/25 픽셀 동일**. alpha-only 비교를 사용하지 않음 |
| 기존 note 및 공유 자원 | 원래 notes paragraph/style 보존, 11쪽에 설명만 추가. relationships, masters, layouts, theme, media와 part inventory 보존 |
| 구조 원본 | **241 nodes, 257 edges**의 소유, 포함, 실제 경계 endpoint, 직교 경로와 무관 노드 관통 검사 통과. PPTX 자체 라벨/좌표/preset, SVG node geometry, draw.io owner/edge route 대조 통과 |
| 라벨 이관 | 99쌍 실제 XML 대조, 제거된 200쌍 명시적 skip. preset text rectangle을 고려한 inset 보정 확인 |
| 편집 가능성 | 추가 picture 0. native 도형·텍스트·표 유지. 선은 편집 가능한 경로와 화살촉이며, 자동 재연결 PowerPoint connector는 아님. draw.io에는 source/target 참조가 있음 |
| 로컬 형식 | package integrity, layout geometry, native table, ArtifactTool 재import 통과 |
| 서체 및 언어 | Calibri, 정수 pt, ko-KR 위반 0 |

v9 원본 SHA-256: `ff3e7849df698c49e97e72a9e2a4dc61858b57d07b72a07702951dcaf87ad08d`.

v10 최종 SHA-256: `459fe15be4cb4f620e4e33d26aa9a2b7654f8d6dc204fe5975f5a21211231a75`.

[Artifact Tool 검사](whole-deck-v10-artifact-validation.json), [보존 보고서](whole-deck-v10-preservation.json), [독립 XML/원본 검증](whole-deck-v10-independent-check.json), [Scene 검사](whole-deck-v10-scene-checks.json), [전체 렌더 비교](whole-deck-v10-render-comparison.json)를 보존한다. Windows 검증 결과는 아래 발행 기록에 연결한다. 이전 v8/v9 또는 push-trigger 기본 파일의 성공을 v10 검증으로 세지 않는다.

검증은 표현, 구조 계약 일치와 파일 형식에 한정된다. 실제 VIA 구현·모델 실행·성능을 새로 검증한 결과가 아니다. DP1/2/4 기존 PoC, DP3/DP44 합성 예시, DP5 미평가 설계의 근거 상태를 유지한다. DP44 정확도 100%는 고정 정답 fixture이며 일반 정확도 우세의 측정값이 아니다.

## 재생성과 수정 원본

[설계 문서](../../architecture/12-decisions/decision-packages/seven-problem-review/presentation-guide-review-20261022/README.md)의 11개 SVG/draw.io를 함께 관리한다. Component 또는 호출 경로를 고치면 Scene, PPTX, SVG, draw.io, PNG와 문서를 같이 갱신하고 다시 렌더한다.

- [통합 생성기](../../../scripts/presentations/build_reference_guide_v10.mjs)
- [native/SVG/draw.io renderer](../../../scripts/presentations/whole_deck_v10_scene_io.mjs)
- [표와 비대상 개체 보존 패키저](../../../scripts/presentations/package_whole_deck_v10.py)
- [라벨 이관기](../../../scripts/presentations/migrate_native_shape_labels.py)와 [검토한 299쌍 mapping](../../../scripts/presentations/whole_deck_v10_label_map.json)
- [독립 최종 검증기](../../../scripts/presentations/check_whole_deck_v10.py)
- [도형별 text rectangle 보정 근거](../../architecture/12-decisions/decision-packages/seven-problem-review/presentation-guide-review-20261022/reviews/native-label-migration.md)

생성기는 새 private build 폴더에 복사해 bundled `node_modules`를 연결한 환경에서 실행한다. 저장소의 Scene와 패키저를 네 번째 인자로 지정한다. Node/Python/skill 위치는 현재 Codex bundled runtime 경로를 사용한다. `V10_ALLOW_PARTIAL=1`은 개발용 29쪽 probe에만 쓰며 최종 생성에서는 지정하지 않는다.

```sh
runtime_root=/Users/changbae/.cache/codex-runtimes/codex-primary-runtime/dependencies
build_dir=$(mktemp -d /private/tmp/via-guide-v10-XXXXXX)
cp scripts/presentations/build_reference_guide_v10.mjs "$build_dir/"
ln -s "$runtime_root/node/node_modules" "$build_dir/node_modules"
"$runtime_root/node/bin/node" "$build_dir/build_reference_guide_v10.mjs" "$PWD" "$build_dir" "$PWD/docs/presentations_files/dp1-dp4-reference-style-20261019/VIA-DP1-DP5-DP44-reference-style-36slides-v9-windows.pptx" "$PWD/scripts/presentations"
```


## GitHub 발행과 최종 Windows 결과

산출물 커밋 `d0f255476de6e5a038123020031d34e71eb358e0`을 `main`에 push했다. **실제 v10 파일 경로를 지정한** [Windows workflow_dispatch](https://github.com/cbyche/via-sw-architecture/actions/runs/39635290107)가 success이며, 같은 커밋의 [Architecture CI](https://github.com/cbyche/via-sw-architecture/actions/runs/39635259279)도 success다. push 자동 실행의 과거 기본 PPTX 검증을 v10 결과로 세지 않는다.

[SDK 보고서](whole-deck-v10-windows-openxml.json)의 SHA-256은 위 최종 v10과 같다. Microsoft Windows 10.0.20348, Open XML SDK 3.3.0, Office2021 형식으로 36쪽 열기 성공과 schema 오류 0개를 확인했다. Windows의 Calibri/정수 pt/ko-KR 검사도 통과했다. [환경 기록](whole-deck-v10-windows-environment.json)에 PowerPoint 미설치가 명시되어 있으므로 native PowerPoint 앱의 실행·편집·렌더링은 검증했다고 주장하지 않는다.

이 발행 기록을 추가하는 후속 커밋은 PPTX와 그림 원본을 바꾸지 않는다. 최종 검증 범위에 남은 고신뢰 P1/P2는 발견하지 못했으며, 과거 수치의 관측·합성·미평가 구분과 조건부 권고를 그대로 유지한다.
