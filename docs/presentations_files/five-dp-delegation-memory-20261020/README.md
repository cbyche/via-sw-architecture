# VIA DP5 위임 메모리 추가 발표

> 기반 파일 정정: 이 폴더의 40장 통합본은 사용자가 의도한 v4 reference-style 파일을 사용하지 않았다. 현재 요청의 최종본은 [정확한 v4 기반의 21장 v5](../dp1-dp4-reference-style-20261019/dp5-v5-review.md)다. 이 폴더는 이전 산출물로 보존한다.

2026-10-20. [기존 최종 발표](../four-dp-final-20261019/README.md)의 구조와 편집 가능한 도형·표 서식을 유지해 DP5를 추가했다. 기존 파일을 덮어쓰지 않았다.

- [통합 발표 PPTX](output/VIA-five-DP-with-memory-20261020-v2.pptx): DP1~4의 기존 30장과 DP5 설계 10장, 총 40장.
- [DP5 설계 원문](../../architecture/12-decisions/decision-packages/seven-problem-review/dp5-delegation-memory-20261020.md): 두 기본안, 데이터와 처리 계약, QA별 관련성과 차별화 가설, 선택 조건.
- [발표 순서](storyboard.md).
- [전체 미리보기](render-list.md).
- [통합 제작 원본](source/build.mjs), [기존 구조도 원본](source/structures.mjs), [DP5 원본](source/dp5.mjs), [기존 고정 수치](source/deck-data.json).

## 내용과 상태

DP5는 원본과 명시적 사실을 공통으로 보존한 뒤, 미확정 위임 관계를 **A 요청 시 해석**하거나 **B 사건 후 사전 해석·관리**하는 비교다. DP1 B/C의 근거 생산 축과 DP4 A/B의 준비·갱신 지식을 위임 관계에 적용했다. 기존 DP1 C와 동일한 생산은 공유하고 중복 효과·비용으로 계산하지 않는다.

DP1~4의 2026-10-19 관측값과 조건부 권고는 원래의 한계와 함께 유지했다. 이후 다른 작업에서 수정 중인 결과를 가져오지 않았다. `source/deck-data.json`은 원본과 byte 단위로 같다. 발표 DP4는 기존 저장소 05 handoff이며 새 발표 DP5와 다르다.

DP5 25~34장은 **설계 가설이며 미측정·미선택**이다. 새 모델 실행이나 구현 실험은 없다. QA07은 2026-10-20의 요청 의미 해석 정확도 정의이고, 과거 부하 추가 지연 QA07과 구별한다. QA07 우선순위·점수 구간을 추가 승인한 자료가 아니다.

## 재생성

기존 발표와 같은 Node ESM + `@oai/artifact-tool` bundle 26.1015.12040, Apple SD Gothic Neo, 1280×720 canvas를 사용한다. `.build/`는 실행 복사본, dependency symlink와 검증·렌더 기록용이다. `source/`의 세 mjs 파일을 `.build/`로 복사하고 해당 런타임 node_modules를 연결한 뒤 아래 명령을 사용한다. 새 revision을 사용해야 하며 기존 output을 덮어쓰지 않는다.

```sh
/Users/changbae/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/bin/node docs/presentations_files/five-dp-delegation-memory-20261020/.build/build.mjs --export --final --revision=local-copy-1
```

재생성은 모델 호출을 수행하지 않는다. 코드에 쓰인 source 경로는 기존 frozen 데이터의 provenance이며 현재 결과를 새로 집계하지 않는다.

미리보기는 실제 내보낸 PPTX를 Artifact Tool로 다시 읽어 만든 PNG다. PowerPoint 앱이나 Google Slides에서의 실물 편집 검증을 대신하지 않는다.
