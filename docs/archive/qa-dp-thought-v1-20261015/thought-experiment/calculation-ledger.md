# 사고 실험 수치 원장

> QA-DP-THOUGHT-v1-20261015 / **전 항목 작성자 예상값. 실측 아님.**

이 문서는 [가정 원본](assumptions.json)과 공통 시나리오를 산술 집계한 결과다. [해석과 DP별 대응](../qa-dp-thought-experiment.md)을 먼저 읽는다. [전체 사건 / 목표 / 비용 / 메모리 JSON](predictions.json)은 72개 점추정과 그 분자 및 분모를 보존한다. 소수 자릿수는 계산 재현용이며 측정 정밀도가 아니다.

## 1. 72개 예상값과 점수

| QA / 단위 | 44A | 44B | 47A | 47B | 45A | 45B | 42A | 42B |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| QA01 첫 응답 지연시간 / s | 0.827458 (2점) | 0.823392 (2점) | 0.827458 (2점) | 0.856875 (1점) | 0.827458 (2점) | 0.783958 (2점) | 0.827458 (2점) | 0.829258 (2점) |
| QA02 지칭 대상 식별 정확도 / % | 97.727273 (4점) | 97.727273 (4점) | 97.727273 (4점) | 97.727273 (4점) | 97.727273 (4점) | 97.272727 (4점) | 97.727273 (4점) | 97.727273 (4점) |
| QA03 VIA 처리시간 / s | 2.350395 (4점) | 2.340289 (4점) | 2.350395 (4점) | 2.377105 (4점) | 2.350395 (4점) | 2.260132 (4점) | 2.350395 (4점) | 2.355368 (4점) |
| QA04 변경 범위 / modules/change | 2.333333 (2점) | 2.666667 (1점) | 2.333333 (2점) | 2.333333 (2점) | 2.333333 (2점) | 2.833333 (1점) | 2.333333 (2점) | 2.333333 (2점) |
| QA05 모델 사용 비용 / USD/bundle | 2.391576 (1점) | 2.391576 (1점) | 2.391576 (1점) | 2.392291 (1점) | 2.391576 (1점) | 2.55038 (0점) | 2.391576 (1점) | 2.391576 (1점) |
| QA06 최대 RAM 사용량 / MB | 386 (3점) | 394 (3점) | 386 (3점) | 409 (3점) | 386 (3점) | 434 (2점) | 386 (3점) | 446 (2점) |
| QA07 부하 추가 지연 / ms | 70 (4점) | 68 (4점) | 70 (4점) | 75 (4점) | 70 (4점) | 72.5 (4점) | 70 (4점) | 70 (4점) |
| QA08 병행 수정 중복 / modules/pair | 0.5 (3점) | 0.666667 (2점) | 0.5 (3점) | 0.5 (3점) | 0.5 (3점) | 0.666667 (2점) | 0.5 (3점) | 0.5 (3점) |
| QA09 음성 전달 지연 / ms | 300 (3점) | 300 (3점) | 300 (3점) | 300 (3점) | 300 (3점) | 300 (3점) | 300 (3점) | 300 (3점) |

모든 A 열은 같은 분석 기준 구조다. B는 해당 DP만 바꾼 조건이며 여러 B의 차이를 합산하지 않는다. 점수는 canonical QA 문서의 구간으로 반올림 전에 계산한다.

## 2. 시나리오별 시간 원장

각 셀은 **QA01 평균 초 / QA03 평균 초**다. 전체 값은 시나리오 평균 28개의 단순평균이 아니라 사건 120개 / 목표 76개의 원값 평균이다. 5회는 동일 점추정을 반복한 기대값이며 실행 관측 5회를 꾸며낸 것이 아니다.

