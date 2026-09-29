# 공유 Omni와 끊기지 않는 음성 입력

> 상태: **공유 Omni·on-device·역할 분리·동시 입력은 사용자 지정 / 아래 배치·스케줄링은 주 설계안 / 구현·측정 없음**
> 갱신일: 2026-09-29 · [전체 구조](./architecture.md) · [모델 근거](./model-capability-review.md)

## 1. 선택한 구조

**한 번 적재한 Omni를 음성·semantic 두 역할이 공유하고, 독립적인 경량 Streaming ASR이 사용자 발화의 지속적인 인식과 시간 근거를 담당한다.** 공유 추론 서비스는 두 역할의 동시 세션을 지원하며, 음성 처리 자원을 확보한 뒤 남은 예산으로 의미 해석·응답 구성·기억 요약을 진행한다. 요청·Task·권한·외부 실행의 소유권은 계속 VIA host에 있다.

사용자가 알려준 reference는 약간 fine-tuning한 `Qwen3-Omni-30B-A3B-Instruct`다. 해당 조직 시험은 사용자 제공 배경이며 이 저장소에서 checkpoint·설정·원시 결과를 검증한 `MEASURED_MODEL` 근거가 아니다. 개발 목표는 이에 가까운 품질의 약 10B급 on-device Omni다. 품질 동등성, 압축 후 정확도, 목표 PC 성능은 아직 입증하지 않았다.

Qwen 명명 방식에 맞춰 **Thinker 약 10B**로 크기를 표현한다. Encoder·Talker·decoder·ASR을 모두 합친 전체 parameter·resident memory는 별도로 적는다. Dense/MoE 및 활성 parameter 수는 아직 정하지 않는다. 작은 total parameter 수가 reference의 A3B보다 빠른 추론을 자동 보장하지 않는다.

![공유 Omni와 입력 보호 경로](./diagrams/11-shared-omni-scheduling.svg)

[draw.io 편집 원본](./diagrams/11-shared-omni-scheduling.drawio)

## 2. 공유하는 것과 분리하는 것

| 항목 | 주 설계 |
| --- | --- |
| Omni weights | Thinker·encoder·Talker/decoder 각 모듈을 해당 runtime에 한 번만 적재; 역할별 복제 금지 |
| 역할 | `VOICE`와 `SEMANTIC`; 응답 구성·기억 요약은 semantic 역할의 job 종류이며 추가 모델 아님 |
| Voice session | 현재 입력 stream, 제한된 직접/Core 제안, 승인된 Voice 요약의 음성화; Task 조회·Context 도구·계획 없음 |
| Semantic session | 확정 입력 revision, 필요 원음·화면·대화·Task 근거, 구조화된 해석; bounded read는 host에 제안 |
| Session/KV state | 역할·Conversation·Request·revision·policy 범위별 분리; 같은 weights라는 이유로 hidden state를 섞지 않음 |
| Read-only cache | 동일 모델/build·token prefix·modality preprocessing·접근 범위가 검증된 prefix/encoder 결과만 재사용; 무효화와 비용 기록 |
| 게시·실행 권한 | Controller가 admission, Response Manager가 게시, Task/Gateway가 위임. Runtime은 처리 결과를 반환할 뿐 |

동일 weights를 두 번 호출한 결과는 독립 검증이 아니다. 둘이 같은 오해를 반복할 수 있으며 host의 provenance·revision 검사도 의미의 진실을 증명하지 못한다.

## 3. 발화를 받는 경로를 의미 해석과 분리한다

입력 보장은 세 단계로 구분한다. **녹음만 계속되는 상태를 정상적인 동시 음성 처리라고 부르지 않는다.**

1. **Capture:** Voice Process의 device callback이 monotonic sample sequence로 오디오를 받고 AEC·활동 감지·ring buffer를 처리한다. Omni·Core·Store 완료를 기다리는 lock이나 동기 호출을 두지 않는다. Barge-in은 이 경로에서 출력 세대를 무효화하고 재생을 멈춘다. 활동 감지는 의미상 취소 판단과 다르다.
2. **Recognition:** 별도 Speech Input Worker의 Streaming ASR이 지속적으로 partial/final transcript·revision·시간 근거를 만든다. 주 배치는 CPU 실행 예산·작업 thread·메모리를 확보하고 Omni accelerator 작업과 분리한다. NPU 이동은 해당 장비에서 지원과 동시 부하를 확인한 뒤 가능한 binding이다. Process 분리만으로 CPU·메모리 대역폭·전력까지 격리됐다고 주장하지 않는다.
3. **Omni voice processing:** 같은 오디오 참조를 Model Access의 VOICE session에도 제공한다. Stream ingest만 하고 semantic 완료까지 처리를 미루지 않는다. Omni의 audio encoding·voice prefill/decode에 주기적 예산을 예약하고 semantic과 interleave/batch한다. S2S 자체 지식 직접 응답은 좁은 admission 이후에만 게시한다. Core로 인계된 입력의 불필요한 speculative 답변 생성은 중지한다.

