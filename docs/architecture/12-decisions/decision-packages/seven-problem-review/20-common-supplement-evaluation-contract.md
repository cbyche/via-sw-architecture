# 공통 보충 6조건 평가 계약

2026-10-19 / `COMMON-SUPPLEMENT6-20261019` / **126episode 범위 승인, 입력 명세 사전 등록; 구현, byte 동결, 유료 GO 전**

사용자는 기존 공통9의 315episode를 유지하면서 **6조건 × 모든 7구성 × 3반복 = 126episode**를 별도로 추가하도록 승인했다. 전체 지출 상한은 이전 실행과 개발 probe를 포함한 **$40** 그대로다. 이 문서는 독립 판단 담당 GPT-6 Astra Ultra가 작성한 사전 명세이며, GPT-6.1 Sol High 구현 담당의 fixture, 실행기, 수용 검사는 아직 별도 산출물이다. 문서의 수량은 완료 실적이 아니다.

사용자는 이후 DashScope Singapore로 이 범위의 작성된 합성 입력, 화면, instruction을 보내는 것에도 명시적으로 동의했다. 이 전송 동의와 총량 승인은 최종 source/fixture freeze 및 formal paid GO와 구별한다. 개발용 두 probe의 승인과 결과는 이 보충 입력을 고르는 근거로 사용하지 않는다.

## 1. 목적, 적용 계약과 한계

[19 공통9 계약](19-common9-evaluation-contract.md)의 모든 DP에 같은 QA와 같은 사례를 적용하는 원칙을 유지한다. 보충 모집단은 공통9에 없던 **DP2 독립 의미 작업의 실제 겹침**과 **DP3 작은 밀집 화면 요소**를 그 반대 조건과 함께 관찰한다. 공통9 입력, COMMON-28, 기존 점수 구간과 QA 의미를 바꾸지 않는다. 보충 입력은 새 합성 조건이며 기존 COMMON-28 사례를 그대로 실행했다고 표기하지 않는다.

공통9 대표 결과, 이 보충 결과, 개발 probe, 이전 부분 실행은 각각 따로 보고한다. 합계는 **441episode의 실행 계획**이며 두 모집단의 QA 평균과 예정 분모를 따로 유지한다. 구성당 공통9의215/115/75와 보충의36/24/18을 유지한다. 각 DP의 A 행은 같은 AAAA 관측을 참조하며 독립 표본으로 복제하지 않는다.

이 여섯 조건은 모든 UC 변형이나 전체24개 DP×QA 셀의 예측을 검증하지 않는다. DP2 슬롯 포화 조건, DP 간 조합 상호작용, DP3 자유 영역과 다중 페이지, 실제 Windows 앱의 native API 보편성 및 helper 배치 대안은 여전히 밖에 있다. B의 화면 생산기는 원본 PNG도 받으므로 순수한 text-only 구조라고 주장하지 않는다. 세 반복은 제한된 반복 관찰이며 통계적 충분성이나 최소 표본 수의 근거가 아니다.

공통 method-v3의 실제 source 제공, clock proof, effect gate, host display receipt, 현재성, 권한 검사, 취소와 정산 및 technical-unknown 규칙을 동일하게 사용한다. [독립 method 검토](reviews/common9-method-v3-astra-review-20261019.md)와 최종 freeze의 해당 sidecar가 정확한 구현 세대를 식별한다. 이 문서는 기존 계약19의 역사적 예산 문구나 보존 결과를 덮어쓰지 않는다.

## 2. 구성, 사건과 고정 분모

구성 순서의 기본 목록은 `AAAA, BAAA, CAAA, ABAA, AABA, AAAB, AAAC`다. 네 자리는 발표 DP1/DP2/DP3/DP4 순서이며 저장소01/02/04/05에 대응한다. 모든 구성은 아래 여섯 사례를 전부 실행한다.

| ID | 조건 | 사용자 입력 | QA01 응답 | QA03 최종 목표 | QA02 지칭 판단 |
| --- | --- | ---: | ---: | ---: | ---: |
| S80 | DP2 독립 의미 요청 겹침 | U01, U02 | 2 | 2 | 0 |
| S81 | 같은 두 요청의 순차 도착 | U01, U02 | 2 | 2 | 0 |
| S82 | 같은 초기 요청에 대한 종속 정정 | U01, U02 | 2 | 1 | 0 |
| S83 | 작은 밀집 버튼, native 선택 없음 | U01, U02 | 2 | 1 | 2 |
| S84 | 같은 버튼의 큰 크기, 넓은 간격 | U01, U02 | 2 | 1 | 2 |
| S85 | S83 화면과 실제 작성한 native 선택 | U01, U02 | 2 | 1 | 2 |
| **한 묶음** | **6episode** | **12** | **12** | **8** | **6** |

