# DP01 BASIC108문안 실제 내용 전수검토

저자 검토. 고정 원문·질문 및 pre-authored bounded content조건과 실제 GOAL_COMPLETE 본문을 대조했다. 미표시1건도108분모에 유지했다. 독립 의미심사를 완료했다는 뜻은 아니다. QA02 지칭 대상 정확도 및 host완료와 별도이다. 동일본문hash는 동일goal조건일 때만 연결했다.

|안|고정분모|literal|표기동등|실제의미 PASS|FAIL|PARTIAL|미표시|
|---|---:|---:|---:|---:|---:|---:|---:|
|A|36|30|30|30|5|1|0|
|B|36|29|29|30|3|2|1|
|C|36|27|32|34|0|2|0|

## PASS 외 판정 근거 (실제 원문 인용은 JSON source_evidence 참조)

### basic-dp01-e1-B-r1-f16625 / e1-q2 — NOT_DISPLAYED

- literal=False, equivalent=False
- 실제표시: [NOT_DISPLAYED]
- 판정: 원문 행2/3의 내용을 행3/4로 잘못 인용한 제안이 repair 후에도 current supplied span quote 검사에 실패했다. 미표시 draft의 정답 같은 본문을 성공으로 세지 않는다.
- [raw](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-dp01-e1-B-r1-f16625/trace.json)

### basic-dp01-e2-A-r1-ce936c / e2-q3 — FAIL

- literal=True, equivalent=True
- 실제표시: 개발팀 일정에서 북부 고객이 제안한 시간인 10월 30일 오후 2시는 내부 보안 점검으로 인해 충돌이 발생합니다. 해당 날짜의 오전 10시는 비어 있습니다.
- 판정: 충돌은 맞으나, 덧붙인 “해당 날짜의 오전10시는 비어”는 앞서 지칭한10월30일을 뜻한다. 실제 원문 team-calendar-v1 line2는10월31일 오전10시이다. 고정 marker 검사는 충돌만 검사하므로 이 추가 허위 주장까지 검증하지 못한다.
- [raw](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-dp01-e2-A-r1-ce936c/trace.json)

### basic-dp01-e3-A-r1-1f39fd / e3-q2 — FAIL

- literal=False, equivalent=False
- 실제표시: 평가 기준에 따르면 부가세 포함 가격과 유지보수 기간을 함께 비교하며, 가격이 동일할 경우 유지보수 기간이 더 긴 제품을 우선합니다. 최신 정정 견적(Aster A v2)에 따르면 단가는 부가세 포함 1,180,000원이며, 유지보수 기간은 36개월입니다. 설치비 80,000원은 별도이며, 구매 여부는 아직 결정되지 않았습니다.
- 판정: 현재 견적v2의 가격과 유지보수는 실제 본문에 있지만, 납기 11월1일이 실제 본문에서 빠졌다. 원문 vendor-quote-v2 line2의 필수 정정 내용 누락이다.
- [raw](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-dp01-e3-A-r1-1f39fd/trace.json)

### basic-dp01-e3-A-r2-3d7648 / e3-q2 — FAIL

- literal=False, equivalent=False
- 실제표시: 평가 기준에 따르면 부가세 포함 가격과 유지보수 기간을 함께 비교하며, 가격이 동일할 경우 유지보수 기간이 더 긴 제품을 우선시합니다. 최신 정정 견적(Aster A v2)에 따르면 단가는 부가세 포함 1,180,000원이며, 유지보수 기간은 36개월입니다. 설치비 80,000원은 별도이며, 구매 여부는 아직 결정되지 않았습니다.
- 판정: 현재 가격과 유지보수는 실제 표시했지만 정정 납기11월1일이 본문에서 빠졌다. vendor-quote-v2 line2의 필수정정내용 누락.
- [raw](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-dp01-e3-A-r2-3d7648/trace.json)

### basic-dp01-e3-A-r3-ff0b17 / e3-q2 — FAIL