| S | 사건 / 목표 | 공통 A | 44B | 47B | 45B | 42B |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| S01 | 4 / 4 | 0.8962 / 1.8662 | 0.8922 / 1.8582 | 0.8962 / 1.8662 | 0.7288 / 1.6387 | 0.8962 / 1.8662 |
| S02 | 6 / 6 | 0.94 / 1.98 | 0.936 / 1.972 | 0.94 / 1.98 | 0.94 / 1.98 | 0.94 / 1.98 |
| S03 | 4 / 4 | 1.01 / 2.15 | 1.006 / 2.142 | 1.155 / 2.195 | 1.01 / 2.15 | 1.01 / 2.15 |
| S04 | 2 / 2 | 1.01 / 2.15 | 1.006 / 2.142 | 1.155 / 2.195 | 1.01 / 2.15 | 1.01 / 2.15 |
| S05 | 3 / 1 | 0.7333 / 2.22 | 0.7287 / 2.21 | 0.8167 / 2.37 | 0.7333 / 2.22 | 0.7373 / 2.22 |
| S06 | 3 / 2 | 1.16 / 3.45 | 1.156 / 3.438 | 1.4067 / 3.67 | 1.16 / 3.45 | 1.16 / 3.45 |
| S07 | 2 / 2 | 1.235 / 2.375 | 1.231 / 2.367 | 1.555 / 2.595 | 1.235 / 2.375 | 1.235 / 2.375 |
| S08 | 3 / 1 | 1.01 / 4.53 | 1.006 / 4.514 | 1.18 / 5.04 | 1.01 / 4.53 | 1.012 / 4.548 |
| S09 | 2 / 1 | 1.01 / 4.3 | 1.006 / 4.284 | 1.155 / 4.39 | 1.01 / 4.3 | 1.01 / 4.3 |
| S10 | 2 / 2 | 1.205 / 2.245 | 1.201 / 2.237 | 1.205 / 2.245 | 0.87 / 1.79 | 1.205 / 2.245 |
| S11 | 5 / 3 | 1.072 / 2.4533 | 1.0676 / 2.4447 | 1.072 / 2.4533 | 0.97 / 2.23 | 1.0744 / 2.4533 |
| S12 | 2 / 2 | 1.43 / 2.47 | 1.426 / 2.462 | 1.43 / 2.47 | 1.295 / 2.315 | 1.43 / 2.47 |
| S13 | 2 / 2 | 1.205 / 2.245 | 1.201 / 2.237 | 1.205 / 2.245 | 0.87 / 1.79 | 1.205 / 2.245 |
| S14 | 3 / 3 | 0.94 / 1.98 | 0.936 / 1.972 | 0.94 / 1.98 | 0.94 / 1.98 | 0.94 / 1.98 |
| S15 | 4 / 3 | 0.8413 / 2.1483 | 0.8373 / 2.139 | 0.8988 / 2.1917 | 0.7362 / 2.0083 | 0.8427 / 2.1543 |
| S16 | 8 / 4 | 0.64 / 2.32 | 0.636 / 2.308 | 0.64 / 2.32 | 0.64 / 2.32 | 0.643 / 2.338 |
| S17 | 15 / 5 | 0.6133 / 2.716 | 0.6093 / 2.7016 | 0.6133 / 2.716 | 0.6133 / 2.716 | 0.6173 / 2.7448 |
| S18 | 7 / 3 | 0.6357 / 2.1167 | 0.6314 / 2.1053 | 0.6357 / 2.1167 | 0.6357 / 2.1167 | 0.6391 / 2.1287 |
| S19 | 3 / 1 | 0.62 / 1.64 | 0.6153 / 1.63 | 0.62 / 1.64 | 0.62 / 1.64 | 0.624 / 1.64 |
| S20 | 5 / 1 | 0.624 / 3.98 | 0.62 / 3.96 | 0.624 / 3.98 | 0.624 / 3.98 | 0.6276 / 3.998 |
| S21 | 2 / 2 | 1.43 / 2.47 | 1.426 / 2.462 | 1.43 / 2.47 | 1.095 / 2.015 | 1.43 / 2.47 |
| S22 | 7 / 5 | 0.6743 / 1.924 | 0.6703 / 1.9144 | 0.6743 / 1.924 | 0.6743 / 1.924 | 0.676 / 1.9312 |
| S23 | 3 / 1 | 0.6133 / 1.66 | 0.6093 / 1.652 | 0.6133 / 1.66 | 0.6133 / 1.66 | 0.6173 / 1.66 |
| S24 | 4 / 2 | 0.6388 / 3.485 | 0.6348 / 3.467 | 0.6388 / 3.485 | 0.6388 / 3.485 | 0.6402 / 3.494 |
| S25 | 2 / 2 | 0.695 / 1.615 | 0.691 / 1.607 | 0.695 / 1.615 | 0.695 / 1.615 | 0.695 / 1.615 |
| S26 | 8 / 3 | 0.7075 / 3.6467 | 0.7035 / 3.6293 | 0.7075 / 3.6467 | 0.7075 / 3.6467 | 0.709 / 3.6527 |
| S27 | 5 / 5 | 1.098 / 2.11 | 1.094 / 2.102 | 1.098 / 2.11 | 0.83 / 1.746 | 1.098 / 2.11 |
| S28 | 4 / 4 | 0.83 / 1.82 | 0.826 / 1.812 | 0.83 / 1.82 | 0.83 / 1.82 | 0.83 / 1.82 |

