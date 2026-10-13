# DP-47 입력 취득 자료 — Windows에서 무엇을 얻을 수 있는가

> 상태: **공식 API에 근거한 설계 자료 / 제품 연동과 앱별 지원 범위 미검증** / 2026-10-13
> [DP-47 문제](./04-47-screen-voice-grounding.md) / [대표 장면](./dp47-grounding-scenario.md) / [현재 공통 실행 전제](./04-40-common-execution-contract.md)

## 1. 목적과 현재 전제

사용자는 음성, 화면, pointer, 선택과 앱 상태가 실제로 어떤 경로에서 들어오는지부터 구조를 탐색하도록 했다. API 이름과 취득 한계는 설계 근거이므로 보존한다. 아래 API의 존재를 VIA에 이미 구현했거나 모든 앱에서 검증한 것으로 해석하지 않는다.

현재 비교의 VIA는 PC에서 입력과 로컬 VAD를 처리하고 클라우드 음성 및 의미 모델을 사용한다. 이전 on-device reference Architecture를 바꾸는 문서가 아니다. 원음, 화면 프레임, 장치 입력과 앱 상태는 서로 다른 경로에서 얻으며 시점과 지원 범위도 다르다. 대상 의미와 정정 범위는 이 자료에서 후속 판단해야 한다.

## 2. 음성 원본과 전사

`마이크 → Windows 오디오 입력 → 로컬 오디오 packet → 음성 모델 → 전사 및 관련 이벤트`

