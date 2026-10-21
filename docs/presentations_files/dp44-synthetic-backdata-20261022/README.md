# DP44 합성 백데이터

2026-10-22 / DP44-DELIVERY-SYN-1.0 / SYNTHETIC_AUTHORED_NOT_MEASURED

[선택 결론과 사례 설명](../../architecture/12-decisions/decision-packages/seven-problem-review/dp44-local-voice-execution-20261021/synthetic-comparison-20261022/README.md)을 먼저 읽는다. [HTML 사례 탐색기](outputs/01a154de/guide.html)에서 같은 사례의 A/B 처리 단계와 최초 응답, 비용, 재생 구간을 조회할 수 있다. [Excel 계산표](outputs/01a154de/DP44-synthetic-backdata.xlsx)는 9개 시트로 집계와 원자료를 제공한다.

| 자료 | 내용 |
| --- | --- |
| [synthetic-data.json](synthetic-data.json) | 전체 원장과 가정, 원값 및 집계 |
| [summary.json](summary.json) | A/B QA01 화면과 음성, QA02~07, 유형별 집계 |
| [inputs.csv](inputs.csv), [responses.csv](responses.csv) | 80평가 입력의 정답과 A/B 160응답 |
| [sources.csv](sources.csv), [deliveries.csv](deliveries.csv) | Agent 문안과 판본, 도착, 화면, 음성 진단 |
| [nodes.csv](nodes.csv), [goals.csv](goals.csv) | 3,020단계의 의존과 180개 구성별 목표 원장 |
| [jobs.csv](jobs.csv), [playback.csv](playback.csv), [history.csv](history.csv) | 생성, 취소, 실제 출력에 대응하는 작성 구간, 이력 사건 |
| [usage.csv](usage.csv), [memory.csv](memory.csv), [changes.csv](changes.csv) | 모델 사용량, 동시 RAM, 동일 변경 과제 |
| [validation.json](validation.json), [workbook-validation.json](workbook-validation.json) | 원장 및 Excel 재산출 검증 |
| [publication-validation.json](publication-validation.json) | 문서, 그림, HTML 검사와 검증 한계 |
| [manifest.json](manifest.json) | 최종 원장, 생성 소스, 문서와 산출물의 SHA256 |

Excel 비용은 Rates 시트의 가상 단가를 변경하면 다시 계산된다. 시간 DAG와 메모리의 상태 생존 구간은 Python 생성기가 소유하며, workbook의 임의 셀 편집이 전체 실행 시나리오를 다시 시뮬레이션하지는 않는다. 의미 채택의 100%는 공통 정답 fixture이며 실제 모델 정확도 예측이 아니다.

9개 시트를 렌더해 표시를 확인하고 핵심 집계를 원장과 대조했다. 음성 단가를 두 배로 변경했을 때 사용량 기반 비용이 반영되는지 확인한 뒤 원복했다. native Windows Excel은 실행하지 않았다. HTML은 문법 검사를 통과했으나 로컬 listener 권한과 file URL 보안 정책 때문에 브라우저 시각 검증을 완료하지 못했다. 저장소 검사의 최종 상태는 발행 기록에 남긴다. 원시 및 작성 수치는 이전 실측 자료와 분리했다.
