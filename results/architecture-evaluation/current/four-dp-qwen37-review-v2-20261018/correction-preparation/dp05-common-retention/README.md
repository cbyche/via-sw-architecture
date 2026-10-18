# DP05 기본 재사용 연결 최소 교정

원본 BASIC120/DP05 36회와54문안은 불변이다. 이 source는 승인된 별도 `basic-dp05-reuse-corrected-v1` 36회/54문안을 위한 기본 계약 교정이며 tactic이 아니다. 번호 표현9회/18문안은 같은 교정 binary의 별도 진단이다.

## 실제 원문 역할과 수명

- VIA_COMPOSER/explicit acquire: host selected_reads의 직접 성공 반환만 공통 획득 원문 원장에 즉시 연결한다. compose/receiver 실패 여부와 무관하다. summary/model passage/미표시 facts/수신자 읽기는 넣지 않는다. 현재 판본·권한·허용 범위를 검사하며 exact source/revision/range 중복만 제거한다. source/revoke 변경 시 현재 valid_span 필터가 차단한다.
- B producer: 실제 읽어 host가 붙인 원문 supporting_passages는 기존 work_record 소유로 유지한다. 중복 영구 raw cache를 추가하지 않는다. next build는 기존 common+record를 받는다.
- 공통 prefix: 실제 VIA 보유 common 원문과 B의 현재 유효 record raw support만 임시 canonical raw view로 제공한다. source/revision/range/text/unit만 allowlist하며 derived explanation/조건 자유문구/모델 span 임의 필드를 제외한다. exact span 중복을 제거하고 prefix 종료 시 drop+logical buffer release한다. A/B/C 같은 필드로 실제 보유 원문을 받는다. build에는 B 기존 record가 있으므로 raw를 중복 주입하지 않는다.
- C receiver: 외부가 선택해 읽은 원문은 receiver goal evidence/receipt에 남으며 VIA common으로 자동 승격하지 않는다.

새 모델/요약/의미 record/목적별 capsule cache/읽기 cache/강제 DIRECT는 없다. 실제 모델이 선택하는 READ/DIRECT와 모든 토큰·비용·RAM을 기록한다. fixture/oracle/의미 조건/endpoint/receiver/모델/기존 cap/recovery는 불변이다.

## 기능 검사와 동결 준비

최종 source3모듈만 변경했다. cargo57개 PASS(기존47+원문 유지7+prefix 보호3), binary build 성공, 실제 Rust wire10개 프로토콜 수용검사 PASS다. wire는 deterministic offline double이며 모델 의미·성능 검사가 아니다.

QA04 교정 baseline rebase C04/C06 독립6개 actual offline 수용검사와 각58개 cargo guard도 모두 PASS다. C04=1/1/1, C06=2/3/2 실제 변경 모듈이다. section 갱신 전 실제 취득 원문이 새 판본 후 payload로 재주입되지 않도록 C04 lifecycle의 현재성 필터를 함께 적용했다. C01/C03 controls6은 독립 reviewer가 PASS를 보고했다.

원본3파일/binary의 정확 SHA와 준비 요약은 [readiness-summary.json](readiness-summary.json), [source-differences.json](source-differences.json)에 있다. [wire10 로그](offline-acceptance.jsonl), [최종57가드](cargo-test-model-free-final.txt)도 보존한다. 모든 준비의 실제 모델 API 호출은0이다. 모델 실행은 runtime의 별도 freeze/공유 예산에서 수행한다.

독립 source 검토에서는 읽기 직후 보존/실패 유지/판본·권한·범위/중복/역할 및 prefix allowlist 경계에 필수 수정이 없었다. 최초 QA04 시도의 sandbox ps 권한 실패는 후보 실행 전에 발생했으며 별도 실패 로그로 남겼다.
