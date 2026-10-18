# DP02 B1 불변 문맥 공유: 실제 응답 재생

72회 중 기능 동등성 72/72, 전송 payload 동등성 실패 0, 새 API 0회. 기존 실제 모델 응답 686개를 재생했다.

## 전체 RSS 주지표

| N | B 최고 MB | B1 최고 MB | B 평균 trial peak MB | B1 평균 trial peak MB | B 요청 할당 평균 bytes | B1 요청 할당 평균 bytes |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | 30.097408 | 30.294016 | 29.807957 | 29.807957 | 180133.6 | 130439.9 |
| 2 | 30.801920 | 30.507008 | 30.246229 | 29.858475 | 313252.8 | 212126.0 |
| 4 | 32.325632 | 31.866880 | 31.268864 | 30.557525 | 573372.3 | 365412.7 |

최고는 해당 N의 12회 중 최대 동시 Python+Rust RSS다. 평균은 각 trial의 동시 최고 RSS 12개 평균이며 서로 다른 PID의 최대를 더하지 않았다.

## 구조와 해석

B 기본은 백그라운드 worker의 불변 전체 input, source payload, wire envelope를 깊게 복제한다. B1은 같은 원본 input을 Arc로 공유하고 source와 wire를 빌려 직렬화한다. 각 요청의 변경 가능한 상태, revision, 대화 snapshot과 복구 규칙은 유지했다. 약 7.5KB의 실제 source JSON을 쓰며 padding을 추가하지 않았다.

요청 할당 peak는 감소했지만 N=1 전체 RSS 평균은 동점이고 최고는 B1이 약0.197MB 높았다. N=2는 평균 약0.388MB, N=4는 평균 약0.711MB 낮았으며 N=4 최고는 약0.459MB 낮았다. B1을 작은 복제 부담 보완으로 제시할 수 있으나 일반 사용에서 QA06 우세를 입증했다고 말할 수 없다. 원래 BASIC의 A/B RSS 차이를 이 실험으로 인과 설명할 수 없다.

이전72회는 별도 보존한다. 이번 판본은 원본의 선행 호출 준비(OPERATION_START)에 해당하는 실제 model_call 수신까지 기다린 후 후행 응답을 반환한다. 입력 payload나 결과를 후처리하지 않았다.

고정 첫 호출 wave에서 한 번 관측한 뒤, 실제 원본의 host operation/adoption 순서에 따라 응답을 반환했다. 이 관측 gate와 즉시 재생은 새 QA01/03 지연 실측이 아니다. 원래 B의 N-R 추가 Task 연결 오류도 결과 그대로 보존했다.

## 파일

- plan.json / frozen/freeze.json: 실행 전 조건·source/binary·입력 SHA.
- trials.csv / summary.json: 72개 개별값·역할별 samepeak·초기값·표본 gap.
- trials/*/events.jsonl, samples.json, result.json: 원시 사건·동시 RSS·결과.
- preflight/ 및 preflight-host-order-v1/: 이전 검사와 순서 보정 이력.

가정: 논리 byte와 RSS는 서로 대체하지 않는다. Fresh Python 초기값은 steady baseline이 아니다. 4개 작성 사례와 3개 실제 응답 seed의 기술 통계다.