ASR은 별도의 작은 인식 모델이며 Omni의 두 역할을 대체하지 않는다. 기본 경로에서 인식을 중복하는 비용이 발생하지만, semantic 부하·Omni 재시작과 입력 근거 생성을 분리한다. ASR은 referent·Task·routing·도구를 결정하지 않는다. 원음은 요청 수명과 보관 정책 안에서 참조 가능하게 하여 전사만으로 운율·정정 표현을 잃지 않게 한다.

### 입력 근거와 불일치

`SpeechEvidence`는 stream/Turn ID, sample range, transcript revision, partial/final, 대체된 구간, token/span 시간·오차·방법, capture gap과 source build를 포함한다. 인식 token timestamp는 정확한 단어 경계가 아닐 수 있다. 모호한 단어 경계는 보수적인 acoustic interval로 확장하고 겹치는 화면 후보를 유지한다. ASR final은 recognizer의 확정 revision이지 사실상 무오류 선언이 아니다.

Interaction Manager가 정규화한 transcript revision을 VIA 입력 기록의 기준으로 사용한다. Omni가 다르게 들었으면 이를 덮어쓰지 않고 불일치 proposal로 반환한다. 목표·부정·숫자·수신자·지칭에 영향을 주는 불일치는 원음의 제한된 재확인 또는 clarification으로 해소한다. Canonical revision을 바꾸는 것은 Controller이며 관련 proposal·generation·미전송 admission을 무효화한다. 두 전사의 다수결이나 신뢰도 하나로 외부 업무를 보내지 않는다.

직접 S2S 후보에도 최종 입력과 의미상 충돌이 없어야 한다. 모든 단어를 별도 LLM으로 다시 대조하는 경로를 필수로 만들지는 않는다. 대응 근거 부족·핵심 불일치면 Core로 인계한다. 최소 direct/Core 제안 schema는 정하되 그 판정 정확도는 검증 대상이다.

## 4. Shared Inference Service의 스케줄링

Model Access의 **server 측 단일 scheduler와 weight owner**를 별도 Shared Inference Service에 둔다. Voice/Core는 bounded IPC client이며 각자 모델을 적재하지 않는다. ASR worker는 독립 예산으로 실행하되 동일한 canonical Model Access 계약으로 연계한다. API 진입점의 전역 generate mutex를 두지 않는다.

동시 활성 session은 최소 VOICE 1개 + SEMANTIC 1개를 수용하고, 이미 승인된 음성 출력이 있으면 해당 Talker/decoder 상태까지 계산한다. 여러 client의 동시 호출은 필수지만 매 순간 같은 accelerator에서 서로 다른 kernel이 물리적으로 병렬 실행되어야 한다는 뜻은 아니다. 짧은 계산 단위의 교대와 지원되는 microbatching으로 각 역할의 진행 기한을 지킨다. 이 기한을 못 지키면 단순 비동기 API만으로 요구를 충족했다고 하지 않는다.

| 작업 종류 | 실행 정책 | 포화 시 동작 |
| --- | --- | --- |
| Capture·local stop | 추론 scheduler 밖; 미리 할당한 buffer와 비차단 전달 | 입력 유실을 정상 처리하지 않음; 장치 장애·gap 명시 |
| Speech ASR | 독립 CPU 예산, 입력 진행 우선 | Omni·background 부하 축소; 장기 인식 지연/overflow는 기능 실패 |
| Omni 실시간 Voice | audio encode/prefill/decode와 승인된 playback 유지에 주기적 연산·KV 예약; 기한 가까운 작업 먼저 | 새 긴 작업 admission 중단; 발화 시작 시 기존 출력 생성 취소 |
| 현재 semantic 요청 | 남은 예산에서 보장된 최소 진행량 + deadline; foreground 내 bounded fairness | 긴 prompt는 chunked prefill, 오래된 revision 취소, deadline 결과 명시 |
| 응답 구성 | 짧은 현재 clarification·핵심 결과 우선; 세부 긴 요약은 별도 제한 | UI의 이미 확인된 결과는 즉시 표시; 음성 차례는 별도 대기 |
| 기억·과거 결과 요약 | 여유 예산에서만 진행; 별도 token·KV·시간 상한 | 먼저 pause/cancel; 원본 기록은 유지하고 무한 대기 job을 누적하지 않음 |

