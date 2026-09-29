# 목표 Architecture 모델 기능 확인 원장

> 확인일: 2026-09-29 · **공식 자료 조사 / 사용자 제공 reference 확인 / 이 저장소의 실제 모델 실행 없음**
> [Architecture](./architecture.md) · [공유 Omni 설계·요구 계약](./shared-omni-runtime.md)

## 1. 현재 개발 방향과 근거 수준

사용자는 조직에서 `Qwen3-Omni-30B-A3B-Instruct`를 일부 fine-tuning해 테스트했다고 알려주었다. 이를 품질 reference로 삼고, 비슷한 성능의 약 10B급 on-device Omni를 개발하려는 방향이다. 여기서 해당 checkpoint·학습 변경·시험 환경·성능 결과를 직접 확인하지 않았으므로 `MEASURED_MODEL`이나 VIA QA 결과로 보고하지 않는다.

하나의 Omni를 음성·semantic 두 역할이 공유하고 역할별 Context·session·권한을 분리한다는 방향은 사용자 지정이다. 별도 ASR도 품질 요구에 필요하면 설계할 수 있다. 이전의 ‘반드시 S2S 1개 + 별도 semantic LLM 1개, ASR 추가 금지’ 제약은 현재 목표에 적용하지 않는다. 과거 DP/측정/ADR의 조건과 상태는 보존한다.

## 2. Qwen3-Omni 이름의 30B는 무엇인가

