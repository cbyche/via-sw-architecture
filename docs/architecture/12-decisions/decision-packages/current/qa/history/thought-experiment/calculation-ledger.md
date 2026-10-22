# 구조별 사고 실험 계산 원장

> **이관한 원전 이력:** 본문의 현재·최신·실측 전·검토 완료는 원래 날짜와 대상 범위의 기록이다. 현재 네 DP의 정본 상태와 결과는 [QA 기준 안내](../../README.md) 및 [공통 QA v3](../../../evaluation/common-qa-v3/README.md)에서 확인한다. 이 기록을 최신 실측 또는 현재 설계 승인으로 계승하지 않는다.


> **QA-DP-THOUGHT-v2-20261015 / 작성자 예상치 / 실측 아님**

[발표용 비교표](../qa-dp-thought-experiment.md) / [가정 원본](assumptions.json) / [모든 계산 JSON](predictions.json)

시간과 정확도는 DP별 관련 사건을 사용한다. QA03은 선택 목표의 전체 인과 경로를 보존한다. QA05는 기존 배점 단위인 전체 COMMON-28 묶음의 비용이며 관련 전경 의미 호출 비용도 따로 표시한다. 5회 반복은 같은 기대값이며 실제 관측 5개가 아니다.

## 1. 전체 결과

| QA / 단위 | 44A | 44B | 47A | 47B | 45A | 45B | 42A | 42B |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| QA01 첫 응답 지연시간 / s | 0.636357 (3점) | 0.648167 (3점) | 0.792111 (2점) | 1.173667 (0점) | 0.902412 (1점) | 0.665235 (3점) | 0.540739 (3점) | 0.549145 (3점) |
| QA02 지칭 대상 식별 정확도 / % | 97.733333 (4점) | 97.733333 (4점) | 97.85 (4점) | 96.275 (3점) | 98.4 (4점) | 98.2 (4점) | 98.25 (4점) | 98.25 (4점) |
| QA03 VIA 처리시간 / s | 2.4523 (4점) | 2.4705 (4점) | 2.684133 (3점) | 3.142 (3점) | 2.402353 (4점) | 2.166118 (4점) | 2.436937 (4점) | 2.464625 (4점) |
| QA04 변경 범위 / modules/change | 4 (0점) | 6 (0점) | 3 (1점) | 4.5 (0점) | 3 (1점) | 4 (0점) | 3 (1점) | 4.333333 (0점) |
| QA05 모델 사용 비용 / USD/COMMON-28 bundle | 2.422239 (1점) | 2.422239 (1점) | 2.422239 (1점) | 2.44784 (1점) | 2.422239 (1점) | 2.638279 (0점) | 2.422239 (1점) | 2.422239 (1점) |
| QA06 최대 RAM 사용량 / MB | 386 (3점) | 394 (3점) | 386 (3점) | 409 (3점) | 349 (4점) | 397 (3점) | 386 (3점) | 446 (2점) |
| QA07 부하 추가 지연 / ms | 70 (4점) | 68 (4점) | 70 (4점) | 180 (1점) | 90 (3점) | 110 (2점) | 70 (4점) | 74 (4점) |
| QA08 병행 수정 중복 / modules/pair | 1 (1점) | 3 (0점) | 0 (6점) | 0 (6점) | 0.333333 (4점) | 0.666667 (2점) | 0.333333 (4점) | 1 (1점) |
| QA09 음성 전달 지연 / ms | 300 (3점) | 300 (3점) | 300 (3점) | 300 (3점) | 300 (3점) | 300 (3점) | 300 (3점) | 300 (3점) |

## DP-44

응답 42개, 목표 20개, 지칭 15개. 선정 이유: 입력과 Agent 알림, 정정, 취소, 음성 중단 또는 채널 전환이 실행 연결과 게시 조건을 갱신하는 사건.

### QA01 사건별 처리 단계

모델 호출의 전송 / 첫 토큰 고정 140ms + 입력 1,000토큰당 10ms + 총 출력 토큰당 2ms를 동일 가정한다. 모두 설계용 예산이며 모델 성능 실측이 아니다. 일반 현재 응답의 첫 내용에는 48토큰, 원본 이력의 의미 관계를 함께 푸는 호출에는 500토큰을 예상한다. 화면 지칭은 A/B 모두 64~96토큰의 최종 GroundingProposal 전체가 완성되고 채택된 후 대상과 처리 범위 카드를 표시하며, 그 뒤 공통 상세 설명 호출 650토큰을 별도로 센다. 이 500은 강제 API 제한이 아닌 의미 판단량의 가정이다. producer 중간 결과는 전체 출력 후 결합에 전달한다. A는 원본 이력과 현재 요청을 한 호출로 해석하며 유효 cache와 부분 patch를 허용한다. B도 native 후보와 부분 재사용, 화면 선행 작업을 허용한다.

| 사건 | 구조 경로 | A ms | B ms | B−A ms |
| --- | --- | ---: | ---: | ---: |
| S05-A01 | task | 448 | 460 | 12 |
| S05-A02 | task | 448 | 460 | 12 |
| S05-U01 | fresh | 934 | 944 | 10 |
| S08-A01 | task | 448 | 460 | 12 |
| S08-U01 | expansion | 1334 | 1346 | 12 |
| S08-U02 | partial | 640 | 652 | 12 |
| S11-A01 | task | 448 | 460 | 12 |
| S11-A02 | task | 448 | 460 | 12 |
| S11-U01 | semantic | 1762 | 1772 | 10 |
| S11-U02 | cache | 593 | 605 | 12 |
| S11-U03 | current_comparison | 1672 | 1684 | 12 |
| S18-A01 | task | 448 | 460 | 12 |
| S18-A02 | task | 448 | 460 | 12 |
| S18-A03 | task | 448 | 460 | 12 |
| S18-A04 | task | 448 | 460 | 12 |
| S18-U01 | task | 658 | 668 | 10 |
| S18-U02 | explicit | 608 | 620 | 12 |
| S18-U03 | explicit | 608 | 620 | 12 |
| S19-A01 | task | 448 | 460 | 12 |
| S19-A02 | task | 448 | 460 | 12 |
| S19-U01 | simple | 618 | 628 | 10 |
| S20-A01 | task | 448 | 460 | 12 |
| S20-A02 | task | 448 | 460 | 12 |
| S20-A03 | task | 448 | 460 | 12 |
| S20-U01 | task | 608 | 620 | 12 |
| S20-U02 | task | 608 | 620 | 12 |
| S21-U01 | semantic | 1672 | 1684 | 12 |
| S21-U02 | cache | 593 | 605 | 12 |
| S22-A01 | task | 448 | 460 | 12 |
| S22-A02 | task | 448 | 460 | 12 |
| S22-U01 | task | 608 | 620 | 12 |
| S22-U02 | task | 608 | 620 | 12 |
| S22-U03 | task | 608 | 620 | 12 |
| S22-U04 | task | 608 | 620 | 12 |
| S22-U05 | task | 608 | 620 | 12 |
| S23-A01 | task | 448 | 460 | 12 |
| S23-A02 | task | 448 | 460 | 12 |
| S23-U01 | task | 608 | 620 | 12 |
| S24-A01 | task | 448 | 460 | 12 |
| S24-U01 | task | 608 | 620 | 12 |
| S24-U02 | task | 443 | 455 | 12 |
| S24-U03 | explicit | 608 | 620 | 12 |

