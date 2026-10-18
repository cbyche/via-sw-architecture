# DP05 실제 VIA 원문 재사용 연결 교정 — 독립 읽기 검토

## 확인한 기본 결함

설계 05-agent-handoff.md §2.2는 A에도 이미 얻은 유효 근거를 숨기지 않도록 하고, §4.1은 즉석 구성이 이미 확보한 근거를 버리는 뜻이 아니라고 명시한다. 기존 dp05.rs의 common_evidence는 acquire 사건에서만 확장된다. dp05_handoff.rs는 실제 selected_reads(..., VIA_COMPOSER)로 읽은 passages를 현재 capsule에 넣지만 이를 다음 위임의 actual_common_evidence에 연결하지 않는다. 따라서 실제 읽기 재사용 계약의 구현 누락이며, 새로운 semantic cache 전략을 도입해야만 해결되는 문제가 아니다.

## 최소 계획 검토

2026-10-18 evidence 담당이 전달한 계획: 별도 scratch의 build가 available:&mut Vec를 받아 실제 selected_reads 성공 직후 common helper에 원문을 합친다. source revision/revoke/allowed range 검사는 기존 valid_span을 유지하며 정확 source/revision/start/end 중복만 제거한다. acquire도 같은 helper를 사용한다. 예상 운영 변경은 dp05.rs, dp05_handoff.rs, dp05_sources.rs 세 파일이다. live/frozen source는 바꾸지 않는다.

이 계획의 책임 경계에 동의한다. 특히 compose 또는 receiver가 실패해도 이미 실제 읽은 원문은 공통 근거에 남아야 한다. capsule 성공 후 passages만 사후 추출하면 compose 실패 후 다음 위임에서는 실제 확보한 근거를 다시 잃을 수 있다는 검토 지적을 반영했다.

필수 확인 경계:

- 실제 host read가 반환한 canonical 원문만 저장한다. 모델 summary, selected span이 직접 만든 text, record supporting passages 또는 receiver만 읽은 자료를 VIA 실제 획득으로 승격하지 않는다.
- 다음 목적의 실제 model payload에 같은 현재 source/revision/range/text가 전달되는지 확인한다. 모델이 이를 재사용해 read를 생략하는지는 별도 실제 관측이며 host가 그 선택을 강제하지 않는다.
- compose 실패/receiver 실패 뒤의 실제 원문 보존, source 판본 변경·철회·범위 축소 뒤 무효 원문 불포함, 정확 범위 중복 억제를 확인한다.
- 정답·oracles·기존 source bytes·기존120행은 불변이다. 교정은 ABC 모두 같은 새 freeze/binary로36회 실행하며 수치 우위를 보장하지 않는다.

현재 상태: 계획과 실제 proposed.patch의 운영3파일 읽기 완료, 필수수정 없음. read직후 merge가 compose 오류 전 실행되고, 다음목적 input은 갱신 common_evidence를 받으며, 기존권한·판본무효화와 정확범위dedup이 유지된다. 신규7 pure guard의 코드도 해당 경계와 일치한다. 독립읽은 source3파일·patch·SHA는 dp05-common-retention-reviewed-source/에 보존했다. 이 독립 읽기 자체는 compile/수용 PASS를 주장하지 않는다. C01/C03 실제control 실행은 별도근거로 기록한다.

## 결과 세대와 감사 범위

예정 raw 보존은218 episode = calibration26 + 원래BASIC120 + 보완27 + DP05 교정36 + 교정baseline 줄번호진단9다. 실제 발표용 effective120은 원래DP01/02/04의84 + BASIC_CORRECTED DP05의36이며 고정 QA01 261/QA03 243 분모를 다시 확인한다. 원래DP05의36행은 삭제하거나 교정값으로 덮어쓰지 않는다. 줄번호 진단은 교정 source/binary에서 분리한 DIAGNOSTIC phase이고 BASIC 성공으로 합산하지 않는다.

모델/API·후보실행·컴파일·live source 변경0. 위 계획과 과거 원시/운영 코드의 읽기 및 파생문서만 작성했다.

## 최종 prefix 공정성 경계 읽기 (추가)

B가 이미 소유한 valid work-record의 host-attached raw supporting_passages도 prefix에 제공한다. prefix_original_evidence는 common + 현재 valid block의 원문만 source_id/revision/start_line/end_line/text/unit으로 재구성한다. 임의 모델 필드·explanation/conditions를 넘기지 않고 정확 범위를 중복제거하며, 영구 common 원장은 변경하지 않는다. prefix 호출 뒤 view/payload를 drop하고 논리 buffer release를 남긴다. 실제 운영3파일 hash를 source-differences.json과 독립 대조했고 필수수정 없음. snapshot은 dp05-common-retention-final-prefix-source/다. 담당자의57개 Rust검사 완료는 담당자 증거이며, 본 독립 읽기가 실제 수용실행을 대체하지 않는다. 실제wire/C01/C03는27회 모델 trial 뒤 승인된 창까지 보류한다.