구성당3묶음으로 **18episode,36응답,24목표,18지칭 판단**이다. 고유7구성 전체 예정 수량은 **126episode,252응답,168목표,126지칭 판단**이다. 의무 Agent 업무 사건은 없다. 의미 오류로 후보가 불필요한 위임을 만들더라도 시험자가 성공 Agent 결과를 공급하지 않는다. 실제 발생한 잘못된 위임과 비용은 보존한다.

사건 ID는 `S80-U01`부터 `S85-U02`까지, 응답 ID는 각 `Sxx-RU01/RU02`다. S80/S81의 목표 G01은 U01 하나, G02는 U02 하나를 member로 가진다. S82–S85의 G01은 U01을 원래 시작으로 하고 U01,U02를 순서 있는 members로 가진다. U01의 완료만으로 정정 후 목표를 완료 처리할 수 없다. 목표, 정정 관계는 evaluator가 검사할 기대값이며 후보가 해당 ID로 답을 조회하게 하지 않는다.

QA02는 S83–S85의 U01 표현 `표시한 두 버튼`, U02 표현 `지금 표시한 두 버튼` 각각의 **최초 채택 집합**이다. 반복 수와 무관한 같은 표현의 두 revision을 서로 다른 판단으로 기록한다. D2의 명시적 문서 이름은 이 보충의 지칭 집합 분모에 추가하지 않으며, 필요한 자료를 실제 읽고 답했는지는 기능 진단으로 남긴다.

## 3. 공통 초기 상태, 권한과 시간

공통9의 합성200턴, 완료20Task 초기화, 동일 dependency/model snapshot, K=2, 공통 복구 상한, 새 episode process/cache 경계, Voice OFF와 text callback proxy, 50ms RSS 관측을 유지한다. 각 사례의 아래 원본은 초기화 시 제공한다. 모든 원본은 작성한 합성 자료이며 실제 개인 파일, 메일, 일정, 앱 자료를 수집하지 않는다. 허용 원본의 local read는 true, 외부 업무 전달은 false로 고정한다. 이 권한은 VIA의 도메인 자료 사용 계약이며 원격 모델 전송에 대한 인간의 별도 승인 절차를 대신하지 않는다.

초기 원본의 실제 성공적 제공 종료를 `Tinit`으로 기록한다. 첫 입력은 `Tinit+5초`에 시작하며 producer 완료를 기다리지 않는다. 후보가 아직 준비 중이면 원래 due/source 시각과 실제 전송, capture 지연을 모두 보존한다.

- **S80–S82는 Text 입력**이다. 작성된 typing 대기 길이는0이며 첫 완성 text의 due는 `Tinit+5초`다. 실제 음성 길이를 주장하지 않는다. input-begin/capture 프로토콜은 유지한다.
- **S83–S85는 길이6초로 작성한 전사 발화**다. 첫 source due는 `Tinit+11초`다. 실제 ASR/TTS나 audio 처리시간은 없다.
- S80/S82 U02의 완성 text due는 **U01의 실제 성공한 stdin flush 종료 `provided_at_ns + 2초`**다. authored due, 모델 시작이나 응답 시각으로 대신하지 않는다. 두 구성에서 같은 조건부 외부 도착 규칙을 적용하며 작은 실제 transport 차이는 원시에 남긴다.
- S81 U02는 U01의 실제 상세 `RESPONSE_FINAL` host callback 관측 후2초에 완성 text로 제공한다. S83–S85 U02는 그 callback 후2초에 발화를 시작하고6초 후 source가 된다. semantic correctness가 아니라 실제 final callback이 일정 anchor다.
- U01 실제 제공 후30초까지 final anchor가 없으면 기존 missing-anchor failure를 기록하고 그 fallback 시각 후2초에 U02를 시작한다. S81은 Text, S83–S85는6초 전사다. fallback은 가짜 final이나 순차 조건 충족이 아니다. 후보가 실제 종료했거나 기술 취소됐다면 입력을 성공적으로 제공했다고 만들지 않는다.
- 두 의무 source의 실제 제공/누락이 확정되면 마지막 실제 제공 flush 종료+60초까지 관찰한다. 정상 취소와+120초 정산, 조기 기술 abort의 별도 actual effective-stop 규칙은 method-v3 그대로다.

