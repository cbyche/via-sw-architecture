# 동일 사례·환경의 공통 계산 계약

`COMMON-QA-MODEL-4.0` / 2026-10-22 / [시작](README.md)

## 1. 판본과 근거

사용자는 v3의 개별 DP profile을 같은 사례와 환경에 맞추는 방향을 승인했다. v4는 기존 원값의 배율을 조정하지 않고 새로운 공통 입력과 계산 모델을 작성한다. `ANALYTICAL_SIMULATION_NOT_MEASURED`이며 제품 실행, 모델 호출, 실제 장치 관측은 없다. v3와 기존 실측은 당시 계약 그대로 보존한다.

24개 입력의 설계 커버리지 묶음을 각 구성에 한 번씩 계산한다. 7구성×24=168개의 계산 행은 모델 실험 168회가 아니다. 정규분포 잡음, 임의 실패율, 꾸며낸 반복 편차를 추가하지 않는다. 승자·합격 판정과 새로운 점수표를 만들지 않는다.

## 2. 구성과 동일 조건

DP2 A + DP3 A + DP5 A + DP44 A를 기준으로 한 DP만 변경한다. DP3 B는 R1 모델 회수, R2 상주, R3 worker 분리의 세 실행 대안을 구별한다. 설계 기준은 [DP2](../../dp02/design.md), [DP3 CPU 계약](../../dp03/cpu-runtime-architecture.md), [DP5](../../dp05/design.md), [DP44](../../dp44/presentation-architecture-contract.md)다. 개별 B를 합친 구성은 계산하지 않았다.

| 공통 항목 | 적용 |
| --- | --- |
| 입력과 원본 | 동일 24입력, 현재 revision과 권한, 같은 원본 도착 시각 |
| 목표와 의무 | 요청 결과의 UI 상태와 요청된 음성 전체 준비, 취소 정산·재개 후반을 같은 terminal 조건으로 적용 |
| 환경 | 작성 가정: CPU 8core, RAM 16GB, SSD, GPU 없음 |
| 슬롯 | 의미 모델 2, 음성 생성 4, 무거운 시각 단계 1; 로컬 시각 처리 전체 4 CPU thread 한도 |
| 초기화 | 사례별 공통 원본 snapshot. 기존 확정 사실과 유효 audio는 양안 동일 재사용 |
| 사전 활동 | B 전용 관계·화면 생산, 미사용 생산, 갱신, 폐기와 중단 tail도 비용/RAM에 포함 |
| 비교 단위 | 구성당 입력 24, 등록 목표 24, 지칭 표현 33, 변경 과제 6 |

v4의 의미 모델은 원격 호출 의존성으로 계산한다. Reference Architecture의 공유 on-device Omni 구현 성능을 재현했다고 주장하지 않는다. 순수 계산의 슬롯 수는 실제 공급자가 보장하는 동시 처리량이 아니다. 이전 TEXT의 별도 속도와 238 MB 기반은 사용하지 않는다.

## 3. 모든 DP에 같은 QA 정의와 산식

| QA | 고정된 의미와 집계 | v4의 근거 수준 |
| --- | --- | --- |
| QA01 첫 응답 지연 | 입력 종료→첫 구체적 채택 내용의 화면 표시, 24입력 산술평균. 음성 입력은 발화 종료, 텍스트는 제출 시각을 0으로 사용 | 정상 내용 채택을 가정한 시간 계산. 내용 정확성과 물리적 표시를 관측하지 않음 |
| QA02 지칭 대상 정확도 | 지칭 표현별 최초 채택 대상 집합의 완전 일치, 33표현 micro 집계 | 33개 정답 준비, 후보 출력 없음→`null` |
| QA03 VIA 처리 시간 | 등록 목표의 원래 시작 이후 인과 DAG에서 외부 업무·사용자 대기·순수 음성 재생 duration을 0으로 투영한 최종 시간, 24목표 평균 | 공통 목표의 조건부 계산. 단순 wall-clock 차감이나 전체 node 시간 합산 아님 |
| QA04 변경 범위 | 동일 C01~C06의 중복 없는 모듈 집합 개수를 과제별로 계산한 평균 | `concept::` 모듈의 설계 추정. 실제 Rust diff 아님 |
| QA05 모델 비용 | 24사례의 준비·요청·후속·미사용·취소 활동 전부에 사용량×단가를 적용한 합 | VIA 비용만 산정. 수신 Agent와 전체 시스템 비용은 `null` |
| QA06 RAM | 같은 시각의 중복 없는 로컬 소유 상태 합 중 최대를 전체 사례에서 선택 | RAM 모델. shared pages 한 번, private runtime과 weights/workspace/IPC/audio를 별도 계상. 원격 weights/KV 제외 |
| QA07 요청 의미 정확도 | 입력별 필수 요청 의미 전체의 최초 채택 일치율, 24입력 | 24개 정답 준비, 후보 출력 없음→`null`. 목표·점수 별도 제안과 자동 호환하지 않음 |
| 첫 음성 진단 | 같은 입력 종료→첫 유효 chunk의 장치 출력, 24입력 평균 | QA01과 구별. QA09의 listening-eligible 전달 지표를 대체하지 않음 |

일반 접수 메시지는 QA01 종료점이 아니다. 구체적 수행 의도를 먼저 표시하는 것은 허용하지만, 이 모델은 모든 구성에서 같은 64-token 의미 채택 packet을 표시한다. 먼저 표시 가능한 세부 단위를 따로 예측하지 않으므로 DP2의 가능한 모든 부분 표시 이득을 모델링하지 않는다. QA02/07의 오류가 발견되면 해당 시간 경로를 실제 성공으로 소급하지 않는다.