평균: A 26727/42 = 636.357143ms; B 27223/42 = 648.166667ms. 오답 시간 가산 / 제외 없음.

### QA03 목표별 전체 인과 경로

| 목표 | A ms | B ms |
| --- | ---: | ---: |
| S05-G01 | 2414 | 2424 |
| S08-G01 | 4126 | 4162 |
| S11-G01 | 2962 | 2972 |
| S11-G02 | 1797 | 1809 |
| S11-G03 | 2872 | 2884 |
| S18-G01 | 1762 | 1772 |
| S18-G02 | 2260 | 2284 |
| S18-G03 | 2260 | 2284 |
| S19-G01 | 1518 | 1528 |
| S20-G01 | 3972 | 4008 |
| S21-G01 | 2872 | 2884 |
| S21-G02 | 1797 | 1809 |
| S22-G01 | 1712 | 1724 |
| S22-G02 | 2260 | 2284 |
| S22-G03 | 2260 | 2284 |
| S22-G04 | 1712 | 1724 |
| S22-G05 | 1712 | 1724 |
| S23-G01 | 1712 | 1724 |
| S24-G01 | 5519 | 5567 |
| S24-G02 | 1547 | 1559 |

목표 20개의 원값 평균: A 2.4523초 / B 2.4705초. 선택한 응답 이외의 같은 목표 후속 사건은 JSON의 additional_goal_events와 goal nodes에 보존한다.

### QA02 지칭별 정답률 가정

아래는 반복 실행의 성공 횟수가 아니라 각 지칭이 최종 대상 집합과 정확히 맞을 주관적 예상 확률이다. 어려운 사례를 따로 구분하고 단순 평균한다. 후보 확장 뒤 잔여 오류 위험이며 첫 판단 오류를 최종 오답으로 세지 않는다.

| 지칭 | 위험 유형 | A % | B % |
| --- | --- | ---: | ---: |
| S05-B01 | fresh | 98.5 | 98.5 |
| S05-B02 | fresh | 98.5 | 98.5 |
| S08-B01 | expansion | 96 | 96 |
| S08-B02 | expansion | 96 | 96 |
| S08-B03 | partial | 97 | 97 |
| S08-B04 | partial | 97 | 97 |
| S11-B01 | history_semantic | 97 | 97 |
| S11-B02 | history_cache | 99 | 99 |
| S11-B03 | history_current_comparison | 97 | 97 |
| S11-B04 | history_current_comparison | 97 | 97 |
| S18-B01 | history_explicit | 99 | 99 |
| S18-B02 | history_explicit | 99 | 99 |
| S21-B01 | history_semantic | 97 | 97 |
| S21-B02 | history_cache | 99 | 99 |
| S24-B01 | history_explicit | 99 | 99 |

### A: QA04 / QA08 계획 변경 집합

**DESIGN_CHANGE_REVIEW**. 아래 package / crate / module ID는 계획 식별자다. 실제 Rust 코드 diff가 아니다. 관련 변경 과제의 공통 UI / adapter 수정도 모두 센다.

| 과제 | 수 | 계획 module ID |
| --- | ---: | --- |
| C04 | 4 | `via / lib / via::task::result_sections`<br>`via / lib / via::response::content`<br>`via / lib / via::response::speech_segments`<br>`via / lib / via::ui::response_view` |
| C06 | 4 | `via / lib / via::access::policy`<br>`via / lib / via::context::compose`<br>`via / lib / via::response::content`<br>`via / lib / via::grounding::joint` |

| 독립 변경 쌍 | 공통 모듈 |
| --- | --- |
| C04 / C06 | `via / lib / via::response::content` |

변경 평균 4개; 교집합 평균 1개.


### B: QA04 / QA08 계획 변경 집합

**DESIGN_CHANGE_REVIEW**. 아래 package / crate / module ID는 계획 식별자다. 실제 Rust 코드 diff가 아니다. 관련 변경 과제의 공통 UI / adapter 수정도 모두 센다.

| 과제 | 수 | 계획 module ID |
| --- | ---: | --- |
| C04 | 6 | `via / lib / via::task::result_sections`<br>`via / lib / via::response::composer`<br>`via / lib / via::response::publication_join`<br>`via / lib / via::response::speech_segments`<br>`via / lib / via::ui::response_view`<br>`via / lib / via::dialogue::stage_ports` |
| C06 | 6 | `via / lib / via::access::policy`<br>`via / lib / via::context::compose`<br>`via / lib / via::response::composer`<br>`via / lib / via::response::publication_join`<br>`via / lib / via::dialogue::stage_ports`<br>`via / lib / via::grounding::joint` |

| 독립 변경 쌍 | 공통 모듈 |
| --- | --- |
| C04 / C06 | `via / lib / via::dialogue::stage_ports`<br>`via / lib / via::response::composer`<br>`via / lib / via::response::publication_join` |

변경 평균 6개; 교집합 평균 3개.

### QA05 비용과 QA06 RAM