## 3. QA02 예상 오답 위치

각 항목은 5회 중 1회 틀린다는 정수 근사 가정이다. 실제 실패 기록 / 특정 반복의 관측이 아니다. 나머지 binding은 5/5로 가정한다. 오답의 시간을 삭제하거나 정답 시간으로 바꾸지 않는다.

### 44A (47A/45A/42A/44B/42B와 동일)

예상 215/220 정답 = 97.727273%, 4점.

- `S06-B01`: 원형 궤적의 경계 항목 하나를 추가.
- `S07-B03`: 문서 전환 직후 뒤 지칭의 frame 연결 혼동.
- `S08-B04`: 부분 정정에서 보존 집합의 두 번째 항목 혼동.
- `S11-B04`: 비용 비교와 일정 설명의 관계 혼동.
- `S21-B02`: 중단된 실제 전달 범위의 다음 부분 경계 혼동.

### 47B

예상 215/220 정답 = 97.727273%, 4점.

- `S06-B01`: 원형 궤적의 경계 항목 하나를 추가.
- `S08-B04`: 부분 정정에서 보존 집합의 두 번째 항목 혼동.
- `S11-B04`: 비용 비교와 일정 설명의 관계 혼동.
- `S21-B02`: 중단된 실제 전달 범위의 다음 부분 경계 혼동.
- `S05-B02`: producer의 문서 revision 구별 이익 대신 두 지시의 producer 후보 대응 손실 1건을 가정. 동일 총 오류수.

### 45B

예상 214/220 정답 = 97.272727%, 4점.

- `S06-B01`: 원형 궤적의 경계 항목 하나를 추가.
- `S07-B03`: 문서 전환 직후 뒤 지칭의 frame 연결 혼동.
- `S08-B04`: 부분 정정에서 보존 집합의 두 번째 항목 혼동.
- `S11-B04`: 비용 비교와 일정 설명의 관계 혼동.
- `S21-B02`: 중단된 실제 전달 범위의 다음 부분 경계 혼동.
- `S12-B02`: 유효성 검사는 통과했으나 갱신된 구간과 과거 설명 사이 의미 관계를 잘못 게시하는 경우. 낡은 revision 무검사 사용을 가정하지 않음.

## 4. QA04 / QA08 변경 모듈과 교집합

**DESIGN_CHANGE_REVIEW**다. 아래 ID는 앞으로 Rust 구현에서 대응시킬 계획 식별자이며 저장소에 실재하는 Rust 파일이라고 주장하지 않는다. `via / lib / via::...`의 세 부분은 package / crate-target / 전체 module path다. 어댑터, 상태, 출력 소유권 기준으로 분리했다. 부모 module의 자체 연결 코드가 바뀌지 않는다고 가정하며 실제 바뀌면 가산해야 한다.

### 44A (모든 A 공통)

변경 합 14/6 = 2.333333개, 2점. 교집합 합 3/6 = 0.5개, 3점.

| 변경 | 모듈 수 | 계획 모듈 ID |
| --- | ---: | --- |
| C01 | 1 | `via / lib / via::adapters::model_wire` |
| C02 | 1 | `via / lib / via::adapters::agent_wire` |
| C03 | 2 | `via / lib / via::sources::screen_wire`<br>`via / lib / via::sources::history_wire` |
| C04 | 4 | `via / lib / via::task::result_sections`<br>`via / lib / via::response::content`<br>`via / lib / via::response::speech_segments`<br>`via / lib / via::ui::response_view` |
| C05 | 3 | `via / lib / via::context::compose`<br>`via / lib / via::history::delivery_receipt`<br>`via / lib / via::ui::response_view` |
| C06 | 3 | `via / lib / via::access::policy`<br>`via / lib / via::context::compose`<br>`via / lib / via::response::content` |

