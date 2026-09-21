# VIA 11-A/B 측정 자료

모든 자료는 합성/검토용이며 실제 후보의 결과는 NOT_RUN이다.

- cases.json: 94개 명세. source_fixture와 patch_key의 초기 상태·events를 사용한다.
- fixtures/base.json: Source 서버의 원천 자료, 초기화 시 적재할 대화·Task·권한·기억. 모든 Source를 모델에 무료 주입하지 않는다.
- fixtures/patches.json: 기본 상태 override와 occurrence/availability 시간. process restart 이후에는 초기화 자료를 다시 적재하지 않는다.
- oracle/expected.json: 평가기 전용 의미상 필수/금지 관찰과 ASR-tagged atomic obligation. 후보에 제공하지 않는다. 일부 의미 판정은 사람이 검토한다.
- oracle/obligation-catalog.json: ASR-02/03 degree metric용 obligation ID/kind/ASR tag 원장. 후보 결과 전에 고정한다.
- safety-inputs.json / oracle/safety-opportunities.json: 후보 입력/평가 정답 분리.
- change-cases.json: 24개 원문 변경 명세와 null 원장.
- prompts.json: 8목적의 실제 문자열과 output 예시; 출력예시는 Model 입력에 넣지 않는다.
- qwen-reference-profile.json: 공개 행과 파생 가정.
- raw-results.template.json: 초기 결과 없음.

## 실행

```bash
python benchmark/rebaseline/build_assets.py
python benchmark/rebaseline/measure.py --validate
python benchmark/rebaseline/measure.py --input-tokens 1200 --output-tokens 60
# 공식 tokenizer 파일과 라이브러리가 준비된 환경에서만
python benchmark/rebaseline/token_count.py --tokenizer-dir /path/to/qwen-tokenizer
```

두 번째 명령은 검사 함수만 실행하고, 세 번째 명령은 주어진 token 수의 모델 소계 추정이다. 실제 prompt 측정 또는 후보 p95가 아니다. tokenizer 스크립트는 tokenizer.json과 tokenizer_config.json을 요구하며 공식 tokenizer hash를 검사한다. 라이브러리는 tokenizers와 jinja2만 필요하며 실제 버전이 원장에 기록된다.

measure.py는 지연계산·DAG resource경합·ASR-02/03 obligation degree 집계·변경ID집계·안전분모 및 코드의 양/음성 단위 검사를 제공한다. 후보 구현을 호출하는 adapter나 의미 정확도 자동 평가기는 아니다. 실제 후보 연결 후 trace를 수집하고 리뷰된 oracle로 판정해야 한다.

`build_assets.py`는 승인된07의 변경 표와 11-B의 TC 표를 읽어 명세를 재생성한다. GitHub에는 원문 생성기/측정 코드, 검토 ZIP에는 JSON/CSV/HTML 생성 결과를 함께 제공한다. generator 재실행은 NOT_RUN 템플릿만 다시 만들므로 실제 결과 파일은 별도 results 경로에 보관한다.
