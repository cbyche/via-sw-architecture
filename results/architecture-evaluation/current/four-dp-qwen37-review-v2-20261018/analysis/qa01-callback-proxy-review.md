# QA01 첫 callback과 첫 올바른 내용의 검증 범위

## 현재 수치가 의미하는 것

현재 타이밍 분석은 고정 입력부터 첫 nonempty `DISPLAY_PROXY`의 `ADOPTED_HANDLING` / `DETAILED_RESULT` / `TASK_QUESTION`까지 측정한다. 사용자가 허용한 callback 표시 proxy다. 실제 UI frame을 재거나 첫 내용의 완전한 의미 정답을 기계적으로 증명한 endpoint는 아니다.

`Context::progress`의 host 검사는 resolved adoption, 대상·수행 의도·유효한 형태를 확인하며 각 DP의 source/currentness/geometry guard가 앞에 적용된다. 의미가 틀린 채택도 이 검사들을 통과할 수 있다. 따라서 발표 headline은 **실제 구체적 응답 callback latency proxy, 정확성 별도**다. active QA01의 ‘첫 올바른 구체적 내용’을 완전히 측정했다고 표기하지 않는다.

알려진 잘못된 callback의 시각도 raw에서 삭제하지 않는다. 사후 새로운 correct endpoint를 골라 기존 평균을 바꾸지 않았고 timeout 벌점이나 0을 넣지 않았다. 첫 의미 정답 시각이 필요한 정식 비교에는 동일 사전 내용-unit/정답 계약과 해당 callback을 연결하는 검증이 더 필요하다.

## 기존 진단이 실제 확인한 범위

|DP|첫 내용과 연결한 기존 검사|현재 알려진 오류 / 미관측|확인하지 않은 범위|
|---|---|---|---|
|01|108개 progress의 실제 표시문장·targets·handling intent 저자 전수 읽기. fixed QA02는 첫 원본-ID 집합과 필요한 revision을 검사.|C의 첫 scope·표시문장에 extra 원본이 들어간2건. A/B의 QA02 source set 오류0.|source 제목+처리 의도 안내만으로 상세 가격·날짜·조건 추론 전체가 맞음을 입증하지 않는다. 최종 detail의 날짜 누락/추가 허위 사실을 first handling 오류로 자동 변환하지 않는다.|
|02|fixed N-R는 각 발화 전체의 관계/연결 대상/question/task multiset을 검사. actual first callback과 해당 채택을 raw로 연결.|B E4 U2 r3의 채택 metadata에 불필요한 T-MAIL1건. 해당 표시문장은 R1 비용 정정 의도이며 불필요한 Task를 직접 말하지 않는다.|이 metadata 오류를 보이는 안내 문장 전체 오답1건으로 세지 않는다. N-R 일치만으로 handling 문장의 모든 의미·사실을 증명하지 않는다. Q-MAIL 통지는 public Task 질문 전달이며 사용자 의미 해석 모델 처리시간과 다르다.|
|04|기존 exact-set QA02와 host revision/source/declared geometry 검사. 실제 표시된 A10/B15 first callbacks의 지칭 집합은 기존 oracle와 일치.|A5개 input first callback 미관측. 그중 free3건은 최종 완료도 없음; 초기 revision 실패2건은 후속 revision에서 실제 완료됨. 관측한 callback의 알려진 exact-set 오류0.|지칭 집합 일치가 수행 의도 문장·모든 화면 의미의 완전한 정확도를 증명하는 것은 아니다. 오류 전 proposal은 실제 채택/표시와 구별한다.|
|05|최종 교정36회의 예정 prefix63개를 실제 원 입력·표시 handling·purpose·recipient·대상/source revision과 저자가 전수 대조했다. N-H/본문 검사는 별개다.|저자 수동 판정62 PASS,1 NOT_DISPLAYED. 미표시는 A E3 r2의 adapter/person recipient 혼동으로 실제 callback이 없었다. 현재 관측62개에서 저자가 판정한 handling 오류0이며 전체 독립 의미 검증은 아니다.|주로 구체적인 후속 처리 의도다. 상세 제품 사실·조건 추론 전체를 첫 안내가 증명하지 않는다. N-H나 receiver 본문 marker 성공을 첫 handling 정답으로 자동 전이하지 않는다.|

이 표의 ‘알려진 오류0’은 검사한 필드에서 알려진 오류가 없다는 뜻이며 완전한 의미 정답률100%가 아니다. 현재 QA01 평균에 포함된 알려진 오류와 미검증 사례를 분모에서 숨기지 않는다.

## 실제 알려진 callback 근거

- `basic-dp01-e4-C-r1-67c6c1`, e4-q3: 입력→첫 callback **12843.626583ms**. 기대 원본은 old-research/old-price인데 첫 targets와 표시문장에 old-explanation도 들어갔다.
- `basic-dp01-e4-C-r2-af3f67`, e4-q2: **18580.596875ms**. 동일한 extra 원본과 표시문장 오류다.
- `basic-02-E4-B-r3-899c52`, U2: **3560.370250ms**. actual decision은 CORRECTION→R1이 맞지만 task_id=T-MAIL이 추가됐다. visible text는 ‘Modify the active comparison request R1 … cost …’이고 이 불필요한 Task를 말하지 않는다. 이는 알려진 채택 metadata 오류이며 visible first 문장 오답 판정과 구분한다.

근거 원본은 `basic-v1/raw/runs/<run_id>/trace.json`과 같은 phase의 `episode-metrics.json`에 남아 있다. fixed QA02/N-R 및 proxy timing을 그대로 인용했다. 새로운 사후 oracle는 만들지 않았다.

## QA03와도 분리

GOAL_COMPLETE는 해당 freeze의 host completion 계약을 통과한 상세 결과의 실제 callback이다. 그것이 모든 요청 내용 조건 충족을 뜻하지 않는다. source quote/currentness/shape 계약, 고정 marker/동치 진단, 저자의 실제 본문 검토를 각각 보고한다. 본문 조건이 누락된 GOAL_COMPLETE를 의미 성공으로 포장하지 않는다. 반대로 host 거부로 표시되지 않은 제안은 상세 성공 완료시간에 넣지 않는다.

DP05 QA03는 receiver-shim API 내부 구간을 제외한 명시적 proxy이며 full wall 및 제외 구간과 함께 읽는다. 이는 제품 remote Agent의 end-to-end나 순수 cloud compute만 제거한 시간이 아니다.

## 최종 DP05 첫 handling 저자 검토의 범위

최종 자료는 [prefix 저자 검토](dp05-reuse-union-corrected-v2-prefix-review.json)에63개 고정 기회를 모두 보존했다. 관측62개의 실제 처리 의도는 저자 대조에서 PASS였지만 새로운 정식 QA01 의미 oracle나 독립 전수 판정으로 바꾸지 않는다. A E3r2 h3-mail은 첫 표시가 없으며 provider의 미채택 제안을 실제 첫 callback으로 대체하지 않았다. 기존 pending 상태의 이 문서는 qa01-callback-proxy-review-before-dp05-final.md에 보존했다. timing/raw/고정분모는 변경하지 않았다.