| 안 | 전경 관련 의미 호출 USD | 전체 의미 USD | 공통 음성 USD | 전체 USD / 점수 | 최고 RAM / 점수 |
| --- | ---: | ---: | ---: | ---: | ---: |
| A | 0.136279 | 0.375919 | 2.04632 | 2.422239 / 1점 | 386 MB / 3점 |
| B | 0.136279 | 0.375919 | 2.04632 | 2.422239 / 1점 | 394 MB / 3점 |

전체 비용에는 28개 각 초기 상태의 준비와 갱신을 모두 포함한다. DP-45 B는 초기 56회 의미 관계 생산과 S12 원본 변경의 1회 갱신을 센다. 명시 ID와 전달 receipt 게시에는 모델 호출을 강제하지 않는다. 초기 작업은 2회 × 1.28초 = 2.56초로 입력 제공 전 5초 안에 완료된다는 가정이며 무료 ready store가 아니다. foreground에서 이미 센 갱신은 중복 청구하지 않는다. 원본 가격은 2026-10-14 저장소 snapshot이다.

- A RAM 최고 시점 S08: ui_webview_owned_shared_once 130MB + runtime_libraries_shared_once 38MB + core_tasks_and_db_resident 66MB + http_audio_io_buffers 28MB + screen_decoded_and_compressed_cache 80MB + history_source_cache 32MB + queues_candidate_records 12MB = 386MB.
- B RAM 최고 시점 S08: ui_webview_owned_shared_once 130MB + runtime_libraries_shared_once 38MB + core_tasks_and_db_resident 66MB + http_audio_io_buffers 28MB + screen_decoded_and_compressed_cache 80MB + history_source_cache 32MB + queues_candidate_records 12MB + typed_queues_and_join_records 8MB = 394MB.

### QA07 부하 증가와 QA09 음성 전달

| 안 / 사건 | idle ms | loaded ms | 추가 ms | 부하 지연 구성 ms |
| --- | ---: | ---: | ---: | --- |
| A / S05-U01 | 864 | 934 | 70 | model_admission 40, output_cpu 14, state_queue 16 |
| A / S11-U01 | 1672 | 1762 | 90 | model_admission 60, output_cpu 14, state_queue 16 |
| A / S18-U01 | 608 | 658 | 50 | model_admission 20, output_cpu 14, state_queue 16 |
| A / S19-U01 | 548 | 618 | 70 | model_admission 40, output_cpu 14, state_queue 16 |
| B / S05-U01 | 876 | 944 | 68 | model_admission 40, output_cpu 14, state_queue 10, join_validation 4 |
| B / S11-U01 | 1684 | 1772 | 88 | model_admission 60, output_cpu 14, state_queue 10, join_validation 4 |
| B / S18-U01 | 620 | 668 | 48 | model_admission 20, output_cpu 14, state_queue 10, join_validation 4 |
| B / S19-U01 | 560 | 628 | 68 | model_admission 40, output_cpu 14, state_queue 10, join_validation 4 |

QA09: 양안 37개 의무 음성 각각 요약 준비 110ms + 생성 / 전송 150ms + 장치 40ms = 300ms, 3점. 화면 / 이력 해석은 채택 전이므로 다시 더하지 않는다. 청취 불가능 시간과 오답 시간 벌점을 제외한다.

## DP-47

응답 18개, 목표 15개, 지칭 20개. 선정 이유: 현재 화면의 선택, 포인터, 궤적, 드래그, 화면 전환 또는 그 대상 정정을 해석하는 입력.

### QA01 사건별 처리 단계

모델 호출의 전송 / 첫 토큰 고정 140ms + 입력 1,000토큰당 10ms + 총 출력 토큰당 2ms를 동일 가정한다. 모두 설계용 예산이며 모델 성능 실측이 아니다. 일반 현재 응답의 첫 내용에는 48토큰, 원본 이력의 의미 관계를 함께 푸는 호출에는 500토큰을 예상한다. 화면 지칭은 A/B 모두 64~96토큰의 최종 GroundingProposal 전체가 완성되고 채택된 후 대상과 처리 범위 카드를 표시하며, 그 뒤 공통 상세 설명 호출 650토큰을 별도로 센다. 이 500은 강제 API 제한이 아닌 의미 판단량의 가정이다. producer 중간 결과는 전체 출력 후 결합에 전달한다. A는 원본 이력과 현재 요청을 한 호출로 해석하며 유효 cache와 부분 patch를 허용한다. B도 native 후보와 부분 재사용, 화면 선행 작업을 허용한다.

| 사건 | 구조 경로 | A ms | B ms | B−A ms |
| --- | --- | ---: | ---: | ---: |
| S03-U01 | native | 605 | 595 | -10 |
| S03-U02 | fresh | 684 | 1127 | 443 |
| S03-U03 | native | 605 | 595 | -10 |
| S03-U04 | fresh | 684 | 1127 | 443 |
| S04-U01 | fresh | 714 | 1127 | 413 |
| S04-U02 | native | 605 | 595 | -10 |
| S05-U01 | fresh | 934 | 1427 | 493 |
| S06-U01 | expansion | 1334 | 2371 | 1037 |
| S06-U02 | fresh | 744 | 1127 | 383 |
| S06-U03 | partial | 610 | 595 | -15 |
| S07-U01 | fresh | 774 | 1127 | 353 |
| S07-U02 | expansion | 1334 | 2371 | 1037 |
| S08-U01 | expansion | 1334 | 2371 | 1037 |
| S08-U02 | partial | 640 | 595 | -45 |
| S09-U01 | fresh | 684 | 1127 | 443 |
| S09-U02 | native | 605 | 595 | -10 |
| S15-U01 | fresh | 684 | 1127 | 443 |
| S17-U05 | fresh | 684 | 1127 | 443 |

평균: A 14258/18 = 792.111111ms; B 21126/18 = 1173.666667ms. 오답 시간 가산 / 제외 없음.

### QA03 목표별 전체 인과 경로

| 목표 | A ms | B ms |
| --- | ---: | ---: |
| S03-G01 | 2085 | 2075 |
| S03-G02 | 2164 | 2607 |
| S03-G03 | 2085 | 2075 |
| S03-G04 | 2164 | 2607 |
| S04-G01 | 2194 | 2607 |
| S04-G02 | 2085 | 2075 |
| S05-G01 | 2414 | 2907 |
| S06-G01 | 2814 | 3851 |
| S06-G02 | 4314 | 4682 |
| S07-G01 | 2254 | 2607 |
| S07-G02 | 2814 | 3851 |
| S08-G01 | 4126 | 5118 |
| S09-G01 | 4249 | 4682 |
| S15-G01 | 2164 | 2607 |
| S17-G05 | 2336 | 2779 |

