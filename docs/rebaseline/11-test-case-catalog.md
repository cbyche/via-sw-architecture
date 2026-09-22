# 11. 측정 기준과 Test Case — 공동 검토본

> 상태: **11-A·11-B historical baseline 승인 완료 / W-01~W-03은 11-E에서 W12-G2 Voice 중심으로 재정의 / 측정 구현·target·score 재동결 전**
> 기준 브랜치: `architecture-rebaseline-20260918` / 입력 기준 커밋: `60422f108cb5c282ad6ece1a62d9c1c92ad40b4d`

## 11.1 리뷰 완료 항목

| 문서 | 검토 질문 |
| --- | --- |
| [11-A 측정 기준](./11a-measurement-baseline.md) | 같은 구조 조건에서 무엇을 재며, 모델과 Agent 영향을 어떻게 구분하는가? |
| [11-B 시험 목록](./11b-test-case-catalog.md) | 입력·초기 상태·외부 이벤트·정답·실패 규칙이 실제 사용자 요구와 맞는가? |
| [11-E Voice 반응성 재정의](./11e-voice-responsiveness-measurement-redefinition.md) | Agent delegation 결과, VIA direct response, Agent progress를 어떤 Voice endpoint로 분리해 재는가? |
| [ASR-01 Semantic Model 근거](./11-evidence/asr01-qwen-evidence.md) | Qwen3-8B Windows consumer-GPU planning profile과 VIA 실제 측정을 구분 |
| [ASR-01 S2S Model 근거](./11-evidence/asr01-qwen3-omni-s2s-evidence.md) | Qwen3-Omni-30B-A3B-Instruct의 official theoretical first-packet과 VIA 실제 latency를 구분 |
| [참조 발표자료 적용](./11-evidence/reference-presentation-application.md) | 최종 발표에서 어떤 주장과 근거를 연결할 것인가? |

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

[시험 보조 파일](../../benchmark/rebaseline/README.md)은 입력과 평가 정답을 분리한다. 후보에는 `fixtures/`와 해당 TC의 입력만 노출하고 `oracle/`는 평가기에서만 사용한다. 실제 배포본의 장기 대화·Task 형식을 이 시험 포맷으로 강제하지 않는다.

`cases.json`과 CSV는 94개 전체 목록, `change-cases.json`은 24개 변경 원장, `raw-results.template.json`은 **모두 NOT_RUN/null**로 시작하는 결과 원장이다. 생성 원문은 GitHub에 보관하고 JSON/CSV/HTML은 `build_assets.py`로 재생성한다. 검토 ZIP에는 생성 결과도 동봉했다. candidate의 입력/출력을 연결하는 adapter는 후보 구현에서 작성한다.

## 11.4 다음 게이트

11-A/B의 지표 의미, 비교 경계, 시험 입력/정답은 사용자 리뷰 승인 완료다. [11-C Target & Scoring Baseline](./11c-target-and-scoring-baseline.md)에 7개 ASR의 반복·집계 규칙, 제품 target, 0~5점 구간 초안을 작성했다. **11-C 사용자 승인 전에는 12의 후보 Architecture 비교를 시작하지 않는다.**
