# QA-02 근거 원장 — Qwen3-Omni-30B-A3B-Instruct S2S profile

> 작성일: 2026-09-21
> 상태: **Approved measurement planning evidence**. VIA 실제 측정값 또는 target/score가 아니다.
> 목적: VIA Voice Runtime이 사용하는 S2S Model Runtime의 reference latency profile을 고정한다.

## 1. Reference model

- Model: **Qwen/Qwen3-Omni-30B-A3B-Instruct**
- Architecture: Thinker–Talker MoE
  - Audio Encoder: 약 650M
  - Thinker: 30B total / 3B active
  - Talker: 3B total / 0.3B active
  - MTP: 80M
  - Code2Wav: 200M
- Speech input: Korean 포함 19개 언어
- Speech output: Korean 포함 10개 언어
- Official model repository size: 약 **70.5 GB**
- Official BF16 minimum-memory guidance for the Instruct model is far above ordinary single consumer-GPU capacity (예: 15s video 기준 약 78.85 GB).

Primary sources:
- Technical report: https://arxiv.org/abs/2509.17765
- Official model: https://huggingface.co/Qwen/Qwen3-Omni-30B-A3B-Instruct
- Official deployment / memory guidance: https://github.com/QwenLM/Qwen3-Omni

따라서 VIA의 현재 reference에서는 Qwen3-Omni를 **일반 가정용 GPU에 올라가는 local fixture라고 가정하지 않는다.** Model Runtime은 VIA dependency이므로 remote/high-memory GPU endpoint일 수 있으며, 그 경우 network/transport latency도 QA-02에 포함한다.

## 2. Official streaming latency profile

Qwen3-Omni Technical Report Table 2의 audio 기준:

| Concurrency | End-to-end first-packet | Thinker TPS | Talker TPS | Generation RTF |
| ---: | ---: | ---: | ---: | ---: |
| **1** | **234 ms** | **75 tok/s** | **140 tok/s** | **0.47** |
| 4 | 728 ms | 63 tok/s | 125 tok/s | 0.56 |
| 6 | 1172 ms | 53 tok/s | 110 tok/s | 0.66 |

Concurrency=1의 first-packet decomposition:

    Thinker-Talker tail-packet preprocessing = 72 ms
    Thinker TTFT                              = 88 ms
    Talker TTFT                               = 57 ms
    MTP                                       = 14 ms
    Codec decoder                             = 3 ms
    ------------------------------------------------
    Theoretical audio first packet           = 234 ms

Technical Report는 이를 **theoretical first-packet latency under typical computational resources**라고 설명하며, vLLM + torch.compile + CUDA Graph optimizations를 사용했다고 명시한다. 세부 GPU SKU는 해당 table에서 공개하지 않는다.

따라서 234 ms를 Windows PC 또는 VIA의 measured latency로 부르지 않는다.

## 3. Implementation corroboration

vLLM-Omni는 Qwen3-Omni용 serving benchmark에서 TTFT, TPOT/ITL, E2EL, **audio_ttfp**, **audio_rtf**를 직접 지원한다.

공식 vLLM-Omni regression test의 H100 multi-GPU baseline에서도 Qwen3-Omni-30B-A3B-Instruct의 audio TTFP/RTF를 측정한다. 이는 234 ms와 동일 workload/hardware는 아니지만 실제 serving implementation에서 first-audio-packet 및 RTF를 독립 metric으로 계측할 수 있음을 보여주는 corroborating evidence다.

Sources:
- https://github.com/vllm-project/vllm-omni/blob/main/docs/cli/bench/serve.md
- https://github.com/vllm-project/vllm-omni/blob/main/tests/dfx/perf/tests/test_qwen3_omni_multi_replicas.json

## 4. VIA planning use

S2S Direct Response의 QA-02 model subtotal은 token-throughput 식으로 Qwen3-8B와 합치지 않는다.

    T_s2s_reference_first_audio_hat = 234 ms

위 값은 **S2S Model Runtime 내부의 concurrency=1 theoretical first-packet reference**다.

VIA 사용자의 실제 direct-response estimate는:

    T_user_s2s_hat
    = Voice input-end / turn-end detection overhead
    + VIA Voice Runtime / orchestration overhead
    + network/transport (remote일 경우)
    + S2S Runtime first-packet reference
    + VIA audio delivery/playback start overhead

Qwen3-Omni의 streaming audio input은 turn이 끝나기 전에도 processing될 수 있으므로 실제 시스템에서 user input-end를 어디로 정의하는지는 VIA trace와 함께 보존한다. QA-02 비교에서는 모든 후보에 같은 Voice input fixture와 turn-boundary rule을 사용한다.

## 5. Why concurrency=1

VIA는 한 사용자의 foreground interaction latency를 대표 측정한다. 따라서 primary planning profile은 **concurrency=1**을 사용한다.

4/6 concurrency 값은 background jobs나 shared remote service contention을 보는 secondary evidence로 유지하고 대표 QA-02 값에 직접 섞지 않는다.

## 6. Evidence level and limitation

| 항목 | Evidence level |
| --- | --- |
| 234 ms / 75 TPS / 140 TPS / RTF 0.47 | `OFFICIAL_THEORETICAL_REFERENCE` |
| vLLM-Omni runtime benchmark | `IMPLEMENTATION_CORROBORATION` |
| VIA + network + Voice Runtime latency | `NOT_RUN` |
| 실제 user-experienced p95 | `NOT_RUN` |

가장 중요한 한계는 공식 234 ms table의 상세 GPU SKU가 공개되지 않았다는 점이다. 따라서 target을 234 ms 자체로 정하지 않고 S2S model subtotal의 planning input으로만 사용한다.
