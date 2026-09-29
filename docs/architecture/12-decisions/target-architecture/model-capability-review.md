# 목표 Architecture 모델 기능 확인 원장

> 확인일: 2026-09-29 · 상태: **공식 문서 1차 확인 / 제품 모델 미선정 / 실제 모델 실행 없음**
> [Architecture 본문](./architecture.md) · [남은 설계](./design-completeness.md)

## 1. 확인 범위

현재 목표 Architecture가 채택한 실제 모델·build·provider 설정은 없다. 저장소의 [기존 evaluation profile](../../11-measurement/evaluation-profile.md)은 mock 평가와 optional Qwen3-8B reference를 다루며 목표 제품 모델 선정이 아니다. 그 문서의 과거 실행이나 기기 수치를 현재 제품의 기능·성능 증거로 승격하지 않는다.

우선 저장소에서 언급한 Qwen 계열을 문서 조사 출발점으로 사용했다. Qwen3-Omni 공개 가중치, Qwen-Omni-Realtime 서비스, Qwen3-8B semantic reference는 서로 다른 dependency다. 한쪽 기능을 다른 쪽에 자동 적용하지 않는다. 이 원장은 후보 선택·추천이나 측정 freeze가 아니다.

## 2. 공식 문서로 확인한 것과 남은 것

| VIA 요구 | 확인 근거 | 현재 판단·설계 영향 |
| --- | --- | --- |
| 음성 생성 시작·취소 제어 | Realtime의 `input_audio_buffer.commit`은 자동 응답을 시작하지 않는다고 설명하며 `response.create`·`response.cancel`을 명시한다. [Client events](https://www.alibabacloud.com/help/en/model-studio/client-events) | 해당 API에 제어 수단은 문서화됨. 이미 받은 audio의 playback stop·epoch는 VIA 책임이며 실제 취소 지연은 미실측 |
| 생성 audio와 응답 text 수신 | Realtime의 `response.audio.delta`·`response.audio_transcript.delta`와 response/item 식별자가 있다. [Server events](https://www.alibabacloud.com/help/en/model-studio/server-events) | stream 수신 근거는 있음. 실제 들린 단어 범위·문장별 정밀 정렬·지정 내용 보존까지 증명하지 않음 |
| 별도 ASR 없는 입력 전사 | Realtime 문서는 `input_audio_transcription`을 `qwen3-asr-flash-realtime`이 제공하며 S2S 해석과 달라질 수 있다고 명시한다. [Server events](https://www.alibabacloud.com/help/en/model-studio/server-events) | 이 전사 기능을 S2S native 기능으로 계산할 수 없음. 그대로 의존하면 S2S 1개·semantic LLM 1개 제약과 충돌하므로 적합 판정 불가. API 하나라는 이유로 추가 모델을 숨기지 않음 |
| “이거” 발화 구간과 화면 연결 | 문서의 speech start/end는 발화 경계이고 final transcript 예시는 문자열이다. [Server events](https://www.alibabacloud.com/help/en/model-studio/server-events) | 확인한 event schema만으로 단어/지칭 구간별 acoustic timestamp 확보를 입증하지 못함. 이것은 모든 Qwen build의 불가능 판정이 아니라 현재 증거의 한계 |
| 공개 Qwen3-Omni의 음성·시각 기능 | 공식 저장소는 audio/image/video 입력과 text/speech 출력을 설명한다. [Qwen3-Omni](https://github.com/QwenLM/Qwen3-Omni) | 기본 modality 근거는 있음. Realtime API와 같은 제어·전사·시간 정렬 계약을 제공한다고 간주하지 않음 |
| semantic LLM의 이미지 화면 처리 | Qwen3-8B 공식 card는 causal LM과 text-generation 경로를 설명한다. [Qwen3-8B](https://huggingface.co/Qwen/Qwen3-8B) | 기존 text reference가 이미지 화면까지 지원한다는 근거로 사용할 수 없음. 목표 semantic 모델의 시각 입력 지원을 별도로 확보해야 함 |
| 최소 직접/Core 구분·요약 음성의 의미 보존 | 위 공개 설명만으로 VIA의 좁은 routing과 중요 실패·조건 보존 정확성을 확인할 수 없음 | `미검증`. 프롬프트 지시가 있다는 것과 해당 계약을 안정적으로 지킨다는 것은 다름 |

현재 Realtime 문서는 여러 서비스 버전을 함께 다룬다. 특정 date-pinned build의 지원은 별도 확인해야 하며, 최신 문서를 과거 실행 구성의 증거로 소급하지 않는다. 이 조사에서 API 호출·오디오 전송·모델 다운로드·추론 실행은 하지 않았다.

## 3. 다음 확인과 완료 조건

1. 필요한 native 입력 전사·지칭 시간 근거·출력 제어·semantic 시각 입력을 만족하는 정확한 build/runtime 조합을 확인한다. Provider 문서에 없는 계약은 미확인으로 남긴다.
2. source timestamp의 기준·오차, text/audio 대응 단위, host 생성·취소·재생 제어, 단순 direct/Core 인계 출력을 adapter 계약으로 적는다.
3. 문서와 공개 코드로 충족 가능한 부분, 직접 연결이 필요한 부분, 모델 추가 없이 충족 못 하는 부분을 구분한다.
4. 실행 환경을 확보한 뒤 고정된 간단 입력으로 계약의 입출력을 확인한다. raw event·build·설정을 남기며 제품 E2E나 QA 점수로 부풀리지 않는다. 본 원장은 그 실행의 실험 조건을 동결하지 않는다.
5. 필수 계약이 미확보이면 대체 dependency 또는 구조 수정이 필요하다. 낮은 성능 수치로 기능 부재를 덮거나 추가 ASR/TTS/aligner를 조용히 도입하지 않는다.

공식 문서 확인은 기능 검증의 시작이며 전체 모델 확인 완료가 아니다. 이 작업은 설계 실현 가능성 점검이고 Decision Package 비교나 성능 실험은 아직 시작하지 않는다.
