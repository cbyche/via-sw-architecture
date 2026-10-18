# 최종 union 교정 DP05 독립 지정 본문 검토

완료27/27. 독립 라벨 {'FAIL': 10, 'PASS': 14, 'NOT_DISPLAYED': 3}. Full-PASS14/27; 미표시3/27 포함. 전체54 또는 일반 모델 정확도 지표가 아니다.

|안|PASS|FAIL|NOT_DISPLAYED|
|---|---:|---:|---:|
|A|1|7|1|
|B|6|1|2|
|C|7|2|0|

Legacy blind_reviewed_outputs means current-run author verdict unexposed before own saved judgment, except conservatively excluded prior-exposed identical A r1. Reviewer sees alternatives/source and prior self-reviewed/reconciled repeated outputs; no experimental reviewer blinding claimed.

현재 실행 author 판정은27건 모두 자체판정 저장 전 미노출. 다만 E2 A r1은 과거 노출된 동일본문의 기억으로 별도 비블라인드1을 보수적으로 표시한다. 반복 동일본문·선행 자기판정 친숙도도 공개한다.

Fixed rubric lists PASS/FAIL/PARTIAL/UNRESOLVED/NOT_DISPLAYED but no total-label aggregation priority. Author/independent labels may differ while substantive findings agree; preserve per-check facts and explicit discrepancies.

A E4 세 본문은 필수 예산 전체누락: author PARTIAL / independent FAIL. Full-PASS 여부와 실제 누락 사실은 동일하다. 라벨을 골라 성공률을 변경하지 않는다.

27 unique selection/실제 first GOAL_COMPLETE 또는 부재/원시·sourceSHA 기계대조 모두 PASS. 이는 의미판정 자동검증이 아니다.

- `basic_corrected-dp05-e2-A-r1-462049` / h2-deck: **FAIL** — 현재 공개 가격은 서로 다른데, 표시 본문은 유지보수가 36개월로 길다는 이유만으로 Aster를 우선시한다고 명시한다. 동률 전제가 사라졌고 사실 수치·4문장 형식의 통과와 별도로 조건을 위반한다.
- `basic_corrected-dp05-e2-B-r1-8655ef` / h2-deck: **PASS** — 실제 네 문장에 가격·유지보수·원격정책 미검토·고부하 미시험을 담았다. 가격이 같았다면 유지보수 기간이 결정 요인이라고 명시적으로 가정하며 실제 가격이 다른 사실을 보존한다.
- `basic_corrected-dp05-e2-C-r1-084da4` / h2-deck: **PASS** — 실제 네 문장에 현재 가격·유지보수·미확인 사항을 담고 추천 미확정/구매 승인 별도를 보존한다. 유지보수가 긴 현재 제품을 무조건 선호·추천하지 않아 동률 조건을 위반하지 않는다. 요청은 조건을 지킬 것을 요구하며 모든 기준 문구를 반드시 반복하도록 요구하지 않는다.
- `basic_corrected-dp05-e3-A-r1-4ba071` / h3-mail: **FAIL** — 실제 가격 차이가 미미하다고 인정하면서 유지보수 기간 우선을 사전 합의된 기준으로 바꿔 Aster를 선호한다고 정리한다. 동률 전제가 삭제되었고, 뒤의 추천 미확정·구매 승인 별도 단서가 이 현재 선호 근거를 해소하지 않는다.
- `basic_corrected-dp05-e3-B-r1-211c9c` / h3-mail: **PASS** — 현재 가격·납기·유지보수 값을 맞게 전달하고 가격이 동일할 경우에만 긴 유지보수를 우선한다고 명시한다. 현재 B가 저렴한 사실과 미검토 사항을 보존하며 외부 실행 완료를 주장하지 않는다.
- `basic_corrected-dp05-e3-C-r1-1b13b7` / h3-mail: **FAIL** — 가격은 B가 더 낮으나 계약 조건상 긴 유지보수 제품을 우선한다며 Aster를 권장한다. 현재 가격 차이에도 동률 전제를 없앤 추천이므로 FAIL이며 연구팀 미확정 단서는 이 현재 근거를 해소하지 않는다.
- `basic_corrected-dp05-e4-A-r1-0e3a88` / h4-corrected: **FAIL** — 실제 표시 본문은 정정 단가1,160,000원과 납기11월4일을 맞게 전달하지만, 명시 요청된 예산 조건14,000,000원을 전혀 포함하지 않는다. receiver metadata나 채택 상태의 예산값을 실제 메일 본문 충족으로 대신 세지 않는다.
  - author PARTIAL / independent FAIL, 실질 누락 판단 일치.
