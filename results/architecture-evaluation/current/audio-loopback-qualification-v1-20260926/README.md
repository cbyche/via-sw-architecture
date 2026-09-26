# Audio Loopback Qualification v1

> 실행일: 2026-09-26
> 결과: **PASS**
> Evidence label: `MEASURED_REFERENCE_HARNESS`

BlackHole 2ch의 output과 input을 같은 native CoreAudio device clock에서 구동하고, VIA harness가 만든 known chirp를 다시 캡처했다. 이 qualification은 QA-01~QA-04의 실제 audio endpoint observer를 사용하기 위한 선행 검증이며 제품 latency 결과는 아니다.

## 결과

| 항목 | 관측값 | 통과 조건 |
| --- | ---: | ---: |
| Device | BlackHole 2ch (`BlackHole2ch_UID`) | 48 kHz, 2-channel duplex |
| Captured frames | 72,704 | ≥ 48,000 |
| Peak amplitude | 0.199990 | ≥ 0.1 |
| Best normalized correlation | 1.0 | ≥ 0.95 |
| Output-to-capture offset | 21.333 ms | 진단값 |

## 보존 산출물

- `emitted.wav`: harness가 BlackHole output으로 보낸 48 kHz stereo Float32 waveform
- `captured.wav`: BlackHole input에서 받은 원본 48 kHz stereo Float32 waveform
- `report.json`: 판정 조건, 관측값, 장치 UID와 artifact 이름

재현 명령:

```bash
scripts/architecture/qualify_audio_loopback.sh \
  --qualify \
  --device-uid BlackHole2ch_UID \
  --out-dir results/architecture-evaluation/current/audio-loopback-qualification-v1-20260926
```

이 결과는 audio loopback 경로가 정확히 연결되었음을 보인다. 첫 meaningful audible audio onset 정의, silence·earcon 제외, playback queue와 device onset 계측은 각 QA campaign의 frozen contract에서 별도로 적용한다.
