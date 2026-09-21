# 11-A/B 진행 및 근거 재검토 기록

> 검토일: 2026-09-21
> 검토한 저장소 기준: `dd1fe95dcfa36251a33ac489e7b036ad50c8d3cd`
> 상태: 문서·합성 시험 명세는 리뷰 가능. 실제 모델/VIA 후보 시험과 정확 token 산정은 완료되지 않음.

## 1. 현재 준비 상태

| 항목 | 확인한 산출물 | 상태 |
| --- | --- | --- |
| 08 측정 정의 보정 | 7개 ASR 유지, VIA 직접 LLM/S2S 비용 포함, Safety의 V/N 원자료 정의 | 반영 확인 |
| 11-A 측정 기준 | 시간 경계, TTFT/생성 길이, 자원 경합, 변경 요소, 성공/실패·안전 집계 규칙 | 검토본 |
| 11-B 시험 목록 | 18개 UC의 94개 명시 변형에 대응하는 시험 명세 | 검토본. 실제 녹음·장비 실행 완료가 아님 |
| 변경 분석 | A 9개, M/C 15개로 나눈 24개 계약 변경 | 명세. 후보별 변경 개수는 미측정 |
| 안전 시험 | 24개 판단 기회, 허용 6개/차단 18개 | 설계된 시험 분모. 실제 위반 결과가 아님 |
| Prompt | 8개 목적의 prompt·schema·Context·예시 출력 | 초안. 정확 token 수와 실제 생성 token 수는 미산정 |
| 참고 발표자료 | Word 가이드, Excel 카탈로그, ZIP의 README·manifest·대표 사진 | 검토 |
| 11-C | 목표·0~5점·최종 집계/반복 | 미진행 |
| 12 | DP 선택·후보 비교·결정 | 미진행 |

상세 내용: [11 검토 안내](../11-test-case-catalog.md), [11-A](../11a-measurement-baseline.md), [11-B](../11b-test-case-catalog.md).

## 2. 참고 발표자료의 적용

사용자가 제공한 NEO-Operation pack에는 52장 사진과 Word/Excel/manifest가 있다. 이번 재검토에서는 원본 사진 p9·15·19·32·40·42를 확인했다. 모든 사진의 세부 숫자를 검증한 것으로 주장하지 않는다.

다음 발표 구성을 VIA에 적용한다.

- 본문: 제품 문제 → 요구와 대표 지표 → 구조 후보 → 같은 기준의 비교 → 선택 → 남은 약점과 보완.
- 검증: 처음 제시한 지표로 선택안과 보완안의 결과를 다시 제시.
- 부록: 측정 환경·출처·계산식·원자료·실측/추정의 한계를 보존.

참고자료의 개발 MM, 서버 처리시간, 보안 control 점수나 선정 기준은 다른 과제의 값이다. VIA의 승인된 7개 ASR, 목표, 평가 척도를 그 숫자로 대체하지 않는다. 원본 이미지·Word·Excel을 공개 GitHub 저장소에 복사하지 않는다.

## 3. Qwen 공개 자료 재조회에서 발견한 점

출처: [Qualcomm Qwen3-8B model card](https://huggingface.co/qualcomm/Qwen3-8B), Performance Summary의 `GENIEX_QAIRT / w4a16 / Snapdragon X Elite / context 4096` 행.

| 항목 | 기존 근거 문서 기록 | 이번 웹 조회 표시값 |
| --- | ---: | ---: |
| 첫 token 이후 생성률 | 12.949668 token/s | 13.586086 token/s |
| 짧은 prompt의 TTFT | 0.1555 s | 0.156323 s |
| 전체 context TTFT | 4.976 s | 5.002336 s |

기존 값과 이번 조회값이 일치하지 않는다. 원문 업데이트와 웹 캐시 중 어느 요인인지 확인하지 못했으므로 **둘을 같은 실측 행으로 합치거나 평균하지 않는다.** 이번 기록은 별도 조회 결과이며 기존 계산 profile을 조용히 덮어쓰지 않는다. 최종 profile을 동결할 때 원문 버전 또는 날짜가 확인되는 원자료 사본을 함께 고정해야 한다.

공식 설명에 따르면 생성률은 짧은 prompt와 긴 thinking 응답에서 측정했으며 긴 context에서 느려질 수 있다. 따라서 한국어 비사고 structured JSON의 실측률이라고 부르지 않는다. TTFT 하한은 최대 128 token prompt, 상한은 4096 token 입력에 대응하는 길이 범위이지 신뢰구간 또는 p95가 아니다.

### 같은 검산 입력으로 식만 확인

기존 문서와 같은 `Nin=1200, Nout=60`을 임의의 함수 검산 입력으로 사용한다.

```text
T_hat = ceil(Nin/128) × TTFT_short + (Nout−1)/generation_rate

기존 기록값 사용: 6.1111 s
이번 조회값 사용: 5.9059 s
```

두 값 모두 **실제 prompt token을 세어 얻은 VIA 성능값이 아니다.** 길이에 따른 TTFT를 구간 비례로 근사한 모델 소계이며, 부하·모델 로드·Context·IPC·출력 비용과 실제 분포는 포함하지 않는다. 128/TTFT를 계산한 입력률도 순수 prefill 실측값이 아니다.

이전 대화의 `500/30 token/s`는 확인 가능한 동일 조건의 1차 근거가 부족하므로 계속 기준값에서 제외한다.

## 4. 아직 완료되지 않은 검증

공식 Qwen tokenizer 메타데이터와 chat template는 확인했다.

- [tokenizer 메타데이터](https://huggingface.co/Qwen/Qwen3-8B/blob/main/tokenizer.json)
- [공식 chat template](https://huggingface.co/Qwen/Qwen3-8B/resolve/main/tokenizer_config.json)
- tokenizer SHA256: `aeb13307a71acd8fe81861d94ad54ab689df773318809eed3cbe794b4492dae4`

그러나 현재 실행 환경의 외부 다운로드 제한으로 tokenizer 원본을 가져오지 못했다. 따라서 정확한 입력 token 수를 글자 수 비례 추정으로 바꿔 채우지 않는다. 모델 가중치 실행·S2S 음성 계측·VIA 후보 시험도 수행하지 않았다.

검토 가능한 것은 측정 계약, prompt 원문, 시험 입력/정답 명세와 계산 보조 코드이다. 실제 수치가 필요한 원장에는 `NOT_RUN/null`을 유지한다. 문서 작성 완료와 실제 시험 완료를 구분한다.

## 5. 리뷰 순서

1. 11-A에서 ASR별 측정 경계와 미측정/추정 표시를 확인한다.
2. 11-B에서 화면 지칭·복합 요청·후속 업무·재시작·승인 시험의 입력과 정답을 확인한다.
3. Qwen 근거 원장에서 최종 profile 선택, tokenizer 산정 및 실측 보완 범위를 확인한다.
4. 그 다음에만 11-C의 목표·점수·집계·반복을 동결한다. 후보 결과를 보고 유리한 기준으로 수정하지 않는다.
