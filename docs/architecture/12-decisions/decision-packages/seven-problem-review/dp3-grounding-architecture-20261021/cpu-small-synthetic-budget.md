# DP3 B: 소형 모델의 CPU 전용 배치와 가상 설계 예산

> **후속 사용자 지시로 이번 검토의 512 MB 상한 적용을 유보했다.** 현재 구성은 [CPU runtime·상주·회수 설계](cpu-runtime-architecture.md)를 따른다. 아래 496 MB·시간·정확도는 당시 손익 설명용 합성 가정으로 보존하며 현재 구성의 실측·예측값으로 사용하지 않는다.

2026-10-21 / 사용자 요청: GPU 없이 소형 OmniParser·SAM2, RAM 한도 내 설계와 가정 수치

**상태: DESIGN_PROPOSAL / SYNTHETIC_ASSUMPTIONS / NOT_MEASURED.** 아래 시간·RAM 예산·정확도 예시는 손익을 검토하기 위해 설정한 가상 값이다. 실제 제품 성능 예측, 실행 로그, 달성 결과나 QA 점수가 아니다. 공개 모델의 실제 이름·크기와 가상 실행 수치를 구분한다. 이전 [local 생산기 판본](local-producer.md)의 구조와 실패 계약을 유지하면서 모델 구성·runtime·자원 정책을 줄인다. 이전 판본이나 그림의 caption 포함 구성을 이 CPU 구성으로 오인하지 않는다.

## 1. 실제 존재하는 소형 구성

| 기능 | 이번 구성 | 사실과 설계 전제 |
| --- | --- | --- |
| UI 후보 검출 | OmniParser `icon_detect/model.pt` | 공식 배포 파일 약 40.6 MB. 이전 권고의 MIT `icon_detect_v3`와 다르며 AGPL-3.0이다. 파일 크기에서 parameter 수나 RAM을 역산하지 않는다. |
| UI 아이콘 설명 | Florence caption 모델 제외 | 공식 하위 함수에 `use_local_semantics=False` 경로가 있다. 후보 경계와 OCR은 남기고 아이콘의 의미는 관련 원본 crop을 받은 공통 cloud 의미 LLM이 해석한다. |
| OCR | PP-OCRv5 mobile 검출과 `korean_PP-OCRv5_mobile_rec` 경로를 adapter로 구성하는 후보 | 한국어·영어·숫자 지원용 소형 OCR. native 텍스트가 충분하면 생략한다. 전체 OCR/ML 프레임워크를 모두 상주시킨다는 뜻이 아니다. |
| 필요한 경계/시간 대응 | 공식 `sam2.1_hiera_tiny` | 38.9M parameter. 이전 local 문서도 Tiny를 후보로 제안했으므로 더 작은 공식 SAM2를 새로 발견한 것은 아니다. |
| CPU 실행형 | native CPU inference worker, ONNX Runtime CPU provider 계열로 변환하는 설계 | GPU/NPU offload 없이 실행. INT8을 지원하는 연산은 양자화하고 필요한 부분은 FP32 유지. export·지원 연산·양자화 품질·RAM은 구현 검증 전이다. |

이는 공식 제품명 “OmniParser-Tiny”가 아니라 **caption을 제거한 소형 OmniParser 구성**이다. 기본 wrapper는 caption과 여러 OCR 의존성을 초기화할 수 있으므로 옵션 하나를 끄기만 한 프로세스를 최종 경량 구현으로 간주하지 않는다. 필요한 하위 경로만 구성하는 VIA adapter가 필요하다.

CPU 실행은 GPU RAM 대신 시스템 RAM을 쓰므로 자동으로 메모리를 줄여 주지 않는다. 감소 요인은 작은 모델 선택, caption 제거, 양자화, 한 모델씩 실행, 실제 자원 회수다. SAM2 Tiny의 FP32 weight-only 산술값은 약 155.6 MB이며 이상적인 전체 INT8 weight-only는 약 38.9 MB다. 어느 값도 runtime·활성화·추적 상태를 포함한 실행 RAM은 아니다.

공식 Tiny 입력은 1024이며 기본 `num_maskmem`은 7이다. 작은 crop을 주더라도 내부에서 1024로 확대되면 activation RAM은 줄지 않는다. 이번 예산은 crop 크기를 줄였다는 이유로 activation이 1/4이 된다고 계산하지 않는다. 실제 encoder 해상도 변경은 별도 품질 변경이다.

