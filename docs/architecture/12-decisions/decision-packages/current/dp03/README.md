# DP3 화면과 음성의 지칭 해석

2026-10-22 / 현재 편집 위치 / 기존 설계와 근거 상태 보존

현재 네 대안은 A/B-R1/B-R2/B-R3다. v8 구조 그림은 통합 발표 v10에도 유지된다. 과거 cloud, caption 포함 local, 작은 CPU 예산과 v7 그림을 현재 배치나 측정값으로 읽지 않는다.

2026-10-22 추가한 [QA08 온디바이스 모델 추가 메모리](../qa/qa08-on-device-model-memory.md)는 B-R1/R2/R3의 가중치, 실행 엔진, 추론 상태와 모델 전용 버퍼를 같은 기준으로 관측한다. 목표는 사용자 지정 **1 GB 미만**이며 모델별 개별 상한이 아니다. 아래 기존 설계와 계산값을 새 요구의 실측이나 합격 결과로 바꾸지 않는다.

| 파일 | 역할 |
| --- | --- |
| [design.md](design.md) | 문제, 공통 grounding 계약, 현재 그림과 이전 판본 구분 |
| [cpu-runtime-architecture.md](cpu-runtime-architecture.md) | A/B-R1/B-R2/B-R3의 CPU 실행과 수명, §11 현재 네 대안 |
| [presentation-architecture-contract.md](presentation-architecture-contract.md) | 최신 Component 책임, 호출과 반환, 상태 소유와 실행 배치 |
| [local-producer.md](local-producer.md) | 로컬 생산기의 입력, 출력과 후보 확장 계약; 모델 배치의 현재성은 CPU runtime 문서를 우선 |
| [review.md](review.md) | 설계 판본별 검토와 확인 한계 |
| [history/README.md](history/README.md) | 이전 cloud/local 그림, v7과 작은 CPU 합성 가정의 범위 |

[공통 QA와 목표 및 점수](../qa/README.md) / [평가와 시나리오](../evaluation/README.md) / [현재 발표](../presentation/README.md) / [통합 안내](../README.md)

현재 편집은 이 폴더에서 진행한다. 이전 경로는 보존한 출처이며 새 수정본을 병행 작성하지 않는다. 기존 권고는 사용자 채택이나 accepted ADR 변경을 뜻하지 않는다. 출처 경로와 원본 SHA-256은 [설계 이관 원장](../design-migration.json)에 있다.
