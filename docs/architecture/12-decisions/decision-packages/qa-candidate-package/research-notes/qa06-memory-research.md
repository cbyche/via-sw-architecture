# QA06 조사 기록: 최대 RAM 사용량

> 조사 확인일: 2026-10-14 / `TARGET_EVIDENCE_UNRESOLVED` / `MEASUREMENT_METHOD_PENDING` / Windows 실측 없음
> 정의와 측정 상태의 기준은 [QA06 상세안](../qa06-local-memory.md)이다. 기존 512MiB 목표와 RAM-v1 점수는 철회했다. [조사 목록](./README.md)
> 이전 문서: [목표 철회 전 조사 기록](../../../../../archive/qa-targets-before-task-aligned-revision-20261014/research-notes/qa06-memory-research.md) / [보관 세대 설명](../../../../../archive/qa-targets-before-task-aligned-revision-20261014/README.md)

**VIA와 목적이 유사한 Windows 음성 및 화면 비서의 공식 RAM 상한이나 비교 가능한 사용 중 최고량을 확보하지 못했다. 따라서 목표 수치를 근거와 함께 제시하는 작업은 아직 완료되지 않았다.** 기존 기능별 수치로 512MiB를 유지하거나 다른 임의 숫자로 바꾸지 않는다. 같은 시점의 실제 RAM을 중복 없이 합쳐 최고량을 구한다는 지표는 유지한다.

## 1. 사용자 문제와 조사 방향

삼성 노트북에 기본 탑재되는 VIA는 다른 사용자 앱과 RAM을 나누어 사용한다. 따라서 PC 전체 RAM을 임의로 배분하기보다 **사용 중 실제 RAM 최고량을 얼마나 차지하는가**를 제품 관점에서 제한한다. 사용자에게 내부 메모리 기준이 없음을 확인했으므로 존재하지 않는 삼성 정책이나 합의를 출처로 만들지 않는다.

이전 조사는 명령 UI와 음성인식 및 음성합성의 개별 수치를 제품 상한의 참조로 사용했다. 그러나 사용자 검토에서 지적한 대로 이들은 VIA와 같은 목적의 프로그램이 아니다. 다른 프로그램의 부분 기능이 수백 MB를 사용한다는 사실만으로 VIA 전체의 512MiB 목표를 도출할 수 없다.

이번에는 **Windows에서 음성과 텍스트 및 화면 맥락을 다루는 PC 비서**를 우선 조사했다. 공식 앱 자원 상한과 유지관리자의 재현 가능한 실행 측정을 찾고 시스템 최소 RAM과 모델 파일 크기 및 사용자의 일회성 제보를 구분했다. 공개 관측값과 제품 목표 및 VIA 실측값은 서로 다르다.

## 1.1 이번 조사에서 결정한 사항

| 항목 | 결정 |
| --- | --- |
| 기존 목표 | **512MiB 철회**. 기능별 공개 수치로 전체 제품의 허용량을 정한 논리가 부족함 |
| 기존 점수 | **RAM-v1의 0~6점 구간 철회**. 근거가 없는 목표를 비율로 나누어도 타당한 채점 기준이 되지 않음 |
| 지표 | 같은 시점의 VIA 전용 페이지와 공유 페이지 및 직접 소유 버퍼를 합치고 같은 물리 페이지를 한 번만 센 실제 RAM 관측 최고량 |
| 동종 비교 제품 | **Windows Copilot Voice/Vision**. 음성 대화와 화면 공유라는 목적이 유사함 |
| 목표 상태 | `TARGET_EVIDENCE_UNRESOLVED`. 동종 제품의 비교 가능한 자원 수치나 제품 공존에 따른 정량 한도를 확보하지 못함 |
| 계측 상태 | `MEASUREMENT_METHOD_PENDING`. 공유 페이지와 그래픽 버퍼의 귀속 및 연속 관측을 Windows에서 검증하지 않음 |
| 이번 작업 범위 | 공개 자료와 측정 코드 검토. Windows 실행과 새 성능 측정은 시작하지 않음 |

## 2. 하드웨어와 기본 탑재 정책에서 확인한 내용