S80/S82의 actual-provided trigger를 지원하려고 고정 common9 harness를 몰래 수정하지 않는다. 별도 보충 runner/harness 버전에서 외부 fixture 로딩과 이 trigger를 구현하고, common9와의 정확한 diff 및 유한 수용 검사를 최종 freeze에 포함한다. 후보 Rust binary와 공통 evaluator 의미를 바꾸지 않는 것이 기본 범위다. 필요한 변경이 발견되면 원인과 새 동결을 별도로 판단한다.

초기 구상에서 사용한 capture anchor를 이 계약에서는 actual host provision으로 명확히 했다. 후보 내부 capture 지연에 따라 외부 입력을 늦추지 않으며, 실제 capture 시각은 발동 판정에 따로 사용한다. 이 선택은 보충 모델 결과가 나오기 전에 고정한다.

## 4. S80–S82의 정확한 원본과 요청

세 사례에 아래 두 문서를 모두 제공한다. UTF-8, LF, 마지막 LF 하나로 저장하며 표의 번호는 source line/extent를 고정한다. source `SUP-RELEASE`,revision1의 title은 `공개 검토 문서`, extent는 `P01`…`P08`, 각 line은 해당 번호 한 줄이다. 문장 내용은 다음과 같다.

```text
P01 공개 원칙: 외부 공개본에는 개인 이메일 주소가 없어야 한다. 내부 검토본과 외부 공개본은 별도 판본으로 관리한다.
P02 현재 판본: 초안 r2의 표2에는 개인 이메일 주소가 남아 있으며, 외부 공개용 정리본은 아직 작성하지 않았다.
P03 외부 공개 승인: 보안 승인과 정리본 검토가 모두 완료되어야 외부 공개를 진행할 수 있다. 한 가지 완료만으로는 충분하지 않다.
P04 승인 현황: 보안 승인은 완료됐고, 정리본 검토는 미완료다. 보안 승인은 초안의 이메일 주소를 공개해도 된다는 뜻이 아니다.
P05 일정 제안: 외부 공개 예정 시각은 월요일10시이며, 배포 지원 담당자는 월요일12시부터 지원할 수 있다. 예정 시각 자체가 승인을 대신하지 않는다.
P06 내부 검토: 기존 접근 권한이 있는 내부 검토자에게 내부 검토 링크만 공유하는 것은 가능하다. 이 허용은 외부 발송이나 새 외부 권한 부여를 포함하지 않는다.
P07 다음 조치: 개인 이메일 주소를 제거한 외부 공개용 정리본을 만들고 검토를 요청한다. 외부 공개 시각은 승인 완료와 배포 지원 가능 시각을 확인한 뒤 조정한다.
P08 요청 범위: 이 문서는 가능 여부와 조건을 검토하기 위한 자료다. 설명 요청만으로 문서 수정, 링크 발송, 외부 공개를 실행하지 않는다.
```

source `SUP-MEETING`,revision1,title `회의 안내문`,extent `meeting`은 다음 한 줄이다.

```text
수요일 점검 회의는14시에3층 회의실B에서 시작한다. 운영팀과 품질팀이 참석한다.
```

모든 세 사례 U01의 정확한 text:

> 공개 검토 문서의 여덟 항목을 대조해 월요일10시 외부 공개안을 진행할 수 있는지, 막는 조건과 다음 조치를 네 문장 이내로 설명해줘. 실제 수정이나 발송은 하지 마.

S80/S81 U02의 정확한 text:

> 회의 안내문에서 수요일 점검 회의 장소와 시작 시각을 알려줘.

S82 U02의 정확한 text:

> 앞 요청을 정정할게. 외부 공개 대신 외부 발송 없이 내부 검토 링크만 공유하는 경우로 바꿔서, 가능 여부와 남은 주의사항을 네 문장 이내로 설명해줘. 실제 공유는 하지 마.

후보가 받아도 되는 것은 위 실제 원본과 요청, 과거 실제 채택 상태뿐이다. 미래 U02, 기대 handling/goal/정답 요약은 입력에 포함하지 않는다. 제목을 명시했어도 어떤 범위가 필요한지와 두 요청의 독립성, 정정 관계는 후보가 판단한다.

