# DP01 / DP05 실제 QA04 변경과 수용 결과

2026-10-18. 기준 freeze `2e2da0eb88895cd02036f2e4219ac80373425202a810cb63b639e908c0a0f0e1`.
원본은 `calibration-v2/frozen/source`이며 기본 후보는 수정하지 않았다. 각 기능과 A/B/C는
독립 scratch에 적용하고 실제 Rust callback·원본 제공·후속 인계를 offline 응답기로 실행했다.
모델 API 호출은 0회이며, replay 시간을 QA01/03 성능으로 사용하지 않는다.

| DP / 과제 | A 변경 모듈 | B 변경 모듈 | C 변경 모듈 | 실제 수용한 기능 |
| --- | ---: | ---: | ---: | --- |
| 01 C05 | 1 | 1 | 1 | 실제 표시한 답변의 원본 ID/판본/구간 조회. 미표시·실패·없는 답변 제외 |
| 01 C06 | 3 | 3 | 4 | 원본 중간 구간 철회, 좌우 허용 구간 유지, 현재 근거 소비/생산 차단 |
| 05 C04 | 1 | 1 | 1 | artifact 지정 section 수정, 다른 section·Task 보존, 전달 이력/대기 참조와 다음 인계 갱신 |
| 05 C06 | 2 | 3 | 2 | manifest·원본 읽기·기존 capsule/record의 일부 철회와 현재 epoch 유지 |

공통 대조 과제 C01=1, C03=2의 실행 증거는 runtime 담당이 별도로 보존했다. 그 값을 포함해
사전 고정한 네 과제를 동일 가중 평균하면 01 A/B/C는 **1.75/1.75/2.00**, 05는
**1.50/1.75/1.50**이다. 기능 두 과제만 평균하면 각각 **2.00/2.00/2.50**,
**1.50/2.00/1.50**이다. 이는 해당 네 변경의 범위이며 전반적인 유지보수 우열이 아니다.

## 차이가 난 이유와 동점의 의미

01의 부분 철회는 공통 lifecycle, 원본 권한, 요청 fallback을 바꾼다. C는 별도 관계 producer의
원본 읽기도 허용 잔여 구간으로 고쳐야 해 한 모듈이 더 필요했다. A/B에 없는 producer의
수정은 그 안의 실제 patch에서 제외하고 해당 독립 binary를 다시 수용 검사했다.

05의 부분 철회는 lifecycle과 원본 제공 경로를 바꾼다. B는 지속 업무 record producer도
수정했다. 반면 실제 전달 출처 조회와 artifact section 갱신은 기존 lifecycle/source 계약으로
표현할 수 있어 각 안이 같은 한 모듈에서 수용됐다. 모듈을 재편해 차이를 만들지 않았다.

## 수용 범위

- C05: 실제 완료·표시 ledger만 조회하며 실패·미표시 proposal은 NO_MATCH. 과거 출처와
  현재 usable/stale/revoked를 구분하고 반복 조회는 상태를 바꾸지 않는다.
- C06: 철회된 구간의 실제 요청을 거절하면서 양쪽 구간을 제공한다. 원문 view는 정확한
  허용 조각으로 보존한다. 일부 의존성이 철회된 파생 전달 답변은 host가 새 문장으로
  재구성하지 않고 전체 폐기하며, 독립적인 유효 답변은 보존한다.
- C06: 이전 record의 conditions/unknowns 및 publication coverage.gaps처럼 span이 없는
  파생 자유문도 권한 변경 때 폐기한다. 해당 필드까지 과거 허용 원본의 marker를 넣고,
  이후 모든 실제 모델 payload에 재등장하지 않음을 검사했다. 유효한 독립 record는 남는다.
- C04: section revision과 별도 public source revision을 갱신해 이후 manifest·receiver가
  새 section 본문을 사용한다. stale/duplicate/wrong-task update는 거절한다. B의 구판
  record와 출처 없는 부속 설명을 새 생산 이전에 차단한다. 음성은 OFF이며 대기 section
  참조 일관성만 검사했다.

각 폴더의 `actual-A.patch`/`actual-B.patch`/`actual-C.patch`, `source-A`/`source-B`/`source-C`,
`tests-A.txt`/`tests-B.txt`/`tests-C.txt`, `source-hashes.json`, `acceptance-runs.jsonl`과
상위 `acceptance-driver.py`로 구현·실행을 확인할 수 있다. 분모가 다른 첫 combined-tree
개발 집계와 v1 결과는 이전 QA04 경로에 보존했으며 이 결과와 혼합하지 않는다.

개발 경과시간은 최초 scratch 생성부터 완료까지이며 실제 calibration 진단·협업 대기가
포함된다. 과제별 집중 개발시간이나 개발자 생산성 측정값이 아니다. 매 타이핑을 기록한
수정 횟수도 없으므로 수용 실패 횟수와 저장된 revision/snapshot을 넘어 재작성 횟수를
만들어내지 않는다. 기능 요구는 동일하지만 이 유한 typed-event PoC가 제품 API·재시작·
실제 외부 Agent의 취소/자료 회수를 검증한 결과는 아니다.