| 출처 | 확인한 내용 | 사용할 수 있는 근거 |
| --- | --- | --- |
| [삼성 Galaxy Book6 Ultra NP960UJH-XG3IN](https://www.samsung.com/in/computers/galaxy-book/galaxy-book6-ultra-ultra-7-32gb-1tb-np960ujh-xg3in/) | Windows 11 Home과 RAM 32GB 및 2880×1800 화면. RTX 5070 Laptop GPU의 전용 메모리 8GB | 이전에 조사한 장비의 식별과 화면 및 GPU 조건 |
| [삼성 Galaxy Book6 비교표](https://www.samsung.com/us/computers/galaxy-book/galaxy-book6/) | 16GB 구성 | 제품 공존 확인에 16GB 장비도 포함하자는 제안 |
| [삼성 Book6 라인업 확대 발표](https://news.samsung.com/us/samsung-expands-galaxy-book6-lineup-making-galaxy-book-more-accessible/) | 8GB 및 16GB 구성도 존재 | 16GB를 모든 삼성 노트북의 최소 사양이라고 표현하지 않음 |
| [Microsoft 앱 성능 계획](https://learn.microsoft.com/en-us/windows/apps/develop/performance/planning-measuring-performance) | 앱의 성능 목표를 정하고 측정하는 지침 | 제품에 맞는 명시적인 자원 목표를 정하는 접근 |
| [OEM Memory Footprint 평가 결과](https://learn.microsoft.com/en-us/windows-hardware/test/assessments/results-for-the-memory-footprint-assessment)와 [평가 개요](https://learn.microsoft.com/en-us/windows-hardware/test/assessments/memory-footprint) | 이미지에 포함된 소프트웨어의 메모리 영향 평가 | 출하 이미지와 다른 앱을 유지한 공존 평가 |

조사한 공개 공식 자료에서 **Windows 기본 탑재 프로그램 한 개에 일괄 적용되는 MB/GB 할당 상한은 확인하지 못했다.** 검색 범위 밖의 모든 내부 규정이 없다고 증명한 것은 아니다. 장비 RAM 32GB라는 사실만으로 VIA에 1GB나 4GB를 배분할 수는 없다.

## 3. 동종 제품 조사와 수치 채택 여부

### 3.1 비교 제품으로 채택한 Windows Copilot Voice/Vision

[Microsoft의 Windows Copilot 안내](https://support.microsoft.com/en-au/microsoft-365-copilot/access-microsoft-365-copilot-on-windows)는 새로운 Windows 11 PC의 기본 설치와 음성 대화 및 화면의 앱을 활용하는 Vision을 설명한다. [Vision 공식 안내](https://support.microsoft.com/en-au/microsoft-365-copilot/get-started-vision-microsoft-365-copilot)는 음성 대화 중 화면을 공유하고 해당 내용에 대해 설명과 안내를 받는 기능을 설명한다. 개인 계정도 적용 대상으로 명시되어 있어 소비자 PC의 비교 제품으로 선택했다.

두 문서에는 **앱 RAM 허용량과 사용 중 최고량 및 메모리 카운터가 제시되어 있지 않다.** 기능이 유사하다는 근거로 채택하며 존재하지 않는 공식 메모리 목표까지 부여하지 않는다. Vision 안내에는 사용자를 대신해 클릭하거나 입력하거나 스크롤하지 않는다는 범위도 명시되어 있다. VIA의 Downstream Agent 연결과 여러 Task 사건 처리를 모두 동등하게 제공한다고 가정하지 않는다.

2026-10-14 URL 확인 시 다음 두 기존 경로가 위 링크의 최종 경로로 이동했다. 최종 문서의 적용 대상과 본문을 확인했으며 경로에 `microsoft-365-copilot`이 있다는 이유만으로 기업용 서비스라고 단정하지 않는다.

| 이전 경로 | 확인한 최종 경로 |
| --- | --- |
| `microsoft-copilot/getting-started-with-copilot-on-windows` | `microsoft-365-copilot/access-microsoft-365-copilot-on-windows` |
| `microsoft-copilot/using-copilot-vision-with-microsoft-copilot` | `microsoft-365-copilot/get-started-vision-microsoft-365-copilot` |

### 3.2 그 밖의 목적 유사 제품

| 제품과 공식 자료 | 확인한 기능과 환경 | RAM 수치 및 채택 판단 |
| --- | --- | --- |
| [ChatGPT Windows 앱](https://help.openai.com/en/articles/9982051-using-the-chatgpt-windows-app) | Windows 보조 창에서 대화와 파일 활용 | 공식 요구 사항은 Windows 버전 등을 안내한다. 앱 RAM 상한과 사용 중 최고량은 찾지 못함. 이 문서만으로 모든 음성 및 화면 기능의 Windows 제공 여부를 추정하지 않음 |
| [Gemini Live in Chrome](https://support.google.com/gemini/answer/16363185?hl=en) | PC 브라우저의 페이지 맥락을 활용하는 음성 대화와 중간 발화 | 비서 기능의 RAM 상한과 사용 중 최고량을 찾지 못함. 브라우저 전체 RAM을 해당 기능의 사용량으로 대체할 수 없음 |
| [Galaxy Book의 Bixby 소개](https://news.samsung.com/global/new-galaxy-book2-pro-series-enables-work-from-anywhere-flexibility-with-peace-of-mind-security) | 삼성 노트북의 음성 기능이라는 제품 유사성 | 해당 앱의 RAM 허용량이나 최고량이 없음. 노트북 전체 RAM 사양을 Bixby나 VIA 예산으로 전환할 수 없음 |
| [Leon 저장소](https://github.com/leon-ai/leon/blob/develop/README.md)와 [설정 문서](https://docs.getleon.ai/configuration) | 개인 비서와 클라우드 음성 처리 구성 | 공식 앱 RAM 목표나 재현 가능한 통합 최고량을 찾지 못함. 문서 세대와 구성을 구분해야 함 |
| Cortana와 Alexa for PC | Windows 음성 비서라는 목적 유사성 | 조사 범위에서 공식 앱 RAM 한도나 비교 가능한 사용 중 피크를 찾지 못함. 아래 사용자 제보와 임베디드 AVS 자료는 대체 근거가 아님 |

여기서 “찾지 못했다”는 이번에 확인한 공개 문서와 저장소의 조사 결과다. 모든 비공개 제품 기준이 존재하지 않는다는 주장은 아니다. 최소 시스템 RAM과 설치 파일 크기를 앱 RAM 상한으로 해석하지 않는다.

### 3.3 공개 숫자가 있지만 목표 근거에서 제외한 비서 자료

#### voice-assistant-v2의 697.27 수치

[유지관리자 README](https://github.com/SrabanMondal/voice-assistant-v2)는 사용 중 피크 RSS를 `697.27 MB`로 제시한다. [자원 수집 코드](https://github.com/SrabanMondal/voice-assistant-v2/blob/main/eval/components/resource_sampler.py)와 [실행 코드](https://github.com/SrabanMondal/voice-assistant-v2/blob/main/eval/run_eval.py)를 직접 확인했다.

| 확인 항목 | 공개 코드와 문서에서 확인한 내용 | VIA 목표로 옮길 수 없는 이유 |
| --- | --- | --- |
| 카운터 | 부모 프로세스와 자식들의 RSS를 약 100ms마다 합산 | 공유 물리 페이지를 중복 제거하지 않음 |
| 단위 | 합계를 `1024 * 1024`로 나눔 | 표기는 MB지만 코드의 계산 단위는 MiB임 |
| 관측 구간 | 초기 준비와 대기를 제외한 입력 시작부터 재생 완료까지 | VIA의 앱 시작과 후속 처리까지 포함한 범위와 다름 |
| 기능 범위 | 공개 실행 명령에 `--disable-intent` 사용 | 별도 해석 worker가 제외된 실행이며 전체 해석 기능의 수치가 아님 |
| 모델 프로세스 | Ollama는 별도로 실행하는 전제이며 수집기는 평가 프로세스의 자식만 추적 | 외부 Ollama 서버의 메모리를 포함한 전체 로컬 비서 수치가 아님 |
| 화면과 PC 작업 | Vision과 데스크톱 자동화는 README의 향후 계획 | 화면 맥락과 업무 연결을 다루는 VIA와 기능 범위가 다름 |
| 측정 환경 | 공개 결과의 OS와 하드웨어 및 반복별 원시 기록을 확인하지 못함 | Windows에서 동일 조건으로 측정한 최고량이라고 설명할 수 없음 |

따라서 **697.27을 반올림하거나 여유분을 더해 VIA 목표로 사용하지 않는다.** 저장소와 코드는 조사 시점의 `main`이며 고정된 재현 시험이나 이번 작업의 실행 결과가 아니다.

#### Cortana 100MB 제보

[Microsoft Q&A의 Cortana 100MB 글](https://learn.microsoft.com/en-gb/answers/questions/2786126/cortana-is-disabled-but-still-using-100-mb-of-memo)은 기능을 비활성화했는데도 RAM을 사용한다는 사용자 질문이다. Microsoft 도메인에 있는 게시물이라는 이유만으로 제품팀의 공식 자원 목표가 되지 않는다. 사용 중 최고량과 전체 프로세스 범위 및 통제된 활동이 확인되지 않아 목표에서 제외한다.

#### Amazon AVS의 512MB 비교 조건

[Amazon의 AVS 자원 지침](https://developer.amazon.com/en-US/blogs/alexa/post/2a32d792-d471-4136-8262-79962a2b4d72/cpu-memory-and-storage-for-alexa-built-in-device)은 화면 없는 임베디드 음성 기기를 대상으로 하며 화면 기기를 명시적으로 제외한다. Linux의 `top`과 `/proc/meminfo`를 이용한 기기 전체의 여유 자원 확인을 설명한다. 표의 512MB는 여러 기기 결과를 비교하기 위한 정규화 조건이지 Windows 비서 앱의 허용 RAM이 아니다. 글의 20% 여유 권고도 CPU에 관한 내용이며 VIA RAM 여유율로 바꾸지 않는다.

#### Copilot의 비공식 관측 보도

[2025년 WindowsLatest 보도](https://www.windowslatest.com/2025/03/04/microsofts-copilot-just-got-a-big-upgrade-on-windows-11-more-native-than-chatgpt-now/)는 네이티브 Copilot의 50~100MB를 관측했다고 적고 [2026년 후속 보도](https://www.windowslatest.com/2026/04/29/microsoft-engineer-says-native-apps-are-back-and-it-could-finally-revive-windows-11s-fight-against-web-apps/)는 WebView Copilot의 대기 중 최대 500MB와 사용 중 약 1GB를 적는다. 이는 Microsoft 공식 상한이 아닌 기자의 관측이다. 앱 버전과 고정 활동 및 전체 프로세스 범위와 카운터가 VIA 측정 계약 수준으로 제공되지 않았다. **이 보도를 1GiB 목표나 그 절반인 512MiB의 근거로 채택하지 않는다.** 두 관측을 평균내지도 않는다.

## 3.4 이전에 사용한 부분 기능 자료의 보존과 제외

### Microsoft PowerToys Command Palette

[공식 저장소 PR #40551](https://github.com/microsoft/PowerToys/pull/40551)은 Native AOT 적용 전후의 시작 성능과 메모리를 비교한다. 이전 근거에 사용했던 수치는 **AOT 구성의 시작 구간**이다.

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

명령 실행 UI이며 VIA와 같은 음성 및 화면 대화 비서가 아니다. 타이핑과 확장 기능 사용은 시험에 포함되지 않았다. 사용한 메모리 카운터 종류도 PR에 명시되지 않았고 연결된 상세 계측 브랜치는 조회되지 않았다. **목적과 활동 및 카운터가 달라 VIA의 목표 수치 근거에서 제외한다.** [Microsoft 릴리스 설명](https://devblogs.microsoft.com/commandline/powertoys-0-93-is-here-faster-command-palette-new-dashboard-ux-and-more/)에서 개선을 소개한 사실과 OEM 기본 탑재 앱의 의무 한도는 별개다.

### Vosk 음성인식

[Vosk 공식 모델 안내](https://alphacephei.com/vosk/models)는 작은 모델의 실행 메모리를 약 `300Mb`라고 적는다. 데스크톱에도 작은 모델 사용을 권한다. 원문의 대소문자가 엄밀하지 않으므로 문서에서는 **약 300MB 규모라는 공급자 안내**로 사용하고 정확한 MiB 관측치로 바꾸지 않는다.

이는 음성인식 한 기능의 안내이며 VIA처럼 화면을 이해하고 응답과 업무를 연결하는 프로그램이 아니다. 특정 Windows 장비의 전체 최고량이나 카운터도 제공하지 않는다. **VIA의 목표 수치 근거에서 제외한다.** 모델 파일 크기와 실행 메모리를 구분하며 VIA가 Vosk를 선정했거나 음성인식을 로컬에서 추가 실행한다는 뜻도 아니다.

### Picovoice Orca 음성 합성

[공식 TTS 벤치마크](https://picovoice.ai/docs/benchmark/tts/)에는 Orca의 최고 RAM **41MB**가 제시된다. 장비는 Ryzen 7 5700X와 RAM 64GB이며 LLM 추론과 Python 준비 비용은 제외한다. 음성 합성 한 기능의 수치이므로 **VIA의 목표 수치 근거에서 제외한다.** 공급자 측정의 출처 기록은 보존하되 삼성 Windows 통합 제품의 수치로 일반화하지 않는다.

### whisper.cpp 보조 자료

[whisper.cpp 메모리 표](https://github.com/ggml-org/whisper.cpp#memory-usage)는 tiny 약 **273MB**와 base 약 **388MB**의 메모리 사용을 안내한다. 모델 크기와 실행 구성이 메모리를 크게 바꿀 수 있음을 보여주는 보조 자료다. VIA의 선정 모델이나 정량 목표 산식의 항으로 사용하지 않는다.

### 기존 512MiB 논리를 철회하는 이유

171과 300 및 41을 더하면 숫자로는 512가 나오지만 카운터와 장비 및 단위와 작업 구간이 같지 않고 동일 시점의 통합 시험도 아니다. 이를 산식으로 사용할 수 없다. 합산하지 않고 “수백 MB 규모를 참고해 0.5GiB를 선택했다”고 표현해도 **왜 VIA 전체가 그 수치를 만족해야 하는지와 만족할 수 있는지를 설명하지 못한다.** 따라서 기존 512MiB와 그에 종속된 RAM-v1 점수 전체를 철회한다.

## 4. 무엇을 측정할지 결정한 Windows 자료

| 개념과 출처 | 확인한 정의 | QA06에서의 처리 |
| --- | --- | --- |
| [Working Set](https://learn.microsoft.com/en-us/windows/win32/memory/working-set) | 현재 물리 RAM에 상주하는 프로세스 페이지 | 주지표의 출발점. 공유 중복과 직접 소유 버퍼 확인 필요 |
| [PROCESS_MEMORY_COUNTERS_EX](https://learn.microsoft.com/en-us/windows/win32/api/psapi/ns-psapi-process_memory_counters_ex) | working set과 peak 및 private 사용량 등 서로 다른 카운터 | 확보량과 상주량 및 개별 프로세스 최고량을 혼용하지 않음 |
| [Job Object 메모리 제한](https://learn.microsoft.com/en-us/windows/win32/api/winnt/ns-winnt-jobobject_extended_limit_information) | 프로세스와 Job의 commit 관련 제한 | 실제 RAM 최고량 제한을 그대로 강제하는 기능이라고 설명하지 않음 |
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

`TARGET_EVIDENCE_UNRESOLVED`와 `MEASUREMENT_METHOD_PENDING`은 별개의 미완료 항목이다. **목표 숫자와 근거를 확보하지 못했고 정의대로 관측할 계측 방법도 아직 검증하지 않았다.** 향후 공유 페이지 식별과 버퍼 귀속 및 표본 간 누락과 계측 오버헤드를 Windows에서 확인해야 한다. 전용 영역만 관측했다면 그 부분의 수치로 표시하고 전체 RAM 성공으로 채점하지 않는다.

## 6. 사용 중 최고량을 확인할 활동과 조건

현재 후보는 큰 음성과 의미 모델을 클라우드에서 실행하는 조건이다. VIA의 로컬 UI와 보조 프로세스 및 화면과 음성 자료와 요청 및 Task 상태는 모두 포함한다. 로컬에 실제 모델을 실행하면 해당 가중치와 실행 상태도 포함한다. 기존 참조 Architecture의 모델 배치를 이 문서로 변경하지 않는다.

“표준 활동”이라는 표현만 사용하지 않고 [20요청 활동 명세](../user-activity-20.md)의 요청과 사건을 명시한다.

| 시험 요소 | 조건 제안 |
| --- | --- |
| 사용자 요청 20건 | 화면 지칭 6건 / 과거 조회 6건 / 업무 시작과 제어 4건 / 직접 질의 4건 |
| 정정 | 위 20건 안에 정정 4건 포함. 별도 4건을 더해 24건으로 세지 않음 |
| Agent 사건 6건 | 질문 2건 / 진행 2건 / 결과 2건. 사용자 요청 수와 별도로 기록 |
| 동시 활동 | 정정 발화와 다른 Task의 Agent 사건이 겹치는 구간을 포함. 사건별 순서와 시각은 활동 명세를 따름 |
| 반복과 최고량 | 같은 초기 상태의 활동 5회. 전체 관측 시점과 5회 결과 중 최고량 사용 |
| 관측 구간 | 앱 시작과 초기 준비부터 요청 처리와 후속 처리 종료까지. 대기 상태나 동시 사건 구간만 골라 측정하지 않음 |
| 초기 상태 | 대화 200턴 / 완료 업무 20개 / 진행 업무 5개 |
| 추가 입력 조건 | 2880×1800 화면과 최대 15개 보존 대상 화면 / 입력과 출력 각각 48kHz 16bit mono 최대 60초 분량 |

20건의 구성은 실제 소비자의 이용 빈도를 조사한 결과가 아니라 비교를 위한 활동 제안이다. 이벤트 시각과 화면 및 음성 자료가 실행 자료로 고정되고 계측 방법이 검증되기 전에는 측정 결과를 생성하지 않는다. 압축과 crop 및 공유 참조와 디스크 보존은 양안 모두 허용한다. 원본 근거를 없애거나 강제로 페이지를 밀어내 낮춘 값은 적합한 최적화가 아니다.

실제 제품 확인에서는 대상 SKU와 출하 이미지 및 브라우저와 문서 작업 부하를 고정한다. 16GB 장비도 포함하자는 제안과 8GB 지원 여부는 제품 범위 판단으로 남아 있다. QA01과 QA03의 지연 및 페이지 읽기를 함께 보고 RAM 절감 때문에 사용성이 떨어졌는지 확인한다. 이를 하나의 가중 점수로 합치지는 않는다.

## 7. 채택하지 않은 근거

| 이전 논리 | 사용하지 않는 이유 |
| --- | --- |
| 32GiB = 8 + 16 + 4 + 4의 배분 | OS와 다른 앱 및 VIA의 실제 제품 근거 없이 총량을 나눈 것 |
| 확보량 1GiB 이하 | 사용자가 요구한 실제 동작 중 RAM 최고량과 다른 지표 |
| Windows 기본 탑재 앱은 보통 N MB까지 허용 | 조사한 공개 자료에서 공통 강제 기준 확인 못함 |
| UI와 음성 수치를 더해 VIA가 512MiB에 들어간다고 설명 | 서로 다른 환경과 카운터의 수치이며 통합 기능 실측이 아님 |
| 수백 MB인 부분 기능을 참고했으므로 VIA 전체도 512MiB | 프로그램 목적과 전체 기능 및 제품 공존의 허용 영향을 설명하지 못함 |
| voice-assistant-v2의 697.27에 여유를 더해 새 상한 지정 | RSS 중복과 외부 모델 누락 및 기능 제외와 환경 정보 부족 |
| Copilot 비공식 관측 약 1GB의 절반을 목표로 지정 | 통제된 전체 RAM 측정이 아니며 절반이라는 비율의 제품 근거도 없음 |
| AVS의 512MB를 Windows 음성 비서 한도로 사용 | 화면 없는 임베디드 기기의 정규화 비교 조건임 |
| 공급자 표의 MB를 정확한 MiB로 환산 | 원문 단위와 카운터의 정밀도가 확인되지 않음 |
| Job Object 한도로 RAM 요구 구현 완료 | commit 제한과 실제 RAM 사용량은 다름 |
| 여러 프로세스의 개별 최고값을 더함 | 같은 시점의 전체 최고량을 측정하지 않음 |

## 8. 발표 페이지와 예상 질문에 사용할 내용

- **정의:** “같은 사용자 활동을 처리하는 동안 VIA 전체가 실제 RAM에서 사용하는 공간의 최고량”
- **목표 상태:** “기존 512MiB 철회. 동종 제품의 비교 가능한 수치와 제품 공존 근거 미확보”
- **측정 조건:** “Windows 11에서 화면 지칭 6건과 과거 조회 6건 및 업무 제어 4건과 직접 질의 4건을 수행하며 Agent 사건 6건이 겹치는 활동을 5회 반복”
- **비교 제품:** “Windows Copilot Voice/Vision. 음성과 화면이라는 목적은 유사하나 공식 RAM 상한이나 통제된 최고량은 확보하지 못함”
- **측정 예시:** 같은 시점 전체 380MiB → 다음 시점 전체 400MiB라면 관측 최고량은 400MiB. 설명용 예시이며 제품 실측값이 아니다. 철회한 RAM-v1로 채점하지 않음.

| 심사 질문 | 답변의 핵심 |
| --- | --- |
| 왜 이전 512MiB를 철회했는가 | 목적이 다른 부분 기능의 사용량으로 VIA 전체의 허용량을 정한 논리가 부족하기 때문이다 |
| 그럼 지금 목표 수치는 무엇인가 | 근거를 갖춘 수치를 아직 확보하지 못했다. 이 항목은 완료되지 않았으며 새 숫자나 점수를 만들어 보완하지 않는다 |
| 무엇과 비교해야 하는가 | Windows에서 음성과 화면을 함께 다루는 Copilot Voice/Vision을 우선 비교 제품으로 정했다. 공식 기능 안내는 있으나 자원 상한은 찾지 못했다 |
| 비공식 Copilot 1GB 보도를 쓰면 되지 않는가 | 앱 버전과 활동 및 카운터가 통제되지 않아 전체 최고량의 목표 기준으로 쓰기 어렵다 |
| 모델도 RAM을 사용하는가 | 로컬 모델이면 가중치와 실행 상태를 포함해야 한다. 현재 후보의 큰 모델은 클라우드 조건이며 기존 참조 Architecture를 변경하지 않는다 |
| 지금 바로 이 정의로 측정할 수 있는가 | 전체 공유 영역과 그래픽 버퍼의 귀속 및 중복 제거를 확인해야 한다. 불완전한 값을 전체 RAM 결과로 내지 않는다 |

다음 단계에서 필요한 증거는 같은 대상 장비에서 비교 제품의 음성 대화와 화면 질의에 대한 통합 RAM 자료 또는 출하 이미지와 다른 앱의 공존을 평가해 정한 허용량이다. 공식 문서만으로 VIA와 동등한 제공 범위를 확인하지 못한 과거 근거 보존과 여러 Task 사건 처리 부담은 따로 확인해야 한다. **이번 문서 수정은 Windows 실험과 구현 및 측정 시작을 포함하지 않는다.** 목표 근거와 계측 방법이 확보되면 그 근거를 기록하고 새 목표와 새 점수 버전을 검토한다.

정확한 관측 범위와 상태는 [QA06 상세안](../qa06-local-memory.md)에 둔다. 발표 편집 시 [아홉 QA 페이지 구성안](../slide-page-plan.md)을 함께 사용한다. 조사 기록이 보완됐다는 사실과 정량 목표 도출이 완료됐다는 주장을 구분한다.
