# 음성 입력 근거를 별도 recognizer로 만들 것인가, 공유 Omni에서 받을 것인가

> **상세 검토 후보 / 사용자 선정 전 / native evidence 기능 적합성 미확인** · [목록](./README.md)
> 방안 1은 검토 완료 target, 방안 2는 같은 사용자 기능을 위한 연동 계약 제안이다. 실제 모델 지원·정확도·자원 우열은 확인하지 않았다.

## 1. 배경 — 말하는 동안 바뀐 화면과 나중에 도착한 전사를 연결해야 한다

사용자가 그래프를 가리키며 “이걸”, 다른 표로 옮겨 “아니, 저걸 보고서에 넣어줘”라고 말한다. VIA에는 최종 문장뿐 아니라 각 표현의 발화 구간, 정정 관계, 당시 화면이 필요하다. 전사가 도착한 순간의 포인터만 보면 다른 대상을 위임할 수 있다. 이때 이전 요청의 semantic 추론도 진행 중일 수 있다.

따라서 **음성 대화에 쓰는 Omni와 별개로 입력 근거를 생산할 것인지, Omni의 음성 처리에서 근거도 함께 받을 것인지**를 정해야 한다. 전자는 별도 모델·worker·중복 인식·전사 충돌을 감수한다. 후자는 입력 근거까지 같은 runtime의 기능·계산 진행·장애에 의존한다. Component 이름이 아니라 dependency와 정보 생산 경로가 달라진다.