WASAPI의 [IAudioCaptureClient::GetBuffer](https://learn.microsoft.com/en-us/windows/win32/api/audioclient/nf-audioclient-iaudiocaptureclient-getbuffer)는 capture buffer의 packet, audio frame 수, stream-relative device position, QPC 기반 시간과 discontinuity 또는 timestamp error 등의 flags를 제공한다. packet의 첫 frame 시점과 클라이언트가 읽은 시점을 구별한다. flags가 잘못된 시간이나 누락을 알리면 정상 근거로 덮지 않는다.

로컬 VAD는 발화 시작과 종료 후보를 만드는 처리다. “이 표”라는 단어의 시간은 별도 전사/정렬 계약에 달려 있다. 전사는 원음에서 생산된 파생 결과이고, partial/final/revision/span time 제공 여부는 실제 provider마다 확인한다. 모델의 결과 수신 시각을 녹음 시각으로 사용하지 않는다.

설계에 필요한 관측 항목은 원음의 stream identity, sample 범위와 형식, 제공되는 원본 시간과 유효성, 로컬 수신 시각, provider item과 전사 revision의 대응이다. 이는 정보 요구 목록이며 저장 schema와 보존 기간의 확정은 아니다.

## 3. 화면과 마우스의 취득 경로

| 정보 | 실제 Windows 경로 | 얻을 수 있는 자료 | 이 경로만으로 확정할 수 없는 것 |
| --- | --- | --- | --- |
| 디스플레이 또는 창 이미지 | [Windows.Graphics.Capture](https://learn.microsoft.com/en-us/windows/apps/develop/media-authoring-processing/screen-capture), Direct3D11CaptureFramePool의 FrameArrived와 TryGetNextFrame | 프레임 surface, ContentSize, 캡처 대상과 SystemRelativeTime | 표의 의미, 대상 identity, 사용자의 지칭 의도 |
| 프레임 시점 | [Direct3D11CaptureFrame.SystemRelativeTime](https://learn.microsoft.com/en-us/uwp/api/windows.graphics.capture.direct3d11captureframe.systemrelativetime?view=winrt-26100) | QPC에 근거한 프레임 시간 | 서로 다른 앱 조회가 이 프레임과 원자적으로 같은 상태라는 보장 |
| 마우스 장치 입력 | [Raw Input](https://learn.microsoft.com/en-us/windows/win32/inputdev/about-raw-input), RegisterRawInputDevices와 WM_INPUT / GetRawInputData | device identity와 [RAWMOUSE](https://learn.microsoft.com/en-us/windows/win32/api/winuser/ns-winuser-rawmouse)의 이동, 버튼 transition, wheel | 원시 이동량만으로 당시 화면의 정확한 pointer 좌표, 해당 동작의 의미 |
| 현재 화면 pointer 위치 | [GetCursorPos](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getcursorpos) | 조회한 시점의 screen coordinate | 이미 지나간 모든 경로, 조회 시각 이전의 좌표, 해당 위치의 의미 |
| 좌표의 창 | [WindowFromPoint](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-windowfrompoint) | 조회 좌표에 해당하는 window handle | 보이지 않거나 disabled인 창을 모두 포함하는 완전한 hit test, 창 내부의 표나 문서 identity |

Raw Input은 background 수신을 위한 등록 조건을 지원한다. VIA 창으로 전달되는 WM_MOUSEMOVE만 읽는 것으로 다른 앱의 모든 interaction을 수집했다고 가정하지 않는다. 장치의 raw delta와 화면 pointer 위치에는 좌표 변환과 시점 대응이 필요하며, 나중 GetCursorPos 조회를 각 과거 event의 정확한 좌표로 소급하지 않는다.

캡처 영역, 화면 좌표의 기준, window geometry, DPI 또는 화면 배율과 다중 monitor 조건을 함께 이해해야 한다. 화면 이미지와 UI tree, mouse input의 콜백이 같은 시간에 도착했다는 사실만으로 같은 상태를 관측했다고 단정하지 않는다. 각 source가 제공하는 시간, 로컬 관측 시각과 오차 또는 미제공을 구별한다.

버튼 down, move와 up은 장치 입력이다. 해당 연속 입력이 텍스트 선택, 객체 이동, scrollbar 조작 또는 다른 동작을 일으켰는지는 앱과 결과 상태를 확인해야 한다. up 시점에 화면이 아직 갱신 중일 수 있으므로 페이지 이동 완료를 자동 확정하지 않는다. pointer로 원을 그리는 행동은 버튼을 누르지 않아도 가능하고 앱에 선택 객체를 만들지 않을 수 있다.

## 4. 앱, 창과 focus

| 정보 | Windows 경로 | 계약의 한계 |
| --- | --- | --- |
| 열린 top-level window | [EnumWindows](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-enumwindows) | child window와 모든 실행 process를 열거하는 API가 아니다. Windows 8 이후 desktop app top-level window의 범위라는 문서 조건을 보존한다 |
| 현재 foreground window | [GetForegroundWindow](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getforegroundwindow) | 현재 foreground handle이며 전환 중 NULL일 수 있다. 모든 발화의 지칭 대상으로 자동 채택하지 않는다 |
| window의 thread와 process | [GetWindowThreadProcessId](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getwindowthreadprocessid) | window의 소속 identity다. 파일 경로나 문서 페이지를 제공하지 않는다 |
| GUI thread의 focus, active와 capture 상태 | [GetGUIThreadInfo](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getguithreadinfo) | 대상 thread의 상태 조회이며 모든 창의 동시 snapshot은 아니다 |
| foreground 또는 UI 변화 알림 | [SetWinEventHook](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwineventhook) 등 | 구독한 event의 통지다. 모든 앱 의미 변화를 완전하게 제공하거나 화면 캡처와 동시 확정하는 계약이 아니다 |

실행 중인 앱, 열린 창, foreground window, keyboard focus와 pointer 아래 창은 각각 다른 정보다. PDF가 foreground인 동안 사용자가 옆의 spreadsheet 선택 범위를 가리킬 수 있다. 발화 뒤 메일 앱으로 이동해도 이전 요청의 대상이 메일로 바뀌는 것은 아니다.

## 5. UI Automation과 선택의 지원 범위

UI Automation에서 VIA는 client이고 UI 정보를 노출하는 쪽은 provider다. [Microsoft의 provider 설명](https://learn.microsoft.com/en-us/windows/win32/winauto/uiauto-providersoverview)에 따르면 Win32, Windows Forms와 WPF의 표준 컨트롤은 기본 provider를 이용할 수 있다. 사용자 정의 컨트롤은 native provider 또는 적절한 client-side proxy가 필요하다. 적절한 provider가 없으면 HWND에서 얻는 위치 같은 기본 정보만 남을 수 있다.

따라서 앱이 직접 모든 provider를 구현해야만 쓸 수 있다는 주장도, Windows가 모든 앱의 내용을 자동으로 구조화한다는 주장도 맞지 않는다. 앱 또는 framework, 호환 proxy가 실제 어떤 요소와 pattern을 노출하는지 확인해야 한다. [표준 proxy와 사용자 정의 컨트롤](https://learn.microsoft.com/en-us/windows/win32/winauto/uiauto-clientsideprovider)

| 필요한 정보 | 취득 예 | 주의점 |
| --- | --- | --- |
| UI 요소와 구조 | [UI Automation tree](https://learn.microsoft.com/en-us/windows/win32/winauto/uiauto-treeoverview)와 지원 property/pattern 조회 | tree는 동적으로 바뀌며 모든 시각적 문서 객체가 개별 요소인 것은 아니다 |
| 선택된 텍스트 | [IUIAutomationTextPattern::GetSelection](https://learn.microsoft.com/en-us/windows/win32/api/uiautomationclient/nf-uiautomationclient-iuiautomationtextpattern-getselection) | 해당 control의 TextPattern과 selection 지원을 확인한다. caret만 있는 빈 range와 선택을 구별한다 |
| 선택된 요소 | [IUIAutomationSelectionPattern::GetCurrentSelection](https://learn.microsoft.com/en-us/windows/win32/api/uiautomationclient/nf-uiautomationclient-iuiautomationselectionpattern-getcurrentselection) | 해당 provider가 SelectionPattern으로 노출하는 요소의 범위다 |
| 과거의 선택 | 보존한 선택 자료 또는 현재 남아 있는 선택의 유효 상태 | 기록하지 않은 과거 selection을 현재 조회만으로 복원했다고 가정하지 않는다 |

Adobe는 Acrobat과 Reader의 screen reader 지원과 PDF의 접근성 기능을 설명한다. 문서의 구조와 tags, scanned page의 텍스트 제공 여부가 내용 접근에 영향을 준다. 이것만으로 특정 Reader 버전의 표, cell, 선택과 page 상태가 UI Automation에서 모두 제공된다고 보증하지 않는다. [Adobe 접근성 기능](https://helpx.adobe.com/acrobat/using/accessibility-features-pdfs.html)

DP-47 대표 장면의 scanned graph가 별도 UI 요소나 숫자 데이터로 노출되지 않는 조건은 설계 검토용 조건이다. 실제 검증 전에는 Reader 전체가 UI Automation을 지원하지 않는다고 일반화하지 않는다. 선택 조회 실패는 “선택 없음”, 미지원은 “대상이 없음”과 각각 구별한다.

## 6. 화면 이미지 크기와 비용

실제 캡처 영역이 1920×1280 pixels라면 그 크기의 image frame을 받을 수 있다. FrameArrived 콜백을 받은 뒤 프레임을 GPU surface로 읽는 것과 PNG/JPEG 파일로 저장하는 것은 별도 작업이다. 매번 디스크에 저장하도록 API가 강제하지 않는다. 프레임을 반납한 뒤 같은 surface reference를 그대로 보관할 수 없으므로 필요한 보존을 위한 복사와 수명 관리가 필요하다. [프레임 취득과 수명](https://learn.microsoft.com/en-us/windows/apps/develop/media-authoring-processing/screen-capture)

BGRA 4 bytes/pixel의 tight pixel payload라는 단순 가정이면 다음과 같다.

`1920 × 1280 × 4 = 9,830,400 bytes = 9.375 MiB/frame`

같은 프레임을 초당 10장씩 8초 보존하면 80장, raw pixel payload는 750 MiB다. 이 계산은 저장 구조, GPU allocation, stride, 인코더, 원음과 기타 상태를 제외한 설명용 산술이며 실제 footprint 측정이 아니다. 압축 후 크기는 이미지 내용과 형식에 달려 있다. 10fps와 8초는 예시이며 capture rate와 보존 한도의 선정이 아니다.

모델 입력 비용도 raw MiB와 같은 값이 아니다. 이미지 또는 video의 입력 개수, 해상도 처리와 tokenization, audio 길이, API 과금과 재처리에 따라 달라진다. 원본 보존, 업로드 자료와 모델 입력을 구분해 비용을 검토해야 한다. 여기서 특정 provider 가격이나 측정 결과를 확정하지 않는다.

## 7. 사용자가 제안한 단순 접근의 가능성과 한계

### 7.1 원음과 화면, 마우스 및 앱 자료를 함께 모델에 전달

음성과 이미지를 함께 이해하는 multimodal 모델이면 원음, 프레임 또는 video와 함께 마우스/앱 관측을 text/구조 자료로 전달하고 지칭 연결을 제안하게 하는 접근을 탐색할 수 있다. Qwen3-Omni의 공식 예시는 한 user message의 image, audio와 text 입력을 포함한다. 이는 입력 능력의 reference이고 VIA의 시간 grounding 정확성을 검증한 증거가 아니다. [Qwen 공식 예시](https://github.com/QwenLM/Qwen3-Omni)

현재 사용하는 모든 text LLM이나 hosted API가 같은 입력 조합을 지원한다고 가정하지 않는다. 실제 service의 동시 입력, item 수, 해상도, audio/video length와 시간 정보 전달 계약을 확인해야 한다. 마우스 관측에는 source time, coordinate basis와 window 관계를 표현하고 원음/화면의 상대 시점을 보존해야 한다. 자료를 많이 제공해도 없어진 frame이나 미지원 selection을 모델이 복구한다고 보증하지 않는다.

원본을 폭넓게 제공하는 초기 접근은 후보 메커니즘의 출발점으로 검토할 수 있다. 특정 A안, 비교 기준선, 호출 횟수 또는 최종 winner로 선정하지 않는다.

### 7.2 전사 도착 시각과 처리 속도에서 발화 시점을 추정

평균 전사 지연을 수신 시각에서 빼거나 알려진 원음 구간 길이에 전사 문자의 위치를 비례 배분하면 대략적인 시점 후보를 만들 수 있다. 하지만 음성 전달, queue와 계산의 지연, 발화 속도와 침묵이 달라지므로 수신 시각 하나에서 단어별 실제 시점을 유일하게 복원할 수 없다. 불규칙한 streaming partial도 같은 속도로 생성된다는 보장이 없다.

로컬 원음의 시작과 sample 범위가 있으면 그것을 시간의 기준으로 삼고 provider가 실제로 제공하는 span time 또는 명시적 추정 오차를 연결한다. 평균 속도에서 만든 시점은 추정으로 유지한다. 페이지 이동이나 짧은 pointing 사이에 오차가 걸치면 어느 대상인지 모호할 수 있다. 별도 word aligner 도입은 추가 모델/자원/실패 계약을 요구하는 후속 선택이며 지금 숨은 의존성으로 넣지 않는다.

## 8. 다음 설계에서 확인할 빈칸

1. 실제 음성 API가 주는 전사 span time, revision과 원음 참조 계약.
2. 실제 Windows capture의 범위, frame 수명, pointer 포함 여부와 좌표 기준.
3. 지원 앱/version별 UIA 요소와 selection/focus/scroll 정보의 관측 가능 범위.
4. source별 원본 시간과 수신 시각, 오차와 누락 표현, 서로 다른 조회의 일관성.
5. 원본의 취득과 보존 범위, 모델에 제공할 자료를 고르는 메커니즘, 정정 시 재사용 및 재판단 비용.

이 자료는 DP-47의 입력 현실을 명확히 하기 위한 기록이다. Windows 연동 코드, 실제 앱 검증, 모델 실행, 가격 계산과 품질 측정은 수행하지 않았다.
