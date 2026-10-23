# DP51 API와 원본 코드 조사

[진입점](README.md) / [설계](design.md) / [검토](review.md)

2026-10-23 정적 조사. 아래는 API 문서 및 source를 읽은 사실과 VIA 적용 설계를 구분한다. Windows 제품 실행, 장치 계측과 provider capability 검증은 수행하지 않았다. 링크의 main/docs branch는 조회 이후 바뀔 수 있으며 구현 freeze로 사용하지 않는다.

## 1. 저장소의 실제 출발점

| 자료 | 확인한 사실 | 이번 설계에 대한 의미 |
| --- | --- | --- |
| [대표 UC](../../../../05-representative-use-cases.md) | UC-03/04는 사전 선택, 클릭 없는 포인팅, 복수 지시, scroll/창 변경 및 전사 지연/수정을 포함 | 마지막 화면만 읽는 경로로 충분하지 않음 |
| [기존 관측 reader](../../../../../../prototypes/candidates/four_dp_poc/src/dp04/observations.rs) | revision의 image_path에서 PNG를 읽고 source identity를 기록 | 실제 OS 화면 수집과 버퍼 수명은 이 코드의 앞단 |
| [fixture 생성기](../../../../../../benchmark/architecture/four_dp_poc/assets/screen/build_fixtures.py) | 작성한 SVG를 PNG로 만들고 frame/event/발화 시각을 부여 | 실제 사용자 capture 및 ASR 시각을 관측한 결과가 아님 |
| [기존 grounding 실행](../../../../../../prototypes/candidates/four_dp_poc/src/dp04.rs) | 준비된 관측 입력과 revision gate로 의미 경로 수행 | 실제 OS 입력에 같은 근거가 확보된다는 증거가 아님 |
| [기존 DP3](../dp03/README.md)와 [설계](../dp03/design.md) | 역사 화면, pointer path 및 source revision을 공통 조건으로 둠 | 이번 질문은 그 공통 자료를 실제로 얻고 보관하는 구현 |
| [참조 구조](../../../target-architecture/architecture.md) 및 [기억 수명](../../../target-architecture/memory-and-context-lifecycle.md) | Interaction Manager의 bounded evidence, clock/gap와 pin/회수 책임 | 기존 요구 및 참고 계약. 새 저장 구조의 승자를 고정하지 않음 |
| [평가 환경](../../../../06-fixed-assumptions.md) | 기존 비교 환경은 MacBook Air | Windows 제품 binding의 실행 증거로 전용하지 않음 |
| [현재 QA](../qa/README.md) 및 [Galaxy Book 요구 출처](../qa/targets-source-qa08-platform.md) | 전체 RAM 및 추가 모델 메모리의 범위를 별도로 기록 | GPU/CPU 버퍼와 로컬 모델 비용을 임의로 제외하지 않음 |

이번 조사에서 prototypes/candidates와 benchmark/architecture의 rs/py/cpp/swift/toml을 대상으로 WASAPI, IAudioCaptureClient, Direct3D11CaptureFrame, IDXGIOutputDuplication, SetWinEventHook, WH_MOUSE_LL, CGDisplayStream, ScreenCaptureKit 사용을 검색했으며 해당 이름의 연결 구현을 찾지 못했다. 이는 검색한 active source 범위의 사실이며 모든 외부 의존성에 capture 기능이 없다는 증명은 아니다.

## 2. 음성 시각과 OS 사건

