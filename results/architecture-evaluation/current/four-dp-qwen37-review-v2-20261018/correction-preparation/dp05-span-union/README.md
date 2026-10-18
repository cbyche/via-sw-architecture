# DP05 집계 span 최소 계약 교정

이전 `basic-dp05-reuse-corrected-v1`은11행 완료/25미실행 상태로 중단했다. 모든 실제 호출·실패·중간 수동판정은 보존한다. 새 최종 phase는 `basic-dp05-reuse-union-corrected-v2`36회와 같은 binary의 번호 표현 진단9회다.

## 추가 변경 범위

기존 공통 원문 취득/임시 prefix 교정3모듈 중 `dp05_sources.rs`1파일만 추가 수정했다. 집계 `source_spans`에서 같은 원본/판본/행 단위의 실제 공급 범위가 인접·겹치며 빈틈없이 요청 범위를 덮으면 허용한다. 현재 source 권한/판본과 모든 제공 조각을 재검사한다. 빈틈·다른 원본·다른 판본·다른 단위·철회/권한 밖은 허용하지 않는다.

기존 결과 스키마에 unit이 생략되면 one_based_line_inclusive이다. 명시한 잘못된 단위 문자열/숫자/불리언/null은 거절한다. fact별 정확 quote 원문/행/현재성 검사는 바꾸지 않았다. 모델·prompt·fixture·oracle·endpoint·cap·복구 횟수도 불변이다.

## 실제 준비 검사

- cargo65개 PASS(기존57+집계 coverage8).
- 구 binary에서 ABC 인접 집계 거절3개를 실제 wire로 재현했다.
- 새 binary에서 ABC 인접 허용/빈틈 거절/quote 불일치 거절9개 실제 wire PASS다.
- 기존 공통 취득·실패 후 보존·정정·철회·범위 축소10개 실제 wire를 새 binary에 재적용해 PASS했다.
- QA04 C04/C06 독립6개 실제 offline 수용검사와 각66개 guard PASS다. 변경 모듈 수 C04=1/1/1,C06=2/3/2를 유지했다.

모든 wire는 프로토콜 기능 검사 double이며 실제 모델 성능 검사가 아니다. 준비의 모델 API 호출은0이다. [정확 SHA/증거 요약](readiness-summary.json), [추가1파일과 기본 대비3파일](source-differences.json), [독립 읽기 검토 대상 diff](proposed.patch)를 보존한다. 실제 실행은 runtime의 별도 동결·공유 예산으로만 한다.
