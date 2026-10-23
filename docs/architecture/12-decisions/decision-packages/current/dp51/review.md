# DP51 검토 결과와 남은 결정

[진입점](README.md) / [상황](scenarios.md) / [설계](design.md) / [근거](references.md)

2026-10-23 / 작성자 정적 검토 / 독립 리뷰 및 제품 실행 검증 아님

## 1. 이번 문서 범위의 판단

DP51은 기존 컨셉 대안을 계승하지 않고 실제 수집 및 읽기 경로에서 출발했다. 늦은 ASR가 과거 화면을 요청하는 순간, 저장 자료를 독립 image로 바로 읽을지 기준과 변경분에서 복원할지에 따라 호출, 상태 및 메모리 수명이 달라진다. 이 두 구조를 후속 비교 대상으로 유지한다.

사건 후 새 screenshot만 찍는 방식은 정적인 앱에서 비용 이익이 있을 수 있으나, 지나간 화면과 클릭 없는 포인팅을 같은 수준으로 지원하지 못한다. 현재 전체 UC의 강한 동등 대안으로 표시하지 않는다.

독립 이미지는 계약을 설명하는 첫 구현 기준이며 제품 승자가 아니다. 변경분 구조도 우월안으로 채택하지 않았다. 더 많은 자료가 남을 수 있다는 이유만으로 QA02가 자동 개선되거나 모든 RAM 조건을 만족한다고 주장하지 않는다.

이번 산출 범위인 대표 상황, 코드 수준 구조, 오류 및 자원 수명, 근거와 검토 항목의 문서화는 완료했다. 미확정 지원 범위와 수치, 구현 및 검증은 별도 상태로 남긴다.

## 2. 작성 중 점검하여 반영한 사항

| 항목 | 위험 | 반영 |
| --- | --- | --- |
| 시각 분리 | capture 요청 또는 callback 시각을 당시 픽셀 시각으로 오인 | source/received/copy complete/read interval 분리 |
| clock 매핑 | raw QPC tick과 WASAPI 100ns 값을 혼용 | adapter 정규화와 epoch/오차 기록 |
| 선택 조회 | 늦게 읽은 현재 selection을 과거 선택으로 소급 | 조회 구간을 가진 DTO, unsupported/unknown 구분 |
| 창과 좌표 | 현재 전경 창/DPI를 과거 pointer에 적용 | window observation 및 geometry 판본 분리 |
| callback 정체 | UIA, readback 또는 모델을 기다리며 입력 유실 | 작은 기록/유한 job 전달, 별도 worker |
| GPU source 수명 | 비동기 복사 종료 전에 OS frame 또는 destination 재사용 | pending job이 완료까지 자원 및 예산 소유 |
| queue overflow | 같은 가득 찬 queue로 gap을 보내 gap도 유실 | 별도 고정 크기 loss latch |
| 이력 ring | 색인에서 제거한 공간을 사용 중 소비자와 공유 | Retired와 Freed, lease와 byte quota 분리 |
| 사용권 반환 | 입력 queue 포화로 Release가 사라져 영구 pin | 발급 수 상한 및 별도 원장/예약 control 경로, 멱등 반환 |
| delta 기준 | 마지막 OS 취득과 마지막 저장 성공 기준이 다름 | base capture sequence 확인, 실패 뒤 full 재개 |
| move 복원 | 겹치는 move를 in-place 처리하여 source 훼손 | 이전 image 기준 이동 후 dirty 적용, scratch 예약 |
| delta 회수 | 기준 image만 지워 남은 frame을 복원 불가 | 묶음 회수 또는 새 기준 생성의 비용 명시 |
| 정정 | ASR 판본만 확인하고 늦은 evidence 변경 누락 | input/evidence/permission 판본 검사 |
| 취소 | 취소 신호 또는 timeout을 실제 사용 종료로 해석 | GPU/worker 종료 확인 전 quota 유지 |
| source fence | 내부 queue 처리 완료를 OS 과거 사건의 영구 종료로 오인 | 관측 범위, pending/unknown 및 view 무효화 |
| 제품 범위 | 창별 capture의 빈 구간이나 사전 선택 미수집을 숨김 | display 구독 가정 및 지원 한계 명시 |

설계 검토를 실행 결과로 표시하지 않는다. 특히 thread safety, GPU fence API binding, 정확한 COM 정리와 allocator 동작은 의사코드만으로 검증되지 않는다.

## 3. 같은 QA 의미로 설명하는 인과

