# DP52 사건 추적과 검토 기록

2026-10-23 / 작성자 자체 검토 / 독립 리뷰 및 실행 시험 없음

## 1. 같은 사건의 결과

아래는 코드 실행 결과가 아니라 [설계](design.md)와 [스케치](code-sketch.md)를 따라 확인한 상태 전이다. 실제 성능 수치나 제품 성공률로 해석하지 않는다.

| ID와 사건 | 한 transaction 확정 | 업무 소유자 적용 확인 | 필수 결과 |
| --- | --- | --- | --- |
| R01 정상 U5 답변 | Q1 소비, T1 연결, receipt 및 outbox 함께 commit | 대상 예약 후 T1의 적용 receipt commit, 입력 측 후속 반영 | T1/E1/Q1에 한 번 적용 |
| R02 T2 진행률이 해석과 겹침 | T1 ReadSet에 무관하면 계속 | T1 link revision 유효하면 계속 | 불필요한 재판단과 T2 대기 없음 |
| R03 Q1 철회가 먼저 확정 | 적용 transaction 거절 | 업무 owner가 ticket 거절 receipt 저장 | 철회된 질문에 답변 명령 생성 금지 |
| R04 답변 적용 뒤 Q1 철회 | 이미 적용한 답변과 뒤 철회 모두 기록 | Applied receipt 유지, 뒤 이벤트 반영 | 실제 먼저 적용한 사실을 지우거나 rollback 완료로 주장하지 않음 |
| R05 E1에서 E3으로 실행 교체 | Q1의 실행 및 판본 검사에서 거절 또는 현재 유효성 확인 | T1 owner가 같은 검사 | E3에 과거 Q1 답변을 자동 전달하지 않음 |
| R06 commit 전 crash | 저장 전체 rollback, 입력 재조회 | 예약 전이면 rollback, 예약 후면 영속 ticket 재개 | 입력 무손실 또는 명시 실패 |
| R07 적용 후 확인 유실 | 저장된 입력 receipt 조회 | T1 receipt 조회 또는 같은 LinkId 재전달 | 두 번째 업무 적용 없음 |
| R08 같은 입력 재수신 | 기존 적용 기록 반환 | 기존 plan/ticket/outcome 반환 | 새 Task와 실행 중복 생성 금지 |
| R09 두 입력이 Q1에 답함 | 열린 질문 slot을 먼저 소비한 입력만 답변 적용 | 업무 owner가 같은 slot을 직렬 적용 | 후속은 stale/명시 수정으로 처리 |
| R10 사용자 답변 장시간 없음 | 열린 질문만 저장, 처리 함수 반환 | 질문 상태 저장, owner는 다른 메시지 처리 | 다른 업무와 새 입력 계속 |
| R11 두 업무에 대한 한 입력 | 내부 전이를 한 transaction에 확정 가능 | operation별 적용 및 거절 receipt | 부분 적용을 전체 성공으로 표시하지 않음 |
| R12 발화 뒤 Q2 전달 | 발화 당시 Q1 exposure 유지 | 같은 exposure proof 사용 | 늦은 Q2를 U5 답변 대상으로 바꾸지 않음 |
| R13 출력 확인 지연 또는 유실 | 해당 입력의 exposure 완전성 보류 | 같은 조건으로 ticket 생성 보류 | 생성된 질문을 이미 들은 질문으로 취급하지 않음 |
| R14 업무 owner 재생성 후 옛 writer 반환 | state revision 조건부 쓰기 검증 | owner epoch와 state revision 함께 검증 | 새 상태 덮어쓰기 금지 |
| R15 외부 전송 뒤 응답 전 crash | outbox 전송 불명 및 조회 | 같은 외부 계약 | 로컬 receipt만으로 외부 완료/정확히 한 번 실행 주장 금지 |
| R16 앞 입력 U1이 아직 미연결 | EarlierInput 의존 저장, 해당 입력 재개 | 동일, 새 Task는 예약 시 안정 ID 생성 | 중복 임시 Task나 전체 대화 장벽 없음 |

Q1 철회가 외부에서 먼저 발생했어도 VIA에 아직 도착하지 않았다면 로컬 사건 순서만으로 외부 유효성을 보장할 수 없다. Agent의 reply precondition 또는 수신 거절을 확인해야 한다. 그 지원이 없을 때 보장할 수 있는 것은 VIA가 관측한 상태에 대한 검증이며, 외부 원자적 질문 소비를 보장하지 않는다.

## 2. 발견하여 반영한 설계 문제

