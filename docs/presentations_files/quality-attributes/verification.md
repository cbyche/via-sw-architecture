# ASR-QA 발표자료 확인 범위 — v6

2026-10-11, [03-02 원본](../../architecture/12-decisions/decision-packages/03-02-quality-attribute-definitions.md)의 ASR-QA-v6/P21 투영과 [반복 교차 검수](../../architecture/12-decisions/decision-packages/03-02-cross-review.md)를 확인했다.

- 21개(2/5/3/3/4/4) 전체 사례 선정/QA 대응, 정확성105/630·반응135·처리95, 같은 P21 전체5회 비용 평균/메모리 최대 일치.
- 여섯 QA의 정의/지표/목표/근거/보호 조건과 원본 hash 일치. 우선순위는 기존 번호순, 변경 추천은 미채택.
- 최종 PPTX6장, native table 6개, 슬라이드 래스터 삽입 없음. 요약은 모든 사례 대응표3장 포함.
- [최종 파일 확인 기록](validation-summary.json): package/layout/font/native table/Artifact Tool import 통과.
- 최종 PPTX에서 렌더링한 1920×1080 PNG6장을 각각 육안 확인, 최종 잘림/겹침 문제 발견 없음.
- 32+33+38+37+40+40=220개 archive 파일 bytes/hash 보존. 기존17개 checkpoint 사실과 S1-03 별도 기능 회귀 보존.

[자동 일치 검사](../../../scripts/presentations/check_quality_attributes.py), [원본 투영 재현](../../../scripts/presentations/build_asr_qa_sources.py)과 [공통 활동 검사](../../../scripts/architecture/qa_common_workload.py)를 제공한다. 실제 모델/후보/Windows 실행과 Microsoft PowerPoint 앱 저장·재열기는 수행하지 않았다. 목표와 가정 계산은 측정 결과나 A/B 승자가 아니다.

2026-10-11 Qwen/GPT 개정: 정확한 Instruct-2507/Singapore 단가와 130 token/s 작성자 가정을 원본·JSON·발표 노트에 반영했다. 변경된 요약2장/부록QA-02·04를 각각 확인했고 나머지9 PNG는 이전 검수본과 bytes가 같다. 최종 package/layout/font/native table/Artifact Tool import는 통과했으며 finding/warning은 0이다. 모델/API/Windows 성능 실측은 수행하지 않았다.

2026-10-11 발표 정합성 갱신: 총135반응 = 일반105 + 집중30이며 일반 95% 보호 기준은 100/105다. 115/110 오기를 원본/입력/검산/투영/QA 두 발표와 통합 v5에서 정정했다. 평균/stop 목표와 P21 사례/정답/반복은 유지한다. 정정 전 원본과 native PPTX는 별도 archive에 보존했다.