| 출처 | 확인한 API 사실 | VIA에 적용한 설계 |
| --- | --- | --- |
| [IAudioCaptureClient::GetBuffer](https://learn.microsoft.com/en-us/windows/win32/api/audioclient/nf-audioclient-iaudiocaptureclient-getbuffer) | 패킷 첫 audio frame의 device 위치 및 100ns로 변환된 QPC 값을 제공. timestamp 오류/불연속 플래그가 있음 | 원음 sample을 시간의 기준으로 사용. ASR 수신 시각과 구분하고 오류를 evidence에 남김 |
| [Microsoft WASAPI capture source](https://github.com/microsoft/Windows-universal-samples/blob/main/Samples/WindowsAudioSession/cpp/WASAPICapture.cpp) | event 기반 입력 준비와 capture buffer 읽기 경로를 확인 | 캡처 경로와 후속 모델 소비를 분리할 구체적 출발점. VIA 실행 성능의 증거는 아님 |
| [SetWinEventHook](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwineventhook) | hook 설치 thread의 message loop가 필요. out-of-context 전달과 callback 재진입을 고려해야 함 | callback은 작은 record만 기록. 느린 source 조회를 다른 worker로 이동 |
| [MSLLHOOKSTRUCT](https://learn.microsoft.com/en-us/windows/win32/api/winuser/ns-winuser-msllhookstruct) | pointer 좌표, 메시지 시각, button/wheel 관련 정보를 제공 | 작은 pointer record의 원천. 화면 frame 시각과 같은 값으로 가정하지 않음 |
| [GetMouseMovePointsEx](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getmousemovepointsex) | 최대 64개 이전 좌표와 시각의 이력을 조회. 다중 모니터 좌표 처리 주의 필요 | OS의 제한된 history를 장시간 VIA 이력의 대체로 가정하지 않음 |

OS hook 방식의 pointer 수집률, 실제 callback 지연, injected event 처리와 다른 앱/권한 조합은 미확인이다. 최종 pointer binding을 선택했다는 뜻이 아니다. 시각 매핑의 오차와 손실은 실제 backend에서 검증해야 한다.

## 3. 화면 캡처와 소유권

| 출처 | 확인한 API 또는 source 사실 | VIA에 적용한 설계 |
| --- | --- | --- |
| [Direct3D11CaptureFrame](https://learn.microsoft.com/en-us/uwp/api/windows.graphics.capture.direct3d11captureframe?view=winrt-26100) | SystemRelativeTime은 compositor 렌더 시점의 QPC 시각. ContentSize와 Surface 제공 | image 시각, 수신 시각과 복사 완료 시각을 따로 기록 |
| [Microsoft 화면 캡처 지침 원문](https://github.com/MicrosoftDocs/windows-dev-docs/blob/docs/hub/apps/develop/media-authoring-processing/screen-capture.md) | TryGetNextFrame으로 checkout한 frame은 frame 수명/Dispose에 따라 pool로 돌아감. callback의 무거운 UI 작업을 피하도록 안내 | OS frame을 장시간 분석기에 넘기지 않고 유한한 copy job으로 연결 |
| [Win32CaptureSample 원본](https://github.com/robmikh/Win32CaptureSample/blob/main/Win32CaptureSample/SimpleCapture.cpp) | CreateFreeThreaded, FrameArrived, surface 취득 및 CopyResource/영역 복사, 크기 변경 처리 경로 확인 | callback의 복사 경로와 명시적 size 처리 참고. VIA의 장기 보관/lease/예산 구현은 별도 |
| [Desktop Duplication API](https://learn.microsoft.com/en-us/windows/win32/direct3ddxgi/desktop-dup-api) | 전체 surface와 dirty/move rect를 제공. 복원 시 move를 먼저 적용한 뒤 dirty를 적용 | 기준/변경분 보관의 실제 데이터 원천. 이전 image 의존성과 순서가 필요 |
| [DXGI_OUTDUPL_FRAME_INFO](https://learn.microsoft.com/en-us/windows/win32/api/dxgi1_2/ns-dxgi1_2-dxgi_outdupl_frame_info) | 화면 및 mouse 갱신 시각 분리, 누적 frame 수와 보호 콘텐츠 표시. pointer-only 갱신 시 화면 timestamp가 0일 수 있음 | pointer 사건마다 새 image를 만들지 않음. 누적 갱신에서 중간 화면 coverage를 보장하지 않음 |
| [Microsoft Desktop Duplication 원본](https://raw.githubusercontent.com/microsoft/Windows-classic-samples/main/Samples/DXGIDesktopDuplication/cpp/DuplicationManager.cpp) | AcquireNextFrame와 metadata 취득, output별 mouse 갱신 및 capture 오류 처리 확인 | display별 identity와 오류/재생성 경계의 출발점. bounded 이력 저장은 VIA의 추가 설계 |

Win32CaptureSample은 Microsoft Learn API 문서와 함께 확인한 공개 참고 구현이다. Microsoft의 Desktop Duplication 및 WASAPI 샘플과 저자/저장소가 다르다. 샘플의 frame pool 크기를 VIA의 확정 값으로 복사하지 않는다.

Desktop Duplication의 pointer 위치는 pointer shape의 좌상단으로 표현될 수 있으므로 hotspot 및 display offset 처리 없이 일반 pointer 위치와 합치지 않는다. WGC의 새 dirty 기능 지원은 OS 빌드별로 확인해야 한다. 두 API의 시각, resource lifetime와 metadata 계약을 동일하다고 가정하지 않는다.

화면 렌더 시각을 읽었다는 것은 모니터의 실제 광학 출력 시각을 계측한 것이 아니다. protected content, lock, 앱 종료와 device loss로 읽지 못한 영역은 실제 gap 또는 unsupported로 남겨야 한다.

## 4. 선택과 UI Automation

[Microsoft UI Automation threading 지침](https://learn.microsoft.com/en-us/windows/win32/winauto/uiauto-threading)은 UI를 소유하지 않는 별도 MTA thread에서의 호출과 event 구독/해제 수명에 관한 제약을 설명한다. COM apartment의 수명 때문에 다른 thread로 전달한 객체가 무효화될 수 있는 조건도 있다.

VIA 적용 제안은 다음과 같다.

1. 원본 COM pointer를 역사 선택의 영구 식별자로 저장하지 않는다.
2. 필요한 text/range/rect 값을 복사한 DTO와 실제 조회 구간을 기록한다.
3. 선택 이벤트의 timestamp와 조회 결과를 같은 원자 snapshot이라고 부르지 않는다.
4. UIA가 멈추어도 음성과 픽셀 수집을 멈추지 않는다.

해당 PDF 앱이 TextPattern/SelectionPattern과 필요한 내용/범위를 실제 제공하는지는 확인하지 않았다. API에 인터페이스가 있다는 이유로 모든 PDF, 그래프 및 이미지의 객체 ID를 얻는다고 가정하지 않는다.

## 5. 조사 사실, 설계와 예상의 구분

| 종류 | 이번 자료에 포함한 것 | 포함하지 않은 주장 |
| --- | --- | --- |
| 조사 사실 | 문서의 timestamp/lifetime/threading 계약 및 공개 source 호출 경로 | VIA 기기에서 동일하게 정상 동작했다는 주장 |
| VIA 설계 | FrameMeta, Epoch, lease, byte admission, loss latch, delta dependency | 이미 제품에 구현된 API라는 주장 |
| 분석 예상 | 독립 frame의 임의 접근 이점, delta의 중복 픽셀 절감 가능성과 복원 비용 | ms, 성공률, 압축률, QA 점수와 측정 우열 |
| 산술 예시 | 특정 해상도 BGRA bytes 계산 | 실제 전체 RAM 또는 GPU 사용 최고량 |

후속 구현 시에는 실제 참조 commit, OS/SDK/driver와 픽셀 형식, clock mapping 및 사용한 API capability를 고정해야 한다. 이번 문서 조사는 그 동결을 수행하지 않는다.