평가자 전용 기능 조건은 다음과 같다. U01은 현 상태에서 외부 공개를 진행할 수 없다는 점, 개인정보 정리와 정리본 검토 미완료, 지원 시각 및 후속 확인을 보존해야 한다. S80/S81 U02는 수요일14시, 3층 회의실B를 답한다. S82 U02는 기존 권한의 내부 검토자에 대한 링크 공유 가능성을 설명하면서 외부 발송, 새 외부 권한 부여가 허용되지 않음을 유지하고 실제 공유하지 않는다. 이 조건들은 QA02로 바꾸어 채점하지 않는다. 내용상 오류와 잘못된 Task 연결은 별도 기능 결과로 모두 공개한다.

## 5. S83–S85의 화면, cue와 요청

### 5.1 동일한 내용과 고정 배치

모든 화면 원본은 **2880×1800**, 흰 배경, 동일한 검정 text와 같은 파란색 `열기` action icon이다. 각 행에는 행 이름, 보이는 `B01`…`B12` ID와 action button 하나가 있다. 다른 클릭 가능 요소나 선택 상태는 추가하지 않는다. source는 `SUP-SCREEN`,revision1,title `업무 목록`이며12개 line/extent의 고정 내용은 다음과 같다.

| line/extent | 보이는 행 이름 | 내용 |
| --- | --- | --- |
| 1 / B01 | 요구사항 | 요구사항 문서 열기 |
| 2 / B02 | 일정 | 일정 문서 열기 |
| 3 / B03 | 설계 | 설계 문서 열기 |
| 4 / B04 | 검토 | 검토 문서 열기 |
| 5 / B05 | 구현 | 구현 문서 열기 |
| 6 / B06 | 시험 | 시험 문서 열기 |
| 7 / B07 | 배포 | 배포 문서 열기 |
| 8 / B08 | 운영 | 운영 문서 열기 |
| 9 / B09 | 지원 | 지원 문서 열기 |
| 10 / B10 | 보안 | 보안 문서 열기 |
| 11 / B11 | 회고 | 회고 문서 열기 |
| 12 / B12 | 보관 | 보관 문서 열기 |

각 source text line은 `B03 | 설계 | 설계 문서 열기`와 같은 표 순서, 공백 형식이다. glyph는 같은 고정 font 파일을 사용하고 파일 SHA를 freeze한다. 행 이름과 ID는20px로 동일하며, 숫자 ID가 actual 화면에서 읽을 수 있도록 그린다. font 선택은 모델 결과 전의 가독성 확인과 함께 확정할 구현 항목이다. source catalog에 전체12개 이름/extent를 제공하는 것은 실제 문서 원본이다. 좌표 정답이나 선택 subset을 catalog에 추가하지 않는다.

행 번호 `j=1…12`에 대해 다음 geometry를 작성한다. 좌표는 frame pixel이다.

| 속성 | S83 DENSE 및 S85 NATIVE | S84 SPARSE |
| --- | --- | --- |
| 행 기준 y | `360 + 24×(j−1)` | `200 + 96×(j−1)` |
| button bbox | `[1400,y+4,1416,y+20]`,16×16 | `[1400,y+16,1432,y+48]`,32×32 |
| 행 이름 시작/기준선 | x1100,baseline y+20 | x1100,baseline y+40 |
| ID 시작/기준선 | x1310,baseline y+20 | x1310,baseline y+40 |
| 원형 pointer path 반지름 | button 중심에서11px | button 중심에서22px |

반복 button의16×16 기준 도형은 배경 `#e8f0fe`, 테두리 `#2b6cb0` 두께1, 같은 색의 열기 arrow `M5 11 L11 5 M6 5 H11 V10` 두께1.5다. Sparse는 이 도형 전체를2배로 그린다. target 크기와 간격의 **결합된 반대 조건**이며 두 요인의 개별 효과를 분리하지 않는다. font,12개 내용, 순서,viewport,request,정답 집합과 permission은 같다. PNG와 editable source를 함께 보존한다. 사람이 렌더된 글자, ID, cue 위치를 확인하는 기하/가독성 검사는 허용하지만, 모델 결과를 보고 유리한 fixture를 골라 바꾸지 않는다.

### 5.2 실제 작성한 관측과 두 발화

