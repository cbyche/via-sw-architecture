# ASR-QA 발표자료 확인 범위

2026-10-11, [03-02 원본](../../architecture/12-decisions/decision-packages/03-02-quality-attribute-definitions.md)의 ASR-QA-v5 발표 투영을 확인했다. [반복 교차 검수](../../architecture/12-decisions/decision-packages/03-02-cross-review.md)에 발견/수정 범위를 기록했다.

- 여섯 QA의 정의/대표 지표/목표/근거/보호 조건과 원본 SHA-256 일치.
- W100 사용자 100/Agent 30장면, 같은 초기 자료/상한/갱신/철회 유지. 정확성 R1 CORE 90/540과 시간 각각 30을 같은 장면에서 읽고 자원은 동일 활동 세 run 전체를 읽음.
- 최종 PPTX 6장, native table 18개, 슬라이드 래스터를 삽입한 페이지 없음.
- finalizer package/layout/font/native table/Artifact Tool import 검사 통과.
- 최종 PPTX 재가져오기에서 만든 1920×1080 PNG 6장을 각각 육안 확인. 최종 렌더에서 문구 잘림/표 겹침 문제를 발견하지 않음.
- 이전 V 자료 32개와 v4 자료 33개의 archive bytes/hash 보존 확인. 새 표/문안/JSON/노트/PNG/링크 일치 검사.

[자동 일치 검사](../../../scripts/presentations/check_quality_metrics.py), [원본 투영 재현](../../../scripts/presentations/build_asr_qa_sources.py)과 [공통 활동 검사](../../../scripts/architecture/qa_common_workload.py)를 제공한다. 독립 심사/실제 후보 모델 실행/Windows 메모리 관측/Microsoft PowerPoint 앱 저장과 재열기는 수행하지 않았다. 구조와 렌더 검사는 목표 달성 또는 A/B 승자를 뜻하지 않는다.
