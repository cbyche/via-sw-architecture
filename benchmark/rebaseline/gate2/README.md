# Gate 2 review tooling

`catalog_check.py`는 GitHub Markdown을 원본으로 사용하여 Core 6개의 A/B inventory와 완전한 configuration을 조립한다. 실제 VIA 구현·LLM 실행·score 계산기가 아니다.

```bash
python benchmark/rebaseline/gate2/catalog_check.py --output results/gate2-review
python -m unittest discover -s benchmark/rebaseline/gate2 -p 'test_*.py' -v
```

산출물 `design-manifest.json`에는 source hashes, 모든 C/I/S/D 정의, 7개 고유 configuration, 12개 pair 후보, W-01~12의 미측정 144개 row, 24개 change의 미분석 288개 row가 들어간다. 값·점수·변경 ID가 null인 것은 미실행이지 0점/0개가 아니다.

`validation-summary.json`은 위 연결·형식의 정적 검사 결과만 기록한다. test suite는 validator의 오류 검출을 검증하며 VIA 기능·모델 정확도·p95·실제 장애 격리를 증명하지 않는다. Task별 actor instance 수는 C/D 개수로 복제하지 않는다.

문서 변경 후 생성물을 다시 만들면 source hashes가 바뀐다. 실제 실행 결과는 이 generator의 출력 경로와 다른 results 디렉터리에 보관한다. 기존 7-ASR `build_assets.py`의 generated scoring을 Working-12와 혼용하지 않는다.
