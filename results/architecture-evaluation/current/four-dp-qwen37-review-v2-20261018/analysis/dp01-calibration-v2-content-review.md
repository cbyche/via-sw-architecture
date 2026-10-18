# DP01 calibration-v2 본문 대조와 표현 동치

공식 BASIC 결과가 아닌 E3의 ABC 각 1회, 총 6개 실제 상세 출력이다. 후보·입력·정답 조건과 기존 literal 진단은 그대로 보존했다. 별도 evaluator v2는 모든 안에 같은 월 단위/영어 날짜 동치 규칙을 적용한다. 날짜 실제 누락을 성공으로 바꾸지 않는다. API 추가 호출은 없다.

| 안 / 목표 | 기존 literal | 일반 단위·날짜 동치 | 실제 본문 검토 | 이유 |
| --- | --- | --- | --- | --- |
|A / e3-q1|True|True|PASS_BOUNDED_FACTS_AND_CONDITIONS|현재 원본과 사실·조건 일치. C의 영어 단수 복합 단위도 동치.|
|A / e3-q2|False|False|PARTIAL_REQUIRED_DELIVERY_DATE_OMITTED|11월1일 본문 누락. 실제 내용 누락.|
|B / e3-q1|True|True|PASS_BOUNDED_FACTS_AND_CONDITIONS|현재 원본과 사실·조건 일치. C의 영어 단수 복합 단위도 동치.|
|B / e3-q2|False|False|PARTIAL_REQUIRED_DELIVERY_DATE_OMITTED|11월1일 본문 누락. 실제 내용 누락.|
|C / e3-q1|False|True|PASS_BOUNDED_FACTS_AND_CONDITIONS|현재 원본과 사실·조건 일치. C의 영어 단수 복합 단위도 동치.|
|C / e3-q2|False|True|PASS_BOUNDED_FACTS_AND_CONDITIONS|현재 원본과 사실·조건 일치. C의 영어 단수 복합 단위도 동치.|

기존 literal 2/6, 평행 동치 진단 4/6, 작성자 본문 검토 4/6이다. 이는 기능 점검의 한 사례이며 구조 우열·최종 선택 증거로 일반화하지 않는다. C의 November1st는 현행 content_checks의 november1 그룹에도 이미 매칭된다. 현재 C literal 실패는 36-month와 36months의 단수 복합형 차이이며, 과거 Korean-only semantic_checks와 혼동하지 않는다.

## 실제 출력 근거

### A / e3-q1

[원시 trace](../calibration-v2/raw/runs/calibration_v2-dp01-e3-A-r1-34a104/trace.json)

실제로 설명한 평가 기준은 부가세 포함 가격과 유지보수 기간을 함께 비교하며, 가격이 같을 경우 유지보수 기간이 더 긴 제품을 우선시하는 것입니다. 또한 구매 여부는 아직 정하지 않았으며 견적의 최신 판본 확인이 필요하다고 명시했습니다. 받아둔 Aster A 견적(v1)에 따르면 가격은 1대당 1,250,000원(부가세 포함)이며 유지보수 기간은 36개월입니다.

Source facts and conditional criteria match; English 36-month maintenance is equivalent to 36개월, and November1st is 11월1일 when present. No unshown installation criterion is claimed as previously delivered.

### A / e3-q2

[원시 trace](../calibration-v2/raw/runs/calibration_v2-dp01-e3-A-r1-34a104/trace.json)

평가 기준에 따르면 부가세 포함 가격과 유지보수 기간을 함께 비교하며, 가격이 동일할 경우 유지보수 기간이 더 긴 제품을 우선합니다. 최신 정정 견적(Aster A v2)에 따르면 단가는 부가세 포함 1,180,000원이며, 유지보수 기간은 36개월입니다. 설치비 80,000원은 별도이며, 구매 여부는 아직 결정되지 않았습니다.

The actual final text states current price and maintenance but omits the current delivery date; source v2 line2 states 11월1일. This is not a format issue.

### B / e3-q1

[원시 trace](../calibration-v2/raw/runs/calibration_v2-dp01-e3-B-r1-53f9f9/trace.json)

실제로 설명한 평가 기준은 부가세 포함 가격과 유지보수 기간을 함께 비교하며, 가격이 동일할 경우 유지보수 기간이 더 긴 제품을 우선시하는 것입니다. 또한 구매 여부는 아직 정하지 않았으며 견적의 최신 판본 확인이 필요하다고 명시했습니다.

제공된 Aster A 견적(v1)에 따르면 단가는 부가세 포함 1,250,000원이며 유지보수 기간은 36개월입니다. 납기는 10월 29일이고 설치비는 80,000원이 별도입니다. 수량은 12대이며 계약이나 구매는 아직 체결되지 않았습니다.

Source facts and conditional criteria match; English 36-month maintenance is equivalent to 36개월, and November1st is 11월1일 when present. No unshown installation criterion is claimed as previously delivered.

### B / e3-q2

[원시 trace](../calibration-v2/raw/runs/calibration_v2-dp01-e3-B-r1-53f9f9/trace.json)

평가 기준에 따르면 부가세 포함 가격과 유지보수 기간을 함께 비교하며, 가격이 동일할 경우 유지보수 기간이 더 긴 제품을 우선시합니다. 최신 정정 견적(Aster A v2)에 따르면 단가는 부가세 포함 1,180,000원이며, 유지보수 기간은 36개월입니다. 설치비 80,000원은 별도이며, 구매 여부는 아직 결정되지 않았습니다.

The actual final text states current price and maintenance but omits the current delivery date; source v2 line2 states 11월1일. This is not a format issue.

### C / e3-q1

[원시 trace](../calibration-v2/raw/runs/calibration_v2-dp01-e3-C-r1-71e558/trace.json)

The actually delivered evaluation criteria state that comparison will consider VAT-inclusive price and maintenance period together, prioritizing the product with the longer maintenance period if prices are equal. It was also noted that the purchase decision is not yet made and requires checking the latest quote version. The received Aster A quote (v1) lists a unit price of 1,250,000 KRW (VAT included) for 12 units, with a 36-month maintenance period and a delivery date of October 29. The contract has not yet been signed.

Source facts and conditional criteria match; English 36-month maintenance is equivalent to 36개월, and November1st is 11월1일 when present. No unshown installation criterion is claimed as previously delivered.

### C / e3-q2

[원시 trace](../calibration-v2/raw/runs/calibration_v2-dp01-e3-C-r1-71e558/trace.json)

The evaluation criteria prioritize VAT-inclusive prices and maintenance periods, using the latter as a tiebreaker if prices are equal, while noting that no purchase decision has been made yet. The latest revised quote (v2) for Aster A sets the unit price at 1,180,000 KRW (VAT included), maintains the quantity of 12 units and 36-month maintenance period, keeps the separate installation fee of 80,000 KRW, and corrects the delivery date to November 1st. No contract has been signed.

Source facts and conditional criteria match; English 36-month maintenance is equivalent to 36개월, and November1st is 11월1일 when present. No unshown installation criterion is claimed as previously delivered.