근거: [OmniParser detector 파일](https://huggingface.co/microsoft/OmniParser-v2.0/tree/main/icon_detect), [caption 생략 경로](https://github.com/microsoft/OmniParser/blob/master/util/utils.py), [모델별 라이선스](https://huggingface.co/microsoft/OmniParser-v2.0#license), [SAM2 모델 표](https://github.com/facebookresearch/sam2#model-description), [Tiny 설정](https://github.com/facebookresearch/sam2/blob/main/sam2/configs/sam2.1/sam2.1_hiera_t.yaml), [모바일 OCR](https://www.paddleocr.ai/v3.3.1/en/version3.x/module_usage/text_recognition.html), [CPU 양자화 수단](https://onnxruntime.ai/docs/performance/model-optimizations/quantization.html)

## 2. 실행과 최종 LLM 입력

공통 화면 필요 판별 이후에만 실행한다. A/B의 ASR·timestamp와 cloud S2S·최종 의미 모델은 같다. 전사가 완벽한 경우도 비교한다.

1. native 자료가 충분하면 모델 없이 후보를 등록한다.
2. 그렇지 않으면 detector로 UI 후보 경계를 얻는다. 필요한 OCR은 다음 stage로 실행한다.
3. detector/OCR stage의 가중치·큰 tensor를 회수하고 후보·OCR·원본 참조만 남긴다.
4. 정밀 mask나 시간별 대응이 필요할 때만 SAM2 Tiny를 load하여 실행한다. 동시 frame/object 수를 제한하되 나머지 대상은 추가 batch로 처리한다. 누락시켜 성공으로 세지 않는다.
5. 후보 ID·bbox/필요 polygon·시각별 연결·공간 관계·불확실성과 원본 overview/crop을 최종 LLM에 전달한다. raw mask tensor 전체는 전송하지 않는다.
6. binder의 추가 요청에 따라 필요한 stage를 다시 load한다. 그 비용도 포함한다. 입력/후보 판본과 권한 확인 후 최종 의미를 채택한다.

한 worker의 순차 실행은 이번 CPU 메모리 제약을 위한 명시적 설계 선택이다. 앞 단계와 뒤 단계의 모델·runtime이 실제로 겹쳐 상주하면 그 overlap을 RAM에 포함한다. session 참조를 버렸다는 사실만으로 allocator가 RAM을 반환했다고 간주하지 않는다. 공통 ASR은 시각 stage 중에도 계속 동작하며 CPU 스레드 예산을 나누는 비용을 포함한다.

caption 제거 후에도 B의 차이는 남는다. 최종 LLM은 이름·의도를 해석하지만, 경계·분할·추적 후보는 local producer가 생산한다. 반면 local caption 설명이 없으므로 원본 crop 확인 비용이 늘 수 있고, 이전 B의 cloud 비용 절감 가설은 약해질 수 있다.

## 3. RAM: 512 MB 이내를 목표로 한 가상 배분

**아래는 측정 RAM이 아니라 구현이 맞춰야 할 합성 예산이다.** 모든 단위는 decimal MB다. A/B 공통 메모리 자체도 실제 측정한 값이 아니다.

| 항목 | 가상 예산 |
| --- | ---: |
| 공통 Core·UI·공유 코드 | 100 MB |
| 공통 ASR·timestamp 가중치/실행 상태 | 116 MB |
| 공통 화면 decode·전송·역사자료의 RAM 부분 | 64 MB |
| 공통 음성 buffer·기타 상태 | 20 MB |
| 공통 소계 | **300 MB** |
| B 후보·공간 관계 등 지속 metadata | 16 MB |
| 동시에 실행하는 한 시각 stage 전체 | **180 MB 목표** |
| B 전체 예산 | **496 MB** |
| 512 MB까지 남는 여유 | **16 MB** |

180 MB에는 runtime, 양자화/비양자화 가중치, activation, decode/I/O, 활성 mask·추적 상태와 load 중 임시 복사본을 전부 넣어야 한다. 해당 구현의 stage별 최고값을 각각 180 MB 이하로 제한하는 목표이지, SAM2를 실행해 180 MB가 나왔다는 뜻이 아니다. 기본 1024 SAM2와 표준 Python/PyTorch를 그대로 사용하면 이 수치를 달성한다고 주장하지 않는다.

```text
실제 QA06 = max_t[공통 VIA RAM(t) + B metadata RAM(t) + 활성 stage RAM(t)]
합성 예산 = 300 + 16 + 180 = 496 MB
```

같은 물리 페이지는 한 번 센다. 개별 process peak를 합하는 평가 방식으로 바꾸지 않는다. 모델을 순차로 돌려 peak를 줄이면 load·회수·추가 batch 시간이 QA01/03에 반영된다. 16 MB의 여유는 작으며 실제 공통 peak 또는 stage peak가 예산을 넘으면 목표를 만족한 것으로 표시할 수 없다. 기존 QA06 512 MB 목표나 점수 구간은 변경하지 않는다.

## 4. 시간: 새 복합 요청 한 건의 가상 손익

**설정한 수치이며 CPU benchmark가 아니다.** 화면 생산이 필요한 새 요청 한 건에 적용한다. 모델 최초 load와 생산을 4.0초로 놓고 비교한 계산이다. 기존 전체 QA 모집단의 평균으로 사용하지 않는다. 실제 참조 frame 수·경계 복잡도·추적 길이가 늘면 추가 시간도 늘어야 한다.

| 단계 | A 가정 | 소형 CPU B 가정 |
| --- | ---: | ---: |
| 공통 gate·근거 준비 | 0.4 s | 0.4 s |
| local 모델 load·후보 생산 | 0 | 4.0 s |
| cloud 전송·의미 해석·첫 UI 채택/표시 | 2.0 s | 1.6 s |
| 첫 지칭 포함 응답까지 | **2.4 s** | **6.0 s** |
| 이후 VIA 완료 처리·필요 음성 준비 | 1.0 s | 1.0 s |
| VIA 처리 완료까지 | **3.4 s** | **7.0 s** |

2.0초 대 1.6초는 B의 구조화된 입력이 해석을 일부 줄인다는 가정이다. 값의 근거가 실제 모델 실행이라는 뜻이 아니다. 이 예시에서 B의 추가 4.0초 중 0.4초를 상쇄하여 최종 차이는 3.6초다. native 자료만으로 끝나는 요청에는 이 생산 시간을 가산하지 않는다. 원본 추가조회·확장·retry가 생기면 비용을 더 포함한다.

시작은 사용자 발화 종료이며 공통 ASR의 발화 중 완료분을 다시 더하지 않는다. 일반 안내는 QA01 끝점으로 세지 않는다. 실제 QA03의 외부 Agent 업무/사용자 대기/후속 발화/순수 재생 차례 제외와 VIA 실제 처리 포함 경계는 유지한다.

## 5. 정확도: B의 이익이 나타났다고 가정한 별도 예시

**아래는 결과를 만들어 본 가상 집계이며 성능 예측이 아니다.** 가상 입력 100개에 지칭 표현이 150개 있다고 둔다. QA02는 지칭별 첫 채택, QA07은 입력별 전체 필수 의미의 첫 채택으로 계산한다.

| 가상 집계 | A | 소형 CPU B |
| --- | ---: | ---: |
| QA02 정답 지칭 | 132/150 = **88%** | 141/150 = **94%** |
| QA07 정답 요청 | 80/100 = **80%** | 86/100 = **86%** |

이 설정은 지칭 9개 개선이 요청 6개의 전체 성공으로 이어진 경우다. 같은 대상 개선을 독립된 두 정확도 효과로 합산하지 않는다. 소형 모델/양자화의 시각 오류가 증가해 우열이 반대가 되는 경우도 가능하다. 큰 모델의 정확도를 소형 모델에 계승한 것이 아니며 이 가상 집계에 실제 관측 ID·원시 로그를 꾸며 붙이지 않는다. 기존 COMMON-28, 정답 자료, 목표나 score bands를 변경하지 않는다.

## 6. QA와 라이선스 판단

| QA | 이번 구성에서 달라지는 예상 |
| --- | --- |
| QA01·03 | CPU 생산과 순차 load로 B의 지연 부담이 커질 수 있다. 위 한 요청은 +3.6초로 설정한 예시다. |
| QA02·새 QA07 | 공간적 지칭 개선 경로는 남지만 소형/양자화의 성능 유지가 새 검증 과제다. 위 정확도는 유리한 결과를 가정한 예시다. |
| QA04 | native 변환·양자화 adapter·회수 계약이 추가되므로 B의 유지 부담이 늘 수 있다. 실제 Rust 변경 모듈 수를 측정하지 않았다. |
| QA05 | local 시각 추론의 cloud 청구는 없지만 caption을 뺀 만큼 cloud crop 해석 비용이 늘 수 있다. 동일 COMMON-28 전체 청구액의 감소를 자동으로 가정하지 않는다. |
| QA06 | 합성 예산 A 공통300 MB / B496 MB. 목표 달성 여부는 미확인이고 A보다 B의 메모리 부담은 크다. |

소형 `icon_detect` 선택은 이전 MIT detector 권고에서 **AGPL-3.0 가중치로 돌아가는 변경**이다. 상용 자체가 금지되는 것은 아니지만 제품 배포·결합 방식에 따른 의무를 유지한다. MIT `icon_detect_v3`를 선택하면 라이선스 조건은 달라지나 이 작은 detector의 RAM 예산을 그대로 적용하지 않는다. SAM2.1은 Apache-2.0이며 양자화/변환으로 원래 라이선스 의무가 사라지지 않는다. 실제 작은 MIT 대체 검출기의 채택·학습은 이번 작업에서 수행하지 않았다.

판정: **소형 CPU B는 구현 후보로 구체화할 수 있다.** 핵심 구성은 caption 없는 UI 생산, SAM2 Tiny의 조건부 실행, CPU 양자화와 순차 메모리 회수다. “모델을 CPU로 옮겼으므로 512 MB 달성”이라는 판정은 아니다. 이 문서는 GPU 사용 없는 설계 방향과 가상 예산을 기록하며 실제 구현·모델 실행·계측·선정·외부 발행은 포함하지 않는다.
