# Previous VIA Experiment Tools — 2026-10-11 Archive

> **HISTORICAL ONLY.** 이번 DP41/44/45/42와 ASR-QA-01~06 측정 코드가 아니다.

이전 실험의 후보, runner, fixture, 정답과 결과는 이미 세대별 archive에 보존되어 있다.
이번에는 `scripts/architecture/`에 남아 있던 모델 연동, 음성 qualification과 이전 QA
검사 도구를 옮겨 새 측정 코드의 활성 경로와 분리했다. 파일 내용은 변경하지 않았다.

| 보존 도구 | 이전 용도와 제한 |
| --- | --- |
| [start_local_semantic_model.sh](./start_local_semantic_model.sh) | 이전 로컬 Qwen3-8B 서버 시작. 이번 클라우드 의미 LLM 설정이 아님 |
| [smoke_alibaba_s2s.py](./smoke_alibaba_s2s.py) | 이전 Alibaba S2S WAV 왕복 smoke. 새 의미 LLM이나 GPT Realtime adapter가 아님 |
| [qualify_audio_loopback.sh](./qualify_audio_loopback.sh) | 이전 CoreAudio probe 빌드/실행 wrapper |
| [index_audio_artifacts.py](./index_audio_artifacts.py) | 이전 campaign WAV 원본 digest 생성 |
| [check_evaluation_package.py](./check_evaluation_package.py) | 이전 상세 QA 19행 결과 검사. ASR-QA-01~06 결과 검사가 아님 |
| [check_via_dp03_contract.py](./check_via_dp03_contract.py) | 이전 VIA-DP-03 fixture/oracle, timing, change/fault 계약의 정적 검사 |

[manifest.json](./manifest.json)에 이전 경로, 보존 경로, bytes, SHA-256, 파일 mode와
정리 시작 시 Git HEAD를 기록했다. 활성 경로의 README 다섯 개도 개정 전 원문으로
`navigation-snapshots/`에 보존했다. manifest의 hash는 보존본 내용과 직접 대조할 수 있다.

## 이전 실험 전체의 위치

| 세대 | 후보 코드 | runner, fixture와 정답 | 결과와 보조 도구 |
| --- | --- | --- | --- |
| 2026-09-27 pre-Core-ASR | [Rust/Python 후보](../../../prototypes/archive/pre-core-asr-reference-20260927/README.md) | [benchmark](../../../benchmark/archive/pre-core-asr-reference-20260927/README.md) | [결과](../../../results/architecture-evaluation/archive/pre-core-asr-reference-20260927/README.md) |
| vNext/DP-00 | [Rust 후보와 pilot runner](../../../prototypes/archive/vnext-dp00/dp00/README.md) | [benchmark](../../../benchmark/archive/vnext-dp00/) | 해당 archive 안의 qualification과 calibration 자료 |
| W12-G1 | 해당 세대의 보존 자산 | [benchmark](../../../benchmark/archive/w12-g1/README.md) | [보조 script](../w12-g1/), [결과](../../../results/gate2/archive/w12-g1/) |

기존 세대별 archive의 source와 이전 결과는 이동하거나 수정하지 않았다. 상위 script archive 안내에 이 보관 위치의 링크를 추가했다. 모델 가중치, credential,
Keychain 내용, 가상환경과 재생성 가능한 build/bytecode cache는 이 archive에 복사하지 않았다.

## 실행과 재사용 경계

이 보존 작업에서 모델 호출, 음성 실행, 성능 측정이나 candidate test는 수행하지 않았다.
기존 기본값, QA ID, timing 기준, oracle과 점수 경계는 새 실험으로 자동 승계하지 않는다.

원본 보존을 위해 내부 상대 경로도 그대로 두었다. 특히 loopback wrapper의 옛
`benchmark/architecture/audio_loopback_probe.m`은 지금 활성 경로에 없으며, 해당 source는
[이전 benchmark archive](../../../benchmark/archive/pre-core-asr-reference-20260927/architecture/audio_loopback_probe.m)에 있다.
DP-03 검사기의 repository-root 계산도 이전 배치를 전제로 한다. 이 archive는 그대로
실행할 수 있는 새 측정 환경을 제공하지 않는다.

재사용할 기능은 새 DP/QA 계약에 맞는지 검토한 뒤 활성 경로에 명시적으로 도입한다.
현재 문서 검산과 도식/발표 생성 코드는 유지한다. `qa_common_workload.py`,
`qa_poc_reference.py`와 `dp41_output_contract.py`는 문서/예시의 정적 검사이며 후보 실측이 아니다.