## 4. 시간과 자원의 계산

전체 계수는 [specification.json](specification.json)의 `profile`과 `token_budget`에 고정했다. v3 VOICE/DELIVERY의 작성 가정을 공통 기준으로 옮겼으며 실측 교정값이 아니다. 구조가 필요로 하는 입력량 차이는 토큰 예산에 공개하며 실제 tokenizer 결과가 아니다.

```text
의미 호출 시간 = 0.20 + 입력 토큰/6000 + 출력 토큰/80 (초)
음성 생성 시간 = 0.38 + 입력 토큰/6000 + 생성 audio 초/4
첫 chunk = 같은 생성 job 시작 + setup + min(audio 초, 0.20)/4
첫 장치 출력 = max(현재 채택 완료, 첫 chunk 준비) + 0.04
비용 = text input×0.000001 + text output×0.000004
     + input audio 초×0.001 + generated audio 초×0.002 (USD)
공통 RAM = UI/core 180 + ASR/audio 220 + source ring 96 + history 64 = 560 MB
```

선행 node가 끝나고 실제 자원 슬롯이 비는 가장 이른 시각에 실행한다. 과거에 같은 슬롯을 썼다는 이유만으로 인과 의존을 추가하지 않는다. QA03은 실제 자원 대기 원인을 포함하고 병렬 경로를 MAX로 결합한다. 입력 전 완료된 node는 QA03 원점에서 잘라내되 비용과 RAM 이력에는 남긴다.

DP2 A는 진행 중 현재 step을 기다리며 전체 장기 업무를 기다리는 것으로 약화하지 않는다. B도 필수 결과 의존 또는 두 슬롯 포화 시 대기한다. DP5 A도 이미 확정한 사실은 재사용한다. DP5 B의 stale 관계는 원본 fallback과 갱신 비용을 함께 계산한다.

DP3 B는 UI 후보→필요한 SAM 시간 대응→의미 결합이다. U11/U24의 후보 부족 경로는 **초기 binder의 ExpandNeed 호출→원본 확장→producer 재생산→revision 게시→최종 결합**을 전부 계산한다. 후보 부족은 이 계산 경로의 작성 조건이며 실제 오류 확률을 뜻하지 않는다. U12/U22의 사전 의미 확정도 양안 모두 유상 처리한다. R3는 CPU 예산을 늘리거나 UI/SAM 의존을 제거하지 않는다.

DP44 A는 응답별 선행 생성을 유지하고 B는 현재 발화 선택에 생성을 연결한다. 현재 유효 발화는 공통 비선점 재생 정책을 적용하며 B의 다음 생성은 선택 이후 시작한다. U19의 재개는 양안 모두 기존 유효 buffer를 보존하고 B는 미생성 후반만 생성한다. 첫 chunk 관측 node는 같은 생성 job의 prefix이며 추가 모델 호출·슬롯·비용이 아니다.

## 5. 정확도 판정 준비와 경계

[oracles.json](oracles.json)은 평가자 전용이다. 계산기는 사례 ID와 분모만 읽으며 정답 의미를 후보 출력이나 scheduler 입력으로 사용하지 않는다. 표의 QA02/07은 모든 구성이 미산출이다. DP44에도 고정 100%를 주지 않는다.

Excel 판정 시트는 검증된 최초 채택 출력을 canonical 형식으로 넣는 준비 공간이다. 대상 집합은 중복과 순서를 정규화하고 `conditions.order`는 순서를 보존한다. 필드명·ID의 대소문자와 누락/null/false/0은 구별한다. 셀의 EXACT 수식은 canonical 값 비교이며 자연어 의미 평가기나 관측 adapter가 아니다. 부분 입력만 채워지면 headline은 미산출로 유지한다. 이후 timeout/미채택을 단순 빈칸으로 남겨 분모에서 제외해서는 안 되며 실행 전에 고정 실패 처리와 전체 분모를 동결해야 한다.

기존 QA 정의·목표·점수 및 COMMON-28은 바꾸지 않는다. 새 24입력 값에 과거의 묶음 목표나 점수를 붙이지 않는다. 실제 실행에는 원본 화면/WAV, 모델·runtime 버전과 관측 adapter, 최초 채택 기록, 실패·재시도 규칙 및 동결 manifest가 추가로 필요하다.

## 6. 재현과 산출물

저장소 루트에서 분석 모델과 문서를 생성한다. 모델 서비스나 제품 프로그램은 호출하지 않는다.

```sh
.venv/bin/python scripts/architecture/build_common_qa_v4.py
.venv/bin/python scripts/architecture/check_common_qa_v4.py
node scripts/presentations/render_common_qa_v4.mjs
node scripts/presentations/check_common_qa_v4_html.mjs
```

renderer는 `@oai/artifact-tool`이 있는 Node runtime이 필요하다. `VIA_ARTIFACT_MODULE_BASE`로 해당 node_modules의 부모를 지정할 수 있다. Excel의 비용 단가 변경은 연결 수식으로 반영된다. 시간/메모리 환경 변경은 명세에서 수정하고 원장을 재생성해야 한다. 새 가정이나 실제 관측으로 발행할 때는 v4를 덮어쓰지 않고 새 판본에 명세와 결과를 함께 보존한다.