현재 선택은 [공유 Omni §3~7](../target-architecture/shared-omni-runtime.md#3-발화를-받는-경로를-의미-해석과-분리한다), [전체 구조 §6·14](../target-architecture/architecture.md#14-프로세스-배치fault-boundary), [입력 종료 계약](../target-architecture/control-and-lifecycle.md#1-입력과-종료-판정)에 명시돼 있다. [UC-03·04](../../05-representative-use-cases.md#uc-04)의 지칭, [UC-11](../../05-representative-use-cases.md#uc-11)의 정정, [UC-18](../../05-representative-use-cases.md#uc-18)의 장애 안내와 연결된다.

## 2. 두 가지 구현 방식

### 방안 1 — 독립 Streaming ASR + 공유 Omni

1. Interaction Manager가 같은 microphone sample 구간을 Model Access 계약으로 Speech Input Worker와 Omni VOICE session에 전달한다. 화면 timeline과 clock mapping은 공통 host 책임이다.
2. 별도 경량 Streaming ASR이 partial/final transcript, 대체 구간, span 시각·오차·gap을 만든다. Interaction Manager가 이를 정규화한 SpeechEvidence로 유지한다.
3. Omni의 VoiceProposal이 전사를 다르게 해석하면 불일치로 보존한다. 핵심 대상·부정·수신자 등이 달라지면 Request Controller가 제한된 원음 재확인 또는 clarification으로 해소한다.
4. 최종 전사와 producer watermark/gap을 결합한 입력만 SEALED로 사용한다. 근거가 부족하면 실행을 보류하며, ASR final을 무오류 정답으로 취급하지 않는다.

ASR은 의미·Task·routing을 결정하지 않는다. 입력 근거의 독립 진행이 목적이며, Omni의 음성 처리와 의미 해석도 기존 동시 session·유한 자원 계약을 유지한다. 별도 process라는 사실만으로 열·대역폭 경합까지 해결되지는 않는다.

### 방안 2 — 공유 Omni의 native SpeechEvidence 출력

1. 같은 audio를 공유 Omni의 VOICE 입력 처리에 제공한다. 별도 ASR 모델과 Speech Input Worker는 두지 않고, Model Access가 native evidence stream을 host 계약으로 정규화한다.
2. Native stream은 transcript revision·partial/final·source sample 구간·시간 오차·gap을 제공해야 한다. 결과 도착 시각을 발화 시각으로 쓰거나 host가 없는 timestamp를 만들어서는 안 된다.
3. Interaction Manager가 같은 화면 timeline과 evidence를 결합한다. Request Controller는 native final과 watermark로 입력을 닫고, 정정된 revision에 의존한 proposal·admission을 무효화한다.
4. VoiceProposal은 그 입력 revision에 결합한다. 외부 자료·Task 없이 답하는 좁은 S2S 범위와 host gate는 유지한다. 동일 모델의 input echo 일치는 독립 recognizer와의 대조가 아니므로 인식 오류를 검출한 증거로 쓰지 않는다.

대안에도 streaming evidence를 허용한다. 이전 문서의 turn-final-only 제약을 가져와 일부러 늦게 만들지 않는다. 반대로 native timestamp·동시 진행이 이미 지원된다고 가정하지도 않는다. 이는 모델팀/runtime에 요구할 **관측 가능한 계약**이며 내부 학습·출력 head 설계는 이번 작업에 포함하지 않는다. 추가 학습 가중치가 필요하면 Omni 전체 inventory에 드러내야 한다. 숨은 ASR·aligner를 붙여 단일 dependency라고 부르면 안 된다.

## 3. 같은 상황을 따라가 보면

| 사건 | 독립 ASR | Native Omni evidence |
| --- | --- | --- |
| semantic 작업 중 새 발화 | 독립 recognizer가 전사하고 Omni VOICE도 예약된 계산으로 진행 | Omni scheduler가 evidence 생산까지 진행시켜야 함; 녹음만 쌓으면 기능 미충족 |
| “이걸… 아니 저걸”과 화면 이동 | ASR revision·구간을 당시 화면과 결합 | native revision·구간을 같은 화면과 결합; final-only를 강제하지 않음 |
| 음성을 잘못 인식했지만 schema는 정상 | Omni와 다르면 불일치를 발견할 여지; 둘 다 틀릴 수도 있음 | 같은 모델의 일관된 오인식을 echo 검사로 잡지 못할 수 있음; 원음 재확인·사용자 정정 유지 |
| Shared Inference Service crash | capture·ASR 입력 근거는 유지, 의미 처리·음성 생성은 중단 | capture·local stop은 유지하지만 전사 생산도 중단; 유한 backlog·gap과 미처리 안내 |
| 입력 모델/build 교체 | ASR 계약과 Omni 계약을 각각 점검하고 불일치 처리를 검증 | Voice·semantic·입력 evidence의 결합 호환성을 함께 점검 |

**장애 차이는 보류 이유만으로 지우지 않는다.** UC-18은 확인 불가·안전한 실패도 요구하며 모든 장애에서 자동 성공을 요구하지 않는다. 다만 target이 선택한 ‘Omni 장애 중에도 전사 유지’는 대안이 잃는 실제 능력이다. 향후 이것을 필수 qualification으로 정하면 순수 native안은 부적합할 수 있다. 지금 그 요구를 몰래 완화하거나 ASR 비교 분류를 확정하지 않는다. 독립 ASR을 fallback으로 추가하면 별도 혼합안이며 해당 가중치·기동 지연·메모리를 모두 포함해야 한다.

## 4. 실제 구조 차이와 비용

| 변경 위치 | 방안 1 | 방안 2 |
| --- | --- | --- |
| Dependency / runtime | 경량 ASR·Speech Input Worker + 단일 Omni | 단일 Omni의 native evidence 경로; ASR worker 제거 |
| Model Access | 두 인식 경로의 adapter·build·자원·장애 관리 | native evidence·Voice·semantic의 공통 runtime capability와 자원 계약 |
| Interaction Manager | ASR 기준 전사 + Omni 불일치 연결 | native revision·gap·timestamp 정규화; 이중 전사 조정 경로 제거 |
| Request Controller | ASR final 기반 SEALED·불일치 해소 | native final 기반 SEALED·오류 대응; input echo의 검증 의미 재검토 |
| 자원 | ASR weights·CPU·workspace·stream 및 중복 음성 처리 | ASR 고정 비용 감소 가능, native 출력·KV·공유 계산 비용과 장애 결합 |

같은 Omni weights는 한 번만 센다. 별도 ASR 제거가 전체 peak memory 감소를 보장하지는 않는다. Native 기능의 추가 상태·workspace와 backlog, 독립 ASR의 전체 weights·CPU 비용을 함께 봐야 한다. 실제 timestamp 정확성·인식 품질은 dependency 특성이므로 구조 자체의 정확도 개선과 구분한다.

**현재 방식이 설득력 있는 조건:** 공유 추론이 바쁜 동안에도 입력 근거가 필요하고, Omni의 native evidence 계약보다 독립 recognizer의 계약을 확보·교체하기 쉬우며, 추가 자원 비용을 감당할 수 있는 경우다.

**왜 대안을 선택할 수 있는가:** native evidence가 같은 발화·지칭 기능과 동시 진행을 제공하고, 별도 인식의 중복 계산·메모리·전사 조정 부담이 크며, 공유 runtime 장애 때의 기능 축소를 수용할 수 있는 경우다. 능력 미확인은 실제 확인 항목이지 후보 발견을 멈출 이유가 아니다.

**현재 선택을 다시 볼 조건:** 독립 ASR이 실제로 새로운 입력 지속성·근거 품질을 제공하지 못하면서 전체 자원·불일치 조정 비용을 늘리고, native 계약으로 같은 정상 기능을 제공할 수 있는 경우다. 반대로 native evidence가 semantic 완료를 기다리거나 source-time을 제공하지 못하면 대안은 지원 범위에서 탈락한다. 수치·결과는 아직 없다.

## 5. 다른 후보와 고정할 조건

Audio·화면 관측·권한·단일 Omni·사용자 목표는 공통이다. [해석 방식](./request-interpretation-topology.md), [Context 조회 시점](./context-acquisition-strategy.md), [직접 응답 생성 순서](./direct-response-generation.md)는 바꾸지 않는다. 정정·Task·게시 검증을 제거하지 않으며 normal semantic 부하 중 인식 지속은 양쪽 필수다. 같은 fault를 주되 서로 다른 실제 영향 범위를 보고한다.

영향받는 target 계약은 SpeechEvidence source, InputFinal/SEALED, VoiceProposal input_echo, 모델 inventory와 §14 장애 동작이다. 기준선은 그대로 둔다. Dependency 자체가 선택 대상이므로 양쪽 모델 포트폴리오를 억지로 같게 만들지 않되, 더 우수한 build의 효과를 host 구조의 효과로 포장하지 않는다. 실제 build 적합성은 두 안 모두 검증 전이다.

## 6. 발표 페이지의 핵심

**배경 1장:** 발화·화면 전환·전사 도착의 세 시간축과, 동시에 돌아가는 semantic 작업을 배치한다. 늦게 도착한 “이걸”을 현재 화면에 연결했을 때의 잘못된 대상을 보여준다. 하단 질문은 **“계속 듣고 당시 대상을 찾기 위한 입력 근거를 누구에게 의존할 것인가?”**다.

**비교 1장:** 공통 microphone·화면 timeline·Request Controller 검증은 검정이다. 1안의 audio 분기·ASR worker·불일치 조정과 2안의 native evidence stream·공유 자원 의존은 양쪽 파랑이다. 모델·process·논리 Component를 구분하고 `semantic 실행 중`과 `Omni crash` 두 삽화로 정상 진행과 장애 범위를 보인다. 모델 수만 그리지 않는다. 새 그림은 공동 후보 검토 후 `.drawio`/`.svg`로 제작한다.

## 7. 바로 사용할 발표 요약

**배경 5줄**

1. VIA는 음성의 내용뿐 아니라 지칭 표현이 나온 시각과 당시 화면을 연결해야 한다.
2. 이전 semantic 작업 중에도 새 발화를 계속 인식해야 정정·끼어들기를 처리할 수 있다.
3. 독립 ASR은 입력 진행을 분리하지만 별도 모델과 중복 인식·전사 조정이 필요하다.
4. Omni 통합은 dependency를 줄일 수 있지만 입력 근거까지 공유 계산·장애에 의존한다.
5. 핵심은 recognizer의 소속이 아니라 입력 근거 생산과 실패 경로를 어떻게 구성하는가다.

**설계 비교 8줄**

1. 방안 1은 같은 audio를 독립 ASR과 공유 Omni에 보내고 ASR 전사를 입력 기준으로 삼는다.
2. 방안 2는 별도 ASR 없이 Omni의 native 전사·시각·revision stream을 받는다.
3. 양쪽 모두 당시 화면·clock·gap·최종 revision을 보존하고 host가 요청을 확정한다.
4. 대안에도 streaming을 허용하며, timestamp나 동시 인식 능력이 이미 있다고 주장하지 않는다.
5. 독립안은 모델·worker·불일치 조정 비용, 통합안은 native 기능·공유 scheduler 의존이 남는다.
6. Omni 장애 시 독립안은 전사를 계속할 수 있지만 native안은 입력 근거도 멈출 수 있다.
7. 이중 인식의 효용보다 총비용이 크고 native 계약이 충족되면 대안을 선택할 이유가 있다.
8. Capture만 유지하거나 장애 손실을 숨긴 안은 동등한 대안으로 인정하지 않는다.
