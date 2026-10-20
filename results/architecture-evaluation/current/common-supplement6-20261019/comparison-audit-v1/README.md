# 전체 DP의 동일 입력 비교 감사

2026-10-20 / COMMON-COMPARISON-AUDIT-v1

기존441개 실행을 재분석했다. 새 모델 호출, 후보 실행과 측정 캠페인은 없다. 원시 기록, 기존 평가 결과, QA 정의와 점수는 변경하지 않았다.

## 읽을 파일

- [결론과 전체24셀 정정](../../../../../docs/architecture/12-decisions/decision-packages/seven-problem-review/17-presentation-results.md#14-전체-dp와-qa의-동일-신원-비교-감사)
- [모든 구성과 사례 및 동일 대상의 비교 표](tables.md)
- [수치 집계, 처리 단계별 비용과 메모리 내역](summary.json)
- [응답과 목표3,120쌍](endpoint-pairs.jsonl)
- [지칭 판정744쌍](qa02-pairs.jsonl)
- [입력과441개 원시 trace의 SHA-256](provenance.json)

## 비교 규칙

COMMON9와 COMMON_SUPPLEMENT6는 따로 집계한다. 같은 모집단, 사례, 반복과 endpoint ID를 대응시킨다. QA03의 시작 입력과 복수 입력의 member 집합도 일치시킨다. 전체 예정 대상을 양쪽 관측, 왼쪽만, 오른쪽만, 양쪽 미관측으로 남긴다. 미관측에0초나 벌점 시간을 넣지 않는다.

시간 차이는 같은 대상을 양쪽에서 관측한 집합만의 진단값이며, 예정 대상 전체의 속도 우위나 의미상 성공률이 아니다. QA01은 구체적 텍스트 callback, QA03은 host의 완료 endpoint를 관측했다. 본문의 정답은 이 endpoint 판정기가 보증하지 않는다. 새 QA07 점수는 계산하지 않았다.

QA02는 기존 재채점651건을 유지하고 처음 채택한 대상의 정오를 대응시킨다. QA05는 저장한 호출의 token 비용과 run 비용을 대조한다. QA06은 동시 RSS 관측값이며, 원격 모델 메모리나 Windows commit 메모리가 아니다. QA04는 기존 서식 차이 제외 후 변경 수를 읽었고42개 변경의 수용 시험을 재실행하지 않았다.

## 재현

기존 [증거 패키지](../preservation-recovery184-v1/README.md)와 [중단 시점 패키지](../preservation-final-stopped258-v1/README.md)를 원래 경로로 복원한 뒤 저장소 root에서 실행한다.

```sh
.venv/bin/python benchmark/architecture/common9_poc_20261019/test_comparison_audit_v1.py
.venv/bin/python benchmark/architecture/common9_poc_20261019/comparison_audit_v1.py --output /tmp/via-comparison-audit-v1
```

기술 인증된441개 실행, 지칭 판정651건, endpoint3,120쌍, 지칭 판정744쌍을 검사한다. 모든 원시 trace를 기존 provenance hash와 대조하고, 선택한 실행의 API 비용$11.9452128을 재집계한다. 제외된 이전 실패 run을 성공 데이터에 넣지 않으며 그 비용은 원래 전역 원장에 남긴다.
