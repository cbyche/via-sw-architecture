# v5 검토와 검증

2026-10-23 / [시작](README.md)

## 독립 검토

기존 독립 검토 에이전트 `verify_handoff_commits`가 계산기·명세·원장을 읽기 전용으로 검토했다. 구현 작성과 분리했다.

| ID | 발견 | 수정 | 상태 |
| --- | --- | --- | --- |
| R01 / P2 | Agent 질문·철회 통지의 외부 제공과 VIA 수신·채택 시각이 같아 전송 경로 누락 | source→network0.04→adopt0.03을 분리하고 received_s 기록 | 독립 재검토 완료 |
| R02 / P2 | 기존 목표의 최초 해석2초에 모델 비용과 client 메모리 누락 | 600입력/136출력 토큰의 semantic 작업으로2초 재현, 비용·메모리 포함 | 독립 재검토 완료 |
| R04 / P2 | v4의 메모리 owner·수명, 슬롯 pool과 chunk 식 검증 일부가 새 검사에서 빠짐 | node/config에서 기대 자원 목록과 크기·수명을 독립 도출하고 실제 pool·prefix 식 검사 복원 | 독립 재검토 완료 |
| V01 / 자체 검증 | Excel 엔진에서 음성 시각0을 빈칸으로 오인 | COUNT 기반 숫자 유무 검사, 0/빈칸/복구 probe | 수정 및 재계산 통과 |
| V02 / 자체 보완 | U23의 기존 R0 buffer가 자원 원장에 누락 | 공통 초기 잔여12초를 명시해0.576MB 포함, 새 생성비용 없음 | 독립 재검토 완료 |
| R03 / 보강 | QA01/03 공통 병목과 첫 응답 뒤 잔여 처리 설명 부족 | 전체24목표 진단 표와 구성별 원장 추가 | 독립 재검토 완료 |

초기 검토는 168개 계산의 원래 DAG wall 재현, NORMAL 사전 분류, U19 buffer/재개/tail, U23 사건별 구간, QA08 귀속 피크를 확인했다. 최종 소스 재검토에서 R01/R02/R04 closure와 R03 보강을 확인했고 열린 P1/P2는 없다. 현재 수치의 물리적 타당성이나 실측 우위를 검증했다는 뜻은 아니다.

## 검증 범위

- `check_common_qa_v5.py`: 168사례, 294예정 사건의 시각·분류·분모, DAG wall 재현과 제외 투영, 채널 의무, 사용량/비용, 메모리 전체 경계와 v4 보존.
- Excel: Artifact Tool 수식 재계산, 음성 시각0/빈칸과 비용 단가 변경 후 복구, 정확도 빈 관측 유지, 모든 시트 미리보기 확인.
- 산출물: XLSX 저장값과 원장 대조, 생성 재현, HTML의 모든24입력×6대안 전환과 로컬 링크 확인.
- 저장소: 현재 패키지, 링크, 용어, v4 및 새 v5 검사와 diff 확인.

새로운 오류 주입11건(비용·수신·분류·workspace·weights·관계 producer·슬롯·prefix·점수·완료 edge·최초 해석 예산)을 모두 검출한다. QA07의 기존 해석 정답은 부분 진단이며 응답 적합성 확장 미평가 상태에서 대표값은 미산출로 유지한다.

최종 실행 결과와 해시는 [원장 검사](artifacts/ledger-validation.json), [산출물 검사](artifacts/artifact-validation.json), [HTML 검사](artifacts/html-validation.json), [렌더 기록](artifacts/outputs/01a15dfe-v5/render-verification.json)에 남긴다. 완료 상태는 해당 결과를 따른다.

실제 제품·모델·Windows 계측, native Excel 동작과 실제 브라우저 시각 검증은 수행하지 않았다. HTML 검사는 오프라인 DOM handler/string harness이며 브라우저 레이아웃 검증이 아니다. Excel 그림은 Artifact Tool 렌더다. 내용 정답, 실제 음성 전달, 지원 최대 부하와5회 메모리 검증, 제품 적합성은 미확인이다.