목표 15개의 원값 평균: A 2.684133초 / B 3.142초. 선택한 응답 이외의 같은 목표 후속 사건은 JSON의 additional_goal_events와 goal nodes에 보존한다.

### QA02 지칭별 정답률 가정

아래는 반복 실행의 성공 횟수가 아니라 각 지칭이 최종 대상 집합과 정확히 맞을 주관적 예상 확률이다. 어려운 사례를 따로 구분하고 단순 평균한다. 후보 확장 뒤 잔여 오류 위험이며 첫 판단 오류를 최종 오답으로 세지 않는다.

| 지칭 | 위험 유형 | A % | B % |
| --- | --- | ---: | ---: |
| S03-B01 | native | 99.5 | 99.5 |
| S03-B02 | fresh | 98.5 | 96.5 |
| S03-B03 | native | 99.5 | 99.5 |
| S03-B04 | fresh | 98.5 | 96.5 |
| S04-B01 | fresh | 98.5 | 96.5 |
| S04-B02 | native | 99.5 | 99.5 |
| S05-B01 | fresh | 98.5 | 96.5 |
| S05-B02 | fresh | 98.5 | 96.5 |
| S06-B01 | expansion | 96 | 92 |
| S06-B02 | partial | 97 | 98.5 |
| S07-B01 | fresh | 98.5 | 96.5 |
| S07-B02 | expansion | 96 | 92 |
| S07-B03 | expansion | 96 | 92 |
| S08-B01 | expansion | 96 | 92 |
| S08-B02 | expansion | 96 | 92 |
| S08-B03 | partial | 97 | 98.5 |
| S08-B04 | partial | 97 | 98.5 |
| S09-B01 | native | 99.5 | 99.5 |
| S15-B01 | fresh | 98.5 | 96.5 |
| S17-B01 | fresh | 98.5 | 96.5 |

### A: QA04 / QA08 계획 변경 집합

**DESIGN_CHANGE_REVIEW**. 아래 package / crate / module ID는 계획 식별자다. 실제 Rust 코드 diff가 아니다. 관련 변경 과제의 공통 UI / adapter 수정도 모두 센다.

| 과제 | 수 | 계획 module ID |
| --- | ---: | --- |
| C03 | 2 | `via / lib / via::sources::screen_wire`<br>`via / lib / via::sources::history_wire` |
| C06 | 4 | `via / lib / via::access::policy`<br>`via / lib / via::context::compose`<br>`via / lib / via::response::content`<br>`via / lib / via::grounding::joint` |

| 독립 변경 쌍 | 공통 모듈 |
| --- | --- |
| C03 / C06 | 없음 |

변경 평균 3개; 교집합 평균 0개.


### B: QA04 / QA08 계획 변경 집합

**DESIGN_CHANGE_REVIEW**. 아래 package / crate / module ID는 계획 식별자다. 실제 Rust 코드 diff가 아니다. 관련 변경 과제의 공통 UI / adapter 수정도 모두 센다.

| 과제 | 수 | 계획 module ID |
| --- | ---: | --- |
| C03 | 2 | `via / lib / via::sources::screen_wire`<br>`via / lib / via::sources::history_wire` |
| C06 | 7 | `via / lib / via::access::policy`<br>`via / lib / via::context::compose`<br>`via / lib / via::grounding::speech_producer`<br>`via / lib / via::grounding::screen_producer`<br>`via / lib / via::grounding::candidate_repository`<br>`via / lib / via::grounding::binding`<br>`via / lib / via::response::content` |

| 독립 변경 쌍 | 공통 모듈 |
| --- | --- |
| C03 / C06 | 없음 |

변경 평균 4.5개; 교집합 평균 0개.

### QA05 비용과 QA06 RAM

| 안 | 전경 관련 의미 호출 USD | 전체 의미 USD | 공통 음성 USD | 전체 USD / 점수 | 최고 RAM / 점수 |
| --- | ---: | ---: | ---: | ---: | ---: |
| A | 0.084959 | 0.375919 | 2.04632 | 2.422239 / 1점 | 386 MB / 3점 |
| B | 0.11056 | 0.40152 | 2.04632 | 2.44784 / 1점 | 409 MB / 3점 |

전체 비용에는 28개 각 초기 상태의 준비와 갱신을 모두 포함한다. DP-45 B는 초기 56회 의미 관계 생산과 S12 원본 변경의 1회 갱신을 센다. 명시 ID와 전달 receipt 게시에는 모델 호출을 강제하지 않는다. 초기 작업은 2회 × 1.28초 = 2.56초로 입력 제공 전 5초 안에 완료된다는 가정이며 무료 ready store가 아니다. foreground에서 이미 센 갱신은 중복 청구하지 않는다. 원본 가격은 2026-10-14 저장소 snapshot이다.

- A RAM 최고 시점 S08: ui_webview_owned_shared_once 130MB + runtime_libraries_shared_once 38MB + core_tasks_and_db_resident 66MB + http_audio_io_buffers 28MB + screen_decoded_and_compressed_cache 80MB + history_source_cache 32MB + queues_candidate_records 12MB = 386MB.
- B RAM 최고 시점 S08: ui_webview_owned_shared_once 130MB + runtime_libraries_shared_once 38MB + core_tasks_and_db_resident 66MB + http_audio_io_buffers 28MB + screen_decoded_and_compressed_cache 80MB + history_source_cache 32MB + queues_candidate_records 12MB + candidate_sets_and_revisions 7MB + parallel_crop_serialization_buffers 16MB = 409MB.

### QA07 부하 증가와 QA09 음성 전달

| 안 / 사건 | idle ms | loaded ms | 추가 ms | 부하 지연 구성 ms |
| --- | ---: | ---: | ---: | --- |
| A / S05-U01 | 864 | 934 | 70 | model_admission 40, output_cpu 14, state_queue 16 |
| B / S05-U01 | 1247 | 1427 | 180 | model_admission 140, output_cpu 14, state_queue 16, producer_commit 10 |

