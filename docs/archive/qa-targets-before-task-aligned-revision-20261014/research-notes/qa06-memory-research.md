# QA06 조사 기록: 최대 RAM 사용량

> 조사 확인일: 2026-10-14 / 기본 탑재 제품의 자원 목표 검토 / Windows 실측 없음
> 정의와 측정 및 점수의 기준은 [QA06 상세안](../qa06-local-memory.md)이다. 제품 상한 512MiB와 전체 RAM 계측 미확정 상태를 그대로 유지한다. [조사 목록](./README.md)

## 1. 사용자 문제와 조사 방향

삼성 노트북에 기본 탑재되는 VIA는 다른 사용자 앱과 RAM을 나누어 사용한다. 따라서 PC 전체 RAM을 임의로 배분하기보다 **사용 중 실제 RAM 최고량을 얼마나 차지하는가**를 제품 관점에서 제한한다. 사용자에게 내부 메모리 기준이 없음을 확인했으므로 존재하지 않는 삼성 정책이나 합의를 출처로 만들지 않는다.

조사는 세 가지를 구분했다. Windows가 무엇을 RAM으로 세는지 확인하고 실제 보조 기능의 공개 자원 수치를 찾은 다음 그 규모를 참고해 제품 상한을 제안했다. **공개 사례의 관측값과 VIA의 제품 목표 및 VIA 실측값은 서로 다르다.**

## 2. 하드웨어와 기본 탑재 정책에서 확인한 내용