| 독립 변경 쌍 | 교집합 |
| --- | --- |
| C01 / C02 | 없음 |
| C01 / C03 | 없음 |
| C02 / C03 | 없음 |
| C04 / C05 | `via / lib / via::ui::response_view` |
| C04 / C06 | `via / lib / via::response::content` |
| C05 / C06 | `via / lib / via::context::compose` |

### 44B

변경 합 16/6 = 2.666667개, 1점. 교집합 합 4/6 = 0.666667개, 2점.

| 변경 | 모듈 수 | 계획 모듈 ID |
| --- | ---: | --- |
| C01 | 1 | `via / lib / via::adapters::model_wire` |
| C02 | 1 | `via / lib / via::adapters::agent_wire` |
| C03 | 2 | `via / lib / via::sources::screen_wire`<br>`via / lib / via::sources::history_wire` |
| C04 | 5 | `via / lib / via::task::result_sections`<br>`via / lib / via::response::composer`<br>`via / lib / via::response::publication_join`<br>`via / lib / via::response::speech_segments`<br>`via / lib / via::ui::response_view` |
| C05 | 3 | `via / lib / via::context::compose`<br>`via / lib / via::history::delivery_receipt`<br>`via / lib / via::ui::response_view` |
| C06 | 4 | `via / lib / via::access::policy`<br>`via / lib / via::context::compose`<br>`via / lib / via::response::composer`<br>`via / lib / via::response::publication_join` |

| 독립 변경 쌍 | 교집합 |
| --- | --- |
| C01 / C02 | 없음 |
| C01 / C03 | 없음 |
| C02 / C03 | 없음 |
| C04 / C05 | `via / lib / via::ui::response_view` |
| C04 / C06 | `via / lib / via::response::composer`<br>`via / lib / via::response::publication_join` |
| C05 / C06 | `via / lib / via::context::compose` |

### 47B

변경 합 14/6 = 2.333333개, 2점. 교집합 합 3/6 = 0.5개, 3점.

| 변경 | 모듈 수 | 계획 모듈 ID |
| --- | ---: | --- |
| C01 | 1 | `via / lib / via::adapters::model_wire` |
| C02 | 1 | `via / lib / via::adapters::agent_wire` |
| C03 | 2 | `via / lib / via::sources::screen_wire`<br>`via / lib / via::sources::history_wire` |
| C04 | 4 | `via / lib / via::task::result_sections`<br>`via / lib / via::response::content`<br>`via / lib / via::response::speech_segments`<br>`via / lib / via::ui::response_view` |
| C05 | 3 | `via / lib / via::context::compose`<br>`via / lib / via::history::delivery_receipt`<br>`via / lib / via::ui::response_view` |
| C06 | 3 | `via / lib / via::access::policy`<br>`via / lib / via::context::compose`<br>`via / lib / via::response::content` |

| 독립 변경 쌍 | 교집합 |
| --- | --- |
| C01 / C02 | 없음 |
| C01 / C03 | 없음 |
| C02 / C03 | 없음 |
| C04 / C05 | `via / lib / via::ui::response_view` |
| C04 / C06 | `via / lib / via::response::content` |
| C05 / C06 | `via / lib / via::context::compose` |

### 45B

변경 합 17/6 = 2.833333개, 1점. 교집합 합 4/6 = 0.666667개, 2점.