QA09: 양안 18개 의무 음성 각각 요약 준비 110ms + 생성 / 전송 150ms + 장치 40ms = 300ms, 3점. 화면 / 이력 해석은 채택 전이므로 다시 더하지 않는다. 청취 불가능 시간과 오답 시간 벌점을 제외한다.

## DP-45

응답 17개, 목표 17개, 지칭 20개. 선정 이유: 실제 전달한 설명, 과거 결과 및 버전을 현재 요청과 연결하는 입력. 직접 파일 조회와 공통 UserMemory 선호는 제외.

### QA01 사건별 처리 단계

모델 호출의 전송 / 첫 토큰 고정 140ms + 입력 1,000토큰당 10ms + 총 출력 토큰당 2ms를 동일 가정한다. 모두 설계용 예산이며 모델 성능 실측이 아니다. 일반 현재 응답의 첫 내용에는 48토큰, 원본 이력의 의미 관계를 함께 푸는 호출에는 500토큰을 예상한다. 화면 지칭은 A/B 모두 64~96토큰의 최종 GroundingProposal 전체가 완성되고 채택된 후 대상과 처리 범위 카드를 표시하며, 그 뒤 공통 상세 설명 호출 650토큰을 별도로 센다. 이 500은 강제 API 제한이 아닌 의미 판단량의 가정이다. producer 중간 결과는 전체 출력 후 결합에 전달한다. A는 원본 이력과 현재 요청을 한 호출로 해석하며 유효 cache와 부분 patch를 허용한다. B도 native 후보와 부분 재사용, 화면 선행 작업을 허용한다.

| 사건 | 구조 경로 | A ms | B ms | B−A ms |
| --- | --- | ---: | ---: | ---: |
| S01-U02 | explicit | 608 | 620 | 12 |
| S01-U04 | cache | 428 | 440 | 12 |
| S10-U01 | explicit | 608 | 620 | 12 |
| S10-U02 | cache | 593 | 605 | 12 |
| S11-U01 | semantic | 1762 | 730 | -1032 |
| S11-U02 | cache | 593 | 605 | 12 |
| S11-U03 | current_comparison | 1672 | 1684 | 12 |
| S12-U01 | semantic | 1672 | 620 | -1052 |
| S12-U02 | updated_ready | 1672 | 620 | -1052 |
| S13-U01 | explicit | 608 | 620 | 12 |
| S13-U02 | cache | 593 | 605 | 12 |
| S15-U03 | explicit | 443 | 455 | 12 |
| S18-U02 | explicit | 608 | 620 | 12 |
| S18-U03 | explicit | 608 | 620 | 12 |
| S21-U01 | semantic | 1672 | 620 | -1052 |
| S21-U02 | cache | 593 | 605 | 12 |
| S24-U03 | explicit | 608 | 620 | 12 |

평균: A 15341/17 = 902.411765ms; B 11309/17 = 665.235294ms. 오답 시간 가산 / 제외 없음.

### QA03 목표별 전체 인과 경로

| 목표 | A ms | B ms |
| --- | ---: | ---: |
| S01-G02 | 1812 | 1824 |
| S01-G04 | 1632 | 1644 |
| S10-G01 | 1812 | 1824 |
| S10-G02 | 1797 | 1809 |
| S11-G01 | 2962 | 1934 |
| S11-G02 | 1797 | 1809 |
| S11-G03 | 2872 | 2884 |
| S12-G01 | 2872 | 1824 |
| S12-G02 | 2872 | 1824 |
| S13-G01 | 1812 | 1824 |
| S13-G02 | 1797 | 1809 |
| S15-G03 | 2095 | 2107 |
| S18-G02 | 2260 | 2272 |
| S18-G03 | 2260 | 2272 |
| S21-G01 | 2872 | 1824 |
| S21-G02 | 1797 | 1809 |
| S24-G01 | 5519 | 5531 |

목표 17개의 원값 평균: A 2.402353초 / B 2.166118초. 선택한 응답 이외의 같은 목표 후속 사건은 JSON의 additional_goal_events와 goal nodes에 보존한다.

### QA02 지칭별 정답률 가정

아래는 반복 실행의 성공 횟수가 아니라 각 지칭이 최종 대상 집합과 정확히 맞을 주관적 예상 확률이다. 어려운 사례를 따로 구분하고 단순 평균한다. 후보 확장 뒤 잔여 오류 위험이며 첫 판단 오류를 최종 오답으로 세지 않는다.

| 지칭 | 위험 유형 | A % | B % |
| --- | --- | ---: | ---: |
| S01-B01 | history_explicit | 99 | 99 |
| S01-B02 | history_cache | 99 | 99 |
| S10-B01 | history_explicit | 99 | 99 |
| S10-B02 | history_cache | 99 | 99 |
| S11-B01 | history_semantic | 97 | 96 |
| S11-B02 | history_cache | 99 | 99 |
| S11-B03 | history_current_comparison | 97 | 97 |
| S11-B04 | history_current_comparison | 97 | 97 |
| S12-B01 | history_semantic | 97 | 96 |
| S12-B02 | history_updated_ready | 97 | 96 |
| S13-B01 | history_explicit | 99 | 99 |
| S13-B02 | history_explicit | 99 | 99 |
| S13-B03 | history_cache | 99 | 99 |
| S13-B04 | history_cache | 99 | 99 |
| S15-B02 | history_explicit | 99 | 99 |
| S18-B01 | history_explicit | 99 | 99 |
| S18-B02 | history_explicit | 99 | 99 |
| S21-B01 | history_semantic | 97 | 96 |
| S21-B02 | history_cache | 99 | 99 |
| S24-B01 | history_explicit | 99 | 99 |

### A: QA04 / QA08 계획 변경 집합

**DESIGN_CHANGE_REVIEW**. 아래 package / crate / module ID는 계획 식별자다. 실제 Rust 코드 diff가 아니다. 관련 변경 과제의 공통 UI / adapter 수정도 모두 센다.

| 과제 | 수 | 계획 module ID |
| --- | ---: | --- |
| C03 | 2 | `via / lib / via::sources::screen_wire`<br>`via / lib / via::sources::history_wire` |
| C05 | 3 | `via / lib / via::context::compose`<br>`via / lib / via::history::delivery_receipt`<br>`via / lib / via::ui::response_view` |
| C06 | 4 | `via / lib / via::access::policy`<br>`via / lib / via::context::compose`<br>`via / lib / via::response::content`<br>`via / lib / via::grounding::joint` |

