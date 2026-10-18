# 교차 리뷰: 근거 공급과 업무 인계

2026-10-16 / 화면 지칭 조사 담당자의 독립 문서 리뷰 / 실행과 측정 없음

대상: [문제 1](../01-evidence-supply.md), [문제 4](../04-agent-independent-handoff.md), [공통 QA 후보](../06-qa-candidates-and-coverage.md), [source 원장](../../../../../../references/five-problem-study/evidence-and-handoff.md).

## 1. 현재 판정

문제 1과4는 입력을 받는 주체와 완료 계약을 구별했고, 같은 생산 상태를 사용하면 DP를 통합해야 한다는 중복 조건을 명시했다. H-C의 수신자 읽기는 H-A/B의 생산 시점과 조합 가능한 축으로 정리했으며, native session 재사용과 cache를 특정 안에만 허용하지 않았다. 실제 코드와 VIA에서 추가해야 하는 권한, revision 및 인계 계약도 구별했다.

발견한 P2 한 건과 P3 한 건은 원 작성자가 수정했고 재확인했다. 현재 확인 범위에서 미해소 P1/P2/P3는 없다. 이 판정은 주요 DP의 최종 자격, 정확도 우위나 실행 가능성을 검증한 결과가 아니다.

## 2. 발견과 수정 확인

| ID | 우선도 | 발견 내용과 영향 | 수정 / 확인 상태 |
| --- | --- | --- | --- |
| EH-R1 | P2 | 문제 1 §6의 초기 신규 후보는 ‘현재 해석 모델 입력까지 유효 조건 / 근거가 보존됐는가’를 평가하면서 문제4와 같은 뜻으로 사용할 수 있다고 적었다. 공통 N-H는 외부 실행을 허가하며 확정한 인계 계약을 판정하므로 관찰 지점이 달랐다. 이를 그대로 두면 모델 입력 진단을 인계 성공으로 세고 직접 응답을 포함해 QA coverage를 부풀릴 수 있었다 | **해소.** 문제1 §6은 최종 위임 계약까지 오류가 남는 위임 장면만 N-H에 포함하고, 모델 중간 입력은 진단으로 분리했다. 문제4 §7도 동일 tuple, 실행 허가 endpoint, 전체 계약 판정과 push / pull의 동등한 의미 조건을 사용한다. 수정된 두 절과 06 §3을 재대조했다 |
| EH-R2 | P3 | 문제4 §5의 EX04는 공통 목록상 비용 재조회와 일정 관계 설명인데, H-C 열에는 수신 Agent가 등장한다. 독자는 직접 응답 장면에서만 H-C가 Agent를 추가한 것으로 읽을 수 있다. 위임을 전제로 한 변형임을 명시하면 공통 완료 조건과 N-H 적용 범위가 더 분명해진다 | **해소.** EX03-H / EX04-H를 외부 위임이 추가된 사고실험으로 선언했다. EX04-H에 회의자료 제작 요청을 명시했고, 원래 직접 설명 입력에는 수신 Agent와 N-H를 추가하지 않으며 COMMON-28 변경도 아님을 확인했다 |

## 3. 별도로 확인한 항목

- E-A/B/C는 지속 view, 요청별 view, 재사용 기억의 차이를 설명하며 혼합 가능성을 숨기지 않는다. E-C를 기존 DP-45와 중복 등록하지 않는 조건이 있다.
- H-A/B는 현재 채택 상태의 권위를 공유한다. H-B의 파생 work record가 사용자 수정이나 확인된 외부 실행 사실을 덮어쓰지 않는다.
- H-C는 목표와 필수 조건을 선택적 검색 뒤에 숨기지 않는다. reader capability가 없으면 명시적 fallback 또는 지원 제한으로 남긴다.
- N-H는 외부 Agent의 domain 수행 품질, receipt 시각과 중간 모델 입력을 같은 점수에 섞지 않는다. 유효 reference를 받았지만 외부 Agent가 읽지 않은 경우도 VIA 인계 오류와 구별한다.
- QA01은 구체적인 처리 의도가 먼저 표시될 수 있음을 인정한다. 그 뒤 capsule이나 lease 준비를 첫 응답에 넣지 않는다.
- QA03은 기존 trial endpoint 안의 실제 인과 경로에 포함되는 H-C의 VIA read 처리와 연동만 포함한다. 필수 상세 표시와 전체 음성 요약 준비 endpoint 이후의 downstream pull serving은 별도 진단이며 QA03 완료점을 늘리지 않는다. 외부 Agent의 읽기 선택 / 업무 수행도 제외한다. QA05는 downstream으로 옮겨진 비용을 전체 업무 절약으로 주장하지 않는다.
- QA04의 공통 adapter와 QA06의 bounded buffer를 모든 안에 허용하며 실제 Rust 변경 수나 RAM 수치를 발명하지 않는다.

## 4. 독립 source 확인 범위

문서의 source 원장을 읽고 다음 고정 코드의 실제 내용을 표본 대조했다.

| source | 직접 재조회한 위치 | 대조 결과 |
| --- | --- | --- |
| OpenClaw `e7a155a8ae860d71b1ab5eed45fa4af338011db9` | `extensions/memory-core/src/dreaming-consolidation.ts` L276–455 | 기존 항목 손실 / 중복 / lineage / 문자 예산 검사, null 반환과 append-only fallback 로그를 확인. 파일 쓰기 전체 원자성으로 확대하지 않은 문서 설명과 일치 |
| Deep Agents `7764f1f8d21a594f0cbd31d273a7cad36597d6ba` | `libs/deepagents/deepagents/middleware/subagents.py` L730–870 | isolated 설명 입력, fork의 유효 메시지 / 상태와 private 제외 차이, structured response / 마지막 AI 응답 반환을 확인. 이종 Agent private state migration으로 확대하지 않은 설명과 일치 |
| CrewAI `3da2d4e9932b5f2fa9aa73ea9245e0f8b5c154f8` | `lib/crewai/src/crewai/memory/unified_memory.py` L355–410 | pending save future를 기다리되 예외를 재발생시키지 않는 `drain_writes` 경계 확인. 기억 저장 실패와 원래 업무 실패를 분리한 설명과 일치 |

공통 경계는 현재 QA03 / QA05 상세안과 06의 N-H를 대조했다. 모든 프로젝트의 전체 저장 / 삭제 / session 재개 경로를 독립 추적한 것은 아니다. 실제 상용 Agent API, 원본 접근권한, 모델 실행이나 비용 측정도 수행하지 않았다.
