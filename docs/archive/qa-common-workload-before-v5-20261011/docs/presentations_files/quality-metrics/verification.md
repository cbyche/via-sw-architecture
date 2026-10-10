# ASR-QA 발표자료 확인 범위

2026-10-11, [03-02 원본](../../architecture/12-decisions/decision-packages/03-02-quality-attribute-definitions.md)의 ASR-QA-v4 발표 투영을 확인했다.

- 여섯 QA의 정의/대표 지표/목표/근거/보호 조건과 원본 SHA-256 일치.
- 정확성 18개 상세 사례와 540개 채점 항목 규모, 시간 각 30개, 자원 8시간/3session과 변경 6과제 유지.
- 최종 PPTX 6장, native table 18개, 슬라이드 래스터를 삽입한 페이지 없음.
- finalizer package/layout/font/native table/Artifact Tool import 검사 통과, 지적 0건.
- 최종 PPTX를 재가져와 만든 1920×1080 PNG 6장을 각각 육안 확인. 문구 누락, 잘림, 표 겹침 없음.
- 이전 자료 32개 파일의 archive byte/hash 보존 확인. 새 표/문안/JSON/노트/PNG/링크 일치 검사.

[자동 일치 검사](../../../scripts/presentations/check_quality_metrics.py)와 [원본 투영 재현](../../../scripts/presentations/build_asr_qa_sources.py)을 제공한다. 독립 심사/후보 모델 실행/Windows 실제 메모리 관측/Microsoft PowerPoint 앱 저장과 재열기는 수행하지 않았다. 구조와 렌더 검사는 목표 달성 또는 A/B 승자를 뜻하지 않는다.