S83/S84에는 native 선택을 제공하지 않는다. U01에는 B03과B04 button 중심을 각각 한 번 원형으로 둘러가는 **클릭 없는 raw pointer path 두 개**를 제공한다. U02에는 B03과B05의 두 path를 제공한다. 각 원은 `k=0…16`의17점으로 닫히며, 중심+(r×cos(2πk/16),r×sin(2πk/16))를 소수점3자리로 반올림한다. 첫 path는 발화 시작 후800…1600ms, 둘째는3200…4000ms에 각50ms 간격이다. event type은 `pointer_path`,button은`none`; cue ID,frame ID,원시 좌표와 시각만 넣으며 item ID나 정답 bbox를 cue에 넣지 않는다. path 중간을 선택 click이나 native 선택으로 변환하지 않는다.

S85는 S83과 동일한 원본 pixels를 사용하되 같은 intended set을 작성한 native 선택 동작으로 제공한다. U01의 선택은 B03,B04, U02는 B03,B05다. 선택 갱신은 각 발화 시작 후800ms에 발생한다. `selected_object_ranges_only`, `complete_scene_objects=false`로 선택된 항목의 실제 source/revision/line range/bbox만 제공하며 전체 scene object 목록은 제공하지 않는다. pointer path/move/drag cue는 없다. 선택 metadata는 fixture의 작성된 선택 동작에서 생성하고 evaluator oracle을 읽어 만들지 않는다. 두 방안 모두 같은 native 관측을 받는다. 실제 상용 앱 API를 확인했다고 주장하지 않는다.

모든 frame의 evidence class는 `AUTHORED_RENDERED_SCREEN_NOT_CAPTURED_WINDOWS_APP`다. raw PNG,vector source,전체 source text,raw cue,작성한 selection action과 candidate용 native payload,평가자 oracle을 각각 파일로 남긴다. 관측의 item ID가 화면에 실제 보이는지와 bbox가 같은 좌표계를 쓰는지 확인한다.

frame ID는 S83/S85에서 `sup-dense-v1`, S84에서 `sup-sparse-v1`이다. path cue ID는 `Sxx-Uyy-path01/path02`, 각 cue의 `points`는 `{x,y,t_ms}`의17행 배열이며 top-level `t_ms`는 첫 점의 시각이다. native cue ID는 `S85-Uyy-native`,type은 `native_selection`,`t_ms=800`,`observed_in=frame.native_selection`이고 actual selected `item_ids`를 보존한다. 두 모드의 관측 차이를 숨기거나 서로의 cue를 추가하지 않는다.

기존 후보는 raw `pointer_path`를 의미 결합에서 처리할 수 있다. 이 circle을 단일 `pointer_move`로 변환하거나 `point_inside`의 단일점 검사를 통과한 것으로 가장하지 않는다. native 단축 경로와 circle의 의미 판단 경로 차이는 실제 구조의 일부로 기록한다.

세 사례의 정확한 U01 text:

> 표시한 두 버튼의 행 이름을 알려줘.

세 사례의 정확한 U02 text:

> 직전 요청을 정정할게. 지금 표시한 두 버튼의 행 이름을 알려줘.

U01 expected set은 `{SUP-SCREEN:v1:B03, SUP-SCREEN:v1:B04}`, U02는 `{SUP-SCREEN:v1:B03, SUP-SCREEN:v1:B05}`다. 첫 답의 행 이름은 설계, 검토, 정정 후는 설계, 구현이다. 정확한 canonical literal은 현 source catalog의 실제 format과 대조해 byte freeze한다. 전체 set exact match가 QA02이며 target별 성공 비율로 바꾸지 않는다. U02는 최신 관측으로 set을 다시 지정하므로 보존한 과거 subset을 자동으로 공짜 재사용했다고 주장하지 않는다. 잘못된 첫 채택 뒤 repair가 성공해도 최초 판단을 소급 정답으로 바꾸지 않는다.

## 6. 예측, 발동 판정과 반대 관측

### 6.1 DP2

사전 방향 예측은 **긴 VIA 의미 workflow 중 독립 U02가 도착하고 실제 공통 K=2에 유용한 여유가 있으면 B가 A의 실행 경계 대기를 줄일 수 있다**는 것이다. S80 U02의 QA01, 두 목표의 QA03, 모든 비용과 RSS를 함께 본다. S81은 같은 요청의 순차 반대 조건이며, S82는 독립 병행 대신 정정, 현재성 의존을 요구한다. 병행 자체를 token 절약이라고 예측하지 않는다.

