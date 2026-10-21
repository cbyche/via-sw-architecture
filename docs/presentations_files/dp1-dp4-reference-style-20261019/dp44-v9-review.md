# DP44 통합 발표 v9 검토

2026-10-22. DP44 내용을 추가하고 보존, 수치, 도형과 발표 파일을 검토한 기록이다. VIA 실행이나 성능 실측 결과가 아니다.

## 산출물과 근거

- [통합 발표 v9](VIA-DP1-DP5-DP44-reference-style-36slides-v9-windows.pptx): 36쪽, DP44는 24~30쪽.
- 원본은 [DP3 v8 통합 발표](VIA-DP1-DP5-reference-style-29slides-v8-dp3-guide-fixed-windows.pptx)다. DP1~DP5의 1~23쪽을 유지하고 역사 DP3 appendix를 31~36쪽으로 옮겼다.
- [DP44 설계 계약과 편집 그림](../../architecture/12-decisions/decision-packages/seven-problem-review/dp44-local-voice-execution-20261021/presentation-architecture-contract.md), [가상 비교 원장](../../architecture/12-decisions/decision-packages/seven-problem-review/dp44-local-voice-execution-20261021/synthetic-comparison-20261022/handoff.md)을 따른다.
- 근거 판본은 `DP44-DELIVERY-SYN-1.0`, 근거 종류는 `SYNTHETIC_AUTHORED_NOT_MEASURED`다. 작성된 가상 수치를 실측으로 표현하지 않는다.

| 파일 | SHA-256 |
| --- | --- |
| 원본 v8 | `0b2a87104b740714a9ced05e35b36407ffca723e172e40c2afc5dc00118ace10` |
| 최종 v9 | `ff3e7849df698c49e97e72a9e2a4dc61858b57d07b72a07702951dcaf87ad08d` |

## 보존과 편집 가능성

기존 29개 slide XML은 footer의 현재 쪽수와 전체 쪽수만 변경했다. 기존 notes와 관계 파일도 보존했으며, appendix 구분장 notes에 적힌 명시적 현재 위치만 32~36쪽으로 수정했다. 과거 자료 속 역사적 페이지 참조는 변경하지 않았다.

원본과 최종본을 같은 Artifact Tool로 렌더해 29장 모두 비교했다. 우하단 footer 영역을 제외한 픽셀이 정확히 일치했다. 새 7장은 래스터 이미지 없이 native 도형, 텍스트와 표로 구성했다. Module 이름은 별도 겹친 텍스트가 아닌 해당 도형의 텍스트다. PPTX 경로는 편집 가능한 선 도형이며 자동 재접속 connector는 아니다. draw.io 경로는 실제 source와 target, Module의 parent Component를 가진다.

한 Scene 정의에서 PPTX, SVG와 draw.io를 생성하고 SVG에서 PNG를 파생했다. actual PPTX 렌더는 [전체 36쪽 미리보기](previews-v9-dp44/)에 보존한다.

## 검토와 수정

별도 reviewer가 소스, 문서, PPTX notes와 25~27쪽 렌더를 검토했다. 다음 문제를 수정하고 재검토했다.

| 발견 사항 | 수정과 확인 |
| --- | --- |
| 입력용 Runtime에서 DP2로 바로 이어져 로컬 입력 확정 책임이 빠짐 | Runtime → S2S Runtime Adapter → Input Finalizer → DP2로 연결하고 Local VAD의 종료 경로를 추가 |
| QA 차이의 `+` 표기가 이득인지 비용인지 모호함 | 감소와 절감을 명시하고 원장 수치 대조 |
| 제목 또는 흐름 설명과 선의 겹침 | 경로와 라벨 위치를 분리하고 실제 렌더 확인 |
| 취소된 늦은 음성을 모두 버리는 것처럼 보임 | 구판과 철회는 거절하되 유효 판본의 발화 전환 취소는 terminal까지 생성된 유효 구간을 보존하도록 수정. S05의 tail 보존과 일치 |
| 정정된 판본과 읽기 순서가 A/B에 전달되는 경로 부족 | 27쪽 Response Registry에서 각 제어 Module로 실제 연결 추가 |
| 요청과 반환의 선 종류가 불명확함 | 요청과 전달은 실선, 결과와 복원은 점선으로 정리하고 25~27쪽 범례 확인 |