| 독립 변경 쌍 | 공통 모듈 |
| --- | --- |
| C03 / C05 | 없음 |
| C03 / C06 | 없음 |
| C05 / C06 | `via / lib / via::context::compose` |

변경 평균 3개; 교집합 평균 0.333333개.


### B: QA04 / QA08 계획 변경 집합

**DESIGN_CHANGE_REVIEW**. 아래 package / crate / module ID는 계획 식별자다. 실제 Rust 코드 diff가 아니다. 관련 변경 과제의 공통 UI / adapter 수정도 모두 센다.

| 과제 | 수 | 계획 module ID |
| --- | ---: | --- |
| C03 | 2 | `via / lib / via::sources::screen_wire`<br>`via / lib / via::sources::history_wire` |
| C05 | 4 | `via / lib / via::memory::publisher`<br>`via / lib / via::memory::evidence_reader`<br>`via / lib / via::history::delivery_receipt`<br>`via / lib / via::ui::response_view` |
| C06 | 6 | `via / lib / via::access::policy`<br>`via / lib / via::memory::publisher`<br>`via / lib / via::memory::evidence_reader`<br>`via / lib / via::memory::repository`<br>`via / lib / via::response::content`<br>`via / lib / via::grounding::joint` |

| 독립 변경 쌍 | 공통 모듈 |
| --- | --- |
| C03 / C05 | 없음 |
| C03 / C06 | 없음 |
| C05 / C06 | `via / lib / via::memory::evidence_reader`<br>`via / lib / via::memory::publisher` |

변경 평균 4개; 교집합 평균 0.666667개.

### QA05 비용과 QA06 RAM

| 안 | 전경 관련 의미 호출 USD | 전체 의미 USD | 공통 음성 USD | 전체 USD / 점수 | 최고 RAM / 점수 |
| --- | ---: | ---: | ---: | ---: | ---: |
| A | 0.06996 | 0.375919 | 2.04632 | 2.422239 / 1점 | 349 MB / 4점 |
| B | 0.0564 | 0.591959 | 2.04632 | 2.638279 / 0점 | 397 MB / 3점 |

전체 비용에는 28개 각 초기 상태의 준비와 갱신을 모두 포함한다. DP-45 B는 초기 56회 의미 관계 생산과 S12 원본 변경의 1회 갱신을 센다. 명시 ID와 전달 receipt 게시에는 모델 호출을 강제하지 않는다. 초기 작업은 2회 × 1.28초 = 2.56초로 입력 제공 전 5초 안에 완료된다는 가정이며 무료 ready store가 아니다. foreground에서 이미 센 갱신은 중복 청구하지 않는다. 원본 가격은 2026-10-14 저장소 snapshot이다.

- A RAM 최고 시점 S15: ui_webview_owned_shared_once 130MB + runtime_libraries_shared_once 38MB + core_tasks_and_db_resident 66MB + http_audio_io_buffers 28MB + screen_decoded_and_compressed_cache 45MB + history_source_cache 30MB + queues_candidate_records 12MB = 349MB.
- B RAM 최고 시점 S15: ui_webview_owned_shared_once 130MB + runtime_libraries_shared_once 38MB + core_tasks_and_db_resident 66MB + http_audio_io_buffers 28MB + screen_decoded_and_compressed_cache 45MB + history_source_cache 30MB + queues_candidate_records 12MB + published_relation_store_resident_pages 28MB + publisher_and_reader_buffers 12MB + coverage_and_revision_index 8MB = 397MB.

### QA07 부하 증가와 QA09 음성 전달

| 안 / 사건 | idle ms | loaded ms | 추가 ms | 부하 지연 구성 ms |
| --- | ---: | ---: | ---: | --- |
| A / S11-U01 | 1672 | 1762 | 90 | model_admission 60, output_cpu 14, state_queue 16 |
| B / S11-U01 | 620 | 730 | 110 | model_admission 60, output_cpu 14, state_queue 16, publisher_readback 20 |

QA09: 양안 15개 의무 음성 각각 요약 준비 110ms + 생성 / 전송 150ms + 장치 40ms = 300ms, 3점. 화면 / 이력 해석은 채택 전이므로 다시 더하지 않는다. 청취 불가능 시간과 오답 시간 벌점을 제외한다.

## DP-42

응답 69개, 목표 32개, 지칭 14개. 선정 이유: Request와 Task의 연결, 업무 제어, Agent 사실 반영 및 대화 재연결이 상태 소유 경계를 통과하는 사건.

### QA01 사건별 처리 단계

모델 호출의 전송 / 첫 토큰 고정 140ms + 입력 1,000토큰당 10ms + 총 출력 토큰당 2ms를 동일 가정한다. 모두 설계용 예산이며 모델 성능 실측이 아니다. 일반 현재 응답의 첫 내용에는 48토큰, 원본 이력의 의미 관계를 함께 푸는 호출에는 500토큰을 예상한다. 화면 지칭은 A/B 모두 64~96토큰의 최종 GroundingProposal 전체가 완성되고 채택된 후 대상과 처리 범위 카드를 표시하며, 그 뒤 공통 상세 설명 호출 650토큰을 별도로 센다. 이 500은 강제 API 제한이 아닌 의미 판단량의 가정이다. producer 중간 결과는 전체 출력 후 결합에 전달한다. A는 원본 이력과 현재 요청을 한 호출로 해석하며 유효 cache와 부분 patch를 허용한다. B도 native 후보와 부분 재사용, 화면 선행 작업을 허용한다.

