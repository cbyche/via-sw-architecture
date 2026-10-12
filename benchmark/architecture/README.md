# Current Architecture Measurement Harness

> **Status: DP41 A/B P21 six-QA PoC complete — 2026-10-12.**

[최종 비교 보고서](../../results/architecture-evaluation/current/dp41-p21-mac-text-v7-20261012/final/report.md)와
[raw 재사용 안내](../../results/architecture-evaluation/current/dp41-p21-mac-text-v7-20261012/raw-audits/raw-data-guide.md)에
실제 210건, 독립 변경 12개와 여섯 QA의 원값·보호 조건·실패·paired 차이를 보존했다.

실행 코드는 [dp41_45_poc](./dp41_45_poc/README.md)에 있다. 별도 입력/정답 fixture,
격리 runner, 모델 호출 원장, blind judge, 인과 시간 재계산, 독립 변경 실험과
Mac 메모리 수집기를 구현했다. C0(41A+44A+45A+42A)와 C41(41B+44A+45A+42A)을
P21 각 5회 비교하며, 다른 DP의 B 구현과 전체 제품 구현은 이 실험의 범위 밖이다.

현재 비교의 기준은 [공통 실행 계약](../../docs/architecture/12-decisions/decision-packages/04-40-common-execution-contract.md),
[QA 정의](../../docs/architecture/12-decisions/decision-packages/03-02-quality-attribute-definitions.md)와
[조합 검토](../../docs/architecture/12-decisions/decision-packages/composition-review.md)다.
문서의 목표값이나 계산 예시는 실제 후보 실행 결과가 아니다.

[PoC 수행 순서와 검토 시점](../../docs/architecture/12-decisions/decision-packages/poc-implementation-plan.md)에
대표 세 사례에서 시작하는 단계별 구현, 완료 기준과 사용자 검토 묶음을 기록한다.
[1단계 구현 전 검토 묶음](../../docs/architecture/12-decisions/decision-packages/poc-preimplementation-review/README.md)에
세 사례/정답, 실행 계약, 여섯 QA의 수집/판정, 30개 변경 job과 raw 형식 초안을 정리했다.
이 묶음은 구현 전 검토 이력이며 현재 실행/결과의 상태는 캠페인 원장을 따른다.
현재 QA 원문의 목표와 보호 조건을 달성하도록 후보 출력을 보정하지 않는다.

이전 runner, fixture/oracle, evaluator와 결과는
[이전 실험 전체 보관 안내](../../scripts/archive/pre-dp41-45-experiment-tools-20261011/README.md)에서 찾는다.
기존 archive는 유지하며 이전 DP/QA 계약, 기본값과 점수 산출을 새 실험에 자동 적용하지 않는다.

후보 입력과 평가 전용 정답을 분리하고, 실제 Qwen 의미 호출, 실제 OpenAI 판정,
Agent/음성 모사 및 지정 의미 회귀 재생을 구별한다. raw result는 [현재 결과 위치](../../results/architecture-evaluation/current/README.md)의
새 campaign directory에 보존한다. 이전 result directory를 덮어쓰지 않는다.

## PoC 결과 보고 요구사항 — 2026-10-11 사용자 지정

아래는 사용자가 지정한 수집·집계 요구사항이다. 실행값과 freeze는 각 캠페인에 보존한다.
현재 QA의 cap 적용 평균과 보호 조건을 유지하면서 실패를 포함하는 방법에 따른
차이를 함께 보고한다. 작은 대표 사례의 관측은 해당 사례의 실제 표본 수로 표시하며
P21 전체를 실행한 최종 QA 값으로 표시하지 않는다.

### QA-02: 세 평균을 나란히 보고

| 보고 값 | 산출 방법 | 분모 |
| --- | --- | --- |
| cap 적용 평균 — 현재 QA 주 지표 | 유효 반응은 현재 QA 경계의 시간, 오답·미전달은 5초를 사용 | 전체 지정 관측. P21 5회는 135개 |
| 정답 첫 반응만의 실측 평균 | 최초 핵심 반응이 필수 내용/동작 조건을 충족한 관측의 실제 시간만 평균 | 정답 첫 반응이며 시간이 관측된 수. 최초 오답·미전달 제외 |
| 오답 포함 실측 평균 | 정답 여부와 관계없이 실제 핵심 반응/동작이 관측된 시간을 평균 | 실제 시간이 있는 수. 미전달·시간 누락 제외 |

접수 인사·earcon·모델의 첫 packet·outbox 등록을 핵심 반응으로 세지 않는다.
실제 반응의 Text 표시/핵심 음성 onset 또는 local stop의 마지막 sample을 기록하고,
QA의 유효 종료 판정과 구별한다. 오답 후 정정이 오면 최초 실제 반응과 최초 유효
반응의 시각, 각 응답 ID·내용·판정과 전체 전달 순서를 모두 보존한다.
오답 포함 실측 평균에는 최초 실제 핵심 반응의 시간을 사용하며,
정답 첫 반응 평균에는 처음부터 유효했던 반응의 시간을 사용한다. 오답 후 정정은
별도 건수와 최초 유효 반응 시각으로 보존한다. cap 적용 여부와 보호 조건은
현재 QA 계약으로 판정하고, 뒤늦은 정정으로 앞선 오답 기록을 지우지 않는다.