| 출처 | 확인한 내용 | 사용할 수 있는 근거 |
| --- | --- | --- |
| [삼성 Galaxy Book6 Ultra NP960UJH-XG3IN](https://www.samsung.com/in/computers/galaxy-book/galaxy-book6-ultra-ultra-7-32gb-1tb-np960ujh-xg3in/) | Windows 11 Home과 RAM 32GB 및 2880×1800 화면. RTX 5070 Laptop GPU의 전용 메모리 8GB | 이전에 조사한 장비의 식별과 화면 및 GPU 조건 |
| [삼성 Galaxy Book6 비교표](https://www.samsung.com/us/computers/galaxy-book/galaxy-book6/) | 16GB 구성 | 제품 공존 확인에 16GB 장비도 포함하자는 제안 |
| [삼성 Book6 라인업 확대 발표](https://news.samsung.com/us/samsung-expands-galaxy-book6-lineup-making-galaxy-book-more-accessible/) | 8GB 및 16GB 구성도 존재 | 16GB를 모든 삼성 노트북의 최소 사양이라고 표현하지 않음 |
| [Microsoft 앱 성능 계획](https://learn.microsoft.com/en-us/windows/apps/develop/performance/planning-measuring-performance) | 앱의 성능 목표를 정하고 측정하는 지침 | 제품에 맞는 명시적인 자원 목표를 정하는 접근 |
| [OEM Memory Footprint 평가 결과](https://learn.microsoft.com/en-us/windows-hardware/test/assessments/results-for-the-memory-footprint-assessment)와 [평가 개요](https://learn.microsoft.com/en-us/windows-hardware/test/assessments/memory-footprint) | 이미지에 포함된 소프트웨어의 메모리 영향 평가 | 출하 이미지와 다른 앱을 유지한 공존 평가 |

조사한 공개 공식 자료에서 **Windows 기본 탑재 프로그램 한 개에 일괄 적용되는 MB/GB 할당 상한은 확인하지 못했다.** 검색 범위 밖의 모든 내부 규정이 없다고 증명한 것은 아니다. 장비 RAM 32GB라는 사실만으로 VIA에 1GB나 4GB를 배분할 수는 없다.

## 3. 실제 기능 사례의 수치와 조건

### 3.1 Microsoft PowerToys Command Palette

[공식 저장소 PR #40551](https://github.com/microsoft/PowerToys/pull/40551)은 Native AOT 적용 전후의 시작 성능과 메모리를 비교한다. 이번에 사용하는 수치는 **AOT 구성의 시작 구간**이다.

| 관측 | 공개 값 |
| --- | ---: |
| 시작 피크 1회 | 163.5MB |
| 시작 피크 2회 | 140.6MB |
| 시작 피크 3회 | 170.6MB |
| 시작 피크 4회 | 171.0MB |
| 시작 피크 5회 | 157.1MB |
| 시작 피크의 평균 | **160.56MB** |
| 5회 중 최고 | **171.0MB** |
| 안정 상태 평균 | **95.10MB** |

타이핑과 확장 기능 사용은 시험에 포함되지 않았다. 사용한 메모리 카운터 종류도 PR에 명시되지 않았고 연결된 상세 계측 브랜치는 조회되지 않았다. 그러므로 171MB를 VIA와 동일한 물리 RAM 카운터의 전체 기능 최고량이라고 쓰지 않는다. [Microsoft 릴리스 설명](https://devblogs.microsoft.com/commandline/powertoys-0-93-is-here-faster-command-palette-new-dashboard-ux-and-more/)에서 개선을 소개한 사실과 OEM 기본 탑재 앱의 의무 한도는 별개다.

### 3.2 Vosk 음성인식

[Vosk 공식 모델 안내](https://alphacephei.com/vosk/models)는 작은 모델의 실행 메모리를 약 `300Mb`라고 적는다. 데스크톱에도 작은 모델 사용을 권한다. 원문의 대소문자가 엄밀하지 않으므로 문서에서는 **약 300MB 규모라는 공급자 안내**로 사용하고 정확한 MiB 관측치로 바꾸지 않는다.

이 수치는 특정 Windows 장비에서 측정한 VIA 전체 프로세스의 최고량이 아니다. 모델 파일 크기와 실행 메모리도 구분해야 한다. VIA가 Vosk를 선정했거나 음성인식을 로컬에서 추가 실행한다는 뜻이 아니다.

### 3.3 Picovoice Orca 음성 합성

[공식 TTS 벤치마크](https://picovoice.ai/docs/benchmark/tts/)에는 Orca의 최고 RAM **41MB**가 제시된다. 장비는 Ryzen 7 5700X와 RAM 64GB이며 LLM 추론과 Python 준비 비용은 제외한다. 공급자가 측정한 음성 합성 기능의 공개 사례로 보존하며 Samsung Windows 통합 제품의 수치로 일반화하지 않는다.

### 3.4 whisper.cpp 보조 비교 자료

[whisper.cpp 메모리 표](https://github.com/ggml-org/whisper.cpp#memory-usage)는 tiny 약 **273MB**와 base 약 **388MB**의 메모리 사용을 안내한다. 모델 크기와 실행 구성이 메모리를 크게 바꿀 수 있음을 보여주는 보조 자료다. VIA의 선정 모델이나 정량 목표 산식의 항으로 사용하지 않는다.

### 세 사례를 더하지 않는 이유

171과 300 및 41을 더하면 숫자로는 512가 나오지만 **그 합을 목표의 산식으로 사용하지 않는다.** 카운터와 장비 및 단위와 작업 구간이 같지 않고 동일 시점에 실행한 통합 시험도 아니다. 각 사례는 수십~수백 MB 규모의 보조 기능이 존재한다는 참조점이다. `0.5GiB = 512MiB`는 이 규모를 참고해 선택한 제품 상한이며 세 프로그램의 합계 실측이 아니다.

## 4. 무엇을 측정할지 결정한 Windows 자료

| 개념과 출처 | 확인한 정의 | QA06에서의 처리 |
| --- | --- | --- |
| [Working Set](https://learn.microsoft.com/en-us/windows/win32/memory/working-set) | 현재 물리 RAM에 상주하는 프로세스 페이지 | 주지표의 출발점. 공유 중복과 직접 소유 버퍼 확인 필요 |
| [PROCESS_MEMORY_COUNTERS_EX](https://learn.microsoft.com/en-us/windows/win32/api/psapi/ns-psapi-process_memory_counters_ex) | working set과 peak 및 private 사용량 등 서로 다른 카운터 | 확보량과 상주량 및 개별 프로세스 최고량을 혼용하지 않음 |
| [Job Object 메모리 제한](https://learn.microsoft.com/en-us/windows/win32/api/winnt/ns-winnt-jobobject_extended_limit_information) | 프로세스와 Job의 commit 관련 제한 | 512MiB 실제 RAM 요구를 그대로 강제하는 기능이라고 설명하지 않음 |
| [GPU 메모리의 Task Manager 설명](https://devblogs.microsoft.com/directx/gpus-in-the-task-manager/) | 전용 GPU 메모리와 공유 시스템 메모리 구분 및 프로세스 귀속의 공유 특성 | 시스템 RAM을 쓰는 VIA 소유 그래픽 영역은 포함. 전용 VRAM은 별도 보고 |

`commit`은 Windows가 저장 공간 제공을 약속한 양이며 일부는 페이지 파일에 있을 수 있다. `reserve`는 주소 범위를 예약하는 것이다. 실제 RAM 300MiB와 확보량 800MiB가 동시에 가능하므로 같은 지표로 취급하지 않는다. 이 구분 때문에 이전 확보량 1GiB 목표를 현재 RAM 목표로 단위만 바꾸어 재사용할 수 없다.

전체 VIA의 실제 RAM은 **같은 시점에 VIA가 사용하는 전용 페이지와 공유 페이지 및 직접 소유 버퍼를 합치고 같은 물리 페이지를 한 번만 센 값**이다. 다른 앱도 사용하는 공통 페이지를 포함하므로 VIA 제거 전후의 RAM 차이와도 다르다. OS 공통 서비스의 간접 비용은 설치 전후 공존 진단에서 확인한다.

## 5. 계측 수단을 조사하며 확인한 제약

| 방법 | 놓치거나 잘못 더할 수 있는 부분 |
| --- | --- |
| Private working set의 합 | 공유 영역과 일부 그래픽 버퍼 누락 |
| 전체 working set의 합 | 같은 공유 페이지 중복 |
| 프로세스별 peak의 합 | 서로 다른 시점의 최고값을 합쳐 실제 동시 최고량보다 크게 나옴 |
| WPR/WPA Resident Set | trace 종료 시점의 스냅샷이며 한 trace의 연속 여러 스냅샷이 아님. 공유 가능한 페이지는 Unknown으로 분류될 수 있어 PID만으로 귀속 불가 |
| WPR/WPA Reference Set | 시나리오 동안 접근한 메모리의 영향으로 특정 순간의 실제 RAM 최고량과 다름 |

근거는 [Microsoft Memory Footprint Exercise 1](https://learn.microsoft.com/en-us/windows-hardware/test/wpt/memory-footprint-optimization-exercise-1)과 [WPA Reference Set](https://learn.microsoft.com/en-us/windows-hardware/test/wpt/wpa-reference-set)이다. **WPR만 켜면 중복 제거된 전체 VIA RAM의 연속 최고값을 얻는다고 확정하지 않는다.**

현재 `MEASUREMENT_METHOD_PENDING`은 목표 수치가 없다는 뜻이 아니다. **제품 목표 512MiB는 제안했고 이를 정의대로 관측할 방법을 검증하지 않았다는 뜻**이다. 향후 공유 페이지 식별과 버퍼 귀속 및 표본 간 누락과 계측 오버헤드를 Windows에서 확인해야 한다. 전용 영역만 관측했다면 그 부분의 수치로 표시하고 전체 RAM 성공으로 채점하지 않는다.

## 6. 제품 상한을 적용하는 조건

현재 후보는 큰 음성과 의미 모델을 클라우드에서 실행하는 조건이다. 그래서 VIA의 로컬 UI와 화면 및 음성 자료와 요청 및 Task 상태에 **512MiB**를 요구한다. 약 10B 모델의 로컬 가중치와 실행 상태까지 같은 한도에 들어간다는 뜻이 아니며 기존 참조 Architecture의 모델 배치를 이 문서로 변경하지 않는다.

활동은 QA05와 같은 20요청을 5회 실행한다. 앱 시작과 초기 준비부터 후속 처리 종료까지 포함하며 2880×1800 화면 최대 15장과 입력 및 출력 음성을 같은 조건으로 제공한다. 압축과 crop 및 공유 참조와 디스크 보존은 양안 모두 허용한다. 원본 근거를 없애거나 강제로 페이지를 밀어내 낮춘 값은 적합한 최적화가 아니다.

실제 제품 확인에서는 대상 SKU와 출하 이미지 및 브라우저와 문서 작업 부하를 고정한다. 16GB 장비도 포함하자는 제안과 8GB 지원 여부는 제품 범위 판단으로 남아 있다. QA01과 QA03의 지연 및 페이지 읽기를 함께 보고 RAM 절감 때문에 사용성이 떨어졌는지 확인한다. 이를 하나의 가중 점수로 합치지는 않는다.

## 7. 채택하지 않은 근거

| 이전 논리 | 사용하지 않는 이유 |
| --- | --- |
| 32GiB = 8 + 16 + 4 + 4의 배분 | OS와 다른 앱 및 VIA의 실제 제품 근거 없이 총량을 나눈 것 |
| 확보량 1GiB 이하 | 사용자가 요구한 실제 동작 중 RAM 최고량과 다른 지표 |
| Windows 기본 탑재 앱은 보통 N MB까지 허용 | 조사한 공개 자료에서 공통 강제 기준 확인 못함 |
| UI와 음성 수치를 더해 VIA가 512MiB에 들어간다고 설명 | 서로 다른 환경과 카운터의 수치이며 통합 기능 실측이 아님 |
| 공급자 표의 MB를 정확한 MiB로 환산 | 원문 단위와 카운터의 정밀도가 확인되지 않음 |
| Job Object 한도로 RAM 요구 구현 완료 | commit 제한과 실제 RAM 사용량은 다름 |
| 여러 프로세스의 개별 최고값을 더함 | 같은 시점의 전체 최고량을 측정하지 않음 |

## 8. 발표 페이지와 예상 질문에 사용할 내용

- **핵심 문장:** “명령 UI와 소형 음성 기능의 공개 메모리 사례를 참고해 기본 탑재 VIA의 사용 중 RAM 상한을 0.5GiB로 제안했습니다.”
- **참조 표:** Command Palette 시작 최고 171MB / Vosk 약 300MB 규모 / Orca 최고 41MB. 각기 다른 측정 조건이며 합산하지 않음.
- **측정 도식:** 시각 t1의 전체 380MiB → t2의 전체 400MiB → 관측 최고 400MiB. 현재 점수표에서 3점이며 설명용 예시다.
- **각주:** 제품 목표 제안. 클라우드 모델 조건. 전체 RAM 계측 검증과 실제 제품 공존 평가 필요.

| 심사 질문 | 답변의 핵심 |
| --- | --- |
| 삼성이나 Windows가 512MiB로 제한하는가 | 아니다. 공개 기능 규모를 참고해 구성한 기본 탑재 제품 요구안이다 |
| 왜 정확히 512인가 | 수백 MB 규모의 보조 기능 사례를 참고해 1GiB보다 낮은 0.5GiB를 예산 단위로 선택했고 그것이 512MiB다 |
| 512MiB면 사용자 만족이 보장되는가 | 그런 임계값 연구는 제시하지 않는다. 장비와 다른 앱을 고정한 제품 공존 확인이 필요하다 |
| 세 프로그램을 더한 수치인가 | 아니다. 단위와 카운터 및 장비가 달라 합산 근거로 쓰지 않았다 |
| 모델도 RAM을 사용하는가 | 로컬 모델이면 가중치와 실행 상태를 포함해야 한다. 현재 상한은 큰 모델을 클라우드에서 실행하는 후보 조건이다 |
| 지금 바로 이 정의로 측정할 수 있는가 | 전체 공유 영역과 그래픽 버퍼의 귀속 및 중복 제거를 확인해야 한다. 불완전한 값을 전체 RAM 결과로 내지 않는다 |

정확한 관측 범위와 점수는 [QA06 §3~6](../qa06-local-memory.md#3-단일-지표와-계측-준비-상태)에 둔다. 발표 편집 시 [아홉 QA 페이지 구성안](../slide-page-plan.md)을 함께 사용한다.
