# VIA ASR-QA 목표 근거와 등급 부록

[03-02 정의 원본](../../architecture/12-decisions/decision-packages/03-02-quality-attribute-definitions.md)의 여섯 QA를 각 QA별 한 페이지에 정리했다. 정의/지표/목표/과제 근거, 보호 조건과 번호순 우선순위를 유지한다. 후보 성능은 아직 측정하지 않았다.

- [편집 가능한 PowerPoint, 6장](VIA-quality-metrics.pptx)
- [최종 PPTX에서 렌더링한 PNG](index.html)
- [발표 문안과 원본 연결](../quality-metrics.md)
- [투영 JSON](quality-metrics.json): 원본 SHA-256, 여섯 핵심 정의와 상세 평가 명세, 출처
- [확인 범위](verification.md)
- [이전 발표 archive](../../archive/qa-presentations-before-asr-20261011/README.md): 원래 bytes/hash 보존

PowerPoint native text와 table로 편집 가능하며 슬라이드 PNG를 붙인 파일이 아니다. 16:9, 1920×1080이다. 한글은 Apple SD Gothic Neo, 등급 기호는 Arial Unicode MS를 사용했다. Microsoft PowerPoint 앱에서 저장/재열기는 확인하지 않았다. 글꼴이 다른 환경의 배치는 확인이 필요하다.

## 원본 관리와 재생성

[03-02](../../architecture/12-decisions/decision-packages/03-02-quality-attribute-definitions.md)를 수정한 뒤 아래 투영/검산을 재생성한다. 별도 발표 정의를 JSON에서 독립 개정하지 않는다.

```sh
python3 scripts/architecture/qa_common_workload.py --write
python3 scripts/architecture/qa_poc_reference.py --write
python3 scripts/presentations/build_asr_qa_sources.py
python3 scripts/presentations/build_asr_qa_sources.py --check
```

Codex workspace dependency 도구로 bundled Node/Python/node_modules 경로를 확인한다. Presentations 스킬의 재생성 절차에 따라 새 임시 build 폴더에 [통합 생성기](../../../scripts/presentations/generate_asr_qa_presentations.mjs)를 복사하고 `node_modules`를 bundled modules에 연결한다. `RUNTIME_NODE_MODULES`, `VIA_PRESENTATION_SKILL_DIR`, `VIA_RUNTIME_PYTHON`을 각 환경의 절대 경로로 지정하고 bundled Node로 `generate_asr_qa_presentations.mjs REPO_ABSOLUTE BUILD_ABSOLUTE metrics`를 실행한다. candidate와 별도 final 파일을 사용하며 기존 final 폴더를 재사용하지 않는다. package/layout/native table/font/Artifact Tool 재가져오기를 검증하고 최종 파일에서 PNG를 렌더링한다.

```sh
python3 scripts/presentations/check_quality_attributes.py
python3 scripts/presentations/check_quality_metrics.py
```

등급은 대표 지표의 목표를 표시한다. 보호 조건을 함께 검사하며 서로 다른 QA의 등급을 합산하지 않는다. 목표/계산의 작성자 가정과 공개 단가/장치 사양을 구별하며 실제 A/B 우열이나 제품 목표 달성을 주장하지 않는다.
