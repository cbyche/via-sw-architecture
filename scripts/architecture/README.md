# Architecture Repository Checks

이 디렉터리는 active Architecture 문서와 저장소 구조의 정합성을 검사하는 유지보수 도구를 둔다. Benchmark campaign runner나 result generator의 active 위치가 아니다.

## Current checks

```bash
.venv/bin/python scripts/architecture/check_active_markdown_links.py
.venv/bin/python scripts/architecture/check_active_terminology.py
.venv/bin/python scripts/architecture/check_qa_catalog.py
.venv/bin/python scripts/architecture/generate_target_architecture_diagrams.py --check
.venv/bin/python scripts/architecture/generate_decision_package_diagrams.py --check
.venv/bin/python scripts/architecture/generate_stage4_structure_diagrams.py --check
.venv/bin/python scripts/architecture/generate_mechanism_family_diagrams.py --check
.venv/bin/python scripts/architecture/generate_functional_choice_diagrams.py --check
.venv/bin/python scripts/architecture/generate_six_comparison_diagrams.py --check
.venv/bin/python scripts/architecture/check_evaluation_package.py <result-directory>
```

- `check_active_markdown_links.py`는 active Markdown의 local link와 archive 경계 문제를 찾는다.
- `check_active_terminology.py`는 legacy metric ID과 과거 lifecycle 번호가 active 문서에 다시 섞이지 않는지 확인한다.
- `check_qa_catalog.py`는 draft QA registry의 active/retired ID와 normative Markdown 표가 일치하는지 확인한다.
- `generate_target_architecture_diagrams.py --check`는 파일을 변경하지 않고 target 그림의 선 교차·박스 관통, XML ID, 생성 source와 draw.io/SVG 쌍의 일치를 검사한다. 글자·화살촉 배치는 렌더링으로 별도 검토한다.
- `check_evaluation_package.py`는 공식 DP result가 후보별 active QA 19행, raw evidence,
  독립 replay와 필수 보고서를 모두 갖췄는지 fail-closed로 검사한다.

`index_audio_artifacts.py`는 target Mac에 남기는 대용량 WAV 원본의 상대 경로·크기·
SHA-256 목록을 result의 `raw/audio-artifact-digests.json`으로 만든다. 구조화된 raw와
digest는 Git에 보존하고 WAV 원본은 로컬 evidence로 유지한다.

이전 일곱 후보의 그림 생성기는 [script archive](../archive/decision-reconstruction-component-boundaries-2026-09-30/README.md)로 옮겼다. 재평가 참고용 [기존 네 비교 자료](../../docs/architecture/12-decisions/decision-packages/README.md)의 8페이지는 `generate_decision_package_diagrams.py`로 생성한다. `--check`는 생성물 drift, XML ID, 경계, 연결선의 무관한 node 관통을 검사한다. 실제 glyph와 가독성은 브라우저 렌더링으로 별도 확인한다. 후보마다 실제 topology에 맞춰 배치하며 동일 격자 템플릿을 쓰지 않는다. 직전 여덟 후보 생성기는 [2026-10-01 script archive](../archive/decision-reconstruction-uniform-layouts-2026-10-01/README.md)에 보존했다. Target Architecture 그림 검사는 그대로 유지한다.

현재 4단계의 S-01~06 비교 그림은 `generate_stage4_structure_diagrams.py`로 생성한다. 개별 04-01~06 문서에 SVG와 draw.io를 함께 연결하며, `--check`는 6쌍과 `stage4-review.html`의 생성물 일치, XML, 좌표 및 연결선을 검사한다. 기존 네 자료의 serializer만 재사용하며 그 설계나 QA 판단을 자동 승계하지 않는다. 실제 글자와 화살표 배치는 별도 렌더링으로 확인한다.

## Local model utility

```bash
scripts/architecture/start_local_semantic_model.sh
```

