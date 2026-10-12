# DP41 P21 Mac PoC

DP41 A 모델 주도 READ/CLARIFY/PROPOSE loop와 B 유한 Frame/코드 Resolution Engine을 실제 Qwen 호출로 비교한다. 공통 44A/45A/42A 실행, owner 자료, permission/currentness, async continuation과 출력 경계는 공유한다. A도 병렬 read, B도 부분 의미 호출과 확인 질문을 실행한다.

`fixtures/inputs.json`은 원본·일정 사건을 제공하고 `fixtures/oracles.json`은 별도 evaluator만 읽는다. 후보나 Agent stub에 정답을 주지 않는다. Agent는 실제 payload의 source/recipient/body lineage를 보존하며 보고서/메일 결과와 외부 대기를 모사한다. 동일 전사와 공통 음성 출력 profile의 실제 queue/길이/중단을 실행한다. 이 실험은 음성 인식 품질을 평가하지 않는다.

본 실행은 `campaign.py freeze`가 코드/입력/정답/QA/가격/설정 hash를 고정한 뒤 `run`이 A/B, B/A를 교대로 5회 각각 격리 실행한다. 21×5=105개 trial, 630조건, 135반응, 95사이클이 각 방안의 분모다. 실패는 제외하지 않는다.

QA01은 blinded OpenAI judge의 C1~C6 필수 사실별 판정과 효과·전달 trace 검토, QA02는 5초 실패 cap/정답만/오답 포함 실측 평균, QA03은 30초 cap/완료만 평균이다. 같은 관측 DAG에서 외부 Agent 작업·사용자 대기·사용자 발화의 기간만 0으로 재계산하며 parallel join, resource queue와 VIA 모델/음성 출력 시간은 유지한다. 원 wall-clock도 남긴다.

QA04는 실제 VIA 모델 사용량×고정 단가를 활동별로 계산한다. evaluator 비용은 별도다. QA05는 같은 frozen source에서 M01~M06을 독립 변경하고 실제 delta·실행 회귀·두 reviewer로 U01~U18 책임 수를 확정한다. QA06은 50ms 간격 Mac `task_info`의 phys_footprint/resident size를 수집하며 다섯 활동 전체의 실제 동시합 peak가 주 값이다. 기존 MiB 경계는 Mac PoC의 비교 척도로 적용한다.

원본 `*.jsonl`, Context/응답 JSON, source gzip, 명령 payload, 전달 영수증, DAG, 메모리와 변경 diff를 결과 폴더에 보존한다. credentials/Authorization header는 저장하지 않는다. 실제 모델 측정·모사 사건·변경 회귀를 구분하며 발표 v4 가정값을 결과로 가져오지 않는다.
