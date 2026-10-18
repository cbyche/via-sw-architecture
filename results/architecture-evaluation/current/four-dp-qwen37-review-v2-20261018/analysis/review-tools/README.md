# 파생 검토 도구

이 파일들은 모델/후보/평가기/원시를 수정하지 않고 저장된 실제 trace/request를 읽는다. 모델 API 호출은 없다.

- extract_dp05_final_review.py: 최종 phase의 예정54본문/63prefix를 고정 목록으로 만들고 실제 출력·해시·실패시간을 연결한다. 사람이 쓴 판정은 유지하며 실제본문 해시가 바뀌면 중단한다. 신규본문 판정을 자동으로 내리지 않는다.
- extract_dp05_final_mechanism.py: 실제 요청의 raw 근거, source-backed record span, host retention/read, build분기를 연결한다.
- update_dp05_final_review_manual.py: 저자가 실제 본문을 읽고 내린 수동판정을 저장하는 helper다. 자동 의미 판정기가 아니다.
- audit_dp05_union_span.py: 중단된11행 phase의 supplied 인접범위와 host 거절/실제prompt를 연결한 감사다. 최종phase로 읽지 않는다.
- review_dp01_A2_content.py: BASIC/A2 각6본문을 읽고 고정한 수동판정과 호출/비용 근거를 저장한다.

후보는 evaluator-only 정답값/라벨을 받지 않았다. 허용원문에 적힌 가격·날짜·조건 등은 정상 입력으로 제공했다. 분석자가 이 파일에서 원문·정답과 결과를 함께 읽는 것은 별도 사후 검토이며 후보 입력으로 역전달하지 않는다.