한 번 실행한 거대 prefill이나 image encoder kernel을 software queue 우선순위로 중간 중단할 수 있다고 가정하지 않는다. **가장 긴 비선점 계산 구간을 제한**해야 한다. Runtime 계약은 chunked prefill, decode step 사이 yield/cancel, 허용 image 해상도·crop/token 수, batch 크기와 encoder 작업 상한을 제공해야 한다. 안전한 분할을 지원하지 않는 작업은 입력 기한을 침해하지 않는 크기로만 admission한다. 필수 화면 근거를 몰래 잘라 정확도를 희생하지 않고, 근거 범위를 명시하거나 처리 불가를 알린다.

큰 batch를 채우느라 입력을 기다리지 않으며 deadline-aware microbatch만 허용한다. 취소는 host의 즉시 결과 무효화와 backend의 다음 안전 지점에서 계산 중단·KV 회수를 구분한다. 오래 걸리는 non-preemptible job이 입력 기한을 막으면 backend/profile을 재설계해야 한다.

### 유한 자원과 보장 범위

단순한 “높은 우선순위” 대신 device profile에 다음 예산을 가진다. 수치는 모델·장비 실행 뒤 정하며 이번 작업에서는 측정 freeze를 만들지 않는다.

- ASR의 연속 인식 처리량, 가장 긴 service gap, 최대 transcript lag.
- Omni Voice service gap, semantic 최소 service와 전체 deadline, 가장 긴 비선점 kernel 시간.
- weights 전체 + 실시간 KV 예약 + foreground KV + 제한된 background KV + activations/workspace + 오디오·화면 buffer + VIA/OS 여유 메모리.
- 허용 prompt/image/audio 크기, active session 수, finite queue, 취소·재시작 기한, 전력·열 저하 조건.

고정 window 안의 실시간 작업 비용과 최대 비선점 지연을 처리할 여유가 확인된 경우에만 background 작업을 받는다. 실시간 예약분은 낮은 우선순위 KV 때문에 eviction되지 않아야 한다. Semantic용 여유도 전혀 없는 profile을 정상 제품으로 승인하지 않는다. KV를 내보내거나 재계산하는 경우 재개 비용·정책 revision을 반영하며 pinned evidence를 임의 요약으로 대체하지 않는다.

한 명의 활성 사용자가 의미 해석 중 말하기 시작하는 상황은 필수 지원 범위다. 임의 길이 발화, 무한 동시 요청, 장치/OS crash까지 무조건 성공한다는 주장은 아니다. Buffer는 scheduler jitter를 흡수하며 장기 backlog 해결책이 아니다. 과부하 시 background 중단 → 긴 신규 추론 제한 → 확인된 상태 UI 안내 순으로 대응한다. 입력 유실·deadline miss가 발생하면 gap/실패를 드러내고 불완전한 요청은 dispatch하지 않는다. “나중에 들었으니 성공”으로 바꾸지 않는다.

## 5. 실제 중첩 시나리오

“이 문단을 설명해줘”의 semantic job A가 실행 중일 때 사용자가 “아니, 옆 표만 설명해줘”라고 말한다.

1. Voice가 즉시 입력을 capture하고 기존 playback을 멈춘다. ASR이 A의 완료와 무관하게 새 전사를 만든다. Omni Voice session도 예약된 계산 단위로 새 음성을 처리한다.
2. `InputStarted`가 Controller에 전달되면 같은 Conversation의 미전송 command를 hold한다. 아직 새 목표를 모르므로 모든 Agent 업무를 취소하지 않는다.
3. A의 게시·dispatch admission을 보류한다. 같은 대화의 speculative 생성은 취소하거나 결과를 보류하되 원래 요청 상태는 보존한다. 무관한 Conversation의 유효한 해석은 남은 예산에서 계속 진행한다.
4. 발화 확정 뒤 새 입력·시간별 화면·기존 A를 semantic job B에 제공한다. Interpreter가 정정이면 A를 대체하고, 무관한 추가 질문이면 A와 별도 Request로 처리한다.
5. A가 뒤늦게 완료해도 revision/hold 검사 때문에 이전 대상에 위임하거나 음성을 재개하지 못한다. B도 Controller 검증과 Response admission을 거친다.