- `basic_corrected-dp05-e4-B-r1-3747bb` / h4-corrected: **NOT_DISPLAYED** — 최종 h4-corrected GOAL_COMPLETE/상세 본문 표시가 없다. GOAL_FAILED RECEIVER_BOUNDED_COMPLETION_FAILED 및 fact_claim_or_actual_current_evidence_missing이 기록되며 미표시 초안의 예산/가격 문구를 전달 성공으로 세지 않는다.
- `basic_corrected-dp05-e4-C-r1-12f21f` / h4-corrected: **PASS** — 정정 단가1,160,000원과 납기11월4일, 수량12와 예산상한14,000,000원 내라는 조건을 실제 초안 본문에 포함한다. 설치 일정 미확정 및 제외 범위를 보존하며 권고/발송/계약 완료를 새로 주장하지 않는다.
- `basic_corrected-dp05-e2-A-r2-03071d` / h2-deck: **FAIL** — 서로 다른 가격을 제시한 뒤 유지보수가 더 길다는 이유로 Aster가 우선된다고 명시해 가격 동률 전제를 삭제한다. 끝의 연구팀 추천 미확정 단서는 이 현재 우선순위 주장을 해소하지 않는다.
- `basic_corrected-dp05-e2-B-r2-b8bd03` / h2-deck: **PASS** — 실제 네 문장에 현재 가격·유지보수·보안/미확인 사항을 전달하고 if prices were equal 이라는 동률 가정을 명시한다. 현재 가격 차이를 보존하며 미검토를 완료로 바꾸지 않는다.
- `basic_corrected-dp05-e2-C-r2-a03698` / h2-deck: **PASS** — 실제 네 문장에 현재 가격·유지보수·원격정책 미검토 및 추천 미확정/구매 승인 별도를 전달한다. 현재 긴 유지보수를 이유로 무조건 추천하지 않아 동률 조건을 위반하지 않는다.
- `basic_corrected-dp05-e3-A-r2-a33c38` / h3-mail: **NOT_DISPLAYED** — 실제 h3-mail 상세 GOAL_COMPLETE가 없고 USER_INTERPRETATION_CONTRACT_FAILED가 기록되어 있다. 미표시 내용을 성공 또는 의미 오답의 표시로 추정하지 않는다.
- `basic_corrected-dp05-e3-B-r2-6cc27c` / h3-mail: **PASS** — 현재 가격 차이가 있어 동가시 유지보수 규칙을 이번 최종 결정 요인으로 직접 적용하지 않는다고 명시한다. 가격·유지보수·납기와 제외/불확실성을 현재 원문대로 전달한다.
- `basic_corrected-dp05-e3-C-r2-788e21` / h3-mail: **FAIL** — B가 더 저렴한데도 계약조건상 긴 유지보수 제품을 우선하는 규정으로 Aster가 우위라고 단정한다. 가격 동률 전제가 삭제되었고 추천 미확정 단서는 이 조건 위반을 해소하지 않는다.
- `basic_corrected-dp05-e4-B-r2-8c2cfd` / h4-corrected: **NOT_DISPLAYED** — h4-corrected 최종 상세 GOAL_COMPLETE가 없고 RECEIVER_BOUNDED_COMPLETION_FAILED 및 개별 fact quote 검사 실패2건이 기록된다. 미표시 본문을 전달 성공으로 세지 않는다.
- `basic_corrected-dp05-e4-C-r2-b0009c` / h4-corrected: **PASS** — 실제 메일 초안에 정정단가1,160,000원·12대 합계13,920,000원·예산상한14,000,000원 내·납기11월4일이 모두 맞다. 미확정 설치/계약과 제외 범위를 유지하며 최종 추천·외부 실행 완료를 주장하지 않는다.
- `basic_corrected-dp05-e4-A-r2-391937` / h4-corrected: **FAIL** — 현재 단가1,160,000원·11월4일 예상 납기 및 미확정/제외 범위는 맞지만, 실제 메일 본문에 명시 요청 예산14,000,000원 조건이 전혀 없다. metadata 예산 보존을 실제 본문 충족으로 대신하지 않는다.
  - author PARTIAL / independent FAIL, 실질 누락 판단 일치.