각 실제 trace에서 다음을 별도로 기록한다.

1. U02 actual capture 시 U01의 실제 의미 workflow가 시작되어 아직 terminal에 도달하지 않았는지, 그리고 실제 모델 서비스 구간과 겹쳤는지. 단순한 사전 해석된 Agent fact 표시는 해당하지 않는다.
2. 그 시점의 K=2 사용/queue 및 producer 경합. pending 초기 생산이 슬롯을 모두 사용하면 독립 semantic capacity가 있다고 가정하지 않는다.
3. B에서 U01/U02에 속한 실제 의미 work 구간이 양의 길이로 겹쳤는지, A의 수용 경계와 재사용은 무엇인지. U01이 일찍 끝나거나 B가 실제 병행하지 않았으면 `MECHANISM_NOT_ACTIVATED` 또는 원인이 확인된 미실현으로 표시한다.
4. S82가 같은 목표 revision으로 연결되고 이전 결과의 stale completion/중복 실행을 막았는지. 잘못 독립 목표로 분리한 결과를 병행 이익으로 세지 않는다.

겹침을 만들려고 artificial model sleep, B만의 release trigger, 문서 확장, timing 재추첨을 하지 않는다. 조건이 발동하지 않아도 그 episode는 원래 분모에 남으며 다른 timing으로 자동 재실행하지 않는다. 동일 의미 품질, 완전 계측과 실제 발동이 확인됐는데도 B의 U02 지연 감소가 없거나 반대 방향이면 이 고정 fixture에서의 기대 경로는 지지되지 않는다. A가 재사용/짧은 경계로 이득을 상쇄한 경우 그 실제 경로를 공개한다. 세 반복의 우연한 방향을 일반적 구조 우월성으로 확대하지 않는다.

### 6.2 DP3

S83의 작은 밀집 geometry와 native 부재는 작성 입력으로 확인한다. B가 actual parser와 screen/speech 생산 및 binding/필요한 expansion을 수행했는지, 전체 비용과 RSS를 기록한다. 충분하고 정확한 후보는 B의 정확도 이익을 기대하는 **성공 조건**이지 결과를 제외하는 자격 조건이 아니다. 후보 누락, 잘못된 범위, 생산 실패는 구조 경로의 실제 반대 결과로 남긴다.

S84는 같은 의미를 더 큰 target/넓은 간격으로 해결하는 반대 조건이다. S85는 실제 selected ranges만으로 충분한 조건이며 `GROUNDING_NATIVE_CANDIDATES`와 actual helper dispatch 부재로 parser skip 여부를 확인한다. metadata가 있었는데 parser를 불필요하게 호출했거나 binding이 틀리면 그대로 결과다. 임의로 더 싼 native 실행으로 대체하지 않는다.

S83의 QA02 exact sets에서 B가 더 정확하면서 실제 후보/결합 경로가 그 차이를 설명하면 이 작은 작성 조건의 지지다. A도 같은 set을 맞추면 정확도 차이는 관측되지 않은 것이다. B 누락, 범위/결합 오류로 같거나 낮으면 반대 관측이다. S83/S84의 차이는 size+spacing 묶음 효과이며 helper 유무나 모든 화면 문제의 효과로 확대하지 않는다. native에서 차이가 줄면 해당 반대 조건의 관측이지 작은 밀집 조건 자체의 승패를 바꾸는 점수가 아니다.

DP1과DP4도 같은 여섯 사례의 모든 QA를 보고한다. 이 사례에 실제 handoff가 없으므로 DP4 handoff 이익의 별도 검증이라고 주장하지 않는다. 미사용 producer 비용도 제외하지 않는다. 결과 표는 조건별, 반복별 원시 쌍, 전체 보충 집계,발동/실패/unknown을 함께 제시한다. 일부 활성 episode 평균으로 고정 보충 평균을 교체하지 않는다.

## 7. 동일한 여섯 QA의 보충 집계