[현재 QA](../qa/README.md)의 의미와 점수는 그대로 둔다.

| 관점 | 독립 이미지 보관 | 기준/변경분 보관 | 아직 주장할 수 없는 것 |
| --- | --- | --- | --- |
| QA01 첫 응답 지연 | 과거 frame 임의 읽기에 복원 dependency가 없음 | 복원 및 scratch 대기가 응답 경로에 추가될 수 있음 | source 확보와 전체 처리 경합 미관측 상태에서 ms 우열 |
| QA02 지칭 대상 식별 | 확보한 원본을 바로 제공 | 같은 원본을 무손실 복원하면 입력 정보는 같음 | 저장 형식 자체가 의미 정확도를 높인다는 주장 |
| QA03 전체 VIA 처리 | 반복 전사 정정의 읽기 비용이 상대적으로 단순 | 과거 여러 시점 재조회에서 복원 반복 가능 | 단일 read만으로 전체 목표 처리 우열 |
| QA04 실제 변경 모듈 수 | 단순한 frame read 및 회수 계약 | damage schema, 복원기, dependency/회수 코드가 추가 | 실제 운영 patch와 공통 변경 과제 없이 모듈 수 산출 |
| QA05 모델 사용 비용 | 같은 모델 입력 및 호출이면 직접 차이 없음 | 동일 | CPU/GPU 복사 절감을 모델 청구 절감으로 재명명 |
| QA06 전체 제품 RAM | 전체 frame 복사본이 누적될 수 있음 | 중복 픽셀 절감 가능, pinned 기준과 scratch 추가 | 자체 pixel budget을 전체 제품 RAM 관측값으로 제시 |
| QA07 업무 순서 및 연결 | 이 저장 구조의 직접 차별화 근거 없음 | 동일 | 지칭 오류의 간접 영향을 별도 독립 이익으로 이중 계산 |
| QA08 추가 로컬 모델 메모리 | 이번 저장 구조는 추가 모델을 요구하지 않음 | 동일 | 화면 buffer를 모델 메모리로 분류하거나 QA06과 더함 |

신뢰성은 누락 공개, 오복원, 늦은 결과와 사용 중 메모리 회수에서 비교한다. 별도 새 QA를 만들거나 기존 지표를 대체하지 않는다. 시간 및 비용은 수집, 준비, 폐기, readback, 재조회와 복원 전체를 포함해야 한다. 발화 중 처리했다고 자원 비용이 0이 되는 것은 아니다.

## 4. 유리한 조건과 반증 조건

| 구조 | 선택할 이유가 있는 조건 | 비용 및 약점 | 예상이 약해지는 조건 |
| --- | --- | --- | --- |
| 독립 이미지 | 짧은 이력, 여러 과거 시점의 잦은 읽기, 빠른 첫 접근이 중요 | 전체 복사 및 보관량 | 필요한 보관 범위를 같은 RAM 예산에서 확보하지 못함 |
| 기준/변경분 | 화면 변화가 작고 읽기 빈도가 낮으며 상대적으로 긴 이력이 필요 | 복원 지연, 기준 pin, scratch, 실패 전파와 회수 복잡성 | dirty 영역이 대부분 화면이거나 재조회가 많아 전체 비용/메모리가 절감되지 않음 |
| 사건 후 캡처만 | 정적이고 native 선택 및 조회가 신뢰 가능한 제한된 앱 | 빠른 전환, 무클릭 지시 및 전달 지연에서 근거 미확보 | 현재 UC 전체에 동등 기능을 요구하면 독립안의 자격이 약함 |

같은 관측 frame에서 독립 image와 delta 복원이 같은 픽셀을 주면 QA02의 구조적 정보 차이는 없다. 같은 byte 한도에서 남는 시간 범위와 누락이 달라질 때만 그 원인과 해당 사례를 연결한다. 더 긴 이력을 준 후보에게 무조건 더 쉬운 질문이나 더 좋은 모델을 배정하지 않는다.

## 5. 다음 검토에서 결정해야 할 것

