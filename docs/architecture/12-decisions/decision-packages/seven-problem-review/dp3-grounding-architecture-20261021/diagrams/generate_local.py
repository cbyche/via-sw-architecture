#!/usr/bin/env python3
"""Generate the local-producer variant without altering the cloud reference.

The shared SVG/draw.io primitives and validations come from generate.py.
No model execution, network access or extra dependencies are needed.
"""
from pathlib import Path
import argparse
import generate as base

ROOT = Path(__file__).resolve().parent

def replace(collection, id, **changes):
    item = next(item for item in collection if item["id"] == id)
    item.update(changes)

replace(base.labels, "title", text="로컬 화면 생산기: A 원본 직접 해석 vs B 대상 생산 후 지칭 결합")
replace(base.labels, "legend", text="검정: 공통   |   파랑: A   |   초록: B   |   실선 화살표: 요청·전달   |   점선 화살표: 결과 반환")
replace(base.nodes, "panel_b", name="B  로컬 화면 생산 후 클라우드 지칭 결합")
replace(base.nodes, "local_b", name="VIA 로컬 제어 Process")
replace(base.labels, "cloud_b", text="클라우드 모델 의존성", y=1295)
base.node("local_b_worker", "로컬 시각 Worker", 1930, 760, 410, 470, "group", base.BLACK)
replace(base.nodes, "b_models", name="OmniParser: UI·OCR·설명\nSAM2: 마스크·시간 추적", kind="component", y=910, h=90)
replace(base.labels, "b_perception_call_label", text="3 필요한 모델·구간 요청")
replace(base.labels, "b_perception_return_label", text="후보·선택적 mask/track", size=18)
base.label("b_worker_conditions", "native 근거가 충분하면 생략\nSAM2는 분할·추적 필요 시에만\n첫 사용 시 로드 / 활성 상태만 보존", 1950, 1095, 370, 18, base.GREEN)
base.label("b_worker_local_cost", "가중치·추론·추적 상태: 로컬 자원", 1950, 1200, 370, 18, base.GREEN)
replace(base.labels, "b_record_content", text="RAM: 후보·공간 관계·판본\nmask·track은 필요분만 보존\n원본·불확실성·누락을 연결")
replace(base.labels, "b_model_call_label", text="5 같은 전사·시각 + 후보·관계\n주변 화면·필요 crop·누락", size=18)
replace(base.labels, "common_note", text="발화 중: 공통 기록·ASR·단어 시각\n화면 필요 판별 후에만 아래 경로 실행")
replace(base.labels, "foot1", text="A/B의 S2S와 최종 의미 모델은 클라우드, ASR는 로컬로 공통이다. B의 OmniParser·SAM2 추론은 로컬 Worker에서 조건부 실행한다.")
replace(base.labels, "foot2", text="A는 원본 확대·재조회를 허용한다. B는 후보 누락 시 원본으로 생산 범위를 넓힌다. 로컬화는 정확도·속도 우위를 뜻하지 않는다.")

replace(base.labels, "a_start_label", text="1 시작")
replace(base.labels, "b_start_label", text="1 시작")
replace(base.labels, "a_read_request_label", text="2 원본 조회")
replace(base.labels, "b_read_request_label", text="2 원본 조회")
replace(base.labels, "a_model_call_label", text="3 근거로 직접 해석")
replace(base.labels, "a_adopt_label", text="4 GroundedRequest 제안")
replace(base.labels, "b_write_label", text="4 후보·동작 관계 생산·갱신")
replace(base.labels, "b_adopt_label", text="6 GroundedRequest 제안")


def svg_text():
    return base.svg_text().replace(
        "음성 화면 지칭: A 직접 해석과 B 대상 생산 후 결합",
        "음성 화면 지칭: A 직접 해석과 B 로컬 대상 생산 후 결합"
    ).replace(
        "B는 전문 모델로 대상 동작 기록을 생산한 다음 지칭을 결합한다.",
        "B는 로컬 OmniParser와 조건부 SAM2로 대상 동작 기록을 생산한 다음 클라우드에서 지칭을 결합한다."
    )


def drawio_text():
    return base.drawio_text().replace("dp3-grounding-comparison", "dp3-local-grounding-comparison").replace("A-B 구조 비교", "A-B 로컬 생산 구조 비교")


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    base.validate()
    outputs = {"local-comparison.svg": svg_text(), "local-comparison.drawio": drawio_text()}
    for name, content in outputs.items():
        dest = ROOT / name
        if args.check:
            assert dest.exists() and dest.read_text(encoding="utf-8") == content, f"Out of sync: {name}"
        else:
            dest.write_text(content, encoding="utf-8")
    print("Local comparison SVG/draw.io synchronized; XML, IDs, bounds, orthogonal paths and node crossings checked.")


if __name__ == "__main__":
    main()