- `basic_corrected-dp05-e2-C-r3-714612` / h2-deck: **PASS** — 실제 네 문장에 현재 가격·유지보수·원격정책 미검토 및 추천 미확정/구매 승인 별도를 전달한다. 유지보수가 긴 제품을 현재 무조건 선호하지 않아 동률 조건을 위반하지 않는다.
- `basic_corrected-dp05-e2-A-r3-6c97d3` / h2-deck: **PASS** — 현재 가격·유지보수와 암호화 확인/원격정책·고부하 미검토를 실제 네 문장으로 전달한다. 추천 미확정과 계약 미완료를 보존하며 현재 긴 유지보수 제품에 무조건 우선순위를 부여하지 않는다.
- `basic_corrected-dp05-e2-B-r3-590d54` / h2-deck: **FAIL** — 서로 다른 가격을 제시한 뒤 Aster의 더 긴 유지보수가 preferred option을 만든다고 현재 선호를 주장한다. longer maintenance tie-break라는 이름만 남기고 가격 동률 전제 없이 현재 우선순위를 부여해 조건을 위반한다.
- `basic_corrected-dp05-e3-C-r3-791797` / h3-mail: **PASS** — 가격·납기·유지보수 및 미확인 범위를 현재 원문과 일치하게 전달한다. 동점시 유지보수 규칙을 참고할 수 있다는 일반 기준으로 명시하며 현재 Aster를 우선/추천한다고 결론내리지 않고 추천 미확정·승인 별도를 유지한다.
- `basic_corrected-dp05-e3-A-r3-0028b1` / h3-mail: **FAIL** — 실제 본문이 가격 차이가 있을 경우 긴 유지보수를 우선하도록 앞서 정했다고 명시하여, 원래 가격 동률시 조건을 반대로 바꾸고 Aster를 추천한다. 가격·납기·유지보수·미확인 사실은 맞아도 이 명확한 조건 반전은 실패다.
- `basic_corrected-dp05-e3-B-r3-15540f` / h3-mail: **PASS** — 현재 가격·납기·유지보수 수치가 맞다. 가격이 동일할 경우의 원칙을 명시하고 현재 가격 차이 및 Aster 유지보수 기간이 더 길다는 사실만을 병기하며 무조건 현재 추천으로 바꾸지 않는다. 미검토 성능·설치 별도·계약 미완료도 유지한다.
- `basic_corrected-dp05-e4-C-r3-48f5ab` / h4-corrected: **PASS** — 실제 구매 검토 초안에 정정단가1,160,000원·12대합계13,920,000원·예산14,000,000원 내·11월4일 예상 납기를 맞게 포함한다. 설치 일정 미확정과 미검증 운영비/비공개 연락처 제외를 유지하고 외부 실행을 주장하지 않는다.
- `basic_corrected-dp05-e4-A-r3-2ae48d` / h4-corrected: **FAIL** — 실제 두 문장의 현재 단가1,160,000원·11월4일 예상 납기는 맞지만, 요청된 예산14,000,000원 조건은 본문에 전혀 없다. 표시 metadata 또는 상태값으로 이 필수 본문 누락을 대신하지 않는다.
  - author PARTIAL / independent FAIL, 실질 누락 판단 일치.
- `basic_corrected-dp05-e4-B-r3-934bac` / h4-corrected: **PASS** — 실제 초안에 정정단가1,160,000원·12대합계13,920,000원·예산14,000,000원 내·납기11월4일이 맞게 포함된다. 설치 미확정과 비공개 연락처/설치편의성/미검증 운영비 제외를 보존하며 실제 발송 완료를 주장하지 않는다.

지정27 밖 추가 경계2는 dp05-final-additional-boundary-content-review.json에 별도 보존하며 둘 다 UNRESOLVED. 공식 QA02/N-H/literal·동치 진단을 대체하지 않는다. 기존 중단 phase5판정은 별도 역사 파일에 보존한다. 모델 호출/후보 실행0.