| 변경 | 모듈 수 | 계획 모듈 ID |
| --- | ---: | --- |
| C01 | 1 | `via / lib / via::adapters::model_wire` |
| C02 | 1 | `via / lib / via::adapters::agent_wire` |
| C03 | 2 | `via / lib / via::sources::screen_wire`<br>`via / lib / via::sources::history_wire` |
| C04 | 4 | `via / lib / via::task::result_sections`<br>`via / lib / via::response::content`<br>`via / lib / via::response::speech_segments`<br>`via / lib / via::ui::response_view` |
| C05 | 4 | `via / lib / via::memory::publisher`<br>`via / lib / via::memory::evidence_reader`<br>`via / lib / via::history::delivery_receipt`<br>`via / lib / via::ui::response_view` |
| C06 | 5 | `via / lib / via::access::policy`<br>`via / lib / via::memory::publisher`<br>`via / lib / via::memory::evidence_reader`<br>`via / lib / via::memory::repository`<br>`via / lib / via::response::content` |

| 독립 변경 쌍 | 교집합 |
| --- | --- |
| C01 / C02 | 없음 |
| C01 / C03 | 없음 |
| C02 / C03 | 없음 |
| C04 / C05 | `via / lib / via::ui::response_view` |
| C04 / C06 | `via / lib / via::response::content` |
| C05 / C06 | `via / lib / via::memory::evidence_reader`<br>`via / lib / via::memory::publisher` |

### 42B

변경 합 14/6 = 2.333333개, 2점. 교집합 합 3/6 = 0.5개, 3점.

| 변경 | 모듈 수 | 계획 모듈 ID |
| --- | ---: | --- |
| C01 | 1 | `via / lib / via::adapters::model_wire` |
| C02 | 1 | `via / lib / via::adapters::agent_wire` |
| C03 | 2 | `via / lib / via::sources::screen_wire`<br>`via / lib / via::sources::history_wire` |
| C04 | 4 | `via / lib / via::task::result_sections`<br>`via / lib / via::response::content`<br>`via / lib / via::response::speech_segments`<br>`via / lib / via::ui::response_view` |
| C05 | 3 | `via / lib / via::context::compose`<br>`via / lib / via::history::delivery_receipt`<br>`via / lib / via::ui::response_view` |
| C06 | 3 | `via / lib / via::access::policy`<br>`via / lib / via::context::compose`<br>`via / lib / via::response::content` |

| 독립 변경 쌍 | 교집합 |
| --- | --- |
| C01 / C02 | 없음 |
| C01 / C03 | 없음 |
| C02 / C03 | 없음 |
| C04 / C05 | `via / lib / via::ui::response_view` |
| C04 / C06 | `via / lib / via::response::content` |
| C05 / C06 | `via / lib / via::context::compose` |

## 5. QA05 토큰 비용 원장 요약

같은 28개 묶음 5회의 예상 평균이다. 이전 참조 견적은 출력 예산을 모두 사용했으나 이번 기대 사용량은 총 출력 예산의 1/4이다. reasoning을 제외한 것이 아니며 전체 출력 사용량의 가정이다. Realtime 6개 청구 항목은 참조 원장과 같다. 오류 후 자동 재시도는 가정하지 않지만 채택 전 재판단과 B 생산 / 갱신은 모두 포함한다.

| 안 | 의미 호출 수 | 의미 Text 입력 | 의미 이미지 입력 토큰 | 의미 총 출력 | Realtime USD | 의미 USD | 총 USD / 점수 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 44A | 126 | 1084000 | 108000 | 39380 | 2.04632 | 0.345256 | 2.391576 / 1점 |
| 44B | 126 | 1084000 | 108000 | 39380 | 2.04632 | 0.345256 | 2.391576 / 1점 |
| 47B | 137 | 1107500 | 96000 | 37580 | 2.04632 | 0.345971 | 2.392291 / 1점 |
| 45B | 187 | 1662000 | 108000 | 51300 | 2.04632 | 0.50406 | 2.55038 / 0점 |
| 42B | 126 | 1084000 | 108000 | 39380 | 2.04632 | 0.345256 | 2.391576 / 1점 |

개별 호출의 토큰 / 단가 산식은 predictions.json의 `model_cost_ledger`에 있다. 준비부터 마지막 후속 청구까지 범위이며 별도 무상 초기 B 결과를 주지 않는다. 가격은 기존 저장소의 2026-10-14 snapshot이다.

## 6. QA06 같은 순간 RAM 최고 구성

