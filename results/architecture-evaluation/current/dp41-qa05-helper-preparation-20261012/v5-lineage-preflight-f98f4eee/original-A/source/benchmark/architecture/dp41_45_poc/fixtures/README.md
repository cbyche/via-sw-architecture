# P21 실행 입력과 평가 전용 정답

`inputs.json`은 harness가 읽는 원천 상태와 사건 계획이다. 후보에게 그대로 보내지 않는다.
`fixtures.py`의 `public_inputs(case)`가 반환하는 allowlist만 후보 입력 경계로 전달한다.
그 반환값의 `initial.sources[].text`도 owner repository의 원문이며 최초 모델 context가 아니다.
모델은 동일한 bounded read port를 통해 실제 조회한 원문만 받는다.
owner metadata는 활성 Task와 최근 완료 Task 10개, 최근 interaction 10개의 식별자와 상태를
보여주며 전체 record 수와 이 범위를 함께 기록한다. 과거 응답 본문, 실제 들은 문장, 산출물 본문은
metadata에서 제외한다. 전체 이력은 owner에 유지하며 후보가 bounded query로 읽어야 한다.
metadata의 source handle 제공과 본문 조회 provenance는 서로 다른 기록이다.

`oracles.json`은 evaluator와 judge만 읽는다. C1~C6의 원문 조건, 금지 사실, canonical 발췌,
checkpoint와 final tail을 보존한다. 현재 artifact의 고정 정답 본문이나 결과를 stub에 공급하지 않는다.
생성된 runtime ID는 원천/업무/산출물의 역할과 provenance로 대응시키며 원문의 예시 ID와 문자열이
다르다는 이유로 실패시키지 않는다. S6-03의 비공개 계약 금액은 이 파일에만 있다.

`manifest.json`은 ASR-QA-v6/P21 원본 세 파일의 hash, 사례 순서, 21개 사례/27개 반응/19개 사이클,
활동 배경과 갱신/철회/drain/idle 조건을 기록한다. 원본이 바뀌면 `validate_fixtures()`가 오류를 반환한다.
fixture 구현은 후보 실행이나 모델 품질 측정 완료를 뜻하지 않는다.

| 입력 항목 | 사용 방법 |
| --- | --- |
| `initial` | source/Task/Execution/질문/대화/실제 전달/출력 세대의 원천 before-state |
| `events` | replay-only 고정 원천 offset. 실제 availability 이후 현재 envelope만 후보에게 전달 |
| `preparation.events` | S2-02의 실제 원천 준비 P0~P4. 마지막 사건 뒤 5초 동일 창을 준 뒤 발화 시작 |
| `preparation.restore=false` | S2-04는 S2-02의 실제 상태/cache/이력을 이어받음. 준비된 정답 snapshot으로 복원하지 않음 |
| `preparation.pending_interpretation` | R옛의 실제 후보 job을 만들고 원래 반환을 보류. job이 없으면 가짜 반환 없음 |
| `followups` | 실제 관련 질문 또는 Text 표시 receipt가 생긴 뒤에만 지정 답변/재연결 발화 공급 |
| `QUESTION_REGISTER` | 질문 원천의 availability와 owner 등록. `presentation_requested=false`이며 실제 질문 제시나 receipt가 아님 |
| `consent_source_release` | 실제 질문/범위의 정보 동의가 확인된 뒤 owner의 민수 연락처 원문을 제공. SEND 승인은 계속 없음 |
| `stub_profile` | 동일 외부 시간/capability의 모사 계약. 실제 받은 payload와 source로 생산하며 case ID/정답으로 보정하지 않음 |
| `reactions`, `cycle_start` | harness-only 관측 ID와 시작/종료의 typed 대응. 모델 context에 제공하지 않음 |

준비된 과거 원문, 이미 전달한 발화와 외부에서 이미 완료된 Task의 사실은 before-state에 제공한다.
현재 요청의 새로운 의미, 현재 요약, 제안서와 보고서 결과는 후보/실제 payload를 소비한 stub이
생산해야 한다. SOURCE_CHANGE가 나중에 생기는 source를 초기 owner repository에 미리 넣지 않는다.

음성 입력과 출력은 현재 PoC의 공통 모사 profile을 따라 별도 evidence로 표시한다. 지연 값/발화 길이는
원문의 고정 값과 모사 가정을 구별한다. 원문에 길이가 없는 추가 비힌트 답변은 3초의 공통 가정으로
기록했다. 외부 Agent 질문에는 이 답변을 자동 제공하지 않는다. 현재 요약 event의 중복 간격 +1ms는
구현 전 검토 문서의 공통 profile 제안이며 실제 source의 보장값으로 표시하지 않는다.

`build_activity_baseline()`은 원문 2,000turn, 완료 Task 200개, source 슬롯 30개를 생성한다.
20MiB 원문 payload의 마지막 공백 padding은 background source에만 둔다. 학습된 관계나 정답을
padding으로 넣지 않으며 DB/JSON/index/runtime 메모리는 별도 비용이다.
활동 원장의 B01~B04 갱신 네 건은 `AFTER_S2_04` 기준이며 최종 maintenance에서 B04의 read,
모델 제공과 Agent 제공 권한을 모두 철회한다. `manifest.json`의 `candidate_metadata_contract`는
이 owner 원문 총량과 최초 metadata 범위, 실제 bounded query의 권한 경계를 별도로 기록한다.
semantic read는 read와 model 권한을 요구한다. Agent read는 read와 agent 권한을 요구하며,
후보가 생성한 조회 spec 안의 역할 문자열로 이 구분을 바꿀 수 없다.

새 `source_id`의 원천 게시에는 아직 비어 있는 C 슬롯 한 개를 소비한다. 같은 source의 새 revision은
과거 원문을 보존하며 논리 source 수를 늘리지 않는다. 빈 슬롯이 없으면 명시적으로 거절하고,
동의 뒤 동일 source/version의 빈 원문을 공개하는 작업은 슬롯을 더 소비하지 않는다.

`build_observation_buffers()`는 별도 공통 데이터로 RGBA bytearray 15개를 실제 할당하고 전체 내용을
채운다. 각 frame은 2880×1800×4=20,736,000byte이며 총 311,040,000byte다. frame index에 따라
`(index % 256, 17*index % 256, 31*index % 256, 255)` 픽셀을 반복한다.
`observation_buffer_records()`는 서로 다른 실제 buffer, 크기와 전체 내용 SHA-256을 검증하고
generator와 hash 원장 항목을 반환한다. 원문 20MiB와 별개의 raw buffer이며 활동 동안 유지한다.
이는 공통 메모리 workload다. 화면 의미는 제공된 원천 text를 사용하며 픽셀 인식 정확성을
측정한 것으로 해석하지 않는다. 고정 원문/원천 buffer 이외의 목표 메모리 수치에 맞춘 할당은 없다.