각 평균에 분자·분모, 정답/오답/미전달/시간 누락 수를 붙인다. 실제 시간이 없는
항목에 0초나 cap을 넣어 실측 평균을 만들지 않는다. 해당 부분집합이 비면 평균은
`N/A (n=0)`다. 정답 여부는 지정 반응의 필수 조건으로 판정하고 관련 QA-01 실패를
같이 연결한다. 다른 조건의 실패를 모든 반응 시간에 일괄 적용하지 않는다.

### QA-03: 두 평균을 나란히 보고

| 보고 값 | 산출 방법 | 분모 |
| --- | --- | --- |
| cap 적용 평균 — 현재 QA 주 지표 | 명세상 완료의 VIA 귀속 시간, 귀속 30초 안에 필수 종료가 없는 관측은 30초 | 전체 지정 관측. P21 5회는 95개 |
| 완료 관측만의 실측 평균 | 관측 종료까지 명세상 유효하게 완료된 관측의 실제 VIA 귀속 시간만 평균 | 실제 완료 수. 미완료 제외 |

완료는 각 사례의 명세된 종료 조건을 뜻한다. 허용된 거부·불명확성 안내·취소는
명세를 충족하면 완료지만, 잘못된 payload나 허위 완료 응답은 완료가 아니다.
30초 뒤 유효 완료가 관측되면 늦은 완료로 따로 표시한다. 주 지표에는 기존
30초 cap을 유지하고 완료 관측 평균에는 실제 시간을 사용한다. cap은 실행 중단
시각이 아니며 공통 관측 종료까지 늦은 반환·효과·정정을 계속 보존한다.

완료/늦은 완료/미완료 수와 각 평균의 분자·분모를 공개한다. 완료 표본이 없으면
`N/A (n=0)`다. 실제 wall-clock, 외부 구간과 causal DAG를 남겨 VIA 귀속 시간을
재계산할 수 있게 한다. 미완료를 제외한 낮은 평균으로 미전달 보호 조건을 통과한
것처럼 보고하지 않는다.

### QA-06: macOS의 두 메모리 지표를 모두 수집

이번 PoC 실행 환경은 실제 사용 중인 Mac이다. macOS 버전·hardware·runtime·후보
버전과 process 범위를 기록하며 Windows에서 실행한 결과로 표시하지 않는다.

- `phys_footprint_bytes`: OS가 해당 process에 귀속시키는 메모리 부담. 압축된
  내부 메모리 등의 회계 항목을 포함하므로 현재 resident RAM과 같은 수치가 아니다.
- `resident_size_bytes` / RSS: 현재 물리 메모리에 resident한 page의 크기.
  resident size와 RSS는 같은 계열 지표의 이름이며 세 번째 독립 지표로 세지 않는다.

동일 sampling 회차에서 모든 VIA process/helper/service의 두 값을 읽고, 시각과
process identity·restart generation·수집 성공 여부·회차 내 시간차를 원시 기록에
남긴다. 50ms 주기와 allocation 사건을 함께 관측한다. 두 지표 각각의 process별
시계열, 회차별 전체 합과 그 합의 관측 최대값을 산출한다. 서로 다른 시각의
process별 peak를 더하지 않는다. 누락 process를 0으로 대체하지 않으며 짧은 peak의
sampling 누락 가능성과 회차 내 시간차를 보고한다.

초기 준비·실행·source 갱신·배경 생산·철회·drain 30초·idle 10초와 다섯 반복을
모두 포함한다. cloud 모델 가중치/KV와 외부 Agent·평가 runner/judge는 VIA 합에서
제외하며 각 process의 포함/제외 근거를 원장에 남긴다.

RSS의 process 합에는 공유 page 중복이 있을 수 있다. OS process 합을 고유 물리
메모리 합이라고 부르지 않는다. 후보가 관리하는 공유 allocation과 실제 copy의
ID·크기·생성/해제 시각·보유 process를 별도 원장에 남기고, OS 값에 그 원장 크기를
다시 더하지 않는다. 중복을 검증해 제거할 수 있는 범위와 확인하지 못한 공유를
구별한다. 원시 단위는 byte, 표시 단위는 MiB다.

[Apple의 task 정보 구조](https://github.com/apple-oss-distributions/xnu/blob/main/osfmk/mach/task_info.h)와
[footprint 회계 설명](https://github.com/apple-oss-distributions/xnu/blob/main/osfmk/kern/task.c)을
구현 시 해당 macOS SDK와 확인한다. 두 Mac 지표를 기존 Windows commit 지표나
그 지표의 0–6점으로 자동 환산하지 않는다.

### 원시 기록과 집계의 연결

raw 시각·시간·판정·효과와 cap 적용 시간을 별도 필드로 저장한다. 후보/사례/반복/
지정 관측 ID로 원시 사건, 판정 근거와 집계 행을 연결한다. 집계에는 계산 방법과
실제로 포함·제외한 관측 ID를 남겨 나중에 다른 방식으로 재가공할 수 있게 한다.
세/두 평균을 반복별·전체·사례별로 보고하고 같은 입력의 A/B paired 차이를 붙인다.
진단 평균은 별도 QA 점수로 채점하거나 기존 정확성/미전달 보호 조건을 대체하지 않는다.