| 안 | 최고 시나리오 | 같은 순간 구성 MB | 합 / 점수 |
| --- | --- | --- | ---: |
| 44A | S08 | ui_webview_owned_shared_once: 130<br>runtime_libraries_shared_once: 38<br>core_tasks_and_db_resident: 66<br>http_audio_io_buffers: 28<br>screen_decoded_and_compressed_cache: 80<br>history_source_cache: 32<br>queues_candidate_records: 12 | 386 MB / 3점 |
| 44B | S08 | ui_webview_owned_shared_once: 130<br>runtime_libraries_shared_once: 38<br>core_tasks_and_db_resident: 66<br>http_audio_io_buffers: 28<br>screen_decoded_and_compressed_cache: 80<br>history_source_cache: 32<br>queues_candidate_records: 12<br>typed_queues_and_join_records: 8 | 394 MB / 3점 |
| 47B | S08 | ui_webview_owned_shared_once: 130<br>runtime_libraries_shared_once: 38<br>core_tasks_and_db_resident: 66<br>http_audio_io_buffers: 28<br>screen_decoded_and_compressed_cache: 80<br>history_source_cache: 32<br>queues_candidate_records: 12<br>candidate_sets_and_revisions: 7<br>parallel_crop_serialization_buffers: 16 | 409 MB / 3점 |
| 45B | S08 | ui_webview_owned_shared_once: 130<br>runtime_libraries_shared_once: 38<br>core_tasks_and_db_resident: 66<br>http_audio_io_buffers: 28<br>screen_decoded_and_compressed_cache: 80<br>history_source_cache: 32<br>queues_candidate_records: 12<br>published_relation_store_resident_pages: 28<br>publisher_and_reader_buffers: 12<br>coverage_and_revision_index: 8 | 434 MB / 2점 |
| 42B | S08 | ui_webview_owned_shared_once: 130<br>runtime_libraries_shared_once: 38<br>core_tasks_and_db_resident: 66<br>http_audio_io_buffers: 28<br>screen_decoded_and_compressed_cache: 80<br>history_source_cache: 32<br>queues_candidate_records: 12<br>work_service_private_runtime_heap: 32<br>second_store_resident_pages: 18<br>ipc_and_pending_receipts: 10 | 446 MB / 2점 |

모든 S의 동시 최고 가정을 JSON에 보존했다. 서로 다른 시점의 프로세스 최고값을 더하지 않는다. UI와 공용 라이브러리의 공유 페이지는 한 번만 세고 외부 앱과 클라우드 가중치는 제외한다. 실제 Windows RAM 계측값이 아니다.

## 7. QA07 부하 / 무부하 짝

| 안 | S05 loaded / idle | S11 loaded / idle | S18 loaded / idle | S19 loaded / idle | 평균 증가 ms / 점수 |
| --- | ---: | ---: | ---: | ---: | ---: |
| 44A | 1080 / 1010 | 1380 / 1290 | 770 / 720 | 740 / 670 | 70 / 4점 |
| 44B | 1074 / 1006 | 1374 / 1286 | 764 / 716 | 734 / 666 | 68 / 4점 |
| 47B | 1330 / 1240 | 1380 / 1290 | 770 / 720 | 740 / 670 | 75 / 4점 |
| 45B | 1080 / 1010 | 970 / 870 | 770 / 720 | 740 / 670 | 72.5 / 4점 |
| 42B | 1080 / 1010 | 1380 / 1290 | 770 / 720 | 740 / 670 | 70 / 4점 |

loaded는 QA01의 같은 사건 원값이다. 네 조건 각각 5회 동일 기대값을 사용하고 `max(0, mean_loaded−mean_idle)`를 네 조건에 평균한다.

## 8. QA09 음성 전달

모든 안에서 109개 의무 음성 × 5회 = 545건에 **300ms**를 가정한다. 채택 후 요약 준비 110ms + 첫 음성 생성 / 전송 150ms + 장치 출력 40ms = 300ms → **3점**. 사용자 발화와 앞 음성이 차지한 시간은 제외한다. 전체 음성 데이터 준비 900ms는 QA03의 경계이며 첫 음성 300ms와 다르다. 원본 grounding 이전 시간은 여기에 더하지 않는다. 내용 오류에 시간 벌점을 넣지 않는 사고 실험 원값이며 기능 검사나 실제 음향 관측 통과를 주장하지 않는다.