Reviewer의 최종 재검토는 남은 P1/P2가 없다고 보고했다. 이후 작성자가 모델 반환선 2개의 식별자를 수정해 점선을 적용하고 27쪽 라벨 3곳을 이동했다. 최종 25쪽과 27쪽 렌더 및 보존 검사를 다시 확인했다. 이 검토는 native Windows PowerPoint 실행 확인을 포함하지 않는다.

## 자동 검사와 범위

| 검사 | 결과와 근거 |
| --- | --- |
| 패키지 무결성, slide 수, 표, 레이아웃과 폰트 | 36쪽, finding 0, warning 0. [Artifact Tool 최종화 기록](dp44-v9-artifact-validation.json) |
| 원본 삽입과 보존 | 기존 29쪽, 추가 7쪽. [패키징 기록](dp44-v9-preservation.json) |
| 원본 XML과 notes, 29쪽 픽셀 비교, native 도형과 수치 | PASS. QA 대표값 12개와 정확도 분모 셀 4개 대조. [최종 검사](dp44-v9-final-checks.json) |
| Module 소유와 실제 endpoint, 직교 경로, 다른 Module 및 Component 제목 관통 | 7장 PASS. [Scene 검사](dp44-v9-scene-checks.json) |
| Windows용 Calibri, 정수 pt와 ko-KR | 로컬 패키지 검사 PASS. Windows workflow 결과는 후속 기록 |
| 저장소 링크, 용어, 변경 공백, 그림 XML | active Markdown 936개 링크 PASS, 용어 PASS, `git diff --check` PASS, SVG/draw.io 14개 XML 유효 |

QA01 UI와 별도 첫 음성, QA03의 외부 시간과 재생 시간 제거 경계, 고정 정답 fixture인 QA02/07, 개념 Module 단위 QA04, cloud 요금과 로컬 RAM 배치 조건을 유지한다. 조건부 A 권고와 미평가 A1을 측정된 승자나 보완 효과로 바꾸지 않았다.

## 재생성

원본과 출력은 별도 경로로 둔다. 아래는 이 작업 환경의 bundled Node/Python을 사용한 절차이며, 임시 build 경로는 매 실행 새로 만든다.

```sh
build_dir=$(mktemp -d /private/tmp/via-dp44-v9-XXXXXX)
runtime_root=/Users/changbae/.cache/codex-runtimes/codex-primary-runtime/dependencies
cp docs/presentations_files/dp1-dp4-reference-style-20261019/VIA-DP1-DP5-reference-style-29slides-v8-dp3-guide-fixed-windows.pptx "$build_dir/source-v8.pptx"
cp scripts/presentations/{build_dp44_v9,dp44_v9_pages,dp44_v9_scene}.mjs "$build_dir/"
ln -s "$runtime_root/node/node_modules" "$build_dir/node_modules"
"$runtime_root/node/bin/node" "$build_dir/build_dp44_v9.mjs" "$PWD" "$build_dir"
"$runtime_root/python/bin/python3" scripts/presentations/check_dp44_v9.py . "$build_dir/source-v8.pptx" "$build_dir/output/VIA-DP1-DP5-DP44-reference-style-36slides-v9-windows.pptx" "$build_dir/scene-model.json" --report "$build_dir/final-checks.json"
```

픽셀 보존 검사는 동일 도구로 렌더한 원본 29장 폴더와 최종 36장 폴더를 `--source-previews`와 `--previews`로 전달한다. 파일 작성 timestamp에 따라 재생성 ZIP hash는 달라질 수 있으므로 보존 및 내용 검사를 함께 사용한다.

## 게시와 Windows 검증

이 파일 경로를 지정한 `presentation-windows-check`의 `workflow_dispatch`를 실행한다. push 자동 실행의 기본 파일은 과거 v2이므로 그것을 v9 검증으로 세지 않는다. 실제 v9 결과와 실행 링크는 검증 완료 뒤 이 절에 기록한다.
