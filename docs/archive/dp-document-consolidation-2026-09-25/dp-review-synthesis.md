# VIA-DP 전수 정리·검증 기록

> 2026-09-25 · 18개 후보 문서 정리 · QA 실행 결과 아님

## 1. 목적과 완료 범위

가능한 DP를 완성도 있게 수록하고 중간 요약의 과거 추천이 누락·오해를 만들지 않도록 정리했다. 기존 01~13 A/B를 Component·Process·메시지·상태 수준으로 보완하고 14~18을 추가했다. [전체 index](./README.md)와 [요약](./dp-executive-summary.md)이 현재 진입점이다.

## 2. 중복·누락 검토

이전 9개 계열과 25개 주제를 [매핑 원장](./legacy-dp-mapping.md)에 모두 연결했다. Task writer=14, observation authority=15, working context=16, materialization=17, protected use=18을 추가했다. INT의 route/게시/실행 범위는 04·06·01로 분리했으며 EXEC의 A/B 문자가 현행 11과 뒤집히는 점을 표시했다.

관련 질문을 합치지 않도록 02/14, 05/16/17, 09/15, 04/18, 08/12, 10/11/13을 구별했다. 과거 MODEL placement는 현재 공통 조건이고 외부 domain planning·비PC 범위는 재개하지 않았다.

## 3. QA·후보 검토의 의미

18개 보고서 각각이 현행 19개 QA의 예상·확실성·인과·역할을 갖는다. 구조 인과가 있어도 크기가 작거나 한쪽만 유리할 수 있다. 그런 후보도 inventory에는 보존하고 최종 평가 대상 선정은 별도로 한다. QA-41의 모델 복제 가설과 DP-11의 외부 Agent embed 해석을 제거했다. 메모리 ASR·장애 격리 최우선 추천은 유지하지 않는다.

## 4. 구현 근거와 미확인 사항

- 06: 역할별 prompt/schema가 있으나 실제 모델 정확도·지연은 미측정.
- 09: native fixture 변환 구조가 있으나 외부 제품 capability 검증과 전체 9건 ledger는 미완료.
- 11: bridge·worker fixture는 있으나 OpenClaw·Hermes Client의 제품 fault 근거가 아님.
- 14: SharedTaskService·PerTaskSupervisors·공통 SQLite Repository가 있으나 현행 Voice·복구 QA 검증이 아님.
- 15~18: 이번에 추가한 문서 후보. 제품 A/B 구현·measurement freeze·실행은 하지 않음.

모든 후보에 S2S 1개·semantic LLM 1개를 적용한다. Task supervisor·prompt·session·worker 수와 모델 수를 혼동하지 않는다. 모델 queue·cache·취소 지원은 실제 profile 확인 전이다.

## 5. 검토 단계

| 검토 관점 | 확인·반영한 것 |
| --- | --- |
| 책임·제약 | 모델 수·외부 Runtime 경계·사용자 privacy 범위 유지 |
| 배타성·steelman | cache·lazy·batch·library·복구 hybrid 허용 후 최종 권한 규칙 비교 |
| 구현 추적 | 입력→component→queue/저장→외부 호출→반환·revision 검사를 설명 |
| QA 인과 | 전체 19개 유지, non-participation·미확인·동점 허용, 대표값 과장 금지 |
| 읽기·탐색 | 옛 후보는 compatibility 안내로 전환, 이전 요약은 historical 사본 보존 |

이는 같은 작성자의 반복 검토이며 독립 심사나 실측 통과가 아니다.

## 6. 자동·시각 검증

2026-09-25 로컬 문서 검사 결과다. 제품 QA 측정과 구분한다.

| 검사 | 결과 |
| --- | --- |
| `check_active_markdown_links.py` | PASS — active Markdown 90개 로컬 링크 |
| `check_active_terminology.py` | PASS — 현행 문서의 퇴역 용어 규칙 |
| `check_qa_catalog.py` | PASS — active QA 19개·기존 migration 유지 |
| 보고서 구조 | PASS — 18개 × 11개 절, QA 342행의 ID·5열 구조, A/B 36개 구현도·동작 순서 |
| 사고실험·요약 동기화 | PASS — 사고실험 앵커, NOT_RUN·steelman·배타성 표기, 요약 8개 A/B 그림과 상세 원문 일치 |
| Mermaid 렌더링 | PASS — 76개 전체를 Mermaid 10.9.3·headless Chrome으로 렌더링; HTML label overflow 0건 |
| 시각 검토 | 18개 A/B 36개를 나란히 검토. 05·09·10·17의 과도한 왕복선·가로 확장을 줄인 뒤 재렌더링 |
| 이전 내용 보존 | PASS — 이전 요약 5개·candidate 8개 원문 보존 대조; 기존 사용자 수정도 archive에 보존 |
| `git diff --check` | PASS — 공백 오류 없음 |

렌더링 결과는 로컬 검토용 임시 산출물이며 source에 포함하지 않았다. GitHub의 다른 Mermaid 버전·좁은 화면에서는 배치가 달라질 수 있다. 넓은 그림은 확대하거나 발표에서 A/B 별도 페이지로 사용한다. 글자 잘림 검사·작성자 시각 검토는 독립 심사나 제품 동작 검증을 대신하지 않는다. 실행 코드는 변경하지 않아 Rust 후보 시험은 이번 문서 검증 범위에 포함하지 않았다.

## 7. 보존과 후속 검토

기존 요약 사본은 docs/archive/dp-review-pre-inventory-2026-09-25에 보존했다. 원래 재선정 문서의 사용자 로컬 수정도 포함한다. archive 문구·옛 번호를 현행 규범으로 읽지 않는다. 신규 내용은 VIA-DP·현행 QA만 사용한다.

최종 DP/ASR 선정, 실제 Windows·모델·Agent 연결, 전체 변경 ledger, 목표·점수·freeze·측정은 남아 있다. 현재 문서 수와 QA coverage를 ‘18개 모두 강한 trade-off 입증’으로 해석하지 않는다.