| 결정 항목 | 필요한 이유 | 현재 처리 |
| --- | --- | --- |
| 최소 지원 OS/SDK와 PDF 앱 | capture/selection/time 계약의 실제 capability | Windows binding을 가정하고 미확인 표시 |
| 수집 활성 구간과 display 범위 | 발화 전 선택 및 다른 창 첫 화면을 확보할 수 있는가 | 이미 허용된 interaction 범위에서 미리 수집 |
| 발화 전 보관과 selection anchor 수명 | 늦게 말해도 어떤 근거가 남아야 하는가 | byte와 개수 상한만 구조화, 시간 수치 미확정 |
| 전사 지연 및 정정 지원 범위 | 유한 RAM과 무제한 과거 조회를 동시에 보장할 수 없음 | 만료/누락 계약을 명시 |
| 긴 발화에서 예산 포화 시 UX | 무제한 pin 또는 조용한 데이터 손실 방지 | 상위에 coverage 악화 통지, 제품 안내 문구/시점 미확정 |
| 실제 화면 변화량과 읽기 빈도 | full/delta의 선택 기여를 결정 | 관측 전 예상으로 유지 |
| GPU/CPU 저장 domain | readback, tensor 복사와 총 RAM의 실제 병목 | domain별 원장 요구, 확정 배치 없음 |

수치가 없다는 이유로 모든 구현 판단을 보류하지 않는다. timestamp 분리, 불변 source identity, 사용 중 자원 보호, byte admission과 누락 공개는 어느 보관 구조에도 필요한 계약으로 제안했다.

## 6. 검증 기록

2026-10-23 실행한 정적 검사다. 정적 링크/용어 검사와 실제 Windows 동작 검증을 구별한다.

| 검사 | 상태 및 한계 |
| --- | --- |
| DP51 로컬 링크, 코드 fence 및 금지 구두점 | PASS: 5개 문서, 로컬 링크 40개, 코드 fence와 구두점/whitespace 확인 |
| 저장소 active Markdown 링크 | 최초 실행 FAIL: 동시 작성 중인 DP53/DP55의 아직 없는 연결 파일. DP51 위반 없음. 타 폴더 미수정 |
| 저장소 active 용어 | PASS: check_active_terminology.py |
| current package | 최초 실행 FAIL: DP55에서 아직 없는 03-alternatives.md 연결. DP51 위반 없음 |
| 공통 QA v4~v9 보존 자료 | PASS: check_common_qa_v4.py부터 v9.py까지. 새 제품 관측이 아님 |
| whitespace 및 변경 경로 확인 | git diff --check 및 DP51 로컬 검사 PASS. Git 변경 대상은 이 폴더 5개 문서 |
| Rust compile/test, Windows runtime, 모델 호출, 실측 | 이번 문서 범위에서 수행하지 않음 |

실행은 저장소 .venv/bin/python을 사용했다. 공유 checkout의 다른 폴더가 작성 중이어서 저장소 전체 검사의 최초 결과와 이 문서 범위의 성공을 구분했다. 후속 재검사 결과는 아래에 이어 기록한다.

같은 날 후속 재검사에서는 active Markdown 링크 1,128개 문서 검사가 PASS했다. current package 검사도 PASS_WITH_RECORDED_CONTENT_SYNC_GAP으로 완료했다. 이 상태의 content sync gap은 기존 발표와 공통 QA 자료 사이의 기록된 미반영 상태이며 DP51의 링크 실패가 아니다. 다른 세션의 파일을 수정하지 않고 재검사했다. 최종 stage diff는 DP51 문서 5개만 포함하며 whitespace 검사를 통과했다.

## 7. 공용 연결과 동기화

사용자가 지정한 격리 범위에 따라 dp51 이외의 문서를 수정하지 않았다. 후속 통합 담당자는 다음 연결을 검토할 수 있다.

1. current의 공용 README에 DP51 README를 새 탐색 진입점으로 연결한다.
2. dp51~dp55를 공용 목록에 추가할 때 기존 DP2/DP3/DP5/DP44의 상태와 번호를 바꾸지 않는다.
3. package-manifest가 새 authoring package 목록을 관리해야 한다면 DP51은 신규 탐색 자료로 등록한다. 이전 이관/보존 snapshot의 hash를 바꾸지 않는다.
4. 주변 주제와 EvidenceVersion, lease, expiry/gap 및 permission epoch의 최소 접합 계약을 검토한다. 다른 세션의 state owner를 DP51 문서가 선결하지 않는다.
5. 그림/PPTX가 후속 요청되면 이 문서의 소유권 및 실행 경로를 기준으로 별도 작성하고 paired source와 preview를 검증한다. 현재 발표가 DP51을 반영했다고 표시하지 않는다.

이 연결 필요 사항은 공용 파일을 수정했다는 뜻이 아니다. Git 처리에서도 이 폴더의 완료된 문서만 포함하며 다른 세션의 staged/unstaged 변경은 보존한다.
