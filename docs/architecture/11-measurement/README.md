# 11. 측정 기준과 Test Case — 공동 검토본

> 상태: **W-01~W-03 Voice 정의 완료 / machine contract·측정 구현·target·score 재동결 전**

## 11.1 리뷰 완료 항목

| 문서 | 검토 질문 |
| --- | --- |
| [Voice 반응성 정의](../08-quality-attributes/voice-responsiveness.md) | Agent delegation 결과, VIA direct response, Agent progress를 어떤 Voice endpoint로 분리해 재는가? |
| [Test Case Catalog](./test-case-catalog.md) | 입력·초기 상태·외부 이벤트·정답·실패 규칙이 실제 사용자 요구와 맞는가? |
| [Working-12 Scoring Contract](./scoring-contract.md) | W-04~W-12와 재동결 전 상태를 어떻게 기록하는가? |
| [ASR-01 Semantic Model 근거](../08-quality-attributes/evidence/asr01-qwen-evidence.md) | Windows consumer-GPU planning profile과 VIA 실제 측정을 구분 |
| [ASR-01 S2S Model 근거](../08-quality-attributes/evidence/asr01-qwen3-omni-s2s-evidence.md) | official theoretical first-packet과 VIA 실제 latency를 구분 |
| [참조 발표자료 적용](../08-quality-attributes/evidence/reference-presentation-application.md) | 최종 발표에서 어떤 주장과 근거를 연결할 것인가? |

**ASR은 기존 7개 그대로이다.** 이번 작업은 측정 계약과 시험 입력의 검토이며, 후보 Architecture의 성능·정확도·승패를 산출한 작업이 아니다.

## 11.2 산출물과 증거 상태

| 산출물 | 수량·상태 |
| --- | --- |
| 승인 UC에 연결된 명세 | UC 18개 / 명시 변형 94개 모두 연결 |
| 독립 변경 분석 명세 | Agent 9개 / Model·Context·기록 15개 = 24개 |
| 안전 판단 기회 | 24개, 허용 6개·차단 18개. 기존 UC의 보강 시험이며 새 UC가 아님 |
| 실제 내용이 있는 prompt 초안 | 8개 목적, system·schema·context·user·출력 예시 분리 |
| 원천 자료 | 가상의 문서·메일·일정·화면·대화·Task·권한·기억 JSON, 정해진 입력 이벤트 및 HTML 화면 |
| 실제 사람 녹음·Windows 화면 캡처 | **미제작**. 합성 자료를 실제 사용자 녹음이라고 부르지 않음 |
| 공식 tokenizer 실행 | **미실행**. 다운로드/라이브러리 접근 제한을 기록. 정확한 token 수를 추정 글자 수로 대체하지 않음 |
| 실제 Qwen/S2S 추론 및 VIA 후보 시험 | **미실행** |
| 계산·집계 보조 코드 검증 | 수행. 이는 VIA 품질 평가가 아니라 계산 규칙의 단위 검증 |

94개 TC의 논리 명세와 합성 입력은 준비했다. 음성 timing·S2S 의미 품질까지 검증 가능한 완전한 실제 장비 시험셋이 만들어졌다고 주장하지 않는다. 실행 준비 상태는 각 TC의 `artifacts`에 별도로 기록한다.

## 11.3 원자료 사용

[과거 시험 보조 파일](../../../benchmark/archive/w12-g1/README.md)은 입력과 평가 정답을 분리했다. 새 구현에서도 후보에는 fixture와 해당 TC의 입력만 노출하고 oracle은 평가기에서만 사용한다. 과거 파일은 새 Voice timing 계약이 아니므로 그대로 실행하지 않는다.

`cases.json`과 CSV는 94개 전체 목록, `change-cases.json`은 24개 변경 원장, `raw-results.template.json`은 **모두 NOT_RUN/null**로 시작하는 결과 원장이다. 생성 원문은 GitHub에 보관하고 JSON/CSV/HTML은 `build_assets.py`로 재생성한다. 검토 ZIP에는 생성 결과도 동봉했다. candidate의 입력/출력을 연결하는 adapter는 후보 구현에서 작성한다.

## 11.4 다음 게이트

기능 fixture와 oracle은 검토 기반으로 유지한다. 다음 게이트는 DP별 applicability, Voice event, prompt/token ledger, 반복·집계, target과 score를 결과 전에 새 machine contract로 동결하는 것이다. 그 전에는 새 W-01~W-03 후보 측정을 시작하지 않는다.