| QA | 보충 구성별 집계와 한계 |
| --- | --- |
| QA01 | 같은 callback endpoint의 관측 지연 합/관측 수와 **관측/예정36**. generic ack 제외 및 ordinary wrong-content 진단 분리는 공통 계약 그대로. |
| QA02 | 최초 채택 exact set의 **정답/예정18**. technical-unjudgeable와 ordinary wrong을 분리한다. |
| QA03 | 원래 목표부터 최종 required member revision까지 같은 causal DAG의 관측 VIA 시간 합/관측 수와 **관측/예정24**. 사용자 wait와 외부 업무만 제외하며 모델/queue/기술 receipt는 포함한다. |
| QA04 | 같은 stable candidate baseline의 C01–C06 실제 모듈 변경 수 합/**6과제**. 같은42개 변경 실험을 참조하며126개 변경 관측으로 늘리지 않는다. 아래 추가 회귀를 요구한다. |
| QA05 | 여섯 episode 한 묶음 전체 모델 비용의 **3회 평균**.18episode 총액, 역할별, 폐기/미사용/늦은 usage, unknown 별도. 공통9 묶음 비용과 단순 총량 절감률을 비교하지 않는다. |
| QA06 | 구성당18episode 전체의 최고 동시 RSS. 같은50ms, process/lifecycle 완전성 규칙이며 조건별 피크도 보조 표시한다. |

기술 증명이 불완전한 timing 숫자는 raw 진단에 보존하고 primary certified sum에 조용히 합치지 않는다. 원래 분모와 invalid/unresolved/missing 행은 유지한다. QA02의 known-correct/18은 기술 누락이 있으면 lower bound와 불완전 상태를 함께 표기한다. 실제 지출은 어떤 이유로도 삭제하지 않으며 불완전 RSS 최고값은 lower-bound 진단이다.

per-trace 평가는 동일 evaluator에 명시적인 보충 oracle을 전달한다. 전체 집계는 별도 버전의 adapter가 위12/8/6과3반복을 사용해야 한다. legacy aggregate의 common9 전역 oracle이나43/23/15 상수를 재사용하지 않는다. 공식과 technical eligibility predicate는 공통이고 모집단 식별과 분모만 명시적으로 다르며, 고정 분모와 누락 행의 수용 검사를 남긴다.

QA04는 같은 후보 source를 바꾸는 여섯 과제이므로 측정된 module set을 공통9와 보충 표에 **동일 측정의 참조**로 표시한다. 새로운42개의 가짜 독립 결과를 만들지 않는다. 각 실제 patched copy는 기존 공통9 회귀와 이 여섯 보충 회귀 및 과제별 결정적 수용 검사를 받아야 한다. 이 추가252회 API-free 회귀는 기존378회와 합쳐630개 patched-copy episode이며 유료126에 포함하지 않는다. 아직 실행되지 않았고, 기존42 결과를 상속해 성공 처리할 수 없다. 단순 문서/fixture 변경을 운영 Rust 모듈 수에 더하지 않는다.

## 8. 315와126의 고정 교차 실행 순서

단일 controller가 한 episode씩 실행한다. 공통9의 기존 `campaign.jobs()` 상대 순서와 구성 회전을 그대로 보존한다. 보충을 추가했다고 primary의 사례나 반복 순서를 다시 뽑지 않는다. 정확한441행 manifest는 아래 규칙으로 작성하고 paid GO 전에 hash를 고정한다.

1. primary 반복1,2,3 각각에서 S02,S08,S11,S12,S15,S18의 **7구성 block이 모두 끝난 직후** 보충7구성 block 하나씩 삽입한다. S19,S20,S26 뒤에는 삽입하지 않는다. primary 반복4,5에는 삽입하지 않는다.
2. 보충 기본 조건 순서는 `[S80,S83,S81,S84,S82,S85]`다. 반복1은 그대로, 반복2는 왼쪽2칸, 반복3은 왼쪽4칸 회전한다. 따라서 겹침/화면 조건을 번갈아 배치하며 각 조건의 세 반복은 각 primary 반복 안에서 한 번씩 실행된다.
3. 보충 구성 순서는 §2 기본7구성 목록을 `((반복번호−1)+(조건ID숫자−80)) mod7`칸 왼쪽 회전한다. 같은 조건, 반복의 일곱 구성을 연속 block으로 실행한다. primary의 기존 `((반복번호−1)+primary사례index) mod7` 회전은 변경하지 않는다.
4. 모든 root의 최상위 controller lock과 공유 budget ledger를 사용한다. 서로 다른 population root라고 병렬 paid 실행을 허용하지 않는다. 모델/RSS 관측 창에는 큰 local 검사나QA04 build를 겹치지 않는다.

이 순서는 결과, 속도, 정확도, 실제 구성별 비용을 보고 바꾸지 않는다. 지출 한계가 생길 때 특정 구성이나 모집단을 우선 완주시키지 않으려는 사전 순서다. 일곱 구성이 한 block에 있다는 사실만으로 예상치 못한 중간 예산 중단이 불가능해지는 것은 아니다.

## 9. 예산, 중단과 최종 동결 조건

전체 $40에 이전 확정 지출, 개발 probe,315,126,늦은 usage와 unknown/in-flight reservation을 모두 포함한다. 이 문서는126의 달러 비용이나441 완주 가능성을 추정하지 않는다. 현재 primary-only projection을126까지 감당할 수 있다는 증거로 사용하지 않는다.

유료 formal GO 전에 고정된 실제 보충 직렬화 요청, image byte, 역할별 output/repair 상한으로 **315+126 공동 projection**을 만든다. 보통 예상과 보수적 가정, image tokenization의 불확실성을 나란히 보존한다. 같은 공통 모델, 호출/복구 정책을 유지하며 기본 per-episode cloud cap100을 모든 구성에 적용한다.126에 대해서는 최대12,600회라는 호출 개수 상한이 있을 뿐, 그것이 $40으로 실행 가능하다는 뜻은 아니다. 한도 감소나 모델/복구 정책 변경이 필요하면 결과 전에 별도 공동 계약, freeze 판단을 받는다.

각7구성 block 시작 전 동일한 사전 계산 방식으로 그 block 전체의 보수적 계획액과 ledger의 남은 reservation-aware 잔액을 확인한다. 부족하면 어느 구성을 먼저 골라 실행하지 말고 block 시작 전에 정지한다. 이 계획 확인은 provider 실제 비용 보장이 아니며 각 API의 실제 reservation hard gate는 계속 적용한다. 예상 밖 비용으로 block 중간에 중단되면 실행/미실행 행과 남은 분모를 그대로 유지하고 campaign을 incomplete로 보고한다. 싼 구성만 보충 실행하거나 실패한 반복을 버리고 새 반복을 뽑지 않는다.

공동 projection이 사용자 선택의441episode와 상한을 합리적으로 양립시킬 수 없으면, **formal GO를 보류하고 구체적인 예산/공통 호출 정책 선택을 사용자에게 제시한다.** 승인 없이315나126을 줄이거나 $40을 늘리지 않는다. 단순히 projection이 상한 아래라고 실제 완주를 보장하지도 않는다.

최종 freeze 전에 남은 산출물은 다음과 같다.

- 위 문서, 화면, raw cue, native 작성 동작의 실제 UTF-8/PNG/vector/font hash, candidate 입력과 evaluator-only oracle의 분리 및 typed 연결.
- S80–S85 schedule과 actual-provided trigger 구현의 공통 method-v3 대비 diff, 실제 source-send brackets, 정상/누락/늦은 source 및 기술 abort 수용 검사. 이 명세의0/6초 입력 길이를 전송 지연과 혼동하지 않는다.
- 모든7구성×6사례의 fresh API-free 연결 회귀, 고정12/8/6 분모, 새로운 screen geometry/native provenance의 독립 검토. double의 정답은 후보 입력에 새지 않는다.
- 기존 core/evaluator 및 새 보충 runner/fixture/분석기 전체 mapping, QA04의 추가 여섯 회귀 수용, joint441행 순서와 동일 전역 budget/controller 적용 증거.
- 공동 비용 projection과 실제 잔액, 필요한 외부 모델 목적지/데이터 범위 승인, 별도의 명시적 paid GO.

현재 이 문서가 고정하는 것은 입력 내용, 조건, 분모, 순서, 판정 규칙이다. **실제 fixture와 asset hash는 작성 후 동결 전까지 PENDING**이며, 구현, 수용, paid 결과를 미리 완료로 선언하지 않는다. 입력 명세의 수정이 필요하면 paid 결과 전에 원인과 diff를 남긴다. 과거 결과와 계약19는 보존한다.

구현 담당의 읽기 전용 feasibility 검토는 S80–S85 ID, Text0초의 begin-before-capture 순서, 실제 provided+2초의 별도 scheduler 확장, 기존 pointer-path/native 입력과 명시적 oracle 평가가 현재 ABI와 양립함을 확인했다. 이는 실제 fixture 작성이나 runtime 수용 검사의 대체가 아니다. 후보 Rust나 기존 evaluator API를 바꿔야 한다는 요구는 이 검토에서 발견되지 않았으며, 별도 보충 집계 adapter는 필요하다.