- literal=False, equivalent=False
- 실제표시: 평가 기준에 따르면 부가세 포함 가격과 유지보수 기간을 함께 비교하며, 가격이 동일할 경우 유지보수 기간이 더 긴 제품을 우선시합니다. 최신 정정 견적(Aster A v2)에 따르면 단가는 부가세 포함 1,180,000원이며, 유지보수 기간은 36개월입니다. 설치비 80,000원은 별도이며, 구매 여부는 아직 결정되지 않았습니다.
- 판정: 정정가격과 유지보수는 있지만 실제 표시본문에 정정납기11월1일이 빠졌다(vendor-quote-v2 line2).
- [raw](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-dp01-e3-A-r3-ff0b17/trace.json)

### basic-dp01-e3-B-r1-1599b2 / e3-q2 — FAIL

- literal=False, equivalent=False
- 실제표시: 평가 기준에 따르면 부가세 포함 가격과 유지보수 기간을 함께 비교하며, 가격이 동일할 경우 유지보수 기간이 더 긴 제품을 우선시합니다. 최신 정정 견적(v2)에 따르면 Aster A의 가격은 대당 1,180,000원(부가세 포함)이며, 유지보수 기간은 36개월입니다. 설치비 80,000원은 별도이며, 구매 여부는 아직 결정되지 않았습니다.
- 판정: 현재 견적v2의 가격과 유지보수는 실제 본문에 있지만, 납기 11월1일이 실제 본문에서 빠졌다. 원문 vendor-quote-v2 line2의 필수 정정 내용 누락이다.
- [raw](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-dp01-e3-B-r1-1599b2/trace.json)

### basic-dp01-e3-B-r2-72eabe / e3-q2 — FAIL

- literal=False, equivalent=False
- 실제표시: 평가 기준에 따르면 부가세 포함 가격과 유지보수 기간을 함께 비교하며, 가격이 동일할 경우 유지보수 기간이 더 긴 제품을 우선합니다. 최신 정정 견적(Aster A v2)에 따르면 단가는 부가세 포함 1,180,000원이며, 유지보수 기간은 36개월입니다. 설치비 80,000원은 별도이며, 구매 여부는 아직 결정되지 않았습니다.
- 판정: 현재 가격과 유지보수는 실제 표시했지만 정정 납기11월1일이 본문에서 빠졌다. vendor-quote-v2 line2의 필수정정내용 누락.
- [raw](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-dp01-e3-B-r2-72eabe/trace.json)

### basic-dp01-e3-B-r3-d8e938 / e3-q2 — FAIL

- literal=False, equivalent=False
- 실제표시: 평가 기준에 따르면 부가세 포함 가격과 유지보수 기간을 함께 비교하며, 가격이 동일할 경우 유지보수 기간이 더 긴 제품을 우선시합니다. 최신 정정 견적(Aster A v2)에 따르면 단가는 부가세 포함 1,180,000원이며, 유지보수 기간은 36개월입니다. 설치비 80,000원은 별도이며, 구매 여부는 아직 결정되지 않았습니다.
- 판정: 정정가격과 유지보수는 있지만 실제 표시본문에 정정납기11월1일이 빠졌다(vendor-quote-v2 line2).
- [raw](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-dp01-e3-B-r3-d8e938/trace.json)

### basic-dp01-e4-A-r1-3fe011 / e4-q2 — PARTIAL

- literal=False, equivalent=False
- 실제표시: 9월 공개된 전력 실측 결과는 유휴 상태의 전력만 포함하며, 고부하 작업 전력이나 장기 전력비는 측정하지 않았습니다. 반면, 당시 받아둔 견적은 세금과 배송비를 포함한 8대 기준의 예산 검토용 자료로, 현재 유효한 구매 계약이 아닙니다.
- 판정: 실측과 견적 조건 일부는 맞지만 실제질문과 고정조건의 동일모니터/절전설정 중 하나 이상이 빠졌다. specific hardware settings라는 C의 표현도 구체적 동일모니터+절전 조건을 전달하지 않는다. literal진단과 별도로 필수조건요약의 불완전으로 판정.
- [raw](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-dp01-e4-A-r1-3fe011/trace.json)