| 발견 | 반영 |
| --- | --- |
| 채팅 초기 스케치의 단일 InputState::Applied가 복수 업무의 부분 적용을 표현하지 못함 | 불변 plan과 operation별 Reserved/Applied/Rejected 도입 |
| ticket timeout을 재연결 허가로 오해할 가능성 | 같은 LinkId 조회 및 재전달, 명시 미적용 거절 후에만 재판단 |
| 업무별 actor 사용만으로 장애 격리나 병렬 DB 쓰기를 주장할 위험 | 같은 process/DB의 공통 장애 및 직렬화 명시 |
| snapshot에 존재한 행만 검사하면 새 후보 추가를 놓침 | 조회 범위 membership revision을 ReadSet에 포함 |
| 업무 재생성 전 옛 실행체가 뒤늦게 저장할 가능성 | owner epoch와 state revision 조건부 쓰기 |
| 거절된 ticket을 예외로 loop 밖에 보내면 업무 owner가 종료됨 | 예상 가능한 거절을 영속 RejectReceipt 전이로 처리 |
| 업무별 async 구조의 비동기 summary가 모델 입력을 뒤처지게 할 가능성 | 기본안은 업무 상태와 routing summary를 같은 transaction으로 저장 |
| receipt 성공 시점으로 QA07의 최초 연결 관측을 미룰 위험 | 예약 및 전달이 실제 처리 기준 사용이면 그 최초 채택을 기록 |
| 진행률 coalescing이 delta 이벤트까지 삭제할 위험 | 완전 snapshot만 합치며 delta는 순서와 gap 확인 |
| 판단 반환 뒤 대기/질문을 별도 무조건 쓰기로 저장하면 새 입력 판본과 경쟁 | try_adopt에서 현재성 검사와 함께 저장하고 실패/보류도 attempt 조건부 쓰기 |

## 3. 구조 차이의 반증 조건

- 첫 방식의 짧은 저장 및 전이 경로가 정상 부하에서 다른 업무의 처리 기한을 크게 방해하고, transaction 축소나 공통 handler 분리로 해소되지 않으면 업무 소유자 방식의 이익을 다시 검토한다.
- 업무 소유자 방식에서 늘어난 receipt 왕복, backlog와 재검증이 지배적이고 업무별 상태 해제 및 처리 분리가 정상 사용의 이익을 주지 않으면 주요 대안으로서의 우선순위를 낮춘다.
- 두 방식이 공통 DB writer와 공유 모델 자원에 의해 거의 같은 경로가 되면 차별화가 약하다고 기록한다. 임의 sleep, 불필요한 전역 lock 또는 모델 차이로 성능 격차를 만들지 않는다.
- 두 방식이 같은 올바른 연결을 유지하면 QA07 차별화로 세지 않는다. 중복 방지와 복구 회귀의 중요성은 유지한다.

이는 향후 검증 질문이며 현재 구현 또는 측정 승인이 아니다. 사례별 timeout과 성능 예산, score band를 새로 동결하지 않았다.

## 4. 남은 확인 사항

1. Agent의 독립 실행, 질문 revision, reply precondition, 중복 방지와 재연결 capability.
2. DP54 출력 증거의 전달 범위, watermark와 불명 상태를 DP52 ExposureCut에 연결하는 실제 계약.
3. DP53에서 예약 뒤 정정과 아직 미전송인 명령 보류를 결합하는 계약. 예약을 외부 실행 시작으로 혼동하지 않는다.
4. 대상 PC의 DB 지연, 업무 수, backlog와 모델 호출 상한. 수치가 없어서 queue 상한을 무제한으로 두지는 않는다.
5. 실제 공통 변경 과제에 대한 두 구현의 모듈 파급. 문서의 예상 수정 범위를 실측 QA04로 바꾸지 않는다.

## 5. 정적 문서 검증

작성 범위는 DP52의 README, 설계, 코드 스케치, 근거와 이 검토 기록이다. 별도 다이어그램이나 PPTX를 생성하지 않았다. 새 실행 코드, 테스트 runner와 모델 호출은 없다. 문서 코드 블록은 컴파일 검증 대상이 아니다.

공유 checkout의 다른 세션 변경은 검사에서 발견해도 임의 수정하지 않는다. 최초 실행 결과는 다음과 같다.

| 검사 | 결과와 범위 |
| --- | --- |
| DP52 파일 및 anchor 검사 | PASS. 5개 Markdown, 로컬 링크와 anchor 26개. 기존 패키지 검사의 LINK와 markdown_anchors 함수를 읽기 전용으로 사용 |
| 코드 fence 짝과 중간점 검사 | PASS. 실행 코드 검증이 아닌 문서 구문 검사 |
| check_active_terminology.py | PASS |
| check_common_qa_v4.py ~ check_common_qa_v9.py | 모두 PASS. 기존 자료 정합성 확인이며 DP52 평가 아님 |
| check_active_markdown_links.py | 전체 FAIL. 동시 작성 중인 DP51, DP54, DP55의 미생성 문서 및 DP51의 상대경로 오류. DP52 오류는 보고되지 않음 |
| check_current_package.py | 전체 FAIL. DP51 README의 아직 없는 design.md에서 중단. DP52 범위 검사는 위 별도 검사로 수행 |
| git diff --check 및 git diff --cached --check | PASS. DP52 신규 파일의 staged diff도 확인 |

공용 검사 실패를 DP52 기능 또는 코드 실행 실패로 해석하지 않는다. 동시에 전체 저장소 검증이 통과했다고 표시하지도 않는다. 다른 세션 폴더의 파일을 고치거나 삭제하지 않았다.
