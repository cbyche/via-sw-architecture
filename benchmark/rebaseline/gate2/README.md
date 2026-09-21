# Gate 2 review tooling

`catalog_check.py`는 GitHub Markdown을 원본으로 사용하여 **자체 리뷰 후 남은 Core 4개 DP**의 A/B inventory와 complete configuration을 조립한다. FP-INT01(S2S Direct Fast Path)과 TASK-T01(Event-first + Query Reconciliation)은 공통 fixed principle/tactic이며 score pair가 아니다.

```bash
python benchmark/rebaseline/gate2/catalog_check.py --output results/gate2-review
python -m unittest discover -s benchmark/rebaseline/gate2 -p 'test_*.py' -v
```

산출물에는 5개 고유 configuration, 8개 pair-labelled 후보, W-01~W-12의 미측정 96개 row, 24개 change의 미분석 192개 row가 들어간다. null은 0점/0개가 아니라 NOT_RUN/NOT_ANALYZED다.

이 도구는 설계 ID·조합·전수 ledger의 정합성만 검사한다. VIA 구현, Model accuracy, p95, actual process isolation, Architecture winner를 증명하지 않는다. 사람용 문서에서는 W-ID만 쓰지 않고 전체 ASR 명칭/설명을 병기한다.
