# ASR-01 근거 원장 — Qwen3-8B 계산 profile

> 수집일: 2026-09-21. 공개 자료의 관측 행과 VIA의 가정을 구분한 검토용 원장.
> 제품 HW 사양, SLA, 실제 VIA 성능 또는 비사고 모드 실측을 확정하지 않는다.

## 1. 사용한 1차 출처

| Evidence ID | 원문 / 위치 | 확인한 사실 | 적용 한계 |
| --- | --- | --- | --- |
| E-QWEN | [Qwen 공식 Qwen3-8B model card](https://huggingface.co/Qwen/Qwen3-8B), Model Overview / Non-thinking | 모델 크기 8.2B, 비사고 hard switch 제공 | 기본 모델 기능 설명. 특정 PC 지연 자료 아님 |
| E-QC | [Qualcomm Qwen3-8B model card](https://huggingface.co/qualcomm/Qwen3-8B), Performance Summary의 X Elite / GENIEX_QAIRT 행 | w4a16, context4096, 생성12.949668 tok/s, TTFT0.1555~4.976s | 응답률은 짧은 입력·긴 thinking 응답 조건. 긴 context/짧은 비사고 JSON에 동일 속도 보장 아님 |
| E-TOK | [공식 tokenizer 파일](https://huggingface.co/Qwen/Qwen3-8B/blob/main/tokenizer.json) 및 [chat template](https://huggingface.co/Qwen/Qwen3-8B/resolve/main/tokenizer_config.json) | tokenizer 파일과 공식 template를 사용하여 직렬화해야 함 | tokenization이 추론 성공률 또는 출력 길이 예측을 보장하지 않음 |

E-QC는 TTFT 하한이 최대128 token의 한 prompt-processor 구간이고 상한은4096 길이에 대응한다고 설명한다. 생성률은 첫 응답 token **이후**의 속도이다. 위 수치들은 해당 표의 한 행을 함께 가져온 것이며 서로 다른 장치·runtime 행을 결합하지 않았다.

자료의 `main`과 웹 캐시는 갱신될 수 있다. 이번 최종 readback 행의 숫자와 조회일·runtime을 `qwen-reference-profile.json`에 고정했다. 문서 중간 조회에서 다른 숫자가 보이더라도 같은 profile에 섞지 않는다. 원문 commit, 세부 PC SKU/OS 빌드, 표의 반복 분포는 확인하지 못했으므로 `NOT_REPORTED`로 남긴다. 이 자료를 Windows 목표 장치의 실측값이라고 부르지 않는다.

## 2. 확인한 값, 계산한 값, 채택하지 않은 값

| 구분 | 값 | 의미 |
| --- | --- | --- |
| 공개 행의 생성률 | 12.949668 tok/s | 해당 Qualcomm 구성의 출처 관측값 |
| 공개 TTFT 범위 | 0.1555~4.976s | 입력 길이에 따른 출처 범위. 확률 구간 또는 p95가 아님 |
| 등가 입력률 | 128/0.1555 ≈ 823.15 token/s | **TTFT에서 계산한 등가값**. 순수 prefill 실측 tok/s가 아님 |
| 검토용 입력 처리 모형 | ceil(Nin/128)×0.1555s | 구간 비례 가정. 실제 길이별 벤치마크 곡선이 아님 |
| 이전 500/30 tok/s | 기준값에서 제외 | 앞서 연결한 llama.cpp discussion에서 해당 Qwen 행의 근거를 확인하지 못함 |

단순 숫자를 보수적이라고 이름 붙이지 않는다. 이 profile이 모든 8~10B 모델, 다른 accelerator, quantization, OS에 통용된다고 주장하지 않는다. 다른 runtime 행이나 0.5배/2배 속도를 시험할 때는 별도 sensitivity 가정으로 명시한다.

## 3. 계산식과 검산 예시

```text
B = ceil(Nin / 128)
T_first_hat = B × 0.1555
T_complete_hat = T_first_hat + (Nout - 1) / 12.949668
```

Nin은 최종 chat template까지 직렬화한 입력 token 수다. Nout은 응답 완료에 필요한 생성 token 수이며 EOS·검증을 위해 기다린 token도 실제 원장에 기록한다. `Nin + Nout ≤ 4096`인 profile 범위 안에서만 계산한다. 원문 상한4096의 TTFT 자체는 출력용 공간을 확보한 실제 응답 완료 시험과 구분한다.

**산식 검산용 입력값** Nin=1200, Nout=60을 주면:

```text
B = 10
추정 첫 token = 1.555초
그 뒤 59 token = 4.5561초
모델 응답 완료 소계 = 6.1111초
```

이 1200/60은 실제 prompt 측정 결과가 아니라 함수 검산용 값이다. 실제 VIA 전체 지연도 아니며, 아직 모델 로드·queue·입력 확보·출력 전달 등 빠진 비용을 0으로 간주하지 않는다.

## 4. prompt·output token 원장

[8개 prompt 초안의 생성 원문](../../../benchmark/rebaseline/build_assets.py)은 grounding, refinement, compound, task association, Agent selection, handling, response 구성, 통합 판단을 다룬다. 서로 다른 목적을 반드시 각각 한 번씩 호출하라는 pipeline 제안이 아니다.

각 항목은 system rule, JSON schema 지시, 실제 합성 Context, 사용자 요청, 수작업 output 예시를 분리한다. 예시 출력의 정답값은 모델 입력에 넣지 않는다. 공식 tokenizer 실행 뒤 field별 설명용 수, template 전체 입력 수, 출력 예시 수, 실제 생성 수를 구분한다.

공식 tokenizer.json의 확인된 SHA256:

`aeb13307a71acd8fe81861d94ad54ab689df773318809eed3cbe794b4492dae4`

**이번 실행 환경에서는 tokenizer 다운로드 및 해당 라이브러리 사용이 완료되지 않았다.** 따라서 원장에 가짜 token 수를 채우지 않았다. `token_count.py`는 공식 파일/해시가 없으면 계산을 거부한다. tokenizer와 template 파일, 사용 라이브러리 버전 및 직렬화 prompt hash를 함께 기록하도록 구현했다. 모델 가중치는 token counting에 필요하지 않다.

## 5. S2S와 실제 정확도는 별도 증거 필요

Qwen3-8B text generation 속도를 S2S audio token, 이미지 encode, 음성 인식·합성 시간에 적용하지 않는다. Voice 경로에는 음성 입력 종료·Voice Runtime 이벤트·실제 재생 시각을 측정하는 별도 trace가 필요하다.

본 profile은 구조의 호출 횟수·입력량·출력량·의존 관계를 같은 기준으로 비교하기 위한 입력이다. 실제 모델의 한국어 판별 정확도, 빠른 쪽의 더 나은 completion, 실제 지연 p95를 이 자료 하나로 증명하지 않는다.
