# Evidence Status

> **PRELIMINARY_HARNESS_DIAGNOSTIC — NOT ACCEPTED AS FINAL QA EVIDENCE**

이 실행은 19개 행과 raw-data 보존 형식을 검증했지만, active QA contract 전체를 구현하지 못했다.

- QA-01~03은 physical audible endpoint 대신 renderer proxy를 사용했다.
- QA-04는 acoustic barge-in-to-last-audible-sample이 아니라 queue-clear proxy다.
- QA-05는 input-end 이후 semantic target resolution과 실제 presentation endpoint를 모두 포함하지 않았다.
- QA-12는 전체 semantic predicate가 아니라 4-field routing subset이다.
- QA-13/14는 active predicate pack의 일부만 구현했다.
- QA-15는 일부 scenario를 명시적 oracle 없이 기본 PASS로 처리했다.
- QA-11은 모든 applicable predicate를 평가하는 완전한 integrated oracle이 아니다.

Raw evidence는 측정기 개발 provenance로 보존한다. 이 결과의 수치를 최종 DP 비교, ASR 선정 또는 Architecture 결정에 사용하지 않는다. QA-01~15 공통 machine contract와 failure-sentinel qualification을 통과한 새 freeze만 후속 DP campaign의 입력이 될 수 있다.