`start_local_semantic_model.sh`는 freeze된 Qwen3-8B Q4_K_M SHA-256을 확인한 뒤 localhost의 OpenAI-compatible endpoint를 시작한다. 모델 파일과 secret은 저장소에 넣지 않는다. 구체 profile은 [VIA Core Evaluation Profile](../../docs/architecture/11-measurement/evaluation-profile.md)을 따른다.

Alibaba S2S credential smoke test는 macOS Keychain의 `via-dashscope-api-key`, `via-dashscope-workspace-id`를 읽어 고정 WAV를 전송한다.

```bash
.venv/bin/python scripts/architecture/smoke_alibaba_s2s.py \
  --wav /path/to/mono-16khz-pcm16.wav \
  --out-dir /private/tmp/via-s2s-smoke
```

응답 WAV와 secret을 제외한 event timing trace만 지정한 output directory에 쓴다. 이 결과는 provider 연결과 model audio packet을 확인하는 `MEASURED_MODEL` smoke evidence이며 physical speaker onset 또는 `PRODUCT_E2E`가 아니다.

새 active README나 contract directory를 추가하면 link checker의 scan scope도 함께 검토한다. 이전 W12-G1 measurement/review scripts는 [script archive](../archive/README.md)에 보존한다.

확대 탐색 04-14~16의 세 비교 그림은 `generate_mechanism_family_diagrams.py`로 생성한다. `--check`는 SVG/draw.io 일치, XML 참조, 소유 경계와 연결 경로를 확인한다. 실제 글자 배치와 구조 의미는 별도로 검토하며 이 검사를 구현이나 품질 검증으로 취급하지 않는다.

핵심 기능 탐색 04-19~21의 세 비교 그림은 `generate_functional_choice_diagrams.py`로 생성한다. 동일한 source로 SVG/draw.io, XML 참조와 연결 경로를 검사한다. A/B는 참조 구조에 고정되지 않으며 실제 글자 배치와 의미는 별도로 검토한다.

현재 04-31~36의 구조와 사건 흐름 12쌍은 `generate_six_comparison_diagrams.py`로 생성한다. `--check`는 source 일치, XML, 소유 경계와 직교 경로 및 실제 연결 노드의 끝점을 검사한다. 본문/그림의 의미와 읽기 품질은 렌더로 별도 확인한다. 이전 두 생성기와 그림은 보존된 탐색의 재현 자료다.

35/36의 메인 구조도는 `state_security_presentation.py`의 16:9 scene을 기존 `generate_six_comparison_diagrams.py`가 호출해 생성한다. 별도 중복 생성 경로는 없다. 다른 메인/사건도는 유지하며 동일 `--check`가 12쌍 전체를 검사한다. 발표용 PNG는 최종 SVG를 로컬 브라우저에서 2560×1440으로 렌더한 편의 산출물이다. 원본은 SVG/draw.io 공통 scene이며, 수정 후 PNG도 다시 내보내고 실제 글꼴/화살표를 확인한다.

## 04-41 요청 의미 확정 도식

`generate_request_resolution_diagrams.py`는 41의 메인 구조도와 A/B 사건도 세 쌍을 생성한다. `--check`는 SVG/draw.io 일치, XML, 소유와 경로를 검사한다. 복원한 31의 생성기와 독립적으로 관리하며 설계 의미는 41 본문을 따른다.

## 04-42 대화와 업무 수명 발표 도식

`python3 scripts/architecture/generate_lifecycle_ownership_diagrams.py`는 42의 16:9 메인 구조도, 수명 시간축, A/B 답변 처리 사건도, 변경 영향 비교도 다섯 쌍과 `choice42-review.html`을 생성한다. `--check`는 source 일치, XML ID/연결, owner 포함, canvas와 직교 경로를 검사한다. SVG/draw.io는 같은 scene으로 생성한다. 별도 브라우저 렌더링으로 한글·실제 글자 겹침·발표 크기 가독성을 확인한다. PNG는 최종 SVG를 브라우저에서 내보낸 편의 산출물이며 생성기 원본은 아니다. 기존 생성기와 설계는 변경하지 않는다.