긴 Agent 결과 요약 중 새 발화가 오면 요약을 멈추거나 뒤로 보내고 입력·새 요청을 우선한다. 이미 확인된 상세 결과는 화면에 남는다. 공유 Omni 중단 시에는 양쪽 Omni 역할이 함께 불가하지만 Voice capture·ASR·local stop과 Core의 Agent event 수신은 유지한다. 입력 접수 사실과 요청 처리 완료를 구분해 UI로 안내한다.

## 6. 우리가 정하는 모델·runtime 계약

다음은 모델팀의 미래 결정으로 미루지 않는 **요구 계약 초안**이다. 일부는 모델 학습, 일부는 serving/runtime, 일부는 VIA host가 구현한다.

| 계약 | 요구 입력·출력 | 구현 책임·미지원 시 처리 |
| --- | --- | --- |
| SpeechEvidence | sample sequence·clock → partial/final text, span timing·revision·gap | ASR + host timestamp 정규화. 필수 근거가 없으면 지칭 확정 금지 |
| VoiceProposal | 입력 revision·제한된 역할 → `DIRECT_CANDIDATE / HANDOFF`, 의존성 표시·답변 text/audio 참조 | 공유 Omni. Context/tool 탐색 없음; unknown·형식 오류는 Core |
| SemanticProposal | 원문·선택된 원음·화면·Task·receipt → field 상태·근거·bounded read/clarification 제안 | 같은 Omni semantic session. 최대 2회 refinement 주안·host 검증 유지 |
| SpeechRender | 승인된 Voice 요약문·publication/output epoch → segment ID·text 대응·audio·종료/오류 | Omni 음성 출력 모듈. 자율적으로 내용을 다시 판단·추가하지 않음; 대응 불가 구간 release 금지 |
| InferenceJob | role/job/session ID·input/context/policy revision·deadline·resource class·token/KV 한도 | Model Access 검증, runtime scheduler admission·동시 처리·실제 비용 event |
| Cancel/Pause | job/generation 지정 → host 무효화, cancel 접수, 계산 종료·KV 회수 상태 | runtime 안전 지점에서 처리; 즉시 GPU 중단을 약속하지 않음. 지원 안 하면 제한된 단위 재제출 |
| Restart | model build·runtime incarnation 변경 → 세션 종료와 새 ready 상태 | cache를 버리고 host 기록으로 필요 세션 재구성; 과거 음성/외부 업무 자동 재실행 금지 |

출력 schema 준수와 문장의 사실 정확성은 별개다. S2S direct 허용 판단, 한국어 정정·지칭, text/audio 의미 보존은 실제 모델에서 검증해야 한다. 모든 기능을 현 reference의 fine-tuning만으로 확보 가능하다고 단정하지 않는다. 실현 가능한 개별 기능의 공개 근거와 통합 시 남은 공백은 [모델 확인 원장](./model-capability-review.md)에 기록한다.

## 7. 비용과 설계를 다시 볼 조건

- 추가 ASR의 weights·CPU·전력·recognition 오류, Omni와의 중복 음성 처리, 전사 불일치 조정 비용이 생긴다. ASR 출력이 semantic 모델의 잘못된 전제를 굳히지 않도록 원음·revision을 남긴다.
- Omni weight 공유는 메모리를 아끼지만 두 역할의 계산·KV는 공짜가 아니며 모델 장애가 두 역할에 함께 영향을 준다.
- 작은 chunk는 latency 격차를 줄이는 대신 throughput·전력 효율을 낮출 수 있다. Voice 예약 때문에 semantic 완료가 느려질 수 있다.
- shared runtime 교체나 checkpoint 교체가 양쪽 역할에 영향을 준다. Adapter로 이 결합을 없앴다고 주장하지 않는다.
- 지원 입력·동시 부하에서 음성 인식 또는 Omni Voice 진행이 semantic 작업 뒤로 계속 밀리거나, 필수 근거를 잘라야만 실행되거나, semantic이 지속 기아 상태이면 이 배치/profile은 부적합하다.
- native Omni만으로 같은 입력 지속성·시간 근거·fault 조건을 더 작은 총비용으로 충족한다는 근거가 생기면 별도 ASR의 필요성을 재검토한다. 현재 그 근거가 확보됐다고 보지는 않는다.

이는 목표 설계의 약점과 실현 조건이다. 새 Decision Package, 상대 후보 순위, 수치형 ASR 검증 계약이나 결과는 아직 만들지 않는다.
