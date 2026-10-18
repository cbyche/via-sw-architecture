# VIA 네 주요 DP 발표자료

2026-10-19. 추가 실험 집계와 30장 발표자료의 편집·시각 검토를 완료했다. 최종 권고와 숫자는 [발표 결과](../../architecture/12-decisions/decision-packages/seven-problem-review/17-presentation-results.md)를 따른다.

## 파일 구분

- [최종 PPTX](output/VIA-four-DP-20261019-final-v1.pptx): native 도형·연결선·표로 편집 가능한30장. `output/`에는 최종 PPTX만 둔다.
- [전체30장 미리보기](render-list.md): 최종 PPTX를 다시 읽어 만든 `previews/`의 PNG를 연결한다.
- [storyboard.md](storyboard.md): 발표 순서와 내용 원본.
- [source/build.mjs](source/build.mjs), [source/structures.mjs](source/structures.mjs): 편집 가능한 native 도형·연결선·표를 만드는 공개 JavaScript 원본.
- [source/deck-data.json](source/deck-data.json): 모델 호출 없이 다시 만드는 데 사용하는 수치 입력과 최종 원본 보고서 SHA.
- `.build/`: 로컬 초안 PPTX, 렌더 PNG, 임시 실행 복사본과 검증 기록. Git에는 최종 검증 JSON과 검토·교정 이력을 선별 보존하며, 실행 복사본·중복 렌더·`node_modules` 연결은 포함하지 않는다.

## 모델 없는 재생성

저장소를 다른 위치에 두어도 `source/`와 `.build/`의 깊이를 유지하면 저장소 경로를 상대적으로 찾는다. Node.js ESM, Codex의 `@oai/artifact-tool` 및 Presentations 스킬이 필요하다. 이 세션은 bundle `26.1015.12040`과 macOS의 실제 `Apple SD Gothic Neo` 글꼴을 사용했다. 다른 OS의 글꼴 대체 결과를 검증했다는 뜻은 아니다.

다음 명령은 저장소 루트에서 실행한다. 경로는 이 세션에서 확인한 절대 경로다. 다른 Codex 설치에서는 `RUNTIME_NODE_MODULES`, `RUNTIME_PYTHON`, `PRESENTATIONS_SKILL_DIR`를 실제 설치 경로로 지정한다. 의존성을 새로 설치하거나 모델/API를 호출하지 않는다.

```sh
mkdir -p docs/presentations_files/four-dp-final-20261019/.build
cp docs/presentations_files/four-dp-final-20261019/source/build.mjs docs/presentations_files/four-dp-final-20261019/.build/build.mjs
cp docs/presentations_files/four-dp-final-20261019/source/structures.mjs docs/presentations_files/four-dp-final-20261019/.build/structures.mjs
ln -s /Users/changbae/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules docs/presentations_files/four-dp-final-20261019/.build/node_modules
/Users/changbae/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/bin/node docs/presentations_files/four-dp-final-20261019/.build/build.mjs --export --final --revision=local-copy-1
```

`node_modules` 연결이 이미 같은 경로를 가리키면 `ln -s`를 생략한다. 기존 최종 파일을 덮어쓰지 않는다. 새로 만들 때는 다른 `--revision`을 지정한다. `--final`을 생략하면 `.build/draft-output/`에만 만든다. 글꼴·16:9·native table·PPTX 무결성·Artifact Tool import 검사 후 내보내며, 최종 PPTX를 다시 읽어 모든 장의 PNG를 `.build/renders-<revision>/`에 만든다. 이 세션의 최초 authoring marker는 이미 한 번 실행했다. 재생성에 원래 실험이나 모델 호출이 필요하지 않다.

## 근거

- [새 실험 인덱스](../../../results/architecture-evaluation/current/four-dp-presentation-20261019/README.md)
- [기존 전체 원시·실패 보존 패키지](../../../results/architecture-evaluation/current/four-dp-poc-evidence-20261019/README.md)
- [기존 보완 실험 결과](../../architecture/12-decisions/decision-packages/seven-problem-review/15-corrected-poc-results.md)
- [추가 비교 계약](../../architecture/12-decisions/decision-packages/seven-problem-review/16-presentation-selection-contract.md)
- [수치·코드·시각 검토 기록](../../architecture/12-decisions/decision-packages/seven-problem-review/reviews/presentation-review-20261019.md)

시간의 미관측과 본문 조건 미충족을 숨기지 않는다. 완료 판정 관측, 별도 본문 진단, 실제 모델 사용량, 재생의 RSS/할당량은 서로 다른 관찰이다.