공식 [Qwen3-Omni Technical Report §2.5 Table 1](https://arxiv.org/html/2509.17765v1#S2.SS5)은 다음과 같이 모듈을 따로 기재한다.

| 모듈 | 보고서의 parameter 표기 |
| --- | --- |
| Thinker | 30B-A3B |
| Audio encoder | 650M |
| Vision encoder | 540M |
| Talker | 3B-A0.3B |
| MTP | 80M |
| Code2Wav | 200M |

따라서 모델명의 **30B는 Thinker의 전체 parameter 규모이며 전체 Omni 구성의 합이 아니다. A3B는 Thinker에서 활성화되는 parameter 규모를 가리킨다.** 표의 반올림 수치를 단순 합산하면 약 34.47B다. 이는 공표된 모듈 수치의 산술 합이며 실제 checkpoint tensor를 센 정확한 총수나 RAM 측정값이 아니다. Instruct는 Thinker와 Talker를 포함한다는 [공식 model card](https://huggingface.co/Qwen/Qwen3-Omni-30B-A3B-Instruct) 설명과도 일치한다.

같은 방식으로 VIA의 목표를 **Thinker 약 10B + 별도 집계한 encoder·음성 출력 모듈·ASR**로 기록한다. 주변 모듈까지 Qwen과 똑같은 크기로 고정하지 않는다. Dense/MoE·활성 크기는 미정이다. 약 10B가 30B-A3B보다 반드시 적은 연산을 쓴다는 뜻도 아니다.

`ESTIMATED_MODEL_ONLY`: Thinker weights만 10B × 2 byte ≈ 20 GB(BF16), 10B × 0.5 byte ≈ 5 GB(이상적인 4-bit payload)다. Quantization metadata, 혼합 정밀도 모듈, encoder/Talker/decoder/ASR, KV·activation·workspace·OS·VIA buffer는 별도다. 이는 목표 PC 적합성이나 quantization 후 품질 확인이 아니다.

## 3. 현재 기능에 근거한 설계와 남은 공백

| 요구 | 공식 근거 | VIA 설계에서의 해석·한계 |
| --- | --- | --- |
| 한 Omni의 음성·시각 입력과 text/speech 출력 | [Qwen3-Omni 공식 저장소](https://github.com/QwenLM/Qwen3-Omni), [model card](https://huggingface.co/Qwen/Qwen3-Omni-30B-A3B-Instruct) | 역할 공유의 출발점. 우리 PC의 동시 두 session·기한·권한 분리까지 자동 제공하는 증거는 아님 |
| Streaming·concurrent serving | [기술 보고서 §2.5](https://arxiv.org/html/2509.17765v1#S2.SS5): chunked prefill, Thinker/Talker 비동기 처리, concurrency별 latency | 구현 가능한 기법의 근거. 서버 조건의 공개 latency를 on-device VIA 값으로 복사하지 않음 |
| 큰 prefill과 decode 교대 | [vLLM v0.21.0 Optimization](https://docs.vllm.ai/en/v0.21.0/configuration/optimization/): chunked prefill과 decode batching | bounded scheduling의 근거. vLLM 선정·Mac 지원·hard real-time 보장·Omni 모든 kernel의 분할 지원을 뜻하지 않음 |
| 경량 Streaming ASR | [Qwen3-ASR 공식 저장소](https://github.com/QwenLM/Qwen3-ASR): 0.6B/1.7B, 한국어 포함, streaming 지원 | 별도 인식기의 현실성 근거. 공개 구현의 streaming은 timestamp 반환 미지원이라고 명시됨. 이것을 바로 VIA 완성 ASR로 선정하지 않음 |
| Token 시간 정보의 streaming 인식 계약 | [sherpa-onnx 공식 C API](https://github.com/k2-fsa/sherpa-onnx/blob/master/sherpa-onnx/c-api/c-api.h): online recognizer result의 tokens와 optional timestamps | 자체 Streaming ASR의 시간 근거 계약은 실재 기능을 참고한다. 지원 모델에서만 시각이 있으며 정밀 word start/end·오차·한국어 품질을 자동 보장하지 않음 |
| 더 정밀한 acoustic 정렬 | [Qwen3-ASR / ForcedAligner](https://github.com/QwenLM/Qwen3-ASR): 별도 0.6B aligner, 한국어 포함 | 필요시 선택 가능한 기능 근거. 주안은 시간 근거를 내는 ASR이며 추가 aligner는 아직 채택하지 않음. 도입 시 모델·지연·메모리 별도 집계 |
| Host 생성·취소, text/audio 식별 | [Realtime client events](https://www.alibabacloud.com/help/en/model-studio/client-events), [server events](https://www.alibabacloud.com/help/en/model-studio/server-events) | 현재 서비스가 제공하는 제어 계약의 예. 이 API와 공개 Qwen3-Omni checkpoint는 다른 dependency; 즉시 계산 중단·물리 재생 중단은 별개 |
| ASR과 Omni 해석 불일치 | [Realtime server events](https://www.alibabacloud.com/help/en/model-studio/server-events): 전사 helper와 native 해석이 다를 수 있음 | Canonical transcript revision과 불일치 proposal을 분리한다. API 하나에 숨겨진 별도 ASR을 모델 수에서 누락하지 않음 |
| VIA 좁은 direct/Core 제안·structured semantic·지정 Voice 내용 보존 | 위 modality·serving 기능만으로 VIA 수준의 판정 정확성까지 입증되지 않음 | 우리가 요구 schema·권한·실패 처리를 정한다. 학습/adapter 구현과 실제 통합 확인은 남음; 무오류·완벽한 의미 보존 가정 금지 |

공개 자료는 기능별 실현 가능성의 근거이며, **공유 Omni + 입력 ASR + 실시간 예약 + 한국어 지칭 처리의 통합 성공 증거는 아니다.** 이 조사에서는 모델 다운로드·학습·API 호출·추론을 실행하지 않았다.

## 4. 다음 구현 전에 닫을 항목

- 모델팀에 넘길 계약은 [공유 Omni 설계 §6](./shared-omni-runtime.md#6-우리가-정하는-모델runtime-계약)의 초안을 사용한다. 모델·serving·host의 구현 책임을 구분한다.
- 정확한 checkpoint/build, encoder·Talker·ASR portfolio, quantization과 목표 PC 자원 profile을 기록한다. 기존 mock/reference evaluation profile을 새 제품 실측으로 해석하지 않는다.
- 동시 입력에서 capture뿐 아니라 recognition·Omni Voice의 진행이 유지되는지, semantic도 기한 안에 진행하는지 확인해야 한다. 공개 서버 throughput만으로 PC 적합성을 판정하지 않는다.
- 한국어/영문명·숫자·부정·정정·지칭의 전사와 시간 오차, 입력 revision 간 취소, text/audio 내용·전달 범위, crash 재연결을 실제 연동에서 확인해야 한다.
- Native 계약을 못 제공하면 근거와 함께 adapter·helper·profile을 바꾼다. 기능 부재를 낮은 latency나 추가 모델 은폐로 감추지 않는다.

이번 문서는 기능 근거와 설계 요구를 정리한다. 새 Decision Package, 측정 freeze, 품질 동등성 또는 최적 성능 판정을 선언하지 않는다.
