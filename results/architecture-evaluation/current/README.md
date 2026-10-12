# Current Architecture Evaluation Results

> **Status: DP41 A/B six-QA PoC measurement complete — 2026-10-12.**

[최종 비교 보고서](./dp41-p21-mac-text-v7-20261012/final/report.md)는
C0(41A+44A+45A+42A)와 C41(41B+44A+45A+42A)의 P21 각 5회,
총 210건과 독립 변경 실험 12개를 비교한다. 실제 Qwen 의미 호출과 OpenAI 판정,
전사 입력·공통 음성 모형 및 실제 Mac 실행을 사용했다.

| QA 원지표 | A | B |
| --- | ---: | ---: |
| 01 정확성 | 400/630 · 63.492% | 359/630 · 56.984% |
| 02 실패 cap 포함 평균 | 6.420초 | 6.144초 |
| 03 미완료 cap 포함 평균 | 23.534초 | 25.639초 |
| 04 의미 API usage 환산 비용/P21 | $0.132040 | $0.143066 |
| 05 변경 기능 책임 수/과제 | 1.5 | 1.5 |
| 06 Mac phys_footprint peak | 1014.408MiB | 648.532MiB |

수치와 점수의 산출은 완료했으며 양안 모두 QA01~04의 보호 조건은 실패했다.
다섯 paired 차이는 어느 축도 고정한 실용 차이와 일관성 조건을 함께 만족하지 않았다.
QA05의 실제 코드 변경 범위는 동점이며 추가 M04 자연어 실행은 양안 모두 FAIL이다.
전체 제품의 자격이나 A/B 승자를 이 수치로 선언하지 않는다.

- [전체 지표·보호 조건·paired 차이](./dp41-p21-mac-text-v7-20261012/final/qa-results.json)
- [완료 모집단 검수](./dp41-p21-mac-text-v7-20261012/final/completion.json)
- [raw 경로와 재가공 안내](./dp41-p21-mac-text-v7-20261012/raw-audits/raw-data-guide.md)
- [독립 재계산 검수](./dp41-p21-mac-text-v7-20261012/raw-audits/final-independent-review.json)
- [원시 증거 무결성](./dp41-p21-mac-text-v7-20261012/final/evidence-integrity.json)
- [준비·본 실행·judge 전체 호출 비용](./dp41-p21-mac-text-v7-20261012/final/spend-inventory.json)

본 결과는 v7 고정 계약에만 속한다. V1~6과 preparation은 제외·준비 이력이며
실패와 실제 사용 비용을 보존한다. DP44/45/42의 B는 아직 측정하지 않았다.

현재 논의하는 QA는 ASR-QA-01~06이다. 정의, 목표, 보호 조건과 점수 경계는
[현재 QA 문서](../../../docs/architecture/12-decisions/decision-packages/03-02-quality-attribute-definitions.md)를 따른다.
문서 검산과 발표의 가정값은 이 디렉터리의 실측 evidence가 아니다.

이전 실험의 결과와 코드 위치는
[전체 보관 안내](../../../scripts/archive/pre-dp41-45-experiment-tools-20261011/README.md)에 정리했다.
기존 archive 결과는 이전 계약의 provenance이며 현재 QA 값이나 A/B 승자의 근거가 아니다.

각 실험은 결과 전에 정한 DP/QA 계약, fixture/oracle과 source digest, 실제 실행
환경, 모든 실패/timeout, raw trace와 집계 근거를 campaign directory에 함께 보존한다.
mock 실행, 실제 모델 실행과 계산 추정은 구별하고 이전 결과를 덮어쓰지 않는다.

2026-10-11 사용자 지정 보고 방식은
[현재 harness의 보고 요구사항](../../../benchmark/architecture/README.md#poc-결과-보고-요구사항--2026-10-11-사용자-지정)을 따른다.
QA-02는 cap 적용/정답만/오답 포함 실측의 세 평균, QA-03는 cap 적용/완료만의 두
평균을 표본 수와 실패·누락 수와 함께 저장한다. cap으로 raw 시간을 덮어쓰지 않는다.
QA-06은 실제 Mac 환경의 `phys_footprint`와 resident size/RSS를 모두 보존한다.
Mac 관측을 Windows 실측이나 기존 commit 점수로 바꾸어 표시하지 않는다.
이 보고 요구사항은 위 DP41 v7 결과의 CSV/JSON과 최종 보고서에 모두 구현·실행했다.
