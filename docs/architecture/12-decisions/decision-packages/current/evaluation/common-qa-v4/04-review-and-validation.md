# 독립 검토와 검증 기록

2026-10-22 / `COMMON-QA-MODEL-4.0` / [시작](README.md)

## 1. 범위

작성자는 공통 입력·가정·계산기·문서·Excel·HTML을 작성했다. 별도 검토자 `/root/verify_handoff_commits`가 설계 대비 계산 경로와 의미, 검증 코드의 누락 검출 능력을 읽기 전용으로 검토했다. 검토자는 파일 수정이나 commit을 하지 않았다. 이전 v3 검토를 v4 검토의 대용으로 사용하지 않았다.

이 검토는 실제 모델 정확도, 장치 처리 시간, 실제 RAM, Windows Excel 동작이나 제품 적합성을 검증한 것이 아니다. 명세에 따른 계산과 해석의 일관성 검토다.

## 2. 독립 검토 발견과 수정

| ID | 중요도 | 발견 | 수정과 확인 |
| --- | --- | --- | --- |
| R01 | P2 | DP3 B 선행 시각 후보 생산 후의 의미 확정 비용이 A와 달리 빠짐 | U12/U22의 B 세 구성에 `pre_adopt` 호출과 비용 추가. 독립 재확인 완료 |
| R02 | P2 | 후보 부족 보완에서 producer 재생산과 초기 binder 비용이 누락됨 | U11/U24에 초기 binder `expand_need`(1,640입력/32출력), 원본 확장, UI/SAM 재생산, revision 게시, 최종 binder를 연결. 6행의 단계와 $0.001768 초기 호출 비용 독립 재확인 완료 |
| R03 | P2 | U19의 첫 음성은 chunk 기준인데 순수 재생은 전체 생성 완료 뒤에 시작함 | 같은 job prefix를 관측하고 실제 `first_audio_output`에서 `question_play` 시작. 7구성 재확인 완료 |
| R04 | P2 | DP44 B가 현재 수행 안내를 재생하는 중 다음 응답을 미리 생성할 수 있음 | 공통 `next_speech_selection` 의존을 두고 B 생성은 다음 선택 이후 시작. A는 준비를 계속할 수 있음. 독립 재확인 완료 |
| V01 | P2 | 비용행이 누락·중복되거나 요율이 바뀌어도 기존 검사에서 통과할 수 있음 | 모든 node의 사용량과 profile 요율에서 예상 비용 multiset을 도출해 일대일 비교 |
| V02 | P2 | RAM owner, workspace와 수명이 잘못되어도 기존 interval 합 검사로 놓칠 수 있음 | node 종류에서 필수 load/workspace/producer 상태, owner, MB와 유지 구간을 독립 도출. node.state와 interval의 동시 누락도 검사 |
| V03 | P2 | 실제 슬롯을 막지 않는 선행 작업을 자원 대기 원인으로 기록할 수 있음 | 슬롯 풀을 다시 구성해 slot 범위, 가장 이른 실행, 실제 predecessor와 blocking edge 확인 |
| V04 | P2 | 의미 호출과 달리 음성 및 첫 chunk 시간식은 검사하지 않음 | profile setup, 입력 토큰, 생성량과 속도로 speech와 prefix duration 확인 |

작성자 자체 점검에서는 공유 runtime의 중복 시작, warm A의 중복 호출, 과거 슬롯 사용만으로 생기는 가짜 의존, 기존 목표 정정의 원점, 음성 후속 작업의 RAM 관측 범위를 수정했다. QA02/QA07의 새 정확도를 이전 오류율이나 정답 복사로 만들지 않는 원칙은 초기 방법론 검토부터 적용했다.

최종 독립 재검토에서 R01~R04와 V01~V04가 모두 닫혔으며 남은 P1/P2는 보고되지 않았다. 이후 표시 반올림 경계는 12자리에서 부동소수점 잔차를 정리한 뒤 같은 표시 자릿수를 적용하도록 맞췄다. 원장 숫자는 그대로 두고 최종 반출·재현·HTML 검증을 다시 통과했다.

## 3. 실행한 검증

| 검사 | 확인 범위 | 상태 |
| --- | --- | --- |
| 원장 독립 재계산 | 7구성, 168사례, 2,507단계, 1,247비용행, 657 RAM interval, 42변경 과제 | PASS |
| 결정적 재현 | 별도 임시 경로에서 ledger/summary/CSV 및 사례·결과 Markdown을 재생성해 바이트 비교 | PASS |
| 오류 주입 | 비용 누락·중복·잘못된 rate, SAM workspace 누락, 범위 밖 slot, 잘못된 음성 생성량의 6종 변조 거부 | PASS |
| Excel 계산 | 4,953수식, 요약 80셀과 JSON 대조, 전체 오류 값 검색, 단가 변경 반영 후 복원 | PASS |
| 정확도 빈칸 | 231개 지칭과 168개 의미 관측 셀을 빈칸으로 유지. 전체 입력 및 오답 probe 후 복원, 미산출 확인 | PASS |
| XLSX 반출 | ZIP, 수식과 cached numeric/string 값, 빈 결과 수식의 허용 범위, 요약 재읽기 | PASS |
| Excel 시각 확인 | 10개 시트의 주요 표 렌더를 직접 확인. 전체비교는 전체 10방안과 차이 표 확인 | PASS |
| HTML | embedded JSON과 원장, 문법, 로컬 링크, 24사례×6비교=144개 선택 조합의 offline DOM handler/문자열 확인 | PASS |
| 패키지 보존과 링크 | immutable snapshot 127개, COMMON-28 28사례, 발표 36쪽, 그림 35쌍과 로컬 링크·용어 검사 | PASS; 발표 v10 내용 미반영 상태 보존 |
| 실제 브라우저 화면 | file URL이 기존 브라우저 보안 정책으로 차단되어 우회하지 않음. offline 검증을 실제 렌더로 주장하지 않음 | NOT_COMPLETED |
| Native Excel/제품/모델 | 실행하지 않음 | NOT_RUN |

기계 기록: [Excel 작성 검증](artifacts/outputs/01a15dfe-v4/render-verification.json), [반출·재현·오류 주입](artifacts/artifact-validation.json), [HTML 검사](artifacts/html-validation.json). Excel의 원장 단위 trace와 모든 사전·미사용·취소 과금 행을 보존한다.

## 4. 보존과 남은 경계

이전 v3 문서·원시·XLSX·HTML을 수정하지 않았다. 현재 패키지의 immutable snapshot은 기존 SHA256으로 검증한다. COMMON-28, 기존 QA01~06의 정의와 점수, QA07의 별도 제안 및 과거 실측, accepted ADR, Reference Architecture와 그림, 발표 v10을 변경하지 않았다. 다른 세션의 QA07 상세화 commit은 이번 변경에 포함하지 않는다.

검사 통과는 작성 가정의 현실성을 증명하지 않는다. 전체 560 MB 기반은 512 MB 요구를 초과하며 실장 메모리 계측이 필요하다. 정확도 후보 출력은 0개다. 실제 입력 자산, adapter, 실패 규칙과 실행 동결은 후속 실측의 준비 작업이다. 결합된 B 성능이나 최종 대안 채택도 미판정이다.