| 사건 | 구조 경로 | A ms | B ms | B−A ms |
| --- | --- | ---: | ---: | ---: |
| S08-A01 | task | 448 | 466 | 18 |
| S08-U01 | expansion | 1334 | 1334 | 0 |
| S08-U02 | partial | 640 | 640 | 0 |
| S13-U01 | explicit | 608 | 608 | 0 |
| S13-U02 | cache | 593 | 593 | 0 |
| S15-A01 | task | 448 | 466 | 18 |
| S15-U03 | explicit | 443 | 443 | 0 |
| S16-A01 | task | 448 | 466 | 18 |
| S16-A02 | task | 448 | 466 | 18 |
| S16-A03 | task | 448 | 466 | 18 |
| S16-A04 | task | 448 | 466 | 18 |
| S16-U01 | task | 608 | 608 | 0 |
| S16-U02 | task | 608 | 608 | 0 |
| S16-U03 | task | 608 | 608 | 0 |
| S16-U04 | task | 608 | 608 | 0 |
| S17-A01 | task | 448 | 466 | 18 |
| S17-A02 | task | 448 | 466 | 18 |
| S17-A03 | task | 448 | 466 | 18 |
| S17-A04 | task | 448 | 466 | 18 |
| S17-A05 | task | 448 | 466 | 18 |
| S17-A06 | task | 448 | 466 | 18 |
| S17-A07 | task | 448 | 466 | 18 |
| S17-A08 | task | 448 | 466 | 18 |
| S17-A09 | task | 448 | 466 | 18 |
| S17-A10 | task | 448 | 466 | 18 |
| S17-U01 | task | 608 | 608 | 0 |
| S17-U02 | task | 608 | 608 | 0 |
| S17-U03 | task | 608 | 608 | 0 |
| S17-U04 | task | 608 | 608 | 0 |
| S17-U05 | fresh | 684 | 684 | 0 |
| S18-A01 | task | 448 | 466 | 18 |
| S18-A02 | task | 448 | 466 | 18 |
| S18-A03 | task | 448 | 466 | 18 |
| S18-A04 | task | 448 | 466 | 18 |
| S18-U01 | task | 658 | 662 | 4 |
| S18-U02 | explicit | 608 | 608 | 0 |
| S18-U03 | explicit | 608 | 608 | 0 |
| S19-A01 | task | 448 | 466 | 18 |
| S19-A02 | task | 448 | 466 | 18 |
| S20-A01 | task | 448 | 466 | 18 |
| S20-A02 | task | 448 | 466 | 18 |
| S20-A03 | task | 448 | 466 | 18 |
| S20-U01 | task | 608 | 608 | 0 |
| S20-U02 | task | 608 | 608 | 0 |
| S22-A01 | task | 448 | 466 | 18 |
| S22-A02 | task | 448 | 466 | 18 |
| S22-U01 | task | 608 | 608 | 0 |
| S22-U02 | task | 608 | 608 | 0 |
| S22-U03 | task | 608 | 608 | 0 |
| S22-U04 | task | 608 | 608 | 0 |
| S22-U05 | task | 608 | 608 | 0 |
| S23-A01 | task | 448 | 466 | 18 |
| S23-A02 | task | 448 | 466 | 18 |
| S23-U01 | task | 608 | 608 | 0 |
| S24-A01 | task | 448 | 466 | 18 |
| S24-U01 | task | 608 | 608 | 0 |
| S24-U02 | task | 443 | 443 | 0 |
| S24-U03 | explicit | 608 | 608 | 0 |
| S25-U01 | simple | 548 | 548 | 0 |
| S25-U02 | task | 608 | 608 | 0 |
| S26-A01 | task | 448 | 466 | 18 |
| S26-A02 | task | 448 | 466 | 18 |
| S26-U01 | task | 608 | 608 | 0 |
| S26-U03 | task | 608 | 608 | 0 |
| S26-U04 | task | 608 | 608 | 0 |
| S26-U05 | task | 608 | 608 | 0 |
| S26-U06 | task | 608 | 608 | 0 |
| S28-U03 | task | 608 | 608 | 0 |
| S28-U04 | task | 608 | 608 | 0 |

평균: A 37311/69 = 540.73913ms; B 37891/69 = 549.144928ms. 오답 시간 가산 / 제외 없음.

### QA03 목표별 전체 인과 경로

| 목표 | A ms | B ms |
| --- | ---: | ---: |
| S08-G01 | 4126 | 4168 |
| S13-G01 | 1812 | 1812 |
| S13-G02 | 1797 | 1797 |
| S15-G03 | 2095 | 2137 |
| S16-G01 | 2260 | 2302 |
| S16-G02 | 2260 | 2302 |
| S16-G03 | 2260 | 2302 |
| S16-G04 | 2260 | 2302 |
| S17-G01 | 2260 | 2302 |
| S17-G02 | 2808 | 2892 |
| S17-G03 | 2808 | 2892 |
| S17-G04 | 2808 | 2892 |
| S17-G05 | 2336 | 2378 |
| S18-G01 | 1762 | 1766 |
| S18-G02 | 2260 | 2302 |
| S18-G03 | 2260 | 2302 |
| S20-G01 | 3972 | 4014 |
| S22-G01 | 1712 | 1712 |
| S22-G02 | 2260 | 2302 |
| S22-G03 | 2260 | 2302 |
| S22-G04 | 1712 | 1712 |
| S22-G05 | 1712 | 1712 |
| S23-G01 | 1712 | 1712 |
| S24-G01 | 5519 | 5561 |
| S24-G02 | 1547 | 1547 |
| S25-G01 | 1448 | 1448 |
| S25-G02 | 1712 | 1712 |
| S26-G01 | 3424 | 3424 |
| S26-G02 | 3424 | 3424 |
| S26-G03 | 3972 | 4014 |
| S28-G03 | 1712 | 1712 |
| S28-G04 | 1712 | 1712 |

목표 32개의 원값 평균: A 2.436937초 / B 2.464625초. 선택한 응답 이외의 같은 목표 후속 사건은 JSON의 additional_goal_events와 goal nodes에 보존한다.

### QA02 지칭별 정답률 가정

아래는 반복 실행의 성공 횟수가 아니라 각 지칭이 최종 대상 집합과 정확히 맞을 주관적 예상 확률이다. 어려운 사례를 따로 구분하고 단순 평균한다. 후보 확장 뒤 잔여 오류 위험이며 첫 판단 오류를 최종 오답으로 세지 않는다.

| 지칭 | 위험 유형 | A % | B % |
| --- | --- | ---: | ---: |
| S08-B01 | expansion | 96 | 96 |
| S08-B02 | expansion | 96 | 96 |
| S08-B03 | partial | 97 | 97 |
| S08-B04 | partial | 97 | 97 |
| S13-B01 | history_explicit | 99 | 99 |
| S13-B02 | history_explicit | 99 | 99 |
| S13-B03 | history_cache | 99 | 99 |
| S13-B04 | history_cache | 99 | 99 |
| S15-B02 | history_explicit | 99 | 99 |
| S17-B01 | fresh | 98.5 | 98.5 |
| S18-B01 | history_explicit | 99 | 99 |
| S18-B02 | history_explicit | 99 | 99 |
| S24-B01 | history_explicit | 99 | 99 |
| S26-B01 | other | 99 | 99 |

