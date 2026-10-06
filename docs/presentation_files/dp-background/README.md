# VIA DP 배경 슬라이드 — 04-41~45

각 문서의 공통 사용자 상황, 설계 난점, 구조적 선택 질문을 담은 **1920×1080 / 16:9 / 문서별 1페이지**다. `presentation_example1`과 `presentation_example2`의 배경 설명·상황 도식·핵심 설계 과제 구성을 참고했다.

[5페이지 draw.io 편집 원본](./VIA-DP-background-41-45.drawio) / [전체 미리보기](./index.html)

| 문서 | 배경 주제 | 개별 draw.io | SVG | PNG |
| --- | --- | --- | --- | --- |
| [04-41](../../architecture/12-decisions/decision-packages/04-41-request-resolution-control.md) | 해석과 조회의 상호 의존, 의미 해결 제어 | [원본](../../architecture/12-decisions/decision-packages/diagrams/dp41-background.drawio) | [미리보기](../../architecture/12-decisions/decision-packages/diagrams/dp41-background.svg) | [삽입용](./dp41-background.png) |
| [04-42](../../architecture/12-decisions/decision-packages/04-42-lifecycle-ownership.md) | 대화·요청·질문·업무·외부 실행의 다른 수명 | [원본](../../architecture/12-decisions/decision-packages/diagrams/dp42-background.drawio) | [미리보기](../../architecture/12-decisions/decision-packages/diagrams/dp42-background.svg) | [삽입용](./dp42-background.png) |
| [04-43](../../architecture/12-decisions/decision-packages/04-43-request-interpretation.md) | F1~F6의 상호 의존과 의미 생산 책임 | [원본](../../architecture/12-decisions/decision-packages/diagrams/dp43-background.drawio) | [미리보기](../../architecture/12-decisions/decision-packages/diagrams/dp43-background.svg) | [삽입용](./dp43-background.png) |
| [04-44](../../architecture/12-decisions/decision-packages/04-44-continuous-interaction.md) | 지속 발화와 비동기 업무 사건의 실행 조직 | [원본](../../architecture/12-decisions/decision-packages/diagrams/dp44-background.drawio) | [미리보기](../../architecture/12-decisions/decision-packages/diagrams/dp44-background.svg) | [삽입용](./dp44-background.png) |
| [04-45](../../architecture/12-decisions/decision-packages/04-45-memory-and-context.md) | 흩어진 과거 근거의 조합과 공통 기억 계약 | [원본](../../architecture/12-decisions/decision-packages/diagrams/dp45-background.drawio) | [미리보기](../../architecture/12-decisions/decision-packages/diagrams/dp45-background.svg) | [삽입용](./dp45-background.png) |

draw.io에서 **File → Open from → Device**로 원본을 열면 텍스트, 도형, 연결선을 개별 편집할 수 있다. 통합 파일에는 다섯 페이지가 있으며, 개별 파일과 SVG는 각 Architecture 문서에 연결했다. PNG는 발표자료 삽입용이다.

이 슬라이드는 기존 비교의 대안 정의와 미선정·미측정 상태를 따른다. 배경 도식은 공통 문제의 설명이며 최종 구현의 호출 그래프나 대안 선택 결과가 아니다. 특히 42의 다른 논리 수명은 두 안의 공통 요구이고, 43은 A/C 표기를 유지한다.

원본 장면은 [generate_dp_background_slides.py](../../../scripts/architecture/generate_dp_background_slides.py)에 있다. 저장소 루트에서 다음 명령으로 개별 SVG/draw.io, 통합 draw.io 및 HTML을 재생성하거나 일치를 확인한다.

```sh
python3 scripts/architecture/generate_dp_background_slides.py
python3 scripts/architecture/generate_dp_background_slides.py --check
```

PNG는 같은 SVG를 1920×1080으로 렌더링한 결과다. 수정 후 해당 PNG도 함께 갱신한다.