### basic-dp01-e4-A-r1-3fe011 / e4-q3 — FAIL

- literal=True, equivalent=True
- 실제표시: The unconfirmed matters are that high-load power and long-term power ratios were not measured for the research results, and the price quote is not a current valid purchase contract.
- 판정: 원문 old-research line3의 장기 전력비(long-term electricity cost)를 실제본문 long-term power ratios로 바꾸었다. 미확인 핵심 비용 항목을 다른 물리량으로 표현해 정확한 요약으로 보기 어렵다.
- [raw](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-dp01-e4-A-r1-3fe011/trace.json)

### basic-dp01-e4-B-r1-184e18 / e4-q2 — PARTIAL

- literal=False, equivalent=False
- 실제표시: 9월 공개된 전력 실측 결과는 유휴 전력만 포함하며, 견적은 세금과 배송비가 포함된 8대 기준의 예산 검토용 자료입니다. 두 결과 모두 고부하 전력이나 현재 구매 계약 상태를 반영하지 않습니다.
- 판정: 실측과 견적 조건 일부는 맞지만 실제질문과 고정조건의 동일모니터/절전설정 중 하나 이상이 빠졌다. specific hardware settings라는 C의 표현도 구체적 동일모니터+절전 조건을 전달하지 않는다. literal진단과 별도로 필수조건요약의 불완전으로 판정.
- [raw](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-dp01-e4-B-r1-184e18/trace.json)

### basic-dp01-e4-B-r1-184e18 / e4-q3 — PARTIAL

- literal=False, equivalent=False
- 실제표시: 두 원본 모두 고부하 전력 데이터와 현재 구매 계약 상태를 반영하지 않았습니다.
- 판정: 고부하전력과 현재계약의 한계를 언급했지만 미측정 장기전력비를 빠뜨렸다. 원문 old-research line3의 미확인사항 누락.
- [raw](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-dp01-e4-B-r1-184e18/trace.json)

### basic-dp01-e4-C-r1-67c6c1 / e4-q2 — PARTIAL

- literal=False, equivalent=False
- 실제표시: The September power research measured idle power under specific hardware settings but did not measure high-load power or long-term costs. The price quotes were for 8 units including tax and shipping, intended for budget review rather than as a current purchase contract.
- 판정: 실측과 견적 조건 일부는 맞지만 실제질문과 고정조건의 동일모니터/절전설정 중 하나 이상이 빠졌다. specific hardware settings라는 C의 표현도 구체적 동일모니터+절전 조건을 전달하지 않는다. literal진단과 별도로 필수조건요약의 불완전으로 판정.
- [raw](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-dp01-e4-C-r1-67c6c1/trace.json)

### basic-dp01-e4-C-r3-8e8dfc / e4-q2 — PARTIAL

- literal=False, equivalent=False
- 실제표시: The September power research measured idle power under identical monitor and power-saving settings but did not measure high-load power or long-term electricity costs. The received price quotes were materials used for the September budget review and are not current purchase contracts.
- 판정: 실측의 동일monitor/절전setting은 전달했으나 견적의8대·세금·배송 포함 조건을 모두 빠뜨렸다(old-price line2). 조건차이 재설명 요구에 대한 불완전한 답이다.
- [raw](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-dp01-e4-C-r3-8e8dfc/trace.json)

## Progress별도검토

108개 실제 ADOPTION_COMMITTED 문구는 질문에 대응하는 구체적 처리의도였다. 실제처리 완료를 사칭하지 않았다. source 제목+model intent를 표출하므로 snake_case/영어가 섞이는 proxy 문구의 제품 UX 품질까지 검증한 것은 아니다. C E4q3r1 및E4q2r2는 old-explanation을 원지칭대상에 추가하여 최초target-set이 틀렸다. actualprogress시간을 버리지 않았고 내용본문 pass 여부와 분리했다.

[전수 기계판독 기록](dp01-basic-content-review.json)