### A: QA04 / QA08 계획 변경 집합

**DESIGN_CHANGE_REVIEW**. 아래 package / crate / module ID는 계획 식별자다. 실제 Rust 코드 diff가 아니다. 관련 변경 과제의 공통 UI / adapter 수정도 모두 센다.

| 과제 | 수 | 계획 module ID |
| --- | ---: | --- |
| C02 | 1 | `via / lib / via::adapters::agent_wire` |
| C04 | 4 | `via / lib / via::task::result_sections`<br>`via / lib / via::response::content`<br>`via / lib / via::response::speech_segments`<br>`via / lib / via::ui::response_view` |
| C06 | 4 | `via / lib / via::access::policy`<br>`via / lib / via::context::compose`<br>`via / lib / via::response::content`<br>`via / lib / via::grounding::joint` |

| 독립 변경 쌍 | 공통 모듈 |
| --- | --- |
| C02 / C04 | 없음 |
| C02 / C06 | 없음 |
| C04 / C06 | `via / lib / via::response::content` |

변경 평균 3개; 교집합 평균 0.333333개.


### B: QA04 / QA08 계획 변경 집합

**DESIGN_CHANGE_REVIEW**. 아래 package / crate / module ID는 계획 식별자다. 실제 Rust 코드 diff가 아니다. 관련 변경 과제의 공통 UI / adapter 수정도 모두 센다.

| 과제 | 수 | 계획 module ID |
| --- | ---: | --- |
| C02 | 1 | `via / lib / via::adapters::agent_wire` |
| C04 | 6 | `via / lib / via::task::result_sections`<br>`via / lib / via::response::content`<br>`via / lib / via::response::speech_segments`<br>`via / lib / via::ui::response_view`<br>`via / lib / via::work::event_contract`<br>`via / lib / via::dialogue::task_projection` |
| C06 | 6 | `via / lib / via::access::policy`<br>`via / lib / via::context::compose`<br>`via / lib / via::response::content`<br>`via / lib / via::work::event_contract`<br>`via / lib / via::dialogue::task_projection`<br>`via / lib / via::grounding::joint` |

| 독립 변경 쌍 | 공통 모듈 |
| --- | --- |
| C02 / C04 | 없음 |
| C02 / C06 | 없음 |
| C04 / C06 | `via / lib / via::dialogue::task_projection`<br>`via / lib / via::response::content`<br>`via / lib / via::work::event_contract` |

변경 평균 4.333333개; 교집합 평균 1개.

### QA05 비용과 QA06 RAM

| 안 | 전경 관련 의미 호출 USD | 전체 의미 USD | 공통 음성 USD | 전체 USD / 점수 | 최고 RAM / 점수 |
| --- | ---: | ---: | ---: | ---: | ---: |
| A | 0.194479 | 0.375919 | 2.04632 | 2.422239 / 1점 | 386 MB / 3점 |
| B | 0.194479 | 0.375919 | 2.04632 | 2.422239 / 1점 | 446 MB / 2점 |

전체 비용에는 28개 각 초기 상태의 준비와 갱신을 모두 포함한다. DP-45 B는 초기 56회 의미 관계 생산과 S12 원본 변경의 1회 갱신을 센다. 명시 ID와 전달 receipt 게시에는 모델 호출을 강제하지 않는다. 초기 작업은 2회 × 1.28초 = 2.56초로 입력 제공 전 5초 안에 완료된다는 가정이며 무료 ready store가 아니다. foreground에서 이미 센 갱신은 중복 청구하지 않는다. 원본 가격은 2026-10-14 저장소 snapshot이다.

- A RAM 최고 시점 S08: ui_webview_owned_shared_once 130MB + runtime_libraries_shared_once 38MB + core_tasks_and_db_resident 66MB + http_audio_io_buffers 28MB + screen_decoded_and_compressed_cache 80MB + history_source_cache 32MB + queues_candidate_records 12MB = 386MB.
- B RAM 최고 시점 S08: ui_webview_owned_shared_once 130MB + runtime_libraries_shared_once 38MB + core_tasks_and_db_resident 66MB + http_audio_io_buffers 28MB + screen_decoded_and_compressed_cache 80MB + history_source_cache 32MB + queues_candidate_records 12MB + work_service_private_runtime_heap 32MB + second_store_resident_pages 18MB + ipc_and_pending_receipts 10MB = 446MB.

### QA07 부하 증가와 QA09 음성 전달

| 안 / 사건 | idle ms | loaded ms | 추가 ms | 부하 지연 구성 ms |
| --- | ---: | ---: | ---: | --- |
| A / S05-U01 | 864 | 934 | 70 | model_admission 40, output_cpu 14, state_queue 16 |
| A / S11-U01 | 1672 | 1762 | 90 | model_admission 60, output_cpu 14, state_queue 16 |
| A / S18-U01 | 608 | 658 | 50 | model_admission 20, output_cpu 14, state_queue 16 |
| A / S19-U01 | 548 | 618 | 70 | model_admission 40, output_cpu 14, state_queue 16 |
| B / S05-U01 | 864 | 938 | 74 | model_admission 40, output_cpu 14, state_queue 8, ipc_projection 12 |
| B / S11-U01 | 1672 | 1766 | 94 | model_admission 60, output_cpu 14, state_queue 8, ipc_projection 12 |
| B / S18-U01 | 608 | 662 | 54 | model_admission 20, output_cpu 14, state_queue 8, ipc_projection 12 |
| B / S19-U01 | 548 | 622 | 74 | model_admission 40, output_cpu 14, state_queue 8, ipc_projection 12 |

QA09: 양안 61개 의무 음성 각각 요약 준비 110ms + 생성 / 전송 150ms + 장치 40ms = 300ms, 3점. 화면 / 이력 해석은 채택 전이므로 다시 더하지 않는다. 청취 불가능 시간과 오답 시간 벌점을 제외한다.
